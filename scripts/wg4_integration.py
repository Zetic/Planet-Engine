from __future__ import annotations

import json
from pathlib import Path


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    if old not in text:
        raise RuntimeError(f"anchor not found in {path}: {old[:100]!r}")
    write(path, text.replace(old, new, 1))


# WASM crate registration + protocol v7.
replace_once(
    "rust/interlink-worldgen-wasm/src/lib.rs",
    "mod inheritance_bridge;\npub use inheritance_bridge::WasmWorldgenInheritance;\n",
    "mod inheritance_bridge;\npub use inheritance_bridge::WasmWorldgenInheritance;\nmod topography_bridge;\npub use topography_bridge::WasmWorldgenTopography;\n",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/lib.rs",
    "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 6;",
    "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 7;",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/lib.rs",
    "assert_eq!(worldgen_protocol_version(), 6);",
    "assert_eq!(worldgen_protocol_version(), 7);",
)

# CLI WG-4 command.
replace_once(
    "rust/interlink-worldgen-cli/src/main.rs",
    "    generate_tectonics, inherit_physical_state, GeologyRequest, LithosphereRequest,\n    PlanetPhysicalParameters, PlateScaleClass, SyntheticRequest, TectonicFragmentKind,\n    TectonicsRequest, WORLDGEN_ENGINE_VERSION,\n",
    "    generate_tectonics, generate_initial_topography, inherit_boundary_interfaces,\n    inherit_physical_state, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters,\n    PlateScaleClass, SyntheticRequest, TectonicFragmentKind, TectonicsRequest,\n    TopographyRequest, WORLDGEN_ENGINE_VERSION,\n",
)
replace_once(
    "rust/interlink-worldgen-cli/src/main.rs",
    '"interlink-worldgen-cli <generate|benchmark|topology|tectonics|geology|lithosphere|inheritance|profile>',
    '"interlink-worldgen-cli <generate|benchmark|topology|tectonics|geology|lithosphere|inheritance|topography|profile>',
)
replace_once(
    "rust/interlink-worldgen-cli/src/main.rs",
    '            | "inheritance"\n            | "profile"',
    '            | "inheritance"\n            | "topography"\n            | "profile"',
)

