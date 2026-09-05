from pathlib import Path


def replace_exact(path: str, old: str, new: str, label: str, count: int = 1) -> None:
    file_path = Path(path)
    text = file_path.read_text()
    found = text.count(old)
    if found != count:
        raise RuntimeError(f"{label}: expected {count} match(es), found {found}")
    file_path.write_text(text.replace(old, new, count))


# WG-4 remains cumulatively available, but the cumulative browser protocol and final
# physical frontier are now WG-7D / protocol v18.
replace_exact(
    'tests/wg4Topography.test.ts',
    "test('WG-4 browser contract remains available under protocol v17', () => {",
    "test('WG-4 browser contract remains available under protocol v18', () => {",
    'WG-4 protocol title',
)
replace_exact(
    'tests/wg4Topography.test.ts',
    "assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff\\/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, WG-7A fluvial erosion\\/sediment diagnostics, WG-7B bounded terrain evolution, and WG-7C post-erosion hydrology reconciliation/i);",
    "assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff\\/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, WG-7A fluvial erosion\\/sediment diagnostics, WG-7B bounded terrain evolution, WG-7C post-erosion hydrology reconciliation, and WG-7D bounded lake-sediment infill/i);",
    'WG-4 cumulative pipeline assertion',
)

# Dedicated command contracts also carry the cumulative protocol version.
replace_exact(
    'tests/wg6Drainage.test.ts',
    'assert.equal(command.protocolVersion, 17);',
    'assert.equal(command.protocolVersion, 18);',
    'WG-6A command protocol',
)

# WG-7A/WG-7B remain available as historical diagnostic stages while WG-7D is the
# current physical frontier.
for path, stage in [
    ('tests/wg7Erosion.test.ts', 'WG-7A'),
    ('tests/wg7TerrainEvolution.test.ts', 'WG-7B'),
]:
    replace_exact(
        path,
        f"test('{stage} cumulative browser contract is protocol v17 and single-request', () => {{",
        f"test('{stage} cumulative browser contract is protocol v18 and single-request', () => {{",
        f'{stage} protocol title',
    )
    replace_exact(
        path,
        'assert.match(html, /Current physical frontier: WG-7C/);',
        'assert.match(html, /Current physical frontier: WG-7D/);',
        f'{stage} current frontier',
    )

# WG-7C is retained as compact pre-infill ancestry/change diagnostics in v18; WG-7D
# owns the full canonical final hydrology state.
replace_exact(
    'tests/wg7Reconciliation.test.ts',
    "test('WG-7C cumulative browser contract is protocol v17 and memory-conscious', () => {",
    "test('WG-7C cumulative browser contract is protocol v18 and memory-conscious', () => {",
    'WG-7C protocol title',
)
replace_exact(
    'tests/wg7Reconciliation.test.ts',
    'assert.match(bridge, /reconciliation: PostErosionHydrologyState/);',
    "assert.match(bridge, /reconciliation: ReconciliationDiagnostics/);\n  assert.match(bridge, /infill: LakeSedimentInfillState/);",
    'WG-7C compact ancestry state',
)
replace_exact(
    'tests/wg7Reconciliation.test.ts',
    "assert.match(worker, /progress\\('packaging', 16, 17/);",
    "assert.match(worker, /progress\\('packaging', 17, 18/);",
    'WG-7C packaging progress',
)
replace_exact(
    'tests/wg7Reconciliation.test.ts',
    'assert.match(html, /Current physical frontier: WG-7C/);',
    'assert.match(html, /Current physical frontier: WG-7D/);',
    'WG-7C current frontier',
)
replace_exact(
    'tests/wg7Reconciliation.test.ts',
    "  assert.match(lab, /reconciliationMetrics\\.reconciledSeasonalHash/);\n",
    "  assert.match(lab, /reconciliationMetrics\\.reconciledSeasonalHash/);\n  assert.match(lab, /reconciliationMetrics\\.reconciledSeasonalHash !== loaded\\.infillMetrics\\.preInfillSeasonalHash/);\n  assert.match(lab, /infillMetrics\\.postInfillSeasonalHash !== loaded\\.seasonalMetrics\\.seasonalHydrologyHash/);\n",
    'WG-7C/WG-7D ancestry assertions',
)

# The low-level cumulative contracts are unchanged except for the protocol bump.
replace_exact(
    'tests/worldgenRewrite.test.ts',
    'const PROTOCOL = 17;',
    'const PROTOCOL = 18;',
    'legacy cumulative protocol constant',
)
replace_exact(
    'tests/worldgenRewrite.test.ts',
    "test('Planet Engine browser protocol v17 preserves WG-0 through WG-3.75 contracts', () => {",
    "test('Planet Engine browser protocol v18 preserves WG-0 through WG-3.75 contracts', () => {",
    'legacy cumulative protocol title',
)

# Composite views keep WG-7C diagnostics, but the physical-world result is now WG-7D.
replace_exact(
    'tests/worldgenCompositeViews.test.ts',
    "test('composite physical-world views reuse the WG-7C cumulative result without protocol changes', () => {",
    "test('composite physical-world views retain WG-7C diagnostics under protocol v18', () => {",
    'composite-view protocol title',
)
for old, new, label in [
    ("const protocol = readFileSync(join(root, 'src/worldgen/protocol.ts'), 'utf8');", "const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');", 'WG-7D protocol read'),
    ("const worker = readFileSync(join(root, 'src/worldgen/worldgenWorker.ts'), 'utf8');", "const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');", 'WG-7D worker read'),
    ("const lab = readFileSync(join(root, 'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts'), 'utf8');", "const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');", 'WG-7D lab read'),
]:
    replace_exact('tests/worldgenCompositeViews.test.ts', old, new, label)

print('WG-7D browser acceptance migration applied')
