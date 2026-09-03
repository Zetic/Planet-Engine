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

test('Planet Engine browser protocol v6 preserves prior stages and adds bounded WG-3.75 inheritance transport', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 6);
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

  const synthetic = worldgenSyntheticCommand(7, { seed: 'wg0', width: 512, height: 256 });
  assert.deepEqual(synthetic, { protocolVersion: 6, requestId: 7, type: 'generate-synthetic', payload: { seed: 'wg0', width: 512, height: 256 } });
  const topology = worldgenTopologyCommand(8, { level: 4 });
  assert.deepEqual(topology, { protocolVersion: 6, requestId: 8, type: 'generate-topology', payload: { level: 4 } });
  const tectonics = worldgenTectonicsCommand(9, { seed: 'wg2', level: 5, plateCount: 16 });
  assert.deepEqual(tectonics, { protocolVersion: 6, requestId: 9, type: 'generate-tectonics', payload: { seed: 'wg2', level: 5, plateCount: 16 } });
  const geology = worldgenGeologyCommand(10, { seed: 'wg3', level: 5, plateCount: 16 });
  assert.deepEqual(geology, { protocolVersion: 6, requestId: 10, type: 'generate-geology', payload: { seed: 'wg3', level: 5, plateCount: 16 } });
  const lithosphere = worldgenLithosphereCommand(11, { seed: 'wg3-5', level: 5, plateCount: 16 });
  assert.deepEqual(lithosphere, { protocolVersion: 6, requestId: 11, type: 'generate-lithosphere', payload: { seed: 'wg3-5', level: 5, plateCount: 16 } });
  const inheritance = worldgenInheritanceCommand(12, { seed: 'wg3-75', coarseLevel: 4, fineLevel: 6, plateCount: 16 });
  assert.deepEqual(inheritance, { protocolVersion: 6, requestId: 12, type: 'generate-inheritance', payload: { seed: 'wg3-75', coarseLevel: 4, fineLevel: 6, plateCount: 16 } });

  assert.throws(() => validateSyntheticRequest({ seed: '', width: 1, height: 1 }), /seed/i);
  assert.throws(() => validateSyntheticRequest({ seed: 'x', width: 4096, height: 4096 }), /limited/i);
  assert.doesNotThrow(() => validateTopologyRequest({ level: 7 }));
  assert.throws(() => validateTopologyRequest({ level: 8 }), /0 through 7/i);
  assert.throws(() => validateTopologyRequest({ level: 1.5 }), /integer/i);
  assert.doesNotThrow(() => validateTectonicsRequest({ seed: 'x', level: 6, plateCount: 24 }));
  assert.throws(() => validateTectonicsRequest({ seed: '', level: 5, plateCount: 16 }), /seed/i);
  assert.throws(() => validateTectonicsRequest({ seed: 'x', level: 7, plateCount: 16 }), /0 through 6/i);
  assert.throws(() => validateTectonicsRequest({ seed: 'x', level: 5, plateCount: 3 }), /4 through 48/i);
  assert.throws(() => validateTectonicsRequest({ seed: 'x', level: 0, plateCount: 13 }), /sample count/i);
  assert.doesNotThrow(() => validateGeologyRequest({ seed: 'x', level: 6, plateCount: 24 }));
  assert.throws(() => validateGeologyRequest({ seed: '', level: 5, plateCount: 16 }), /seed/i);
  assert.throws(() => validateGeologyRequest({ seed: 'x', level: 7, plateCount: 16 }), /0 through 6/i);
  assert.throws(() => validateGeologyRequest({ seed: 'x', level: 5, plateCount: 3 }), /4 through 48/i);
  assert.throws(() => validateGeologyRequest({ seed: 'x', level: 0, plateCount: 13 }), /sample count/i);
  assert.doesNotThrow(() => validateLithosphereRequest({ seed: 'x', level: 6, plateCount: 24 }));
  assert.throws(() => validateLithosphereRequest({ seed: '', level: 5, plateCount: 16 }), /seed/i);
  assert.throws(() => validateLithosphereRequest({ seed: 'x', level: 7, plateCount: 16 }), /0 through 6/i);
  assert.throws(() => validateLithosphereRequest({ seed: 'x', level: 5, plateCount: 3 }), /4 through 48/i);
  assert.throws(() => validateLithosphereRequest({ seed: 'x', level: 0, plateCount: 13 }), /sample count/i);
  assert.doesNotThrow(() => validateInheritanceRequest({ seed: 'x', coarseLevel: 4, fineLevel: 7, plateCount: 24 }));
  assert.throws(() => validateInheritanceRequest({ seed: '', coarseLevel: 4, fineLevel: 6, plateCount: 16 }), /seed/i);
  assert.throws(() => validateInheritanceRequest({ seed: 'x', coarseLevel: 7, fineLevel: 7, plateCount: 16 }), /0 through 6/i);
  assert.throws(() => validateInheritanceRequest({ seed: 'x', coarseLevel: 5, fineLevel: 4, plateCount: 16 }), /fine level/i);
  assert.throws(() => validateInheritanceRequest({ seed: 'x', coarseLevel: 0, fineLevel: 1, plateCount: 13 }), /sample count/i);
});

