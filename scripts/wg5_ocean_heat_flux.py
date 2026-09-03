from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

needle = '''fn mean_neighbor(topology: &GeodesicTopology, values: &[f64], sample: usize) -> f64 {
    let neighbors = topology.neighbors_of(sample as u32);
    if neighbors.is_empty() {
        return values[sample];
    }
    neighbors
        .iter()
        .map(|neighbor| values[*neighbor as usize])
        .sum::<f64>()
        / neighbors.len() as f64
}
'''
addition = needle + '''
fn mean_ocean_neighbor(
    topology: &GeodesicTopology,
    ocean: &[bool],
    values: &[f64],
    sample: usize,
) -> f64 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for neighbor in topology.neighbors_of(sample as u32) {
        let index = *neighbor as usize;
        if ocean[index] {
            sum += values[index];
            count += 1;
        }
    }
    if count > 0 { sum / count as f64 } else { values[sample] }
}
'''
assert needle in text
text = text.replace(needle, addition, 1)

start = text.index('fn correct_ocean_currents(')
end = text.index('\nfn validate_inputs(', start)
block = text[start:end]
old_sig = '''fn correct_ocean_currents(
    ocean: &[bool],
    geometry: &OceanProjectionGeometry,
    current_east: &mut [f64],
    current_north: &mut [f64],
    parameters: &ClimateParameters,
) -> f64 {'''
new_sig = '''fn correct_ocean_currents(
    ocean: &[bool],
    geometry: &OceanProjectionGeometry,
    current_east: &mut [f64],
    current_north: &mut [f64],
    projected_edge_transport_m2_s: &mut [f64],
    parameters: &ClimateParameters,
) -> f64 {
    debug_assert_eq!(projected_edge_transport_m2_s.len(), geometry.edges.len());
    projected_edge_transport_m2_s.fill(0.0);'''
assert old_sig in block
block = block.replace(old_sig, new_sig, 1)
block = block.replace('    let mut edge_flux = vec![0.0; geometry.edges.len()];\n', '')
block = block.replace('edge_flux[', 'projected_edge_transport_m2_s[')
block = block.replace('// Reconstruct the best-fit local ENU display/advection vector from the\n', '// Reconstruct the best-fit local ENU display/diagnostic vector from the\n')
text = text[:start] + block + text[end:]

marker = '''fn validate_inputs(
'''
helper = '''fn conservative_ocean_heat_tendency(
    geometry: &OceanProjectionGeometry,
    edge_transport_m2_s: &[f64],
    temperature_k: &[f64],
    cell_area_m2: &[f64],
    output_k_s: &mut [f64],
) {
    debug_assert_eq!(edge_transport_m2_s.len(), geometry.edges.len());
    output_k_s.fill(0.0);
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let transport = edge_transport_m2_s[edge_index];
        if transport.abs() <= 1.0e-18 {
            continue;
        }
        let upstream = if transport >= 0.0 { edge.a } else { edge.b };
        let advected_anomaly_k = temperature_k[upstream] - 273.15;
        let heat_transport = transport * advected_anomaly_k;
        output_k_s[edge.a] -= heat_transport / cell_area_m2[edge.a].max(1.0);
        output_k_s[edge.b] += heat_transport / cell_area_m2[edge.b].max(1.0);
    }
}

'''
assert marker in text
text = text.replace(marker, helper + marker, 1)

needle = '''    let ocean_projection_geometry = build_ocean_projection_geometry(
        topology,
        &ocean,
        &east_bases,
        &north_bases,
        planet.radius_m,
    );

    let mut temperature = vec![0.0; sample_count];
'''
replacement = '''    let ocean_projection_geometry = build_ocean_projection_geometry(
        topology,
        &ocean,
        &east_bases,
        &north_bases,
        planet.radius_m,
    );
    let mut ocean_edge_transport_m2_s = vec![0.0; ocean_projection_geometry.edges.len()];
    let mut ocean_heat_tendency_k_s = vec![0.0; sample_count];

    let mut temperature = vec![0.0; sample_count];
'''
assert needle in text
text = text.replace(needle, replacement, 1)

needle = '''            current_east.fill(0.0);
            current_north.fill(0.0);
            if planet.surface_water_mass_kg > 0.0 {
'''
replacement = '''            current_east.fill(0.0);
            current_north.fill(0.0);
            ocean_edge_transport_m2_s.fill(0.0);
            ocean_heat_tendency_k_s.fill(0.0);
            if planet.surface_water_mass_kg > 0.0 {
'''
assert needle in text
text = text.replace(needle, replacement, 1)

