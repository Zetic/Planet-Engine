from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"marker not found in {path}: {old[:100]!r}")
    target.write_text(text.replace(old, new, 1))


# Protocol v11 and WG-6A contracts.
replace_once(
    "src/worldgen/protocol.ts",
    "export const WORLDGEN_PROTOCOL_VERSION = 10;",
    "export const WORLDGEN_PROTOCOL_VERSION = 11;",
)
replace_once(
    "src/worldgen/protocol.ts",
    "export const WORLDGEN_CLIMATE_FINE_MAX_LEVEL = 7;\n",
    "export const WORLDGEN_CLIMATE_FINE_MAX_LEVEL = 7;\nexport const WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL = 6;\nexport const WORLDGEN_DRAINAGE_FINE_MAX_LEVEL = 7;\nexport const WORLDGEN_INVALID_SAMPLE_ID = 0xffff_ffff;\nexport const WORLDGEN_DRAINAGE_OUTLET_NONE = 0;\nexport const WORLDGEN_DRAINAGE_OUTLET_OCEAN = 1;\nexport const WORLDGEN_DRAINAGE_OUTLET_INTERNAL = 2;\n",
)
replace_once(
    "src/worldgen/protocol.ts",
    "export interface WorldgenClimateRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }\n",
    "export interface WorldgenClimateRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }\nexport interface WorldgenDrainageRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }\n",
)

drainage_types = r'''
export interface WorldgenDrainageMetrics {
  sampleCount: number;
  landSampleCount: number;
  oceanSampleCount: number;
  basinCount: number;
  depressionCount: number;
  depressionSampleCount: number;
  landAreaM2: number;
  terminalContributingAreaM2: number;
  areaConservationRelativeError: number;
  maximumContributingAreaM2: number;
  maximumDepressionDepthM: number;
  drainageHash: string;
}

export interface WorldgenDrainageResult {
  engineVersion: number;
  coarseLevel: number;
  fineLevel: number;
  stage: WorldgenStageMetadata;
  metrics: WorldgenDrainageMetrics;
  topographyHash: string;
  topologyHash: string;
  planetParameterHash: string;
  positions: Float64Array;
  faces: Uint32Array;
  neighborOffsets: Uint32Array;
  neighbors: Uint32Array;
  solidElevationM: Float32Array;
  elevationAboveSeaLevelM: Float32Array;
  submergedMask: Uint8Array;
  receiver: Uint32Array;
  outletSample: Uint32Array;
  outletKind: Uint8Array;
  basinId: Uint32Array;
  depressionId: Uint32Array;
  hydrologicEscapeElevationM: Float32Array;
  depressionDepthM: Float32Array;
  contributingAreaM2: Float64Array;
  drainageOrder: Uint32Array;
  basinOutletSamples: Uint32Array;
  basinOutletKinds: Uint8Array;
  basinAreasM2: Float64Array;
  depressionFloorSamples: Uint32Array;
  depressionFloorElevationsM: Float64Array;
  depressionSpillElevationsM: Float64Array;
  depressionAreasM2: Float64Array;
}

'''
replace_once(
    "src/worldgen/protocol.ts",
    "export interface WorldgenSyntheticCommand { protocolVersion: number; requestId: number; type: 'generate-synthetic'; payload: WorldgenSyntheticRequest; }",
    drainage_types + "export interface WorldgenSyntheticCommand { protocolVersion: number; requestId: number; type: 'generate-synthetic'; payload: WorldgenSyntheticRequest; }",
)
replace_once(
    "src/worldgen/protocol.ts",
    "export interface WorldgenClimateCommand { protocolVersion: number; requestId: number; type: 'generate-climate'; payload: WorldgenClimateRequest; }\nexport type WorldgenCommand = WorldgenSyntheticCommand | WorldgenTopologyCommand | WorldgenTectonicsCommand | WorldgenGeologyCommand | WorldgenLithosphereCommand | WorldgenInheritanceCommand | WorldgenTopographyCommand | WorldgenClimateCommand;",
    "export interface WorldgenClimateCommand { protocolVersion: number; requestId: number; type: 'generate-climate'; payload: WorldgenClimateRequest; }\nexport interface WorldgenDrainageCommand { protocolVersion: number; requestId: number; type: 'generate-drainage'; payload: WorldgenDrainageRequest; }\nexport type WorldgenCommand = WorldgenSyntheticCommand | WorldgenTopologyCommand | WorldgenTectonicsCommand | WorldgenGeologyCommand | WorldgenLithosphereCommand | WorldgenInheritanceCommand | WorldgenTopographyCommand | WorldgenClimateCommand | WorldgenDrainageCommand;",
)
replace_once(
    "src/worldgen/protocol.ts",
    "export interface WorldgenGeneratedClimateEvent { protocolVersion: number; requestId: number; type: 'generated-climate'; payload: WorldgenClimateResult; }\nexport interface WorldgenGenerationProgressEvent",
    "export interface WorldgenGeneratedClimateEvent { protocolVersion: number; requestId: number; type: 'generated-climate'; payload: WorldgenClimateResult; }\nexport interface WorldgenGeneratedDrainageEvent { protocolVersion: number; requestId: number; type: 'generated-drainage'; payload: WorldgenDrainageResult; }\nexport interface WorldgenGenerationProgressEvent",
)
replace_once(
    "src/worldgen/protocol.ts",
    "export type WorldgenEvent = WorldgenGeneratedSyntheticEvent | WorldgenGeneratedTopologyEvent | WorldgenGeneratedTectonicsEvent | WorldgenGeneratedGeologyEvent | WorldgenGeneratedLithosphereEvent | WorldgenGeneratedInheritanceEvent | WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenGenerationProgressEvent | WorldgenErrorEvent;",
    "export type WorldgenEvent = WorldgenGeneratedSyntheticEvent | WorldgenGeneratedTopologyEvent | WorldgenGeneratedTectonicsEvent | WorldgenGeneratedGeologyEvent | WorldgenGeneratedLithosphereEvent | WorldgenGeneratedInheritanceEvent | WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenGeneratedDrainageEvent | WorldgenGenerationProgressEvent | WorldgenErrorEvent;",
)

