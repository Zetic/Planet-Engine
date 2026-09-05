import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-7C cumulative browser contract is protocol v17 and memory-conscious', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 17);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');
  const html = fs.readFileSync('index.html', 'utf8');
  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  for (const field of ['reconciliationStage', 'reconciliationMetrics', 'lakeKindChangedMask', 'lakeDepthDeltaM', 'annualRealizedDischargeDeltaM3S', 'flowRegimeChangedMask', 'flowPresenceDelta']) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  assert.match(bridge, /generate_post_erosion_hydrology/);
  assert.match(bridge, /post-erosion-hydrology/);
  assert.match(bridge, /reconciliation: PostErosionHydrologyState/);
  assert.doesNotMatch(bridge, /\n    seasonal: SeasonalHydrologyState,/);
  assert.doesNotMatch(bridge, /\n    runoff: RunoffState,/);
  assert.doesNotMatch(bridge, /\n    lakes: LakeState,/);
  assert.match(worker, /progress\('packaging', 16, 17/);
  for (const mode of ['reconciliation-lake-depth-delta', 'reconciliation-lake-change', 'reconciliation-realized-discharge-delta', 'reconciliation-flow-presence-delta', 'reconciliation-flow-regime-change']) {
    assert.match(html, new RegExp(mode));
    assert.match(lab, new RegExp(mode));
  }
  assert.match(html, /Current physical frontier: WG-7C/);
  assert.match(lab, /WG-7C reconciliation hash/);
  assert.match(lab, /erosionMetrics\.drainageHash !== loaded\.reconciliationMetrics\.preErosionDrainageHash/);
  assert.match(lab, /erosionMetrics\.lakeHash !== loaded\.reconciliationMetrics\.preErosionLakeHash/);
  assert.doesNotMatch(lab, /erosionMetrics\.drainageHash !== loaded\.drainageMetrics\.drainageHash/);
  assert.match(lab, /reconciliationMetrics\.preErosionSeasonalHash/);
  assert.match(lab, /reconciliationMetrics\.reconciledSeasonalHash/);
});
