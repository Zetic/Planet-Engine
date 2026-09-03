use interlink_worldgen::{
    build_icosphere, generate_coupled_climate_with_diagnostics, generate_crust_and_history,
    generate_initial_topography, generate_lithosphere, generate_tectonics,
    inherit_boundary_interfaces, inherit_physical_state, ClimateRequest, GeologyRequest,
    LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,
    CLIMATE_STAGE_ID, CLIMATE_STAGE_VERSION,
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
    phases: u8,
}

fn parse_value<T: std::str::FromStr>(
    args: &[String],
    name: &str,
    default: T,
) -> Result<T, String> {
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
        .unwrap_or_else(|| "ci-wg5-performance".to_owned());
    let options = Options {
        seed,
        coarse_level: parse_value(&args, "--coarse-level", 4_u8)?,
        level: parse_value(&args, "--level", 6_u8)?,
        plates: parse_value(&args, "--plates", 16_u16)?,
        runs: parse_value(&args, "--runs", 1_u8)?,
        phases: parse_value(&args, "--phases", 24_u8)?,
    };
    if options.coarse_level > options.level {
        return Err("--coarse-level cannot exceed --level".to_owned());
    }
    if options.runs == 0 {
        return Err("--runs must be at least 1".to_owned());
    }
    if !(4..=48).contains(&options.phases) {
        return Err("--phases must be from 4 through 48".to_owned());
    }
    Ok(options)
}

fn tropical_precipitation_centroid_excursion_deg(
    topology: &interlink_worldgen::GeodesicTopology,
    precipitation_phase_rate_mm_year: &[f32],
    phase_count: usize,
) -> (f64, f64, f64) {
    let sample_count = topology.metrics().sample_count as usize;
    debug_assert_eq!(
        precipitation_phase_rate_mm_year.len(),
        phase_count * sample_count
    );
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;

    for phase in 0..phase_count {
        let phase_offset = phase * sample_count;
        let mut weighted_latitude = 0.0;
        let mut precipitation_weight = 0.0;
        for sample in 0..sample_count {
            let latitude_deg = topology.positions()[sample][2]
                .clamp(-1.0, 1.0)
                .asin()
                .to_degrees();
            if latitude_deg.abs() > 30.0 {
                continue;
            }
            let precipitation =
                f64::from(precipitation_phase_rate_mm_year[phase_offset + sample]).max(0.0);
            if precipitation <= 0.0 {
                continue;
            }
            let weight = precipitation * topology.dual_area_steradians()[sample];
            weighted_latitude += latitude_deg * weight;
            precipitation_weight += weight;
        }
        if precipitation_weight > 0.0 {
            let centroid = weighted_latitude / precipitation_weight;
            minimum = minimum.min(centroid);
            maximum = maximum.max(centroid);
        }
    }

    if !minimum.is_finite() || !maximum.is_finite() {
        (0.0, 0.0, 0.0)
    } else {
        (minimum, maximum, maximum - minimum)
    }
}

fn main() -> Result<(), String> {
    let options = parse_options()?;
    let planet = PlanetPhysicalParameters::earthlike_reference();

    let setup_started = Instant::now();
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
    let setup_ms = setup_started.elapsed().as_secs_f64() * 1_000.0;

    let mut request = ClimateRequest::new(options.seed.as_str());
    request.parameters.orbital_phase_count = options.phases;
    let mut elapsed_ms = Vec::with_capacity(usize::from(options.runs));
    let mut final_summary = None;
    let mut final_centroid = (0.0, 0.0, 0.0);

    for run in 0..options.runs {
        let started = Instant::now();
        let mut progress = |_completed_years: u8, _maximum_years: u8| {};
        let (climate, diagnostics) = generate_coupled_climate_with_diagnostics(
            &fine,
            &terrain,
            planet,
            &request,
            &mut progress,
        )
        .map_err(|error| error.to_string())?;
        let duration_ms = started.elapsed().as_secs_f64() * 1_000.0;
        elapsed_ms.push(duration_ms);
        let centroid = tropical_precipitation_centroid_excursion_deg(
            &fine,
            &diagnostics.precipitation_phase_rate_mm_year,
            usize::from(climate.metrics.orbital_phase_count),
        );
        println!(
            "run={} climate_ms={:.3} samples={} phases={} spinup_years={} max_moisture_substeps={} mean_temperature_k={:.6} mean_precipitation_mm={:.6} p95_precipitation_mm={:.6} mean_wind_m_s={:.6} max_wind_m_s={:.6} mean_current_m_s={:.6} snow_fraction={:.6} sea_ice_fraction={:.6} thermal_rms_k={:.6} tropical_precip_centroid_min_deg={:.6} tropical_precip_centroid_max_deg={:.6} tropical_precip_centroid_excursion_deg={:.6} moisture_error={:.6e} climate_hash={}",
            run + 1,
            duration_ms,
            climate.metrics.sample_count,
            climate.metrics.orbital_phase_count,
            climate.metrics.spinup_years,
            climate.metrics.maximum_moisture_transport_substeps,
            climate.metrics.mean_temperature_k,
            climate.metrics.mean_annual_precipitation_mm,
            climate.metrics.p95_annual_precipitation_mm,
            climate.metrics.mean_wind_speed_m_s,
            climate.metrics.maximum_wind_speed_m_s,
            climate.metrics.mean_surface_current_m_s,
            climate.metrics.persistent_snow_area_fraction,
            climate.metrics.sea_ice_area_fraction,
            climate.metrics.final_temperature_rms_change_k,
            centroid.0,
            centroid.1,
            centroid.2,
            climate.metrics.moisture_budget_relative_error,
            climate.metrics.climate_hash_hex(),
        );
        final_centroid = centroid;
        final_summary = Some(climate.metrics);
    }

    elapsed_ms.sort_by(|a, b| a.total_cmp(b));
    let median_ms = elapsed_ms[elapsed_ms.len() / 2];
    let mean_ms = elapsed_ms.iter().sum::<f64>() / elapsed_ms.len() as f64;
    let summary = final_summary.expect("at least one benchmark run");
    println!(
        "wg5_performance stage={}@{} seed={} coarse_level={} fine_level={} plates={} phases={} setup_ms={:.3} runs={} mean_climate_ms={:.3} median_climate_ms={:.3} mean_temperature_k={:.6} mean_precipitation_mm={:.6} p95_precipitation_mm={:.6} mean_wind_m_s={:.6} max_wind_m_s={:.6} mean_current_m_s={:.6} snow_fraction={:.6} sea_ice_fraction={:.6} thermal_rms_k={:.6} tropical_precip_centroid_excursion_deg={:.6} moisture_error={:.6e} final_spinup_years={} final_hash={}",
        CLIMATE_STAGE_ID,
        CLIMATE_STAGE_VERSION,
        options.seed,
        options.coarse_level,
        options.level,
        options.plates,
        summary.orbital_phase_count,
        setup_ms,
        options.runs,
        mean_ms,
        median_ms,
        summary.mean_temperature_k,
        summary.mean_annual_precipitation_mm,
        summary.p95_annual_precipitation_mm,
        summary.mean_wind_speed_m_s,
        summary.maximum_wind_speed_m_s,
        summary.mean_surface_current_m_s,
        summary.persistent_snow_area_fraction,
        summary.sea_ice_area_fraction,
        summary.final_temperature_rms_change_k,
        final_centroid.2,
        summary.moisture_budget_relative_error,
        summary.spinup_years,
        summary.climate_hash_hex(),
    );
    Ok(())
}
