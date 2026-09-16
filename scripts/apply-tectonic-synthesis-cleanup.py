from pathlib import Path
import re


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    assert count == 1, f"{label}: expected 1 occurrence, found {count}"
    return text.replace(old, new, 1)


# Physical output changes in this PR require a new engine identity.
lib = Path('rust/interlink-worldgen/src/lib.rs')
text = lib.read_text()
text = replace_once(text, 'pub const WORLDGEN_ENGINE_VERSION: u32 = 16;', 'pub const WORLDGEN_ENGINE_VERSION: u32 = 17;', 'engine version')
lib.write_text(text)

# The cumulative browser packet now carries compact historical identity/morphology fields.
wasm_lib = Path('rust/interlink-worldgen-wasm/src/lib.rs')
text = wasm_lib.read_text()
text = replace_once(text, 'pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 20;', 'pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 21;', 'wasm protocol')
wasm_lib.write_text(text)

protocol = Path('src/worldgen/protocol.ts')
text = protocol.read_text()
text = replace_once(text, 'export const WORLDGEN_PROTOCOL_VERSION = 20;', 'export const WORLDGEN_PROTOCOL_VERSION = 21;', 'browser protocol')
needle = '''  positions: Float64Array;\n  neighborOffsets: Uint32Array;\n  neighbors: Uint32Array;\n  plateIds: Uint16Array;\n  crustKind: Uint8Array;\n'''
replacement = '''  positions: Float64Array;\n  neighborOffsets: Uint32Array;\n  neighbors: Uint32Array;\n  originPlateIds: Uint16Array;\n  historicalFragmentIds: Uint16Array;\n  currentPlateIds: Uint16Array;\n  crustProvinceId: Uint16Array;\n  crustBirthAgeMyr: Float32Array;\n  latestHistoricalEventKind: Uint8Array;\n  latestHistoricalEventAgeMyr: Float32Array;\n  historicalRiftIntensity: Float32Array;\n  historicalRiftAgeMyr: Float32Array;\n  historicalShearIntensity: Float32Array;\n  historicalSutureIntensity: Float32Array;\n  historicalSutureAgeMyr: Float32Array;\n  passiveMarginIndex: Float32Array;\n  activeOrogenIntensity: Float32Array;\n  fossilOrogenIntensity: Float32Array;\n  plateIds: Uint16Array;\n  crustKind: Uint8Array;\n'''
# Only change the cumulative climate packet, not earlier result interfaces.
start = text.index('export interface WorldgenClimateResult')
prefix, body = text[:start], text[start:]
body = replace_once(body, needle, replacement, 'climate historical packet fields')
protocol.write_text(prefix + body)

