from pathlib import Path


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


climate_path = "rust/interlink-worldgen/src/climate.rs"
climate = read(climate_path)
climate = replace_once(
    climate,
    'pub const CLIMATE_STAGE_VERSION: u32 = 1;',
    'pub const CLIMATE_STAGE_VERSION: u32 = 2;',
    'climate stage version',
)

climate = replace_once(
    climate,
    'fn daily_mean_insolation(latitude: f64, declination: f64, stellar_flux: f64) -> f64 {\n'
    '    let x = -latitude.tan() * declination.tan();\n'
    '    let hour_angle = if x >= 1.0 {\n'
    '        0.0\n'
    '    } else if x <= -1.0 {\n'
    '        std::f64::consts::PI\n'
    '    } else {\n'
    '        x.acos()\n'
    '    };\n'
    '    let value = stellar_flux / std::f64::consts::PI\n'
    '        * (hour_angle * latitude.sin() * declination.sin()\n'
    '            + latitude.cos() * declination.cos() * hour_angle.sin());\n'
    '    value.max(0.0)\n'
    '}\n',
    'fn daily_mean_insolation(latitude: f64, declination: f64, stellar_flux: f64) -> f64 {\n'
    '    let x = -latitude.tan() * declination.tan();\n'
    '    let hour_angle = if x >= 1.0 {\n'
    '        0.0\n'
    '    } else if x <= -1.0 {\n'
    '        std::f64::consts::PI\n'
    '    } else {\n'
    '        x.acos()\n'
    '    };\n'
    '    let value = stellar_flux / std::f64::consts::PI\n'
    '        * (hour_angle * latitude.sin() * declination.sin()\n'
    '            + latitude.cos() * declination.cos() * hour_angle.sin());\n'
    '    value.max(0.0)\n'
    '}\n\n'
    'fn atmospheric_surface_height_m(submerged: bool, elevation_above_sea_level_m: f64) -> f64 {\n'
    '    if submerged {\n'
    '        0.0\n'
    '    } else {\n'
    '        elevation_above_sea_level_m.max(0.0)\n'
    '    }\n'
    '}\n\n'
    'fn solve_orbital_forcing(\n'
    '    mean_longitude_rad: f64,\n'
    '    eccentricity: f64,\n'
    '    longitude_of_periapsis_rad: f64,\n'
    ') -> (f64, f64) {\n'
    '    let mean_anomaly = (mean_longitude_rad - longitude_of_periapsis_rad).rem_euclid(TWO_PI);\n'
    '    let mut eccentric_anomaly = if eccentricity < 0.8 {\n'
    '        mean_anomaly\n'
    '    } else {\n'
    '        std::f64::consts::PI\n'
    '    };\n'
    '    for _ in 0..16 {\n'
    '        let residual = eccentric_anomaly - eccentricity * eccentric_anomaly.sin() - mean_anomaly;\n'
    '        let derivative = 1.0 - eccentricity * eccentric_anomaly.cos();\n'
    '        let step = residual / derivative.max(1.0e-12);\n'
    '        eccentric_anomaly -= step;\n'
    '        if step.abs() <= 1.0e-13 {\n'
    '            break;\n'
    '        }\n'
    '    }\n'
    '    let half_e = 0.5 * eccentric_anomaly;\n'
    '    let true_anomaly = 2.0\n'
    '        * ((1.0 + eccentricity).sqrt() * half_e.sin())\n'
    '            .atan2((1.0 - eccentricity).sqrt() * half_e.cos());\n'
    '    let solar_longitude = true_anomaly + longitude_of_periapsis_rad;\n'
    '    let radius_over_a = (1.0 - eccentricity * eccentric_anomaly.cos()).max(1.0e-6);\n'
    '    (solar_longitude, 1.0 / (radius_over_a * radius_over_a))\n'
    '}\n',
    'orbital helpers',
)

