use interlink_worldgen::{
    build_climate_calibration_report, build_icosphere, generate_coupled_climate,
    generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, ClimateRequest,
    GeologyRequest, LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest,
    TopographyRequest,
};

#[test]
fn calibration_report_is_deterministic_finite_and_diagnostic_only() {
    let seed = "wg5-calibration-test";
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(3).unwrap();
    let fine = build_icosphere(4).unwrap();
    let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, 12), planet).unwrap();
    let geology =
        generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet)
            .unwrap();
    let lithosphere = generate_lithosphere(
        &coarse,
        &tectonics,
        &geology,
        &LithosphereRequest::new(seed),
    )
    .unwrap();
    let inherited =
        inherit_physical_state(&fine, 3, &tectonics, &geology, &lithosphere, planet).unwrap();
    let boundaries =
        inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids)
            .unwrap();
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(seed),
    )
    .unwrap();
    let request = ClimateRequest::new(seed);
    let climate = generate_coupled_climate(&fine, &terrain, planet, &request).unwrap();
    let original_hash = climate.metrics.climate_hash;

    let mut no_orography_request = request.clone();
    no_orography_request.parameters.maximum_orographic_fraction = 0.0;
    let no_orography =
        generate_coupled_climate(&fine, &terrain, planet, &no_orography_request).unwrap();

    let first = build_climate_calibration_report(
        &fine,
        &terrain,
        planet,
        &request,
        &climate,
        Some(&no_orography),
    )
    .unwrap();
    let second = build_climate_calibration_report(
        &fine,
        &terrain,
        planet,
        &request,
        &climate,
        Some(&no_orography),
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(climate.metrics.climate_hash, original_hash);
    assert_eq!(first.latitude_bands.len(), 12);
    assert!(first.mean_land_elevation_m.is_finite());
    assert!(first.p95_land_elevation_m >= first.median_land_elevation_m);
    assert!((0.0..=1.0).contains(&first.land_area_above_2km_fraction));
    assert!((0.0..=1.0).contains(&first.land_area_above_4km_fraction));
    assert!(first.land_area_above_4km_fraction <= first.land_area_above_2km_fraction);
    assert!(first.effective_absorbed_shortwave_w_m2.is_finite());
    assert!(first.outgoing_longwave_proxy_w_m2.is_finite());
    assert!(first.toa_energy_imbalance_proxy_w_m2.is_finite());
    assert!((0.0..=1.0).contains(&first.reconstructed_wind_cap_fraction));
    assert!((0.0..=1.0).contains(&first.moisture_transport_limiter_fraction));
    assert!((0.0..=2.0).contains(&first.mean_state_relative_humidity_p05));
    assert!((0.0..=2.0).contains(&first.mean_state_relative_humidity_p50));
    assert!((0.0..=2.0).contains(&first.mean_state_relative_humidity_p95));
    assert!(first.mean_state_relative_humidity_p05 <= first.mean_state_relative_humidity_p50);
    assert!(first.mean_state_relative_humidity_p50 <= first.mean_state_relative_humidity_p95);
    assert!(first.mean_potential_evaporation_mm >= 0.0);
    assert!(first.precipitation_to_evaporation_ratio.is_finite());
    assert!(first.no_orography_mean_annual_precipitation_mm.is_some());
    assert!(first.orographic_precipitation_causal_fraction.is_some());
    assert!(first.mean_snowfall_mm >= 0.0);
    assert!(first.annual_mean_ocean_heat_tendency_rms_index >= 0.0);
    assert!(first.latitude_bands.iter().all(|band| {
        band.area_fraction >= 0.0
            && band.mean_temperature_k.is_finite()
            && band.mean_precipitation_mm >= 0.0
            && band.mean_relative_humidity_proxy.is_finite()
    }));
}
