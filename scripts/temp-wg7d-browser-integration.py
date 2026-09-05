from pathlib import Path
import re


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


# -----------------------------------------------------------------------------
# Engine + protocol versions.
# -----------------------------------------------------------------------------
p = 'rust/interlink-worldgen/src/lib.rs'
s = read(p)
s = replace_once(s, 'pub const WORLDGEN_ENGINE_VERSION: u32 = 10;', 'pub const WORLDGEN_ENGINE_VERSION: u32 = 11;', 'engine version')
write(p, s)

p = 'rust/interlink-worldgen-wasm/src/lib.rs'
s = read(p)
s = replace_once(s, 'pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 17;', 'pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 18;', 'wasm protocol')
write(p, s)

# WASM bridge tests carry explicit engine-version acceptance.
for p in [
    'rust/interlink-worldgen-wasm/tests/climate_bridge.rs',
    'rust/interlink-worldgen-wasm/tests/drainage_bridge.rs',
    'rust/interlink-worldgen-wasm/tests/topography_bridge.rs',
]:
    s = read(p)
    if 'generator_version(), 10' in s:
        s = s.replace('generator_version(), 10', 'generator_version(), 11')
    write(p, s)

# -----------------------------------------------------------------------------
# Memory-conscious cumulative WASM bridge.
# -----------------------------------------------------------------------------
p = 'rust/interlink-worldgen-wasm/src/climate_bridge.rs'
s = read(p)
s = replace_once(
    s,
    '    generate_initial_topography, generate_lakes_closed_basins, generate_lithosphere,\n    generate_post_erosion_hydrology, generate_runoff_discharge, generate_seasonal_hydrology,\n',
    '    generate_initial_topography, generate_lake_sediment_infill, generate_lakes_closed_basins,\n    generate_lithosphere, generate_post_erosion_hydrology, generate_runoff_discharge,\n    generate_seasonal_hydrology,\n',
    'bridge generator imports',
)
s = replace_once(
    s,
    '    InheritedBoundarySet, InheritedPhysicalState, LakeRequest, LithosphereRequest,\n    PlanetPhysicalParameters, PostErosionHydrologyRequest, PostErosionHydrologyState,\n    RunoffRequest, SeasonalHydrologyRequest, TectonicsRequest, TerrainEvolutionRequest,\n    TerrainEvolutionState, TopographyRequest, TopographyState, WORLDGEN_ENGINE_VERSION,\n',
    '    InheritedBoundarySet, InheritedPhysicalState, LakeRequest, LakeSedimentInfillRequest,\n    LakeSedimentInfillState, LithosphereRequest, PlanetPhysicalParameters,\n    PostErosionHydrologyMetrics, PostErosionHydrologyRequest, PostErosionHydrologyState,\n    RunoffRequest, SeasonalHydrologyRequest, StageIdentity, TectonicsRequest,\n    TerrainEvolutionRequest, TerrainEvolutionState, TopographyRequest, TopographyState,\n    WORLDGEN_ENGINE_VERSION,\n',
    'bridge type imports',
)
s = replace_once(s, 'const GENERATION_STAGE_COUNT: u32 = 17;', 'const GENERATION_STAGE_COUNT: u32 = 18;', 'generation stage count')

marker = '#[wasm_bindgen]\npub struct WasmWorldgenClimate {'
lightweight = '''#[derive(Clone, Debug)]
struct ReconciliationDiagnostics {
    stage: StageIdentity,
    metrics: PostErosionHydrologyMetrics,
    lake_kind_changed_mask: Vec<u8>,
    lake_depth_delta_m: Vec<f32>,
    annual_realized_discharge_delta_m3_s: Vec<f32>,
    flow_regime_changed_mask: Vec<u8>,
    flow_presence_delta: Vec<f32>,
}

#[wasm_bindgen]
pub struct WasmWorldgenClimate {'''
s = replace_once(s, marker, lightweight, 'lightweight reconciliation diagnostics')
s = replace_once(
    s,
    '    evolution: TerrainEvolutionState,\n    reconciliation: PostErosionHydrologyState,\n    planet: PlanetPhysicalParameters,\n',
    '    evolution: TerrainEvolutionState,\n    reconciliation: ReconciliationDiagnostics,\n    infill: LakeSedimentInfillState,\n    planet: PlanetPhysicalParameters,\n',
    'bridge state fields',
)

old = '''        report_generation_progress(progress, "post-erosion-hydrology", 15, 1, 1);
        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;

        Ok(Self {'''
