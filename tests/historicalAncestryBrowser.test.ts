import assert from 'node:assert/strict';
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
