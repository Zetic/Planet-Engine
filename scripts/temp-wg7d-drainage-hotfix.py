from pathlib import Path
import re


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


bridge_path = Path('rust/interlink-worldgen-wasm/src/climate_bridge.rs')
bridge = bridge_path.read_text()
start = bridge.index('    pub fn drainage_stage_id')
end = bridge.index('    pub fn runoff_stage_id', start)
section = bridge[start:end]
pattern = re.compile(r'self\.evolution\s*\.\s*post_erosion_drainage')
count = len(pattern.findall(section))
if count != 19:
    raise RuntimeError(f'canonical drainage reroute: expected 19 stale WG-7B references, found {count}')
section = pattern.sub('self.infill.post_infill_drainage', section)
if pattern.search(section):
    raise RuntimeError('canonical drainage reroute left a stale WG-7B reference')
if section.count('self.infill.post_infill_drainage') < 20:
    raise RuntimeError('canonical drainage reroute did not produce the expected WG-7D final-state references')
bridge = bridge[:start] + section + bridge[end:]
bridge_path.write_text(bridge)

rust_test_path = Path('rust/interlink-worldgen-wasm/tests/climate_bridge.rs')
rust_test = rust_test_path.read_text()
rust_test = replace_once(
    rust_test,
    '    assert!(output.atmospheric_shortwave_reflectivity() > 0.0);\n}',
    '''    assert!(output.atmospheric_shortwave_reflectivity() > 0.0);\n\n    // Canonical hydrology exposed to the browser must be one coherent WG-7D final state.\n    assert_eq!(\n        output.drainage_hash_hex(),\n        output.infill_post_infill_drainage_hash_hex()\n    );\n    assert_eq!(output.runoff_drainage_hash_hex(), output.drainage_hash_hex());\n    assert_eq!(output.lake_drainage_hash_hex(), output.drainage_hash_hex());\n    assert_eq!(\n        output.seasonal_drainage_hash_hex(),\n        output.drainage_hash_hex()\n    );\n}''',
    'runtime final drainage ancestry assertions',
)
rust_test_path.write_text(rust_test)

browser_test_path = Path('tests/worldgenCompositeViews.test.ts')
browser_test = browser_test_path.read_text()
addition = '''\n\ntest('WG-7D canonical browser drainage getters all source the post-infill state', () => {\n  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');\n  const start = bridge.indexOf('pub fn drainage_stage_id');\n  const end = bridge.indexOf('pub fn runoff_stage_id', start);\n  assert.notEqual(start, -1);\n  assert.notEqual(end, -1);\n  const drainageSection = bridge.slice(start, end);\n  assert.match(drainageSection, /self\\.infill\\.post_infill_drainage/);\n  assert.doesNotMatch(drainageSection, /self\\.evolution\\s*\\.\\s*post_erosion_drainage/);\n});\n'''
if 'WG-7D canonical browser drainage getters all source the post-infill state' not in browser_test:
    browser_test += addition
browser_test_path.write_text(browser_test)

print(f'WG-7D canonical drainage hotfix applied ({count} stale references rerouted)')
