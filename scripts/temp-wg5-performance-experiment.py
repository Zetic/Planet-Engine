from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''fn scalar_gradient(
    topology: &GeodesicTopology,
    values: &[f64],
    radius_m: f64,
    sample: usize,
    east: [f64; 3],
    north: [f64; 3],
) -> (f64, f64) {
    let origin = topology.positions()[sample];
    let neighbors = topology.neighbors_of(sample as u32);
    let lengths = topology.neighbor_arc_lengths_of(sample as u32);
    let mut east_gradient = 0.0;
    let mut north_gradient = 0.0;
    let mut weight_sum = 0.0;
    for (neighbor, arc) in neighbors.iter().zip(lengths.iter()) {
        let neighbor_index = *neighbor as usize;
        let neighbor_position = topology.positions()[neighbor_index];
        let radial = dot(neighbor_position, origin);
        let tangent = [
            neighbor_position[0] - origin[0] * radial,
            neighbor_position[1] - origin[1] * radial,
            neighbor_position[2] - origin[2] * radial,
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
        let distance = (*arc * radius_m).max(1.0);
        let derivative = (values[neighbor_index] - values[sample]) / distance;
        let weight = 1.0 / distance;
        east_gradient += derivative * dot(direction, east) * weight;
        north_gradient += derivative * dot(direction, north) * weight;
        weight_sum += weight;
    }
    if weight_sum > 0.0 {
        (east_gradient / weight_sum, north_gradient / weight_sum)
    } else {
        (0.0, 0.0)
    }
}
''',
    '''#[derive(Clone, Copy, Debug)]
struct ScalarGradientTerm {
    direction_east: f64,
    direction_north: f64,
    distance_m: f64,
}

#[derive(Clone, Debug)]
struct ScalarGradientGeometry {
    offsets: Vec<usize>,
    terms: Vec<ScalarGradientTerm>,
}

fn build_scalar_gradient_geometry(
    topology: &GeodesicTopology,
    radius_m: f64,
    east_bases: &[[f64; 3]],
    north_bases: &[[f64; 3]],
) -> ScalarGradientGeometry {
    let sample_count = topology.metrics().sample_count as usize;
    let mut offsets = Vec::with_capacity(sample_count + 1);
    let mut terms = Vec::with_capacity(sample_count.saturating_mul(6));
    offsets.push(0);
    for sample in 0..sample_count {
        let origin = topology.positions()[sample];
        let neighbors = topology.neighbors_of(sample as u32);
        let lengths = topology.neighbor_arc_lengths_of(sample as u32);
        for (neighbor, arc) in neighbors.iter().zip(lengths.iter()) {
            let neighbor_index = *neighbor as usize;
            let neighbor_position = topology.positions()[neighbor_index];
            let radial = dot(neighbor_position, origin);
            let tangent = [
                neighbor_position[0] - origin[0] * radial,
                neighbor_position[1] - origin[1] * radial,
                neighbor_position[2] - origin[2] * radial,
            ];
            let tangent_norm = dot(tangent, tangent).sqrt();
            if tangent_norm <= 1.0e-15 {
                terms.push(ScalarGradientTerm {
                    direction_east: 0.0,
                    direction_north: 0.0,
                    distance_m: 0.0,
                });
                continue;
            }
            let direction = [
                tangent[0] / tangent_norm,
                tangent[1] / tangent_norm,
                tangent[2] / tangent_norm,
            ];
            terms.push(ScalarGradientTerm {
                direction_east: dot(direction, east_bases[sample]),
                direction_north: dot(direction, north_bases[sample]),
                distance_m: (*arc * radius_m).max(1.0),
            });
        }
        offsets.push(terms.len());
    }
    ScalarGradientGeometry { offsets, terms }
}

fn scalar_gradient_cached(
    topology: &GeodesicTopology,
    geometry: &ScalarGradientGeometry,
    values: &[f64],
    sample: usize,
) -> (f64, f64) {
    let neighbors = topology.neighbors_of(sample as u32);
    let start = geometry.offsets[sample];
    let end = geometry.offsets[sample + 1];
    let terms = &geometry.terms[start..end];
    debug_assert_eq!(neighbors.len(), terms.len());
    let mut east_gradient = 0.0;
    let mut north_gradient = 0.0;
    let mut weight_sum = 0.0;
    for (neighbor, term) in neighbors.iter().zip(terms.iter()) {
        if term.distance_m <= 0.0 {
            continue;
        }
        let neighbor_index = *neighbor as usize;
        let derivative = (values[neighbor_index] - values[sample]) / term.distance_m;
        let weight = 1.0 / term.distance_m;
        east_gradient += derivative * term.direction_east * weight;
        north_gradient += derivative * term.direction_north * weight;
        weight_sum += weight;
    }
    if weight_sum > 0.0 {
        (east_gradient / weight_sum, north_gradient / weight_sum)
    } else {
        (0.0, 0.0)
    }
}
''',
    'replace scalar gradient with cached geometry',
)

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
''',
    '''    let scalar_gradient_geometry = build_scalar_gradient_geometry(
        topology,
        planet.radius_m,
        &east_bases,
        &north_bases,
    );
    let mut terrain_gradient_east = vec![0.0; sample_count];
    let mut terrain_gradient_north = vec![0.0; sample_count];
    for i in 0..sample_count {
        let (east, north) = scalar_gradient_cached(
            topology,
            &scalar_gradient_geometry,
            &terrain_height_m,
            i,
        );
''',
    'cache scalar gradient geometry for terrain',
)

replace_once(
    '''                    let (gradient_east, gradient_north) = scalar_gradient(
                        topology,
                        &temperature,
                        planet.radius_m,
                        i,
                        east_bases[i],
                        north_bases[i],
                    );
''',
    '''                    let (gradient_east, gradient_north) = scalar_gradient_cached(
                        topology,
                        &scalar_gradient_geometry,
                        &temperature,
                        i,
                    );
''',
    'reuse scalar gradient geometry for atmospheric temperature',
)

path.write_text(text)