drainage_validator = r'''
export function validateDrainageRequest(request: WorldgenDrainageRequest): void {
  if (!request.seed.trim()) throw new Error('WG-6A drainage seed must not be empty.');
  if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL) throw new Error(`WG-6A coarse level must be an integer from 0 through ${WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_DRAINAGE_FINE_MAX_LEVEL) throw new Error(`WG-6A fine level must be an integer from coarse level through ${WORLDGEN_DRAINAGE_FINE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES) throw new Error(`WG-6A plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
  const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
  if (request.plateCount > coarseSamples) throw new Error('WG-6A plate count cannot exceed coarse topology sample count.');
}

'''
replace_once(
    "src/worldgen/protocol.ts",
    "export function worldgenSyntheticCommand(requestId: number, payload: WorldgenSyntheticRequest): WorldgenSyntheticCommand",
    drainage_validator + "export function worldgenSyntheticCommand(requestId: number, payload: WorldgenSyntheticRequest): WorldgenSyntheticCommand",
)
replace_once(
    "src/worldgen/protocol.ts",
    "export function worldgenClimateCommand(requestId: number, payload: WorldgenClimateRequest): WorldgenClimateCommand { validateClimateRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-climate', payload }; }",
    "export function worldgenClimateCommand(requestId: number, payload: WorldgenClimateRequest): WorldgenClimateCommand { validateClimateRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-climate', payload }; }\nexport function worldgenDrainageCommand(requestId: number, payload: WorldgenDrainageRequest): WorldgenDrainageCommand { validateDrainageRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-drainage', payload }; }",
)

# Client contract.
replace_once(
    "src/worldgen/worldgenClient.ts",
    "  validateClimateRequest,\n",
    "  validateClimateRequest,\n  validateDrainageRequest,\n",
)
replace_once(
    "src/worldgen/worldgenClient.ts",
    "  worldgenClimateCommand,\n",
    "  worldgenClimateCommand,\n  worldgenDrainageCommand,\n",
)
replace_once(
    "src/worldgen/worldgenClient.ts",
    "  type WorldgenClimateResult,\n",
    "  type WorldgenClimateResult,\n  type WorldgenDrainageRequest,\n  type WorldgenDrainageResult,\n",
)
replace_once(
    "src/worldgen/worldgenClient.ts",
    "type WorldgenResult = WorldgenSyntheticResult | WorldgenTopologyResult | WorldgenTectonicsResult | WorldgenGeologyResult | WorldgenLithosphereResult | WorldgenInheritanceResult | WorldgenTopographyResult | WorldgenClimateResult;",
    "type WorldgenResult = WorldgenSyntheticResult | WorldgenTopologyResult | WorldgenTectonicsResult | WorldgenGeologyResult | WorldgenLithosphereResult | WorldgenInheritanceResult | WorldgenTopographyResult | WorldgenClimateResult | WorldgenDrainageResult;",
)
replace_once(
    "src/worldgen/worldgenClient.ts",
    "type WorldgenRequestCommand = ReturnType<typeof worldgenSyntheticCommand> | ReturnType<typeof worldgenTopologyCommand> | ReturnType<typeof worldgenTectonicsCommand> | ReturnType<typeof worldgenGeologyCommand> | ReturnType<typeof worldgenLithosphereCommand> | ReturnType<typeof worldgenInheritanceCommand> | ReturnType<typeof worldgenTopographyCommand> | ReturnType<typeof worldgenClimateCommand>;",
    "type WorldgenRequestCommand = ReturnType<typeof worldgenSyntheticCommand> | ReturnType<typeof worldgenTopologyCommand> | ReturnType<typeof worldgenTectonicsCommand> | ReturnType<typeof worldgenGeologyCommand> | ReturnType<typeof worldgenLithosphereCommand> | ReturnType<typeof worldgenInheritanceCommand> | ReturnType<typeof worldgenTopographyCommand> | ReturnType<typeof worldgenClimateCommand> | ReturnType<typeof worldgenDrainageCommand>;",
)
replace_once(
    "src/worldgen/worldgenClient.ts",
    "  generateClimate(request: WorldgenClimateRequest, onProgress?: (progress: WorldgenGenerationProgress) => void): Promise<WorldgenClimateResult>;\n",
    "  generateClimate(request: WorldgenClimateRequest, onProgress?: (progress: WorldgenGenerationProgress) => void): Promise<WorldgenClimateResult>;\n  generateDrainage(request: WorldgenDrainageRequest): Promise<WorldgenDrainageResult>;\n",
)
replace_once(
    "src/worldgen/worldgenClient.ts",
    "    generateClimate(input, onProgress) { validateClimateRequest(input); return request<WorldgenClimateResult>(worldgenClimateCommand(nextRequestId++, input), onProgress); },\n",
    "    generateClimate(input, onProgress) { validateClimateRequest(input); return request<WorldgenClimateResult>(worldgenClimateCommand(nextRequestId++, input), onProgress); },\n    generateDrainage(input) { validateDrainageRequest(input); return request<WorldgenDrainageResult>(worldgenDrainageCommand(nextRequestId++, input)); },\n",
)