topography_fn = r'''
fn topography(options: &Options) -> Result<(), String> {
    if options.coarse_level > options.level {
        return Err("--coarse-level cannot exceed --level".to_owned());
    }
    let started = Instant::now();
    let coarse = build_icosphere(options.coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(options.level).map_err(|error| error.to_string())?;
    let parameters = PlanetPhysicalParameters::earthlike_reference();
    let tectonics = generate_tectonics(
        &coarse,
        &TectonicsRequest::new(options.seed.as_str(), options.plates),
        parameters,
    )
    .map_err(|error| error.to_string())?;
    let geology = generate_crust_and_history(
        &coarse,
        &tectonics,
        &GeologyRequest::new(options.seed.as_str()),
        parameters,
    )
    .map_err(|error| error.to_string())?;
    let lithosphere = generate_lithosphere(
        &coarse,
        &tectonics,
        &geology,
        &LithosphereRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let inherited = inherit_physical_state(
        &fine,
        options.coarse_level,
        &tectonics,
        &geology,
        &lithosphere,
        parameters,
    )
    .map_err(|error| error.to_string())?;
    let boundaries = inherit_boundary_interfaces(
        &coarse,
        &fine,
        &tectonics,
        &geology,
        &inherited.plate_ids,
    )
    .map_err(|error| error.to_string())?;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        parameters,
        &TopographyRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let metrics = &terrain.metrics;
    println!("Project Interlink Planet Engine WG-4 Initial Physical Topography");
    println!("engine_version={}", WORLDGEN_ENGINE_VERSION);
    println!("stage={}@{}", terrain.stage.id, terrain.stage.version);
    println!("seed={} macro_plates={}", options.seed, options.plates);
    println!("levels coarse={} fine={}", options.coarse_level, options.level);
    println!("samples={} fine_boundaries={}", metrics.sample_count, boundaries.boundaries.len());
    println!("topography_hash={} topography_parameter_hash={}", metrics.topography_hash_hex(), metrics.parameter_hash_hex());
    println!("inheritance_hash={} boundary_hash={} planet_parameter_hash={}", inherited.inheritance_hash_hex(), boundaries.boundary_hash_hex(), parameters.parameter_hash_hex());
    println!("elevation_m min={:.3} p05={:.3} median={:.3} p95={:.3} max={:.3}", metrics.minimum_solid_elevation_m, metrics.p05_solid_elevation_m, metrics.median_solid_elevation_m, metrics.p95_solid_elevation_m, metrics.maximum_solid_elevation_m);
    match metrics.sea_level_m {
        Some(level) => println!("sea_level_m={:.6}", level),
        None => println!("sea_level_m=none"),
    }
    println!("area_fraction land={:.6} ocean={:.6}", metrics.land_area_fraction, metrics.ocean_area_fraction);
    println!("mean_land_elevation_m={:.3} mean_water_depth_m={:.3} maximum_water_depth_m={:.3}", metrics.mean_land_elevation_m, metrics.mean_water_depth_m, metrics.maximum_water_depth_m);
    println!("water_volume_m3 target={:.6e} solved={:.6e} relative_error={:.6e}", metrics.target_water_volume_m3, metrics.solved_water_volume_m3, metrics.water_volume_relative_error);
    println!("clamped_samples={}", metrics.clamped_sample_count);
    println!("upstream tectonic_hash={} geology_hash={} lithosphere_hash={}", tectonics.metrics.tectonic_hash_hex(), geology.metrics.geology_hash_hex(), lithosphere.metrics.lithosphere_hash_hex());
    println!("elapsed_ms={:.3}", started.elapsed().as_secs_f64() * 1_000.0);
    Ok(())
}

'''
replace_once(
    "rust/interlink-worldgen-cli/src/main.rs",
    "fn profile(_options: &Options) -> Result<(), String> {",
    topography_fn + "fn profile(_options: &Options) -> Result<(), String> {",
)
replace_once(
    "rust/interlink-worldgen-cli/src/main.rs",
    '        "inheritance" => inheritance(&options),\n        "profile" => profile(&options),',
    '        "inheritance" => inheritance(&options),\n        "topography" => topography(&options),\n        "profile" => profile(&options),',
)

# npm convenience script.
package_path = Path("package.json")
package = json.loads(package_path.read_text())
package["scripts"]["worldgen:topography"] = "cargo run -p interlink-worldgen-cli -- topography --seed test-world --coarse-level 4 --level 6 --plates 16"
package_path.write_text(json.dumps(package, indent=2) + "\n")

# Browser protocol v7 + WG-4 transport contract.
protocol = read("src/worldgen/protocol.ts")
protocol = protocol.replace("export const WORLDGEN_PROTOCOL_VERSION = 6;", "export const WORLDGEN_PROTOCOL_VERSION = 7;", 1)
protocol = protocol.replace(
    "export const WORLDGEN_INHERITANCE_FINE_MAX_LEVEL = 7;",
    "export const WORLDGEN_INHERITANCE_FINE_MAX_LEVEL = 7;\nexport const WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL = 6;\nexport const WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL = 7;",
    1,
)
protocol = protocol.replace(
    "export interface WorldgenInheritanceRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }",
    "export interface WorldgenInheritanceRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }\nexport interface WorldgenTopographyRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }",
    1,
)

