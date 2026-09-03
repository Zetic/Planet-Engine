from pathlib import Path

climate_path = Path('rust/interlink-worldgen/src/climate.rs')
text = climate_path.read_text()

start = text.index('fn current_divergence(')
end = text.index('fn validate_inputs(')
replacement = r'''#[derive(Clone, Copy, Debug)]
struct OceanProjectionEdge {
    a: usize,
    b: usize,
    a_east: f64,
    a_north: f64,
    b_east: f64,
    b_north: f64,
    interface_length_m: f64,
    conductance: f64,
}

#[derive(Clone, Debug)]
struct OceanProjectionGeometry {
    edges: Vec<OceanProjectionEdge>,
    diagonal: Vec<f64>,
}

fn edge_direction_components(
    topology: &GeodesicTopology,
    from: usize,
    to: usize,
    east: [f64; 3],
    north: [f64; 3],
) -> Option<(f64, f64)> {
    let origin = topology.positions()[from];
    let target = topology.positions()[to];
    let radial = dot(target, origin);
    let tangent = [
        target[0] - origin[0] * radial,
        target[1] - origin[1] * radial,
        target[2] - origin[2] * radial,
    ];
    let magnitude = dot(tangent, tangent).sqrt();
    if magnitude <= 1.0e-15 {
        return None;
    }
    let direction = [
        tangent[0] / magnitude,
        tangent[1] / magnitude,
        tangent[2] / magnitude,
    ];
    Some((dot(direction, east), dot(direction, north)))
}

fn build_ocean_projection_geometry(
    topology: &GeodesicTopology,
    ocean: &[bool],
    east_bases: &[[f64; 3]],
    north_bases: &[[f64; 3]],
    radius_m: f64,
) -> OceanProjectionGeometry {
    let mut edges = Vec::new();
    let mut diagonal = vec![0.0; ocean.len()];
    for a in 0..ocean.len() {
        if !ocean[a] {
            continue;
        }
        for ((neighbor, arc), interface_arc) in topology
            .neighbors_of(a as u32)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(a as u32).iter())
            .zip(topology.neighbor_interface_arc_lengths_of(a as u32).iter())
        {
            let b = *neighbor as usize;
            if b <= a || !ocean[b] {
                continue;
            }
            let Some((a_east, a_north)) = edge_direction_components(
                topology,
                a,
                b,
                east_bases[a],
                north_bases[a],
            ) else {
                continue;
            };
            let Some((b_east, b_north)) = edge_direction_components(
                topology,
                b,
                a,
                east_bases[b],
                north_bases[b],
            ) else {
                continue;
            };
            let distance_m = (*arc * radius_m).max(1.0);
            let interface_length_m = (*interface_arc * radius_m).max(1.0);
            let conductance = (interface_length_m / distance_m).max(1.0e-12);
            edges.push(OceanProjectionEdge {
                a,
                b,
                a_east,
                a_north,
                b_east,
                b_north,
                interface_length_m,
                conductance,
            });
            diagonal[a] += conductance;
            diagonal[b] += conductance;
        }
    }
    OceanProjectionGeometry { edges, diagonal }
}

fn apply_ocean_laplacian(
    geometry: &OceanProjectionGeometry,
    values: &[f64],
    output: &mut [f64],
) {
    output.fill(0.0);
    for edge in &geometry.edges {
        let contribution = edge.conductance * (values[edge.a] - values[edge.b]);
        output[edge.a] += contribution;
        output[edge.b] -= contribution;
    }
}

fn correct_ocean_currents(
    ocean: &[bool],
    geometry: &OceanProjectionGeometry,
    current_east: &mut [f64],
    current_north: &mut [f64],
    parameters: &ClimateParameters,
) -> f64 {
    if geometry.edges.is_empty() {
        current_east.fill(0.0);
        current_north.fill(0.0);
        return 0.0;
    }

    // Convert endpoint ENU vectors into one antisymmetric transport value per
    // ocean-ocean interface. Land interfaces never enter this graph.
    let mut edge_flux = vec![0.0; geometry.edges.len()];
    let mut divergence = vec![0.0; ocean.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let outward_a = current_east[edge.a] * edge.a_east
            + current_north[edge.a] * edge.a_north;
        let outward_b = current_east[edge.b] * edge.b_east
            + current_north[edge.b] * edge.b_north;
        let normal_speed = 0.5 * (outward_a - outward_b);
        let flux = normal_speed * edge.interface_length_m;
        edge_flux[edge_index] = flux;
        divergence[edge.a] += flux;
        divergence[edge.b] -= flux;
    }

    // Solve L p = div(q) with a diagonally preconditioned conjugate-gradient
    // projection. Because edge flux is antisymmetric, each connected ocean
    // component has zero net right-hand side and the constant pressure null
    // mode does not affect the corrected transport.
    let mut pressure = vec![0.0; ocean.len()];
    let mut residual = divergence.clone();
    let mut preconditioned = vec![0.0; ocean.len()];
    let mut direction = vec![0.0; ocean.len()];
    let mut laplacian_direction = vec![0.0; ocean.len()];
    for i in 0..ocean.len() {
        if ocean[i] && geometry.diagonal[i] > 0.0 {
            preconditioned[i] = residual[i] / geometry.diagonal[i];
            direction[i] = preconditioned[i];
        }
    }
    let mut rho = residual
        .iter()
        .zip(preconditioned.iter())
        .map(|(r, z)| r * z)
        .sum::<f64>();
    for _ in 0..usize::from(parameters.ocean_current_correction_iterations) {
        if !rho.is_finite() || rho <= 1.0e-24 {
            break;
        }
        apply_ocean_laplacian(geometry, &direction, &mut laplacian_direction);
        let denominator = direction
            .iter()
            .zip(laplacian_direction.iter())
            .map(|(d, q)| d * q)
            .sum::<f64>();
        if !denominator.is_finite() || denominator <= 1.0e-24 {
            break;
        }
        let alpha = rho / denominator;
        for i in 0..ocean.len() {
            pressure[i] += alpha * direction[i];
            residual[i] -= alpha * laplacian_direction[i];
            preconditioned[i] = if ocean[i] && geometry.diagonal[i] > 0.0 {
                residual[i] / geometry.diagonal[i]
            } else {
                0.0
            };
        }
        let next_rho = residual
            .iter()
            .zip(preconditioned.iter())
            .map(|(r, z)| r * z)
            .sum::<f64>();
        if !next_rho.is_finite() || next_rho <= 1.0e-24 {
            break;
        }
        let beta = next_rho / rho;
        for i in 0..ocean.len() {
            direction[i] = preconditioned[i] + beta * direction[i];
        }
        rho = next_rho;
    }

    let mut projected_divergence = vec![0.0; ocean.len()];
    let mut perimeter = vec![0.0; ocean.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let correction = edge.conductance * (pressure[edge.a] - pressure[edge.b]);
        edge_flux[edge_index] -= correction;
        projected_divergence[edge.a] += edge_flux[edge_index];
        projected_divergence[edge.b] -= edge_flux[edge_index];
        perimeter[edge.a] += edge.interface_length_m;
        perimeter[edge.b] += edge.interface_length_m;
    }

    // Reconstruct the best-fit local ENU display/advection vector from the
    // conservative edge-normal transports. This also naturally turns flow
    // along coastlines because blocked land edges are absent from the solve.
    let mut matrix_ee = vec![0.0; ocean.len()];
    let mut matrix_en = vec![0.0; ocean.len()];
    let mut matrix_nn = vec![0.0; ocean.len()];
    let mut rhs_e = vec![0.0; ocean.len()];
    let mut rhs_n = vec![0.0; ocean.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let normal_speed = edge_flux[edge_index] / edge.interface_length_m;
        for (sample, east, north, speed) in [
            (edge.a, edge.a_east, edge.a_north, normal_speed),
            (edge.b, edge.b_east, edge.b_north, -normal_speed),
        ] {
            let weight = edge.interface_length_m;
            matrix_ee[sample] += weight * east * east;
            matrix_en[sample] += weight * east * north;
            matrix_nn[sample] += weight * north * north;
            rhs_e[sample] += weight * speed * east;
            rhs_n[sample] += weight * speed * north;
        }
    }
    current_east.fill(0.0);
    current_north.fill(0.0);
    for i in 0..ocean.len() {
        if !ocean[i] || perimeter[i] <= 0.0 {
            continue;
        }
        let trace = matrix_ee[i] + matrix_nn[i];
        let regularization = (trace * 1.0e-10).max(1.0e-12);
        let a = matrix_ee[i] + regularization;
        let b = matrix_en[i];
        let d = matrix_nn[i] + regularization;
        let determinant = a * d - b * b;
        if determinant.abs() <= 1.0e-18 {
            continue;
        }
        let east = (rhs_e[i] * d - rhs_n[i] * b) / determinant;
        let north = (rhs_n[i] * a - rhs_e[i] * b) / determinant;
        (current_east[i], current_north[i]) =
            clamp_vector(east, north, parameters.maximum_surface_current_m_s);
    }

    let mut residual_speed = 0.0;
    let mut residual_samples = 0.0;
    for i in 0..ocean.len() {
        if ocean[i] && perimeter[i] > 0.0 {
            residual_speed += projected_divergence[i].abs() / perimeter[i];
            residual_samples += 1.0;
        }
    }
    if residual_samples > 0.0 {
        residual_speed / residual_samples
    } else {
        0.0
    }
}

'''
text = text[:start] + replacement + text[end:]

