use interlink_worldgen::{
    build_icosphere, generate_coupled_climate_with_diagnostics, generate_crust_and_history,
    generate_drainage_topology, generate_initial_topography, generate_lakes_closed_basins,
    generate_lithosphere, generate_runoff_discharge, generate_seasonal_hydrology,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, ClimateRequest,
    DrainageRequest, GeologyRequest, LakeRequest, LithosphereRequest, PlanetPhysicalParameters,
    RunoffRequest, SeasonalHydrologyRequest, TectonicsRequest, TopographyRequest,
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
        .unwrap_or_else(|| "ci-wg6d-seasonal".to_owned());
    let options = Options {
        seed,
        coarse_level: parse_value(&args, "--coarse-level", 4_u8)?,
        level: parse_value(&args, "--level", 6_u8)?,
        plates: parse_value(&args, "--plates", 16_u16)?,
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
    let request = SeasonalHydrologyRequest::new(options.seed.as_str());

    let mut durations_ms = Vec::with_capacity(options.runs as usize);
    let mut last = None;
    for _ in 0..options.runs {
        let started = Instant::now();
        let seasonal = generate_seasonal_hydrology(
            &fine,
            &terrain,
            &climate,
            &climate_diagnostics,
            &drainage,
            &runoff,
            &lakes,
            planet,
            &request,
        )
        .map_err(|error| error.to_string())?;
        durations_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
        last = Some(seasonal);
    }
    let seasonal = last.expect("at least one WG-6D benchmark run");
    let mean_ms = durations_ms.iter().sum::<f64>() / durations_ms.len() as f64;
    let mut sorted = durations_ms.clone();
    sorted.sort_by(f64::total_cmp);
    let median_ms = sorted[sorted.len() / 2];

    println!("WG-6D seasonal hydrology benchmark");
    println!(
        "seed={} coarse_level={} level={} plates={} samples={} phases={} runs={}",
        options.seed,
        options.coarse_level,
        options.level,
        options.plates,
        fine.metrics().sample_count,
        seasonal.metrics.orbital_phase_count,
        options.runs
    );
    println!(
        "runtime_ms mean={mean_ms:.3} median={median_ms:.3} samples={:?}",
        durations_ms
    );
    println!(
        "runoff annual_mean_local_m3_s={:.6} annual_target_error={:.3e} snowmelt_fraction={:.6}",
        seasonal.metrics.annual_mean_local_runoff_m3_s,
        seasonal.metrics.annual_local_runoff_closure_relative_error,
        seasonal.metrics.snowmelt_runoff_fraction,
    );
    println!(
        "potential terminal_mean_m3_s={:.6} max_phase_m3_s={:.6} routing_error={:.3e}",
        seasonal
            .metrics
            .annual_mean_terminal_potential_discharge_m3_s,
        seasonal.metrics.maximum_phase_potential_discharge_m3_s,
        seasonal
            .metrics
            .seasonal_routing_conservation_relative_error,
    );
    println!(
        "realized terminal_mean_m3_s={:.6} max_phase_m3_s={:.6} water_balance_error={:.3e}",
        seasonal
            .metrics
            .annual_mean_terminal_realized_discharge_m3_s,
        seasonal.metrics.maximum_phase_realized_discharge_m3_s,
        seasonal.metrics.seasonal_water_balance_relative_error,
    );
    println!(
        "lakes active={} spinup_years={} cycle_error={:.3e} max_level_range_m={:.6} precip_m3_s={:.6} evap_m3_s={:.6} terminal_storage_m3_s={:.6}",
        seasonal.metrics.active_lake_count,
        seasonal.metrics.lake_spinup_years,
        seasonal.metrics.final_lake_cycle_relative_change,
        seasonal.metrics.maximum_seasonal_lake_level_range_m,
        seasonal.metrics.annual_mean_lake_precipitation_m3_s,
        seasonal.metrics.annual_mean_lake_evaporation_m3_s,
        seasonal.metrics.annual_mean_unreleased_terminal_storage_m3_s,
    );
    println!(
        "flow dry={} intermittent={} perennial={}",
        seasonal.metrics.dry_flow_sample_count,
        seasonal.metrics.intermittent_flow_sample_count,
        seasonal.metrics.perennial_flow_sample_count,
    );
    println!(
        "lake_cycle surface_change_m={:.9}",
        seasonal.metrics.final_lake_surface_cycle_change_m,
    );
    println!(
        "hash seasonal={} lakes={} runoff={} drainage={} climate={} parameters={}",
        seasonal.metrics.seasonal_hydrology_hash_hex(),
        seasonal.metrics.lake_hash_hex(),
        seasonal.metrics.runoff_hash_hex(),
        seasonal.metrics.drainage_hash_hex(),
        seasonal.metrics.climate_hash_hex(),
        seasonal.metrics.seasonal_parameter_hash_hex(),
    );
    Ok(())
}
