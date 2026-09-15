import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

test('cumulative L8 protocol excludes solver-only transport fields', () => {
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const start = protocol.indexOf('export interface WorldgenClimateResult');
  const end = protocol.indexOf('export interface WorldgenDrainageMetrics', start);
  const climate = protocol.slice(start, end);
  for (const field of ['faces', 'outletSample', 'outletKind', 'drainageOrder', 'runoffStage', 'localRunoffM3S', 'lakeSpillSamples', 'streamPowerIndex', 'sedimentTransportCapacityKgS']) {
    assert.doesNotMatch(climate, new RegExp(`\\b${field}:`));
  }
});

test('final WASM object explicitly releases solver lifetimes', () => {
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');
  const topology = fs.readFileSync('rust/interlink-worldgen/src/topology.rs', 'utf8');
  const causal = fs.readFileSync('rust/interlink-worldgen/src/causal_pipeline.rs', 'utf8');
  for (const term of ['release_topography_scratch', 'release_post_erosion_scratch', 'release_post_infill_scratch', 'release_post_generation_scratch']) assert.match(bridge, new RegExp(term));
  assert.match(topology, /release_post_generation_scratch/);
  assert.match(causal, /structural_fabric_strength = Vec::new/);
});