climate = replace_once(
    climate,
    'fn build_ocean_projection_geometry(\n',
    'fn symmetric_edge_normal_wind_m_s(\n'
    '    topology: &GeodesicTopology,\n'
    '    a: usize,\n'
    '    b: usize,\n'
    '    east_bases: &[[f64; 3]],\n'
    '    north_bases: &[[f64; 3]],\n'
    '    wind_east: &[f64],\n'
    '    wind_north: &[f64],\n'
    ') -> Option<f64> {\n'
    '    let (a_east, a_north) =\n'
    '        edge_direction_components(topology, a, b, east_bases[a], north_bases[a])?;\n'
    '    let (b_east, b_north) =\n'
    '        edge_direction_components(topology, b, a, east_bases[b], north_bases[b])?;\n'
    '    let outward_a = wind_east[a] * a_east + wind_north[a] * a_north;\n'
    '    let outward_b = wind_east[b] * b_east + wind_north[b] * b_north;\n'
    '    Some(0.5 * (outward_a - outward_b))\n'
    '}\n\n'
    'fn build_ocean_projection_geometry(\n',
    'symmetric atmospheric edge velocity helper',
)

climate = replace_once(
    climate,
    '    let specific_gas_constant =\n'
    '        UNIVERSAL_GAS_CONSTANT / physical.atmospheric_mean_molar_mass_kg_per_mol;\n'
    '    let phase_seconds = planet.orbital_period_s / phase_count as f64;\n',
    '    let specific_gas_constant =\n'
    '        UNIVERSAL_GAS_CONSTANT / physical.atmospheric_mean_molar_mass_kg_per_mol;\n'
    '    let atmospheric_heat_capacity_response =\n'
    '        (1_004.0 / physical.atmospheric_specific_heat_j_per_kg_k).clamp(0.25, 4.0);\n'
    '    let phase_seconds = planet.orbital_period_s / phase_count as f64;\n',
    'specific heat causal response',
)

climate = replace_once(
    climate,
    '        terrain_height_m[i] = if ocean[i] {\n'
    '            0.0\n'
    '        } else {\n'
    '            f64::from(terrain.elevation_above_sea_level_m[i]).max(0.0)\n'
    '        };\n',
    '        terrain_height_m[i] = atmospheric_surface_height_m(\n'
    '            ocean[i],\n'
    '            f64::from(terrain.elevation_above_sea_level_m[i]),\n'
    '        );\n',
    'surface height construction',
)
climate = replace_once(
    climate,
    '    let terrain_values = terrain\n'
    '        .elevation_above_sea_level_m\n'
    '        .iter()\n'
    '        .map(|value| f64::from(*value))\n'
    '        .collect::<Vec<_>>();\n',
    '    let terrain_values = terrain_height_m.clone();\n',
    'atmospheric terrain gradient surface',
)

climate = replace_once(
    climate,
    '    let mut wind_north_sin = vec![0.0; sample_count];\n'
    '    let mut sst_sum = vec![0.0; sample_count];\n',
    '    let mut wind_north_sin = vec![0.0; sample_count];\n'
    '    let mut wind_speed_sum = vec![0.0; sample_count];\n'
    '    let mut maximum_wind_speed_over_phases = 0.0_f64;\n'
    '    let mut sst_sum = vec![0.0; sample_count];\n',
    'instantaneous wind accumulators',
)
climate = replace_once(
    climate,
    '    let mut final_divergence_residual = 0.0;\n',
    '    let mut maximum_ocean_divergence_residual = 0.0_f64;\n',
    'divergence accumulator declaration',
)

climate = replace_once(
    climate,
    '        wind_north_cos.fill(0.0);\n'
    '        wind_north_sin.fill(0.0);\n'
    '        sst_sum.fill(0.0);\n',
    '        wind_north_cos.fill(0.0);\n'
    '        wind_north_sin.fill(0.0);\n'
    '        wind_speed_sum.fill(0.0);\n'
    '        maximum_wind_speed_over_phases = 0.0;\n'
    '        sst_sum.fill(0.0);\n',
    'wind accumulator reset',
)
climate = replace_once(
    climate,
    '        moisture_budget_error_year = 0.0;\n\n'
    '        for phase in 0..phase_count {\n'
    '            let orbital_angle = TWO_PI * phase as f64 / phase_count as f64;\n'
    '            let eccentricity = physical.orbital_eccentricity;\n'
    '            let distance_factor = ((1.0\n'
    '                + eccentricity * (orbital_angle - physical.longitude_of_periapsis_rad).cos())\n'
    '                / (1.0 - eccentricity * eccentricity))\n'
    '                .powi(2);\n'
    '            let declination = (planet.axial_tilt_rad.sin() * orbital_angle.sin()).asin();\n'
    '            let phase_angle = orbital_angle;\n',
    '        moisture_budget_error_year = 0.0;\n'
    '        maximum_ocean_divergence_residual = 0.0;\n\n'
    '        for phase in 0..phase_count {\n'
    '            let mean_longitude = TWO_PI * phase as f64 / phase_count as f64;\n'
    '            let (solar_longitude, distance_factor) = solve_orbital_forcing(\n'
    '                mean_longitude,\n'
    '                physical.orbital_eccentricity,\n'
    '                physical.longitude_of_periapsis_rad,\n'
    '            );\n'
    '            let declination =\n'
    '                (planet.axial_tilt_rad.sin() * solar_longitude.sin()).asin();\n'
    '            let phase_angle = mean_longitude;\n',
    'equal-time Kepler orbital forcing',
)

