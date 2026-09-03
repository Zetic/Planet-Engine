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
struct AtmosphericHeatGeometry {
    edges: Vec<AtmosphericHeatEdge>,
    diagonal_geometry: Vec<f64>,
}
''',
    '''#[derive(Clone, Debug)]
struct AtmosphericHeatGeometry {
    edges: Vec<AtmosphericHeatEdge>,
    diagonal_geometry: Vec<f64>,
}

#[derive(Clone, Debug)]
struct AtmosphericHeatWorkspace {
    capacity: Vec<f64>,
    rhs: Vec<f64>,
    diagonal: Vec<f64>,
    x: Vec<f64>,
    matrix_x: Vec<f64>,
    residual: Vec<f64>,
    preconditioned: Vec<f64>,
    direction: Vec<f64>,
    matrix_direction: Vec<f64>,
}

impl AtmosphericHeatWorkspace {
    fn new(sample_count: usize) -> Self {
        Self {
            capacity: vec![0.0; sample_count],
            rhs: vec![0.0; sample_count],
            diagonal: vec![0.0; sample_count],
            x: vec![0.0; sample_count],
            matrix_x: vec![0.0; sample_count],
            residual: vec![0.0; sample_count],
            preconditioned: vec![0.0; sample_count],
            direction: vec![0.0; sample_count],
            matrix_direction: vec![0.0; sample_count],
        }
    }
}
''',
    'heat workspace definition',
)

replace_once(
    '''fn diffuse_atmospheric_heat(
    geometry: &AtmosphericHeatGeometry,
    temperature: &mut [f64],
''',
    '''fn diffuse_atmospheric_heat(
    geometry: &AtmosphericHeatGeometry,
    workspace: &mut AtmosphericHeatWorkspace,
    temperature: &mut [f64],
''',
    'heat diffuse signature',
)

replace_once(
    '''    let mut capacity = vec![0.0; temperature.len()];
    let mut rhs = vec![0.0; temperature.len()];
    let mut diagonal = vec![0.0; temperature.len()];
    for i in 0..temperature.len() {
''',
    '''    let AtmosphericHeatWorkspace {
        capacity,
        rhs,
        diagonal,
        x,
        matrix_x,
        residual,
        preconditioned,
        direction,
        matrix_direction,
    } = workspace;
    debug_assert_eq!(capacity.len(), temperature.len());
    for i in 0..temperature.len() {
''',
    'reuse heat coefficient workspace',
)

replace_once(
    '''    let mut x = temperature.to_vec();
    let mut matrix_x = vec![0.0; x.len()];
    apply_atmospheric_heat_matrix(geometry, &capacity, diffusion_scale_j_k, &x, &mut matrix_x);
    let mut residual = rhs
        .iter()
        .zip(matrix_x.iter())
        .map(|(b, ax)| b - ax)
        .collect::<Vec<_>>();
    let mut preconditioned = residual
        .iter()
        .enumerate()
        .map(|(i, r)| r / diagonal[i].max(1.0e-18))
        .collect::<Vec<_>>();
    let mut direction = preconditioned.clone();
    let mut matrix_direction = vec![0.0; x.len()];
''',
    '''    x.copy_from_slice(temperature);
    apply_atmospheric_heat_matrix(geometry, capacity, diffusion_scale_j_k, x, matrix_x);
    for i in 0..x.len() {
        residual[i] = rhs[i] - matrix_x[i];
        preconditioned[i] = residual[i] / diagonal[i].max(1.0e-18);
        direction[i] = preconditioned[i];
    }
''',
    'reuse heat CG vectors',
)

replace_once(
    '''        apply_atmospheric_heat_matrix(
            geometry,
            &capacity,
            diffusion_scale_j_k,
            &direction,
            &mut matrix_direction,
        );
''',
    '''        apply_atmospheric_heat_matrix(
            geometry,
            capacity,
            diffusion_scale_j_k,
            direction,
            matrix_direction,
        );
''',
    'reuse heat matrix direction',
)

replace_once(
    '''    let atmospheric_heat_geometry = build_atmospheric_heat_geometry(topology, planet.radius_m);
    let atmospheric_moisture_edges =
''',
    '''    let atmospheric_heat_geometry = build_atmospheric_heat_geometry(topology, planet.radius_m);
    let mut atmospheric_heat_workspace = AtmosphericHeatWorkspace::new(sample_count);
    let atmospheric_moisture_edges =
''',
    'create heat workspace once',
)

replace_once(
    '''            diffuse_atmospheric_heat(
                &atmospheric_heat_geometry,
                &mut temperature,
''',
    '''            diffuse_atmospheric_heat(
                &atmospheric_heat_geometry,
                &mut atmospheric_heat_workspace,
                &mut temperature,
''',
    'pass main heat workspace',
)

replace_once(
    '''        let mut temperature = [300.0, 280.0];
        let before =
''',
    '''        let mut temperature = [300.0, 280.0];
        let mut workspace = AtmosphericHeatWorkspace::new(2);
        let before =
''',
    'create heat test workspace',
)

replace_once(
    '''        diffuse_atmospheric_heat(
            &geometry,
            &mut temperature,
''',
    '''        diffuse_atmospheric_heat(
            &geometry,
            &mut workspace,
            &mut temperature,
''',
    'pass heat test workspace',
)

path.write_text(text)