new = '''        report_generation_progress(progress, "post-erosion-hydrology", 15, 1, 1);

        report_generation_progress(progress, "lake-sediment-infill", 16, 0, 1);
        let infill = generate_lake_sediment_infill(
            &fine_topology,
            &terrain,
            &climate,
            &diagnostics,
            &drainage,
            &lakes,
            &erosion,
            &evolution,
            &reconciliation,
            planet,
            &LakeSedimentInfillRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "lake-sediment-infill", 16, 1, 1);

        // WG-7D owns the final drainage/runoff/lake/seasonal state. Retain only the compact
        // WG-7C ancestry/change diagnostics so an L7 browser result does not keep two complete
        // seasonal hydrology states alive at once.
        let PostErosionHydrologyState {
            stage,
            metrics,
            lake_kind_changed_mask,
            lake_depth_delta_m,
            annual_realized_discharge_delta_m3_s,
            flow_regime_changed_mask,
            flow_presence_delta,
            ..
        } = reconciliation;
        let reconciliation = ReconciliationDiagnostics {
            stage,
            metrics,
            lake_kind_changed_mask,
            lake_depth_delta_m,
            annual_realized_discharge_delta_m3_s,
            flow_regime_changed_mask,
            flow_presence_delta,
        };
        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;

        Ok(Self {'''
s = replace_once(s, old, new, 'WG-7D constructor integration')
s = replace_once(
    s,
    '            evolution,\n            reconciliation,\n            planet,\n',
    '            evolution,\n            reconciliation,\n            infill,\n            planet,\n',
    'WG-7D constructor state assignment',
)

# Canonical public hydrology now means the final WG-7D state. WG-7B and WG-7C remain exposed
# through their explicitly named diagnostic blocks/hashes.
def replace_in_section(text: str, start: str, end: str, old: str, new: str, label: str) -> str:
    a = text.index(start)
    b = text.index(end, a)
    section = text[a:b]
    if old not in section:
        raise RuntimeError(f'{label}: no target in section')
    section = section.replace(old, new)
    return text[:a] + section + text[b:]

s = replace_in_section(s, '    pub fn drainage_stage_id', '    pub fn runoff_stage_id', 'self.evolution.post_erosion_drainage', 'self.infill.post_infill_drainage', 'final drainage getters')
s = replace_in_section(s, '    pub fn runoff_stage_id', '    pub fn lake_stage_id', 'self.reconciliation.reconciled_runoff', 'self.infill.reconciled_runoff', 'final runoff getters')
s = replace_in_section(s, '    pub fn lake_stage_id', '    pub fn seasonal_stage_id', 'self.reconciliation.reconciled_lakes', 'self.infill.reconciled_lakes', 'final lake getters')
s = replace_in_section(s, '    pub fn seasonal_stage_id', '    pub fn erosion_stage_id', 'self.reconciliation.reconciled_seasonal', 'self.infill.reconciled_seasonal', 'final seasonal getters')

# Append WG-7D stage, ancestry, conservation, and public terrain getters to the WASM surface.
insert_at = s.rfind('\n}')
if insert_at < 0:
    raise RuntimeError('could not locate climate bridge impl terminator')
