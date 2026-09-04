use interlink_worldgen::{
    build_hydroclimate_closure_report, build_icosphere,
    generate_coupled_climate_reference_with_diagnostics, generate_coupled_climate_with_diagnostics,
    generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    ClimateGenerationDiagnostics, ClimateRequest, ClimateState, GeodesicTopology, GeologyRequest,
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
    climate_level: Option<u8>,
    compare_reference: bool,
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
        .unwrap_or_else(|| "ci-wg5-performance".to_owned());
    let options = Options {
        seed,
        coarse_level: parse_value(&args, "--coarse-level", 4_u8)?,
        level: parse_value(&args, "--level", 6_u8)?,
        plates: parse_value(&args, "--plates", 16_u16)?,
        runs: parse_value(&args, "--runs", 1_u8)?,
        climate_level: args
            .iter()
            .position(|arg| arg == "--climate-level")
            .and_then(|index| args.get(index + 1))
            .map(|value| {
                value
                    .parse::<u8>()
                    .map_err(|_| format!("invalid value for --climate-level: {value}"))
            })
            .transpose()?,
        compare_reference: args.iter().any(|arg| arg == "--compare-reference"),
    };
    if options.coarse_level > options.level {
        return Err("--coarse-level cannot exceed --level".to_owned());
    }
    if options.runs == 0 {
        return Err("--runs must be at least 1".to_owned());
    }
    if options
        .climate_level
        .is_some_and(|level| level > options.level)
    {
        return Err("--climate-level cannot exceed --level".to_owned());
    }
    Ok(options)
}

fn weighted_rmse(topology: &GeodesicTopology, reference: &[f32], candidate: &[f32]) -> f64 {
    let mut weighted_squared_error = 0.0;
    let mut area = 0.0;
    for index in 0..reference.len() {
        let weight = topology.dual_area_steradians()[index];
        let error = f64::from(candidate[index]) - f64::from(reference[index]);
        weighted_squared_error += error * error * weight;
        area += weight;
    }
    (weighted_squared_error / area.max(1.0e-18)).sqrt()
}

fn weighted_mean(
    topology: &GeodesicTopology,
    values: &[f32],
    include: impl Fn(usize) -> bool,
) -> f64 {
    let mut weighted = 0.0;
    let mut area = 0.0;
    for (index, value) in values.iter().enumerate() {
        if !include(index) {
            continue;
        }
        let weight = topology.dual_area_steradians()[index];
        weighted += f64::from(*value) * weight;
        area += weight;
    }
    weighted / area.max(1.0e-18)
}

fn weighted_subset_rmse(
    topology: &GeodesicTopology,
    reference: &[f32],
    candidate: &[f32],
    include: impl Fn(usize) -> bool,
) -> f64 {
    let mut weighted_squared_error = 0.0;
    let mut area = 0.0;
    for index in 0..reference.len() {
        if !include(index) {
            continue;
        }
        let weight = topology.dual_area_steradians()[index];
        let error = f64::from(candidate[index]) - f64::from(reference[index]);
        weighted_squared_error += error * error * weight;
        area += weight;
    }
    (weighted_squared_error / area.max(1.0e-18)).sqrt()
}

fn weighted_correlation(topology: &GeodesicTopology, reference: &[f32], candidate: &[f32]) -> f64 {
    let reference_mean = weighted_mean(topology, reference, |_| true);
    let candidate_mean = weighted_mean(topology, candidate, |_| true);
    let mut covariance = 0.0;
    let mut reference_variance = 0.0;
    let mut candidate_variance = 0.0;
    for index in 0..reference.len() {
        let weight = topology.dual_area_steradians()[index];
        let reference_delta = f64::from(reference[index]) - reference_mean;
        let candidate_delta = f64::from(candidate[index]) - candidate_mean;
        covariance += reference_delta * candidate_delta * weight;
        reference_variance += reference_delta * reference_delta * weight;
        candidate_variance += candidate_delta * candidate_delta * weight;
    }
    covariance
        / (reference_variance * candidate_variance)
            .sqrt()
            .max(1.0e-18)
}

fn weighted_standard_deviation(topology: &GeodesicTopology, values: &[f32]) -> f64 {
    let mean = weighted_mean(topology, values, |_| true);
    let mut variance = 0.0;
    let mut area = 0.0;
    for (index, value) in values.iter().enumerate() {
        let weight = topology.dual_area_steradians()[index];
        let delta = f64::from(*value) - mean;
        variance += delta * delta * weight;
        area += weight;
    }
    (variance / area.max(1.0e-18)).sqrt()
}

