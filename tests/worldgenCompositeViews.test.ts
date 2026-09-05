import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('composite physical-world views reuse the WG-7C cumulative result without protocol changes', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 17);
  const html = fs.readFileSync('index.html', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(html, /id="worldgen-preset"/);
  assert.match(html, /value="physical-world"/);
  for (const overlay of ['evolved-topography', 'final-rivers', 'final-lakes', 'basin-divides', 'cryosphere']) {
    assert.match(html, new RegExp(`value="${overlay}"`));
    assert.match(source, new RegExp(overlay));
  }
  assert.match(source, /function evolvedHypsometricColor/);
  assert.match(source, /function drawFinalRiverOverlay/);
  assert.match(source, /function drawFinalLakeOverlay/);
  assert.match(source, /function drawCryosphereOverlay/);
  assert.match(source, /VIEW_PRESETS/);
  assert.match(source, /result\.realizedDischargeM3S/);
  assert.match(source, /result\.terrainDeltaM/);
});