climate = replace_once(
    climate,
    '                let neighbor_temperature = mean_neighbor(topology, &previous_temperature, i);\n'
    '                let transported_target = radiative_target[i]\n'
    '                    + parameters.atmospheric_heat_relaxation\n'
    '                        * (neighbor_temperature - previous_temperature[i]);\n',
    '                let neighbor_temperature = mean_neighbor(topology, &previous_temperature, i);\n'
    '                let atmospheric_transport = if atmosphere_exists {\n'
    '                    parameters.atmospheric_heat_relaxation\n'
    '                        * atmospheric_heat_capacity_response\n'
    '                        * (neighbor_temperature - previous_temperature[i])\n'
    '                } else {\n'
    '                    0.0\n'
    '                };\n'
    '                let transported_target = radiative_target[i] + atmospheric_transport;\n',
    'airless atmospheric transport and specific heat',
)

climate = replace_once(
    climate,
    '                final_divergence_residual = correct_ocean_currents(\n'
    '                    &ocean,\n'
    '                    &ocean_projection_geometry,\n'
    '                    &mut current_east,\n'
    '                    &mut current_north,\n'
    '                    &mut ocean_edge_transport_m2_s,\n'
    '                    &parameters,\n'
    '                );\n',
    '                let phase_divergence_residual = correct_ocean_currents(\n'
    '                    &ocean,\n'
    '                    &ocean_projection_geometry,\n'
    '                    &mut current_east,\n'
    '                    &mut current_north,\n'
    '                    &mut ocean_edge_transport_m2_s,\n'
    '                    &parameters,\n'
    '                );\n'
    '                maximum_ocean_divergence_residual =\n'
    '                    maximum_ocean_divergence_residual.max(phase_divergence_residual);\n',
    'maximum seasonal ocean divergence',
)

old_moisture_projection = '''                        let position = topology.positions()[j];
                        let radial = dot(position, origin);
                        let tangent = [
                            position[0] - origin[0] * radial,
                            position[1] - origin[1] * radial,
                            position[2] - origin[2] * radial,
                        ];
                        let tangent_norm = dot(tangent, tangent).sqrt();
                        if tangent_norm <= 1.0e-15 {
                            continue;
                        }
                        let direction = [
                            tangent[0] / tangent_norm,
                            tangent[1] / tangent_norm,
                            tangent[2] / tangent_norm,
                        ];
                        let projected = wind_east[i] * dot(direction, east_bases[i])
                            + wind_north[i] * dot(direction, north_bases[i]);
'''
new_moisture_projection = '''                        let Some(projected) = symmetric_edge_normal_wind_m_s(
                            topology,
                            i,
                            j,
                            &east_bases,
                            &north_bases,
                            &wind_east,
                            &wind_north,
                        ) else {
                            continue;
                        };
'''
climate = replace_once(
    climate,
    old_moisture_projection,
    new_moisture_projection,
    'symmetric moisture edge transport',
)
climate = replace_once(
    climate,
    '                for i in 0..sample_count {\n'
    '                    let origin = topology.positions()[i];\n'
    '                    for (neighbor_index, arc) in topology\n',
    '                for i in 0..sample_count {\n'
    '                    for (neighbor_index, arc) in topology\n',
    'remove obsolete moisture origin',
)

