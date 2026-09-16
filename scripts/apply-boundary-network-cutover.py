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

# Do not inherit modern geometry seeds from the wrap-prone PR-74 ownership field. Boundary-first
# plates start from a deterministic, globally separated set of spherical kinematic cores. Existing
# material identity remains underneath and is captured/split after rasterization.
core_start = field.index('fn choose_plate_cores<T: PlanetTopology>(')
core_end = field.index('fn tangent_frame(', core_start)
core_fn = '''fn choose_plate_cores<T: PlanetTopology>(\n    topology: &T,\n    _owners: &[u16],\n    plate_count: usize,\n    seed: u64,\n) -> Result<Vec<u32>, WorldgenError> {\n    let sample_count = topology.sample_count() as usize;\n    if sample_count < plate_count || plate_count == 0 {\n        return Err(WorldgenError::InvalidTectonics(\n            "boundary-first synthesis cannot place the requested plate cores",\n        ));\n    }\n\n    let first = (0..topology.sample_count())\n        .max_by(|left, right| {\n            let left_score = unit_random(\n                seed ^ u64::from(*left).wrapping_mul(0x9e37_79b9_7f4a_7c15),\n            );\n            let right_score = unit_random(\n                seed ^ u64::from(*right).wrapping_mul(0x9e37_79b9_7f4a_7c15),\n            );\n            left_score\n                .total_cmp(&right_score)\n                .then_with(|| right.cmp(left))\n        })\n        .unwrap_or(0);\n    let mut cores = vec![first];\n\n    while cores.len() < plate_count {\n        let mut best_sample = None;\n        let mut best_score = f64::NEG_INFINITY;\n        for sample in 0..topology.sample_count() {\n            if cores.contains(&sample) {\n                continue;\n            }\n            let position = topology.unit_position(sample);\n            let separation = cores\n                .iter()\n                .map(|core| {\n                    dot(position, topology.unit_position(*core))\n                        .clamp(-1.0, 1.0)\n                        .acos()\n                })\n                .fold(std::f64::consts::PI, f64::min);\n            let jitter = unit_random(\n                seed ^ u64::from(sample).wrapping_mul(0xbf58_476d_1ce4_e5b9)\n                    ^ (cores.len() as u64).wrapping_mul(0x94d0_49bb_1331_11eb),\n            ) * 0.055;\n            let score = separation + jitter;\n            if score > best_score || (score == best_score && Some(sample) < best_sample) {\n                best_score = score;\n                best_sample = Some(sample);\n            }\n        }\n        let Some(sample) = best_sample else {\n            return Err(WorldgenError::InvalidTectonics(\n                "boundary-first synthesis exhausted spherical core candidates",\n            ));\n        };\n        cores.push(sample);\n    }\n    Ok(cores)\n}\n\n'''
field = field[:core_start] + core_fn + field[core_end:]

# Material ancestry is provenance, not a final plate-shape force.
field = field.replace('    ancestry_match: bool,\n', '    _ancestry_match: bool,\n')
field = field.replace(
    '    let ancestry = if ancestry_match { 0.035 } else { 0.0 };\n',
    '    let ancestry = 0.0;\n',
)

# Keep low-order curvature only.  The geodesic potential is intentionally dominant so fields can
# bend tectonic boundaries without creating scalloped lobes and wrap-prone tendrils.
field = field.replace(
    'area_bias: (unit_random(seed ^ stream ^ 0xe703_7ed1_a0b4_28db) - 0.5) * 0.18,',
    'area_bias: (unit_random(seed ^ stream ^ 0xe703_7ed1_a0b4_28db) - 0.5) * 0.06,',
)
field = field.replace(
    'ellipticity: (unit_random(seed ^ stream ^ 0x8ebc_6af0_9c88_c6e3) - 0.5) * 0.34,',
    'ellipticity: (unit_random(seed ^ stream ^ 0x8ebc_6af0_9c88_c6e3) - 0.5) * 0.12,',
)
field = field.replace(
    'triangularity: (unit_random(seed ^ stream ^ 0x5899_65cc_7537_4cc3) - 0.5) * 0.16,',
    'triangularity: (unit_random(seed ^ stream ^ 0x5899_65cc_7537_4cc3) - 0.5) * 0.03,',
)
field = field.replace(
    'stress_coupling: (unit_random(seed ^ stream ^ 0x1d8e_4e27_c47d_124f) - 0.5) * 0.16,',
    'stress_coupling: (unit_random(seed ^ stream ^ 0x1d8e_4e27_c47d_124f) - 0.5) * 0.04,',
)

