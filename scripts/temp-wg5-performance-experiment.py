from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''#[derive(Clone, Debug)]
struct OceanProjectionGeometry {
    edges: Vec<OceanProjectionEdge>,
    diagonal: Vec<f64>,
}
''',
    '''#[derive(Clone, Debug)]
struct OceanProjectionGeometry {
    edges: Vec<OceanProjectionEdge>,
    diagonal: Vec<f64>,
    perimeter: Vec<f64>,
    matrix_ee: Vec<f64>,
    matrix_en: Vec<f64>,
    matrix_nn: Vec<f64>,
}

#[derive(Clone, Debug)]
struct OceanProjectionWorkspace {
    divergence: Vec<f64>,
    pressure: Vec<f64>,
    residual: Vec<f64>,
    preconditioned: Vec<f64>,
    direction: Vec<f64>,
    laplacian_direction: Vec<f64>,
    projected_divergence: Vec<f64>,
    rhs_e: Vec<f64>,
    rhs_n: Vec<f64>,
}

impl OceanProjectionWorkspace {
    fn new(sample_count: usize) -> Self {
        Self {
            divergence: vec![0.0; sample_count],
            pressure: vec![0.0; sample_count],
            residual: vec![0.0; sample_count],
            preconditioned: vec![0.0; sample_count],
            direction: vec![0.0; sample_count],
            laplacian_direction: vec![0.0; sample_count],
            projected_divergence: vec![0.0; sample_count],
            rhs_e: vec![0.0; sample_count],
            rhs_n: vec![0.0; sample_count],
        }
    }
}
''',
    'ocean geometry/workspace definition',
)

replace_once(
    '''    OceanProjectionGeometry { edges, diagonal }
}
''',
    '''    let mut perimeter = vec![0.0; ocean.len()];
    let mut matrix_ee = vec![0.0; ocean.len()];
    let mut matrix_en = vec![0.0; ocean.len()];
    let mut matrix_nn = vec![0.0; ocean.len()];
    for edge in &edges {
        perimeter[edge.a] += edge.interface_length_m;
        perimeter[edge.b] += edge.interface_length_m;
        for (sample, east, north) in [
            (edge.a, edge.a_east, edge.a_north),
            (edge.b, edge.b_east, edge.b_north),
        ] {
            let weight = edge.interface_length_m;
            matrix_ee[sample] += weight * east * east;
            matrix_en[sample] += weight * east * north;
            matrix_nn[sample] += weight * north * north;
        }
    }
    OceanProjectionGeometry {
        edges,
        diagonal,
        perimeter,
        matrix_ee,
        matrix_en,
        matrix_nn,
    }
}
''',
    'precompute ocean reconstruction geometry',
)

replace_once(
    '''    projected_edge_transport_m2_s: &mut [f64],
    parameters: &ClimateParameters,
) -> f64 {
''',
    '''    projected_edge_transport_m2_s: &mut [f64],
    workspace: &mut OceanProjectionWorkspace,
    parameters: &ClimateParameters,
) -> f64 {
''',
    'ocean correction signature',
)

replace_once(
    '''    if geometry.edges.is_empty() {
        current_east.fill(0.0);
        current_north.fill(0.0);
        return 0.0;
    }

    // Convert endpoint ENU vectors into one antisymmetric transport value per
''',
    '''    if geometry.edges.is_empty() {
        current_east.fill(0.0);
        current_north.fill(0.0);
        return 0.0;
    }

    let OceanProjectionWorkspace {
        divergence,
        pressure,
        residual,
        preconditioned,
        direction,
        laplacian_direction,
        projected_divergence,
        rhs_e,
        rhs_n,
    } = workspace;
    divergence.fill(0.0);

    // Convert endpoint ENU vectors into one antisymmetric transport value per
''',
    'destructure ocean workspace',
)

replace_once(
    '''    let mut divergence = vec![0.0; ocean.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
''',
    '''    for (edge_index, edge) in geometry.edges.iter().enumerate() {
''',
    'reuse divergence workspace',
)

replace_once(
    '''    let mut pressure = vec![0.0; ocean.len()];
    let mut residual = divergence.clone();
    let mut preconditioned = vec![0.0; ocean.len()];
    let mut direction = vec![0.0; ocean.len()];
    let mut laplacian_direction = vec![0.0; ocean.len()];
    for i in 0..ocean.len() {
''',
    '''    pressure.fill(0.0);
    residual.copy_from_slice(divergence);
    preconditioned.fill(0.0);
    direction.fill(0.0);
    laplacian_direction.fill(0.0);
    for i in 0..ocean.len() {
''',
    'reuse CG workspace',
)

replace_once(
    '''    let mut projected_divergence = vec![0.0; ocean.len()];
    let mut perimeter = vec![0.0; ocean.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
''',
    '''    projected_divergence.fill(0.0);
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
''',
    'reuse projected divergence',
)

replace_once(
    '''        projected_divergence[edge.a] += projected_edge_transport_m2_s[edge_index];
        projected_divergence[edge.b] -= projected_edge_transport_m2_s[edge_index];
        perimeter[edge.a] += edge.interface_length_m;
        perimeter[edge.b] += edge.interface_length_m;
''',
    '''        projected_divergence[edge.a] += projected_edge_transport_m2_s[edge_index];
        projected_divergence[edge.b] -= projected_edge_transport_m2_s[edge_index];
''',
    'remove repeated perimeter accumulation',
)

replace_once(
    '''    let mut matrix_ee = vec![0.0; ocean.len()];
    let mut matrix_en = vec![0.0; ocean.len()];
    let mut matrix_nn = vec![0.0; ocean.len()];
    let mut rhs_e = vec![0.0; ocean.len()];
    let mut rhs_n = vec![0.0; ocean.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
''',
    '''    rhs_e.fill(0.0);
    rhs_n.fill(0.0);
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
''',
    'reuse reconstruction rhs',
)

replace_once(
    '''            let weight = edge.interface_length_m;
            matrix_ee[sample] += weight * east * east;
            matrix_en[sample] += weight * east * north;
            matrix_nn[sample] += weight * north * north;
            rhs_e[sample] += weight * speed * east;
''',
    '''            let weight = edge.interface_length_m;
            rhs_e[sample] += weight * speed * east;
''',
    'remove repeated reconstruction matrix accumulation',
)

text = text.replace('perimeter[i]', 'geometry.perimeter[i]')
text = text.replace('matrix_ee[i]', 'geometry.matrix_ee[i]')
text = text.replace('matrix_en[i]', 'geometry.matrix_en[i]')
text = text.replace('matrix_nn[i]', 'geometry.matrix_nn[i]')

replace_once(
    '''    let mut ocean_edge_transport_m2_s = vec![0.0; ocean_projection_geometry.edges.len()];
    let mut ocean_heat_tendency_k_s = vec![0.0; sample_count];
''',
    '''    let mut ocean_projection_workspace = OceanProjectionWorkspace::new(sample_count);
    let mut ocean_edge_transport_m2_s = vec![0.0; ocean_projection_geometry.edges.len()];
    let mut ocean_heat_tendency_k_s = vec![0.0; sample_count];
''',
    'create ocean workspace once',
)

replace_once(
    '''                    &mut current_north,
                    &mut ocean_edge_transport_m2_s,
                    &parameters,
                );
''',
    '''                    &mut current_north,
                    &mut ocean_edge_transport_m2_s,
                    &mut ocean_projection_workspace,
                    &parameters,
                );
''',
    'pass ocean workspace',
)

path.write_text(text)