# Worker imports.
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "  validateClimateRequest,\n",
    "  validateClimateRequest,\n  validateDrainageRequest,\n",
)
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "  type WorldgenGeneratedClimateEvent,\n",
    "  type WorldgenGeneratedClimateEvent,\n  type WorldgenGeneratedDrainageEvent,\n",
)
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "  type WorldgenGeologyResult,\n",
    "  type WorldgenGeologyResult,\n  type WorldgenDrainageResult,\n",
)

wasm_drainage = r'''
interface WasmDrainage {
  generator_version(): number; stage_id(): string; stage_version(): number; stage_seed_hex(): string;
  coarse_level(): number; fine_level(): number; sample_count(): number; plate_count(): number;
  drainage_hash_hex(): string; topography_hash_hex(): string; topology_hash_hex(): string; planet_parameter_hash_hex(): string;
  land_sample_count(): number; ocean_sample_count(): number; basin_count(): number; depression_count(): number; depression_sample_count(): number;
  land_area_m2(): number; terminal_contributing_area_m2(): number; area_conservation_relative_error(): number; maximum_contributing_area_m2(): number; maximum_depression_depth_m(): number;
  positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array;
  solid_elevation_m(): Float32Array; elevation_above_sea_level_m(): Float32Array; submerged_mask(): Uint8Array;
  receiver(): Uint32Array; outlet_sample(): Uint32Array; outlet_kind(): Uint8Array; basin_id(): Uint32Array; depression_id(): Uint32Array;
  hydrologic_escape_elevation_m(): Float32Array; depression_depth_m(): Float32Array; contributing_area_m2(): Float64Array; drainage_order(): Uint32Array;
  basin_outlet_samples(): Uint32Array; basin_outlet_kinds(): Uint8Array; basin_areas_m2(): Float64Array;
  depression_floor_samples(): Uint32Array; depression_floor_elevations_m(): Float64Array; depression_spill_elevations_m(): Float64Array; depression_areas_m2(): Float64Array;
  free(): void;
}
'''
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "interface WorldgenWasmModule {",
    wasm_drainage + "interface WorldgenWasmModule {",
)
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "  WasmWorldgenClimate: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number, progress?: (stageId: string, stageIndex: number, stageCount: number, completed: number, total: number) => void) => WasmClimate;\n",
    "  WasmWorldgenClimate: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number, progress?: (stageId: string, stageIndex: number, stageCount: number, completed: number, total: number) => void) => WasmClimate;\n  WasmWorldgenDrainage: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number) => WasmDrainage;\n",
)
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenGenerationProgressEvent",
    "WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenGeneratedDrainageEvent | WorldgenGenerationProgressEvent",
)

