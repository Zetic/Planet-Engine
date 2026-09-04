from pathlib import Path


def once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)


# Rust/WASM bridge.
path = Path("rust/interlink-worldgen-wasm/src/climate_bridge.rs")
text = path.read_text()
text = once(
    text,
    "    generate_drainage_topology, generate_initial_topography, generate_lakes_closed_basins,\n",
    "    generate_drainage_topology, generate_fluvial_erosion_sediment, generate_initial_topography,\n    generate_lakes_closed_basins,\n",
    "bridge erosion generator import",
)
text = once(
    text,
    "    ClimatePhysicalParameters, ClimateRequest, ClimateState, DrainageRequest, DrainageState,\n    GeodesicTopology, GeologyRequest, InheritedBoundarySet, InheritedPhysicalState, LakeRequest,\n",
    "    ClimatePhysicalParameters, ClimateRequest, ClimateState, DrainageRequest, DrainageState,\n    FluvialErosionRequest, FluvialErosionState, GeodesicTopology, GeologyRequest,\n    InheritedBoundarySet, InheritedPhysicalState, LakeRequest,\n",
    "bridge erosion type imports",
)
text = once(text, "const GENERATION_STAGE_COUNT: u32 = 14;", "const GENERATION_STAGE_COUNT: u32 = 15;", "bridge stage count")
text = once(
    text,
    "    seasonal: SeasonalHydrologyState,\n    planet: PlanetPhysicalParameters,\n",
    "    seasonal: SeasonalHydrologyState,\n    erosion: FluvialErosionState,\n    planet: PlanetPhysicalParameters,\n",
    "bridge erosion state field",
)
text = once(
    text,
    "        report_generation_progress(progress, \"seasonal-hydrology\", 12, 1, 1);\n        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;\n",
    "        report_generation_progress(progress, \"seasonal-hydrology\", 12, 1, 1);\n\n        report_generation_progress(progress, \"fluvial-erosion-sediment\", 13, 0, 1);\n        let erosion = generate_fluvial_erosion_sediment(\n            &fine_topology,\n            &inherited,\n            &terrain,\n            &drainage,\n            &lakes,\n            &seasonal,\n            planet,\n            &FluvialErosionRequest::new(seed.as_str()),\n        )\n        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"fluvial-erosion-sediment\", 13, 1, 1);\n        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;\n",
    "bridge erosion generation",
)
text = once(
    text,
    "            lakes,\n            seasonal,\n            planet,\n",
    "            lakes,\n            seasonal,\n            erosion,\n            planet,\n",
    "bridge erosion state construction",
)
bridge_tail = """    pub fn seasonal_phase_lake_volume_m3(&self) -> Vec<f64> {
        self.seasonal.phase_lake_volume_m3.clone()
    }
}"""
bridge_new_tail = """    pub fn seasonal_phase_lake_volume_m3(&self) -> Vec<f64> {
        self.seasonal.phase_lake_volume_m3.clone()
    }

    pub fn erosion_stage_id(&self) -> String {
        self.erosion.stage.id.to_owned()
    }
    pub fn erosion_stage_version(&self) -> u32 {
        self.erosion.stage.version
    }
    pub fn erosion_stage_seed_hex(&self) -> String {
        format!("{:016x}", self.erosion.stage.derived_seed)
    }
    pub fn fluvial_erosion_hash_hex(&self) -> String {
        self.erosion.metrics.fluvial_erosion_hash_hex()
    }
    pub fn erosion_parameter_hash_hex(&self) -> String {
        self.erosion.metrics.erosion_parameter_hash_hex()
    }
    pub fn erosion_inheritance_hash_hex(&self) -> String {
        self.erosion.metrics.inheritance_hash_hex()
    }
    pub fn erosion_topography_hash_hex(&self) -> String {
        self.erosion.metrics.topography_hash_hex()
    }
    pub fn erosion_drainage_hash_hex(&self) -> String {
        self.erosion.metrics.drainage_hash_hex()
    }
    pub fn erosion_lake_hash_hex(&self) -> String {
        self.erosion.metrics.lake_hash_hex()
    }
    pub fn erosion_seasonal_hydrology_hash_hex(&self) -> String {
        self.erosion.metrics.seasonal_hydrology_hash_hex()
    }
    pub fn erosive_sample_count(&self) -> u32 {
        self.erosion.metrics.erosive_sample_count
    }
    pub fn active_lake_trap_count(&self) -> u32 {
        self.erosion.metrics.active_lake_trap_count
    }
    pub fn maximum_effective_discharge_m3_s(&self) -> f64 {
        self.erosion.metrics.maximum_effective_discharge_m3_s
    }
    pub fn maximum_channel_slope(&self) -> f64 {
        self.erosion.metrics.maximum_channel_slope
    }
    pub fn maximum_channel_width_m(&self) -> f64 {
        self.erosion.metrics.maximum_channel_width_m
    }
    pub fn maximum_incision_potential_m_per_year(&self) -> f64 {
        self.erosion.metrics.maximum_incision_potential_m_per_year
    }
    pub fn total_sediment_generated_kg_s(&self) -> f64 {
        self.erosion.metrics.total_sediment_generated_kg_s
    }
    pub fn total_land_deposition_kg_s(&self) -> f64 {
        self.erosion.metrics.total_land_deposition_kg_s
    }
    pub fn total_lake_deposition_kg_s(&self) -> f64 {
        self.erosion.metrics.total_lake_deposition_kg_s
    }
    pub fn total_terminal_ocean_deposition_kg_s(&self) -> f64 {
        self.erosion.metrics.total_terminal_ocean_deposition_kg_s
    }
    pub fn maximum_sediment_load_kg_s(&self) -> f64 {
        self.erosion.metrics.maximum_sediment_load_kg_s
    }
    pub fn sediment_conservation_relative_error(&self) -> f64 {
        self.erosion.metrics.sediment_conservation_relative_error
    }
    pub fn effective_discharge_m3_s(&self) -> Vec<f32> {
        self.erosion.effective_discharge_m3_s.clone()
    }
    pub fn channel_slope(&self) -> Vec<f32> {
        self.erosion.channel_slope.clone()
    }
    pub fn channel_width_m(&self) -> Vec<f32> {
        self.erosion.channel_width_m.clone()
    }
    pub fn erodibility_index(&self) -> Vec<f32> {
        self.erosion.erodibility_index.clone()
    }
    pub fn stream_power_index(&self) -> Vec<f32> {
        self.erosion.stream_power_index.clone()
    }
    pub fn incision_potential_m_per_year(&self) -> Vec<f32> {
        self.erosion.incision_potential_m_per_year.clone()
    }
    pub fn local_sediment_supply_kg_s(&self) -> Vec<f32> {
        self.erosion.local_sediment_supply_kg_s.clone()
    }
    pub fn sediment_transport_capacity_kg_s(&self) -> Vec<f32> {
        self.erosion.sediment_transport_capacity_kg_s.clone()
    }
    pub fn sediment_load_kg_s(&self) -> Vec<f32> {
        self.erosion.sediment_load_kg_s.clone()
    }
    pub fn sediment_deposition_kg_s(&self) -> Vec<f32> {
        self.erosion.sediment_deposition_kg_s.clone()
    }
}"""
text = once(text, bridge_tail, bridge_new_tail, "bridge erosion getters")
path.write_text(text)

