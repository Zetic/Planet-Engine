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
