import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-6B cumulative browser contract is protocol v16 and single-request', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 16);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  for (const field of ['actualEvapotranspirationMm', 'localRunoffMm', 'runoffFraction', 'potentialDischargeM3S', 'runoffMetrics', 'drainageMetrics']) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  assert.match(lab, /potential-discharge/);
  assert.match(lab, /annual-runoff/);
  assert.match(lab, /runoff-fraction/);
  assert.match(lab, /actual-et/);
  assert.doesNotMatch(lab, /client\.generateDrainage\(/);
  assert.match(lab, /client\.generateClimate\(request, handleGenerationProgress\)/);
});
