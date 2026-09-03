from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''struct ScalarGradientGeometry {
    offsets: Vec<usize>,
    terms: Vec<ScalarGradientTerm>,
}
''',
    '''struct ScalarGradientGeometry {
    offsets: Vec<usize>,
    neighbors: Vec<u32>,
    terms: Vec<ScalarGradientTerm>,
}
''',
    'add scalar gradient neighbor cache',
)

replace_once(
    '''    let mut offsets = Vec::with_capacity(sample_count + 1);
    let mut terms = Vec::with_capacity(sample_count.saturating_mul(6));
''',
    '''    let mut offsets = Vec::with_capacity(sample_count + 1);
    let mut cached_neighbors = Vec::with_capacity(sample_count.saturating_mul(6));
    let mut terms = Vec::with_capacity(sample_count.saturating_mul(6));
''',
    'allocate scalar gradient neighbor cache',
)

replace_once(
    '''        for (neighbor, arc) in neighbors.iter().zip(lengths.iter()) {
            let neighbor_index = *neighbor as usize;
''',
    '''        for (neighbor, arc) in neighbors.iter().zip(lengths.iter()) {
            let neighbor_index = *neighbor as usize;
            cached_neighbors.push(*neighbor);
''',
    'populate scalar gradient neighbor cache',
)

replace_once(
    '''    ScalarGradientGeometry { offsets, terms }
''',
    '''    ScalarGradientGeometry {
        offsets,
        neighbors: cached_neighbors,
        terms,
    }
''',
    'return scalar gradient neighbor cache',
)

replace_once(
    '''fn scalar_gradient_cached(
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
''',
    '''fn scalar_gradient_cached(
    _topology: &GeodesicTopology,
    geometry: &ScalarGradientGeometry,
    values: &[f64],
    sample: usize,
) -> (f64, f64) {
    let start = geometry.offsets[sample];
    let end = geometry.offsets[sample + 1];
    let neighbors = &geometry.neighbors[start..end];
    let terms = &geometry.terms[start..end];
    debug_assert_eq!(neighbors.len(), terms.len());
''',
    'use cached scalar gradient neighbors',
)

path.write_text(text)
