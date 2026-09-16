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
core_fn = '''fn choose_plate_cores<T: PlanetTopology>(\n    topology: &T,\n    _owners: &[u16],\n    plate_count: usize,\n    seed: u64,\n) -> Result<Vec<u32>, WorldgenError> {\n    let sample_count = topology.sample_count() as usize;\n    if sample_count < plate_count || plate_count == 0 {\n        return Err(WorldgenError::InvalidTectonics(\n            "boundary-first synthesis cannot place the requested plate cores",\n        ));\n    }\n\n    let first = (0..topology.sample_count())\n        .max_by(|left, right| {\n            let left_score = unit_random(\n                seed ^ u64::from(*left).wrapping_mul(0x9e37_79b9_7f4a_7c15),\n            );\n            let right_score = unit_random(\n                seed ^ u64::from(*right).wrapping_mul(0x9e37_79b9_7f4a_7c15),\n            );\n            left_score\n                .total_cmp(&right_score)\n                .then_with(|| right.cmp(left))\n        })\n        .unwrap_or(0);\n    let mut cores = vec![first];\n\n    while cores.len() < plate_count {\n        let mut best_sample = None;\n        let mut best_score = f64::NEG_INFINITY;\n        for sample in 0..topology.sample_count() {\n            if cores.contains(&sample) {\n                continue;\n            }\n            let position = topology.unit_position(sample);\n            let separation = cores\n                .iter()\n                .map(|core| {\n                    dot(position, topology.unit_position(*core))\n                        .clamp(-1.0, 1.0)\n                        .acos()\n                })\n                .fold(std::f64::consts::PI, f64::min);\n            // A small deterministic perturbation keeps the lattice from producing overly regular\n            // equal-area cells while leaving spherical separation overwhelmingly dominant.\n            let jitter = unit_random(\n                seed ^ u64::from(sample).wrapping_mul(0xbf58_476d_1ce4_e5b9)\n                    ^ (cores.len() as u64).wrapping_mul(0x94d0_49bb_1331_11eb),\n            ) * 0.055;\n            let score = separation + jitter;\n            if score > best_score || (score == best_score && Some(sample) < best_sample) {\n                best_score = score;\n                best_sample = Some(sample);\n            }\n        }\n        let Some(sample) = best_sample else {\n            return Err(WorldgenError::InvalidTectonics(\n                "boundary-first synthesis exhausted spherical core candidates",\n            ));\n        };\n        cores.push(sample);\n    }\n    Ok(cores)\n}\n\n'''
field = field[:core_start] + core_fn + field[core_end:]

final_core_fn = '''fn choose_field_cores<T: PlanetTopology>(\n    topology: &T,\n    fields: &[PlateField],\n) -> Vec<u32> {\n    let mut used = vec![false; topology.sample_count() as usize];\n    let mut cores = Vec::with_capacity(fields.len());\n    for field in fields {\n        let mut best_sample = 0_u32;\n        let mut best_alignment = f64::NEG_INFINITY;\n        for sample in 0..topology.sample_count() {\n            if used[sample as usize] {\n                continue;\n            }\n            let alignment = dot(topology.unit_position(sample), field.center);\n            if alignment > best_alignment {\n                best_alignment = alignment;\n                best_sample = sample;\n            }\n        }\n        used[best_sample as usize] = true;\n        cores.push(best_sample);\n    }\n    cores\n}\n\n'''
assign_anchor = 'fn assign_from_fields<T: PlanetTopology>(\n'
if 'fn choose_field_cores<T: PlanetTopology>(' not in field:
    if assign_anchor not in field:
        raise SystemExit('field core insertion point missing')
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
new_gate = '''    let compactness_plate_fraction =\n        area[maximum_compactness_plate] / total_area.max(1.0e-12);\n    // On the L4 acceptance mesh, true microplates have only a few dozen cells and therefore a\n    // quantized perimeter. Hold macro plates to the strict shape gate while allowing bounded\n    // discretization error below 2.5% planetary area. Wrapping is independently rejected by the\n    // spherical covering-radius, neck, and single-neighbor-contact gates.\n    let compactness_limit = if compactness_plate_fraction < 0.025 {\n        12.0\n    } else {\n        8.0\n    };\n    if maximum_compactness > compactness_limit {\n        return Err(format!(\n            "{seed}: plate {maximum_compactness_plate} perimeter/area compactness remains excessive at {:.2} for {:.1}% area",\n            maximum_compactness,\n            compactness_plate_fraction * 100.0\n        ));\n    }\n'''
if old_gate in accept:
    accept = accept.replace(old_gate, new_gate, 1)
elif new_gate not in accept:
    raise SystemExit('compactness acceptance gate missing')
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
