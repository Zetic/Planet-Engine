import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-4.5 lithology is part of cumulative protocol v23', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 23);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');

  for (const field of [
    'bedrockClass',
    'rockStrengthIndex',
    'lithologyErodibilityIndex',
    'permeabilityIndex',
    'weatheringSusceptibility',
    'finesFraction',
    'carbonateFraction',
  ]) {
    assert.match(protocol, new RegExp(`\\b${field}\\b`));
  }
  assert.match(protocol, /lithologyStage: WorldgenStageMetadata/);
  assert.match(protocol, /lithologyHash: string/);
  assert.match(worker, /output\.bedrock_class\(\)/);
  assert.match(worker, /output\.lithology_hash_hex\(\)/);
  assert.match(bridge, /generate_lithology_substrate/);
  assert.match(bridge, /release_topography_scratch/);
});

test('Planet Engine Lab exposes WG-4.5 material diagnostics', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  for (const mode of [
    'bedrock-class',
    'rock-strength',
    'lithology-erodibility',
    'permeability',
    'weathering-susceptibility',
    'fines-fraction',
    'carbonate-fraction',
  ]) {
    assert.match(html, new RegExp(`value="${mode}"`));
  }
});
