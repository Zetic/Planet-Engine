from pathlib import Path

changed = 0
for path in sorted(Path('tests').glob('*.test.ts')):
    text = path.read_text()
    updated = text.replace('protocol v15', 'protocol v16')
    updated = updated.replace('protocol v15 ', 'protocol v16 ')
    updated = updated.replace('WORLDGEN_PROTOCOL_VERSION, 15', 'WORLDGEN_PROTOCOL_VERSION, 16')
    updated = updated.replace('const PROTOCOL = 15;', 'const PROTOCOL = 16;')
    if updated != text:
        path.write_text(updated)
        changed += 1
if changed < 7:
    raise SystemExit(f'expected to migrate at least seven protocol regression files, changed {changed}')