getters = r'''

    pub fn infill_stage_id(&self) -> String {
        self.infill.stage.id.to_owned()
    }
    pub fn infill_stage_version(&self) -> u32 {
        self.infill.stage.version
    }
    pub fn infill_stage_seed_hex(&self) -> String {
        format!("{:016x}", self.infill.stage.derived_seed)
    }
    pub fn lake_sediment_infill_hash_hex(&self) -> String {
        self.infill.metrics.lake_sediment_infill_hash_hex()
    }
    pub fn infill_parameter_hash_hex(&self) -> String {
        self.infill.metrics.infill_parameter_hash_hex()
    }
    pub fn infill_geomorphic_duration_years(&self) -> f64 {
        self.infill.metrics.geomorphic_duration_years
    }
    pub fn infill_historical_lake_trap_count(&self) -> u32 {
        self.infill.metrics.historical_lake_trap_count
    }
    pub fn infill_filled_depression_count(&self) -> u32 {
        self.infill.metrics.filled_depression_count
    }
    pub fn infill_filled_sample_count(&self) -> u32 {
        self.infill.metrics.filled_sample_count
    }
    pub fn infill_capacity_limited_depression_count(&self) -> u32 {
        self.infill.metrics.capacity_limited_depression_count
    }
    pub fn infill_maximum_fill_depth_m(&self) -> f64 {
        self.infill.metrics.maximum_fill_depth_m
    }
    pub fn infill_total_historical_lake_delivery_kg_s(&self) -> f64 {
        self.infill.metrics.total_historical_lake_delivery_kg_s
    }
    pub fn infill_total_applied_lake_fill_equivalent_kg_s(&self) -> f64 {
        self.infill.metrics.total_applied_lake_fill_equivalent_kg_s
    }
    pub fn infill_total_unapplied_lake_sediment_kg_s(&self) -> f64 {
        self.infill.metrics.total_unapplied_lake_sediment_kg_s
    }
    pub fn infill_total_applied_lake_fill_volume_m3(&self) -> f64 {
        self.infill.metrics.total_applied_lake_fill_volume_m3
    }
    pub fn infill_sediment_conservation_relative_error(&self) -> f64 {
        self.infill.metrics.sediment_conservation_relative_error
    }
    pub fn infill_pre_infill_lake_count(&self) -> u32 {
        self.infill.metrics.pre_infill_lake_count
    }
    pub fn infill_post_infill_lake_count(&self) -> u32 {
        self.infill.metrics.post_infill_lake_count
    }
    pub fn infill_post_infill_runoff_conservation_relative_error(&self) -> f64 {
        self.infill.metrics.post_infill_runoff_conservation_relative_error
    }
    pub fn infill_post_infill_lake_water_balance_relative_error(&self) -> f64 {
        self.infill.metrics.post_infill_lake_water_balance_relative_error
    }
    pub fn infill_post_infill_seasonal_routing_relative_error(&self) -> f64 {
        self.infill.metrics.post_infill_seasonal_routing_relative_error
    }
    pub fn infill_post_infill_seasonal_water_balance_relative_error(&self) -> f64 {
        self.infill.metrics.post_infill_seasonal_water_balance_relative_error
    }
    pub fn infill_topography_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.topography_hash)
    }
    pub fn infill_climate_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.climate_hash)
    }
    pub fn infill_pre_erosion_drainage_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.pre_erosion_drainage_hash)
    }
    pub fn infill_pre_erosion_lake_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.pre_erosion_lake_hash)
    }
    pub fn infill_fluvial_erosion_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.fluvial_erosion_hash)
    }
    pub fn infill_terrain_evolution_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.terrain_evolution_hash)
    }
    pub fn infill_post_erosion_hydrology_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.post_erosion_hydrology_hash)
    }
    pub fn infill_input_evolved_surface_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.input_evolved_surface_hash)
    }
    pub fn infill_pre_infill_drainage_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.pre_infill_drainage_hash)
    }
    pub fn infill_pre_infill_runoff_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.pre_infill_runoff_hash)
    }
    pub fn infill_pre_infill_lake_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.pre_infill_lake_hash)
    }
    pub fn infill_pre_infill_seasonal_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.pre_infill_seasonal_hash)
    }
    pub fn infill_post_infill_surface_hash_hex(&self) -> String {
        self.infill.metrics.post_infill_surface_hash_hex()
    }
    pub fn infill_post_infill_drainage_hash_hex(&self) -> String {
        self.infill.metrics.post_infill_drainage_hash_hex()
    }
    pub fn infill_post_infill_runoff_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.post_infill_runoff_hash)
    }
    pub fn infill_post_infill_lake_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.post_infill_lake_hash)
    }
    pub fn infill_post_infill_seasonal_hash_hex(&self) -> String {
        format!("{:016x}", self.infill.metrics.post_infill_seasonal_hash)
    }
    pub fn post_infill_solid_elevation_m(&self) -> Vec<f32> {
        self.infill.post_infill_solid_elevation_m.clone()
    }
    pub fn lake_fill_depth_m(&self) -> Vec<f32> {
        self.infill.lake_fill_depth_m.clone()
    }
'''
s = s[:insert_at] + getters + s[insert_at:]
write(p, s)

# -----------------------------------------------------------------------------
# Browser protocol v18.
# -----------------------------------------------------------------------------
p = 'src/worldgen/protocol.ts'
s = read(p)
s = replace_once(s, 'export const WORLDGEN_PROTOCOL_VERSION = 17;', 'export const WORLDGEN_PROTOCOL_VERSION = 18;', 'browser protocol')
metrics_marker = '''export interface WorldgenClimateResult {'''
infill_metrics = '''export interface WorldgenLakeSedimentInfillMetrics {
  sampleCount: number;
  geomorphicDurationYears: number;
  historicalLakeTrapCount: number;
  filledDepressionCount: number;
  filledSampleCount: number;
  capacityLimitedDepressionCount: number;
  maximumFillDepthM: number;
  totalHistoricalLakeDeliveryKgS: number;
  totalAppliedLakeFillEquivalentKgS: number;
  totalUnappliedLakeSedimentKgS: number;
  totalAppliedLakeFillVolumeM3: number;
  sedimentConservationRelativeError: number;
  preInfillLakeCount: number;
  postInfillLakeCount: number;
  postInfillRunoffConservationRelativeError: number;
  postInfillLakeWaterBalanceRelativeError: number;
  postInfillSeasonalRoutingRelativeError: number;
  postInfillSeasonalWaterBalanceRelativeError: number;
  infillParameterHash: string;
  topographyHash: string;
  climateHash: string;
  preErosionDrainageHash: string;
  preErosionLakeHash: string;
  fluvialErosionHash: string;
  terrainEvolutionHash: string;
  postErosionHydrologyHash: string;
  inputEvolvedSurfaceHash: string;
  preInfillDrainageHash: string;
  preInfillRunoffHash: string;
  preInfillLakeHash: string;
  preInfillSeasonalHash: string;
  postInfillSurfaceHash: string;
  postInfillDrainageHash: string;
  postInfillRunoffHash: string;
  postInfillLakeHash: string;
  postInfillSeasonalHash: string;
  lakeSedimentInfillHash: string;
}

export interface WorldgenClimateResult {'''
s = replace_once(s, metrics_marker, infill_metrics, 'protocol infill metrics interface')
s = replace_once(
    s,
    '''  flowRegimeChangedMask: Uint8Array;
  flowPresenceDelta: Float32Array;
}
''',
    '''  flowRegimeChangedMask: Uint8Array;
  flowPresenceDelta: Float32Array;
  infillStage: WorldgenStageMetadata;
  infillMetrics: WorldgenLakeSedimentInfillMetrics;
  postInfillSolidElevationM: Float32Array;
  lakeFillDepthM: Float32Array;
}
''',
    'protocol WG-7D result fields',
)
write(p, s)

