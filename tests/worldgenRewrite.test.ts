import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

import {
  WORLDGEN_BOUNDARY_CONVERGENT,
  WORLDGEN_BOUNDARY_DIVERGENT,
  WORLDGEN_BOUNDARY_TRANSFORM,
  WORLDGEN_CRUST_CONTINENTAL,
  WORLDGEN_CRUST_OCEANIC,
  WORLDGEN_CRUST_TRANSITIONAL,
  WORLDGEN_FRAGMENT_MICROPLATE,
  WORLDGEN_FRAGMENT_TERRANE,
  WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION,
  WORLDGEN_GEOLOGY_CONTINENTAL_RIFT,
  WORLDGEN_GEOLOGY_MAX_LEVEL,
  WORLDGEN_GEOLOGY_OCEANIC_RIDGE,
  WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION,
  WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION,
  WORLDGEN_GEOLOGY_TRANSFORM,
  WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE,
  WORLDGEN_INHERITANCE_COARSE_MAX_LEVEL,
  WORLDGEN_INHERITANCE_FINE_MAX_LEVEL,
  WORLDGEN_LITHOSPHERE_MAX_LEVEL,
  WORLDGEN_PLATE_INTERMEDIATE,
  WORLDGEN_PLATE_MAJOR,
  WORLDGEN_PLATE_MINOR,
  WORLDGEN_PROTOCOL_VERSION,
  WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN,
  WORLDGEN_STRUCTURE_NONE,
  WORLDGEN_STRUCTURE_RIFT,
  WORLDGEN_STRUCTURE_SUTURE,
  WORLDGEN_STRUCTURE_TRANSFORM,
  WORLDGEN_SUBDUCTION_NONE,
  WORLDGEN_SUBDUCTION_PLATE_A,
  WORLDGEN_SUBDUCTION_PLATE_B,
  WORLDGEN_SYNTHETIC_MAX_SAMPLES,
  WORLDGEN_TECTONICS_MAX_LEVEL,
  WORLDGEN_TECTONICS_MAX_PLATES,
  WORLDGEN_TECTONICS_MIN_PLATES,
  WORLDGEN_TOPOLOGY_MAX_LEVEL,
  validateGeologyRequest,
  validateInheritanceRequest,
  validateLithosphereRequest,
  validateSyntheticRequest,
  validateTectonicsRequest,
  validateTopologyRequest,
  worldgenGeologyCommand,
  worldgenInheritanceCommand,
  worldgenLithosphereCommand,
  worldgenSyntheticCommand,
  worldgenTectonicsCommand,
  worldgenTopologyCommand,
} from '../dist/worldgen/protocol.js';

const PROTOCOL = 14;

test('Planet Engine browser protocol v14 preserves WG-0 through WG-3.75 contracts', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, PROTOCOL);
  assert.equal(WORLDGEN_SYNTHETIC_MAX_SAMPLES, 4_194_304);
  assert.equal(WORLDGEN_TOPOLOGY_MAX_LEVEL, 7);
  assert.equal(WORLDGEN_TECTONICS_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_GEOLOGY_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_LITHOSPHERE_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_INHERITANCE_COARSE_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_INHERITANCE_FINE_MAX_LEVEL, 7);
  assert.equal(WORLDGEN_TECTONICS_MIN_PLATES, 4);
  assert.equal(WORLDGEN_TECTONICS_MAX_PLATES, 48);
  assert.deepEqual([WORLDGEN_BOUNDARY_CONVERGENT, WORLDGEN_BOUNDARY_DIVERGENT, WORLDGEN_BOUNDARY_TRANSFORM], [1, 2, 3]);
  assert.deepEqual([WORLDGEN_CRUST_OCEANIC, WORLDGEN_CRUST_TRANSITIONAL, WORLDGEN_CRUST_CONTINENTAL], [1, 2, 3]);
  assert.deepEqual([WORLDGEN_PLATE_MAJOR, WORLDGEN_PLATE_INTERMEDIATE, WORLDGEN_PLATE_MINOR], [1, 2, 3]);
  assert.deepEqual([
    WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION,
    WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION,
    WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION,
    WORLDGEN_GEOLOGY_OCEANIC_RIDGE,
    WORLDGEN_GEOLOGY_CONTINENTAL_RIFT,
    WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE,
    WORLDGEN_GEOLOGY_TRANSFORM,
  ], [1, 2, 3, 4, 5, 6, 7]);
  assert.deepEqual([WORLDGEN_SUBDUCTION_NONE, WORLDGEN_SUBDUCTION_PLATE_A, WORLDGEN_SUBDUCTION_PLATE_B], [0, 1, 2]);
  assert.deepEqual([
    WORLDGEN_STRUCTURE_NONE,
    WORLDGEN_STRUCTURE_SUTURE,
    WORLDGEN_STRUCTURE_RIFT,
    WORLDGEN_STRUCTURE_TRANSFORM,
    WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN,
  ], [0, 1, 2, 3, 4]);
  assert.deepEqual([WORLDGEN_FRAGMENT_TERRANE, WORLDGEN_FRAGMENT_MICROPLATE], [1, 2]);

  assert.deepEqual(worldgenSyntheticCommand(7, { seed: 'wg0', width: 512, height: 256 }), { protocolVersion: PROTOCOL, requestId: 7, type: 'generate-synthetic', payload: { seed: 'wg0', width: 512, height: 256 } });
  assert.deepEqual(worldgenTopologyCommand(8, { level: 4 }), { protocolVersion: PROTOCOL, requestId: 8, type: 'generate-topology', payload: { level: 4 } });
  assert.deepEqual(worldgenTectonicsCommand(9, { seed: 'wg2', level: 5, plateCount: 16 }), { protocolVersion: PROTOCOL, requestId: 9, type: 'generate-tectonics', payload: { seed: 'wg2', level: 5, plateCount: 16 } });
  assert.deepEqual(worldgenGeologyCommand(10, { seed: 'wg3', level: 5, plateCount: 16 }), { protocolVersion: PROTOCOL, requestId: 10, type: 'generate-geology', payload: { seed: 'wg3', level: 5, plateCount: 16 } });
  assert.deepEqual(worldgenLithosphereCommand(11, { seed: 'wg3-5', level: 5, plateCount: 16 }), { protocolVersion: PROTOCOL, requestId: 11, type: 'generate-lithosphere', payload: { seed: 'wg3-5', level: 5, plateCount: 16 } });
  assert.deepEqual(worldgenInheritanceCommand(12, { seed: 'wg3-75', coarseLevel: 4, fineLevel: 6, plateCount: 16 }), { protocolVersion: PROTOCOL, requestId: 12, type: 'generate-inheritance', payload: { seed: 'wg3-75', coarseLevel: 4, fineLevel: 6, plateCount: 16 } });

  assert.throws(() => validateSyntheticRequest({ seed: '', width: 1, height: 1 }), /seed/i);
  assert.throws(() => validateSyntheticRequest({ seed: 'x', width: 4096, height: 4096 }), /limited/i);
  assert.doesNotThrow(() => validateTopologyRequest({ level: 7 }));
  assert.throws(() => validateTopologyRequest({ level: 8 }), /0 through 7/i);
  assert.doesNotThrow(() => validateTectonicsRequest({ seed: 'x', level: 6, plateCount: 24 }));
  assert.throws(() => validateTectonicsRequest({ seed: 'x', level: 5, plateCount: 3 }), /4 through 48/i);
  assert.doesNotThrow(() => validateGeologyRequest({ seed: 'x', level: 6, plateCount: 24 }));
  assert.doesNotThrow(() => validateLithosphereRequest({ seed: 'x', level: 6, plateCount: 24 }));
  assert.doesNotThrow(() => validateInheritanceRequest({ seed: 'x', coarseLevel: 4, fineLevel: 7, plateCount: 24 }));
  assert.throws(() => validateInheritanceRequest({ seed: 'x', coarseLevel: 5, fineLevel: 4, plateCount: 16 }), /fine level/i);
});

