import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-6D cumulative browser contract is protocol v17 and single-request', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 17);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  const html = fs.readFileSync('index.html', 'utf8');
  for (const field of [
    'seasonalStage', 'seasonalMetrics', 'seasonalPhaseLocalRunoffM3S',
    'seasonalPhaseSnowmeltRunoffM3S', 'seasonalPhaseSnowStorageMm',
    'seasonalPhasePotentialDischargeM3S', 'seasonalPhaseRealizedDischargeM3S',
    'seasonalFlowPresenceFraction', 'seasonalFlowRegime',
    'seasonalPhaseLakeSurfaceElevationM', 'seasonalPhaseLakeAreaM2', 'seasonalPhaseLakeVolumeM3',
  ]) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  for (const mode of ['seasonal-realized-discharge', 'seasonal-flow-presence', 'seasonal-flow-regime', 'seasonal-snow-storage']) {
    assert.match(html, new RegExp(mode));
    assert.match(lab, new RegExp(mode));
  }
  assert.match(lab, /loaded\.seasonalMetrics\.climateHash !== loaded\.metrics\.climateHash/);
  assert.match(lab, /loaded\.seasonalMetrics\.drainageHash !== loaded\.drainageMetrics\.drainageHash/);
  assert.match(lab, /loaded\.seasonalMetrics\.runoffHash !== loaded\.runoffMetrics\.runoffHash/);
  assert.match(lab, /loaded\.seasonalMetrics\.lakeHash !== loaded\.lakeMetrics\.lakeHash/);
  assert.doesNotMatch(lab, /client\.generateDrainage\(/);
  assert.match(lab, /client\.generateClimate\(request, handleGenerationProgress\)/);
});
