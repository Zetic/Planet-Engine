from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if new in text:
        return
    if old not in text:
        raise SystemExit(f"missing patch anchor in {path}: {old!r}")
    p.write_text(text.replace(old, new, 1))


# Make the accepted inheritance identity explicit so downstream stages cannot accidentally
# dereference the legacy compatibility payload when they mean the causal state identity.
replace_once(
    'rust/interlink-worldgen/src/causal_pipeline.rs',
    '''impl InheritedPhysicalState {\n    pub fn inheritance_hash_hex(&self) -> String {\n        format!("{:016x}", self.causal_inheritance_hash)\n    }\n''',
    '''impl InheritedPhysicalState {\n    pub fn inheritance_hash(&self) -> u64 {\n        self.causal_inheritance_hash\n    }\n    pub fn inheritance_hash_hex(&self) -> String {\n        format!("{:016x}", self.inheritance_hash())\n    }\n''',
)

p = Path('rust/interlink-worldgen/src/erosion.rs')
text = p.read_text()
old = 'inherited.inheritance_hash,'
count = text.count(old)
if count != 2:
    raise SystemExit(f'expected exactly two stale WG-7A legacy inheritance identity reads, found {count}')
text = text.replace(old, 'inherited.inheritance_hash(),')
text = text.replace(
    'pub const FLUVIAL_EROSION_STAGE_VERSION: u32 = 1;',
    'pub const FLUVIAL_EROSION_STAGE_VERSION: u32 = 2;',
    1,
)
p.write_text(text)

# Turn the existing low-level WG-7A benchmark into a causal identity smoke as well.
replace_once(
    'rust/interlink-worldgen-cli/examples/erosion_performance.rs',
    '''    let state = last.expect("at least one WG-7A benchmark run");\n    let mean_ms = durations_ms.iter().sum::<f64>() / durations_ms.len() as f64;\n''',
    '''    let state = last.expect("at least one WG-7A benchmark run");\n    assert_eq!(\n        state.metrics.inheritance_hash,\n        inherited.inheritance_hash(),\n        "WG-7A must record the accepted causal fine-state inheritance identity",\n    );\n    let mean_ms = durations_ms.iter().sum::<f64>() / durations_ms.len() as f64;\n''',
)

print('WG-7A causal identity fix applied')