climate = replace_once(
    climate,
    '                wind_north_cos[i] += wind_north[i] * phase_cos;\n'
    '                wind_north_sin[i] += wind_north[i] * phase_sin;\n'
    '                sst_sum[i] += sea_surface_temperature[i];\n',
    '                wind_north_cos[i] += wind_north[i] * phase_cos;\n'
    '                wind_north_sin[i] += wind_north[i] * phase_sin;\n'
    '                let phase_wind_speed = norm2(wind_east[i], wind_north[i]);\n'
    '                wind_speed_sum[i] += phase_wind_speed;\n'
    '                maximum_wind_speed_over_phases =\n'
    '                    maximum_wind_speed_over_phases.max(phase_wind_speed);\n'
    '                sst_sum[i] += sea_surface_temperature[i];\n',
    'seasonal wind speed accumulation',
)

climate = replace_once(
    climate,
    '    let phase_count_f64 = phase_count as f64;\n',
    '    if final_temperature_rms_change > parameters.convergence_temperature_rms_k {\n'
    '        return Err(WorldgenError::InvalidClimate(\n'
    '            "WG-5 climate did not converge within the configured spin-up bound",\n'
    '        ));\n'
    '    }\n\n'
    '    let phase_count_f64 = phase_count as f64;\n',
    'core convergence rejection',
)

climate = replace_once(
    climate,
    '    let mut wind_speed_values = vec![0.0_f32; sample_count];\n'
    '    for i in 0..sample_count {\n'
    '        wind_speed_values[i] =\n'
    '            norm2(f64::from(wind_east_mean[i]), f64::from(wind_north_mean[i])) as f32;\n'
    '    }\n'
    '    let mean_wind_speed = area_weighted_mean(topology, &wind_speed_values);\n'
    '    let maximum_wind_speed = wind_speed_values.iter().copied().fold(0.0_f32, f32::max) as f64;\n',
    '    let wind_speed_mean = wind_speed_sum\n'
    '        .iter()\n'
    '        .map(|value| (value / phase_count_f64) as f32)\n'
    '        .collect::<Vec<_>>();\n'
    '    let mean_wind_speed = area_weighted_mean(topology, &wind_speed_mean);\n'
    '    let maximum_wind_speed = maximum_wind_speed_over_phases;\n',
    'wind metrics semantics',
)

climate = replace_once(
    climate,
    '    let moisture_budget_relative_error = moisture_budget_error_year / total_budget_scale;\n'
    '    let total_area = topology.metrics().total_area_steradians.max(1.0e-12);\n',
    '    let moisture_budget_relative_error = moisture_budget_error_year / total_budget_scale;\n'
    '    if moisture_budget_relative_error > 1.0e-8 {\n'
    '        return Err(WorldgenError::InvalidClimate(\n'
    '            "WG-5 atmospheric moisture budget did not close within tolerance",\n'
    '        ));\n'
    '    }\n'
    '    let total_area = topology.metrics().total_area_steradians.max(1.0e-12);\n',
    'core moisture closure rejection',
)

