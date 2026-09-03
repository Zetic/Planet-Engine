from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing anchor in {path}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


def sub_once(path: str, pattern: str, replacement: str) -> None:
    p = Path(path)
    text = p.read_text()
    text, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"expected one regex match in {path}: {pattern[:100]!r}; got {count}")
    p.write_text(text)


# -----------------------------------------------------------------------------
# Core climate: retain final spin-up-year phase precipitation as diagnostic-only
# data and expose a progress-aware entry point without changing ClimateState/hash.
# -----------------------------------------------------------------------------
path = "rust/interlink-worldgen/src/climate.rs"
p = Path(path)
s = p.read_text()
anchor = """impl ClimateRequest {\n    pub fn new(seed: impl Into<String>) -> Self {\n        Self {\n            seed: seed.into(),\n            physical: ClimatePhysicalParameters::default(),\n            parameters: ClimateParameters::default(),\n        }\n    }\n}\n"""
insert = anchor + """\n#[derive(Clone, Debug, PartialEq)]\npub struct ClimateGenerationDiagnostics {\n    /// Final spin-up-year precipitation rate for each retained orbital phase.\n    /// Layout is phase-major: phase * sample_count + sample. Values are\n    /// annualized mm/year-equivalent rates for direct phase-to-phase comparison.\n    pub precipitation_phase_rate_mm_year: Vec<f32>,\n}\n"""
if anchor not in s:
    raise SystemExit("ClimateRequest anchor missing")
s = s.replace(anchor, insert, 1)
old_sig = """pub fn generate_coupled_climate(\n    topology: &GeodesicTopology,\n    terrain: &TopographyState,\n    planet: PlanetPhysicalParameters,\n    request: &ClimateRequest,\n) -> Result<ClimateState, WorldgenError> {\n"""
new_sig = """pub fn generate_coupled_climate(\n    topology: &GeodesicTopology,\n    terrain: &TopographyState,\n    planet: PlanetPhysicalParameters,\n    request: &ClimateRequest,\n) -> Result<ClimateState, WorldgenError> {\n    let mut progress = |_completed_years: u8, _maximum_years: u8| {};\n    let (climate, _) = generate_coupled_climate_internal(\n        topology, terrain, planet, request, false, &mut progress,\n    )?;\n    Ok(climate)\n}\n\npub fn generate_coupled_climate_with_diagnostics(\n    topology: &GeodesicTopology,\n    terrain: &TopographyState,\n    planet: PlanetPhysicalParameters,\n    request: &ClimateRequest,\n    progress: &mut dyn FnMut(u8, u8),\n) -> Result<(ClimateState, ClimateGenerationDiagnostics), WorldgenError> {\n    generate_coupled_climate_internal(topology, terrain, planet, request, true, progress)\n}\n\nfn generate_coupled_climate_internal(\n    topology: &GeodesicTopology,\n    terrain: &TopographyState,\n    planet: PlanetPhysicalParameters,\n    request: &ClimateRequest,\n    capture_precipitation_phases: bool,\n    progress: &mut dyn FnMut(u8, u8),\n) -> Result<(ClimateState, ClimateGenerationDiagnostics), WorldgenError> {\n"""
if old_sig not in s:
    raise SystemExit("climate function signature missing")
s = s.replace(old_sig, new_sig, 1)
anchor = """    let sample_count = topology.metrics().sample_count as usize;\n    let phase_count = usize::from(parameters.orbital_phase_count);\n    let stage_seed = derive_stage_seed(request.seed.as_str(), CLIMATE_NAMESPACE);\n"""
insert = """    let sample_count = topology.metrics().sample_count as usize;\n    let phase_count = usize::from(parameters.orbital_phase_count);\n    let mut precipitation_phase_rate_mm_year = if capture_precipitation_phases {\n        vec![0.0_f32; phase_count * sample_count]\n    } else {\n        Vec::new()\n    };\n    let stage_seed = derive_stage_seed(request.seed.as_str(), CLIMATE_NAMESPACE);\n"""
if anchor not in s:
    raise SystemExit("phase-count anchor missing")
s = s.replace(anchor, insert, 1)
anchor = """    for year in 0..parameters.maximum_spinup_years {\n        let start_temperature = temperature.clone();\n"""
insert = """    for year in 0..parameters.maximum_spinup_years {\n        progress(year, parameters.maximum_spinup_years);\n        let start_temperature = temperature.clone();\n"""
if anchor not in s:
    raise SystemExit("spinup loop anchor missing")
s = s.replace(anchor, insert, 1)
anchor = """                let moisture_after = moisture_mass.iter().sum::<f64>();\n                let expected_change = phase_evaporation - phase_precipitation;\n"""
insert = """                if capture_precipitation_phases {\n                    let offset = phase * sample_count;\n                    let annualization = phase_count as f64;\n                    for i in 0..sample_count {\n                        precipitation_phase_rate_mm_year[offset + i] =\n                            (precipitation_mass_phase[i] / cell_area_m2[i].max(1.0) * annualization)\n                                as f32;\n                    }\n                }\n                let moisture_after = moisture_mass.iter().sum::<f64>();\n                let expected_change = phase_evaporation - phase_precipitation;\n"""
if anchor not in s:
    raise SystemExit("phase precipitation diagnostic anchor missing")
s = s.replace(anchor, insert, 1)
anchor = """        final_temperature_rms_change = (squared_change / (sample_count as f64 * 1.5)).sqrt();\n        spinup_years = year + 1;\n        if spinup_years >= parameters.minimum_spinup_years\n"""
insert = """        final_temperature_rms_change = (squared_change / (sample_count as f64 * 1.5)).sqrt();\n        spinup_years = year + 1;\n        progress(spinup_years, parameters.maximum_spinup_years);\n        if spinup_years >= parameters.minimum_spinup_years\n"""
if anchor not in s:
    raise SystemExit("spinup completion anchor missing")
s = s.replace(anchor, insert, 1)
s = s.replace("    Ok(ClimateState {\n", "    let climate_state = ClimateState {\n", 1)
old_end = """        persistent_snow_potential,\n        sea_ice_potential,\n    })\n}\n\n#[cfg(test)]\n"""
new_end = """        persistent_snow_potential,\n        sea_ice_potential,\n    };\n\n    Ok((\n        climate_state,\n        ClimateGenerationDiagnostics {\n            precipitation_phase_rate_mm_year,\n        },\n    ))\n}\n\n#[cfg(test)]\n"""
if old_end not in s:
    raise SystemExit("climate state return anchor missing")
s = s.replace(old_end, new_end, 1)
p.write_text(s)

replace_once(
    "rust/interlink-worldgen/src/lib.rs",
    "    generate_coupled_climate, ClimateMetrics, ClimateParameters, ClimatePhysicalParameters,\n",
    "    generate_coupled_climate, generate_coupled_climate_with_diagnostics,\n    ClimateGenerationDiagnostics, ClimateMetrics, ClimateParameters, ClimatePhysicalParameters,\n",
)

# -----------------------------------------------------------------------------
# WASM bridge: optional JS progress callback, stage boundaries, and phase rain.
# -----------------------------------------------------------------------------
replace_once(
    "rust/interlink-worldgen-wasm/Cargo.toml",
    'wasm-bindgen = "=0.2.127"\n',
    'wasm-bindgen = "=0.2.127"\njs-sys = "0.3"\n',
)