topography_types = r'''
export interface WorldgenTopographyMetrics {
  coarseSampleCount: number;
  fineSampleCount: number;
  plateCount: number;
  fineBoundaryEdgeCount: number;
  minimumSolidElevationM: number;
  maximumSolidElevationM: number;
  meanSolidElevationM: number;
  p05SolidElevationM: number;
  medianSolidElevationM: number;
  p95SolidElevationM: number;
  hasSeaLevel: boolean;
  seaLevelM: number;
  landAreaFraction: number;
  oceanAreaFraction: number;
  meanLandElevationM: number;
  meanWaterDepthM: number;
  maximumWaterDepthM: number;
  targetWaterVolumeM3: number;
  solvedWaterVolumeM3: number;
  waterVolumeRelativeError: number;
  clampedSampleCount: number;
  coarseTopologyHash: string;
  fineTopologyHash: string;
  tectonicHash: string;
  geologyHash: string;
  lithosphereHash: string;
  inheritanceHash: string;
  boundaryHash: string;
  planetParameterHash: string;
  topographyParameterHash: string;
  topographyHash: string;
}

export interface WorldgenTopographyResult {
  engineVersion: number;
  coarseLevel: number;
  fineLevel: number;
  stage: WorldgenStageMetadata;
  metrics: WorldgenTopographyMetrics;
  parameters: WorldgenPlanetPhysicalProfile;
  positions: Float64Array;
  faces: Uint32Array;
  neighborOffsets: Uint32Array;
  neighbors: Uint32Array;
  plateIds: Uint16Array;
  crustKind: Uint8Array;
  boundarySamples: Uint32Array;
  geologicalBoundaryRegimes: Uint8Array;
  isostaticElevationM: Float32Array;
  thermalElevationM: Float32Array;
  orogenicElevationM: Float32Array;
  ridgeElevationM: Float32Array;
  riftBasinElevationM: Float32Array;
  trenchElevationM: Float32Array;
  arcElevationM: Float32Array;
  mantleDynamicElevationM: Float32Array;
  solidElevationM: Float32Array;
  elevationAboveSeaLevelM: Float32Array;
  waterDepthM: Float32Array;
  submergedMask: Uint8Array;
}

'''
anchor = "export interface WorldgenSyntheticCommand"
if anchor not in protocol:
    raise RuntimeError("protocol command anchor missing")
protocol = protocol.replace(anchor, topography_types + anchor, 1)
protocol = protocol.replace(
    "export interface WorldgenInheritanceCommand { protocolVersion: number; requestId: number; type: 'generate-inheritance'; payload: WorldgenInheritanceRequest; }\nexport type WorldgenCommand = WorldgenSyntheticCommand | WorldgenTopologyCommand | WorldgenTectonicsCommand | WorldgenGeologyCommand | WorldgenLithosphereCommand | WorldgenInheritanceCommand;",
    "export interface WorldgenInheritanceCommand { protocolVersion: number; requestId: number; type: 'generate-inheritance'; payload: WorldgenInheritanceRequest; }\nexport interface WorldgenTopographyCommand { protocolVersion: number; requestId: number; type: 'generate-topography'; payload: WorldgenTopographyRequest; }\nexport type WorldgenCommand = WorldgenSyntheticCommand | WorldgenTopologyCommand | WorldgenTectonicsCommand | WorldgenGeologyCommand | WorldgenLithosphereCommand | WorldgenInheritanceCommand | WorldgenTopographyCommand;",
    1,
)
protocol = protocol.replace(
    "export interface WorldgenGeneratedInheritanceEvent { protocolVersion: number; requestId: number; type: 'generated-inheritance'; payload: WorldgenInheritanceResult; }\nexport interface WorldgenErrorEvent",
    "export interface WorldgenGeneratedInheritanceEvent { protocolVersion: number; requestId: number; type: 'generated-inheritance'; payload: WorldgenInheritanceResult; }\nexport interface WorldgenGeneratedTopographyEvent { protocolVersion: number; requestId: number; type: 'generated-topography'; payload: WorldgenTopographyResult; }\nexport interface WorldgenErrorEvent",
    1,
)
protocol = protocol.replace(
    "WorldgenGeneratedInheritanceEvent | WorldgenErrorEvent;",
    "WorldgenGeneratedInheritanceEvent | WorldgenGeneratedTopographyEvent | WorldgenErrorEvent;",
    1,
)

validator = r'''
export function validateTopographyRequest(request: WorldgenTopographyRequest): void {
  if (!request.seed.trim()) throw new Error('WG-4 topography seed must not be empty.');
  if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL) throw new Error(`WG-4 coarse level must be an integer from 0 through ${WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL) throw new Error(`WG-4 fine level must be an integer from coarse level through ${WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES) throw new Error(`WG-4 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
  const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
  if (request.plateCount > coarseSamples) throw new Error('WG-4 plate count cannot exceed coarse topology sample count.');
}

'''
protocol = protocol.replace(
    "export function worldgenSyntheticCommand",
    validator + "export function worldgenSyntheticCommand",
    1,
)
protocol += "\nexport function worldgenTopographyCommand(requestId: number, payload: WorldgenTopographyRequest): WorldgenTopographyCommand { validateTopographyRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-topography', payload }; }\n"
write("src/worldgen/protocol.ts", protocol)

