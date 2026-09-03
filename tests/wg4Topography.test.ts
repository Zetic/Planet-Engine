import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {
  WORLDGEN_PROTOCOL_VERSION,
  WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL,
  WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL,
  validateTopographyRequest,
  worldgenTopographyCommand,
} from '../dist/worldgen/protocol.js';

test('WG-4 browser protocol v7 exposes bounded coarse-to-fine topography generation', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 7);
  assert.equal(WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL, 7);
  assert.doesNotThrow(() => validateTopographyRequest({ seed: 'wg4', coarseLevel: 4, fineLevel: 7, plateCount: 18 }));
  assert.throws(() => validateTopographyRequest({ seed: '', coarseLevel: 4, fineLevel: 6, plateCount: 18 }), /seed/i);
  assert.throws(() => validateTopographyRequest({ seed: 'wg4', coarseLevel: 5, fineLevel: 4, plateCount: 18 }), /fine level/i);
  assert.deepEqual(worldgenTopographyCommand(77, { seed: 'wg4', coarseLevel: 4, fineLevel: 6, plateCount: 18 }), { protocolVersion: 7, requestId: 77, type: 'generate-topography', payload: { seed: 'wg4', coarseLevel: 4, fineLevel: 6, plateCount: 18 } });
});

test('WG-4 lab exposes causal terrain components and keeps downstream stages out of scope', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  assert.match(html, /PLANET ENGINE · WG-4/);
  for (const term of ['Elevation above sea level', 'Bathymetry', 'Isostatic support', 'Oceanic thermal subsidence', 'Orogenic', 'Ridge relief', 'Rift \/ basin', 'Trench relief', 'Volcanic arc relief', 'Mantle dynamic support']) assert.match(html, new RegExp(term, 'i'));
  assert.match(html, /pre-erosional tectonic topography/i);
  assert.match(html, /No climate, drainage, river incision, sediment transport/);
  assert.doesNotMatch(html, /resource node|Region Inspector|NAV/);
});

test('WG-4 renderer preserves the high-resolution interaction performance contract', () => {
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenTopographyLabStandalone.ts', 'utf8');
  assert.match(source, /requestAnimationFrame/);
  assert.match(source, /Float32Array/);
  assert.match(source, /canvas\.width !== width/);
  assert.match(source, /interactive && count > 20_000/);
  assert.doesNotMatch(source, /pointermove[\s\S]{0,500}redraw\(\)/);
});