generate_drainage = r'''
async function generateDrainage(command: Extract<WorldgenCommand, { type: 'generate-drainage' }>): Promise<WorldgenDrainageResult> {
  validateDrainageRequest(command.payload);
  const module = await loadWorldgenWasm();
  const startedAt = nowMs();
  const output = new module.WasmWorldgenDrainage(command.payload.seed, command.payload.coarseLevel, command.payload.fineLevel, command.payload.plateCount);
  try {
    const positions = output.positions(); const faces = output.faces(); const neighborOffsets = output.neighbor_offsets(); const neighbors = output.neighbors();
    const solidElevationM = output.solid_elevation_m(); const elevationAboveSeaLevelM = output.elevation_above_sea_level_m(); const submergedMask = output.submerged_mask();
    const receiver = output.receiver(); const outletSample = output.outlet_sample(); const outletKind = output.outlet_kind(); const basinId = output.basin_id(); const depressionId = output.depression_id();
    const hydrologicEscapeElevationM = output.hydrologic_escape_elevation_m(); const depressionDepthM = output.depression_depth_m(); const contributingAreaM2 = output.contributing_area_m2(); const drainageOrder = output.drainage_order();
    const basinOutletSamples = output.basin_outlet_samples(); const basinOutletKinds = output.basin_outlet_kinds(); const basinAreasM2 = output.basin_areas_m2();
    const depressionFloorSamples = output.depression_floor_samples(); const depressionFloorElevationsM = output.depression_floor_elevations_m(); const depressionSpillElevationsM = output.depression_spill_elevations_m(); const depressionAreasM2 = output.depression_areas_m2();
    return {
      engineVersion: output.generator_version(), coarseLevel: output.coarse_level(), fineLevel: output.fine_level(),
      stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) },
      metrics: {
        sampleCount: output.sample_count(), landSampleCount: output.land_sample_count(), oceanSampleCount: output.ocean_sample_count(), basinCount: output.basin_count(), depressionCount: output.depression_count(), depressionSampleCount: output.depression_sample_count(),
        landAreaM2: output.land_area_m2(), terminalContributingAreaM2: output.terminal_contributing_area_m2(), areaConservationRelativeError: output.area_conservation_relative_error(), maximumContributingAreaM2: output.maximum_contributing_area_m2(), maximumDepressionDepthM: output.maximum_depression_depth_m(), drainageHash: output.drainage_hash_hex(),
      },
      topographyHash: output.topography_hash_hex(), topologyHash: output.topology_hash_hex(), planetParameterHash: output.planet_parameter_hash_hex(),
      positions, faces, neighborOffsets, neighbors, solidElevationM, elevationAboveSeaLevelM, submergedMask,
      receiver, outletSample, outletKind, basinId, depressionId, hydrologicEscapeElevationM, depressionDepthM, contributingAreaM2, drainageOrder,
      basinOutletSamples, basinOutletKinds, basinAreasM2, depressionFloorSamples, depressionFloorElevationsM, depressionSpillElevationsM, depressionAreasM2,
    };
  } finally { output.free(); }
}

'''
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "workerScope.addEventListener('message', async messageEvent => {",
    generate_drainage + "workerScope.addEventListener('message', async messageEvent => {",
)
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "    if (command.type === 'generate-climate') {",
    "    if (command.type === 'generate-drainage') {\n      const result = await generateDrainage(command);\n      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-drainage', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.solidElevationM.buffer, result.elevationAboveSeaLevelM.buffer, result.submergedMask.buffer, result.receiver.buffer, result.outletSample.buffer, result.outletKind.buffer, result.basinId.buffer, result.depressionId.buffer, result.hydrologicEscapeElevationM.buffer, result.depressionDepthM.buffer, result.contributingAreaM2.buffer, result.drainageOrder.buffer, result.basinOutletSamples.buffer, result.basinOutletKinds.buffer, result.basinAreasM2.buffer, result.depressionFloorSamples.buffer, result.depressionFloorElevationsM.buffer, result.depressionSpillElevationsM.buffer, result.depressionAreasM2.buffer]); return;\n    }\n    if (command.type === 'generate-climate') {",
)

# WASM protocol bump.
replace_once(
    "rust/interlink-worldgen-wasm/src/lib.rs",
    "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 10;",
    "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 11;",
)

# Existing protocol assertions follow the current browser protocol.
for test_path in ["tests/wg5Climate.test.ts", "tests/wg4Topography.test.ts", "tests/worldgenRewrite.test.ts"]:
    text = Path(test_path).read_text()
    text = text.replace("WORLDGEN_PROTOCOL_VERSION, 10", "WORLDGEN_PROTOCOL_VERSION, 11")
    text = text.replace("const PROTOCOL = 10;", "const PROTOCOL = 11;")
    text = text.replace("protocol v10", "protocol v11")
    Path(test_path).write_text(text)

# Correct the Lab timer wording: the bridge constructor includes upstream terrain generation.
replace_once(
    "src/worldgen/diagnostics/worldgenDrainageLabStandalone.ts",
    " ms WG-6A bridge`",
    " ms through WG-6A`",
)
