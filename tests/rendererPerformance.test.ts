import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');

test('main cumulative renderer coalesces pointer motion to animation frames', () => {
  assert.match(source, /requestAnimationFrame\(/);
  assert.match(source, /if \(frameRequest\) return/);
  assert.match(source, /frameRequest = requestAnimationFrame/);
  assert.match(source, /scheduleRedraw\(true\)/);
});

test('main cumulative renderer reuses projection storage and canvas backing dimensions', () => {
  assert.match(source, /type ProjectionBuffers/);
  assert.match(source, /new Float32Array\(loaded\.metrics\.fineSampleCount\)/);
  assert.match(source, /new Uint8Array\(loaded\.metrics\.fineSampleCount\)/);
  assert.match(source, /if \(canvas\.width !== width\) canvas\.width = width/);
  assert.match(source, /if \(canvas\.height !== height\) canvas\.height = height/);
});

test('main cumulative equirectangular diagnostics avoid oversized static Canvas paths', () => {
  assert.match(source, /const fastPoints = \(projection === 'map' \|\| interactive\) && count > 20_000/);
  assert.match(source, /if \(fastPoints\) context\.fillRect\(x - 0\.75, y - 0\.75, 1\.5, 1\.5\)/);
});
