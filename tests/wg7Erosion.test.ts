import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-7A cumulative browser contract is protocol v15 and single-request', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 15);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');
  const html = fs.readFileSync('index.html', 'utf8');
  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  for (const field of [
    'erosionStage', 'erosionMetrics', 'effectiveDischargeM3S', 'channelSlope', 'channelWidthM',
    'erodibilityIndex', 'streamPowerIndex', 'incisionPotentialMPerYear', 'localSedimentSupplyKgS',
    'sedimentTransportCapacityKgS', 'sedimentLoadKgS', 'sedimentDepositionKgS',
  ]) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  assert.match(bridge, /generate_fluvial_erosion_sediment/);
  assert.match(bridge, /fluvial-erosion-sediment/);
  assert.match(worker, /erosion_seasonal_hydrology_hash_hex/);
  assert.match(worker, /sediment_conservation_relative_error/);
  assert.match(worker, /client\.generateClimate|generateClimate/);
  assert.doesNotMatch(worker, /generateErosion/);
  for (const mode of ['erosion-effective-discharge', 'erosion-channel-slope', 'erosion-channel-width', 'erosion-erodibility', 'erosion-incision-potential', 'erosion-sediment-supply', 'erosion-sediment-load', 'erosion-sediment-deposition']) {
    assert.match(html, new RegExp(mode));
    assert.match(lab, new RegExp(mode));
  }
  assert.match(html, /Current physical frontier: WG-7A/);
  assert.match(lab, /WG-7A sediment closure/);
  assert.match(lab, /erosionMetrics\.seasonalHydrologyHash/);
  assert.match(lab, /erosionMetrics\.topographyHash/);
  assert.match(lab, /mode !== 'erosion-sediment-deposition'/);
});
