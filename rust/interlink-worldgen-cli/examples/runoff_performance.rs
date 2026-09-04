use interlink_worldgen::{
    build_icosphere, generate_coupled_climate, generate_crust_and_history,
    generate_drainage_topology, generate_initial_topography, generate_lithosphere,
    generate_runoff_discharge, generate_tectonics, inherit_boundary_interfaces,
    inherit_physical_state, ClimateRequest, DrainageRequest, GeologyRequest, LithosphereRequest,
    PlanetPhysicalParameters, RunoffRequest, TectonicsRequest, TopographyRequest,
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
    value.parse::<T>().map_err(|_| format!("invalid value for {name}: {value}"))
}

fn parse_options() -> Result<Options, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let seed = args
        .iter()
        .position(|arg| arg == "--seed")
        .and_then(|index| args.get(index + 1))
        .cloned()
        .unwrap_or_else(|| "ci-wg6b-runoff".to_owned());
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
    let boundaries = inherit_boundary_interfaces(
        &coarse,
        &fine,
        &tectonics,
        &geology,
        &inherited.plate_ids,
    )
    .map_err(|error| error.to_string())?;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let climate = generate_coupled_climate(
        &fine,
        &terrain,
        planet,
        &ClimateRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let drainage = generate_drainage_topology(
        &fine,
        &terrain,
        planet,
        &DrainageRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let request = RunoffRequest::new(options.seed.as_str());

    let mut durations_ms = Vec::with_capacity(options.runs as usize);
    let mut last = None;
    for _ in 0..options.runs {
        let started = Instant::now();
        let runoff = generate_runoff_discharge(
            &fine,
            &terrain,
            &climate,
            &drainage,
            planet,
            &request,
        )
        .map_err(|error| error.to_string())?;
        durations_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
        last = Some(runoff);
    }
    let runoff = last.expect("at least one runoff benchmark run");
    let mean_ms = durations_ms.iter().sum::<f64>() / durations_ms.len() as f64;
    let mut sorted = durations_ms.clone();
    sorted.sort_by(f64::total_cmp);
    let median_ms = sorted[sorted.len() / 2];

    println!("WG-6B runoff benchmark");
    println!(
        "seed={} coarse_level={} level={} plates={} samples={} runs={}",
        options.seed,
        options.coarse_level,
        options.level,
        options.plates,
        fine.metrics().sample_count,
        options.runs
    );
    println!(
        "runtime_ms mean={mean_ms:.3} median={median_ms:.3} samples={:?}",
        durations_ms
    );
    println!(
        "water_balance precip_mm={:.3} aet_mm={:.3} runoff_mm={:.3} runoff_fraction={:.6}",
        runoff.metrics.mean_land_precipitation_mm,
        runoff.metrics.mean_land_actual_evapotranspiration_mm,
        runoff.metrics.mean_land_runoff_mm,
        runoff.metrics.land_runoff_fraction,
    );
    println!(
        "discharge total_local_m3_s={:.6} terminal_m3_s={:.6} max_m3_s={:.6} relative_error={:.3e}",
        runoff.metrics.total_local_runoff_m3_s,
        runoff.metrics.terminal_discharge_m3_s,
        runoff.metrics.maximum_potential_discharge_m3_s,
        runoff.metrics.discharge_conservation_relative_error,
    );
    println!(
        "hash runoff={} climate={} drainage={} parameters={}",
        runoff.metrics.runoff_hash_hex(),
        runoff.metrics.climate_hash_hex(),
        runoff.metrics.drainage_hash_hex(),
        runoff.metrics.runoff_parameter_hash_hex(),
    );
    Ok(())
}