test('Planet Engine source stays independent from legacy gameplay world objects through WG-6D', () => {
  const files = [
    'src/worldgen/protocol.ts',
    'src/worldgen/worldgenClient.ts',
    'src/worldgen/worldgenWorker.ts',
    'src/worldgen/diagnostics/worldgenLabStandalone.ts',
    'src/worldgen/diagnostics/worldgenInheritanceLabStandalone.ts',
    'src/worldgen/diagnostics/worldgenTopographyLabStandalone.ts',
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
  ];
  const forbidden = [/\.\.\/world\//, /Region\b/, /MapSelection\b/, /GeographyPatch\b/, /resourceNode/i];
  for (const path of files) {
    const source = fs.readFileSync(path, 'utf8');
    for (const pattern of forbidden) assert.doesNotMatch(source, pattern, `${path} must not depend on ${pattern}`);
  }
  for (const path of [
    'rust/interlink-worldgen/src/topology.rs',
    'rust/interlink-worldgen/src/coordinates.rs',
    'rust/interlink-worldgen/src/tectonics.rs',
    'rust/interlink-worldgen/src/geology.rs',
    'rust/interlink-worldgen/src/lithosphere.rs',
    'rust/interlink-worldgen/src/refinement.rs',
    'rust/interlink-worldgen/src/boundary_refinement.rs',
    'rust/interlink-worldgen/src/topography.rs',
    'rust/interlink-worldgen/src/climate.rs',
    'rust/interlink-worldgen/src/drainage.rs',
    'rust/interlink-worldgen/src/runoff.rs',
    'rust/interlink-worldgen/tests/climate_ensemble.rs',
    'rust/interlink-worldgen-wasm/Cargo.toml',
    'rust/interlink-worldgen-cli/Cargo.toml',
    'docs/worldgen-rewrite/GEOLOGY.md',
    'docs/worldgen-rewrite/LITHOSPHERE.md',
    'docs/worldgen-rewrite/MULTIRESOLUTION.md',
    'docs/worldgen-rewrite/PLANET_PARAMETERS.md',
    'docs/worldgen-rewrite/TOPOGRAPHY.md',
    'docs/worldgen-rewrite/WG5_CLIMATE.md',
  ]) assert.ok(fs.existsSync(path), `${path} must exist`);
});

test('WG-3.75 inheritance diagnostic remains available as an upstream debugging surface', () => {
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenInheritanceLabStandalone.ts', 'utf8');
  assert.match(source, /generateInheritance/);
  assert.match(source, /boundary-provenance/);
  assert.match(source, /inherited-mask/);
  assert.match(source, /nearestCoarseSource/);
  assert.doesNotMatch(source, /solidElevationM|waterDepthM/);
});