test('new Planet Engine source stays independent from legacy gameplay world objects', () => {
  const files = [
    'src/worldgen/protocol.ts',
    'src/worldgen/worldgenClient.ts',
    'src/worldgen/worldgenWorker.ts',
    'src/worldgen/diagnostics/worldgenLabStandalone.ts',
    'src/worldgen/diagnostics/worldgenInheritanceLabStandalone.ts',
  ];
  const forbidden = [/\.\.\/world\//, /Region\b/, /MapSelection\b/, /GeographyPatch\b/, /resourceNode/i];
  for (const path of files) {
    const source = fs.readFileSync(path, 'utf8');
    for (const pattern of forbidden) assert.doesNotMatch(source, pattern, `${path} must not depend on ${pattern}`);
  }
  assert.ok(fs.existsSync('rust/interlink-worldgen/src/topology.rs'));
  assert.ok(fs.existsSync('rust/interlink-worldgen/src/coordinates.rs'));
  assert.ok(fs.existsSync('rust/interlink-worldgen/src/tectonics.rs'));
  assert.ok(fs.existsSync('rust/interlink-worldgen/src/geology.rs'));
  assert.ok(fs.existsSync('rust/interlink-worldgen/src/lithosphere.rs'));
  assert.ok(fs.existsSync('rust/interlink-worldgen/src/refinement.rs'));
  assert.ok(fs.existsSync('rust/interlink-worldgen/src/boundary_refinement.rs'));
  assert.ok(fs.existsSync('rust/interlink-worldgen-wasm/Cargo.toml'));
  assert.ok(fs.existsSync('rust/interlink-worldgen-cli/Cargo.toml'));
  assert.ok(fs.existsSync('docs/worldgen-rewrite/GEOLOGY.md'));
  assert.ok(fs.existsSync('docs/worldgen-rewrite/LITHOSPHERE.md'));
  assert.ok(fs.existsSync('docs/worldgen-rewrite/MULTIRESOLUTION.md'));
  assert.ok(fs.existsSync('docs/worldgen-rewrite/PLANET_PARAMETERS.md'));
});

test('WG-3.75 lab renders multiresolution inheritance and fine boundary provenance without terrain', () => {
  const html = fs.readFileSync('worldgen-lab.html', 'utf8');
  assert.match(html, /WORLDGEN REWRITE · WG-3\.75/);
  assert.match(html, /Coarse physical level/);
  assert.match(html, /Fine diagnostic level/);
  assert.match(html, /Inherited coarse samples/);
  assert.match(html, /Nearest coarse provenance/);
  assert.match(html, /Boundary provenance/);
  assert.match(html, /Macro plate ownership/);
  assert.match(html, /Refined kinematic domains/);
  assert.match(html, /Fine tectonic boundaries/);
  assert.match(html, /Fine geological regimes/);
  assert.match(html, /Crust type/);
  assert.match(html, /Lithospheric strength/);
  assert.match(html, /Structural zone type/);
  assert.match(html, /Fragmentation propensity/);
  assert.match(html, /water inventory/i);
  assert.match(html, /No elevation, bathymetry, sea-level solve/);
  assert.doesNotMatch(html, /Region Inspector|resource node|NAV/i);
});
