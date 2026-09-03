use interlink_worldgen::{
    build_hydroclimate_closure_report, build_icosphere, generate_coupled_climate,
    generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, ClimateRequest,
    GeologyRequest, LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest,
    TopographyRequest,
};

#[test]
fn hydroclimate_closure_report_is_deterministic_finite_and_diagnostic_only() {
    let seed = "wg5-hydroclimate-closure-test";
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

    let first =
        build_hydroclimate_closure_report(&fine, &terrain, &climate, Some(&no_orography)).unwrap();
    let second =
        build_hydroclimate_closure_report(&fine, &terrain, &climate, Some(&no_orography)).unwrap();

    assert_eq!(first, second);
    assert_eq!(climate.metrics.climate_hash, original_hash);
    assert_eq!(first.sample_count, climate.metrics.sample_count);
    assert_eq!(first.latitude_bands.len(), 12);
    assert!(first.mean_land_precipitation_mm >= 0.0);
    assert!(first.land_precipitation_p05_mm <= first.land_precipitation_p50_mm);
    assert!(first.land_precipitation_p50_mm <= first.land_precipitation_p95_mm);
    assert!(first.land_precipitation_spatial_cv.is_finite());
    assert!(first.land_precipitation_seasonality_p50 >= 0.0);
    assert!(first.land_precipitation_seasonality_p95 >= first.land_precipitation_seasonality_p50);
    assert!(first.mean_land_potential_evaporation_mm >= 0.0);
    assert!(first.land_potential_evaporation_p95_mm >= 0.0);
    assert!(first.land_aridity_index_p05 <= first.land_aridity_index_p50);
    assert!(first.land_aridity_index_p50 <= first.land_aridity_index_p95);
    assert!((0.0..=1.0).contains(&first.land_aridity_below_0_2_fraction));
    assert!((0.0..=1.0).contains(&first.land_aridity_below_0_5_fraction));
    assert!((0.0..=1.0).contains(&first.land_aridity_at_least_1_fraction));
    assert!(first.land_aridity_below_0_2_fraction <= first.land_aridity_below_0_5_fraction);
    assert!((0.0..=1.0).contains(&first.persistent_snow_land_area_fraction));
    assert!((0.0..=1.0).contains(&first.sea_ice_ocean_area_fraction));
    assert!(first
        .no_orography_land_precipitation_rms_difference_mm
        .is_some_and(|value| value >= 0.0));
    assert!(first
        .no_orography_land_precipitation_rms_fraction_of_mean
        .is_some_and(|value| value >= 0.0));
    assert!(first.latitude_bands.iter().all(|band| {
        band.area_fraction >= 0.0
            && (0.0..=1.0).contains(&band.land_area_fraction)
            && band.mean_precipitation_mm >= 0.0
            && band.mean_snowfall_mm >= 0.0
            && band.mean_persistent_snow_potential >= 0.0
            && band.mean_sea_ice_potential >= 0.0
    }));
}
