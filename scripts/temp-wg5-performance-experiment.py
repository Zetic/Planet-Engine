from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''struct AtmosphericHeatEdge {
    a: usize,
    b: usize,
    geometric_conductance: f64,
}

#[derive(Clone, Debug)]
struct AtmosphericHeatGeometry {
    edges: Vec<AtmosphericHeatEdge>,
    diagonal_geometry: Vec<f64>,
}

fn build_atmospheric_heat_geometry(
    topology: &GeodesicTopology,
    radius_m: f64,
) -> AtmosphericHeatGeometry {
''',
    '''struct AtmosphericHeatEdge {
    a: usize,
    b: usize,
    geometric_conductance: f64,
    diffusion_conductance_j_k: f64,
}

#[derive(Clone, Debug)]
struct AtmosphericHeatGeometry {
    edges: Vec<AtmosphericHeatEdge>,
    diagonal_geometry: Vec<f64>,
    diffusion_diagonal_j_k: Vec<f64>,
}

fn build_atmospheric_heat_geometry(
    topology: &GeodesicTopology,
    radius_m: f64,
    diffusion_scale_j_k: f64,
) -> AtmosphericHeatGeometry {
''',
    'heat geometry definitions',
)

replace_once(
    '''            edges.push(AtmosphericHeatEdge {
                a,
                b,
                geometric_conductance,
            });
''',
    '''            edges.push(AtmosphericHeatEdge {
                a,
                b,
                geometric_conductance,
                diffusion_conductance_j_k: diffusion_scale_j_k * geometric_conductance,
            });
''',
    'scaled edge conductance',
)

replace_once(
    '''    AtmosphericHeatGeometry {
        edges,
        diagonal_geometry,
    }
}

fn apply_atmospheric_heat_matrix(
    geometry: &AtmosphericHeatGeometry,
    thermal_capacity_j_k: &[f64],
    diffusion_scale_j_k: f64,
    values: &[f64],
''',
    '''    let diffusion_diagonal_j_k = diagonal_geometry
        .iter()
        .map(|value| diffusion_scale_j_k * value)
        .collect();
    AtmosphericHeatGeometry {
        edges,
        diagonal_geometry,
        diffusion_diagonal_j_k,
    }
}

fn apply_atmospheric_heat_matrix(
    geometry: &AtmosphericHeatGeometry,
    thermal_capacity_j_k: &[f64],
    values: &[f64],
''',
    'scaled diagonal and matrix signature',
)

replace_once(
    '''    for edge in &geometry.edges {
        let contribution =
            diffusion_scale_j_k * edge.geometric_conductance * (values[edge.a] - values[edge.b]);
''',
    '''    for edge in &geometry.edges {
        let contribution =
            edge.diffusion_conductance_j_k * (values[edge.a] - values[edge.b]);
''',
    'matrix edge coefficient',
)

replace_once(
    '''    physical: ClimatePhysicalParameters,
    parameters: ClimateParameters,
    phase_seconds: f64,
) {
''',
    '''    physical: ClimatePhysicalParameters,
    parameters: ClimateParameters,
    reference_column_capacity_j_m2_k: f64,
) {
''',
    'diffuse signature',
)

replace_once(
    '''    let reference_column_capacity_j_m2_k = planet.reference_surface_pressure_pa
        / planet.surface_gravity_m_s2
        * physical.atmospheric_specific_heat_j_per_kg_k;
    let diffusion_scale_j_k = phase_seconds
        * parameters.atmospheric_heat_diffusivity_m2_s
        * reference_column_capacity_j_m2_k;
''',
    '',
    'remove repeated diffusion scale calculation',
)

replace_once(
    '''        diagonal[i] = capacity[i] + diffusion_scale_j_k * geometry.diagonal_geometry[i];
''',
    '''        diagonal[i] = capacity[i] + geometry.diffusion_diagonal_j_k[i];
''',
    'scaled diagonal lookup',
)

replace_once(
    '''    apply_atmospheric_heat_matrix(geometry, &capacity, diffusion_scale_j_k, &x, &mut matrix_x);
''',
    '''    apply_atmospheric_heat_matrix(geometry, &capacity, &x, &mut matrix_x);
''',
    'initial heat matrix call',
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
    '''        apply_atmospheric_heat_matrix(geometry, &capacity, &direction, &mut matrix_direction);
''',
    'iterative heat matrix call',
)

caller_old = '''    let atmospheric_heat_geometry = build_atmospheric_heat_geometry(topology, planet.radius_m);
    let atmospheric_moisture_edges =
'''
caller_new = '''    let atmospheric_reference_column_capacity_j_m2_k = planet.reference_surface_pressure_pa
        / planet.surface_gravity_m_s2
        * physical.atmospheric_specific_heat_j_per_kg_k;
    let atmospheric_heat_diffusion_scale_j_k = phase_seconds
        * parameters.atmospheric_heat_diffusivity_m2_s
        * atmospheric_reference_column_capacity_j_m2_k;
    let atmospheric_heat_geometry = build_atmospheric_heat_geometry(
        topology,
        planet.radius_m,
        atmospheric_heat_diffusion_scale_j_k,
    );
    let atmospheric_moisture_edges =
'''
replace_once(caller_old, caller_new, 'heat geometry caller')

call_old = '''                physical,
                parameters,
                phase_seconds,
            );
'''
call_new = '''                physical,
                parameters,
                atmospheric_reference_column_capacity_j_m2_k,
            );
'''
replace_once(call_old, call_new, 'diffuse heat caller')

path.write_text(text)