# Browser client.
client = read("src/worldgen/worldgenClient.ts")
client = client.replace("  validateSyntheticRequest,", "  validateSyntheticRequest,\n  validateTopographyRequest,", 1)
client = client.replace("  worldgenSyntheticCommand,", "  worldgenSyntheticCommand,\n  worldgenTopographyCommand,", 1)
client = client.replace("  type WorldgenSyntheticRequest,", "  type WorldgenSyntheticRequest,\n  type WorldgenTopographyRequest,\n  type WorldgenTopographyResult,", 1)
client = client.replace(" | WorldgenInheritanceResult;", " | WorldgenInheritanceResult | WorldgenTopographyResult;", 1)
client = client.replace(" | ReturnType<typeof worldgenInheritanceCommand>;", " | ReturnType<typeof worldgenInheritanceCommand> | ReturnType<typeof worldgenTopographyCommand>;", 1)
client = client.replace("  generateInheritance(request: WorldgenInheritanceRequest): Promise<WorldgenInheritanceResult>;", "  generateInheritance(request: WorldgenInheritanceRequest): Promise<WorldgenInheritanceResult>;\n  generateTopography(request: WorldgenTopographyRequest): Promise<WorldgenTopographyResult>;", 1)
client = client.replace("    generateInheritance(input) { validateInheritanceRequest(input); return request<WorldgenInheritanceResult>(worldgenInheritanceCommand(nextRequestId++, input)); },", "    generateInheritance(input) { validateInheritanceRequest(input); return request<WorldgenInheritanceResult>(worldgenInheritanceCommand(nextRequestId++, input)); },\n    generateTopography(input) { validateTopographyRequest(input); return request<WorldgenTopographyResult>(worldgenTopographyCommand(nextRequestId++, input)); },", 1)
write("src/worldgen/worldgenClient.ts", client)

# Worker protocol/WASM plumbing.
worker = read("src/worldgen/worldgenWorker.ts")
worker = worker.replace("  validateSyntheticRequest,", "  validateSyntheticRequest,\n  validateTopographyRequest,", 1)
worker = worker.replace("  type WorldgenGeneratedSyntheticEvent,", "  type WorldgenGeneratedSyntheticEvent,\n  type WorldgenGeneratedTopographyEvent,", 1)
worker = worker.replace("  type WorldgenSyntheticResult,", "  type WorldgenSyntheticResult,\n  type WorldgenTopographyResult,", 1)
wasm_topography_interface = r'''
interface WasmTopography {
  generator_version(): number; stage_id(): string; stage_version(): number; stage_seed_hex(): string;
  coarse_level(): number; fine_level(): number; coarse_sample_count(): number; fine_sample_count(): number; plate_count(): number; fine_boundary_edge_count(): number;
  topography_hash_hex(): string; topography_parameter_hash_hex(): string; inheritance_hash_hex(): string; boundary_hash_hex(): string; planet_parameter_hash_hex(): string; coarse_topology_hash_hex(): string; fine_topology_hash_hex(): string; tectonic_hash_hex(): string; geology_hash_hex(): string; lithosphere_hash_hex(): string;
  minimum_solid_elevation_m(): number; maximum_solid_elevation_m(): number; mean_solid_elevation_m(): number; p05_solid_elevation_m(): number; median_solid_elevation_m(): number; p95_solid_elevation_m(): number;
  has_sea_level(): boolean; sea_level_m(): number; land_area_fraction(): number; ocean_area_fraction(): number; mean_land_elevation_m(): number; mean_water_depth_m(): number; maximum_water_depth_m(): number; target_water_volume_m3(): number; solved_water_volume_m3(): number; water_volume_relative_error(): number; clamped_sample_count(): number;
  radius_m(): number; surface_gravity_m_s2(): number; surface_water_mass_kg(): number; equivalent_global_water_depth_m(): number; ocean_water_density_kg_per_m3(): number; isostatic_mantle_density_kg_per_m3(): number;
  positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array; plate_ids(): Uint16Array; crust_kind(): Uint8Array; boundary_samples(): Uint32Array; geological_boundary_regimes(): Uint8Array;
  isostatic_elevation_m(): Float32Array; thermal_elevation_m(): Float32Array; orogenic_elevation_m(): Float32Array; ridge_elevation_m(): Float32Array; rift_basin_elevation_m(): Float32Array; trench_elevation_m(): Float32Array; arc_elevation_m(): Float32Array; mantle_dynamic_elevation_m(): Float32Array; solid_elevation_m(): Float32Array; elevation_above_sea_level_m(): Float32Array; water_depth_m(): Float32Array; submerged_mask(): Uint8Array;
  free(): void;
}
'''
worker = worker.replace("interface WorldgenWasmModule {", wasm_topography_interface + "interface WorldgenWasmModule {", 1)
worker = worker.replace("  WasmWorldgenInheritance: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number) => WasmInheritance;", "  WasmWorldgenInheritance: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number) => WasmInheritance;\n  WasmWorldgenTopography: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number) => WasmTopography;", 1)
worker = worker.replace("WorldgenGeneratedInheritanceEvent | WorldgenErrorEvent", "WorldgenGeneratedInheritanceEvent | WorldgenGeneratedTopographyEvent | WorldgenErrorEvent", 1)