old_hash = '''    let mut climate_hash = FNV_OFFSET_BASIS;
    climate_hash = fnv_update(climate_hash, &stage_seed.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &terrain.metrics.topography_hash.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &planet.parameter_hash().to_le_bytes());
    climate_hash = fnv_update(climate_hash, &physical_hash.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &model_hash.to_le_bytes());
    climate_hash = hash_f32_slice(climate_hash, &temperature_mean);
    climate_hash = hash_f32_slice(climate_hash, &wind_east_mean);
    climate_hash = hash_f32_slice(climate_hash, &wind_north_mean);
    climate_hash = hash_f32_slice(climate_hash, &sst_mean);
    climate_hash = hash_f32_slice(climate_hash, &sst_cos_out);
    climate_hash = hash_f32_slice(climate_hash, &sst_sin_out);
    climate_hash = hash_f32_slice(climate_hash, &current_east_mean);
    climate_hash = hash_f32_slice(climate_hash, &current_north_mean);
    climate_hash = hash_f32_slice(climate_hash, &current_east_cos_out);
    climate_hash = hash_f32_slice(climate_hash, &current_east_sin_out);
    climate_hash = hash_f32_slice(climate_hash, &current_north_cos_out);
    climate_hash = hash_f32_slice(climate_hash, &current_north_sin_out);
    climate_hash = hash_f32_slice(climate_hash, &annual_precipitation_mm);
    climate_hash = hash_f32_slice(climate_hash, &aridity_index);
'''
new_hash = '''    let mut climate_hash = FNV_OFFSET_BASIS;
    climate_hash = fnv_update(climate_hash, CLIMATE_STAGE_ID.as_bytes());
    climate_hash = fnv_update(climate_hash, &CLIMATE_STAGE_VERSION.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &stage_seed.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &terrain.metrics.topography_hash.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &planet.parameter_hash().to_le_bytes());
    climate_hash = fnv_update(climate_hash, &physical_hash.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &model_hash.to_le_bytes());
    for values in [
        &annual_mean_insolation_f32,
        &seasonal_insolation_amplitude,
        &temperature_mean,
        &temperature_annual_cos,
        &temperature_annual_sin,
        &temperature_min_f32,
        &temperature_max_f32,
        &pressure_f32,
        &wind_east_mean,
        &wind_north_mean,
        &wind_east_cos_out,
        &wind_east_sin_out,
        &wind_north_cos_out,
        &wind_north_sin_out,
        &sst_mean,
        &sst_cos_out,
        &sst_sin_out,
        &current_east_mean,
        &current_north_mean,
        &current_east_cos_out,
        &current_east_sin_out,
        &current_north_cos_out,
        &current_north_sin_out,
        &current_speed_mean,
        &ocean_heat_transport,
        &humidity_mean,
        &annual_precipitation_mm,
        &precipitation_seasonality,
        &potential_evaporation_mm,
        &moisture_balance_mm,
        &aridity_index,
        &snowfall_fraction,
        &persistent_snow_potential,
        &sea_ice_potential,
    ] {
        climate_hash = hash_f32_slice(climate_hash, values);
    }
'''
climate = replace_once(climate, old_hash, new_hash, 'complete climate state hash')
climate = replace_once(
    climate,
    '        ocean_divergence_residual_m_s: final_divergence_residual,\n',
    '        ocean_divergence_residual_m_s: maximum_ocean_divergence_residual,\n',
    'maximum divergence metric',
)

unit_tests = r'''

    #[test]
    fn atmospheric_surface_ignores_submerged_relief() {
        assert_eq!(atmospheric_surface_height_m(true, -8_000.0), 0.0);
        assert_eq!(atmospheric_surface_height_m(true, -50.0), 0.0);
        assert_eq!(atmospheric_surface_height_m(false, 1_250.0), 1_250.0);
        assert_eq!(atmospheric_surface_height_m(false, -2.0), 0.0);
    }

    #[test]
    fn eccentric_orbit_uses_equal_time_kepler_geometry() {
        let e = 0.7;
        let periapsis = 0.9;
        let (_, peri_flux) = solve_orbital_forcing(periapsis, e, periapsis);
        let (_, apo_flux) = solve_orbital_forcing(periapsis + std::f64::consts::PI, e, periapsis);
        assert!((peri_flux - 1.0 / (1.0 - e).powi(2)).abs() < 1.0e-10);
        assert!((apo_flux - 1.0 / (1.0 + e).powi(2)).abs() < 1.0e-10);
        assert!(peri_flux > apo_flux);
        let (longitude, flux) = solve_orbital_forcing(2.1, 0.94, periapsis);
        assert!(longitude.is_finite());
        assert!(flux.is_finite() && flux > 0.0);
    }

    #[test]
    fn atmospheric_face_velocity_is_orientation_invariant() {
        let topology = crate::build_icosphere(1).unwrap();
        let count = topology.positions().len();
        let mut east_bases = Vec::with_capacity(count);
        let mut north_bases = Vec::with_capacity(count);
        for position in topology.positions() {
            let basis = crate::tangent_basis(*position).unwrap();
            east_bases.push(basis.east);
            north_bases.push(basis.north);
        }
        let a = 0usize;
        let b = topology.neighbors_of(a as u32)[0] as usize;
        let mut east = vec![0.0; count];
        let mut north = vec![0.0; count];
        east[a] = 7.0;
        north[a] = -2.0;
        east[b] = -4.0;
        north[b] = 3.5;
        let forward = symmetric_edge_normal_wind_m_s(
            &topology,
            a,
            b,
            &east_bases,
            &north_bases,
            &east,
            &north,
        )
        .unwrap();
        let reverse = symmetric_edge_normal_wind_m_s(
            &topology,
            b,
            a,
            &east_bases,
            &north_bases,
            &east,
            &north,
        )
        .unwrap();
        assert!((forward + reverse).abs() < 1.0e-12);
    }
'''
idx = climate.rfind('\n}')
if idx < 0:
    raise RuntimeError('climate test module terminator not found')