# Make the cumulative WASM generator own the historical frontend rather than reconstructing
# modern WG2/WG3/WG3.5 independently, and retain only compact fine diagnostic fields.
bridge = Path('rust/interlink-worldgen-wasm/src/climate_bridge.rs')
text = bridge.read_text()
text = replace_once(
    text,
    'use interlink_worldgen::{\n',
    'use interlink_worldgen::{\n    build_historical_tectonic_morphology, generate_historical_frontend,\n    generate_lithosphere_from_history, inherit_historical_identity, HistoricalLithosphereRequest,\n    InheritedHistoricalIdentity,\n',
    'climate historical imports',
)
text = replace_once(
    text,
    '    inherited: InheritedPhysicalState,\n    boundaries: InheritedBoundarySet,\n',
    '''    inherited: InheritedPhysicalState,\n    historical_identity: InheritedHistoricalIdentity,\n    historical_morphology_hash: String,\n    latest_event_kind: Vec<u8>,\n    latest_event_age_myr: Vec<f32>,\n    historical_rift_intensity: Vec<f32>,\n    historical_rift_age_myr: Vec<f32>,\n    historical_shear_intensity: Vec<f32>,\n    historical_suture_intensity: Vec<f32>,\n    historical_suture_age_myr: Vec<f32>,\n    passive_margin_index: Vec<f32>,\n    active_orogen_intensity: Vec<f32>,\n    fossil_orogen_intensity: Vec<f32>,\n    boundaries: InheritedBoundarySet,\n''',
    'climate struct history fields',
)
pattern = re.compile(
    r'''        report_generation_progress\(progress, "tectonics", 2, 0, 1\);\n.*?        report_generation_progress\(progress, "lithosphere", 4, 1, 1\);\n''',
    re.S,
)
new_pipeline = '''        report_generation_progress(progress, "tectonics", 2, 0, 1);\n        let frontend = generate_historical_frontend(\n            &coarse_topology,\n            &HistoricalLithosphereRequest::new(seed.as_str(), plate_count),\n            planet,\n        )\n        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, "tectonics", 2, 1, 1);\n        report_generation_progress(progress, "geology", 3, 0, 1);\n        let historical_identity = inherit_historical_identity(\n            &fine_topology,\n            coarse_level,\n            &frontend.historical,\n        )\n        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let morphology = build_historical_tectonic_morphology(\n            &coarse_topology,\n            &frontend.historical,\n            &frontend.tectonics,\n            seed.as_str(),\n        )\n        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, "geology", 3, 1, 1);\n        report_generation_progress(progress, "lithosphere", 4, 0, 1);\n        let lithosphere = generate_lithosphere_from_history(\n            &coarse_topology,\n            &frontend.historical,\n            &frontend.tectonics,\n            &frontend.geology,\n            &LithosphereRequest::new(seed.as_str()),\n        )\n        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let tectonics = frontend.tectonics;\n        let geology = frontend.geology;\n        report_generation_progress(progress, "lithosphere", 4, 1, 1);\n'''
text, count = pattern.subn(new_pipeline, text, count=1)
assert count == 1, f'climate historical pipeline replacement count={count}'
inherit_done = '        report_generation_progress(progress, "inheritance", 5, 1, 1);\n'
history_projection = '''        report_generation_progress(progress, "inheritance", 5, 1, 1);\n        let nearest = &inherited.map.nearest_coarse_source;\n        let inherit_f32 = |values: &[f32]| {\n            nearest\n                .iter()\n                .map(|source| values[*source as usize])\n                .collect::<Vec<_>>()\n        };\n        let inherit_u8 = |values: &[u8]| {\n            nearest\n                .iter()\n                .map(|source| values[*source as usize])\n                .collect::<Vec<_>>()\n        };\n        let latest_event_kind = inherit_u8(&morphology.latest_event_kind);\n        let latest_event_age_myr = inherit_f32(&morphology.latest_event_age_myr);\n        let historical_rift_intensity = inherit_f32(&morphology.rift_intensity);\n        let historical_rift_age_myr = inherit_f32(&morphology.rift_age_myr);\n        let historical_shear_intensity = inherit_f32(&morphology.shear_intensity);\n        let historical_suture_intensity = inherit_f32(&morphology.suture_intensity);\n        let historical_suture_age_myr = inherit_f32(&morphology.suture_age_myr);\n        let passive_margin_index = inherit_f32(&morphology.passive_margin_index);\n        let active_orogen_intensity = inherit_f32(&morphology.active_orogen_intensity);\n        let fossil_orogen_intensity = inherit_f32(&morphology.fossil_orogen_intensity);\n        let historical_morphology_hash = morphology.metrics.morphology_hash_hex();\n'''
text = replace_once(text, inherit_done, history_projection, 'historical morphology projection')
text = replace_once(
    text,
    '            fine_topology,\n            inherited,\n            boundaries,\n',
    '''            fine_topology,\n            inherited,\n            historical_identity,\n            historical_morphology_hash,\n            latest_event_kind,\n            latest_event_age_myr,\n            historical_rift_intensity,\n            historical_rift_age_myr,\n            historical_shear_intensity,\n            historical_suture_intensity,\n            historical_suture_age_myr,\n            passive_margin_index,\n            active_orogen_intensity,\n            fossil_orogen_intensity,\n            boundaries,\n''',
    'climate Self history fields',
)
getter_anchor = '''    pub fn neighbors(&self) -> Vec<u32> {\n        self.fine_topology.neighbor_indices().to_vec()\n    }\n\n'''
history_getters = '''    pub fn neighbors(&self) -> Vec<u32> {\n        self.fine_topology.neighbor_indices().to_vec()\n    }\n\n    pub fn historical_identity_hash_hex(&self) -> String {\n        self.historical_identity.identity_hash_hex()\n    }\n    pub fn historical_morphology_hash_hex(&self) -> String {\n        self.historical_morphology_hash.clone()\n    }\n    pub fn origin_plate_ids(&self) -> Vec<u16> {\n        self.historical_identity.origin_plate_ids.clone()\n    }\n    pub fn historical_fragment_ids(&self) -> Vec<u16> {\n        self.historical_identity.fragment_ids.clone()\n    }\n    pub fn current_plate_ids(&self) -> Vec<u16> {\n        self.historical_identity.current_plate_ids.clone()\n    }\n    pub fn crust_province_id(&self) -> Vec<u16> {\n        self.inherited.crust_province_id.clone()\n    }\n    pub fn crust_birth_age_myr(&self) -> Vec<f32> {\n        self.historical_identity.crust_birth_age_myr.clone()\n    }\n    pub fn latest_historical_event_kind(&self) -> Vec<u8> {\n        self.latest_event_kind.clone()\n    }\n    pub fn latest_historical_event_age_myr(&self) -> Vec<f32> {\n        self.latest_event_age_myr.clone()\n    }\n    pub fn historical_rift_intensity(&self) -> Vec<f32> {\n        self.historical_rift_intensity.clone()\n    }\n    pub fn historical_rift_age_myr(&self) -> Vec<f32> {\n        self.historical_rift_age_myr.clone()\n    }\n    pub fn historical_shear_intensity(&self) -> Vec<f32> {\n        self.historical_shear_intensity.clone()\n    }\n    pub fn historical_suture_intensity(&self) -> Vec<f32> {\n        self.historical_suture_intensity.clone()\n    }\n    pub fn historical_suture_age_myr(&self) -> Vec<f32> {\n        self.historical_suture_age_myr.clone()\n    }\n    pub fn passive_margin_index(&self) -> Vec<f32> {\n        self.passive_margin_index.clone()\n    }\n    pub fn active_orogen_intensity(&self) -> Vec<f32> {\n        self.active_orogen_intensity.clone()\n    }\n    pub fn fossil_orogen_intensity(&self) -> Vec<f32> {\n        self.fossil_orogen_intensity.clone()\n    }\n\n'''
text = replace_once(text, getter_anchor, history_getters, 'historical climate getters')
bridge.write_text(text)

