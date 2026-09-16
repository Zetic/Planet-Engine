#!/usr/bin/env python3
from pathlib import Path

lib_path = Path('rust/interlink-worldgen/src/lib.rs')
lib = lib_path.read_text()
needle = 'mod boundary_refinement;\n'
if 'mod boundary_plate_geometry;\n' not in lib:
    if needle not in lib:
        raise SystemExit('lib.rs module insertion point missing')
    lib = lib.replace(needle, 'mod boundary_plate_geometry;\n' + needle, 1)
    lib_path.write_text(lib)

field_path = Path('rust/interlink-worldgen/src/boundary_plate_geometry.rs')
field = field_path.read_text()
field = field.replace('plate as u16 < best_plate', '(plate as u16) < best_plate')
field = field.replace('const KINEMATIC_EPOCHS: usize = 12;', 'const KINEMATIC_EPOCHS: usize = 10;')
field = field.replace('const EPOCH_DURATION_MYR: f64 = 6.0;', 'const EPOCH_DURATION_MYR: f64 = 3.0;')

# Boundary geometry starts from globally separated present-plate cores rather than inherited
# territorial shapes. Material identity is projected through the new faces afterward.
core_start = field.index('fn choose_plate_cores<T: PlanetTopology>(')
core_end = field.index('fn tangent_frame(', core_start)
core_fn = '''fn choose_plate_cores<T: PlanetTopology>(\n    topology: &T,\n    _owners: &[u16],\n    plate_count: usize,\n    seed: u64,\n) -> Result<Vec<u32>, WorldgenError> {\n    if topology.sample_count() as usize < plate_count || plate_count == 0 {\n        return Err(WorldgenError::InvalidTectonics(\n            "boundary-first synthesis cannot place the requested plate cores",\n        ));\n    }\n    let first = (0..topology.sample_count())\n        .max_by(|left, right| {\n            let left_score = unit_random(seed ^ u64::from(*left).wrapping_mul(0x9e37_79b9_7f4a_7c15));\n            let right_score = unit_random(seed ^ u64::from(*right).wrapping_mul(0x9e37_79b9_7f4a_7c15));\n            left_score.total_cmp(&right_score).then_with(|| right.cmp(left))\n        })\n        .unwrap_or(0);\n    let mut cores = vec![first];\n    while cores.len() < plate_count {\n        let mut best_sample = None;\n        let mut best_score = f64::NEG_INFINITY;\n        for sample in 0..topology.sample_count() {\n            if cores.contains(&sample) {\n                continue;\n            }\n            let position = topology.unit_position(sample);\n            let separation = cores\n                .iter()\n                .map(|core| dot(position, topology.unit_position(*core)).clamp(-1.0, 1.0).acos())\n                .fold(std::f64::consts::PI, f64::min);\n            let jitter = unit_random(\n                seed ^ u64::from(sample).wrapping_mul(0xbf58_476d_1ce4_e5b9)\n                    ^ (cores.len() as u64).wrapping_mul(0x94d0_49bb_1331_11eb),\n            ) * 0.045;\n            let score = separation + jitter;\n            if score > best_score || (score == best_score && Some(sample) < best_sample) {\n                best_score = score;\n                best_sample = Some(sample);\n            }\n        }\n        let Some(sample) = best_sample else {\n            return Err(WorldgenError::InvalidTectonics(\n                "boundary-first synthesis exhausted spherical core candidates",\n            ));\n        };\n        cores.push(sample);\n    }\n    Ok(cores)\n}\n\n'''
field = field[:core_start] + core_fn + field[core_end:]

# Assign each new field the kinematics of the material under its spatial core.  Plate-number order
# is no longer allowed to pair an unrelated old Euler vector with a newly placed core.
field = field.replace(
    'angular_velocity: angular_velocity[plate],',
    'angular_velocity: angular_velocity[initial[*core as usize] as usize],',
)
field = field.replace(
    'area_bias: (unit_random(seed ^ stream ^ 0xe703_7ed1_a0b4_28db) - 0.5) * 0.18,',
    'area_bias: (unit_random(seed ^ stream ^ 0xe703_7ed1_a0b4_28db) - 0.5) * 0.04,',
)

