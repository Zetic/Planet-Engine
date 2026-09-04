use interlink_worldgen_wasm::WasmWorldgenClimate;

#[test]
fn climate_bridge_exposes_coupled_climate_and_accepted_surface() {
    let output = WasmWorldgenClimate::new("wg5-wasm".to_owned(), 3, 4, 12, None).unwrap();

    assert_eq!(output.generator_version(), 9);
    assert_eq!(output.stage_id(), "climate:coupled-surface");
    assert_eq!(output.stage_version(), 6);
    assert_eq!(output.global_solver_level(), 4);
    assert_eq!(
        output.global_solver_sample_count(),
        output.fine_sample_count()
    );
    assert_eq!(output.coarse_level(), 3);
    assert_eq!(output.fine_level(), 4);
    assert_eq!(output.orbital_phase_count(), 24);
    assert!(output.spinup_years() >= 4);
    assert_eq!(
        output.fine_sample_count() as usize,
        output.temperature_mean_k().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.current_east_mean_m_s().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.annual_precipitation_mm().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize * output.orbital_phase_count() as usize,
        output.precipitation_phase_rate_mm_year().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.elevation_above_sea_level_m().len()
    );
    assert!(output.mean_temperature_k().is_finite());
    assert!(output.maximum_wind_speed_m_s() > 0.0);
    assert!(output.maximum_surface_current_m_s() > 0.0);
    assert!(output.moisture_budget_relative_error() < 1.0e-8);
    assert_eq!(output.climate_hash_hex().len(), 16);
    assert_eq!(output.topography_hash_hex().len(), 16);
    assert_eq!(output.inheritance_hash_hex().len(), 16);
    assert_eq!(output.climate_physical_parameter_hash_hex().len(), 16);
    assert_eq!(output.climate_model_parameter_hash_hex().len(), 16);
    assert!(output.orbital_eccentricity() > 0.0);
    assert!(output.atmospheric_mean_molar_mass_kg_per_mol() > 0.0);
    assert!(output.atmospheric_shortwave_reflectivity() > 0.0);
}