needle = '    let mut temperature = vec![0.0; sample_count];\n'
assert needle in text
text = text.replace(
    needle,
    '    let ocean_projection_geometry = build_ocean_projection_geometry(\n'
    '        topology,\n'
    '        &ocean,\n'
    '        &east_bases,\n'
    '        &north_bases,\n'
    '        planet.radius_m,\n'
    '    );\n\n'
    + needle,
    1,
)

old_call = '''                final_divergence_residual = correct_ocean_currents(
                    topology,
                    &ocean,
                    &east_bases,
                    &north_bases,
                    &mut current_east,
                    &mut current_north,
                    &parameters,
                );'''
new_call = '''                final_divergence_residual = correct_ocean_currents(
                    &ocean,
                    &ocean_projection_geometry,
                    &mut current_east,
                    &mut current_north,
                    &parameters,
                );'''
assert old_call in text
text = text.replace(old_call, new_call, 1)
text = text.replace('ocean_current_correction_iterations: 4,', 'ocean_current_correction_iterations: 6,', 1)
climate_path.write_text(text)

test_path = Path('rust/interlink-worldgen/tests/climate_ensemble.rs')
test_text = test_path.read_text()
needle = '    assert!(first.metrics.mean_surface_current_m_s > 0.0);\n'
assert needle in test_text
addition = needle + '''    assert!(
        first.metrics.ocean_divergence_residual_m_s
            < first.metrics.mean_surface_current_m_s * 0.10 + 1.0e-6,
        "projected ocean transport should have a small divergence residual: residual={} mean_current={}",
        first.metrics.ocean_divergence_residual_m_s,
        first.metrics.mean_surface_current_m_s,
    );
'''
test_text = test_text.replace(needle, addition, 1)
test_path.write_text(test_text)