# Worker typing and packet construction.
worker = Path('src/worldgen/worldgenWorker.ts')
text = worker.read_text()
wasm_methods_anchor = '  positions(): Float64Array;  neighbor_offsets(): Uint32Array; neighbors(): Uint32Array;\n  plate_ids(): Uint16Array; crust_kind(): Uint8Array;'
wasm_methods_replacement = '''  positions(): Float64Array;  neighbor_offsets(): Uint32Array; neighbors(): Uint32Array;\n  historical_identity_hash_hex(): string; historical_morphology_hash_hex(): string;\n  origin_plate_ids(): Uint16Array; historical_fragment_ids(): Uint16Array; current_plate_ids(): Uint16Array; crust_province_id(): Uint16Array; crust_birth_age_myr(): Float32Array;\n  latest_historical_event_kind(): Uint8Array; latest_historical_event_age_myr(): Float32Array; historical_rift_intensity(): Float32Array; historical_rift_age_myr(): Float32Array; historical_shear_intensity(): Float32Array; historical_suture_intensity(): Float32Array; historical_suture_age_myr(): Float32Array; passive_margin_index(): Float32Array; active_orogen_intensity(): Float32Array; fossil_orogen_intensity(): Float32Array;\n  plate_ids(): Uint16Array; crust_kind(): Uint8Array;'''
text = replace_once(text, wasm_methods_anchor, wasm_methods_replacement, 'WasmClimate history methods')
climate_arrays = '''    const plateIds = output.plate_ids(); const crustKind = output.crust_kind(); const nearestCoarseSource = output.nearest_coarse_source();'''
climate_arrays_new = '''    const originPlateIds = output.origin_plate_ids(); const historicalFragmentIds = output.historical_fragment_ids(); const currentPlateIds = output.current_plate_ids(); const crustProvinceId = output.crust_province_id(); const crustBirthAgeMyr = output.crust_birth_age_myr();\n    const latestHistoricalEventKind = output.latest_historical_event_kind(); const latestHistoricalEventAgeMyr = output.latest_historical_event_age_myr(); const historicalRiftIntensity = output.historical_rift_intensity(); const historicalRiftAgeMyr = output.historical_rift_age_myr(); const historicalShearIntensity = output.historical_shear_intensity(); const historicalSutureIntensity = output.historical_suture_intensity(); const historicalSutureAgeMyr = output.historical_suture_age_myr(); const passiveMarginIndex = output.passive_margin_index(); const activeOrogenIntensity = output.active_orogen_intensity(); const fossilOrogenIntensity = output.fossil_orogen_intensity();\n    const plateIds = output.plate_ids(); const crustKind = output.crust_kind(); const nearestCoarseSource = output.nearest_coarse_source();'''
# This string also appears in topography packaging; apply only in generateClimate tail.
climate_start = text.index('async function generateClimate')
prefix, tail = text[:climate_start], text[climate_start:]
tail = replace_once(tail, climate_arrays, climate_arrays_new, 'worker climate historical arrays')
result_anchor = '      positions, neighborOffsets, neighbors, plateIds, crustKind, nearestCoarseSource, inheritedSampleMask, crustAgeMyr, crustThicknessKm,'
result_new = '      positions, neighborOffsets, neighbors, originPlateIds, historicalFragmentIds, currentPlateIds, crustProvinceId, crustBirthAgeMyr, latestHistoricalEventKind, latestHistoricalEventAgeMyr, historicalRiftIntensity, historicalRiftAgeMyr, historicalShearIntensity, historicalSutureIntensity, historicalSutureAgeMyr, passiveMarginIndex, activeOrogenIntensity, fossilOrogenIntensity, plateIds, crustKind, nearestCoarseSource, inheritedSampleMask, crustAgeMyr, crustThicknessKm,'
tail = replace_once(tail, result_anchor, result_new, 'worker climate result history fields')
worker.write_text(prefix + tail)

