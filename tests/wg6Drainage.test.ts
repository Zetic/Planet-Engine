import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {
  WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL,
  WORLDGEN_DRAINAGE_FINE_MAX_LEVEL,
  WORLDGEN_INVALID_SAMPLE_ID,
  WORLDGEN_PROTOCOL_VERSION,
  validateDrainageRequest,
  worldgenDrainageCommand,
} from '../dist/worldgen/protocol.js';

test('WG-6A browser protocol is versioned and bounded', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 15);
  assert.equal(WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_DRAINAGE_FINE_MAX_LEVEL, 7);
  assert.equal(WORLDGEN_INVALID_SAMPLE_ID, 0xffff_ffff);

  assert.doesNotThrow(() => validateDrainageRequest({
    seed: 'wg6a-browser',
    coarseLevel: 4,
    fineLevel: 6,
    plateCount: 16,
  }));

  assert.throws(
    () => validateDrainageRequest({ seed: '', coarseLevel: 4, fineLevel: 6, plateCount: 16 }),
    /seed must not be empty/i,
  );
  assert.throws(
    () => validateDrainageRequest({ seed: 'wg6a', coarseLevel: 5, fineLevel: 4, plateCount: 16 }),
    /fine level/i,
  );
  assert.throws(
    () => validateDrainageRequest({ seed: 'wg6a', coarseLevel: 4, fineLevel: 6, plateCount: 3 }),
    /plate count/i,
  );
});

test('WG-6A command uses the dedicated drainage transport contract', () => {
  const payload = { seed: 'wg6a-command', coarseLevel: 4, fineLevel: 6, plateCount: 16 };
  const command = worldgenDrainageCommand(91, payload);
  assert.equal(command.protocolVersion, 15);
  assert.equal(command.requestId, 91);
  assert.equal(command.type, 'generate-drainage');
  assert.deepEqual(command.payload, payload);
});


test('primary Planet Engine Lab blends WG-6A into the main physical diagnostic surface', () => {
  const page = fs.readFileSync('index.html', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(page, /THROUGH WG-7A/);
  for (const mode of ['contributing-area', 'basins', 'flow-direction', 'depression-depth', 'depressions', 'escape-elevation']) {
    assert.match(page, new RegExp(`value=["']${mode}["']`));
  }
  assert.doesNotMatch(source, /client\.generateDrainage\(/);
  assert.match(source, /client\.generateClimate\(request, handleGenerationProgress\)/);
  assert.match(source, /drainageMetrics/);
  assert.match(source, /loaded\.runoffMetrics\.climateHash !== loaded\.metrics\.climateHash/);
  assert.match(source, /loaded\.runoffMetrics\.drainageHash !== loaded\.drainageMetrics\.drainageHash/);
  assert.match(source, /drainageMetrics/);
  assert.match(source, /renderDrainageDiagnostic/);
});