# -----------------------------------------------------------------------------
# Worker interface + packaging.
# -----------------------------------------------------------------------------
p = 'src/worldgen/worldgenWorker.ts'
s = read(p)
s = replace_once(
    s,
    '''  reconciliation_lake_kind_changed_mask(): Uint8Array; reconciliation_lake_depth_delta_m(): Float32Array; reconciliation_annual_realized_discharge_delta_m3_s(): Float32Array; reconciliation_flow_regime_changed_mask(): Uint8Array; reconciliation_flow_presence_delta(): Float32Array;
  free(): void;
}''',
    '''  reconciliation_lake_kind_changed_mask(): Uint8Array; reconciliation_lake_depth_delta_m(): Float32Array; reconciliation_annual_realized_discharge_delta_m3_s(): Float32Array; reconciliation_flow_regime_changed_mask(): Uint8Array; reconciliation_flow_presence_delta(): Float32Array;
  infill_stage_id(): string; infill_stage_version(): number; infill_stage_seed_hex(): string; lake_sediment_infill_hash_hex(): string; infill_parameter_hash_hex(): string;
  infill_geomorphic_duration_years(): number; infill_historical_lake_trap_count(): number; infill_filled_depression_count(): number; infill_filled_sample_count(): number; infill_capacity_limited_depression_count(): number; infill_maximum_fill_depth_m(): number;
  infill_total_historical_lake_delivery_kg_s(): number; infill_total_applied_lake_fill_equivalent_kg_s(): number; infill_total_unapplied_lake_sediment_kg_s(): number; infill_total_applied_lake_fill_volume_m3(): number; infill_sediment_conservation_relative_error(): number;
  infill_pre_infill_lake_count(): number; infill_post_infill_lake_count(): number; infill_post_infill_runoff_conservation_relative_error(): number; infill_post_infill_lake_water_balance_relative_error(): number; infill_post_infill_seasonal_routing_relative_error(): number; infill_post_infill_seasonal_water_balance_relative_error(): number;
  infill_topography_hash_hex(): string; infill_climate_hash_hex(): string; infill_pre_erosion_drainage_hash_hex(): string; infill_pre_erosion_lake_hash_hex(): string; infill_fluvial_erosion_hash_hex(): string; infill_terrain_evolution_hash_hex(): string; infill_post_erosion_hydrology_hash_hex(): string; infill_input_evolved_surface_hash_hex(): string;
  infill_pre_infill_drainage_hash_hex(): string; infill_pre_infill_runoff_hash_hex(): string; infill_pre_infill_lake_hash_hex(): string; infill_pre_infill_seasonal_hash_hex(): string; infill_post_infill_surface_hash_hex(): string; infill_post_infill_drainage_hash_hex(): string; infill_post_infill_runoff_hash_hex(): string; infill_post_infill_lake_hash_hex(): string; infill_post_infill_seasonal_hash_hex(): string;
  post_infill_solid_elevation_m(): Float32Array; lake_fill_depth_m(): Float32Array;
  free(): void;
}''',
    'worker wasm interface',
)
s = s.replace("progress('packaging', 16, 17, 0, 1);", "progress('packaging', 17, 18, 0, 1);")
s = s.replace("progress('packaging', 16, 17, 1, 1);", "progress('packaging', 17, 18, 1, 1);")
s = replace_once(
    s,
    '''    const lakeKindChangedMask = output.reconciliation_lake_kind_changed_mask(); const lakeDepthDeltaM = output.reconciliation_lake_depth_delta_m(); const annualRealizedDischargeDeltaM3S = output.reconciliation_annual_realized_discharge_delta_m3_s(); const flowRegimeChangedMask = output.reconciliation_flow_regime_changed_mask(); const flowPresenceDelta = output.reconciliation_flow_presence_delta();
    const result: WorldgenClimateResult = {''',
    '''    const lakeKindChangedMask = output.reconciliation_lake_kind_changed_mask(); const lakeDepthDeltaM = output.reconciliation_lake_depth_delta_m(); const annualRealizedDischargeDeltaM3S = output.reconciliation_annual_realized_discharge_delta_m3_s(); const flowRegimeChangedMask = output.reconciliation_flow_regime_changed_mask(); const flowPresenceDelta = output.reconciliation_flow_presence_delta();
    const postInfillSolidElevationM = output.post_infill_solid_elevation_m(); const lakeFillDepthM = output.lake_fill_depth_m();
    const result: WorldgenClimateResult = {''',
    'worker WG-7D vector extraction',
)
s = replace_once(
    s,
    '''      lakeKindChangedMask, lakeDepthDeltaM, annualRealizedDischargeDeltaM3S, flowRegimeChangedMask, flowPresenceDelta,
    };''',
    '''      lakeKindChangedMask, lakeDepthDeltaM, annualRealizedDischargeDeltaM3S, flowRegimeChangedMask, flowPresenceDelta,
      infillStage: { id: output.infill_stage_id(), version: output.infill_stage_version(), stageSeed: output.infill_stage_seed_hex(), durationMs: 0 },
      infillMetrics: { sampleCount: output.fine_sample_count(), geomorphicDurationYears: output.infill_geomorphic_duration_years(), historicalLakeTrapCount: output.infill_historical_lake_trap_count(), filledDepressionCount: output.infill_filled_depression_count(), filledSampleCount: output.infill_filled_sample_count(), capacityLimitedDepressionCount: output.infill_capacity_limited_depression_count(), maximumFillDepthM: output.infill_maximum_fill_depth_m(), totalHistoricalLakeDeliveryKgS: output.infill_total_historical_lake_delivery_kg_s(), totalAppliedLakeFillEquivalentKgS: output.infill_total_applied_lake_fill_equivalent_kg_s(), totalUnappliedLakeSedimentKgS: output.infill_total_unapplied_lake_sediment_kg_s(), totalAppliedLakeFillVolumeM3: output.infill_total_applied_lake_fill_volume_m3(), sedimentConservationRelativeError: output.infill_sediment_conservation_relative_error(), preInfillLakeCount: output.infill_pre_infill_lake_count(), postInfillLakeCount: output.infill_post_infill_lake_count(), postInfillRunoffConservationRelativeError: output.infill_post_infill_runoff_conservation_relative_error(), postInfillLakeWaterBalanceRelativeError: output.infill_post_infill_lake_water_balance_relative_error(), postInfillSeasonalRoutingRelativeError: output.infill_post_infill_seasonal_routing_relative_error(), postInfillSeasonalWaterBalanceRelativeError: output.infill_post_infill_seasonal_water_balance_relative_error(), infillParameterHash: output.infill_parameter_hash_hex(), topographyHash: output.infill_topography_hash_hex(), climateHash: output.infill_climate_hash_hex(), preErosionDrainageHash: output.infill_pre_erosion_drainage_hash_hex(), preErosionLakeHash: output.infill_pre_erosion_lake_hash_hex(), fluvialErosionHash: output.infill_fluvial_erosion_hash_hex(), terrainEvolutionHash: output.infill_terrain_evolution_hash_hex(), postErosionHydrologyHash: output.infill_post_erosion_hydrology_hash_hex(), inputEvolvedSurfaceHash: output.infill_input_evolved_surface_hash_hex(), preInfillDrainageHash: output.infill_pre_infill_drainage_hash_hex(), preInfillRunoffHash: output.infill_pre_infill_runoff_hash_hex(), preInfillLakeHash: output.infill_pre_infill_lake_hash_hex(), preInfillSeasonalHash: output.infill_pre_infill_seasonal_hash_hex(), postInfillSurfaceHash: output.infill_post_infill_surface_hash_hex(), postInfillDrainageHash: output.infill_post_infill_drainage_hash_hex(), postInfillRunoffHash: output.infill_post_infill_runoff_hash_hex(), postInfillLakeHash: output.infill_post_infill_lake_hash_hex(), postInfillSeasonalHash: output.infill_post_infill_seasonal_hash_hex(), lakeSedimentInfillHash: output.lake_sediment_infill_hash_hex() },
      postInfillSolidElevationM, lakeFillDepthM,
    };''',
    'worker WG-7D result packaging',
)
s = replace_once(
    s,
    'result.flowRegimeChangedMask.buffer, result.flowPresenceDelta.buffer]); return;',
    'result.flowRegimeChangedMask.buffer, result.flowPresenceDelta.buffer, result.postInfillSolidElevationM.buffer, result.lakeFillDepthM.buffer]); return;',
    'worker WG-7D transfer buffers',
)
write(p, s)