generate_topography = r'''
async function generateTopography(command: Extract<WorldgenCommand, { type: 'generate-topography' }>): Promise<WorldgenTopographyResult> {
  validateTopographyRequest(command.payload);
  const module = await loadWorldgenWasm();
  const startedAt = nowMs();
  const output = new module.WasmWorldgenTopography(command.payload.seed, command.payload.coarseLevel, command.payload.fineLevel, command.payload.plateCount);
  try {
    const positions = output.positions(); const faces = output.faces(); const neighborOffsets = output.neighbor_offsets(); const neighbors = output.neighbors();
    const plateIds = output.plate_ids(); const crustKind = output.crust_kind(); const boundarySamples = output.boundary_samples(); const geologicalBoundaryRegimes = output.geological_boundary_regimes();
    const isostaticElevationM = output.isostatic_elevation_m(); const thermalElevationM = output.thermal_elevation_m(); const orogenicElevationM = output.orogenic_elevation_m(); const ridgeElevationM = output.ridge_elevation_m(); const riftBasinElevationM = output.rift_basin_elevation_m(); const trenchElevationM = output.trench_elevation_m(); const arcElevationM = output.arc_elevation_m(); const mantleDynamicElevationM = output.mantle_dynamic_elevation_m(); const solidElevationM = output.solid_elevation_m(); const elevationAboveSeaLevelM = output.elevation_above_sea_level_m(); const waterDepthM = output.water_depth_m(); const submergedMask = output.submerged_mask();
    return {
      engineVersion: output.generator_version(), coarseLevel: output.coarse_level(), fineLevel: output.fine_level(),
      stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) },
      metrics: {
        coarseSampleCount: output.coarse_sample_count(), fineSampleCount: output.fine_sample_count(), plateCount: output.plate_count(), fineBoundaryEdgeCount: output.fine_boundary_edge_count(),
        minimumSolidElevationM: output.minimum_solid_elevation_m(), maximumSolidElevationM: output.maximum_solid_elevation_m(), meanSolidElevationM: output.mean_solid_elevation_m(), p05SolidElevationM: output.p05_solid_elevation_m(), medianSolidElevationM: output.median_solid_elevation_m(), p95SolidElevationM: output.p95_solid_elevation_m(),
        hasSeaLevel: output.has_sea_level(), seaLevelM: output.sea_level_m(), landAreaFraction: output.land_area_fraction(), oceanAreaFraction: output.ocean_area_fraction(), meanLandElevationM: output.mean_land_elevation_m(), meanWaterDepthM: output.mean_water_depth_m(), maximumWaterDepthM: output.maximum_water_depth_m(), targetWaterVolumeM3: output.target_water_volume_m3(), solvedWaterVolumeM3: output.solved_water_volume_m3(), waterVolumeRelativeError: output.water_volume_relative_error(), clampedSampleCount: output.clamped_sample_count(),
        coarseTopologyHash: output.coarse_topology_hash_hex(), fineTopologyHash: output.fine_topology_hash_hex(), tectonicHash: output.tectonic_hash_hex(), geologyHash: output.geology_hash_hex(), lithosphereHash: output.lithosphere_hash_hex(), inheritanceHash: output.inheritance_hash_hex(), boundaryHash: output.boundary_hash_hex(), planetParameterHash: output.planet_parameter_hash_hex(), topographyParameterHash: output.topography_parameter_hash_hex(), topographyHash: output.topography_hash_hex(),
      },
      parameters: { radiusM: output.radius_m(), surfaceGravityMS2: output.surface_gravity_m_s2(), surfaceWaterMassKg: output.surface_water_mass_kg(), equivalentGlobalWaterDepthM: output.equivalent_global_water_depth_m(), oceanWaterDensityKgPerM3: output.ocean_water_density_kg_per_m3(), isostaticMantleDensityKgPerM3: output.isostatic_mantle_density_kg_per_m3(), internalHeatFluxWPerM2: 0, mantleThermalExpansivityPerK: 0 },
      positions, faces, neighborOffsets, neighbors, plateIds, crustKind, boundarySamples, geologicalBoundaryRegimes,
      isostaticElevationM, thermalElevationM, orogenicElevationM, ridgeElevationM, riftBasinElevationM, trenchElevationM, arcElevationM, mantleDynamicElevationM, solidElevationM, elevationAboveSeaLevelM, waterDepthM, submergedMask,
    };
  } finally { output.free(); }
}

'''
worker = worker.replace("workerScope.addEventListener('message', async messageEvent => {", generate_topography + "workerScope.addEventListener('message', async messageEvent => {", 1)
worker = worker.replace(
    "    throw new Error(`Unsupported worldgen command '${String((command as { type?: unknown }).type)}'.`);",
    "    if (command.type === 'generate-topography') {\n      const result = await generateTopography(command);\n      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-topography', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.crustKind.buffer, result.boundarySamples.buffer, result.geologicalBoundaryRegimes.buffer, result.isostaticElevationM.buffer, result.thermalElevationM.buffer, result.orogenicElevationM.buffer, result.ridgeElevationM.buffer, result.riftBasinElevationM.buffer, result.trenchElevationM.buffer, result.arcElevationM.buffer, result.mantleDynamicElevationM.buffer, result.solidElevationM.buffer, result.elevationAboveSeaLevelM.buffer, result.waterDepthM.buffer, result.submergedMask.buffer]); return;\n    }\n    throw new Error(`Unsupported worldgen command '${String((command as { type?: unknown }).type)}'.`);",
    1,
)
write("src/worldgen/worldgenWorker.ts", worker)