climate = climate[:idx] + unit_tests + climate[idx:]
write(climate_path, climate)

ensemble_path = "rust/interlink-worldgen/tests/climate_ensemble.rs"
ensemble = read(ensemble_path)
ensemble = replace_once(
    ensemble,
    '''    assert!(climate.temperature_mean_k.iter().all(|value| value.is_finite()));
    assert!(climate.wind_east_mean_m_s.iter().all(|value| *value == 0.0));
    assert!(climate.wind_north_mean_m_s.iter().all(|value| *value == 0.0));
    assert!(climate.annual_precipitation_mm.iter().all(|value| *value == 0.0));
}
''',
    '''    assert!(climate.temperature_mean_k.iter().all(|value| value.is_finite()));
    assert!(climate.wind_east_mean_m_s.iter().all(|value| *value == 0.0));
    assert!(climate.wind_north_mean_m_s.iter().all(|value| *value == 0.0));
    assert!(climate.annual_precipitation_mm.iter().all(|value| *value == 0.0));

    let mut no_transport_request = ClimateRequest::new("wg5-airless");
    no_transport_request.parameters.atmospheric_heat_relaxation = 0.0;
    let no_transport = generate_coupled_climate(
        &topology,
        &terrain,
        airless,
        &no_transport_request,
    )
    .unwrap();
    assert_eq!(
        climate.temperature_mean_k, no_transport.temperature_mean_k,
        "airless surfaces must not retain atmospheric lateral heat redistribution",
    );
}
''',
    'airless causal validation',
)
ensemble += r'''

#[test]
fn atmospheric_specific_heat_changes_thermal_redistribution_on_fixed_wg4_surface() {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let (topology, terrain) = generated_surface("wg5-specific-heat", planet);
    let reference = ClimateRequest::new("wg5-specific-heat");
    let normal = generate_coupled_climate(&topology, &terrain, planet, &reference).unwrap();
    let mut high_heat_capacity = reference.clone();
    high_heat_capacity.physical.atmospheric_specific_heat_j_per_kg_k *= 2.0;
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
'''
write(ensemble_path, ensemble)

math_path = Path("src/worldgen/diagnostics/worldgenClimateMath.ts")
math_path.write_text('''export function reconstructAnnualHarmonicFromBasis(\n  mean: number,\n  cosine: number,\n  sine: number,\n  phaseCosine: number,\n  phaseSine: number,\n): number {\n  return mean + cosine * phaseCosine + sine * phaseSine;\n}\n\nexport function reconstructAnnualHarmonic(\n  mean: number,\n  cosine: number,\n  sine: number,\n  phase: number,\n): number {\n  const angle = phase * Math.PI * 2;\n  return reconstructAnnualHarmonicFromBasis(mean, cosine, sine, Math.cos(angle), Math.sin(angle));\n}\n\nexport function mapVectorDelta(\n  eastValue: number,\n  northValue: number,\n  latitudeRad: number,\n  width: number,\n  height: number,\n): [number, number] {\n  const speed = Math.hypot(eastValue, northValue);\n  if (speed < 1e-9) return [0, 0];\n  const cosLat = Math.max(0.18, Math.cos(latitudeRad));\n  return [\n    (eastValue / speed) * width * 0.014 / cosLat,\n    -(northValue / speed) * height * 0.026,\n  ];\n}\n''')

