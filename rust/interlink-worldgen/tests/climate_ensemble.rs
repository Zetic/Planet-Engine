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
    assert_eq!(
        first.annual_precipitation_mm,
        second.annual_precipitation_mm
    );
    assert_eq!(
        first.metrics.sample_count as usize,
        topology.positions().len()
    );
    assert!(first.metrics.mean_temperature_k > 220.0);
    assert!(first.metrics.mean_temperature_k < 330.0);
    assert!(first.metrics.mean_wind_speed_m_s > 0.1);
    assert!(first.metrics.mean_surface_current_m_s > 0.0);
    assert!(
        first.metrics.ocean_divergence_residual_m_s
            < first.metrics.mean_surface_current_m_s * 0.10 + 1.0e-6,
        "projected ocean transport should have a small divergence residual: residual={} mean_current={}",
        first.metrics.ocean_divergence_residual_m_s,
        first.metrics.mean_surface_current_m_s,
    );
    assert!(first.metrics.maximum_surface_current_m_s > 0.01);
    assert!(first.metrics.mean_annual_precipitation_mm >= 0.0);
    assert!(first.metrics.moisture_budget_relative_error < 1.0e-8);

    let mut no_ocean_heat_request = request.clone();
    no_ocean_heat_request.parameters.ocean_advection_relaxation = 0.0;
    let no_ocean_heat =
        generate_coupled_climate(&topology, &terrain, planet, &no_ocean_heat_request).unwrap();
    assert_ne!(
        first.sea_surface_temperature_mean_k, no_ocean_heat.sea_surface_temperature_mean_k,
        "surface-current heat advection must causally affect SST",
    );

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
fn latent_energy_availability_limits_ocean_evaporation_on_fixed_wg4_surface() {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let (topology, terrain) = generated_surface("wg5-latent-energy", planet);
    let normal_request = ClimateRequest::new("wg5-latent-energy");
    let normal = generate_coupled_climate(&topology, &terrain, planet, &normal_request).unwrap();

    let mut constrained_request = normal_request.clone();
    constrained_request.parameters.evaporation_energy_fraction = 0.05;
    let constrained =
        generate_coupled_climate(&topology, &terrain, planet, &constrained_request).unwrap();

    assert!(constrained.metrics.global_evaporation_kg < normal.metrics.global_evaporation_kg);
    assert!(constrained.metrics.moisture_budget_relative_error < 1.0e-8);
    assert_ne!(
        constrained.metrics.climate_hash,
        normal.metrics.climate_hash
    );
}

#[test]
fn dry_planet_has_no_ocean_current_or_ocean_evaporation() {
    let mut dry = PlanetPhysicalParameters::earthlike_reference();
    dry.surface_water_mass_kg = 0.0;
    let (topology, terrain) = generated_surface("wg5-dry", dry);
    let climate =
        generate_coupled_climate(&topology, &terrain, dry, &ClimateRequest::new("wg5-dry"))
            .unwrap();

    assert!(terrain.submerged_mask.iter().all(|value| *value == 0));
    assert!(climate
        .current_speed_mean_m_s
        .iter()
        .all(|value| *value == 0.0));
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

    assert!(climate
        .temperature_mean_k
        .iter()
        .all(|value| value.is_finite()));
    assert!(climate.wind_east_mean_m_s.iter().all(|value| *value == 0.0));
    assert!(climate
        .wind_north_mean_m_s
        .iter()
        .all(|value| *value == 0.0));
    assert!(climate
        .annual_precipitation_mm
        .iter()
        .all(|value| *value == 0.0));
    for (index, submerged) in terrain.submerged_mask.iter().enumerate() {
        if *submerged != 0 {
            assert!(
                (climate.sea_surface_temperature_mean_k[index]
                    - climate.temperature_mean_k[index])
                    .abs()
                    < 1.0e-4,
                "airless ocean surfaces must not retain a distinct atmospheric temperature reservoir",
            );
        }
    }

    let mut no_transport_request = ClimateRequest::new("wg5-airless");
    no_transport_request
        .parameters
        .atmospheric_heat_diffusivity_m2_s = 1.0;
    let no_transport =
        generate_coupled_climate(&topology, &terrain, airless, &no_transport_request).unwrap();
    assert_eq!(
        climate.temperature_mean_k, no_transport.temperature_mean_k,
        "airless surfaces must not retain atmospheric lateral heat redistribution",
    );
}