# Browser protocol.
path = Path("src/worldgen/protocol.ts")
text = path.read_text()
text = once(text, "export const WORLDGEN_PROTOCOL_VERSION = 14;", "export const WORLDGEN_PROTOCOL_VERSION = 15;", "browser protocol version")
seasonal_interface = """export interface WorldgenSeasonalHydrologyMetrics {
  sampleCount: number;
  orbitalPhaseCount: number;
  activeLakeCount: number;
  dryFlowSampleCount: number;
  intermittentFlowSampleCount: number;
  perennialFlowSampleCount: number;
  maximumPhaseLocalRunoffM3S: number;
  maximumPhasePotentialDischargeM3S: number;
  maximumPhaseRealizedDischargeM3S: number;
  snowmeltRunoffFraction: number;
  annualMeanLocalRunoffM3S: number;
  annualLocalRunoffClosureRelativeError: number;
  annualMeanTerminalPotentialDischargeM3S: number;
  seasonalRoutingConservationRelativeError: number;
  annualMeanTerminalRealizedDischargeM3S: number;
  annualMeanLakePrecipitationM3S: number;
  annualMeanLakeEvaporationM3S: number;
  annualMeanUnreleasedTerminalStorageM3S: number;
  seasonalWaterBalanceRelativeError: number;
  lakeSpinupYears: number;
  finalLakeCycleRelativeChange: number;
  finalLakeSurfaceCycleChangeM: number;
  maximumSeasonalLakeLevelRangeM: number;
  seasonalParameterHash: string;
  climateHash: string;
  drainageHash: string;
  runoffHash: string;
  lakeHash: string;
  seasonalHydrologyHash: string;
}
"""
erosion_interface = seasonal_interface + """
export interface WorldgenFluvialErosionMetrics {
  sampleCount: number;
  orbitalPhaseCount: number;
  erosiveSampleCount: number;
  activeLakeTrapCount: number;
  maximumEffectiveDischargeM3S: number;
  maximumChannelSlope: number;
  maximumChannelWidthM: number;
  maximumIncisionPotentialMPerYear: number;
  totalSedimentGeneratedKgS: number;
  totalLandDepositionKgS: number;
  totalLakeDepositionKgS: number;
  totalTerminalOceanDepositionKgS: number;
  maximumSedimentLoadKgS: number;
  sedimentConservationRelativeError: number;
  erosionParameterHash: string;
  inheritanceHash: string;
  topographyHash: string;
  drainageHash: string;
  lakeHash: string;
  seasonalHydrologyHash: string;
  fluvialErosionHash: string;
}
"""
text = once(text, seasonal_interface, erosion_interface, "erosion protocol metrics")
climate_tail = """  seasonalPhaseLakeSurfaceElevationM: Float32Array;
  seasonalPhaseLakeAreaM2: Float64Array;
  seasonalPhaseLakeVolumeM3: Float64Array;
}
"""
climate_new_tail = """  seasonalPhaseLakeSurfaceElevationM: Float32Array;
  seasonalPhaseLakeAreaM2: Float64Array;
  seasonalPhaseLakeVolumeM3: Float64Array;
  erosionStage: WorldgenStageMetadata;
  erosionMetrics: WorldgenFluvialErosionMetrics;
  effectiveDischargeM3S: Float32Array;
  channelSlope: Float32Array;
  channelWidthM: Float32Array;
  erodibilityIndex: Float32Array;
  streamPowerIndex: Float32Array;
  incisionPotentialMPerYear: Float32Array;
  localSedimentSupplyKgS: Float32Array;
  sedimentTransportCapacityKgS: Float32Array;
  sedimentLoadKgS: Float32Array;
  sedimentDepositionKgS: Float32Array;
}
"""
text = once(text, climate_tail, climate_new_tail, "erosion climate result fields")
path.write_text(text)