fn tropical_precipitation_centroid_excursion_deg(
    topology: &GeodesicTopology,
    diagnostics: &ClimateGenerationDiagnostics,
    phase_count: usize,
) -> f64 {
    let sample_count = topology.metrics().sample_count as usize;
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    for phase in 0..phase_count {
        let mut latitude_moment = 0.0;
        let mut precipitation = 0.0;
        for sample in 0..sample_count {
            let latitude_deg = topology.positions()[sample][2]
                .clamp(-1.0, 1.0)
                .asin()
                .to_degrees();
            if latitude_deg.abs() > 30.0 {
                continue;
            }
            let weight = topology.dual_area_steradians()[sample]
                * f64::from(
                    diagnostics.precipitation_phase_rate_mm_year[phase * sample_count + sample],
                );
            latitude_moment += latitude_deg * weight;
            precipitation += weight;
        }
        if precipitation > 0.0 {
            let centroid = latitude_moment / precipitation;
            minimum = minimum.min(centroid);
            maximum = maximum.max(centroid);
        }
    }
    if minimum.is_finite() && maximum.is_finite() {
        maximum - minimum
    } else {
        0.0
    }
}

fn print_comparison(
    topology: &GeodesicTopology,
    terrain: &interlink_worldgen::TopographyState,
    reference: &ClimateState,
    reference_diagnostics: &ClimateGenerationDiagnostics,
    candidate: &ClimateState,
    candidate_diagnostics: &ClimateGenerationDiagnostics,
) -> Result<(), String> {
    let reference_report = build_hydroclimate_closure_report(topology, terrain, reference, None)
        .map_err(|error| error.to_string())?;
    let candidate_report = build_hydroclimate_closure_report(topology, terrain, candidate, None)
        .map_err(|error| error.to_string())?;
    let phases = usize::from(reference.metrics.orbital_phase_count);
    println!(
        "comparison temperature_rmse_k={:.6} temperature_correlation={:.6} reference_mean_temperature_k={:.6} candidate_mean_temperature_k={:.6} reference_temperature_sd_k={:.6} candidate_temperature_sd_k={:.6} land_temperature_rmse_k={:.6} ocean_temperature_rmse_k={:.6} precipitation_rmse_mm={:.6} precipitation_correlation={:.6} reference_precipitation_sd_mm={:.6} candidate_precipitation_sd_mm={:.6} pet_rmse_mm={:.6} pet_correlation={:.6} reference_mean_precipitation_mm={:.6} candidate_mean_precipitation_mm={:.6} reference_land_precipitation_mm={:.6} candidate_land_precipitation_mm={:.6} reference_ocean_precipitation_mm={:.6} candidate_ocean_precipitation_mm={:.6} reference_land_pet_mm={:.6} candidate_land_pet_mm={:.6} reference_land_p05_mm={:.6} candidate_land_p05_mm={:.6} reference_land_p50_mm={:.6} candidate_land_p50_mm={:.6} reference_land_p95_mm={:.6} candidate_land_p95_mm={:.6} reference_tropical_migration_deg={:.6} candidate_tropical_migration_deg={:.6}",
        weighted_rmse(
            topology,
            &reference.temperature_mean_k,
            &candidate.temperature_mean_k,
        ),
        weighted_correlation(
            topology,
            &reference.temperature_mean_k,
            &candidate.temperature_mean_k,
        ),
        weighted_mean(topology, &reference.temperature_mean_k, |_| true),
        weighted_mean(topology, &candidate.temperature_mean_k, |_| true),
        weighted_standard_deviation(topology, &reference.temperature_mean_k),
        weighted_standard_deviation(topology, &candidate.temperature_mean_k),
        weighted_subset_rmse(
            topology,
            &reference.temperature_mean_k,
            &candidate.temperature_mean_k,
            |index| terrain.submerged_mask[index] == 0,
        ),
        weighted_subset_rmse(
            topology,
            &reference.temperature_mean_k,
            &candidate.temperature_mean_k,
            |index| terrain.submerged_mask[index] != 0,
        ),
        weighted_rmse(
            topology,
            &reference.annual_precipitation_mm,
            &candidate.annual_precipitation_mm,
        ),
        weighted_correlation(
            topology,
            &reference.annual_precipitation_mm,
            &candidate.annual_precipitation_mm,
        ),
        weighted_standard_deviation(topology, &reference.annual_precipitation_mm),
        weighted_standard_deviation(topology, &candidate.annual_precipitation_mm),
        weighted_rmse(
            topology,
            &reference.potential_evaporation_mm,
            &candidate.potential_evaporation_mm,
        ),
        weighted_correlation(
            topology,
            &reference.potential_evaporation_mm,
            &candidate.potential_evaporation_mm,
        ),
        reference.metrics.mean_annual_precipitation_mm,
        candidate.metrics.mean_annual_precipitation_mm,
        reference_report.mean_land_precipitation_mm,
        candidate_report.mean_land_precipitation_mm,
        weighted_mean(topology, &reference.annual_precipitation_mm, |index| {
            terrain.submerged_mask[index] != 0
        }),
        weighted_mean(topology, &candidate.annual_precipitation_mm, |index| {
            terrain.submerged_mask[index] != 0
        }),
        reference_report.mean_land_potential_evaporation_mm,
        candidate_report.mean_land_potential_evaporation_mm,
        reference_report.land_precipitation_p05_mm,
        candidate_report.land_precipitation_p05_mm,
        reference_report.land_precipitation_p50_mm,
        candidate_report.land_precipitation_p50_mm,
        reference_report.land_precipitation_p95_mm,
        candidate_report.land_precipitation_p95_mm,
        tropical_precipitation_centroid_excursion_deg(
            topology,
            reference_diagnostics,
            phases,
        ),
        tropical_precipitation_centroid_excursion_deg(
            topology,
            candidate_diagnostics,
            phases,
        ),
    );
    Ok(())
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
    let setup_ms = setup_started.elapsed().as_secs_f64() * 1_000.0;

    let mut request = ClimateRequest::new(options.seed.as_str());
    if let Some(climate_level) = options.climate_level {
        request.parameters.maximum_global_climate_level = climate_level;
    }
    let mut elapsed_ms = Vec::with_capacity(usize::from(options.runs));
    let mut final_summary = None;
    let reference = if options.compare_reference {
        let started = Instant::now();
        let mut progress = |_completed_years: u8, _maximum_years: u8| {};
        let result = generate_coupled_climate_reference_with_diagnostics(
            &fine,
            &terrain,
            planet,
            &request,
            &mut progress,
        )
        .map_err(|error| error.to_string())?;
        println!(
            "reference climate_ms={:.3} hash={}",
            started.elapsed().as_secs_f64() * 1_000.0,
            result.0.metrics.climate_hash_hex(),
        );
        Some(result)
    } else {
        None
    };

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
        println!(
            "run={} climate_ms={:.3} samples={} solver_level={} solver_samples={} phases={} spinup_years={} max_moisture_substeps={} mean_temperature_k={:.6} mean_precipitation_mm={:.6} moisture_error={:.6e} climate_hash={}",
            run + 1,
            duration_ms,
            climate.metrics.sample_count,
            climate.metrics.global_solver_level,
            climate.metrics.global_solver_sample_count,
            climate.metrics.orbital_phase_count,
            climate.metrics.spinup_years,
            climate.metrics.maximum_moisture_transport_substeps,
            climate.metrics.mean_temperature_k,
            climate.metrics.mean_annual_precipitation_mm,
            climate.metrics.moisture_budget_relative_error,
            climate.metrics.climate_hash_hex(),
        );
        if let Some((reference_climate, reference_diagnostics)) = &reference {
            print_comparison(
                &fine,
                &terrain,
                reference_climate,
                reference_diagnostics,
                &climate,
                &diagnostics,
            )?;
        }
        final_summary = Some(climate.metrics);
    }

    elapsed_ms.sort_by(|a, b| a.total_cmp(b));
    let median_ms = elapsed_ms[elapsed_ms.len() / 2];
    let mean_ms = elapsed_ms.iter().sum::<f64>() / elapsed_ms.len() as f64;
    let summary = final_summary.expect("at least one benchmark run");
    println!(
        "wg5_performance stage={}@{} seed={} coarse_level={} fine_level={} plates={} setup_ms={:.3} runs={} mean_climate_ms={:.3} median_climate_ms={:.3} final_spinup_years={} final_hash={}",
        CLIMATE_STAGE_ID,
        CLIMATE_STAGE_VERSION,
        options.seed,
        options.coarse_level,
        options.level,
        options.plates,
        setup_ms,
        options.runs,
        mean_ms,
        median_ms,
        summary.spinup_years,
        summary.climate_hash_hex(),
    );
    Ok(())
}
