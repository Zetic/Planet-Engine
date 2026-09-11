import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import type { WorldgenClimateResult } from '../dist/worldgen/protocol.js';
import { buildGpuPositions, cameraForWorldDirectionAtScreen, pickNearestSample, screenToWorldDirection } from '../dist/worldgen/diagnostics/worldgenL8GlobeRenderer.js';

test('L8 GPU upload converts protocol Float64 positions to Float32 values', () => {
  const protocolPositions = new Float64Array([1, 0, -0.5, Math.PI, -Math.E, 0.25]);
  const gpuPositions = buildGpuPositions(protocolPositions);
  assert.ok(gpuPositions instanceof Float32Array);
  assert.equal(gpuPositions.length, protocolPositions.length);
  assert.equal(gpuPositions.byteLength, protocolPositions.length * Float32Array.BYTES_PER_ELEMENT);
  for (let index = 0; index < protocolPositions.length; index += 1) {
    assert.ok(Math.abs(gpuPositions[index]! - protocolPositions[index]!) < 1e-6);
  }
});

test('L8 globe camera maps the viewport center to the front-facing world direction', () => {
  const direction = screenToWorldDirection(550, 380, 1100, 760, { yaw: 0, pitch: 0, zoom: 1 });
  assert.ok(direction);
  assert.ok(Math.abs(direction[0] - 1) < 1e-12);
  assert.ok(Math.abs(direction[1]) < 1e-12);
  assert.ok(Math.abs(direction[2]) < 1e-12);
  assert.equal(screenToWorldDirection(-500, -500, 1100, 760, { yaw: 0, pitch: 0, zoom: 1 }), null);
});

test('cursor-anchored zoom preserves the inspected world direction', () => {
  const initial = { yaw: -0.65, pitch: 0.25, zoom: 1 };
  const x = 690, y = 310, width = 1100, height = 760;
  const anchor = screenToWorldDirection(x, y, width, height, initial);
  assert.ok(anchor);
  const zoomed = cameraForWorldDirectionAtScreen(anchor, x, y, width, height, 5, initial);
  const recovered = screenToWorldDirection(x, y, width, height, zoomed);
  assert.ok(recovered);
  const dot = anchor[0] * recovered[0] + anchor[1] * recovered[1] + anchor[2] * recovered[2];
  assert.ok(dot > 0.999999999, `zoom anchor drifted: dot=${dot}`);
});

test('L8 picking refines from a coarse seed set through topology neighbors', () => {
  const result = {
    positions: new Float32Array([
      1, 0, 0,
      0, 1, 0,
      0, 0, 1,
      0.1, 0.99, 0,
    ]),
    neighborOffsets: new Uint32Array([0, 1, 3, 4, 6]),
    neighbors: new Uint32Array([1, 0, 3, 3, 1, 2]),
    metrics: { fineSampleCount: 4 },
  } as unknown as WorldgenClimateResult;
  assert.equal(pickNearestSample(result, [0, 1, 0], 3), 1);
  const tilted = [0.1, 0.995, 0] as [number, number, number];
  assert.equal(pickNearestSample(result, tilted, 2), 3);
});

test('L8 lab exposes GPU globe rendering, persistent zoom, and direct tile inspection', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  const css = fs.readFileSync('styles/worldgenLab.css', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  const gpu = fs.readFileSync('src/worldgen/diagnostics/worldgenL8GlobeRenderer.ts', 'utf8');
  assert.match(html, /id="worldgen-surface"/);
  assert.match(html, /id="worldgen-zoom"/);
  assert.match(html, /id="worldgen-cell-inspector"/);
  assert.match(css, /#worldgen-surface/);
  assert.match(source, /new L8GlobeRenderer/);
  assert.match(source, /addEventListener\('wheel'/);
  assert.match(source, /drawLocalDualCells/);
  assert.match(source, /pickNearestSample/);
  assert.match(source, /screenToWorldDirection/);
  assert.match(source, /cameraForWorldDirectionAtScreen/);
  assert.match(gpu, /getContext\('webgl2'/);
  assert.match(gpu, /drawArrays\(gl\.POINTS/);
  assert.doesNotMatch(gpu, /if \(gl_PointSize/);
  assert.match(gpu, /buildGpuPositions\(result\.positions\)/);
  assert.doesNotMatch(gpu, /bufferData\(gl\.ARRAY_BUFFER, result\.positions/);
});