# WG-4 lab HTML is the Pages root and direct compatibility page.
html = '''<!DOCTYPE html>\n<html lang="en">\n<head>\n  <meta charset="UTF-8">\n  <meta name="viewport" content="width=device-width, initial-scale=1.0">\n  <title>Planet Engine · WG-4 Lab</title>\n  <link rel="stylesheet" href="styles/base.css">\n  <link rel="stylesheet" href="styles/worldgenLab.css">\n</head>\n<body class="worldgen-lab-body">\n  <main class="worldgen-lab">\n    <header class="worldgen-lab-header"><div><p class="worldgen-lab-kicker">PLANET ENGINE · WG-4</p><h1>PLANET ENGINE LAB</h1><p>Initial physical topography derived from inherited crust, tectonic boundary provenance, lithospheric mechanics, mantle support, and an explicit global water-volume solve.</p></div></header>\n    <section class="worldgen-lab-controls" aria-label="WG-4 topography controls">\n      <label>Seed <input id="worldgen-seed" type="text" value="interlink-wg4"></label>\n      <label>Coarse physical level <input id="worldgen-coarse-level" type="number" min="0" max="6" value="4"></label>\n      <label>Fine terrain level <input id="worldgen-level" type="number" min="0" max="7" value="6"></label>\n      <label>Plate count <input id="worldgen-plates" type="number" min="4" max="48" value="16"></label>\n      <label>Projection <select id="worldgen-projection"><option value="globe">Orthographic globe</option><option value="map">Equirectangular map</option></select></label>\n      <label>Diagnostic <select id="worldgen-visualization">\n        <optgroup label="Physical surface"><option value="relative-elevation" selected>Elevation above sea level</option><option value="solid-elevation">Solid elevation / datum</option><option value="land-water">Land / water</option><option value="water-depth">Bathymetry / water depth</option></optgroup>\n        <optgroup label="Topographic forcing"><option value="isostatic">Isostatic support</option><option value="thermal">Oceanic thermal subsidence</option><option value="orogenic">Orogenic / collision uplift</option><option value="ridge-relief">Ridge relief</option><option value="rift-basin">Rift / basin subsidence</option><option value="trench-relief">Trench relief</option><option value="arc-relief">Volcanic arc relief</option><option value="mantle-relief">Mantle dynamic support</option></optgroup>\n        <optgroup label="Context"><option value="plates">Macro plate ownership</option><option value="geological-boundaries">Inherited geological boundaries</option></optgroup>\n      </select></label>\n      <button id="worldgen-generate" type="button">Generate topography</button>\n    </section>\n    <p id="worldgen-status" class="worldgen-lab-status">Initializing Planet Engine Worker…</p>\n    <section class="worldgen-lab-grid"><div class="worldgen-lab-viewport"><canvas id="worldgen-field" aria-label="WG-4 physical topography diagnostics"></canvas></div><aside><h2>Physical diagnostics</h2><div id="worldgen-metrics" class="worldgen-lab-metrics"></div><div class="worldgen-lab-note"><strong>WG-4 scope</strong><p>The solid surface combines crustal isostatic support, oceanic age subsidence, collision/ridge/rift/subduction morphology, inherited basin tendency, broad mantle support, and lithospheric mechanical filtering.</p><p>Earth-like surface-water mass is converted to a physical volume and solved against the generated basin geometry to derive sea level, bathymetry, and initial land/ocean distribution.</p><p>This is pre-erosional tectonic topography. No climate, drainage, river incision, sediment transport, glaciation, detailed lithology, resource deposits, Regions, Features, or gameplay cutover are generated here.</p></div></aside></section>\n  </main>\n  <script type="module" src="dist/worldgen/diagnostics/worldgenTopographyLabStandalone.js"></script>\n</body>\n</html>\n'''
write("index.html", html)
write("worldgen-lab.html", html)

