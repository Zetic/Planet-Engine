import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('composite physical-world views retain WG-7C diagnostics under protocol v18', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 18);
  const html = fs.readFileSync('index.html', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(html, /id="worldgen-preset"/);
  assert.match(html, /value="physical-world"/);
  for (const overlay of ['evolved-topography', 'final-rivers', 'final-lakes', 'basin-divides', 'cryosphere']) {
    assert.match(html, new RegExp(`value="${overlay}"`));
    assert.match(source, new RegExp(overlay));
  }
  assert.match(source, /function evolvedHypsometricColor/);
  assert.match(source, /function drawFinalRiverOverlay/);
  assert.match(source, /function drawFinalLakeOverlay/);
  assert.match(source, /function drawCryosphereOverlay/);
  assert.match(source, /VIEW_PRESETS/);
  assert.match(source, /result\.realizedDischargeM3S/);
  assert.match(source, /result\.terrainDeltaM/);
});


test('WG-7D final physical world uses post-infill terrain and final hydrology ancestry', () => {
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(protocol, /WORLDGEN_PROTOCOL_VERSION = 18/);
  assert.match(protocol, /infillMetrics: WorldgenLakeSedimentInfillMetrics/);
  assert.match(protocol, /postInfillSolidElevationM: Float32Array/);
  assert.match(worker, /infill_post_infill_drainage_hash_hex/);
  assert.match(worker, /postInfillSolidElevationM/);
  assert.match(lab, /result\.postInfillSolidElevationM\[sample\]/);
  assert.match(lab, /WG-7D final drainage identity mismatch/);
  assert.match(lab, /lake-sediment-infill/);
});


test('WG-7D canonical browser drainage getters all source the post-infill state', () => {
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');
  const start = bridge.indexOf('pub fn drainage_stage_id');
  const end = bridge.indexOf('pub fn runoff_stage_id', start);
  assert.notEqual(start, -1);
  assert.notEqual(end, -1);
  const drainageSection = bridge.slice(start, end);
  assert.match(drainageSection, /self\.infill\.post_infill_drainage/);
  assert.doesNotMatch(drainageSection, /self\.evolution\s*\.\s*post_erosion_drainage/);
});
