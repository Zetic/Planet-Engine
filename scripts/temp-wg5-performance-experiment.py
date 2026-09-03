from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''    let mut terrain_gradient_east = vec![0.0; sample_count];
    let mut terrain_gradient_north = vec![0.0; sample_count];
    for i in 0..sample_count {
        let (east, north) = scalar_gradient(
            topology,
            &terrain_height_m,
            planet.radius_m,
            i,
            east_bases[i],
            north_bases[i],
        );
        terrain_gradient_east[i] = east;
        terrain_gradient_north[i] = north;
    }

    let atmospheric_heat_geometry = build_atmospheric_heat_geometry(topology, planet.radius_m);
''',
    '''    let mut terrain_gradient_east = vec![0.0; sample_count];
    let mut terrain_gradient_north = vec![0.0; sample_count];
    for i in 0..sample_count {
        let (east, north) = scalar_gradient(
            topology,
            &terrain_height_m,
            planet.radius_m,
            i,
            east_bases[i],
            north_bases[i],
        );
        terrain_gradient_east[i] = east;
        terrain_gradient_north[i] = north;
    }

    // These circulation factors depend only on immutable planet/topography
    // state. WG-5 otherwise recalculates them for every cell in every orbital
    // phase of every spin-up year.
    let mut wind_rotational_blend = vec![0.0; sample_count];
    let mut wind_zonal_base = vec![0.0; sample_count];
    let mut wind_meridional_base = vec![0.0; sample_count];
    let mut wind_topographic_drag = vec![0.0; sample_count];
    let mut ocean_coriolis_deflection = vec![0.0; sample_count];
    let mut ocean_bathymetric_mobility = vec![0.0; sample_count];
    for i in 0..sample_count {
        let latitude_abs_deg = latitude[i].abs().to_degrees();
        wind_rotational_blend[i] = ((latitude_abs_deg - rotational_transition_start_deg)
            / rotational_transition_width_deg)
            .clamp(0.0, 1.0);
        wind_zonal_base[i] = baseline_zonal_wind(latitude[i], rotation_ratio);
        wind_meridional_base[i] = if latitude_abs_deg < hadley_edge_deg {
            -latitude[i].signum()
                * 2.6
                * overturning_strength
                * (1.0 - latitude_abs_deg / hadley_edge_deg)
        } else {
            0.0
        };
        let slope = norm2(terrain_gradient_east[i], terrain_gradient_north[i]);
        wind_topographic_drag[i] = 1.0 / (1.0 + parameters.topographic_wind_drag * slope);
        if ocean[i] {
            ocean_coriolis_deflection[i] = coriolis_deflection_factor(
                latitude[i],
                omega,
                parameters.ocean_coriolis_deflection,
            );
            ocean_bathymetric_mobility[i] =
                (water_depth_m[i] / parameters.ocean_bathymetric_drag_depth_m)
                    .clamp(0.08, 1.0)
                    .sqrt();
        }
    }

    let atmospheric_heat_geometry = build_atmospheric_heat_geometry(topology, planet.radius_m);
''',
    'insert phase-invariant circulation cache',
)

replace_once(
    '''                    let latitude_abs_deg = latitude[i].abs().to_degrees();
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
                    let slope = norm2(terrain_gradient_east[i], terrain_gradient_north[i]);
                    let drag = 1.0 / (1.0 + parameters.topographic_wind_drag * slope);
                    let east = (zonal + rotational_blend * geostrophic_east) * drag;
                    let north = (meridional + rotational_blend * geostrophic_north) * drag;
''',
    '''                    let geostrophic_east = -gradient_north
                        * 1_000_000.0
                        * parameters.wind_thermal_gradient_scale
                        * rotational_strength;
                    let geostrophic_north = gradient_east
                        * 1_000_000.0
                        * parameters.wind_thermal_gradient_scale
                        * rotational_strength;
                    let east = (wind_zonal_base[i]
                        + wind_rotational_blend[i] * geostrophic_east)
                        * wind_topographic_drag[i];
                    let north = (wind_meridional_base[i]
                        + wind_rotational_blend[i] * geostrophic_north)
                        * wind_topographic_drag[i];
''',
    'reuse cached wind circulation factors',
)

replace_once(
    '''                    let deflection = coriolis_deflection_factor(
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
''',
    '''                    let deflection = ocean_coriolis_deflection[i];
                    let mobility = ocean_bathymetric_mobility[i];
                    let east = parameters.ocean_wind_coupling
                        * mobility
                        * (wind_east[i] + deflection * wind_north[i]);
                    let north = parameters.ocean_wind_coupling
                        * mobility
                        * (wind_north[i] - deflection * wind_east[i]);
''',
    'reuse cached ocean circulation factors',
)

path.write_text(text)
