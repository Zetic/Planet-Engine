use interlink_worldgen::{
    analyze_topography_morphology, build_icosphere, generate_crust_and_history,
    generate_initial_topography, generate_lithosphere, generate_tectonics,
    inherit_boundary_interfaces, inherit_physical_state, GeologyRequest, LithosphereRequest,
    PlanetPhysicalParameters, TectonicsRequest, TopographyReliefMorphology, TopographyRequest,
};

fn print_profile(seed: &str, cohort: &str, profile: &TopographyReliefMorphology) {
    println!(
        "profile\t{cohort}\t{seed}\t{:?}\tedges={}\tsources={}\tpeak_abs_m={:.3}\tpeak_mean_abs_m={:.3}\tpeak_distance_km={:.1}\thalf_peak_width_km={:.1}",
        profile.feature,
        profile.source_edge_count,
        profile.source_sample_count,
        profile.peak_absolute_component_relief_m,
        profile.peak_mean_absolute_component_relief_m,
        profile.peak_distance_m / 1_000.0,
        profile.effective_half_peak_width_m / 1_000.0,
    );
    for band in &profile.bands {
        println!(
            "band\t{cohort}\t{seed}\t{:?}\t{:.0}-{:.0}km\tsamples={}\tarea_km2={:.0}\tmean_relief_m={:.3}\tmean_abs_relief_m={:.3}\tmean_solid_m={:.3}\tsubmerged={:.5}",
            profile.feature,
            band.minimum_distance_m / 1_000.0,
            band.maximum_distance_m / 1_000.0,
            band.sample_count,
            band.area_m2 / 1.0e6,
            band.mean_component_relief_m,
            band.mean_absolute_component_relief_m,
            band.mean_solid_elevation_m,
            band.submerged_area_fraction,
        );
    }
}

fn run(seed: &str, cohort: &str) -> Result<(), String> {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(5).map_err(|error| error.to_string())?;
    let fine = build_icosphere(7).map_err(|error| error.to_string())?;
    let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, 16), planet)
        .map_err(|error| error.to_string())?;
    let geology =
        generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet)
            .map_err(|error| error.to_string())?;
    let lithosphere = generate_lithosphere(
        &coarse,
        &tectonics,
        &geology,
        &LithosphereRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;
    let inherited = inherit_physical_state(&fine, 5, &tectonics, &geology, &lithosphere, planet)
        .map_err(|error| error.to_string())?;
    let boundaries =
        inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids)
            .map_err(|error| error.to_string())?;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;
    let report = analyze_topography_morphology(&fine, &inherited, &boundaries, planet, &terrain)
        .map_err(|error| error.to_string())?;

    println!(
        "world\t{cohort}\t{seed}\tlevel={}\tsamples={}\ttopography_hash={:016x}\tboundary_hash={:016x}",
        report.topology_level,
        report.sample_count,
        report.topography_hash,
        report.boundary_hash,
    );
    for profile in [
        &report.ridge,
        &report.continental_rift,
        &report.transitional_divergence,
        &report.trench,
        &report.arc,
        &report.orogen,
    ] {
        print_profile(seed, cohort, profile);
    }
    for band in &report.ocean_age_depth {
        println!(
            "ocean_age\t{cohort}\t{seed}\t{:.0}-{:.0}Myr\tsamples={}\tarea_km2={:.0}\tmean_solid_m={:.3}\tmean_depth_m={:.3}",
            band.minimum_age_myr,
            band.maximum_age_myr,
            band.sample_count,
            band.area_m2 / 1.0e6,
            band.mean_solid_elevation_m,
            band.mean_water_depth_m,
        );
    }
    println!(
        "quiet_ocean\t{cohort}\t{seed}\tsamples={}\tarea_km2={:.0}\toceanic_area_fraction={:.5}\tmean_gradient_m_per_km={:.6}\tgradient_edges={}\tmean_turn_deg={:.3}\trms_turn_deg={:.3}",
        report.quiet_ocean.sample_count,
        report.quiet_ocean.area_m2 / 1.0e6,
        report.quiet_ocean.oceanic_submerged_area_fraction,
        report.quiet_ocean.mean_gradient_m_per_km,
        report.quiet_ocean.gradient_edge_count,
        report.quiet_ocean.mean_gradient_turn_degrees,
        report.quiet_ocean.rms_gradient_turn_degrees,
    );
    Ok(())
}

fn main() -> Result<(), String> {
    let calibration = [
        "3",
        "interlink-wg7c",
        "wg4-boundary-a",
        "wg4-boundary-b",
        "wg4-boundary-c",
        "wg4-boundary-d",
    ];
    let holdout = [
        "wg4-morphology-holdout-a",
        "wg4-morphology-holdout-b",
        "wg4-morphology-holdout-c",
        "wg4-morphology-holdout-d",
    ];

    for seed in calibration {
        run(seed, "calibration")?;
    }
    for seed in holdout {
        run(seed, "holdout")?;
    }
    Ok(())
}
