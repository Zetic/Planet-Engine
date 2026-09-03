from pathlib import Path

path = Path('rust/interlink-worldgen-wasm/src/lib.rs')
text = path.read_text()
marker = 'mod inheritance_bridge;\npub use inheritance_bridge::WasmWorldgenInheritance;\n'
insert = 'mod climate_bridge;\npub use climate_bridge::WasmWorldgenClimate;\n'
if insert not in text:
    if marker not in text:
        raise SystemExit('WASM bridge export marker not found')
    text = text.replace(marker, insert + marker, 1)
path.write_text(text)
