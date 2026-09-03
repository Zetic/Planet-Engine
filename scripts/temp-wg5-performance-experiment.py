from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''fn conservative_ocean_heat_tendency(
    geometry: &OceanProjectionGeometry,
''',
    '''#[derive(Clone, Debug)]
struct OceanHeatWorkspace {
    outgoing_transport_m2_s: Vec<f64>,
    donor_scale: Vec<f64>,
}

impl OceanHeatWorkspace {
    fn new(sample_count: usize) -> Self {
        Self {
            outgoing_transport_m2_s: vec![0.0; sample_count],
            donor_scale: vec![1.0; sample_count],
        }
    }
}

fn conservative_ocean_heat_tendency(
    geometry: &OceanProjectionGeometry,
    workspace: &mut OceanHeatWorkspace,
''',
    'add ocean heat workspace',
)

replace_once(
    '''    let mut outgoing_transport_m2_s = vec![0.0; temperature_k.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
''',
    '''    let OceanHeatWorkspace {
        outgoing_transport_m2_s,
        donor_scale,
    } = workspace;
    debug_assert_eq!(outgoing_transport_m2_s.len(), temperature_k.len());
    outgoing_transport_m2_s.fill(0.0);
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
''',
    'reuse outgoing transport scratch',
)

replace_once(
    '''    let mut donor_scale = vec![1.0; temperature_k.len()];
    for sample in 0..temperature_k.len() {
''',
    '''    donor_scale.fill(1.0);
    for sample in 0..temperature_k.len() {
''',
    'reuse donor scale scratch',
)

replace_once(
    '''    let mut ocean_edge_transport_m2_s = vec![0.0; ocean_projection_geometry.edges.len()];
    let mut ocean_heat_tendency_k_s = vec![0.0; sample_count];
''',
    '''    let mut ocean_edge_transport_m2_s = vec![0.0; ocean_projection_geometry.edges.len()];
    let mut ocean_heat_workspace = OceanHeatWorkspace::new(sample_count);
    let mut ocean_heat_tendency_k_s = vec![0.0; sample_count];
''',
    'allocate ocean heat workspace once',
)

replace_once(
    '''            conservative_ocean_heat_tendency(
                &ocean_projection_geometry,
                &ocean_edge_transport_m2_s,
''',
    '''            conservative_ocean_heat_tendency(
                &ocean_projection_geometry,
                &mut ocean_heat_workspace,
                &ocean_edge_transport_m2_s,
''',
    'pass ocean heat workspace',
)

replace_once(
    '''        let mut tendency = vec![0.0; 2];
        conservative_ocean_heat_tendency(
            &geometry,
            &[20.0],
''',
    '''        let mut tendency = vec![0.0; 2];
        let mut workspace = OceanHeatWorkspace::new(2);
        conservative_ocean_heat_tendency(
            &geometry,
            &mut workspace,
            &[20.0],
''',
    'update conservative heat test workspace',
)

replace_once(
    '''        let mut tendency = [0.0, 0.0];
        conservative_ocean_heat_tendency(
            &geometry,
            &[100.0],
''',
    '''        let mut tendency = [0.0, 0.0];
        let mut workspace = OceanHeatWorkspace::new(2);
        conservative_ocean_heat_tendency(
            &geometry,
            &mut workspace,
            &[100.0],
''',
    'update cfl test first workspace call',
)

replace_once(
    '''        conservative_ocean_heat_tendency(
            &geometry,
            &[100.0],
            &temperature,
            &area,
            10.0,
            0.0,
''',
    '''        conservative_ocean_heat_tendency(
            &geometry,
            &mut workspace,
            &[100.0],
            &temperature,
            &area,
            10.0,
            0.0,
''',
    'update cfl test second workspace call',
)

path.write_text(text)