# Main interactive lab: put the historical ancestry/morphology views directly beside production output.
index = Path('index.html')
text = index.read_text()
text = replace_once(
    text,
    '        <p>Generate one deterministic physical planet through WG-7D, then inspect topology, tectonics, geology, lithosphere, initial topography, climate, canonical final drainage, lakes, seasonal realized flow, erosive forcing, sediment routing, terrain evolution, post-erosion reconciliation, and lake-sediment infill from one matched physical state.</p>',
    '        <p>Generate one deterministic physical planet through WG-7D, then inspect historical material ancestry, tectonic synthesis, geology, lithosphere, topography, climate, hydrology, erosion, sediment routing, terrain evolution, reconciliation, and lake-sediment infill from one matched physical state.</p>',
    'main lab header',
)
tectonics_group = '          <optgroup label="Tectonics">\n            <option value="plates">Macro plate ownership</option>'
historical_group = '''          <optgroup label="Historical material / tectonic synthesis">\n            <option value="historical-origin">Ancestral origin plates</option>\n            <option value="historical-fragments">Persistent crust fragments</option>\n            <option value="historical-current">Current plate ownership</option>\n            <option value="historical-provenance">WG-3 crust provenance</option>\n            <option value="historical-crust-birth-age">Crust formation / birth age</option>\n            <option value="historical-event">Latest historical event type</option>\n            <option value="historical-event-age">Latest historical event age</option>\n            <option value="historical-rift">Historical rift intensity</option>\n            <option value="historical-rift-age">Historical rift age</option>\n            <option value="historical-shear">Historical shear intensity</option>\n            <option value="historical-suture">Historical suture intensity</option>\n            <option value="historical-suture-age">Historical suture age</option>\n            <option value="historical-passive-margin">Passive margin potential</option>\n            <option value="historical-active-orogen">Active orogen intensity</option>\n            <option value="historical-fossil-orogen">Fossil orogen intensity</option>\n          </optgroup>\n          <optgroup label="Tectonics">\n            <option value="plates">Macro plate ownership</option>'''
text = replace_once(text, tectonics_group, historical_group, 'historical main-lab options')
index.write_text(text)