# Protocol/browser regressions focused on WG-4 contract.
Path("tests/wg4Topography.test.ts").write_text(r'''import assert from 'node:assert/strict';
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
''')

# Documentation.
Path("docs/worldgen-rewrite/TOPOGRAPHY.md").write_text(r'''# WG-4 Initial Physical Topography

WG-4 is the first Planet Engine stage that owns authoritative solid-surface elevation and initial bathymetry. It consumes accepted WG-3.75 physical inheritance and fine boundary provenance; it does not regenerate tectonics or use a dominant arbitrary terrain-noise field.

## Stage boundary

```text
WG-3.75 inherited crust / history / lithosphere
        +
fine geological boundary provenance
        +
PlanetPhysicalParameters
        ↓
crustal isostatic support
        +
oceanic age / thermal subsidence
        +
collision, ridge, rift, trench and arc responses
        +
basin/subsidence history
        +
broad mantle dynamic support
        ↓
lithospheric mechanical filtering
        ↓
area-weighted solid-surface datum
        ↓
global surface-water volume solve
        ↓
solid elevation + sea level + water depth + land/ocean mask
```

## Physical components

The v1 terrain state keeps forcing components separately inspectable: isostatic, oceanic thermal, orogenic/collision, ridge, rift/basin, trench, arc, and mantle-dynamic elevation. The final solid surface is their mechanically filtered sum. This accounting is diagnostic and prevents tectonic relief from becoming an opaque final noise function.

Crustal support uses WG-3 thickness and density against the explicit isostatic mantle density. Oceanic and transitional crust subsides with a bounded square-root age relation. Fine inherited boundary interfaces seed geodesic distance fields for collision, spreading, rifting and polarized subduction morphology. Subduction polarity keeps trenches on the subducting plate and arc uplift on the overriding plate, with the arc peak displaced inland from the interface.

WG-3.5 effective elastic thickness, weakness, and structural fabric control a bounded finite-volume neighbor filter using WG-1 center-distance and dual-interface geometry. This is a first mechanical-response approximation, not a full elastic thin-shell solver.

## Datum and water solve

The mechanically expressed solid surface is shifted to zero area-weighted global mean. This datum is arbitrary but deterministic; physical land/ocean classification comes only after the water solve.

For a candidate sea level `S`, standing-water volume is integrated as:

```text
V(S) = Σ area_sr[i] × radius² × max(0, S - elevation[i])
```

WG-4 solves this monotonic equation against `surface_water_mass_kg / ocean_water_density_kg_per_m3`. Wet profiles therefore derive sea level from basin volume rather than a fixed land percentile. Zero-water profiles expose no fictitious sea level or submerged samples.

WG-4's water mask is an initial hydrostatic standing-water surface. Closed-basin routing, lakes, rivers, overflow and freshwater belong to later hydrology.

## Resolution

WG-4 consumes WG-3.75 coarse-to-fine inheritance. The intended global production investigation is accepted L6 physical truth inherited onto an L7 terrain substrate; lower levels remain supported for tests and fast diagnostics. WG-4 never reruns WG-2/WG-3/WG-3.5 independently at the terrain level.

## Determinism

Stage identity is `terrain:initial-topography@1` with namespace `terrain:structure:v1`. The topography hash includes stage/version/seed, WG-4 model parameters, planetary parameters, WG-3.75 inheritance identity, fine boundary identity, ordered solid elevation, sea-level state, and ordered water depth. Upstream tectonic/geology/lithosphere/inheritance hashes are not mutated.

## Explicit non-goals

WG-4 does not generate climate, drainage, river incision, erosion, sediment transport, glaciation, mature coastlines, detailed lithology, resource deposits, gameplay Regions/Features, factories, or meter-scale global terrain. Those remain downstream stages.
''')

