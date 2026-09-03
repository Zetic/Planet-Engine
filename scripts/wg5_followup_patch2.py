from pathlib import Path

path = Path('rust/interlink-worldgen-wasm/tests/climate_bridge.rs')
text = path.read_text()
old = '    assert_eq!(output.stage_version(), 1);\n'
new = '    assert_eq!(output.stage_version(), 2);\n'
if text.count(old) != 1:
    raise RuntimeError(f'expected one climate bridge stage-version assertion, found {text.count(old)}')
path.write_text(text.replace(old, new, 1))
print('WG-5 follow-up bridge-version patch applied')
