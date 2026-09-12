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


test('physical-world hypsometric palette doubles L8 relief and bathymetry detail while compressing deep-ocean contrast', () => {
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  const start = source.indexOf('function hypsometricColor');
  const end = source.indexOf('function bucketize', start);
  assert.notEqual(start, -1);
  assert.notEqual(end, -1);
  const palette = source.slice(start, end);

  const oceanThresholds = ['25', '50', '75', '100', '150', '250', '350', '500', '700', '900', '1_200', '1_500', '1_800', '2_200', '2_600', '3_000', '3_500', '4_000', '4_500', '5_250', '6_000', '6_750', '7_500'];
  for (const threshold of oceanThresholds) assert.match(palette, new RegExp(`depth <= ${threshold}`));

  const landThresholds = ['25', '50', '75', '100', '150', '200', '300', '400', '550', '700', '850', '1_000', '1_250', '1_500', '1_750', '2_000', '2_375', '2_750', '3_125', '3_500', '3_875', '4_250', '4_625', '5_000', '5_500', '6_000', '6_750'];
  for (const threshold of landThresholds) {
    const matches = palette.match(new RegExp(`elevation < ${threshold}`, 'g')) ?? [];
    assert.equal(matches.length, 2, `expected initial and evolved land bands at ${threshold} m`);
  }

  for (const color of ['#b7e5e6', '#a4dce1', '#87c9d8', '#69b7cf', '#22536e', '#20516c']) assert.match(palette, new RegExp(color));
  for (const retiredHighContrastDeepOceanColor of ['#092847', '#0d335a', '#123f6c']) assert.doesNotMatch(palette, new RegExp(retiredHighContrastDeepOceanColor));
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