fn mean_field(values: &[f32]) -> f64 {
    values.iter().map(|value| f64::from(*value)).sum::<f64>() / values.len() as f64
}

fn zonal_fraction(east: &[f32], north: &[f32]) -> f64 {
    let east_total = east
        .iter()
        .map(|value| f64::from(*value).abs())
        .sum::<f64>();
    let north_total = north
        .iter()
        .map(|value| f64::from(*value).abs())
        .sum::<f64>();
    east_total / (east_total + north_total).max(1.0e-12)
}

#[test]
fn axial_tilt_controls_seasonal_insolation_on_a_fixed_wg4_surface() {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let (topology, terrain) = generated_surface("wg5-tilt", planet);
    let normal = generate_coupled_climate(
        &topology,
        &terrain,
        planet,
        &ClimateRequest::new("wg5-tilt"),
    )
    .unwrap();
    let mut zero_tilt = planet;
    zero_tilt.axial_tilt_rad = 0.0;
    let no_tilt = generate_coupled_climate(
        &topology,
        &terrain,
        zero_tilt,
        &ClimateRequest::new("wg5-tilt"),
    )
    .unwrap();
    let normal_amplitude = mean_field(&normal.seasonal_insolation_amplitude_w_m2);
    let zero_tilt_amplitude = mean_field(&no_tilt.seasonal_insolation_amplitude_w_m2);
    assert!(
        zero_tilt_amplitude < normal_amplitude * 0.90,
        "removing axial tilt should materially reduce seasonal insolation amplitude: normal={normal_amplitude} zero_tilt={zero_tilt_amplitude}",
    );
}

#[test]
fn rotation_rate_changes_circulation_and_faster_rotation_is_more_zonal() {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let (topology, terrain) = generated_surface("wg5-rotation", planet);
    let mut slow = planet;
    slow.rotation_period_s *= 4.0;
    let mut fast = planet;
    fast.rotation_period_s *= 0.5;
    let slow_climate = generate_coupled_climate(
        &topology,
        &terrain,
        slow,
        &ClimateRequest::new("wg5-rotation"),
    )
    .unwrap();
    let fast_climate = generate_coupled_climate(
        &topology,
        &terrain,
        fast,
        &ClimateRequest::new("wg5-rotation"),
    )
    .unwrap();
    let slow_zonal = zonal_fraction(
        &slow_climate.wind_east_mean_m_s,
        &slow_climate.wind_north_mean_m_s,
    );
    let fast_zonal = zonal_fraction(
        &fast_climate.wind_east_mean_m_s,
        &fast_climate.wind_north_mean_m_s,
    );
    assert!(
        fast_zonal > slow_zonal,
        "faster rotation should increase zonal control: slow={slow_zonal} fast={fast_zonal}",
    );
    assert_ne!(
        slow_climate.wind_east_mean_m_s,
        fast_climate.wind_east_mean_m_s
    );
    assert_ne!(
        slow_climate.current_east_mean_m_s,
        fast_climate.current_east_mean_m_s
    );
}

#[test]
fn atmospheric_specific_heat_changes_thermal_redistribution_on_fixed_wg4_surface() {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let (topology, terrain) = generated_surface("wg5-specific-heat", planet);
    let reference = ClimateRequest::new("wg5-specific-heat");
    let normal = generate_coupled_climate(&topology, &terrain, planet, &reference).unwrap();
    let mut high_heat_capacity = reference.clone();
    high_heat_capacity
        .physical
        .atmospheric_specific_heat_j_per_kg_k *= 2.0;
    let high = generate_coupled_climate(&topology, &terrain, planet, &high_heat_capacity).unwrap();
    assert_ne!(normal.temperature_mean_k, high.temperature_mean_k);
    assert_ne!(normal.wind_east_mean_m_s, high.wind_east_mean_m_s);
}

#[test]
fn core_rejects_unconverged_climate_instead_of_returning_a_state() {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let (topology, terrain) = generated_surface("wg5-nonconverged", planet);
    let mut request = ClimateRequest::new("wg5-nonconverged");
    request.parameters.minimum_spinup_years = 1;
    request.parameters.maximum_spinup_years = 1;
    request.parameters.convergence_temperature_rms_k = 1.0e-12;
    let error = generate_coupled_climate(&topology, &terrain, planet, &request).unwrap_err();
    assert!(error.to_string().contains("did not converge"));
}
