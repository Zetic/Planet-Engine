import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-6C cumulative browser contract is protocol v17 and single-request', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 17);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  const html = fs.readFileSync('index.html', 'utf8');
  for (const field of ['lakeMetrics', 'lakeId', 'lakeKind', 'lakeFraction', 'lakeDepthM', 'realizedDischargeM3S']) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  for (const mode of ['realized-discharge', 'lake-depth', 'lake-state', 'lake-fraction']) assert.match(html, new RegExp(mode));
  assert.match(lab, /loaded\.lakeMetrics\.climateHash !== loaded\.metrics\.climateHash/);
  assert.match(lab, /loaded\.lakeMetrics\.drainageHash !== loaded\.drainageMetrics\.drainageHash/);
  assert.match(lab, /loaded\.lakeMetrics\.runoffHash !== loaded\.runoffMetrics\.runoffHash/);
  assert.doesNotMatch(lab, /client\.generateDrainage\(/);
  assert.match(lab, /client\.generateClimate\(request, handleGenerationProgress\)/);
  assert.ok(!fs.existsSync('drainage.html'));
  assert.ok(!fs.existsSync('worldgen-lab.html'));
});