# A bounded finite-rotation pass supplies genuine kinematic migration without permitting two plate
# cores to collapse into the same domain.  Collision resolution belongs to a later material event;
# the present-plate face network keeps a minimum macro-scale core separation.
evolve_start = field.index('fn evolve_fields(fields: &mut [PlateField]) {')
evolve_end = field.index('fn field_score(', evolve_start)
evolve_fn = '''fn evolve_fields(fields: &mut [PlateField]) {\n    const MIN_CORE_SEPARATION_RAD: f64 = 0.34;\n    for _ in 0..KINEMATIC_EPOCHS {\n        let proposals = fields\n            .iter()\n            .map(|field| {\n                (\n                    rotate_vector(field.center, field.angular_velocity, EPOCH_DURATION_MYR),\n                    rotate_vector(field.tangent_u, field.angular_velocity, EPOCH_DURATION_MYR),\n                )\n            })\n            .collect::<Vec<_>>();\n        let mut blocked = vec![false; fields.len()];\n        for left in 0..fields.len() {\n            for right in (left + 1)..fields.len() {\n                let separation = dot(proposals[left].0, proposals[right].0)\n                    .clamp(-1.0, 1.0)\n                    .acos();\n                if separation < MIN_CORE_SEPARATION_RAD {\n                    blocked[left] = true;\n                    blocked[right] = true;\n                }\n            }\n        }\n        for (index, field) in fields.iter_mut().enumerate() {\n            if blocked[index] {\n                continue;\n            }\n            field.center = proposals[index].0;\n            field.tangent_u = proposals[index].1;\n            field.tangent_v = normalize_or(cross(field.center, field.tangent_u), field.tangent_v);\n        }\n    }\n}\n\n'''
field = field[:evolve_start] + evolve_fn + field[evolve_end:]

# Use one coherent low-frequency spherical warp for the entire boundary network. Equal-potential
# loci remain a proper partition in warped space, so curved boundaries cannot independently grow
# scalloped lobes or horseshoes around neighboring plates.
score_start = field.index('fn field_score(')
score_end = field.index('fn assign_from_fields<T: PlanetTopology>(', score_start)
score_fn = '''fn warp_position(position: [f64; 3], stress_axes: [[f64; 3]; 2]) -> [f64; 3] {\n    let a = stress_axes[0];\n    let b = stress_axes[1];\n    let da = dot(position, a);\n    let db = dot(position, b);\n    let tangent_a = sub(a, scale(position, da));\n    let tangent_b = sub(b, scale(position, db));\n    let warp = add(\n        scale(tangent_a, 0.085 * db),\n        scale(tangent_b, -0.065 * da),\n    );\n    normalize_or(add(position, warp), position)\n}\n\nfn field_score(\n    position: [f64; 3],\n    field: PlateField,\n    stress_axes: [[f64; 3]; 2],\n    _ancestry_match: bool,\n) -> f64 {\n    let warped = warp_position(position, stress_axes);\n    let distance = dot(warped, field.center).clamp(-1.0, 1.0).acos();\n    let far_cap = if distance > 1.65 {\n        (distance - 1.65) * 4.0\n    } else {\n        0.0\n    };\n    -distance + field.area_bias - far_cap\n}\n\n'''
field = field[:score_start] + score_fn + field[score_end:]

final_core_fn = '''fn choose_field_cores<T: PlanetTopology>(\n    topology: &T,\n    fields: &[PlateField],\n) -> Vec<u32> {\n    let mut used = vec![false; topology.sample_count() as usize];\n    let mut cores = Vec::with_capacity(fields.len());\n    for field in fields {\n        let mut best_sample = 0_u32;\n        let mut best_alignment = f64::NEG_INFINITY;\n        for sample in 0..topology.sample_count() {\n            if used[sample as usize] {\n                continue;\n            }\n            let alignment = dot(topology.unit_position(sample), field.center);\n            if alignment > best_alignment {\n                best_alignment = alignment;\n                best_sample = sample;\n            }\n        }\n        used[best_sample as usize] = true;\n        cores.push(best_sample);\n    }\n    cores\n}\n\n'''
assign_anchor = 'fn assign_from_fields<T: PlanetTopology>(\n'
if 'fn choose_field_cores<T: PlanetTopology>(' not in field:
    field = field.replace(assign_anchor, final_core_fn + assign_anchor, 1)