lab_path = "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts"
lab = read(lab_path)
lab = replace_once(
    lab,
    "import { createWorldgenClient } from '../worldgenClient.js';\n",
    "import { createWorldgenClient } from '../worldgenClient.js';\nimport { mapVectorDelta, reconstructAnnualHarmonicFromBasis } from './worldgenClimateMath.js';\n",
    'climate math imports',
)
lab = replace_once(
    lab,
    '''function seasonalValue(mean: number, cosine: number, sine: number, phase: number): number {
  const angle = phase * TWO_PI;
  return mean + cosine * Math.cos(angle) + sine * Math.sin(angle);
}
''',
    '''function seasonalValue(mean: number, cosine: number, sine: number, phase: number): number {
  const angle = phase * TWO_PI;
  return reconstructAnnualHarmonicFromBasis(mean, cosine, sine, Math.cos(angle), Math.sin(angle));
}
''',
    'seasonal scalar helper',
)
lab = replace_once(
    lab,
    '  for (let index = 0; index < scratch.length; index += 1) scratch[index] = mean[index]! + cosine[index]! * c + sine[index]! * s;\n',
    '  for (let index = 0; index < scratch.length; index += 1) scratch[index] = reconstructAnnualHarmonicFromBasis(mean[index]!, cosine[index]!, sine[index]!, c, s);\n',
    'seasonal array helper',
)
lab = replace_once(
    lab,
    '''  if (projection === 'map') {
    const cosLat = Math.max(0.18, Math.cos(lat));
    return [tangent[0] * width * 0.014 / cosLat, -tangent[2] * height * 0.026];
  }
''',
    '''  if (projection === 'map') return mapVectorDelta(eastValue, northValue, lat, width, height);
''',
    'flat map ENU vector projection',
)
write(lab_path, lab)

browser_test_path = "tests/wg5Climate.test.ts"
browser_test = read(browser_test_path)
browser_test = replace_once(
    browser_test,
    "} from '../dist/worldgen/protocol.js';\n",
    "} from '../dist/worldgen/protocol.js';\nimport { mapVectorDelta, reconstructAnnualHarmonic } from '../dist/worldgen/diagnostics/worldgenClimateMath.js';\n",
    'browser math test import',
)
browser_test += r'''

test('WG-5 seasonal reconstruction is numerically defined by stored annual harmonics', () => {
  assert.equal(reconstructAnnualHarmonic(10, 2, 3, 0), 12);
  assert.ok(Math.abs(reconstructAnnualHarmonic(10, 2, 3, 0.25) - 13) < 1e-12);
  assert.ok(Math.abs(reconstructAnnualHarmonic(10, 2, 3, 0.5) - 8) < 1e-12);
});

test('WG-5 flat-map vector projection uses local ENU components directly', () => {
  const [eastDx, eastDy] = mapVectorDelta(8, 0, 0, 1100, 550);
  assert.ok(eastDx > 0);
  assert.ok(Math.abs(eastDy) < 1e-12);
  const [northDx, northDy] = mapVectorDelta(0, 8, 0, 1100, 550);
  assert.ok(Math.abs(northDx) < 1e-12);
  assert.ok(northDy < 0);
  const [eastAtMidLatDx] = mapVectorDelta(8, 0, Math.PI / 4, 1100, 550);
  assert.ok(eastAtMidLatDx > eastDx, 'equirectangular longitude scale should expand by 1/cos(latitude)');
});

test('WG-5 climate hash source covers every public climate output vector', () => {
  const source = fs.readFileSync('rust/interlink-worldgen/src/climate.rs', 'utf8');
  for (const field of [
    'annual_mean_insolation_f32', 'seasonal_insolation_amplitude',
    'temperature_mean', 'temperature_annual_cos', 'temperature_annual_sin', 'temperature_min_f32', 'temperature_max_f32',
    'pressure_f32', 'wind_east_mean', 'wind_north_mean', 'wind_east_cos_out', 'wind_east_sin_out', 'wind_north_cos_out', 'wind_north_sin_out',
    'sst_mean', 'sst_cos_out', 'sst_sin_out', 'current_east_mean', 'current_north_mean',
    'current_east_cos_out', 'current_east_sin_out', 'current_north_cos_out', 'current_north_sin_out',
    'current_speed_mean', 'ocean_heat_transport', 'humidity_mean', 'annual_precipitation_mm',
    'precipitation_seasonality', 'potential_evaporation_mm', 'moisture_balance_mm', 'aridity_index',
    'snowfall_fraction', 'persistent_snow_potential', 'sea_ice_potential',
  ]) assert.match(source, new RegExp(`&${field}[,\\n]`));
});
'''
write(browser_test_path, browser_test)