# README pipeline and concise docs hooks.
readme = read("README.md")
readme = readme.replace("WG-4 initial physical topography (next)", "WG-4 initial physical topography", 1)
readme = readme.replace("multiresolution physical inheritance, planetary physical profiles, native diagnostics", "multiresolution physical inheritance, initial physical topography/bathymetry, planetary physical profiles, native diagnostics", 1)
write("README.md", readme)

for path, addition in {
    "docs/worldgen-rewrite/DETERMINISM.md": "\n## WG-4 topography identity\n\nWG-4 uses `terrain:structure:v1`. Its ordered topography hash binds the WG-4 stage/model parameter identity to the accepted WG-3.75 inherited-state hash, fine boundary hash, explicit planetary-parameter hash, solid-surface elevation, sea-level state, and water depth. Topography changes must not mutate accepted WG-2, WG-3, WG-3.5, or WG-3.75 identities.\n",
    "docs/worldgen-rewrite/VALIDATION.md": "\n## WG-4 initial-topography gates\n\nWG-4 acceptance requires finite sample-aligned component and final elevation fields; deterministic topography identity; unchanged upstream identities; nontrivial crustal/isostatic and tectonic relief; bounded safety-clamp use; area-weighted hypsometry; exact dry-profile behavior; and an Earth-like water-volume solve whose integrated standing-water volume matches the explicit target within numerical tolerance. Ensemble validation checks that oceanic cooling tends toward deeper old seafloor and that collision, ridge, rift, trench/arc polarity, basin tendency and mantle support produce the expected signed statistical responses without requiring any generated seed to reproduce Earth exactly.\n",
    "docs/worldgen-rewrite/RESOLUTION.md": "\n## WG-4 terrain checkpoint\n\nWG-4 now consumes accepted coarse physics exclusively through WG-3.75 inheritance. L7 remains the intended initial global terrain-quality target (~163,842 samples / ~56 km characteristic spacing on the Earth-like profile), while lower levels are used for CI and diagnostics. Finer terrain must remain a refinement of accepted physical truth rather than a rerun of tectonics.\n",
    "docs/worldgen-rewrite/PLANET_PARAMETERS.md": "\nWG-4 now consumes the explicit isostatic mantle density, surface-water mass, ocean-water density and planetary radius. The water inventory is converted to target volume and solved against generated basin geometry; zero-water profiles produce no fictitious sea-level state.\n",
}.items():
    text = read(path)
    if addition.strip() not in text:
        write(path, text.rstrip() + "\n" + addition)

# Keep docs overview current where present.
overview = Path("docs/worldgen-rewrite/README.md")
if overview.exists():
    text = overview.read_text()
    text = text.replace("WASM protocol v6", "WASM protocol v7")
    text += "\n\n## WG-4\n\nWG-4 derives initial solid elevation, tectonic bathymetry, a deterministic vertical datum, and water-volume-conserving sea level from accepted WG-3.75 truth. See `TOPOGRAPHY.md`.\n"
    overview.write_text(text)
