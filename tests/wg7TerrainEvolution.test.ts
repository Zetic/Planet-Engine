import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-7B cumulative browser contract is protocol v16 and single-request', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 16);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');
  const html = fs.readFileSync('index.html', 'utf8');
  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  for (const field of [
    'evolutionStage', 'evolutionMetrics', 'evolvedSolidElevationM', 'terrainDeltaM',
    'appliedErosionM', 'appliedDepositionM', 'receiverChangedMask',
    'postErosionContributingAreaM2', 'postErosionPotentialDischargeM3S',
  ]) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  assert.match(bridge, /generate_bounded_terrain_evolution/);
  assert.match(bridge, /bounded-terrain-evolution/);
  assert.match(worker, /terrain_evolution_hash_hex/);
  assert.match(worker, /post_erosion_runoff_conservation_relative_error/);
  assert.match(worker, /client\.generateClimate|generateClimate/);
  assert.doesNotMatch(worker, /generateEvolution/);
  for (const mode of [
    'evolution-solid-elevation', 'evolution-terrain-delta', 'evolution-applied-erosion',
    'evolution-applied-deposition', 'evolution-receiver-change', 'evolution-contributing-area',
    'evolution-potential-discharge',
  ]) {
    assert.match(html, new RegExp(mode));
    assert.match(lab, new RegExp(mode));
  }
  assert.match(html, /Current physical frontier: WG-7B/);
  assert.match(lab, /WG-7B sediment closure/);
  assert.match(lab, /evolutionMetrics\.fluvialErosionHash/);
  assert.match(lab, /evolutionMetrics\.topographyHash/);
  assert.match(lab, /evolutionMetrics\.drainageHash/);
  assert.match(lab, /evolutionMetrics\.runoffHash/);
  assert.match(lab, /evolutionMetrics\.lakeHash/);
});
