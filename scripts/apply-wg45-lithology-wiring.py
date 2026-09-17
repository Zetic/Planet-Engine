from pathlib import Path

lib = Path('rust/interlink-worldgen/src/lib.rs')
s = lib.read_text()
replacements = [
    ('mod lithosphere;\nmod orogen_provinces;', 'mod lithosphere;\nmod lithology;\nmod orogen_provinces;'),
    (
        'pub use lithosphere::{\n    LithosphereMetrics, LithosphereRequest, StructuralZoneKind, TectonicFragment,\n    TectonicFragmentKind, LITHOSPHERE_STAGE_ID, LITHOSPHERE_STAGE_VERSION, MAX_TECTONIC_FRAGMENTS,\n};\n',
        'pub use lithosphere::{\n    LithosphereMetrics, LithosphereRequest, StructuralZoneKind, TectonicFragment,\n    TectonicFragmentKind, LITHOSPHERE_STAGE_ID, LITHOSPHERE_STAGE_VERSION, MAX_TECTONIC_FRAGMENTS,\n};\npub use lithology::{\n    generate_lithology_substrate, BedrockClass, LithologyMetrics, LithologyRequest, LithologyState,\n    LITHOLOGY_STAGE_ID, LITHOLOGY_STAGE_VERSION,\n};\n',
    ),
    ('pub const WORLDGEN_ENGINE_VERSION: u32 = 20;', 'pub const WORLDGEN_ENGINE_VERSION: u32 = 21;'),
    ('    InvalidLithosphere(&\'static str),\n    InvalidRefinement', '    InvalidLithosphere(&\'static str),\n    InvalidLithology(&\'static str),\n    InvalidRefinement'),
    ('            | Self::InvalidLithosphere(message)\n            | Self::InvalidRefinement', '            | Self::InvalidLithosphere(message)\n            | Self::InvalidLithology(message)\n            | Self::InvalidRefinement'),
]
for old, new in replacements:
    if old not in s:
        raise SystemExit(f'lib anchor not found: {old[:80]!r}')
    s = s.replace(old, new, 1)
lib.write_text(s)

ci = Path('.github/workflows/ci.yml')
s = ci.read_text()
anchor = '''      - name: Verify continental freeboard and shelf hypsometry
        run: cargo run --release -p interlink-worldgen-cli --example continental_hypsometry_acceptance
'''
insert = anchor + '''      - name: Verify historical lithology and substrate
        run: cargo run --release -p interlink-worldgen-cli --example lithology_substrate_acceptance
'''
if anchor not in s:
    raise SystemExit('CI hypsometry anchor not found')
ci.write_text(s.replace(anchor, insert, 1))