lab = Path('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts')
text = lab.read_text()
plate_color = "function plateColor(id: number): string { return `hsl(${(id * 137.507764 + 18) % 360} 60% 55%)`; }"
color_helpers = plate_color + '''\nfunction historicalIdentityColor(id: number, offset = 42): string { return `hsl(${(id * 137.507764 + offset) % 360} 62% 55%)`; }\nfunction historicalEventColor(kind: number): string {\n  if (kind === 1) return '#f59e42';\n  if (kind === 2) return '#50b9e8';\n  if (kind === 3) return '#7656d6';\n  if (kind === 4) return '#e94f4f';\n  if (kind === 5) return '#e8d35a';\n  if (kind === 6) return '#5dd18b';\n  if (kind === 7) return '#c178df';\n  return '#101923';\n}'''
text = replace_once(text, plate_color, color_helpers, 'historical lab colors')
scalar_anchor = "    case 'crust-age': return { values: result.crustAgeMyr, minimum: 0, maximum: 3_500, lowHue: 205, highHue: 24 };"
scalar_history = '''    case 'historical-crust-birth-age': return { values: result.crustBirthAgeMyr, minimum: 0, maximum: 3_500, lowHue: 205, highHue: 24 };\n    case 'historical-event-age': return { values: result.latestHistoricalEventAgeMyr, minimum: 0, maximum: 350, lowHue: 205, highHue: 24 };\n    case 'historical-rift': return { values: result.historicalRiftIntensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 25 };\n    case 'historical-rift-age': return { values: result.historicalRiftAgeMyr, minimum: 0, maximum: 350, lowHue: 205, highHue: 24 };\n    case 'historical-shear': return { values: result.historicalShearIntensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 55 };\n    case 'historical-suture': return { values: result.historicalSutureIntensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 350 };\n    case 'historical-suture-age': return { values: result.historicalSutureAgeMyr, minimum: 0, maximum: 350, lowHue: 205, highHue: 24 };\n    case 'historical-passive-margin': return { values: result.passiveMarginIndex, minimum: 0, maximum: 1, lowHue: 215, highHue: 155 };\n    case 'historical-active-orogen': return { values: result.activeOrogenIntensity, minimum: 0, maximum: 1, lowHue: 215, highHue: 15 };\n    case 'historical-fossil-orogen': return { values: result.fossilOrogenIntensity, minimum: 0, maximum: 1, lowHue: 215, highHue: 285 };\n''' + scalar_anchor
text = replace_once(text, scalar_anchor, scalar_history, 'historical scalar modes')
land_water = "  if (mode === 'land-water') return result.submergedMask[sample] ? '#214d7a' : '#a99b72';\n"
historical_colors = land_water + '''  if (mode === 'historical-origin') return historicalIdentityColor(result.originPlateIds[sample]!, 42);\n  if (mode === 'historical-fragments') return historicalIdentityColor(result.historicalFragmentIds[sample]!, 104);\n  if (mode === 'historical-current') return historicalIdentityColor(result.currentPlateIds[sample]!, 18);\n  if (mode === 'historical-provenance') return historicalIdentityColor(result.crustProvinceId[sample]! & 0x7fff, 154);\n  if (mode === 'historical-event') return historicalEventColor(result.latestHistoricalEventKind[sample]!);\n'''
text = replace_once(text, land_water, historical_colors, 'historical categorical modes')
lab.write_text(text)