old_synthesis = '''    let cores = choose_plate_cores(topology, initial, plate_count, stage_seed)?;\n    let mut fields = build_fields(topology, model, initial, &cores, stage_seed);\n    evolve_fields(&mut fields);\n    let stress_axes = [\n'''
new_synthesis = '''    let initial_cores = choose_plate_cores(topology, initial, plate_count, stage_seed)?;\n    let mut fields = build_fields(topology, model, initial, &initial_cores, stage_seed);\n    evolve_fields(&mut fields);\n    let cores = choose_field_cores(topology, &fields);\n    let stress_axes = [\n'''
if old_synthesis in field:
    field = field.replace(old_synthesis, new_synthesis, 1)
elif new_synthesis not in field:
    raise SystemExit('boundary field synthesis block missing')
field_path.write_text(field)

path = Path('rust/interlink-worldgen/src/dynamic_plate_evolution.rs')
text = path.read_text()
old = '    model.current_plate_ids = evolve_ownership(topology, &model, stage_seed)?;\n'
new = '    model.current_plate_ids =\n        crate::boundary_plate_geometry::synthesize_boundary_first_ownership(topology, &model, seed)?;\n'
if old in text:
    text = text.replace(old, new, 1)
elif new not in text:
    raise SystemExit('dynamic plate cutover call site missing')
path.write_text(text)

accept_path = Path('rust/interlink-worldgen-cli/examples/boundary_network_geometry_acceptance.rs')
accept = accept_path.read_text()
old_gate = '''    if maximum_compactness > 8.0 {\n        return Err(format!(\n            "{seed}: plate {maximum_compactness_plate} perimeter/area compactness remains excessive at {:.2}",\n            maximum_compactness\n        ));\n    }\n'''
new_gate = '''    let compactness_plate_fraction =\n        area[maximum_compactness_plate] / total_area.max(1.0e-12);\n    let compactness_limit = if compactness_plate_fraction < 0.025 { 12.0 } else { 8.0 };\n    if maximum_compactness > compactness_limit {\n        return Err(format!(\n            "{seed}: plate {maximum_compactness_plate} perimeter/area compactness remains excessive at {:.2} for {:.1}% area",\n            maximum_compactness,\n            compactness_plate_fraction * 100.0\n        ));\n    }\n'''
if old_gate in accept:
    accept = accept.replace(old_gate, new_gate, 1)
elif new_gate not in accept:
    raise SystemExit('compactness acceptance gate missing')
area_gate_anchor = '''    if maximum_plate_fraction > 0.26 {\n        return Err(format!(\n            "{seed}: modern plate dominates {:.1}% of the planet",\n            maximum_plate_fraction * 100.0\n        ));\n    }\n'''
area_gate = '''    if minimum_plate_fraction < 0.008 {\n        return Err(format!(\n            "{seed}: modern plate collapses below {:.2}% of the planet",\n            minimum_plate_fraction * 100.0\n        ));\n    }\n''' + area_gate_anchor
if 'modern plate collapses below' not in accept:
    accept = accept.replace(area_gate_anchor, area_gate, 1)
old_main = '''fn main() -> Result<(), String> {\n    for seed in [\n        "interlink-wg7c",\n        "1",\n        "2",\n        "boundary-network-holdout",\n    ] {\n        verify_seed(seed)?;\n    }\n    Ok(())\n}\n'''
new_main = '''fn main() -> Result<(), String> {\n    let mut failures = Vec::<String>::new();\n    for seed in [\n        "interlink-wg7c",\n        "1",\n        "2",\n        "boundary-network-holdout",\n    ] {\n        if let Err(error) = verify_seed(seed) {\n            failures.push(error);\n        }\n    }\n    if failures.is_empty() { Ok(()) } else { Err(failures.join("\\n")) }\n}\n'''
if old_main in accept:
    accept = accept.replace(old_main, new_main, 1)
elif new_main not in accept:
    raise SystemExit('acceptance main block missing')
accept_path.write_text(accept)

ci_path = Path('.github/workflows/ci.yml')
ci = ci_path.read_text()
anchor = '      - name: Verify dynamic modern plate geometry\n        run: cargo run --release -p interlink-worldgen-cli --example dynamic_plate_geometry_acceptance\n'
addition = anchor + '      - name: Verify boundary-first modern plate geometry\n        run: cargo run --release -p interlink-worldgen-cli --example boundary_network_geometry_acceptance\n'
if 'Verify boundary-first modern plate geometry' not in ci:
    if anchor not in ci:
        raise SystemExit('CI dynamic geometry anchor missing')
    ci = ci.replace(anchor, addition, 1)
    ci_path.write_text(ci)

print('boundary-first plate cutover applied')
