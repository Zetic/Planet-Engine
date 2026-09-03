from pathlib import Path

climate = Path('rust/interlink-worldgen/src/climate.rs')
text = climate.read_text()

# Make the explicit ocean heat step stable at every supported resolution while
# retaining ocean_advection_relaxation as the model-strength control.
text = text.replace(
    '    pub ocean_advection_relaxation: f64,\n',
    '    pub ocean_advection_relaxation: f64,\n    pub ocean_advection_cfl_limit: f64,\n',
    1,
)
text = text.replace(
    '            ocean_advection_relaxation: 0.025,\n',
    '            ocean_advection_relaxation: 0.025,\n            ocean_advection_cfl_limit: 0.45,\n',
    1,
)
text = text.replace(
    '            self.maximum_surface_current_m_s,\n            self.evaporation_relaxation,\n',
    '            self.maximum_surface_current_m_s,\n            self.ocean_advection_cfl_limit,\n            self.evaporation_relaxation,\n',
    1,
)
text = text.replace(
    '            self.ocean_advection_relaxation,\n            self.condensation_relative_humidity,\n',
    '            self.ocean_advection_relaxation,\n            self.ocean_advection_cfl_limit,\n            self.condensation_relative_humidity,\n',
    1,
)
text = text.replace(
    '            self.ocean_advection_relaxation,\n            self.evaporation_relaxation,\n',
    '            self.ocean_advection_relaxation,\n            self.ocean_advection_cfl_limit,\n            self.evaporation_relaxation,\n',
    1,
)

start = text.index('fn conservative_ocean_heat_tendency(')
end = text.index('\nfn validate_inputs(', start)
text = text[:start] + '''fn conservative_ocean_heat_tendency(
    geometry: &OceanProjectionGeometry,
    edge_transport_m2_s: &[f64],
    temperature_k: &[f64],
    cell_area_m2: &[f64],
    phase_seconds: f64,
    advection_relaxation: f64,
    cfl_limit: f64,
    output_k_s: &mut [f64],
) {
    debug_assert_eq!(edge_transport_m2_s.len(), geometry.edges.len());
    output_k_s.fill(0.0);
    if advection_relaxation <= 0.0 || phase_seconds <= 0.0 {
        return;
    }

    // The projected edge transport is conservative, but one climatology phase
    // spans many physical advection times at L7. Limit aggregate donor outflow
    // rather than clamping cell tendencies independently so the explicit
    // donor-cell heat step remains both stable and conservative.
    let mut outgoing_transport_m2_s = vec![0.0; temperature_k.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let transport = edge_transport_m2_s[edge_index];
        if transport > 0.0 {
            outgoing_transport_m2_s[edge.a] += transport;
        } else if transport < 0.0 {
            outgoing_transport_m2_s[edge.b] += -transport;
        }
    }
    let mut donor_scale = vec![1.0; temperature_k.len()];
    for sample in 0..temperature_k.len() {
        let outgoing = outgoing_transport_m2_s[sample];
        if outgoing <= 0.0 {
            continue;
        }
        let requested_fraction = outgoing * phase_seconds * advection_relaxation
            / cell_area_m2[sample].max(1.0);
        if requested_fraction > cfl_limit {
            donor_scale[sample] = cfl_limit / requested_fraction;
        }
    }

    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let transport = edge_transport_m2_s[edge_index];
        if transport.abs() <= 1.0e-18 {
            continue;
        }
        let upstream = if transport >= 0.0 { edge.a } else { edge.b };
        let effective_transport = transport * advection_relaxation * donor_scale[upstream];
        let advected_anomaly_k = temperature_k[upstream] - 273.15;
        let heat_transport = effective_transport * advected_anomaly_k;
        output_k_s[edge.a] -= heat_transport / cell_area_m2[edge.a].max(1.0);
        output_k_s[edge.b] += heat_transport / cell_area_m2[edge.b].max(1.0);
    }
}
''' + text[end:]

old_call = '''            conservative_ocean_heat_tendency(
                &ocean_projection_geometry,
                &ocean_edge_transport_m2_s,
                &previous_sst,
                &cell_area_m2,
                &mut ocean_heat_tendency_k_s,
            );
'''
new_call = '''            conservative_ocean_heat_tendency(
                &ocean_projection_geometry,
                &ocean_edge_transport_m2_s,
                &previous_sst,
                &cell_area_m2,
                phase_seconds,
                parameters.ocean_advection_relaxation,
                parameters.ocean_advection_cfl_limit,
                &mut ocean_heat_tendency_k_s,
            );
'''
assert old_call in text
text = text.replace(old_call, new_call, 1)
text = text.replace(
    '''                let advection_delta = (ocean_heat_tendency_k_s[i]
                    * phase_seconds
                    * parameters.ocean_advection_relaxation)
                    .clamp(-4.0, 4.0);
''',
    '''                let advection_delta =
                    (ocean_heat_tendency_k_s[i] * phase_seconds).clamp(-4.0, 4.0);
''',
    1,
)

