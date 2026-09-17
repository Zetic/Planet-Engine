from pathlib import Path

p = Path('rust/interlink-worldgen-wasm/src/lib.rs')
s = p.read_text()
old = 'pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 21;'
if old not in s:
    raise SystemExit('Rust WASM protocol v21 anchor not found')
p.write_text(s.replace(old, 'pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 22;', 1))

for p in Path('tests').glob('*.test.ts'):
    s = p.read_text()
    updated = s.replace('protocol v21', 'protocol v22')
    updated = updated.replace('protocol V21', 'protocol V22')
    updated = updated.replace('WORLDGEN_PROTOCOL_VERSION, 21', 'WORLDGEN_PROTOCOL_VERSION, 22')
    updated = updated.replace('WORLDGEN_PROTOCOL_VERSION = 21', 'WORLDGEN_PROTOCOL_VERSION = 22')
    updated = updated.replace('const PROTOCOL = 21;', 'const PROTOCOL = 22;')
    updated = updated.replace('/WORLDGEN_PROTOCOL_VERSION = 21/', '/WORLDGEN_PROTOCOL_VERSION = 22/')
    if updated != s:
        p.write_text(updated)