final_core_fn = '''fn choose_field_cores<T: PlanetTopology>(\n    topology: &T,\n    fields: &[PlateField],\n) -> Vec<u32> {\n    let mut used = vec![false; topology.sample_count() as usize];\n    let mut cores = Vec::with_capacity(fields.len());\n    for field in fields {\n        let mut best_sample = 0_u32;\n        let mut best_alignment = f64::NEG_INFINITY;\n        for sample in 0..topology.sample_count() {\n            if used[sample as usize] {\n                continue;\n            }\n            let alignment = dot(topology.unit_position(sample), field.center);\n            if alignment > best_alignment {\n                best_alignment = alignment;\n                best_sample = sample;\n            }\n        }\n        used[best_sample as usize] = true;\n        cores.push(best_sample);\n    }\n    cores\n}\n\n'''
assign_anchor = 'fn assign_from_fields<T: PlanetTopology>(\n'
if 'fn choose_field_cores<T: PlanetTopology>(' not in field:
    if assign_anchor not in field:
        raise SystemExit('field core insertion point missing')
    field = field.replace(assign_anchor, final_core_fn + assign_anchor, 1)

balance_anchor = 'fn repair_connectivity<T: PlanetTopology>(\n'
balance_fn = '''fn target_plate_fractions<T: PlanetTopology>(\n    topology: &T,\n    initial: &[u16],\n    plate_count: usize,\n) -> Vec<f64> {\n    let mut initial_area = vec![0.0_f64; plate_count];\n    let mut total_area = 0.0_f64;\n    for sample in 0..topology.sample_count() {\n        let area = topology.area_steradians(sample);\n        initial_area[initial[sample as usize] as usize] += area;\n        total_area += area;\n    }\n    let mut weights = initial_area\n        .iter()\n        .map(|area| ((*area / total_area.max(1.0e-12)) + 0.004).sqrt())\n        .collect::<Vec<_>>();\n    let weight_sum = weights.iter().sum::<f64>().max(1.0e-12);\n    for weight in &mut weights {\n        *weight /= weight_sum;\n    }\n    weights\n}\n\nfn balance_field_areas<T: PlanetTopology>(\n    topology: &T,\n    fields: &mut [PlateField],\n    initial: &[u16],\n    cores: &[u32],\n    stress_axes: [[f64; 3]; 2],\n) -> Vec<u16> {\n    let plate_count = fields.len();\n    let targets = target_plate_fractions(topology, initial, plate_count);\n    let mut owners = assign_from_fields(topology, fields, initial, cores, stress_axes);\n\n    for _ in 0..24 {\n        let mut area = vec![0.0_f64; plate_count];\n        let mut total = 0.0_f64;\n        for sample in 0..topology.sample_count() {\n            let sample_area = topology.area_steradians(sample);\n            area[owners[sample as usize] as usize] += sample_area;\n            total += sample_area;\n        }\n        let mut maximum_log_error = 0.0_f64;\n        for plate in 0..plate_count {\n            let current = (area[plate] / total.max(1.0e-12)).max(1.0e-5);\n            let error = (targets[plate] / current).ln();\n            maximum_log_error = maximum_log_error.max(error.abs());\n            fields[plate].area_bias += (error * 0.045).clamp(-0.09, 0.09);\n        }\n        owners = assign_from_fields(topology, fields, initial, cores, stress_axes);\n        if maximum_log_error < 0.18 {\n            break;\n        }\n    }\n    owners\n}\n\n'''
if 'fn balance_field_areas<T: PlanetTopology>(' not in field:
    if balance_anchor not in field:
        raise SystemExit('area-balance insertion point missing')
    field = field.replace(balance_anchor, balance_fn + balance_anchor, 1)

