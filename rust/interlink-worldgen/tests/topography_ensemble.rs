use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, GeologyRequest,
    LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,
};

struct Generated {
    inherited: interlink_worldgen::InheritedPhysicalState,
    terrain: interlink_worldgen::TopographyState,
}

fn generate(seed: &str, water_mass_kg: Option<f64>) -> Generated {
    let mut planet = PlanetPhysicalParameters::earthlike_reference();
    if let Some(water_mass_kg) = water_mass_kg {
        planet.surface_water_mass_kg = water_mass_kg;
    }
    let coarse = build_icosphere(3).unwrap();
    let fine = build_icosphere(4).unwrap();
    let tectonics = generate_tectonics(
        &coarse,
        &TectonicsRequest::new(seed, 12),
        planet,
    )
    .unwrap();
    let geology = generate_crust_and_history(
        &coarse,
        &tectonics,
        &GeologyRequest::new(seed),
        planet,
    )
    .unwrap();
    let lithosphere = generate_lithosphere(
        &coarse,
        &tectonics,
        &geology,
        &LithosphereRequest::new(seed),
    )
    .unwrap();
    let inherited = inherit_physical_state(
        &fine,
        3,
        &tectonics,
        &geology,
        &lithosphere,
        planet,
    )
    .unwrap();
    let boundaries = inherit_boundary_interfaces(
        &coarse,
        &fine,
        &tectonics,
        &geology,
        &inherited.plate_ids,
    )
    .unwrap();
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(seed),
    )
    .unwrap();
    Generated { inherited, terrain }
}

fn conditional_mean(values: &[f32], predicate: impl Fn(usize) -> bool) -> Option<f64> {
    let mut sum = 0.0;
    let mut count = 0usize;
    for (index, value) in values.iter().enumerate() {
        if predicate(index) {
            sum += f64::from(*value);
            count += 1;
        }
    }
    (count > 0).then_some(sum / count as f64)
}

#[test]
fn multi_seed_topography_expresses_expected_signed_physical_responses() {
    let mut old_ocean_is_deeper = 0;
    let mut orogenic_high_is_higher = 0;
    let mut basin_high_is_lower = 0;
    let mut subduction_morphology_worlds = 0;

    for seed in ["wg4-e0", "wg4-e1", "wg4-e2", "wg4-e3", "wg4-e4"] {
        let generated = generate(seed, None);
        let inherited = &generated.inherited;
        let terrain = &generated.terrain;

        let young = conditional_mean(&terrain.thermal_elevation_m, |i| {
            inherited.crust_kind[i] == 1 && inherited.crust_age_myr[i] < 60.0
        });
        let old = conditional_mean(&terrain.thermal_elevation_m, |i| {
            inherited.crust_kind[i] == 1 && inherited.crust_age_myr[i] > 120.0
        });
        if let (Some(young), Some(old)) = (young, old) {
            if old < young { old_ocean_is_deeper += 1; }
        }

        let high_orogeny = conditional_mean(&terrain.orogenic_elevation_m, |i| inherited.orogenic_history[i] > 0.65);
        let low_orogeny = conditional_mean(&terrain.orogenic_elevation_m, |i| inherited.orogenic_history[i] < 0.20);
        if let (Some(high), Some(low)) = (high_orogeny, low_orogeny) {
            if high > low { orogenic_high_is_higher += 1; }
        }

        let high_basin = conditional_mean(&terrain.rift_basin_elevation_m, |i| inherited.basin_potential[i] > 0.65);
        let low_basin = conditional_mean(&terrain.rift_basin_elevation_m, |i| inherited.basin_potential[i] < 0.20);
        if let (Some(high), Some(low)) = (high_basin, low_basin) {
            if high < low { basin_high_is_lower += 1; }
        }

        let deepest_trench = terrain.trench_elevation_m.iter().copied().fold(0.0_f32, f32::min);
        let highest_arc = terrain.arc_elevation_m.iter().copied().fold(0.0_f32, f32::max);
        if deepest_trench < -100.0 && highest_arc > 100.0 { subduction_morphology_worlds += 1; }

        assert!(terrain.metrics.clamped_sample_count < terrain.metrics.sample_count / 20);
        assert!(terrain.metrics.water_volume_relative_error < 1.0e-10);
    }

    assert!(old_ocean_is_deeper >= 3, "oceanic thermal subsidence must deepen old seafloor across the ensemble");
    assert!(orogenic_high_is_higher >= 4, "orogenic history must produce positive relief across the ensemble");
    assert!(basin_high_is_lower >= 4, "basin potential must produce negative relief across the ensemble");
    assert!(subduction_morphology_worlds >= 2, "the ensemble must exercise both trench and arc morphology");
}

#[test]
fn changing_only_water_inventory_changes_flooding_not_the_solid_surface() {
    let wet = generate("wg4-water-profile", None);
    let half_water = generate("wg4-water-profile", Some(7.0e20));
    assert_eq!(wet.terrain.solid_elevation_m, half_water.terrain.solid_elevation_m);
    assert_ne!(wet.terrain.metrics.topography_hash, half_water.terrain.metrics.topography_hash);
    assert_ne!(wet.terrain.metrics.sea_level_m, half_water.terrain.metrics.sea_level_m);
    assert_ne!(wet.terrain.submerged_mask, half_water.terrain.submerged_mask);
}