needle = '''                    &mut current_east,
                    &mut current_north,
                    &parameters,
                );
'''
replacement = '''                    &mut current_east,
                    &mut current_north,
                    &mut ocean_edge_transport_m2_s,
                    &parameters,
                );
'''
assert needle in text
text = text.replace(needle, replacement, 1)

old = '''            let previous_sst = sea_surface_temperature.clone();
            let mut next_sst = previous_sst.clone();
            for i in 0..sample_count {
                if !ocean[i] {
                    next_sst[i] = temperature[i];
                    continue;
                }
                let (sst_gradient_east, sst_gradient_north) = scalar_gradient(
                    topology,
                    &previous_sst,
                    planet.radius_m,
                    i,
                    east_bases[i],
                    north_bases[i],
                );
                let advection_k_s = -(current_east[i] * sst_gradient_east
                    + current_north[i] * sst_gradient_north);
                let advection_delta = (advection_k_s
                    * phase_seconds
                    * parameters.ocean_advection_relaxation)
                    .clamp(-4.0, 4.0);
                let neighbor_sst = mean_neighbor(topology, &previous_sst, i);
                next_sst[i] = (previous_sst[i]
                    + advection_delta
                    + parameters.ocean_temperature_diffusion * (neighbor_sst - previous_sst[i])
                    + parameters.air_sea_exchange_relaxation
                        * (temperature[i] - previous_sst[i]))
                    .clamp(260.0, 325.0);
            }
'''
new = '''            let previous_sst = sea_surface_temperature.clone();
            conservative_ocean_heat_tendency(
                &ocean_projection_geometry,
                &ocean_edge_transport_m2_s,
                &previous_sst,
                &cell_area_m2,
                &mut ocean_heat_tendency_k_s,
            );
            let mut next_sst = previous_sst.clone();
            for i in 0..sample_count {
                if !ocean[i] {
                    next_sst[i] = temperature[i];
                    continue;
                }
                let advection_delta = (ocean_heat_tendency_k_s[i]
                    * phase_seconds
                    * parameters.ocean_advection_relaxation)
                    .clamp(-4.0, 4.0);
                let neighbor_sst = mean_ocean_neighbor(topology, &ocean, &previous_sst, i);
                next_sst[i] = (previous_sst[i]
                    + advection_delta
                    + parameters.ocean_temperature_diffusion * (neighbor_sst - previous_sst[i])
                    + parameters.air_sea_exchange_relaxation
                        * (temperature[i] - previous_sst[i]))
                    .clamp(260.0, 325.0);
            }
'''
assert old in text
text = text.replace(old, new, 1)

old = '''                let (sst_gradient_east, sst_gradient_north) = scalar_gradient(
                    topology,
                    &sea_surface_temperature,
                    planet.radius_m,
                    i,
                    east_bases[i],
                    north_bases[i],
                );
                ocean_heat_transport_sum[i] += if ocean[i] {
                    -(current_east[i] * sst_gradient_east
                        + current_north[i] * sst_gradient_north)
                        * 1_000_000.0
                } else {
                    0.0
                };
'''
new = '''                ocean_heat_transport_sum[i] += if ocean[i] {
                    ocean_heat_tendency_k_s[i] * 1_000_000.0
                } else {
                    0.0
                };
'''
assert old in text
text = text.replace(old, new, 1)

needle = '''    #[test]
    fn rotation_response_broadens_slow_hadley_cells_and_zeroes_equatorial_coriolis() {
'''
assert needle in text
unit = '''    #[test]
    fn conservative_edge_heat_transport_preserves_area_weighted_heat_anomaly() {
        let geometry = OceanProjectionGeometry {
            edges: vec![OceanProjectionEdge {
                a: 0,
                b: 1,
                a_east: 1.0,
                a_north: 0.0,
                b_east: -1.0,
                b_north: 0.0,
                interface_length_m: 10.0,
                conductance: 1.0,
            }],
            diagonal: vec![1.0, 1.0],
        };
        let mut tendency = vec![0.0; 2];
        conservative_ocean_heat_tendency(
            &geometry,
            &[20.0],
            &[300.0, 280.0],
            &[100.0, 200.0],
            &mut tendency,
        );
        let weighted = tendency[0] * 100.0 + tendency[1] * 200.0;
        assert!(weighted.abs() < 1.0e-12);
        assert!(tendency[0] < 0.0);
        assert!(tendency[1] > 0.0);
    }

'''
text = text.replace(needle, unit + needle, 1)
path.write_text(text)