# Atmospheric moisture transport is also an explicit graph step. Individual
# edge fractions were bounded, but several outgoing edges could collectively
# export more water than a donor contained, and the later zero clamp created
# mass. Build requested transfers first and scale every donor's paired transfers
# to its available water before applying them.
moisture_start = text.index('                let mut transport_delta = vec![0.0; sample_count];')
moisture_end_marker = '''                for i in 0..sample_count {
                    moisture_mass[i] = (moisture_mass[i] + transport_delta[i]).max(0.0);
                }
'''
moisture_end = text.index(moisture_end_marker, moisture_start) + len(moisture_end_marker)
new_moisture = '''                let mut requested_transfers = Vec::<(usize, usize, f64)>::new();
                let mut requested_outflow = vec![0.0; sample_count];
                for i in 0..sample_count {
                    let origin = topology.positions()[i];
                    for (neighbor_index, arc) in topology
                        .neighbors_of(i as u32)
                        .iter()
                        .zip(topology.neighbor_arc_lengths_of(i as u32).iter())
                    {
                        let j = *neighbor_index as usize;
                        if j <= i {
                            continue;
                        }
                        let position = topology.positions()[j];
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
                        let distance = (*arc * planet.radius_m).max(1.0);
                        let fraction = (projected.abs() * phase_seconds / distance
                            * parameters.moisture_transport_cfl)
                            .clamp(0.0, 0.22);
                        let (donor, receiver) = if projected >= 0.0 { (i, j) } else { (j, i) };
                        let requested = moisture_mass[donor] * fraction;
                        if requested > 0.0 {
                            requested_transfers.push((donor, receiver, requested));
                            requested_outflow[donor] += requested;
                        }
                    }
                }
                let mut donor_scale = vec![1.0; sample_count];
                for i in 0..sample_count {
                    if requested_outflow[i] > moisture_mass[i] && requested_outflow[i] > 0.0 {
                        donor_scale[i] = moisture_mass[i] / requested_outflow[i];
                    }
                }
                let mut transport_delta = vec![0.0; sample_count];
                for (donor, receiver, requested) in requested_transfers {
                    let transfer = requested * donor_scale[donor];
                    transport_delta[donor] -= transfer;
                    transport_delta[receiver] += transfer;
                }
                for i in 0..sample_count {
                    moisture_mass[i] = (moisture_mass[i] + transport_delta[i]).max(0.0);
                }
'''
text = text[:moisture_start] + new_moisture + text[moisture_end:]

# Unit-lock conservative CFL limiting on a minimal two-cell ocean graph.
insert_at = text.rfind('\n}')
assert insert_at > 0
unit_test = r'''

    #[test]
    fn ocean_heat_advection_cfl_limiter_is_conservative_and_bounds_donor_exchange() {
        let geometry = OceanProjectionGeometry {
            edges: vec![OceanProjectionEdge {
                a: 0,
                b: 1,
                a_east: 1.0,
                a_north: 0.0,
                b_east: -1.0,
                b_north: 0.0,
                interface_length_m: 1.0,
                conductance: 1.0,
            }],
            diagonal: vec![1.0, 1.0],
        };
        let temperature = [300.0, 280.0];
        let area = [100.0, 100.0];
        let mut tendency = [0.0, 0.0];
        conservative_ocean_heat_tendency(
            &geometry,
            &[100.0],
            &temperature,
            &area,
            10.0,
            1.0,
            0.45,
            &mut tendency,
        );
        let donor_fraction = -tendency[0] * 10.0 / (temperature[0] - 273.15);
        assert!((donor_fraction - 0.45).abs() < 1.0e-12);
        let area_weighted_tendency = tendency[0] * area[0] + tendency[1] * area[1];
        assert!(area_weighted_tendency.abs() < 1.0e-10);

        conservative_ocean_heat_tendency(
            &geometry,
            &[100.0],
            &temperature,
            &area,
            10.0,
            0.0,
            0.45,
            &mut tendency,
        );
        assert_eq!(tendency, [0.0, 0.0]);
    }
'''
text = text[:insert_at] + unit_test + text[insert_at:]
climate.write_text(text)

# Document the stability/conservation contract.
doc = Path('docs/worldgen-rewrite/WG5_CLIMATE.md')
doc_text = doc.read_text()
needle = 'WG-5 intentionally includes a reduced B+ surface-ocean circulation model: currents are generated from wind stress, Coriolis response, WG-4 ocean connectivity, coastlines, and bathymetric mobility; SST transport feeds back into the atmospheric thermal solution. It does not attempt a full 3-D salinity/thermohaline ocean.\n'
replacement = needle + '\nProjected ocean-edge transports drive SST advection through a conservative donor-cell update. Aggregate donor outflow is CFL-limited per orbital phase, so the explicit heat step remains stable as mesh spacing shrinks through the L7 quality target without weakening circulation at coarser levels. Atmospheric moisture transport likewise scales aggregate outgoing graph transfers to the donor water mass before applying paired transfers, preserving moisture mass instead of relying on post-transport zero clamps.\n'
assert needle in doc_text
doc.write_text(doc_text.replace(needle, replacement, 1))