p = Path("rust/interlink-worldgen-wasm/src/climate_bridge.rs")
s = p.read_text()
s = s.replace(
    "    build_icosphere, generate_coupled_climate, generate_crust_and_history,\n",
    "    build_icosphere, generate_coupled_climate_with_diagnostics, generate_crust_and_history,\n",
    1,
)
anchor = "use wasm_bindgen::prelude::*;\n"
helper = anchor + """\nconst GENERATION_STAGE_COUNT: u32 = 10;\n\nfn report_generation_progress(\n    callback: Option<&js_sys::Function>,\n    stage: &str,\n    stage_index: u32,\n    completed: u32,\n    total: u32,\n) {\n    let Some(callback) = callback else {\n        return;\n    };\n    let _ = callback.call5(\n        &JsValue::NULL,\n        &JsValue::from_str(stage),\n        &JsValue::from_f64(stage_index as f64),\n        &JsValue::from_f64(GENERATION_STAGE_COUNT as f64),\n        &JsValue::from_f64(completed as f64),\n        &JsValue::from_f64(total.max(1) as f64),\n    );\n}\n"""
if anchor not in s:
    raise SystemExit("wasm prelude anchor missing")
s = s.replace(anchor, helper, 1)
s = s.replace(
    "    climate_physical: ClimatePhysicalParameters,\n",
    "    climate_physical: ClimatePhysicalParameters,\n    precipitation_phase_rate_mm_year: Vec<f32>,\n",
    1,
)
s = s.replace(
    "        plate_count: u16,\n    ) -> Result<WasmWorldgenClimate, JsValue> {\n",
    "        plate_count: u16,\n        progress: Option<js_sys::Function>,\n    ) -> Result<WasmWorldgenClimate, JsValue> {\n",
    1,
)
s = s.replace(
    "        let planet = PlanetPhysicalParameters::earthlike_reference();\n        let coarse_topology =\n",
    "        let planet = PlanetPhysicalParameters::earthlike_reference();\n        let progress = progress.as_ref();\n        report_generation_progress(progress, \"coarse-topology\", 0, 0, 1);\n        let coarse_topology =\n",
    1,
)
s = s.replace(
    "            build_icosphere(coarse_level).map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let fine_topology =\n",
    "            build_icosphere(coarse_level).map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"coarse-topology\", 0, 1, 1);\n        report_generation_progress(progress, \"fine-topology\", 1, 0, 1);\n        let fine_topology =\n",
    1,
)
s = s.replace(
    "            build_icosphere(fine_level).map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let tectonics = generate_tectonics(\n",
    "            build_icosphere(fine_level).map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"fine-topology\", 1, 1, 1);\n        report_generation_progress(progress, \"tectonics\", 2, 0, 1);\n        let tectonics = generate_tectonics(\n",
    1,
)
s = s.replace(
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let geology = generate_crust_and_history(\n",
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"tectonics\", 2, 1, 1);\n        report_generation_progress(progress, \"geology\", 3, 0, 1);\n        let geology = generate_crust_and_history(\n",
    1,
)
s = s.replace(
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let lithosphere = generate_lithosphere(\n",
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"geology\", 3, 1, 1);\n        report_generation_progress(progress, \"lithosphere\", 4, 0, 1);\n        let lithosphere = generate_lithosphere(\n",
    1,
)
s = s.replace(
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let inherited = inherit_physical_state(\n",
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"lithosphere\", 4, 1, 1);\n        report_generation_progress(progress, \"inheritance\", 5, 0, 1);\n        let inherited = inherit_physical_state(\n",
    1,
)
s = s.replace(
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let boundaries = inherit_boundary_interfaces(\n",
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"inheritance\", 5, 1, 1);\n        report_generation_progress(progress, \"boundary-refinement\", 6, 0, 1);\n        let boundaries = inherit_boundary_interfaces(\n",
    1,
)
s = s.replace(
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let terrain = generate_initial_topography(\n",
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"boundary-refinement\", 6, 1, 1);\n        report_generation_progress(progress, \"topography\", 7, 0, 1);\n        let terrain = generate_initial_topography(\n",
    1,
)
s = s.replace(
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let climate_request = ClimateRequest::new(seed);\n        let climate_physical = climate_request.physical;\n        let climate = generate_coupled_climate(&fine_topology, &terrain, planet, &climate_request)\n            .map_err(|error| JsValue::from_str(&error.to_string()))?;\n",
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"topography\", 7, 1, 1);\n        let climate_request = ClimateRequest::new(seed);\n        let climate_physical = climate_request.physical;\n        report_generation_progress(\n            progress,\n            \"climate-spinup\",\n            8,\n            0,\n            climate_request.parameters.maximum_spinup_years as u32,\n        );\n        let mut climate_progress = |completed_years: u8, maximum_years: u8| {\n            report_generation_progress(\n                progress,\n                \"climate-spinup\",\n                8,\n                completed_years as u32,\n                maximum_years as u32,\n            );\n        };\n        let (climate, diagnostics) = generate_coupled_climate_with_diagnostics(\n            &fine_topology,\n            &terrain,\n            planet,\n            &climate_request,\n            &mut climate_progress,\n        )\n        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;\n",
    1,
)
s = s.replace(
    "            climate_physical,\n            coarse_topology_hash: coarse_topology.metrics().topology_hash_hex(),\n",
    "            climate_physical,\n            precipitation_phase_rate_mm_year,\n            coarse_topology_hash: coarse_topology.metrics().topology_hash_hex(),\n",
    1,
)
anchor = """    pub fn annual_precipitation_mm(&self) -> Vec<f32> {\n        self.climate.annual_precipitation_mm.clone()\n    }\n"""
insert = anchor + """    pub fn precipitation_phase_rate_mm_year(&self) -> Vec<f32> {\n        self.precipitation_phase_rate_mm_year.clone()\n    }\n"""
if anchor not in s:
    raise SystemExit("annual precipitation bridge method missing")
s = s.replace(anchor, insert, 1)
p.write_text(s)

# Native bridge test uses no JS progress callback and verifies the phase diagnostic.
p = Path("rust/interlink-worldgen-wasm/tests/climate_bridge.rs")
s = p.read_text()
s = s.replace(
    'WasmWorldgenClimate::new("wg5-wasm".to_owned(), 3, 4, 12).unwrap()',
    'WasmWorldgenClimate::new("wg5-wasm".to_owned(), 3, 4, 12, None).unwrap()',
    1,
)
anchor = """    assert_eq!(\n        output.fine_sample_count() as usize,\n        output.annual_precipitation_mm().len()\n    );\n"""
insert = anchor + """    assert_eq!(\n        output.fine_sample_count() as usize * output.orbital_phase_count() as usize,\n        output.precipitation_phase_rate_mm_year().len()\n    );\n"""
if anchor not in s:
    raise SystemExit("bridge precipitation test anchor missing")
s = s.replace(anchor, insert, 1)
p.write_text(s)

# -----------------------------------------------------------------------------
# Browser protocol/client: progress events, timings, exact seasonal rain trace.
# -----------------------------------------------------------------------------
p = Path("src/worldgen/protocol.ts")
s = p.read_text().replace("export const WORLDGEN_PROTOCOL_VERSION = 8;", "export const WORLDGEN_PROTOCOL_VERSION = 9;", 1)
anchor = """export interface WorldgenStageMetadata { id: string; version: number; stageSeed: string; durationMs: number; }\nexport interface WorldgenUnseededStageMetadata { id: string; version: number; durationMs: number; }\n"""
insert = anchor + """export interface WorldgenGenerationTiming { stageId: string; durationMs: number; }\nexport interface WorldgenGenerationProgress {\n  stageId: string;\n  stageIndex: number;\n  stageCount: number;\n  completed: number;\n  total: number;\n  elapsedMs: number;\n  stageElapsedMs: number;\n}\n"""
if anchor not in s:
    raise SystemExit("protocol stage metadata anchor missing")
s = s.replace(anchor, insert, 1)
s = s.replace(
    "  moistureBudgetRelativeError: number;\n  persistentSnowAreaFraction: number;\n",
    "  moistureBudgetRelativeError: number;\n  moistureTransportLimiterFraction: number;\n  maximumMoistureTransportSubsteps: number;\n  persistentSnowAreaFraction: number;\n",
    1,
)
s = s.replace(
    "  stage: WorldgenStageMetadata;\n  metrics: WorldgenClimateMetrics;\n",
    "  stage: WorldgenStageMetadata;\n  generationTimings: WorldgenGenerationTiming[];\n  metrics: WorldgenClimateMetrics;\n",
    1,
)
s = s.replace(
    "  annualPrecipitationMm: Float32Array;\n  precipitationSeasonality: Float32Array;\n",
    "  annualPrecipitationMm: Float32Array;\n  precipitationPhaseRateMmYear: Float32Array;\n  precipitationSeasonality: Float32Array;\n",
    1,
)
anchor = """export interface WorldgenGeneratedClimateEvent { protocolVersion: number; requestId: number; type: 'generated-climate'; payload: WorldgenClimateResult; }\nexport interface WorldgenErrorEvent { protocolVersion: number; requestId: number; type: 'error'; payload: { message: string }; }\nexport type WorldgenEvent = WorldgenGeneratedSyntheticEvent | WorldgenGeneratedTopologyEvent | WorldgenGeneratedTectonicsEvent | WorldgenGeneratedGeologyEvent | WorldgenGeneratedLithosphereEvent | WorldgenGeneratedInheritanceEvent | WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenErrorEvent;\n"""
replace = """export interface WorldgenGeneratedClimateEvent { protocolVersion: number; requestId: number; type: 'generated-climate'; payload: WorldgenClimateResult; }\nexport interface WorldgenGenerationProgressEvent { protocolVersion: number; requestId: number; type: 'progress'; payload: WorldgenGenerationProgress; }\nexport interface WorldgenErrorEvent { protocolVersion: number; requestId: number; type: 'error'; payload: { message: string }; }\nexport type WorldgenEvent = WorldgenGeneratedSyntheticEvent | WorldgenGeneratedTopologyEvent | WorldgenGeneratedTectonicsEvent | WorldgenGeneratedGeologyEvent | WorldgenGeneratedLithosphereEvent | WorldgenGeneratedInheritanceEvent | WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenGenerationProgressEvent | WorldgenErrorEvent;\n"""
if anchor not in s:
    raise SystemExit("protocol event union anchor missing")
s = s.replace(anchor, replace, 1)
p.write_text(s)

p = Path("src/worldgen/worldgenClient.ts")
s = p.read_text()
s = s.replace("  type WorldgenGeologyRequest,\n", "  type WorldgenGenerationProgress,\n  type WorldgenGeologyRequest,\n", 1)
s = s.replace(
    "interface PendingRequest { resolve: (result: WorldgenResult) => void; reject: (error: Error) => void; }",
    "interface PendingRequest { resolve: (result: WorldgenResult) => void; reject: (error: Error) => void; progress?: (progress: WorldgenGenerationProgress) => void; }",
    1,
)
s = s.replace(
    "  generateClimate(request: WorldgenClimateRequest): Promise<WorldgenClimateResult>;\n",
    "  generateClimate(request: WorldgenClimateRequest, onProgress?: (progress: WorldgenGenerationProgress) => void): Promise<WorldgenClimateResult>;\n",
    1,
)
old = """    const request = pending.get(message.requestId);\n    if (!request) return;\n    pending.delete(message.requestId);\n    if (message.type === 'error') request.reject(new Error(message.payload.message)); else request.resolve(message.payload);\n"""
new = """    const request = pending.get(message.requestId);\n    if (!request) return;\n    if (message.type === 'progress') {\n      request.progress?.(message.payload);\n      return;\n    }\n    pending.delete(message.requestId);\n    if (message.type === 'error') request.reject(new Error(message.payload.message)); else request.resolve(message.payload);\n"""
if old not in s:
    raise SystemExit("client message handler anchor missing")
s = s.replace(old, new, 1)
s = s.replace(
    "  function request<T extends WorldgenResult>(command: WorldgenRequestCommand): Promise<T> {\n",
    "  function request<T extends WorldgenResult>(command: WorldgenRequestCommand, progress?: (progress: WorldgenGenerationProgress) => void): Promise<T> {\n",
    1,
)
s = s.replace(
    "return new Promise((resolve, reject) => { pending.set(command.requestId, { resolve: result => resolve(result as T), reject }); worker.postMessage(command); });",
    "return new Promise((resolve, reject) => { pending.set(command.requestId, { resolve: result => resolve(result as T), reject, progress }); worker.postMessage(command); });",
    1,
)
s = s.replace(
    "    generateClimate(input) { validateClimateRequest(input); return request<WorldgenClimateResult>(worldgenClimateCommand(nextRequestId++, input)); },\n",
    "    generateClimate(input, onProgress) { validateClimateRequest(input); return request<WorldgenClimateResult>(worldgenClimateCommand(nextRequestId++, input), onProgress); },\n",
    1,
)
p.write_text(s)

# -----------------------------------------------------------------------------
# Worker: pass progress callback into WASM, derive stage timings, transfer phase rain.
# -----------------------------------------------------------------------------
p = Path("src/worldgen/worldgenWorker.ts")
s = p.read_text()
s = s.replace(
    "  type WorldgenGeneratedGeologyEvent,\n",
    "  type WorldgenGeneratedGeologyEvent,\n  type WorldgenGenerationProgressEvent,\n  type WorldgenGenerationTiming,\n",
    1,
)
s = s.replace(
    "  moisture_budget_relative_error(): number; persistent_snow_area_fraction(): number;",
    "  moisture_budget_relative_error(): number; moisture_transport_limiter_fraction(): number; maximum_moisture_transport_substeps(): number; persistent_snow_area_fraction(): number;",
    1,
)
s = s.replace(
    "  annual_precipitation_mm(): Float32Array; precipitation_seasonality(): Float32Array;",
    "  annual_precipitation_mm(): Float32Array; precipitation_phase_rate_mm_year(): Float32Array; precipitation_seasonality(): Float32Array;",
    1,
)
s = s.replace(
    "  WasmWorldgenClimate: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number) => WasmClimate;\n",
    "  WasmWorldgenClimate: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number, progress?: (stageId: string, stageIndex: number, stageCount: number, completed: number, total: number) => void) => WasmClimate;\n",
    1,
)
s = s.replace(
    "WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenErrorEvent",
    "WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenGenerationProgressEvent | WorldgenErrorEvent",
    1,
)
# Replace the climate generator wholesale; keep all field mappings, adding progress/timing/phase rain.
pattern = r"async function generateClimate\(command: Extract<WorldgenCommand, \{ type: 'generate-climate' \}>\): Promise<WorldgenClimateResult> \{.*?\n\}\n\nworkerScope.addEventListener"
match = re.search(pattern, s, re.S)
if not match:
    raise SystemExit("worker generateClimate block missing")
old_block = match.group(0)
# Preserve the existing extraction/return body and patch it around the constructor.
body = old_block[:-len("\n\nworkerScope.addEventListener")]
body = body.replace(
    "  const startedAt = nowMs();\n  const output = new module.WasmWorldgenClimate(command.payload.seed, command.payload.coarseLevel, command.payload.fineLevel, command.payload.plateCount);\n  try {\n",
    "  const startedAt = nowMs();\n  const timings: WorldgenGenerationTiming[] = [];\n  let activeStageId = '';\n  let activeStageStartedAt = startedAt;\n  const finalizeActiveStage = (endedAt: number): void => {\n    if (!activeStageId) return;\n    timings.push({ stageId: activeStageId, durationMs: Math.max(0, endedAt - activeStageStartedAt) });\n    activeStageId = '';\n  };\n  const progress = (stageId: string, stageIndex: number, stageCount: number, completed: number, total: number): void => {\n    const now = nowMs();\n    if (activeStageId !== stageId) {\n      finalizeActiveStage(now);\n      activeStageId = stageId;\n      activeStageStartedAt = now;\n    }\n    workerScope.postMessage({\n      protocolVersion: WORLDGEN_PROTOCOL_VERSION,\n      requestId: command.requestId,\n      type: 'progress',\n      payload: { stageId, stageIndex, stageCount, completed, total: Math.max(1, total), elapsedMs: Math.max(0, now - startedAt), stageElapsedMs: Math.max(0, now - activeStageStartedAt) },\n    });\n  };\n  const output = new module.WasmWorldgenClimate(command.payload.seed, command.payload.coarseLevel, command.payload.fineLevel, command.payload.plateCount, progress);\n  try {\n    progress('packaging', 9, 10, 0, 1);\n",
    1,
)
body = body.replace(
    "const annualPrecipitationMm = output.annual_precipitation_mm(); const precipitationSeasonality = output.precipitation_seasonality();",
    "const annualPrecipitationMm = output.annual_precipitation_mm(); const precipitationPhaseRateMmYear = output.precipitation_phase_rate_mm_year(); const precipitationSeasonality = output.precipitation_seasonality();",
    1,
)
body = body.replace(
    "moistureBudgetRelativeError: output.moisture_budget_relative_error(), persistentSnowAreaFraction:",
    "moistureBudgetRelativeError: output.moisture_budget_relative_error(), moistureTransportLimiterFraction: output.moisture_transport_limiter_fraction(), maximumMoistureTransportSubsteps: output.maximum_moisture_transport_substeps(), persistentSnowAreaFraction:",
    1,
)
body = body.replace(
    "      engineVersion: output.generator_version(), coarseLevel: output.coarse_level(), fineLevel: output.fine_level(),\n      stage:",
    "      engineVersion: output.generator_version(), coarseLevel: output.coarse_level(), fineLevel: output.fine_level(),\n      generationTimings: [],\n      stage:",
    1,
)
body = body.replace(
    "specificHumidityMean, annualPrecipitationMm, precipitationSeasonality,",
    "specificHumidityMean, annualPrecipitationMm, precipitationPhaseRateMmYear, precipitationSeasonality,",
    1,
)
# Convert direct return object into a mutable result so packaging timing is included.
body = body.replace("    return {\n      engineVersion:", "    const result: WorldgenClimateResult = {\n      engineVersion:", 1)
body = body.replace(
    "      annualMeanInsolationWM2, seasonalInsolationAmplitudeWM2, temperatureMeanK, temperatureAnnualCosK, temperatureAnnualSinK, temperatureMinK, temperatureMaxK, localPressurePa, windEastMeanMS, windNorthMeanMS, windEastAnnualCosMS, windEastAnnualSinMS, windNorthAnnualCosMS, windNorthAnnualSinMS, seaSurfaceTemperatureMeanK, seaSurfaceTemperatureAnnualCosK, seaSurfaceTemperatureAnnualSinK, currentEastMeanMS, currentNorthMeanMS, currentEastAnnualCosMS, currentEastAnnualSinMS, currentNorthAnnualCosMS, currentNorthAnnualSinMS, currentSpeedMeanMS, oceanHeatTransportIndex, specificHumidityMean, annualPrecipitationMm, precipitationPhaseRateMmYear, precipitationSeasonality, potentialEvaporationMm, moistureBalanceMm, aridityIndex, snowfallFraction, persistentSnowPotential, seaIcePotential,\n    };\n  } finally { output.free(); }\n}",
    "      annualMeanInsolationWM2, seasonalInsolationAmplitudeWM2, temperatureMeanK, temperatureAnnualCosK, temperatureAnnualSinK, temperatureMinK, temperatureMaxK, localPressurePa, windEastMeanMS, windNorthMeanMS, windEastAnnualCosMS, windEastAnnualSinMS, windNorthAnnualCosMS, windNorthAnnualSinMS, seaSurfaceTemperatureMeanK, seaSurfaceTemperatureAnnualCosK, seaSurfaceTemperatureAnnualSinK, currentEastMeanMS, currentNorthMeanMS, currentEastAnnualCosMS, currentEastAnnualSinMS, currentNorthAnnualCosMS, currentNorthAnnualSinMS, currentSpeedMeanMS, oceanHeatTransportIndex, specificHumidityMean, annualPrecipitationMm, precipitationPhaseRateMmYear, precipitationSeasonality, potentialEvaporationMm, moistureBalanceMm, aridityIndex, snowfallFraction, persistentSnowPotential, seaIcePotential,\n    };\n    progress('packaging', 9, 10, 1, 1);\n    finalizeActiveStage(nowMs());\n    result.generationTimings = timings;\n    return result;\n  } finally { output.free(); }\n}",
    1,
)
if "precipitationPhaseRateMmYear" not in body or "generationTimings" not in body:
    raise SystemExit("worker climate rewrite did not retain new fields")
s = s[:match.start()] + body + "\n\nworkerScope.addEventListener" + s[match.end():]
s = s.replace(
    "result.annualPrecipitationMm.buffer, result.precipitationSeasonality.buffer",
    "result.annualPrecipitationMm.buffer, result.precipitationPhaseRateMmYear.buffer, result.precipitationSeasonality.buffer",
    1,
)
p.write_text(s)

# -----------------------------------------------------------------------------
# Lab HTML: keep viewport/right-column widths, move view controls into diagnostics,
# move details below both columns, and add an overlay dropdown.
# -----------------------------------------------------------------------------
p = Path("index.html")
html = p.read_text()
diag_match = re.search(r"      <label>Diagnostic\n        <select id=\"worldgen-visualization\">.*?        </select>\n      </label>\n", html, re.S)
if not diag_match:
    raise SystemExit("diagnostic select block missing")
diagnostic_select = re.search(r"<select id=\"worldgen-visualization\">.*?</select>", diag_match.group(0), re.S).group(0)
for option in [
    '            <option value="winds">Seasonal prevailing winds</option>\n',
    '            <option value="currents">Seasonal surface ocean currents</option>\n',
    '            <option value="tectonic-boundaries">Fine tectonic boundaries</option>\n',
    '            <option value="geological-boundaries">Fine geological regimes</option>\n',
]:
    diagnostic_select = diagnostic_select.replace(option, "")
diagnostic_select = diagnostic_select.replace(
    '            <option value="precipitation">Annual precipitation</option>\n            <option value="precip-seasonality">Precipitation seasonality</option>',
    '            <option value="precipitation">Annual precipitation</option>\n            <option value="seasonal-precipitation">Seasonal precipitation</option>\n            <option value="precip-seasonality">Annual precipitation seasonality</option>',
    1,
)
html = html[:diag_match.start()] + html[diag_match.end():]
html = re.sub(r"      <label>Projection\n.*?      </label>\n", "", html, count=1, flags=re.S)
html = re.sub(r"      <label>Season / orbital phase\n.*?      </label>\n", "", html, count=1, flags=re.S)
old_aside = re.search(r"      <aside>\n        <h2>Physical diagnostics</h2>.*?      </aside>", html, re.S)
if not old_aside:
    raise SystemExit("old diagnostics aside missing")
new_aside = f'''      <aside class="worldgen-lab-diagnostics">\n        <h2>Diagnostics</h2>\n        <div class="worldgen-lab-diagnostic-controls">\n          <label>Diagnostic\n            {diagnostic_select}\n          </label>\n          <details id="worldgen-overlays" class="worldgen-overlay-menu">\n            <summary>Overlays <strong id="worldgen-overlay-summary">None</strong></summary>\n            <div class="worldgen-overlay-options">\n              <label><input type="checkbox" data-worldgen-overlay value="topography" data-label="Topographic contours"> Topographic contours</label>\n              <label><input type="checkbox" data-worldgen-overlay value="coastline" data-label="Coastline"> Coastline</label>\n              <label><input type="checkbox" data-worldgen-overlay value="winds" data-label="Prevailing winds"> Prevailing winds</label>\n              <label><input type="checkbox" data-worldgen-overlay value="currents" data-label="Surface currents"> Surface currents</label>\n              <label><input type="checkbox" data-worldgen-overlay value="tectonic-boundaries" data-label="Tectonic boundaries"> Tectonic boundaries</label>\n              <label><input type="checkbox" data-worldgen-overlay value="geological-boundaries" data-label="Geological boundaries"> Geological boundaries</label>\n            </div>\n          </details>\n          <label>Map view\n            <select id="worldgen-projection">\n              <option value="globe">Orthographic globe</option>\n              <option value="map">Equirectangular map</option>\n            </select>\n          </label>\n          <label>Season / orbital phase\n            <input id="worldgen-season" type="range" min="0" max="1000" value="0" step="1">\n            <span id="worldgen-season-value">0.0% orbit</span>\n          </label>\n        </div>\n      </aside>'''
html = html[:old_aside.start()] + new_aside + html[old_aside.end():]
details = '''\n\n    <section class="worldgen-lab-details" aria-label="Planet details and generation telemetry">\n      <div class="worldgen-lab-detail-block">\n        <div class="worldgen-lab-detail-heading">\n          <h2>Generation</h2>\n          <span id="worldgen-generation-timer">Idle</span>\n        </div>\n        <div class="worldgen-generation-current">\n          <strong id="worldgen-generation-stage">Ready</strong>\n          <span id="worldgen-generation-step"></span>\n        </div>\n        <progress id="worldgen-generation-progress" max="100" value="0"></progress>\n        <div id="worldgen-generation-profile" class="worldgen-generation-profile"></div>\n      </div>\n      <div class="worldgen-lab-detail-block">\n        <h2>Physical diagnostics</h2>\n        <div id="worldgen-metrics" class="worldgen-lab-metrics"></div>\n      </div>\n      <div class="worldgen-lab-note">\n        <strong>Current physical frontier: WG-5</strong>\n        <p>One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, and WG-5 coupled-climate pipeline. Every diagnostic mode inspects that same generated planet.</p>\n        <p>Overlays are independent of the selected diagnostic, so climate fields can be compared directly against topographic contours, coastlines, boundaries, winds, and currents. Seasonal precipitation displays the retained final spin-up-year orbital phases; annual precipitation seasonality remains an annual summary statistic.</p>\n        <p>The physical surface remains pre-erosional. Drainage, river incision, sediment transport, glacier flow, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.</p>\n      </div>\n    </section>\n'''
html = html.replace("    </section>\n  </main>", "    </section>" + details + "  </main>", 1)
p.write_text(html)
Path("worldgen-lab.html").write_text(html)

# CSS replacement preserves the existing grid and viewport dimensions exactly.
Path("styles/worldgenLab.css").write_text(r'''.worldgen-lab-body { overflow: auto; background: #080d14; }
.worldgen-lab { width: min(1500px, 100%); max-width: none; min-height: 100vh; margin: 0 auto; padding: 24px; }
.worldgen-lab-header { position: static; display: flex; justify-content: space-between; align-items: flex-start; gap: 24px; padding: 0 0 18px; background: transparent; border-bottom: 1px solid #253246; }
.worldgen-lab-header h1 { margin: 2px 0 4px; color: #b7d6ff; font-size: 1.45rem; letter-spacing: 0.14em; }
.worldgen-lab-header p { color: #7f91a7; }
.worldgen-lab-kicker { color: #5f7898 !important; font-size: 11px; letter-spacing: 0.15em; }
.worldgen-lab-controls { display: flex; align-items: end; gap: 12px; flex-wrap: wrap; margin: 18px 0 12px; }
.worldgen-lab-controls label,
.worldgen-lab-diagnostic-controls label { display: grid; gap: 4px; color: #7f91a7; font-size: 12px; }
.worldgen-lab-controls input,
.worldgen-lab-controls select,
.worldgen-lab-diagnostic-controls input,
.worldgen-lab-diagnostic-controls select { width: 180px; padding: 7px 9px; background: #101925; border: 1px solid #2d4057; color: #d6e3f2; font: inherit; }
.worldgen-lab-controls input:focus,
.worldgen-lab-controls select:focus,
.worldgen-lab-diagnostic-controls input:focus,
.worldgen-lab-diagnostic-controls select:focus { outline: none; border-color: #6699cc; }
.worldgen-lab-controls input[type="number"] { width: 110px; }
.worldgen-lab-controls select { min-width: 180px; }
.worldgen-lab-status { min-height: 24px; margin-bottom: 14px; color: #92a9c3; }
.worldgen-lab-status--ok { color: #8fc7a3; }
.worldgen-lab-status--error { color: #e59595; }
.worldgen-lab-grid { display: grid; grid-template-columns: minmax(0, 1fr) minmax(260px, 340px); gap: 18px; align-items: start; }
.worldgen-lab-viewport { min-height: 360px; background: #06090e; border: 1px solid #253246; display: grid; place-items: center; overflow: hidden; }
.worldgen-lab-viewport canvas { width: 100%; height: auto; image-rendering: pixelated; display: block; touch-action: none; cursor: grab; user-select: none; }
.worldgen-lab-viewport canvas:active { cursor: grabbing; }
.worldgen-lab-grid aside { background: #0d151f; border: 1px solid #253246; padding: 14px; }
.worldgen-lab-grid h2,
.worldgen-lab-details h2 { margin-bottom: 10px; }
.worldgen-lab-diagnostic-controls { display: grid; gap: 14px; }
.worldgen-lab-diagnostic-controls select,
.worldgen-lab-diagnostic-controls input[type="range"] { width: 100%; }
.worldgen-overlay-menu { position: relative; border: 1px solid #2d4057; background: #101925; color: #7f91a7; font-size: 12px; }
.worldgen-overlay-menu summary { display: flex; justify-content: space-between; gap: 10px; padding: 8px 9px; cursor: pointer; list-style: none; }
.worldgen-overlay-menu summary::-webkit-details-marker { display: none; }
.worldgen-overlay-menu summary strong { color: #c1d0df; font-weight: normal; text-align: right; }
.worldgen-overlay-options { display: grid; gap: 7px; padding: 8px 9px 10px; border-top: 1px solid #253246; }
.worldgen-overlay-options label { display: flex; grid-template-columns: none; align-items: center; gap: 8px; }
.worldgen-overlay-options input { width: auto; margin: 0; padding: 0; }
.worldgen-lab-details { margin-top: 18px; display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 18px; }
.worldgen-lab-detail-block { background: #0d151f; border: 1px solid #253246; padding: 14px; }
.worldgen-lab-detail-heading { display: flex; justify-content: space-between; gap: 16px; align-items: baseline; }
.worldgen-lab-detail-heading span { color: #92a9c3; font-variant-numeric: tabular-nums; }
.worldgen-generation-current { display: flex; justify-content: space-between; gap: 12px; min-height: 22px; color: #92a9c3; font-size: 12px; }
.worldgen-generation-current strong { color: #c1d0df; font-weight: normal; }
#worldgen-generation-progress { width: 100%; height: 12px; margin: 8px 0 12px; }
.worldgen-generation-profile { display: grid; gap: 4px; }
.worldgen-generation-profile > div { display: grid; grid-template-columns: minmax(130px, 1fr) auto auto; gap: 12px; padding: 3px 0; border-bottom: 1px dotted #243144; color: #9eb0c4; font-size: 12px; }
.worldgen-generation-profile span:nth-child(2),
.worldgen-generation-profile span:nth-child(3) { font-variant-numeric: tabular-nums; color: #c1d0df; }
.worldgen-lab-metrics { display: grid; gap: 5px; }
.worldgen-lab-metrics > div { display: grid; grid-template-columns: 150px minmax(0, 1fr); gap: 8px; border-bottom: 1px dotted #243144; padding: 4px 0; }
.worldgen-lab-metrics strong { color: #6f849d; font-weight: normal; }
.worldgen-lab-metrics span { overflow-wrap: anywhere; color: #c1d0df; }
.worldgen-lab-note { grid-column: 1 / -1; border: 1px solid #253246; background: #0d151f; padding: 12px 14px; color: #7f91a7; font-size: 12px; }
.worldgen-lab-note strong { color: #a9bfd8; }
.worldgen-lab-note p + p { margin-top: 8px; }
@media (max-width: 900px) { .worldgen-lab-grid, .worldgen-lab-details { grid-template-columns: 1fr; } .worldgen-lab-header { flex-direction: column; } .worldgen-lab-note { grid-column: auto; } }
''')

# -----------------------------------------------------------------------------
# Lab renderer / controls / telemetry.
# -----------------------------------------------------------------------------
p = Path("src/worldgen/diagnostics/worldgenClimateLabStandalone.ts")
s = p.read_text()
s = s.replace(
    "  type WorldgenClimateResult,\n",
    "  type WorldgenClimateResult,\n  type WorldgenGenerationProgress,\n",
    1,
)
anchor = """function magnitudeField(east: Float32Array, north: Float32Array, scratch: Float32Array): Float32Array {\n  for (let index = 0; index < scratch.length; index += 1) scratch[index] = Math.hypot(east[index]!, north[index]!);\n  return scratch;\n}\n"""
insert = anchor + """function seasonalPhaseRate(\n  phases: Float32Array,\n  phaseCount: number,\n  sampleCount: number,\n  phase: number,\n  scratch: Float32Array,\n): Float32Array {\n  if (phaseCount <= 0 || phases.length !== phaseCount * sampleCount) {\n    scratch.fill(0);\n    return scratch;\n  }\n  const scaled = ((phase % 1) + 1) % 1 * phaseCount;\n  const lower = Math.floor(scaled) % phaseCount;\n  const upper = (lower + 1) % phaseCount;\n  const t = scaled - Math.floor(scaled);\n  const lowerOffset = lower * sampleCount;\n  const upperOffset = upper * sampleCount;\n  for (let index = 0; index < sampleCount; index += 1) {\n    scratch[index] = phases[lowerOffset + index]! * (1 - t) + phases[upperOffset + index]! * t;\n  }\n  return scratch;\n}\n"""
if anchor not in s:
    raise SystemExit("magnitude field anchor missing")
s = s.replace(anchor, insert, 1)
s = s.replace(
    "    case 'precipitation': return { values: result.annualPrecipitationMm, minimum: 0, maximum: 2_500, lowHue: 45, highHue: 205 };\n",
    "    case 'precipitation': return { values: result.annualPrecipitationMm, minimum: 0, maximum: 2_500, lowHue: 45, highHue: 205 };\n    case 'seasonal-precipitation': return { values: seasonalPhaseRate(result.precipitationPhaseRateMmYear, result.metrics.orbitalPhaseCount, result.metrics.fineSampleCount, phase, seasonalScratch), minimum: 0, maximum: 5_000, lowHue: 45, highHue: 205 };\n",
    1,
)
s = s.replace(
    "const phaseKey = ['seasonal-temperature', 'seasonal-sst'].includes(mode) ? phase.toFixed(3) : 'mean';",
    "const phaseKey = ['seasonal-temperature', 'seasonal-sst', 'seasonal-precipitation'].includes(mode) ? phase.toFixed(3) : 'mean';",
    1,
)
s = s.replace(
    "const cacheKey = `${mode}:${['seasonal-temperature', 'seasonal-sst'].includes(mode) ? phase.toFixed(3) : 'mean'}`;",
    "const cacheKey = `${mode}:${['seasonal-temperature', 'seasonal-sst', 'seasonal-precipitation'].includes(mode) ? phase.toFixed(3) : 'mean'}`;",
    1,
)
# Overlay helpers operate only on projected sample buffers, so canvas dimensions remain untouched.
overlay_helpers = r'''
type EdgeOverlayCache = {
  result: WorldgenClimateResult | null;
  coastline: Uint32Array;
  contours: Array<{ level: number; pairs: Uint32Array }>;
};
let edgeOverlayCache: EdgeOverlayCache = { result: null, coastline: new Uint32Array(0), contours: [] };
const TOPOGRAPHIC_CONTOURS_M = [500, 1_000, 2_000, 3_000, 4_500] as const;

function ensureEdgeOverlayCache(result: WorldgenClimateResult): EdgeOverlayCache {
  if (edgeOverlayCache.result === result) return edgeOverlayCache;
  const coastline: number[] = [];
  const contourPairs = TOPOGRAPHIC_CONTOURS_M.map(() => [] as number[]);
  for (let a = 0; a < result.metrics.fineSampleCount; a += 1) {
    const start = result.neighborOffsets[a]!;
    const end = result.neighborOffsets[a + 1]!;
    for (let cursor = start; cursor < end; cursor += 1) {
      const b = result.neighbors[cursor]!;
      if (b <= a) continue;
      if (result.submergedMask[a] !== result.submergedMask[b]) coastline.push(a, b);
      if (result.submergedMask[a] || result.submergedMask[b]) continue;
      const ea = result.elevationAboveSeaLevelM[a]!;
      const eb = result.elevationAboveSeaLevelM[b]!;
      for (let levelIndex = 0; levelIndex < TOPOGRAPHIC_CONTOURS_M.length; levelIndex += 1) {
        const level = TOPOGRAPHIC_CONTOURS_M[levelIndex]!;
        if ((ea < level && eb >= level) || (eb < level && ea >= level)) contourPairs[levelIndex]!.push(a, b);
      }
    }
  }
  edgeOverlayCache = {
    result,
    coastline: Uint32Array.from(coastline),
    contours: TOPOGRAPHIC_CONTOURS_M.map((level, index) => ({ level, pairs: Uint32Array.from(contourPairs[index]!) })),
  };
  return edgeOverlayCache;
}

function strokeSamplePairs(context: CanvasRenderingContext2D, pairs: Uint32Array, buffers: ProjectionBuffers, projection: string, width: number, strokeStyle: string, lineWidth: number): void {
  context.save();
  context.strokeStyle = strokeStyle;
  context.lineWidth = lineWidth;
  context.lineCap = 'round';
  context.beginPath();
  for (let cursor = 0; cursor < pairs.length; cursor += 2) {
    const a = pairs[cursor]!, b = pairs[cursor + 1]!;
    if (!buffers.visible[a] || !buffers.visible[b]) continue;
    const ax = buffers.x[a]!, bx = buffers.x[b]!;
    if (projection === 'map' && Math.abs(ax - bx) > width * 0.45) continue;
    context.moveTo(ax, buffers.y[a]!);
    context.lineTo(bx, buffers.y[b]!);
  }
  context.stroke();
  context.restore();
}

function drawBoundaryOverlay(context: CanvasRenderingContext2D, result: WorldgenClimateResult, kind: 'tectonic-boundaries' | 'geological-boundaries', projection: string, width: number, buffers: ProjectionBuffers): void {
  const buckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => kind === 'tectonic-boundaries'
    ? tectonicBoundaryColor(result.boundaryKinds[boundary]!)
    : geologicalBoundaryColor(result.geologicalBoundaryRegimes[boundary]!));
  context.save();
  context.lineCap = 'round';
  context.lineWidth = 1.8;
  for (const bucket of buckets) {
    context.strokeStyle = bucket.color;
    context.beginPath();
    for (let cursor = 0; cursor < bucket.indices.length; cursor += 1) {
      const boundary = bucket.indices[cursor]!;
      const a = result.boundarySamples[boundary * 2]!, b = result.boundarySamples[boundary * 2 + 1]!;
      if (!buffers.visible[a] || !buffers.visible[b]) continue;
      const ax = buffers.x[a]!, bx = buffers.x[b]!;
      if (projection === 'map' && Math.abs(ax - bx) > width * 0.45) continue;
      context.moveTo(ax, buffers.y[a]!);
      context.lineTo(bx, buffers.y[b]!);
    }
    context.stroke();
  }
  context.restore();
}

function drawDiagnosticOverlays(context: CanvasRenderingContext2D, result: WorldgenClimateResult, overlays: ReadonlySet<string>, phase: number, projection: string, yaw: number, pitch: number, width: number, height: number, buffers: ProjectionBuffers, animation: number): void {
  if (overlays.size === 0) return;
  const edgeCache = ensureEdgeOverlayCache(result);
  if (overlays.has('topography')) {
    const alphas = [0.24, 0.32, 0.42, 0.54, 0.68];
    for (let index = 0; index < edgeCache.contours.length; index += 1) {
      strokeSamplePairs(context, edgeCache.contours[index]!.pairs, buffers, projection, width, `rgba(245,248,252,${alphas[index]!})`, index >= 3 ? 1.1 : 0.8);
    }
  }
  if (overlays.has('coastline')) strokeSamplePairs(context, edgeCache.coastline, buffers, projection, width, 'rgba(225,236,246,0.78)', 1.15);
  if (overlays.has('tectonic-boundaries')) drawBoundaryOverlay(context, result, 'tectonic-boundaries', projection, width, buffers);
  if (overlays.has('geological-boundaries')) drawBoundaryOverlay(context, result, 'geological-boundaries', projection, width, buffers);
  if (overlays.has('winds')) drawVectors(context, result, 'winds', phase, projection, yaw, pitch, width, height, buffers, animation);
  if (overlays.has('currents')) drawVectors(context, result, 'currents', phase, projection, yaw, pitch, width, height, buffers, animation);
}

'''
marker = "function renderPlanet(canvas: HTMLCanvasElement, result: WorldgenClimateResult, projection: string, mode: string, phase: number, yaw: number, pitch: number, buffers: ProjectionBuffers, interactive: boolean, animation: number): void {"
if marker not in s:
    raise SystemExit("renderPlanet marker missing")
s = s.replace(marker, overlay_helpers + "function renderPlanet(canvas: HTMLCanvasElement, result: WorldgenClimateResult, projection: string, mode: string, overlays: ReadonlySet<string>, phase: number, yaw: number, pitch: number, buffers: ProjectionBuffers, interactive: boolean, animation: number): void {", 1)
s = s.replace(
    "  if (mode === 'winds' || mode === 'currents') drawVectors(context, result, mode, phase, projection, yaw, pitch, width, height, buffers, animation);\n}",
    "  if (mode === 'winds' || mode === 'currents') drawVectors(context, result, mode, phase, projection, yaw, pitch, width, height, buffers, animation);\n  drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation);\n}",
    1,
)
# DOM controls + telemetry.
anchor = """const visualization = element<HTMLSelectElement>('worldgen-visualization');\nconst season = element<HTMLInputElement>('worldgen-season');\nconst seasonValue = element<HTMLElement>('worldgen-season-value');\nconst generate = element<HTMLButtonElement>('worldgen-generate');\nconst status = element<HTMLElement>('worldgen-status');\n"""
insert = """const visualization = element<HTMLSelectElement>('worldgen-visualization');\nconst season = element<HTMLInputElement>('worldgen-season');\nconst seasonValue = element<HTMLElement>('worldgen-season-value');\nconst overlaySummary = element<HTMLElement>('worldgen-overlay-summary');\nconst overlayInputs = Array.from(document.querySelectorAll<HTMLInputElement>('input[data-worldgen-overlay]'));\nconst generate = element<HTMLButtonElement>('worldgen-generate');\nconst status = element<HTMLElement>('worldgen-status');\nconst generationProgress = element<HTMLProgressElement>('worldgen-generation-progress');\nconst generationStage = element<HTMLElement>('worldgen-generation-stage');\nconst generationStep = element<HTMLElement>('worldgen-generation-step');\nconst generationTimer = element<HTMLElement>('worldgen-generation-timer');\nconst generationProfile = element<HTMLElement>('worldgen-generation-profile');\n"""
if anchor not in s:
    raise SystemExit("lab DOM anchor missing")
s = s.replace(anchor, insert, 1)
# Reset overlay cache on new generation.
s = s.replace(
    "    styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };\n",
    "    styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };\n    edgeOverlayCache = { result: null, coastline: new Uint32Array(0), contours: [] };\n",
    1,
)
# Selected overlay helpers and progress UI before orbitalPhase.
anchor = """const VECTOR_ANIMATION_INTERVAL_MS = 50;\n\nfunction orbitalPhase(): number { return Number(season.value) / 1000; }\n"""
insert = """const VECTOR_ANIMATION_INTERVAL_MS = 50;\nconst GENERATION_STAGE_LABELS: Record<string, string> = {\n  'coarse-topology': 'Coarse topology',\n  'fine-topology': 'Fine topology',\n  tectonics: 'Tectonics',\n  geology: 'Geological history',\n  lithosphere: 'Lithosphere',\n  inheritance: 'Fine-topology inheritance',\n  'boundary-refinement': 'Boundary refinement',\n  topography: 'Topography + sea level',\n  'climate-spinup': 'Climate spin-up',\n  packaging: 'Packaging / transfer',\n};\nlet generationStartedAt = 0;\nlet generationTimerHandle: ReturnType<typeof setInterval> | null = null;\n\nfunction selectedOverlays(): Set<string> {\n  return new Set(overlayInputs.filter(input => input.checked).map(input => input.value));\n}\nfunction updateOverlaySummary(): void {\n  const selected = overlayInputs.filter(input => input.checked);\n  if (selected.length === 0) overlaySummary.textContent = 'None';\n  else if (selected.length === 1) overlaySummary.textContent = selected[0]!.dataset.label ?? selected[0]!.value;\n  else overlaySummary.textContent = `${selected.length} selected`;\n}\nfunction formatDuration(ms: number): string {\n  if (ms < 1_000) return `${ms.toFixed(0)} ms`;\n  return `${(ms / 1_000).toFixed(2)} s`;\n}\nfunction startGenerationTelemetry(): void {\n  generationStartedAt = performance.now();\n  generationProgress.value = 0;\n  generationStage.textContent = 'Starting';\n  generationStep.textContent = '';\n  generationProfile.replaceChildren();\n  if (generationTimerHandle) clearInterval(generationTimerHandle);\n  const updateTimer = (): void => { generationTimer.textContent = formatDuration(performance.now() - generationStartedAt); };\n  updateTimer();\n  generationTimerHandle = setInterval(updateTimer, 100);\n}\nfunction handleGenerationProgress(progress: WorldgenGenerationProgress): void {\n  const stageFraction = progress.total > 0 ? Math.max(0, Math.min(1, progress.completed / progress.total)) : 0;\n  generationProgress.value = Math.max(0, Math.min(100, (progress.stageIndex + stageFraction) / Math.max(1, progress.stageCount) * 100));\n  generationStage.textContent = GENERATION_STAGE_LABELS[progress.stageId] ?? progress.stageId;\n  generationStep.textContent = progress.stageId === 'climate-spinup'\n    ? `year ${progress.completed} / max ${progress.total}`\n    : progress.completed >= progress.total ? 'complete' : 'running';\n}\nfunction showGenerationProfile(result: WorldgenClimateResult): void {\n  generationProfile.replaceChildren();\n  const total = result.generationTimings.reduce((sum, timing) => sum + timing.durationMs, 0);\n  for (const timing of result.generationTimings) {\n    const row = document.createElement('div');\n    const label = document.createElement('span');\n    const duration = document.createElement('span');\n    const share = document.createElement('span');\n    label.textContent = GENERATION_STAGE_LABELS[timing.stageId] ?? timing.stageId;\n    duration.textContent = formatDuration(timing.durationMs);\n    share.textContent = total > 0 ? `${(timing.durationMs / total * 100).toFixed(1)}%` : '—';\n    row.append(label, duration, share);\n    generationProfile.append(row);\n  }\n}\nfunction finishGenerationTelemetry(result: WorldgenClimateResult): void {\n  if (generationTimerHandle) { clearInterval(generationTimerHandle); generationTimerHandle = null; }\n  generationProgress.value = 100;\n  generationStage.textContent = 'Complete';\n  generationStep.textContent = `${result.metrics.spinupYears} climate spin-up years`;\n  generationTimer.textContent = formatDuration(result.stage.durationMs);\n  showGenerationProfile(result);\n}\n\nfunction orbitalPhase(): number { return Number(season.value) / 1000; }\n"""
if anchor not in s:
    raise SystemExit("animation constant anchor missing")
s = s.replace(anchor, insert, 1)
s = s.replace(
    "  renderPlanet(canvas, current, projection.value, visualization.value, orbitalPhase(), yaw, pitch, buffers, interactive, animationPhase);",
    "  renderPlanet(canvas, current, projection.value, visualization.value, selectedOverlays(), orbitalPhase(), yaw, pitch, buffers, interactive, animationPhase);",
    1,
)
s = s.replace(
    "  if (visualization.value === 'winds' || visualization.value === 'currents') animationRequest = requestAnimationFrame(vectorAnimationFrame);",
    "  const overlays = selectedOverlays();\n  if (visualization.value === 'winds' || visualization.value === 'currents' || overlays.has('winds') || overlays.has('currents')) animationRequest = requestAnimationFrame(vectorAnimationFrame);",
    1,
)
# More useful permanent diagnostics from WG-5 stage 4.
s = s.replace(
    "  metric(metrics, 'Moisture budget error', result.metrics.moistureBudgetRelativeError.toExponential(2));\n",
    "  metric(metrics, 'Moisture budget error', result.metrics.moistureBudgetRelativeError.toExponential(2));\n  metric(metrics, 'Moisture limiter', `${(result.metrics.moistureTransportLimiterFraction * 100).toFixed(4)}% donor steps`);\n  metric(metrics, 'Moisture substeps', `${result.metrics.maximumMoistureTransportSubsteps} maximum`);\n",
    1,
)
# Generation function: live progress callback and profile.
s = s.replace(
    "  generate.disabled = true;\n  status.textContent = 'Generating one physical planet through WG-5 coupled climate in Rust/WASM…';\n  try {\n    const loaded = await client.generateClimate({ seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) });\n",
    "  generate.disabled = true;\n  startGenerationTelemetry();\n  status.textContent = 'Generating one physical planet through WG-5 coupled climate in Rust/WASM…';\n  try {\n    const loaded = await client.generateClimate(\n      { seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) },\n      handleGenerationProgress,\n    );\n",
    1,
)
s = s.replace(
    "    showMetrics(loaded); redraw(false); updateAnimation();\n    status.textContent = `Planet ready through WG-5:",
    "    showMetrics(loaded); redraw(false); updateAnimation(); finishGenerationTelemetry(loaded);\n    status.textContent = `Planet ready through WG-5:",
    1,
)
s = s.replace(
    "  } catch (error) {\n    status.textContent = error instanceof Error ? error.message : String(error);\n  } finally {\n",
    "  } catch (error) {\n    if (generationTimerHandle) { clearInterval(generationTimerHandle); generationTimerHandle = null; }\n    generationStage.textContent = 'Generation failed';\n    generationStep.textContent = '';\n    status.textContent = error instanceof Error ? error.message : String(error);\n  } finally {\n",
    1,
)
# Overlay changes redraw independently from the base diagnostic.
anchor = """projection.addEventListener('change', () => redraw(false));\nvisualization.addEventListener('change', () => { styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; redraw(false); updateAnimation(); });\nseason.addEventListener('input', () => { updateSeasonLabel(); styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; redraw(false); });\n"""
replace = """projection.addEventListener('change', () => redraw(false));\nvisualization.addEventListener('change', () => { styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; redraw(false); updateAnimation(); });\noverlayInputs.forEach(input => input.addEventListener('change', () => { updateOverlaySummary(); redraw(false); updateAnimation(); }));\nseason.addEventListener('input', () => { updateSeasonLabel(); styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; redraw(false); });\n"""
if anchor not in s:
    raise SystemExit("lab event listener anchor missing")
s = s.replace(anchor, replace, 1)
# Ensure initial overlay label is correct.
s = s.replace("updateSeasonLabel();\nvoid generatePlanet();", "updateSeasonLabel();\nupdateOverlaySummary();\nvoid generatePlanet();", 1)
p.write_text(s)

# -----------------------------------------------------------------------------
# Browser regressions: protocol 9, UI structure, seasonal rain, progress/overlays,
# and invariant canvas dimensions.
# -----------------------------------------------------------------------------
p = Path("tests/wg5Climate.test.ts")
s = p.read_text()
s = s.replace("assert.equal(WORLDGEN_PROTOCOL_VERSION, 8);", "assert.equal(WORLDGEN_PROTOCOL_VERSION, 9);", 1)
s = s.replace("    protocolVersion: 8,", "    protocolVersion: 9,", 1)
s = s.replace(
    "    'Seasonal surface ocean currents', 'Annual precipitation', 'Aridity index',\n",
    "    'Annual precipitation', 'Seasonal precipitation', 'Annual precipitation seasonality', 'Aridity index',\n",
    1,
)
# Add dedicated observability regression after the first Lab test.
anchor = """  assert.match(source, /redraw\\(true\\)/);\n});\n\n"""
new_test = anchor + r'''test('WG-5 Lab preserves viewport dimensions while splitting diagnostics, overlays, and details', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  const css = fs.readFileSync('styles/worldgenLab.css', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(html, /class="worldgen-lab-diagnostics"/);
  assert.match(html, /id="worldgen-overlays"/);
  assert.match(html, /Topographic contours/);
  assert.match(html, /id="worldgen-projection"/);
  assert.match(html, /class="worldgen-lab-details"/);
  assert.match(html, /id="worldgen-generation-progress"/);
  assert.match(css, /grid-template-columns:\s*minmax\(0, 1fr\) minmax\(260px, 340px\)/);
  assert.match(source, /const width = 1100;/);
  assert.match(source, /projection === 'map' \? 550 : 760/);
  assert.match(source, /precipitationPhaseRateMmYear/);
  assert.match(source, /drawDiagnosticOverlays/);
  assert.match(source, /handleGenerationProgress/);
});

'''
if anchor not in s:
    raise SystemExit("WG5 browser test insertion anchor missing")
s = s.replace(anchor, new_test, 1)
p.write_text(s)

# Pages/direct page must remain byte-identical after layout change.
# No pages.test change required.

print("Applied Planet Lab observability / overlay / seasonal precipitation patch")
