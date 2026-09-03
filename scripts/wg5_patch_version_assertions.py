from pathlib import Path

path = Path('rust/interlink-worldgen-wasm/src/lib.rs')
text = path.read_text()
old = '        assert_eq!(worldgen_protocol_version(), 7);'
new = '        assert_eq!(worldgen_protocol_version(), 8);'
if old not in text:
    raise SystemExit('WASM protocol assertion marker not found')
path.write_text(text.replace(old, new, 1))

path = Path('rust/interlink-worldgen-wasm/tests/topography_bridge.rs')
text = path.read_text()
text = text.replace('    assert_eq!(output.generator_version(), 7);', '    assert_eq!(output.generator_version(), 8);', 1)
path.write_text(text)
