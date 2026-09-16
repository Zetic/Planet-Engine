from pathlib import Path

p = Path('rust/interlink-worldgen/src/boundary_plate_geometry.rs')
s = p.read_text()

old_struct = '''struct PlateField {
    center: [f64; 3],
    tangent_u: [f64; 3],'''
new_struct = '''struct PlateField {
    center: [f64; 3],
    material_supports: [[f64; 3]; 3],
    tangent_u: [f64; 3],'''
if old_struct in s:
    s = s.replace(old_struct, new_struct, 1)
elif 'material_supports: [[f64; 3]; 3]' not in s:
    raise SystemExit('PlateField struct marker not found')

marker = '\nfn build_fields<T: PlanetTopology>(\n'
if 'fn choose_material_supports<T: PlanetTopology>' not in s:
    helper = '''
fn choose_material_supports<T: PlanetTopology>(
    topology: &T,
    initial: &[u16],
    plate: u16,
    core: u32,
) -> [[f64; 3]; 3] {
    let core_position = topology.unit_position(core);
    let mut supports = [core_position; 3];

    // Two secondary supports summarize the broad inherited material footprint. They are
    // deliberately sparse and capped in angular reach, so they preserve plate identity
    // without tracing the old graph/Voronoi boundary sample-for-sample.
    for slot in 1..3 {
        let mut best = None::<(f64, u32)>;
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            if initial[index] != plate {
                continue;
            }
            let position = topology.unit_position(sample);
            let from_core = dot(position, core_position).clamp(-1.0, 1.0).acos();
            if from_core < 0.18 || from_core > 0.82 {
                continue;
            }
            let minimum_separation = supports[..slot]
                .iter()
                .map(|support| dot(position, *support).clamp(-1.0, 1.0).acos())
                .fold(std::f64::consts::PI, f64::min);
            let score = minimum_separation - (from_core - 0.50).abs() * 0.18;
            let candidate = (score, sample);
            if best
                .map(|current| {
                    candidate.0 > current.0
                        || (candidate.0 == current.0 && candidate.1 < current.1)
                })
                .unwrap_or(true)
            {
                best = Some(candidate);
            }
        }
        if let Some((_, sample)) = best {
            supports[slot] = topology.unit_position(sample);
        } else {
            supports[slot] = supports[slot - 1];
        }
    }
    supports
}
'''
    if marker not in s:
        raise SystemExit('build_fields marker not found')
    s = s.replace(marker, '\n' + helper + marker, 1)

# Build the sparse support set from each provisional material domain.
old_center = '''            let center = topology.unit_position(*core);
            let (tangent_u, tangent_v) = tangent_frame('''
new_center = '''            let center = topology.unit_position(*core);
            let material_supports = choose_material_supports(topology, initial, plate as u16, *core);
            let (tangent_u, tangent_v) = tangent_frame('''
if old_center in s:
    s = s.replace(old_center, new_center, 1)
elif 'let material_supports = choose_material_supports' not in s:
    raise SystemExit('build_fields center marker not found')

old_field = '''            PlateField {
                center,
                tangent_u,'''
new_field = '''            PlateField {
                center,
                material_supports,
                tangent_u,'''
if old_field in s:
    s = s.replace(old_field, new_field, 1)
elif 'material_supports,' not in s:
    raise SystemExit('PlateField initializer marker not found')

# Add a smooth, low-amplitude affinity to the coarse support footprint. This is a
# geometric prior, not a hard ownership lock: explicit rift/structure corridors and
# competing kinematic fields can still move the modern boundary through it.
old_shape = '''    let shape = field.stretch * (u * u - v * v) + field.bend * (2.0 * u * v);

    let corridor_release ='''
new_shape = '''    let shape = field.stretch * (u * u - v * v) + field.bend * (2.0 * u * v);
    let support_distance = field
        .material_supports
        .iter()
        .map(|support| dot(warped, *support).clamp(-1.0, 1.0).acos())
        .fold(std::f64::consts::PI, f64::min);
    let support_affinity = (-(support_distance / 0.42).powi(2)).exp() * 0.115;

    let corridor_release ='''
if old_shape in s:
    s = s.replace(old_shape, new_shape, 1)
elif 'let support_affinity' not in s:
    raise SystemExit('field shape marker not found')

# The local-affinity helper runs before this one and leaves core_support in the final
# score. Add material support beside it without changing the hard geometric penalties.
old_return = '''    -distance + field.area_bias + shape + ancestry + core_support - far_cap
}'''
new_return = '''    -distance + field.area_bias + shape + ancestry + support_affinity + core_support - far_cap
}'''
if old_return in s:
    s = s.replace(old_return, new_return, 1)
elif '+ support_affinity + core_support - far_cap' not in s:
    raise SystemExit('field score return marker not found')

p.write_text(s)