# WASM protocol version.
path = Path("rust/interlink-worldgen-wasm/src/lib.rs")
text = path.read_text()
text = once(text, "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 14;", "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 15;", "wasm protocol version")
path.write_text(text)

# Worker bridge and result packaging.
path = Path("src/worldgen/worldgenWorker.ts")
text = path.read_text()
worker_interface_tail = """  seasonal_phase_local_runoff_m3_s(): Float32Array; seasonal_phase_snowmelt_runoff_m3_s(): Float32Array; seasonal_phase_snow_storage_mm(): Float32Array; seasonal_phase_potential_discharge_m3_s(): Float32Array; seasonal_phase_realized_discharge_m3_s(): Float32Array; seasonal_flow_presence_fraction(): Float32Array; seasonal_flow_regime(): Uint8Array; seasonal_phase_lake_surface_elevation_m(): Float32Array; seasonal_phase_lake_area_m2(): Float64Array; seasonal_phase_lake_volume_m3(): Float64Array;
  free(): void;
}"""
worker_interface_new_tail = """  seasonal_phase_local_runoff_m3_s(): Float32Array; seasonal_phase_snowmelt_runoff_m3_s(): Float32Array; seasonal_phase_snow_storage_mm(): Float32Array; seasonal_phase_potential_discharge_m3_s(): Float32Array; seasonal_phase_realized_discharge_m3_s(): Float32Array; seasonal_flow_presence_fraction(): Float32Array; seasonal_flow_regime(): Uint8Array; seasonal_phase_lake_surface_elevation_m(): Float32Array; seasonal_phase_lake_area_m2(): Float64Array; seasonal_phase_lake_volume_m3(): Float64Array;
  erosion_stage_id(): string; erosion_stage_version(): number; erosion_stage_seed_hex(): string; fluvial_erosion_hash_hex(): string; erosion_parameter_hash_hex(): string; erosion_inheritance_hash_hex(): string; erosion_topography_hash_hex(): string; erosion_drainage_hash_hex(): string; erosion_lake_hash_hex(): string; erosion_seasonal_hydrology_hash_hex(): string;
  erosive_sample_count(): number; active_lake_trap_count(): number; maximum_effective_discharge_m3_s(): number; maximum_channel_slope(): number; maximum_channel_width_m(): number; maximum_incision_potential_m_per_year(): number; total_sediment_generated_kg_s(): number; total_land_deposition_kg_s(): number; total_lake_deposition_kg_s(): number; total_terminal_ocean_deposition_kg_s(): number; maximum_sediment_load_kg_s(): number; sediment_conservation_relative_error(): number;
  effective_discharge_m3_s(): Float32Array; channel_slope(): Float32Array; channel_width_m(): Float32Array; erodibility_index(): Float32Array; stream_power_index(): Float32Array; incision_potential_m_per_year(): Float32Array; local_sediment_supply_kg_s(): Float32Array; sediment_transport_capacity_kg_s(): Float32Array; sediment_load_kg_s(): Float32Array; sediment_deposition_kg_s(): Float32Array;
  free(): void;
}"""
text = once(text, worker_interface_tail, worker_interface_new_tail, "worker erosion interface")
if text.count("progress('packaging', 13, 14,") != 2:
    raise SystemExit(f"worker packaging progress: expected two targets, found {text.count("progress('packaging', 13, 14,")}")
text = text.replace("progress('packaging', 13, 14,", "progress('packaging', 14, 15,")
seasonal_vectors = """    const seasonalPhaseLocalRunoffM3S = output.seasonal_phase_local_runoff_m3_s(); const seasonalPhaseSnowmeltRunoffM3S = output.seasonal_phase_snowmelt_runoff_m3_s(); const seasonalPhaseSnowStorageMm = output.seasonal_phase_snow_storage_mm(); const seasonalPhasePotentialDischargeM3S = output.seasonal_phase_potential_discharge_m3_s(); const seasonalPhaseRealizedDischargeM3S = output.seasonal_phase_realized_discharge_m3_s(); const seasonalFlowPresenceFraction = output.seasonal_flow_presence_fraction(); const seasonalFlowRegime = output.seasonal_flow_regime(); const seasonalPhaseLakeSurfaceElevationM = output.seasonal_phase_lake_surface_elevation_m(); const seasonalPhaseLakeAreaM2 = output.seasonal_phase_lake_area_m2(); const seasonalPhaseLakeVolumeM3 = output.seasonal_phase_lake_volume_m3();
"""
erosion_vectors = seasonal_vectors + """    const effectiveDischargeM3S = output.effective_discharge_m3_s(); const channelSlope = output.channel_slope(); const channelWidthM = output.channel_width_m(); const erodibilityIndex = output.erodibility_index(); const streamPowerIndex = output.stream_power_index(); const incisionPotentialMPerYear = output.incision_potential_m_per_year(); const localSedimentSupplyKgS = output.local_sediment_supply_kg_s(); const sedimentTransportCapacityKgS = output.sediment_transport_capacity_kg_s(); const sedimentLoadKgS = output.sediment_load_kg_s(); const sedimentDepositionKgS = output.sediment_deposition_kg_s();
"""
text = once(text, seasonal_vectors, erosion_vectors, "worker erosion vector reads")
result_tail = """      seasonalPhaseLocalRunoffM3S, seasonalPhaseSnowmeltRunoffM3S, seasonalPhaseSnowStorageMm, seasonalPhasePotentialDischargeM3S, seasonalPhaseRealizedDischargeM3S, seasonalFlowPresenceFraction, seasonalFlowRegime, seasonalPhaseLakeSurfaceElevationM, seasonalPhaseLakeAreaM2, seasonalPhaseLakeVolumeM3,
    };
"""
result_new_tail = """      seasonalPhaseLocalRunoffM3S, seasonalPhaseSnowmeltRunoffM3S, seasonalPhaseSnowStorageMm, seasonalPhasePotentialDischargeM3S, seasonalPhaseRealizedDischargeM3S, seasonalFlowPresenceFraction, seasonalFlowRegime, seasonalPhaseLakeSurfaceElevationM, seasonalPhaseLakeAreaM2, seasonalPhaseLakeVolumeM3,
      erosionStage: { id: output.erosion_stage_id(), version: output.erosion_stage_version(), stageSeed: output.erosion_stage_seed_hex(), durationMs: 0 },
      erosionMetrics: { sampleCount: output.fine_sample_count(), orbitalPhaseCount: output.orbital_phase_count(), erosiveSampleCount: output.erosive_sample_count(), activeLakeTrapCount: output.active_lake_trap_count(), maximumEffectiveDischargeM3S: output.maximum_effective_discharge_m3_s(), maximumChannelSlope: output.maximum_channel_slope(), maximumChannelWidthM: output.maximum_channel_width_m(), maximumIncisionPotentialMPerYear: output.maximum_incision_potential_m_per_year(), totalSedimentGeneratedKgS: output.total_sediment_generated_kg_s(), totalLandDepositionKgS: output.total_land_deposition_kg_s(), totalLakeDepositionKgS: output.total_lake_deposition_kg_s(), totalTerminalOceanDepositionKgS: output.total_terminal_ocean_deposition_kg_s(), maximumSedimentLoadKgS: output.maximum_sediment_load_kg_s(), sedimentConservationRelativeError: output.sediment_conservation_relative_error(), erosionParameterHash: output.erosion_parameter_hash_hex(), inheritanceHash: output.erosion_inheritance_hash_hex(), topographyHash: output.erosion_topography_hash_hex(), drainageHash: output.erosion_drainage_hash_hex(), lakeHash: output.erosion_lake_hash_hex(), seasonalHydrologyHash: output.erosion_seasonal_hydrology_hash_hex(), fluvialErosionHash: output.fluvial_erosion_hash_hex() },
      effectiveDischargeM3S, channelSlope, channelWidthM, erodibilityIndex, streamPowerIndex, incisionPotentialMPerYear, localSedimentSupplyKgS, sedimentTransportCapacityKgS, sedimentLoadKgS, sedimentDepositionKgS,
    };
"""
text = once(text, result_tail, result_new_tail, "worker erosion result")
transfer_tail = "result.seasonalPhaseLakeSurfaceElevationM.buffer, result.seasonalPhaseLakeAreaM2.buffer, result.seasonalPhaseLakeVolumeM3.buffer]); return;"
transfer_new_tail = "result.seasonalPhaseLakeSurfaceElevationM.buffer, result.seasonalPhaseLakeAreaM2.buffer, result.seasonalPhaseLakeVolumeM3.buffer, result.effectiveDischargeM3S.buffer, result.channelSlope.buffer, result.channelWidthM.buffer, result.erodibilityIndex.buffer, result.streamPowerIndex.buffer, result.incisionPotentialMPerYear.buffer, result.localSedimentSupplyKgS.buffer, result.sedimentTransportCapacityKgS.buffer, result.sedimentLoadKgS.buffer, result.sedimentDepositionKgS.buffer]); return;"
text = once(text, transfer_tail, transfer_new_tail, "worker erosion transfer list")
path.write_text(text)

# Carry all cumulative browser tests forward to protocol v15.
for path in Path("tests").glob("*.test.ts"):
    text = path.read_text()
    text = text.replace("protocol v14", "protocol v15")
    text = text.replace("WORLDGEN_PROTOCOL_VERSION, 14", "WORLDGEN_PROTOCOL_VERSION, 15")
    text = text.replace("const PROTOCOL = 14;", "const PROTOCOL = 15;")
    path.write_text(text)

# Dedicated WG-7A browser-contract regression.
Path("tests/wg7Erosion.test.ts").write_text("""import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-7A cumulative browser contract is protocol v15 and single-request', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 15);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');
  for (const field of [
    'erosionStage', 'erosionMetrics', 'effectiveDischargeM3S', 'channelSlope', 'channelWidthM',
    'erodibilityIndex', 'streamPowerIndex', 'incisionPotentialMPerYear', 'localSedimentSupplyKgS',
    'sedimentTransportCapacityKgS', 'sedimentLoadKgS', 'sedimentDepositionKgS',
  ]) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  assert.match(bridge, /generate_fluvial_erosion_sediment/);
  assert.match(bridge, /fluvial-erosion-sediment/);
  assert.match(worker, /erosion_seasonal_hydrology_hash_hex/);
  assert.match(worker, /sediment_conservation_relative_error/);
  assert.match(worker, /client\.generateClimate|generateClimate/);
  assert.doesNotMatch(worker, /generateErosion/);
});
""")