# -----------------------------------------------------------------------------
# Lab: final physical surface + dedicated WG-7D diagnostics and ancestry checks.
# -----------------------------------------------------------------------------
p = 'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts'
s = read(p)
s = replace_once(
    s,
    "const RECONCILIATION_MODES = new Set(['reconciliation-lake-depth-delta', 'reconciliation-lake-change', 'reconciliation-realized-discharge-delta', 'reconciliation-flow-presence-delta', 'reconciliation-flow-regime-change']);\n",
    "const INFILL_MODES = new Set(['infill-solid-elevation', 'infill-fill-depth']);\nconst RECONCILIATION_MODES = new Set(['reconciliation-lake-depth-delta', 'reconciliation-lake-change', 'reconciliation-realized-discharge-delta', 'reconciliation-flow-presence-delta', 'reconciliation-flow-regime-change']);\n",
    'Lab WG-7D mode set',
)
s = replace_once(
    s,
    "    case 'evolution-solid-elevation': return { values: result.evolvedSolidElevationM, minimum: -12_000, maximum: 8_000, lowHue: 225, highHue: 25 };\n",
    "    case 'infill-solid-elevation': return { values: result.postInfillSolidElevationM, minimum: -12_000, maximum: 8_000, lowHue: 225, highHue: 25 };\n    case 'infill-fill-depth': return { values: result.lakeFillDepthM, minimum: 0, maximum: Math.max(0.01, result.infillMetrics.maximumFillDepthM), lowHue: 205, highHue: 35 };\n    case 'evolution-solid-elevation': return { values: result.evolvedSolidElevationM, minimum: -12_000, maximum: 8_000, lowHue: 225, highHue: 25 };\n",
    'Lab WG-7D scalar fields',
)
# Final physical-world coloring and contour overlay use the post-infill surface. Fixed WG-4 ocean
# mask/coastline is intentional in WG-7D v1.
s = s.replace('const elevation = result.elevationAboveSeaLevelM[sample]! + result.terrainDeltaM[sample]!;', 'const elevation = result.postInfillSolidElevationM[sample]! - result.metrics.seaLevelM;')
s = s.replace('const evolvedA = ea + result.terrainDeltaM[a]!;\n      const evolvedB = eb + result.terrainDeltaM[b]!;', 'const evolvedA = result.postInfillSolidElevationM[a]! - result.metrics.seaLevelM;\n      const evolvedB = result.postInfillSolidElevationM[b]! - result.metrics.seaLevelM;')