ci_path = ".github/workflows/ci.yml"
ci = read(ci_path)
ci = replace_once(
    ci,
    '''      - name: Compile browser bridge
        run: cargo check -p interlink-worldgen-wasm --target wasm32-unknown-unknown
''',
    '''      - name: WG-5 L7 convergence and conservation acceptance
        run: cargo run --release -p interlink-worldgen-cli -- climate --seed ci-wg5-l7 --coarse-level 5 --level 7 --plates 24
      - name: Compile browser bridge
        run: cargo check -p interlink-worldgen-wasm --target wasm32-unknown-unknown
''',
    'permanent L7 CI gate',
)
write(ci_path, ci)

doc_path = "docs/worldgen-rewrite/WG5_CLIMATE.md"
doc = read(doc_path)
doc = replace_once(
    doc,
    'WG-5 converts the accepted WG-4 physical surface into a deterministic climatology. It is a generation-time physical solve, not a perpetual post-generation weather simulation.\n',
    'WG-5 converts the accepted WG-4 physical surface into a deterministic climatology. It is a generation-time physical solve, not a perpetual post-generation weather simulation. The corrected climate algorithm is stage version `2`; version `2` tightens orbital, atmospheric-transport, acceptance, diagnostic, and state-identity semantics without changing the browser protocol shape.\n',
    'WG5 stage documentation',
)
doc = replace_once(
    doc,
    'Atmospheric moisture transport likewise scales aggregate outgoing graph transfers to the donor water mass before applying paired transfers, preserving moisture mass instead of relying on post-transport zero clamps.\n',
    'Atmospheric moisture transport likewise scales aggregate outgoing graph transfers to the donor water mass before applying paired transfers, preserving moisture mass instead of relying on post-transport zero clamps. Each undirected atmospheric interface uses a symmetric face-normal velocity reconstructed from both endpoint winds, so moisture routing is independent of which endpoint has the lower mesh sample index. Atmospheric terrain gradients are taken from the exposed land/sea surface: submerged bathymetry is not treated as an atmospheric obstacle or orographic precipitation source.\n',
    'WG5 moisture documentation',
)
doc = replace_once(
    doc,
    'The default solver evaluates 24 orbital phases. Those phases exist only while generating the climatology. The final state stores annual means, extrema, seasonality, and first annual harmonics that can reconstruct seasonal diagnostic views without rerunning WG-5.\n',
    'The default solver evaluates 24 equal-time orbital phases. For eccentric orbits each phase is interpreted as mean longitude, Kepler\'s equation is solved for eccentric anomaly, and the resulting true solar longitude and orbital distance drive declination and stellar-flux scaling. Those phases exist only while generating the climatology. The final state stores annual means, extrema, seasonality, and first annual harmonics that can reconstruct seasonal diagnostic views without rerunning WG-5.\n',
    'WG5 orbital documentation',
)
doc += '''\n## Acceptance and diagnostics hardening\n\nThe public WG-5 generator now rejects a climate state if the configured annual temperature convergence tolerance is not reached or if the final atmospheric moisture budget exceeds the conservation tolerance. Native CLI, Rust callers, WASM, and browser generation therefore share the same acceptance contract.\n\n`atmospheric_specific_heat_j_per_kg_k` now causally scales reduced atmospheric heat redistribution rather than acting only as hash metadata. Airless planets disable that atmospheric redistribution path entirely. Reported wind mean/max statistics are time-aware speed statistics over the retained final climatology year rather than magnitudes of annual-mean vector components, and the reported ocean divergence residual is the worst orbital-phase residual from the retained year.\n\nThe climate hash covers the full public climate output-vector state as well as stage identity, upstream topography identity, planet parameters, and climate parameter hashes. Permanent CI includes an optimized L7 WG-5 convergence/conservation run in addition to the lower-resolution smoke test.\n'''
write(doc_path, doc)

print('WG-5 follow-up review hardening patch applied')
