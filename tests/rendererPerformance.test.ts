import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

const source = fs.readFileSync('src/worldgen/diagnostics/worldgenInheritanceLabStandalone.ts', 'utf8');

test('WG-3.75 globe interaction coalesces pointer motion to animation frames', () => {
  assert.match(source, /requestAnimationFrame\(/);
  assert.match(source, /frameRequest !== null/);
  assert.match(source, /renderPlanet\(canvas, current, projection\.value, visualization\.value, yaw, pitch, drag !== null\)/);
  assert.doesNotMatch(source, /pointermove[^]*?renderPlanet\(/);
});

test('WG-3.75 renderer reuses projection storage and canvas backing dimensions', () => {
  assert.match(source, /ProjectionBuffers/);
  assert.match(source, /new Float32Array\(sampleCount\)/);
  assert.match(source, /new Uint8Array\(sampleCount\)/);
  assert.match(source, /if \(canvas\.width !== width\) canvas\.width = width;/);
  assert.match(source, /if \(canvas\.height !== height\) canvas\.height = height;/);
  assert.doesNotMatch(source, /function samplePosition\(/);
  assert.doesNotMatch(source, /function rotate\(/);
});

test('WG-3.75 renderer caches and batches static display styles', () => {
  assert.match(source, /SCALAR_PALETTE_STEPS = 256/);
  assert.match(source, /function bucketize\(/);
  assert.match(source, /buildStyleCache\(/);
  assert.match(source, /styleCache\.result !== result \|\| styleCache\.mode !== mode/);
  assert.match(source, /const fastPoints = interactive && result\.metrics\.fineSampleCount > 20_000/);
});