# Generation stage label.
s = replace_once(
    s,
    "  'post-erosion-hydrology': 'Post-erosion hydrology reconciliation',\n  packaging: 'Packaging / transfer',",
    "  'post-erosion-hydrology': 'Post-erosion hydrology reconciliation',\n  'lake-sediment-infill': 'Lake sediment infill / final hydrology',\n  packaging: 'Packaging / transfer',",
    'Lab generation labels',
)

# WG-7C is now explicitly pre-infill; canonical hydrology is WG-7D final.
old_checks = '''    if (loaded.reconciliationMetrics.postErosionDrainageHash !== loaded.drainageMetrics.drainageHash) throw new Error('WG-7C final drainage identity mismatch.');
    if (loaded.reconciliationMetrics.reconciledRunoffHash !== loaded.runoffMetrics.runoffHash) throw new Error('WG-7C final runoff identity mismatch.');
    if (loaded.reconciliationMetrics.reconciledLakeHash !== loaded.lakeMetrics.lakeHash) throw new Error('WG-7C final lake identity mismatch.');
    if (loaded.reconciliationMetrics.reconciledSeasonalHash !== loaded.seasonalMetrics.seasonalHydrologyHash) throw new Error('WG-7C final seasonal identity mismatch.');
'''
new_checks = '''    if (loaded.reconciliationMetrics.postErosionDrainageHash !== loaded.infillMetrics.preInfillDrainageHash) throw new Error('WG-7C drainage identity does not match WG-7D pre-infill ancestry.');
    if (loaded.reconciliationMetrics.reconciledRunoffHash !== loaded.infillMetrics.preInfillRunoffHash) throw new Error('WG-7C runoff identity does not match WG-7D pre-infill ancestry.');
    if (loaded.reconciliationMetrics.reconciledLakeHash !== loaded.infillMetrics.preInfillLakeHash) throw new Error('WG-7C lake identity does not match WG-7D pre-infill ancestry.');
    if (loaded.reconciliationMetrics.reconciledSeasonalHash !== loaded.infillMetrics.preInfillSeasonalHash) throw new Error('WG-7C seasonal identity does not match WG-7D pre-infill ancestry.');
    if (loaded.infillMetrics.topographyHash !== loaded.metrics.topographyHash || loaded.infillMetrics.climateHash !== loaded.metrics.climateHash) throw new Error('WG-7D immutable WG-4/WG-5 ancestry mismatch.');
    if (loaded.infillMetrics.preErosionDrainageHash !== loaded.reconciliationMetrics.preErosionDrainageHash || loaded.infillMetrics.preErosionLakeHash !== loaded.reconciliationMetrics.preErosionLakeHash) throw new Error('WG-7D pre-erosion hydrology ancestry mismatch.');
    if (loaded.infillMetrics.fluvialErosionHash !== loaded.erosionMetrics.fluvialErosionHash || loaded.infillMetrics.terrainEvolutionHash !== loaded.evolutionMetrics.terrainEvolutionHash) throw new Error('WG-7D WG-7A/WG-7B geomorphic ancestry mismatch.');
    if (loaded.infillMetrics.postErosionHydrologyHash !== loaded.reconciliationMetrics.postErosionHydrologyHash || loaded.infillMetrics.inputEvolvedSurfaceHash !== loaded.evolutionMetrics.evolvedSurfaceHash) throw new Error('WG-7D WG-7C/evolved-surface ancestry mismatch.');
    if (loaded.infillMetrics.postInfillDrainageHash !== loaded.drainageMetrics.drainageHash) throw new Error('WG-7D final drainage identity mismatch.');
    if (loaded.infillMetrics.postInfillRunoffHash !== loaded.runoffMetrics.runoffHash) throw new Error('WG-7D final runoff identity mismatch.');
    if (loaded.infillMetrics.postInfillLakeHash !== loaded.lakeMetrics.lakeHash) throw new Error('WG-7D final lake identity mismatch.');
    if (loaded.infillMetrics.postInfillSeasonalHash !== loaded.seasonalMetrics.seasonalHydrologyHash) throw new Error('WG-7D final seasonal identity mismatch.');
'''
s = replace_once(s, old_checks, new_checks, 'Lab ancestry checks')