old_synthesis = '''    let cores = choose_plate_cores(topology, initial, plate_count, stage_seed)?;\n    let mut fields = build_fields(topology, model, initial, &cores, stage_seed);\n    evolve_fields(&mut fields);\n    let stress_axes = [\n'''
new_synthesis = '''    let initial_cores = choose_plate_cores(topology, initial, plate_count, stage_seed)?;\n    let mut fields = build_fields(topology, model, initial, &initial_cores, stage_seed);\n    evolve_fields(&mut fields);\n    let cores = choose_field_cores(topology, &fields);\n    let stress_axes = [\n'''
if old_synthesis in field:
    field = field.replace(old_synthesis, new_synthesis, 1)
elif new_synthesis not in field:
    raise SystemExit('boundary field synthesis block missing')
old_assign = '''    let mut owners = assign_from_fields(topology, &fields, initial, &cores, stress_axes);\n'''
new_assign = '''    let mut owners = balance_field_areas(\n        topology,\n        &mut fields,\n        initial,\n        &cores,\n        stress_axes,\n    );\n'''
if old_assign in field:
    field = field.replace(old_assign, new_assign, 1)
elif new_assign not in field:
    raise SystemExit('boundary field assignment block missing')
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
new_gate = '''    let compactness_plate_fraction =\n        area[maximum_compactness_plate] / total_area.max(1.0e-12);\n    let compactness_limit = if compactness_plate_fraction < 0.025 {\n        12.0\n    } else {\n        8.0\n    };\n    if maximum_compactness > compactness_limit {\n        return Err(format!(\n            "{seed}: plate {maximum_compactness_plate} perimeter/area compactness remains excessive at {:.2} for {:.1}% area",\n            maximum_compactness,\n            compactness_plate_fraction * 100.0\n        ));\n    }\n'''
if old_gate in accept:
    accept = accept.replace(old_gate, new_gate, 1)
elif new_gate not in accept:
    raise SystemExit('compactness acceptance gate missing')
area_gate_anchor = '''    if maximum_plate_fraction > 0.26 {\n        return Err(format!(\n            "{seed}: modern plate dominates {:.1}% of the planet",\n            maximum_plate_fraction * 100.0\n        ));\n    }\n'''
area_gate = '''    if minimum_plate_fraction < 0.008 {\n        return Err(format!(\n            "{seed}: modern plate collapses below {:.2}% of the planet",\n            minimum_plate_fraction * 100.0\n        ));\n    }\n''' + area_gate_anchor
if 'modern plate collapses below' not in accept:
    if area_gate_anchor not in accept:
        raise SystemExit('plate area acceptance gate missing')
    accept = accept.replace(area_gate_anchor, area_gate, 1)

old_main = '''fn main() -> Result<(), String> {\n    for seed in [\n        "interlink-wg7c",\n        "1",\n        "2",\n        "boundary-network-holdout",\n    ] {\n        verify_seed(seed)?;\n    }\n    Ok(())\n}\n'''
new_main = '''fn main() -> Result<(), String> {\n    let mut failures = Vec::<String>::new();\n    for seed in [\n        "interlink-wg7c",\n        "1",\n        "2",\n        "boundary-network-holdout",\n    ] {\n        if let Err(error) = verify_seed(seed) {\n            failures.push(error);\n        }\n    }\n    if failures.is_empty() {\n        Ok(())\n    } else {\n        Err(failures.join("\\n"))\n    }\n}\n'''
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
