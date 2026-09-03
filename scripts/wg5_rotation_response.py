from pathlib import Path

climate_path = Path('rust/interlink-worldgen/src/climate.rs')
text = climate_path.read_text()

needle = 'const TWO_PI: f64 = std::f64::consts::PI * 2.0;\n'
assert needle in text
text = text.replace(needle, needle + 'const EARTH_REFERENCE_ROTATION_PERIOD_S: f64 = 86_164.0905;\n', 1)

old = '''fn baseline_zonal_wind(latitude_rad: f64) -> f64 {
    let degrees = latitude_rad.abs().to_degrees();
    if degrees < 30.0 {
        -8.0 * (1.0 - 0.35 * degrees / 30.0)
    } else if degrees < 60.0 {
        10.0 * ((degrees - 30.0) / 30.0 * std::f64::consts::PI).sin()
    } else {
        -4.0 * ((degrees - 60.0) / 30.0 * std::f64::consts::FRAC_PI_2)
            .sin()
            .abs()
    }
}
'''
new = '''fn rotation_response(rotation_period_s: f64) -> f64 {
    (EARTH_REFERENCE_ROTATION_PERIOD_S / rotation_period_s).clamp(0.05, 8.0)
}

fn circulation_cell_edges(rotation_ratio: f64) -> (f64, f64) {
    let hadley_edge_deg = (30.0 / rotation_ratio.sqrt()).clamp(15.0, 60.0);
    let polar_edge_deg = hadley_edge_deg + 0.5 * (90.0 - hadley_edge_deg);
    (hadley_edge_deg, polar_edge_deg)
}

fn baseline_zonal_wind(latitude_rad: f64, rotation_ratio: f64) -> f64 {
    let degrees = latitude_rad.abs().to_degrees();
    let (hadley_edge_deg, polar_edge_deg) = circulation_cell_edges(rotation_ratio);
    let zonal_strength = rotation_ratio.sqrt().clamp(0.35, 2.5);
    if degrees < hadley_edge_deg {
        -8.0 * zonal_strength * (1.0 - 0.35 * degrees / hadley_edge_deg)
    } else if degrees < polar_edge_deg {
        let phase = (degrees - hadley_edge_deg) / (polar_edge_deg - hadley_edge_deg);
        10.0 * zonal_strength * (phase * std::f64::consts::PI).sin()
    } else {
        let phase = ((degrees - polar_edge_deg) / (90.0 - polar_edge_deg)).clamp(0.0, 1.0);
        -4.0 * zonal_strength * (phase * std::f64::consts::FRAC_PI_2).sin().abs()
    }
}

fn coriolis_deflection_factor(latitude_rad: f64, omega: f64, configured_strength: f64) -> f64 {
    let coriolis = 2.0 * omega * latitude_rad.sin();
    if coriolis.abs() <= 1.0e-15 {
        return 0.0;
    }
    let earth_omega = TWO_PI / EARTH_REFERENCE_ROTATION_PERIOD_S;
    let normalized = (coriolis.abs() / (2.0 * earth_omega)).clamp(0.0, 2.5);
    coriolis.signum() * configured_strength * normalized
}
'''
assert old in text
text = text.replace(old, new, 1)

old = '''    let omega = if planet.rotation_period_s > 0.0 {
        TWO_PI / planet.rotation_period_s
    } else {
        0.0
    };
    let atmosphere_exists = planet.reference_surface_pressure_pa > 0.0;
'''
new = '''    let omega = if planet.rotation_period_s > 0.0 {
        TWO_PI / planet.rotation_period_s
    } else {
        0.0
    };
    let rotation_ratio = rotation_response(planet.rotation_period_s);
    let (hadley_edge_deg, _) = circulation_cell_edges(rotation_ratio);
    let rotational_strength = rotation_ratio.sqrt().clamp(0.35, 2.5);
    let overturning_strength = (1.0 / rotation_ratio.sqrt()).clamp(0.4, 2.5);
    let rotational_transition_start_deg = (4.0 / rotational_strength).clamp(2.0, 12.0);
    let rotational_transition_width_deg = (18.0 / rotational_strength).clamp(8.0, 36.0);
    let atmosphere_exists = planet.reference_surface_pressure_pa > 0.0;
'''
assert old in text
text = text.replace(old, new, 1)

old = '''                    let latitude_abs_deg = latitude[i].abs().to_degrees();
                    let rotational_blend = ((latitude_abs_deg - 4.0) / 18.0).clamp(0.0, 1.0);
                    let rotation_sign = if omega >= 0.0 { 1.0 } else { -1.0 };
                    let geostrophic_east = -rotation_sign
                        * gradient_north
                        * 1_000_000.0
                        * parameters.wind_thermal_gradient_scale;
                    let geostrophic_north = rotation_sign
                        * gradient_east
                        * 1_000_000.0
                        * parameters.wind_thermal_gradient_scale;
                    let zonal = baseline_zonal_wind(latitude[i]) * rotation_sign;
                    let meridional = if latitude_abs_deg < 30.0 {
                        -latitude[i].signum() * 2.6 * (1.0 - latitude_abs_deg / 30.0)
                    } else {
                        0.0
                    };
'''
new = '''                    let latitude_abs_deg = latitude[i].abs().to_degrees();
                    let rotational_blend = ((latitude_abs_deg - rotational_transition_start_deg)
                        / rotational_transition_width_deg)
                        .clamp(0.0, 1.0);
                    let geostrophic_east = -gradient_north
                        * 1_000_000.0
                        * parameters.wind_thermal_gradient_scale
                        * rotational_strength;
                    let geostrophic_north = gradient_east
                        * 1_000_000.0
                        * parameters.wind_thermal_gradient_scale
                        * rotational_strength;
                    let zonal = baseline_zonal_wind(latitude[i], rotation_ratio);
                    let meridional = if latitude_abs_deg < hadley_edge_deg {
                        -latitude[i].signum()
                            * 2.6
                            * overturning_strength
                            * (1.0 - latitude_abs_deg / hadley_edge_deg)
                    } else {
                        0.0
                    };
'''
assert old in text
text = text.replace(old, new, 1)

