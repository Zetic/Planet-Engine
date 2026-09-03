use interlink_worldgen::{
    build_icosphere, generate_coupled_climate, generate_crust_and_history,
    generate_initial_topography, generate_lithosphere, generate_tectonics,
    inherit_boundary_interfaces, inherit_physical_state, ClimateRequest, GeologyRequest,
    LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,
};

fn generated_surface(
    seed: &str,
    planet: PlanetPhysicalParameters,
) -> (
    interlink_worldgen::GeodesicTopology,
    interlink_worldgen::TopographyState,
) {
    let coarse = build_icosphere(3).unwrap();
    let fine = build_icosphere(4).unwrap();
    let tectonics =
        generate_tectonics(&coarse, &TectonicsRequest::new(seed, 12), planet).unwrap();
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
    (fine, terrain)
}

#[test]
fn earthlike_climate_is_deterministic_and_couples_atmosphere_ocean_and_moisture() {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let (topology, terrain) = generated_surface("wg5-climate", planet);
    let request = ClimateRequest::new("wg5-climate");
    let first = generate_coupled_climate(&topology, &terrain, planet, &request).unwrap();
    let second = generate_coupled_climate(&topology, &terrain, planet, &request).unwrap();

    assert_eq!(first.metrics.climate_hash, second.metrics.climate_hash);
    assert_eq!(first.temperature_mean_k, second.temperature_mean_k);
    assert_eq!(first.current_east_mean_m_s, second.current_east_mean_m_s);
    assert_eq!(first.annual_precipitation_mm, second.annual_precipitation_mm);
    assert_eq!(first.metrics.sample_count as usize, topology.positions().len());
    assert!(first.metrics.mean_temperature_k > 220.0);
    assert!(first.metrics.mean_temperature_k < 330.0);
    assert!(first.metrics.mean_wind_speed_m_s > 0.1);
    assert!(first.metrics.maximum_surface_current_m_s > 0.01);
    assert!(first.metrics.mean_annual_precipitation_mm >= 0.0);
    assert!(first.metrics.moisture_budget_relative_error < 1.0e-8);

    for i in 0..terrain.submerged_mask.len() {
        if terrain.submerged_mask[i] == 0 {
            assert_eq!(first.current_east_mean_m_s[i], 0.0);
            assert_eq!(first.current_north_mean_m_s[i], 0.0);
        }
    }
}

#[test]
fn stronger_stellar_flux_warms_the_same_solid_planet_without_mutating_wg4() {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let (topology, terrain) = generated_surface("wg5-stellar", planet);
    let original_topography_hash = terrain.metrics.topography_hash;
    let normal = generate_coupled_climate(
        &topology,
        &terrain,
        planet,
        &ClimateRequest::new("wg5-stellar"),
    )
    .unwrap();

    let mut brighter = planet;
    brighter.stellar_flux_w_m2 *= 1.08;
    let warm = generate_coupled_climate(
        &topology,
        &terrain,
        brighter,
        &ClimateRequest::new("wg5-stellar"),
    )
    .unwrap();

    assert_eq!(terrain.metrics.topography_hash, original_topography_hash);
    assert!(warm.metrics.mean_temperature_k > normal.metrics.mean_temperature_k);
    assert_ne!(warm.metrics.climate_hash, normal.metrics.climate_hash);
}

#[test]
fn dry_planet_has_no_ocean_current_or_ocean_evaporation() {
    let mut dry = PlanetPhysicalParameters::earthlike_reference();
    dry.surface_water_mass_kg = 0.0;
    let (topology, terrain) = generated_surface("wg5-dry", dry);
    let climate = generate_coupled_climate(
        &topology,
        &terrain,
        dry,
        &ClimateRequest::new("wg5-dry"),
    )
    .unwrap();

    assert!(terrain.submerged_mask.iter().all(|value| *value == 0));
    assert!(climate.current_speed_mean_m_s.iter().all(|value| *value == 0.0));
    assert_eq!(climate.metrics.global_evaporation_kg, 0.0);
}

#[test]
fn airless_planet_retains_radiative_temperature_but_has_no_wind_or_precipitation() {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let (topology, terrain) = generated_surface("wg5-airless", planet);
    let mut airless = planet;
    airless.reference_surface_pressure_pa = 0.0;
    let climate = generate_coupled_climate(
        &topology,
        &terrain,
        airless,
        &ClimateRequest::new("wg5-airless"),
    )
    .unwrap();

    assert!(climate.temperature_mean_k.iter().all(|value| value.is_finite()));
    assert!(climate.wind_east_mean_m_s.iter().all(|value| *value == 0.0));
    assert!(climate.wind_north_mean_m_s.iter().all(|value| *value == 0.0));
    assert!(climate.annual_precipitation_mm.iter().all(|value| *value == 0.0));
}
