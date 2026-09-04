import assert from 'node:assert/strict';
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
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 11);
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
  assert.equal(command.protocolVersion, 11);
  assert.equal(command.requestId, 91);
  assert.equal(command.type, 'generate-drainage');
  assert.deepEqual(command.payload, payload);
});
