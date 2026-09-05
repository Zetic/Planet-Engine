use interlink_worldgen::{
    build_icosphere, generate_bounded_terrain_evolution, generate_coupled_climate_with_diagnostics,
    generate_crust_and_history, generate_drainage_topology, generate_fluvial_erosion_sediment,
    generate_initial_topography, generate_lake_sediment_infill, generate_lakes_closed_basins,
    generate_lithosphere, generate_post_erosion_hydrology, generate_runoff_discharge,
    generate_seasonal_hydrology, generate_tectonics, inherit_boundary_interfaces,
    inherit_physical_state, ClimateRequest, DrainageRequest, FluvialErosionRequest, GeologyRequest,
    LakeRequest, LakeSedimentInfillRequest, LithosphereRequest, PlanetPhysicalParameters,
    PostErosionHydrologyRequest, RunoffRequest, SeasonalHydrologyRequest, TectonicsRequest,
    TerrainEvolutionRequest, TopographyRequest,
};
use std::env;
use std::time::Instant;

#[derive(Clone, Debug)]
struct Options {
    seed: String,
    coarse_level: u8,
    level: u8,
    plates: u16,
    runs: u8,
}

fn parse_value<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> Result<T, String> {
    let Some(index) = args.iter().position(|arg| arg == name) else {
        return Ok(default);
    };
    let Some(value) = args.get(index + 1) else {
        return Err(format!("missing value after {name}"));
    };
    value
        .parse::<T>()
        .map_err(|_| format!("invalid value for {name}: {value}"))
}

fn parse_options() -> Result<Options, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let seed = args
        .iter()
        .position(|arg| arg == "--seed")
        .and_then(|index| args.get(index + 1))
        .cloned()
        .unwrap_or_else(|| "ci-wg7d-lake-infill".to_owned());
    let options = Options {
        seed,
        coarse_level: parse_value(&args, "--coarse-level", 3_u8)?,
        level: parse_value(&args, "--level", 4_u8)?,
        plates: parse_value(&args, "--plates", 12_u16)?,
        runs: parse_value(&args, "--runs", 1_u8)?,
    };
    if options.coarse_level > options.level {
        return Err("--coarse-level cannot exceed --level".to_owned());
    }
    if options.runs == 0 {
        return Err("--runs must be at least 1".to_owned());
    }
    Ok(options)
}

