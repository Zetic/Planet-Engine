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

test('WG-4 browser contract remains available under protocol v16', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 16);
  assert.equal(WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL, 7);
  assert.doesNotThrow(() => validateTopographyRequest({ seed: 'wg4', coarseLevel: 4, fineLevel: 7, plateCount: 18 }));
  assert.throws(() => validateTopographyRequest({ seed: '', coarseLevel: 4, fineLevel: 6, plateCount: 18 }), /seed/i);
  assert.throws(() => validateTopographyRequest({ seed: 'wg4', coarseLevel: 5, fineLevel: 4, plateCount: 18 }), /fine level/i);
  assert.deepEqual(worldgenTopographyCommand(77, { seed: 'wg4', coarseLevel: 4, fineLevel: 6, plateCount: 18 }), { protocolVersion: 16, requestId: 77, type: 'generate-topography', payload: { seed: 'wg4', coarseLevel: 4, fineLevel: 6, plateCount: 18 } });
});

test('Planet Engine Lab keeps every WG-3.75 and WG-4 view cumulatively through WG-7A', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  assert.match(html, /PLANET ENGINE · THROUGH WG-7A/);
  assert.match(html, />Generate Planet</);
  for (const term of [
    'Elevation above sea level', 'Bathymetry', 'Isostatic support', 'Oceanic thermal subsidence', 'Orogenic / collision uplift', 'Ridge relief', 'Rift / basin subsidence', 'Trench relief', 'Volcanic arc relief',
    'Inherited coarse samples', 'Nearest coarse provenance', 'Boundary provenance', 'Macro plate ownership', 'Refined kinematic domains', 'Fine tectonic boundaries', 'Fine geological regimes',
    'Crust type', 'Crust age', 'Crust thickness', 'Orogenic history', 'Ridge history', 'Trench history', 'Lithospheric strength', 'Lithospheric weakness', 'Structural zone type', 'Fragmentation propensity', 'Fine topology mesh',
  ]) assert.match(html, new RegExp(term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i'));
  assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff\/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, and WG-7A fluvial erosion\/sediment diagnostics/i);
  assert.doesNotMatch(html, /resource node|Region Inspector|NAV/);
});

test('WG-4 Lab controller uses one generated topography result for upstream and terrain diagnostics', () => {
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenTopographyLabStandalone.ts', 'utf8');
  assert.match(source, /generateTopography/);
  assert.doesNotMatch(source, /generateInheritance/);
  for (const field of ['nearestCoarseSource', 'inheritedSampleMask', 'kinematicDomainIds', 'boundaryKinds', 'boundaryCoarseSourceIndices', 'crustAgeMyr', 'crustThicknessKm', 'orogenicHistory', 'ridgeHistory', 'trenchHistory', 'strengthIndex', 'weaknessIndex', 'mantleDynamicSupportIndex', 'structuralZoneKind', 'fragmentationPropensity']) assert.match(source, new RegExp(field));
  assert.match(source, /tectonicBoundaryColor/);
  assert.match(source, /geologicalBoundaryColor/);
  assert.match(source, /provenanceColor/);
});

test('WG-4 browser transport preserves inherited diagnostics and physical profile values', () => {
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/topography_bridge.rs', 'utf8');
  for (const term of ['nearest_coarse_source', 'inherited_sample_mask', 'boundary_kinds', 'boundary_coarse_source_indices', 'kinematic_domain_ids']) assert.match(bridge, new RegExp(term));
  for (const field of ['nearestCoarseSource', 'inheritedSampleMask', 'boundaryKinds', 'boundaryCoarseSourceIndices', 'kinematicDomainIds']) assert.match(protocol, new RegExp(field));
  assert.match(worker, /internalHeatFluxWPerM2:\s*output\.internal_heat_flux_w_per_m2\(\)/);
  assert.match(worker, /mantleThermalExpansivityPerK:\s*output\.mantle_thermal_expansivity_per_k\(\)/);
  assert.doesNotMatch(worker, /internalHeatFluxWPerM2:\s*0/);
  assert.doesNotMatch(worker, /mantleThermalExpansivityPerK:\s*0/);
});

test('WG-4 renderer preserves the high-resolution interaction performance contract', () => {
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenTopographyLabStandalone.ts', 'utf8');
  assert.match(source, /requestAnimationFrame/);
  assert.match(source, /Float32Array/);
  assert.match(source, /canvas\.width !== width/);
  assert.match(source, /interactive && count > 20_000/);
  assert.doesNotMatch(source, /pointermove[\s\S]{0,500}redraw\(\)/);
});