# Include WG-7D in metrics after WG-7C.
old_metric_tail = "  metric(metrics, 'WG-7C reconciliation hash', result.reconciliationMetrics.postErosionHydrologyHash);\n}"
new_metric_tail = """  metric(metrics, 'WG-7C reconciliation hash', result.reconciliationMetrics.postErosionHydrologyHash);
  metric(metrics, 'WG-7D / stage', `v${result.engineVersion} · ${result.infillStage.id}@${result.infillStage.version}`);
  metric(metrics, 'WG-7D infill horizon', `${result.infillMetrics.geomorphicDurationYears.toFixed(0)} y · ${result.infillMetrics.historicalLakeTrapCount.toLocaleString()} historical traps`);
  metric(metrics, 'WG-7D filled depressions / samples', `${result.infillMetrics.filledDepressionCount.toLocaleString()} / ${result.infillMetrics.filledSampleCount.toLocaleString()} · ${result.infillMetrics.capacityLimitedDepressionCount.toLocaleString()} capacity-limited`);
  metric(metrics, 'WG-7D max fill', `${result.infillMetrics.maximumFillDepthM.toFixed(3)} m`);
  metric(metrics, 'WG-7D sediment delivery', `${result.infillMetrics.totalHistoricalLakeDeliveryKgS.toFixed(1)} kg/s · applied ${result.infillMetrics.totalAppliedLakeFillEquivalentKgS.toFixed(1)} · unapplied ${result.infillMetrics.totalUnappliedLakeSedimentKgS.toFixed(1)}`);
  metric(metrics, 'WG-7D lake count', `${result.infillMetrics.preInfillLakeCount.toLocaleString()} → ${result.infillMetrics.postInfillLakeCount.toLocaleString()}`);
  metric(metrics, 'WG-7D sediment closure', result.infillMetrics.sedimentConservationRelativeError.toExponential(2));
  metric(metrics, 'WG-7D final hydro closure', `runoff ${result.infillMetrics.postInfillRunoffConservationRelativeError.toExponential(2)} · lake ${result.infillMetrics.postInfillLakeWaterBalanceRelativeError.toExponential(2)} · seasonal ${result.infillMetrics.postInfillSeasonalWaterBalanceRelativeError.toExponential(2)}`);
  metric(metrics, 'WG-7D surface / drainage hash', `${result.infillMetrics.postInfillSurfaceHash} / ${result.infillMetrics.postInfillDrainageHash}`);
  metric(metrics, 'WG-7D infill hash', result.infillMetrics.lakeSedimentInfillHash);
}"""
s = replace_once(s, old_metric_tail, new_metric_tail, 'Lab WG-7D metrics')
s = s.replace("status.textContent = 'Generating one physical planet through WG-7B bounded terrain evolution in Rust/WASM…';", "status.textContent = 'Generating one physical planet through WG-7D lake sediment infill in Rust/WASM…';")
s = s.replace("status.textContent = `Planet ready through WG-7C:", "status.textContent = `Planet ready through WG-7D:")
s = s.replace("${loaded.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} receivers changed after evolution`;", "${loaded.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} receivers changed after evolution · ${loaded.infillMetrics.filledDepressionCount.toLocaleString()} lake basins infilled`;")
write(p, s)