fn main() -> Result<(), String> {
    let options = parse_options()?;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(options.coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(options.level).map_err(|error| error.to_string())?;
    let tectonics = generate_tectonics(
        &coarse,
        &TectonicsRequest::new(options.seed.as_str(), options.plates),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let geology = generate_crust_and_history(
        &coarse,
        &tectonics,
        &GeologyRequest::new(options.seed.as_str()),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let lithosphere = generate_lithosphere(
        &coarse,
        &tectonics,
        &geology,
        &LithosphereRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let inherited = inherit_physical_state(
        &fine,
        options.coarse_level,
        &tectonics,
        &geology,
        &lithosphere,
        planet,
    )
    .map_err(|error| error.to_string())?;
    let boundaries =
        inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids)
            .map_err(|error| error.to_string())?;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let climate_request = ClimateRequest::new(options.seed.as_str());
    let mut progress = |_completed_years: u8, _maximum_years: u8| {};
    let (climate, climate_diagnostics) = generate_coupled_climate_with_diagnostics(
        &fine,
        &terrain,
        planet,
        &climate_request,
        &mut progress,
    )
    .map_err(|error| error.to_string())?;
    let drainage = generate_drainage_topology(
        &fine,
        &terrain,
        planet,
        &DrainageRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let runoff = generate_runoff_discharge(
        &fine,
        &terrain,
        &climate,
        &drainage,
        planet,
        &RunoffRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let lakes = generate_lakes_closed_basins(
        &fine,
        &terrain,
        &climate,
        &drainage,
        &runoff,
        planet,
        &LakeRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let seasonal = generate_seasonal_hydrology(
        &fine,
        &terrain,
        &climate,
        &climate_diagnostics,
        &drainage,
        &runoff,
        &lakes,
        planet,
        &SeasonalHydrologyRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let erosion = generate_fluvial_erosion_sediment(
        &fine,
        &inherited,
        &terrain,
        &drainage,
        &lakes,
        &seasonal,
        planet,
        &FluvialErosionRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let evolution = generate_bounded_terrain_evolution(
        &fine,
        &terrain,
        &drainage,
        &runoff,
        &lakes,
        &erosion,
        planet,
        &TerrainEvolutionRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let reconciliation = generate_post_erosion_hydrology(
        &fine,
        &terrain,
        &climate,
        &climate_diagnostics,
        &drainage,
        &runoff,
        &lakes,
        &seasonal,
        &evolution,
        planet,
        &PostErosionHydrologyRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let request = LakeSedimentInfillRequest::new(options.seed.as_str());

    let mut durations_ms = Vec::with_capacity(options.runs as usize);
    let mut last = None;
    for _ in 0..options.runs {
        let started = Instant::now();
        let state = generate_lake_sediment_infill(
            &fine,
            &terrain,
            &climate,
            &climate_diagnostics,
            &drainage,
            &lakes,
            &erosion,
            &evolution,
            &reconciliation,
            planet,
            &request,
        )
        .map_err(|error| error.to_string())?;
        durations_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
        last = Some(state);
    }
    let state = last.expect("at least one WG-7D benchmark run");
    let mean_ms = durations_ms.iter().sum::<f64>() / durations_ms.len() as f64;
    let mut sorted = durations_ms.clone();
    sorted.sort_by(f64::total_cmp);
    let median_ms = sorted[sorted.len() / 2];

    println!("WG-7D lake sediment infill benchmark");
    println!(
        "seed={} coarse_level={} level={} plates={} samples={} runs={}",
        options.seed,
        options.coarse_level,
        options.level,
        options.plates,
        fine.metrics().sample_count,
        options.runs,
    );
    println!(
        "runtime_ms mean={mean_ms:.3} median={median_ms:.3} samples={:?}",
        durations_ms
    );
    println!(
        "infill horizon_years={:.3} historical_traps={} filled_depressions={} filled_samples={} capacity_limited={} max_fill_m={:.6}",
        state.metrics.geomorphic_duration_years,
        state.metrics.historical_lake_trap_count,
        state.metrics.filled_depression_count,
        state.metrics.filled_sample_count,
        state.metrics.capacity_limited_depression_count,
        state.metrics.maximum_fill_depth_m,
    );
    println!(
        "sediment delivered_kg_s={:.6} applied_equivalent_kg_s={:.6} unapplied_kg_s={:.6} volume_m3={:.6e} closure={:.3e}",
        state.metrics.total_historical_lake_delivery_kg_s,
        state.metrics.total_applied_lake_fill_equivalent_kg_s,
        state.metrics.total_unapplied_lake_sediment_kg_s,
        state.metrics.total_applied_lake_fill_volume_m3,
        state.metrics.sediment_conservation_relative_error,
    );
    println!(
        "lakes pre={} post={} drainage_depressions={} -> {}",
        state.metrics.pre_infill_lake_count,
        state.metrics.post_infill_lake_count,
        evolution.post_erosion_drainage.metrics.depression_count,
        state.post_infill_drainage.metrics.depression_count,
    );
    println!(
        "closure runoff={:.3e} lake={:.3e} seasonal_routing={:.3e} seasonal_water={:.3e}",
        state.metrics.post_infill_runoff_conservation_relative_error,
        state.metrics.post_infill_lake_water_balance_relative_error,
        state.metrics.post_infill_seasonal_routing_relative_error,
        state.metrics.post_infill_seasonal_water_balance_relative_error,
    );
    println!(
        "seasonal spinup={} surface_drift_m={:.9} max_range_m={:.6}",
        state.reconciled_seasonal.metrics.lake_spinup_years,
        state.reconciled_seasonal.metrics.final_lake_surface_cycle_change_m,
        state.reconciled_seasonal.metrics.maximum_seasonal_lake_level_range_m,
    );
    println!(
        "hash infill={} surface={} drainage={} runoff={:016x} lake={:016x} seasonal={:016x} parameters={}",
        state.metrics.lake_sediment_infill_hash_hex(),
        state.metrics.post_infill_surface_hash_hex(),
        state.metrics.post_infill_drainage_hash_hex(),
        state.metrics.post_infill_runoff_hash,
        state.metrics.post_infill_lake_hash,
        state.metrics.post_infill_seasonal_hash,
        state.metrics.infill_parameter_hash_hex(),
    );
    Ok(())
}