# Browser regressions now enforce that the cumulative lab is the only interactive test surface.
Path('tests/historicalAncestryBrowser.test.ts').write_text(r'''import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

test('main Planet Engine lab exposes historical material and tectonic morphology diagnostics', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  for (const label of [
    'Ancestral origin plates',
    'Persistent crust fragments',
    'Current plate ownership',
    'WG-3 crust provenance',
    'Crust formation / birth age',
    'Latest historical event type',
    'Historical rift intensity',
    'Historical suture intensity',
    'Passive margin potential',
    'Active orogen intensity',
    'Fossil orogen intensity',
  ]) assert.match(html, new RegExp(label));
  assert.equal(fs.existsSync('inheritance.html'), false);
  assert.equal(fs.existsSync('src/worldgen/diagnostics/worldgenInheritanceLabStandalone.ts'), false);
});

test('cumulative protocol carries compact historical diagnostics', () => {
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  for (const field of [
    'originPlateIds', 'historicalFragmentIds', 'currentPlateIds', 'crustProvinceId',
    'crustBirthAgeMyr', 'latestHistoricalEventKind', 'historicalRiftIntensity',
    'historicalSutureIntensity', 'passiveMarginIndex', 'activeOrogenIntensity',
    'fossilOrogenIntensity',
  ]) assert.match(protocol, new RegExp(`${field}:`));
});

test('cumulative WASM bridge derives historical diagnostics from the same frontend', () => {
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  assert.match(bridge, /generate_historical_frontend/);
  assert.match(bridge, /inherit_historical_identity/);
  assert.match(bridge, /build_historical_tectonic_morphology/);
  assert.match(bridge, /generate_lithosphere_from_history/);
  for (const method of ['origin_plate_ids', 'historical_fragment_ids', 'current_plate_ids', 'historical_suture_intensity', 'fossil_orogen_intensity']) {
    assert.match(bridge, new RegExp(`fn ${method}`));
    assert.match(worker, new RegExp(`${method}\\(\\)`));
  }
});
''')

Path('tests/rendererPerformance.test.ts').write_text(r'''import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');

test('main cumulative renderer coalesces pointer motion to animation frames', () => {
  assert.match(source, /requestAnimationFrame\(/);
  assert.match(source, /frameRequest !== null/);
});

test('main cumulative renderer reuses projection storage', () => {
  assert.match(source, /ProjectionBuffers/);
  assert.match(source, /new Float32Array\(sampleCount\)/);
  assert.match(source, /new Uint8Array\(sampleCount\)/);
});

test('main cumulative equirectangular diagnostics avoid oversized static Canvas paths', () => {
  assert.match(source, /const fastPoints = \(projection === 'map' \|\| interactive\) && count > 20_000/);
  assert.match(source, /if \(fastPoints\) context\.fillRect\(x - 0\.75, y - 0\.75, 1\.5, 1\.5\)/);
});
''')

rewrite = Path('tests/worldgenRewrite.test.ts')
text = rewrite.read_text()
text = text.replace('const PROTOCOL = 20;', 'const PROTOCOL = 21;')
text = text.replace("test('Planet Engine browser protocol v18 preserves WG-0 through WG-3.75 contracts'", "test('Planet Engine browser protocol v21 preserves WG-0 through WG-3.75 contracts'")
text = text.replace("    'src/worldgen/diagnostics/worldgenInheritanceLabStandalone.ts',\n", '')
pattern = re.compile(r"test\('WG-3\.75 inheritance diagnostic remains available as an upstream debugging surface'.*?\n}\);", re.S)
replacement = r'''test('historical ancestry diagnostics are consolidated into the cumulative lab', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(html, /historical-origin/);
  assert.match(html, /historical-suture/);
  assert.match(source, /originPlateIds/);
  assert.match(source, /historicalFragmentIds/);
  assert.match(source, /fossilOrogenIntensity/);
  assert.equal(fs.existsSync('inheritance.html'), false);
});'''
text, count = pattern.subn(replacement, text, count=1)
assert count == 1, f'worldgenRewrite inheritance test replacement={count}'
rewrite.write_text(text)

readme = Path('README.md')
text = readme.read_text()
text = text.replace('`inheritance.html` exposes those identities together with WG-3 crust provenance, fossil internal', '`index.html` exposes those identities together with WG-3 crust provenance, fossil internal')
text = re.sub(r'\n- `inheritance\.html` — focused historical material-ancestry diagnostic[^\n]*', '', text)
readme.write_text(text)

# The old separate interactive lab is intentionally retired. Programmatic inheritance APIs remain.
for obsolete in [Path('inheritance.html'), Path('src/worldgen/diagnostics/worldgenInheritanceLabStandalone.ts')]:
    if obsolete.exists():
        obsolete.unlink()