# -----------------------------------------------------------------------------
# HTML labels and WG-7D diagnostic selector.
# -----------------------------------------------------------------------------
p = 'index.html'
s = read(p)
s = replace_once(
    s,
    '''          <optgroup label="Hydrology · Post-erosion reconciliation (WG-7C)">''',
    '''          <optgroup label="Geomorphology · Lake sediment infill / final surface (WG-7D)">
            <option value="infill-solid-elevation">Final post-infill solid elevation</option>
            <option value="infill-fill-depth">Lake sediment fill depth</option>
          </optgroup>
          <optgroup label="Hydrology · Post-erosion reconciliation (WG-7C)">''',
    'HTML WG-7D diagnostic group',
)
s = s.replace('data-label="Evolved topographic contours"> Evolved topographic contours', 'data-label="Final topographic contours"> Final topographic contours')
s = s.replace('<strong>Current physical frontier: WG-7C</strong>', '<strong>Current physical frontier: WG-7D</strong>')
s = replace_once(
    s,
    '<p>One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, WG-7A fluvial erosion/sediment diagnostics, WG-7B bounded terrain evolution, and WG-7C post-erosion hydrology reconciliation in one cumulative Rust/WASM result.</p>',
    '<p>One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, WG-7A fluvial erosion/sediment diagnostics, WG-7B bounded terrain evolution, WG-7C post-erosion hydrology reconciliation, and WG-7D bounded lake-sediment infill in one cumulative Rust/WASM result.</p>',
    'HTML frontier pipeline',
)
s = replace_once(
    s,
    '<p>The WG-4 ocean mask/coastline remains fixed. WG-7B rebuilds drainage once on the evolved surface; WG-7C then rebinds the exact accepted local runoff to that DAG, recomputes lake equilibrium against evolved elevation, and reruns seasonal realized flow/lake storage without rerunning WG-5 climate or mutating terrain again. The browser retains the reconciled drainage/runoff/lake/seasonal state as the final hydrology while pre-erosion WG-6 ancestry survives as compact WG-7C hashes and change diagnostics. Coastline migration, sediment-driven lake infill/capacity change, delta/coastal construction, hillslope transport, glaciers, weathering/soil/lithology, resources, Regions, Features, and gameplay integration remain downstream.</p>',
    '<p>The WG-4 ocean mask/coastline remains fixed. WG-7B rebuilds drainage once on the evolved surface; WG-7C reconciles runoff, lakes, and seasonal flow to that terrain; WG-7D then converts already-conserved historical lake sediment delivery into bounded basin infill, rebuilds drainage once more on the distinct post-infill surface, and reconciles runoff/lakes/seasonal hydrology again without rerunning WG-5 climate. The browser exposes WG-7D hydrology as the final state while retaining WG-7C as compact ancestry/change diagnostics instead of keeping two complete seasonal states in WASM memory. Coastline migration, deltas/offshore construction, floodplains, hillslope transport, glaciers, weathering/soil/lithology, resources, Regions, Features, and gameplay integration remain downstream.</p>',
    'HTML WG-7D semantics',
)
write(p, s)

# -----------------------------------------------------------------------------
# Browser regression coverage for v18 + final-state wiring.
# -----------------------------------------------------------------------------
p = 'tests/worldgenCompositeViews.test.ts'
s = read(p)
addition = '''

test('WG-7D final physical world uses post-infill terrain and final hydrology ancestry', () => {
  const protocol = readFileSync(join(root, 'src/worldgen/protocol.ts'), 'utf8');
  const worker = readFileSync(join(root, 'src/worldgen/worldgenWorker.ts'), 'utf8');
  const lab = readFileSync(join(root, 'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts'), 'utf8');
  assert.match(protocol, /WORLDGEN_PROTOCOL_VERSION = 18/);
  assert.match(protocol, /infillMetrics: WorldgenLakeSedimentInfillMetrics/);
  assert.match(protocol, /postInfillSolidElevationM: Float32Array/);
  assert.match(worker, /infill_post_infill_drainage_hash_hex/);
  assert.match(worker, /postInfillSolidElevationM/);
  assert.match(lab, /result\.postInfillSolidElevationM\[sample\]/);
  assert.match(lab, /WG-7D final drainage identity mismatch/);
  assert.match(lab, /lake-sediment-infill/);
});
'''
if 'WG-7D final physical world uses post-infill terrain' not in s:
    s += addition
write(p, s)

# Update explicit browser protocol assertions from v17 to v18 only where protocol 17 is the
# accepted cumulative worldgen contract. Generic numeric 17s are deliberately untouched.
for p in Path('tests').glob('*.ts'):
    s = p.read_text()
    s = s.replace('WORLDGEN_PROTOCOL_VERSION, 17', 'WORLDGEN_PROTOCOL_VERSION, 18')
    s = s.replace('WORLDGEN_PROTOCOL_VERSION === 17', 'WORLDGEN_PROTOCOL_VERSION === 18')
    s = s.replace('WORLDGEN_PROTOCOL_VERSION).toBe(17)', 'WORLDGEN_PROTOCOL_VERSION).toBe(18)')
    s = s.replace('protocolVersion: 17', 'protocolVersion: 18')
    p.write_text(s)

print('WG-7D browser/WASM integration transforms applied')
