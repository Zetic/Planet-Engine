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