old = '''                    let coriolis = 2.0 * omega * latitude[i].sin();
                    let sign = if coriolis >= 0.0 { 1.0 } else { -1.0 };
                    let mobility = (water_depth_m[i] / parameters.ocean_bathymetric_drag_depth_m)
                        .clamp(0.08, 1.0)
                        .sqrt();
                    let east = parameters.ocean_wind_coupling
                        * mobility
                        * (wind_east[i]
                            + sign * parameters.ocean_coriolis_deflection * wind_north[i]);
                    let north = parameters.ocean_wind_coupling
                        * mobility
                        * (wind_north[i]
                            - sign * parameters.ocean_coriolis_deflection * wind_east[i]);
'''
new = '''                    let deflection = coriolis_deflection_factor(
                        latitude[i],
                        omega,
                        parameters.ocean_coriolis_deflection,
                    );
                    let mobility = (water_depth_m[i] / parameters.ocean_bathymetric_drag_depth_m)
                        .clamp(0.08, 1.0)
                        .sqrt();
                    let east = parameters.ocean_wind_coupling
                        * mobility
                        * (wind_east[i] + deflection * wind_north[i]);
                    let north = parameters.ocean_wind_coupling
                        * mobility
                        * (wind_north[i] - deflection * wind_east[i]);
'''
assert old in text
text = text.replace(old, new, 1)

needle = '''    #[test]
    fn seasonal_solar_geometry_handles_polar_night_and_equatorial_daylight() {
        let equator = daily_mean_insolation(0.0, 0.0, 1361.0);
        let north_pole_winter = daily_mean_insolation(
            std::f64::consts::FRAC_PI_2,
            -23.4_f64.to_radians(),
            1361.0,
        );
        assert!(equator > 400.0);
        assert_eq!(north_pole_winter, 0.0);
    }
'''
addition = needle + '''
    #[test]
    fn rotation_response_broadens_slow_hadley_cells_and_zeroes_equatorial_coriolis() {
        let earth_ratio = rotation_response(EARTH_REFERENCE_ROTATION_PERIOD_S);
        let slow_ratio = rotation_response(EARTH_REFERENCE_ROTATION_PERIOD_S * 4.0);
        let fast_ratio = rotation_response(EARTH_REFERENCE_ROTATION_PERIOD_S * 0.5);
        let (earth_hadley, _) = circulation_cell_edges(earth_ratio);
        let (slow_hadley, _) = circulation_cell_edges(slow_ratio);
        let (fast_hadley, _) = circulation_cell_edges(fast_ratio);
        assert!(slow_hadley > earth_hadley);
        assert!(fast_hadley < earth_hadley);
        let earth_omega = TWO_PI / EARTH_REFERENCE_ROTATION_PERIOD_S;
        assert_eq!(coriolis_deflection_factor(0.0, earth_omega, 0.55), 0.0);
        assert!(coriolis_deflection_factor(45_f64.to_radians(), earth_omega, 0.55) > 0.0);
        assert!(coriolis_deflection_factor(-45_f64.to_radians(), earth_omega, 0.55) < 0.0);
        assert!(
            coriolis_deflection_factor(45_f64.to_radians(), earth_omega * 2.0, 0.55).abs()
                > coriolis_deflection_factor(45_f64.to_radians(), earth_omega, 0.55).abs()
        );
    }
'''
assert needle in text
text = text.replace(needle, addition, 1)
climate_path.write_text(text)

test_path = Path('rust/interlink-worldgen/tests/climate_ensemble.rs')
test_text = test_path.read_text()
needle = '    assert!(first.metrics.moisture_budget_relative_error < 1.0e-8);\n'
assert needle in test_text
addition = needle + '''

    let mut no_ocean_heat_request = request.clone();
    no_ocean_heat_request.parameters.ocean_advection_relaxation = 0.0;
    let no_ocean_heat =
        generate_coupled_climate(&topology, &terrain, planet, &no_ocean_heat_request).unwrap();
    assert_ne!(
        first.sea_surface_temperature_mean_k,
        no_ocean_heat.sea_surface_temperature_mean_k,
        "surface-current heat advection must causally affect SST",
    );
'''
test_text = test_text.replace(needle, addition, 1)

test_text += r'''

fn mean_field(values: &[f32]) -> f64 {
    values.iter().map(|value| f64::from(*value)).sum::<f64>() / values.len() as f64
}

fn zonal_fraction(east: &[f32], north: &[f32]) -> f64 {
    let east_total = east.iter().map(|value| f64::from(*value).abs()).sum::<f64>();
    let north_total = north.iter().map(|value| f64::from(*value).abs()).sum::<f64>();
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
    assert_ne!(slow_climate.wind_east_mean_m_s, fast_climate.wind_east_mean_m_s);
    assert_ne!(slow_climate.current_east_mean_m_s, fast_climate.current_east_mean_m_s);
}
'''
test_path.write_text(test_text)
