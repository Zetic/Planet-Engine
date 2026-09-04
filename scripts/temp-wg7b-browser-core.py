from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)

# --- Rust WASM protocol version -------------------------------------------------
path = Path("rust/interlink-worldgen-wasm/src/lib.rs")
text = path.read_text()
text = replace_once(
    text,
    "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 15;",
    "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 16;",
    "WASM protocol version",
)
path.write_text(text)

# --- Rust cumulative WASM bridge -----------------------------------------------
path = Path("rust/interlink-worldgen-wasm/src/climate_bridge.rs")
text = path.read_text()
text = replace_once(
    text,
    "    build_icosphere, generate_coupled_climate_with_diagnostics, generate_crust_and_history,\n    generate_drainage_topology, generate_fluvial_erosion_sediment, generate_initial_topography,",
    "    build_icosphere, generate_bounded_terrain_evolution, generate_coupled_climate_with_diagnostics,\n    generate_crust_and_history, generate_drainage_topology, generate_fluvial_erosion_sediment,\n    generate_initial_topography,",
    "WASM evolution generator import",
)
text = replace_once(
    text,
    "    SeasonalHydrologyRequest, SeasonalHydrologyState, TectonicsRequest, TopographyRequest,\n    TopographyState, WORLDGEN_ENGINE_VERSION,",
    "    SeasonalHydrologyRequest, SeasonalHydrologyState, TectonicsRequest, TerrainEvolutionRequest,\n    TerrainEvolutionState, TopographyRequest, TopographyState, WORLDGEN_ENGINE_VERSION,",
    "WASM evolution types import",
)
text = replace_once(text, "const GENERATION_STAGE_COUNT: u32 = 15;", "const GENERATION_STAGE_COUNT: u32 = 16;", "WASM stage count")
text = replace_once(
    text,
    "    seasonal: SeasonalHydrologyState,\n    erosion: FluvialErosionState,\n    planet: PlanetPhysicalParameters,",
    "    seasonal: SeasonalHydrologyState,\n    erosion: FluvialErosionState,\n    evolution: TerrainEvolutionState,\n    planet: PlanetPhysicalParameters,",
    "WASM evolution state field",
)
erosion_block = '''        report_generation_progress(progress, "fluvial-erosion-sediment", 13, 0, 1);
        let erosion = generate_fluvial_erosion_sediment(
            &fine_topology,
            &inherited,
            &terrain,
            &drainage,
            &lakes,
            &seasonal,
            planet,
            &FluvialErosionRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "fluvial-erosion-sediment", 13, 1, 1);
        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;
'''
evolution_block = erosion_block.replace(
    "        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;\n",
    '''
        report_generation_progress(progress, "bounded-terrain-evolution", 14, 0, 1);
        let evolution = generate_bounded_terrain_evolution(
            &fine_topology,
            &terrain,
            &drainage,
            &runoff,
            &lakes,
            &erosion,
            planet,
            &TerrainEvolutionRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "bounded-terrain-evolution", 14, 1, 1);
        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;
''',
)
text = replace_once(text, erosion_block, evolution_block, "WASM WG-7B generation")
text = replace_once(
    text,
    "            seasonal,\n            erosion,\n            planet,",
    "            seasonal,\n            erosion,\n            evolution,\n            planet,",
    "WASM evolution state construction",
)
old_tail = '''    pub fn sediment_deposition_kg_s(&self) -> Vec<f32> {
        self.erosion.sediment_deposition_kg_s.clone()
    }
}
'''
new_tail = '''    pub fn sediment_deposition_kg_s(&self) -> Vec<f32> {
        self.erosion.sediment_deposition_kg_s.clone()
    }

    pub fn evolution_stage_id(&self) -> String {
        self.evolution.stage.id.to_owned()
    }
    pub fn evolution_stage_version(&self) -> u32 {
        self.evolution.stage.version
    }
    pub fn evolution_stage_seed_hex(&self) -> String {
        format!("{:016x}", self.evolution.stage.derived_seed)
    }
    pub fn terrain_evolution_hash_hex(&self) -> String {
        self.evolution.metrics.terrain_evolution_hash_hex()
    }
    pub fn evolution_parameter_hash_hex(&self) -> String {
        self.evolution.metrics.evolution_parameter_hash_hex()
    }
    pub fn evolved_surface_hash_hex(&self) -> String {
        self.evolution.metrics.evolved_surface_hash_hex()
    }
    pub fn post_erosion_drainage_hash_hex(&self) -> String {
        self.evolution.metrics.post_erosion_drainage_hash_hex()
    }
    pub fn evolution_topography_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution.metrics.topography_hash)
    }
    pub fn evolution_drainage_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution.metrics.drainage_hash)
    }
    pub fn evolution_runoff_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution.metrics.runoff_hash)
    }
    pub fn evolution_lake_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution.metrics.lake_hash)
    }
    pub fn evolution_fluvial_erosion_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution.metrics.fluvial_erosion_hash)
    }
    pub fn geomorphic_duration_years(&self) -> f64 {
        self.evolution.metrics.geomorphic_duration_years
    }
    pub fn evolved_eroded_sample_count(&self) -> u32 {
        self.evolution.metrics.eroded_sample_count
    }
    pub fn evolved_depositional_sample_count(&self) -> u32 {
        self.evolution.metrics.depositional_sample_count
    }
    pub fn receiver_changed_sample_count(&self) -> u32 {
        self.evolution.metrics.receiver_changed_sample_count
    }
    pub fn receiver_changed_fraction(&self) -> f64 {
        self.evolution.metrics.receiver_changed_fraction
    }
    pub fn maximum_applied_erosion_m(&self) -> f64 {
        self.evolution.metrics.maximum_applied_erosion_m
    }
    pub fn maximum_applied_deposition_m(&self) -> f64 {
        self.evolution.metrics.maximum_applied_deposition_m
    }
    pub fn maximum_absolute_terrain_change_m(&self) -> f64 {
        self.evolution.metrics.maximum_absolute_terrain_change_m
    }
    pub fn mean_land_absolute_terrain_change_m(&self) -> f64 {
        self.evolution.metrics.mean_land_absolute_terrain_change_m
    }
    pub fn total_applied_sediment_generated_kg_s(&self) -> f64 {
        self.evolution.metrics.total_applied_sediment_generated_kg_s
    }
    pub fn evolution_total_land_deposition_kg_s(&self) -> f64 {
        self.evolution.metrics.total_land_deposition_kg_s
    }
    pub fn total_lake_sink_kg_s(&self) -> f64 {
        self.evolution.metrics.total_lake_sink_kg_s
    }
    pub fn total_terminal_ocean_sink_kg_s(&self) -> f64 {
        self.evolution.metrics.total_terminal_ocean_sink_kg_s
    }
    pub fn evolution_sediment_conservation_relative_error(&self) -> f64 {
        self.evolution.metrics.sediment_conservation_relative_error
    }
    pub fn maximum_post_erosion_potential_discharge_m3_s(&self) -> f64 {
        self.evolution.metrics.maximum_post_erosion_potential_discharge_m3_s
    }
    pub fn post_erosion_runoff_conservation_relative_error(&self) -> f64 {
        self.evolution.metrics.post_erosion_runoff_conservation_relative_error
    }
    pub fn evolved_solid_elevation_m(&self) -> Vec<f32> {
        self.evolution.evolved_solid_elevation_m.clone()
    }
    pub fn terrain_delta_m(&self) -> Vec<f32> {
        self.evolution.terrain_delta_m.clone()
    }
    pub fn applied_erosion_m(&self) -> Vec<f32> {
        self.evolution.applied_erosion_m.clone()
    }
    pub fn applied_deposition_m(&self) -> Vec<f32> {
        self.evolution.applied_deposition_m.clone()
    }
    pub fn receiver_changed_mask(&self) -> Vec<u8> {
        self.evolution.receiver_changed_mask.clone()
    }
    pub fn post_erosion_contributing_area_m2(&self) -> Vec<f64> {
        self.evolution.post_erosion_drainage.contributing_area_m2.clone()
    }
    pub fn post_erosion_potential_discharge_m3_s(&self) -> Vec<f32> {
        self.evolution.post_erosion_potential_discharge_m3_s.clone()
    }
}
'''
text = replace_once(text, old_tail, new_tail, "WASM WG-7B getters")
path.write_text(text)

# --- TypeScript protocol --------------------------------------------------------
path = Path("src/worldgen/protocol.ts")
text = path.read_text()
text = replace_once(text, "export const WORLDGEN_PROTOCOL_VERSION = 15;", "export const WORLDGEN_PROTOCOL_VERSION = 16;", "TS protocol version")
metrics_anchor = '''export interface WorldgenFluvialErosionMetrics {
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
'''
evolution_metrics = metrics_anchor + '''
export interface WorldgenTerrainEvolutionMetrics {
  sampleCount: number;
  geomorphicDurationYears: number;
  erodedSampleCount: number;
  depositionalSampleCount: number;
  receiverChangedSampleCount: number;
  receiverChangedFraction: number;
  maximumAppliedErosionM: number;
  maximumAppliedDepositionM: number;
  maximumAbsoluteTerrainChangeM: number;
  meanLandAbsoluteTerrainChangeM: number;
  totalAppliedSedimentGeneratedKgS: number;
  totalLandDepositionKgS: number;
  totalLakeSinkKgS: number;
  totalTerminalOceanSinkKgS: number;
  sedimentConservationRelativeError: number;
  maximumPostErosionPotentialDischargeM3S: number;
  postErosionRunoffConservationRelativeError: number;
  evolutionParameterHash: string;
  topographyHash: string;
  drainageHash: string;
  runoffHash: string;
  lakeHash: string;
  fluvialErosionHash: string;
  evolvedSurfaceHash: string;
  postErosionDrainageHash: string;
  terrainEvolutionHash: string;
}
'''
text = replace_once(text, metrics_anchor, evolution_metrics, "TS WG-7B metrics")
result_anchor = '''  effectiveDischargeM3S: Float32Array;
  channelSlope: Float32Array;
  channelWidthM: Float32Array;
  erodibilityIndex: Float32Array;
  streamPowerIndex: Float32Array;
  incisionPotentialMPerYear: Float32Array;
  localSedimentSupplyKgS: Float32Array;
  sedimentTransportCapacityKgS: Float32Array;
  sedimentLoadKgS: Float32Array;
  sedimentDepositionKgS: Float32Array;
}'''
result_new = '''  effectiveDischargeM3S: Float32Array;
  channelSlope: Float32Array;
  channelWidthM: Float32Array;
  erodibilityIndex: Float32Array;
  streamPowerIndex: Float32Array;
  incisionPotentialMPerYear: Float32Array;
  localSedimentSupplyKgS: Float32Array;
  sedimentTransportCapacityKgS: Float32Array;
  sedimentLoadKgS: Float32Array;
  sedimentDepositionKgS: Float32Array;
  evolutionStage: WorldgenStageMetadata;
  evolutionMetrics: WorldgenTerrainEvolutionMetrics;
  evolvedSolidElevationM: Float32Array;
  terrainDeltaM: Float32Array;
  appliedErosionM: Float32Array;
  appliedDepositionM: Float32Array;
  receiverChangedMask: Uint8Array;
  postErosionContributingAreaM2: Float64Array;
  postErosionPotentialDischargeM3S: Float32Array;
}'''
text = replace_once(text, result_anchor, result_new, "TS WG-7B result")
path.write_text(text)

# --- Worker manual WASM interface + packaging ----------------------------------
path = Path("src/worldgen/worldgenWorker.ts")
text = path.read_text()
wasm_interface_anchor = '''  effective_discharge_m3_s(): Float32Array; channel_slope(): Float32Array; channel_width_m(): Float32Array; erodibility_index(): Float32Array; stream_power_index(): Float32Array; incision_potential_m_per_year(): Float32Array; local_sediment_supply_kg_s(): Float32Array; sediment_transport_capacity_kg_s(): Float32Array; sediment_load_kg_s(): Float32Array; sediment_deposition_kg_s(): Float32Array;
  free(): void;
}'''
wasm_interface_new = '''  effective_discharge_m3_s(): Float32Array; channel_slope(): Float32Array; channel_width_m(): Float32Array; erodibility_index(): Float32Array; stream_power_index(): Float32Array; incision_potential_m_per_year(): Float32Array; local_sediment_supply_kg_s(): Float32Array; sediment_transport_capacity_kg_s(): Float32Array; sediment_load_kg_s(): Float32Array; sediment_deposition_kg_s(): Float32Array;
  evolution_stage_id(): string; evolution_stage_version(): number; evolution_stage_seed_hex(): string; terrain_evolution_hash_hex(): string; evolution_parameter_hash_hex(): string; evolved_surface_hash_hex(): string; post_erosion_drainage_hash_hex(): string; evolution_topography_hash_hex(): string; evolution_drainage_hash_hex(): string; evolution_runoff_hash_hex(): string; evolution_lake_hash_hex(): string; evolution_fluvial_erosion_hash_hex(): string;
  geomorphic_duration_years(): number; evolved_eroded_sample_count(): number; evolved_depositional_sample_count(): number; receiver_changed_sample_count(): number; receiver_changed_fraction(): number; maximum_applied_erosion_m(): number; maximum_applied_deposition_m(): number; maximum_absolute_terrain_change_m(): number; mean_land_absolute_terrain_change_m(): number; total_applied_sediment_generated_kg_s(): number; evolution_total_land_deposition_kg_s(): number; total_lake_sink_kg_s(): number; total_terminal_ocean_sink_kg_s(): number; evolution_sediment_conservation_relative_error(): number; maximum_post_erosion_potential_discharge_m3_s(): number; post_erosion_runoff_conservation_relative_error(): number;
  evolved_solid_elevation_m(): Float32Array; terrain_delta_m(): Float32Array; applied_erosion_m(): Float32Array; applied_deposition_m(): Float32Array; receiver_changed_mask(): Uint8Array; post_erosion_contributing_area_m2(): Float64Array; post_erosion_potential_discharge_m3_s(): Float32Array;
  free(): void;
}'''
text = replace_once(text, wasm_interface_anchor, wasm_interface_new, "worker WG-7B WASM interface")
text = replace_once(text, "progress('packaging', 14, 15, 0, 1);", "progress('packaging', 15, 16, 0, 1);", "worker packaging start")
declaration_anchor = "    const effectiveDischargeM3S = output.effective_discharge_m3_s(); const channelSlope = output.channel_slope(); const channelWidthM = output.channel_width_m(); const erodibilityIndex = output.erodibility_index(); const streamPowerIndex = output.stream_power_index(); const incisionPotentialMPerYear = output.incision_potential_m_per_year(); const localSedimentSupplyKgS = output.local_sediment_supply_kg_s(); const sedimentTransportCapacityKgS = output.sediment_transport_capacity_kg_s(); const sedimentLoadKgS = output.sediment_load_kg_s(); const sedimentDepositionKgS = output.sediment_deposition_kg_s();"
declaration_new = declaration_anchor + "\n    const evolvedSolidElevationM = output.evolved_solid_elevation_m(); const terrainDeltaM = output.terrain_delta_m(); const appliedErosionM = output.applied_erosion_m(); const appliedDepositionM = output.applied_deposition_m(); const receiverChangedMask = output.receiver_changed_mask(); const postErosionContributingAreaM2 = output.post_erosion_contributing_area_m2(); const postErosionPotentialDischargeM3S = output.post_erosion_potential_discharge_m3_s();"
text = replace_once(text, declaration_anchor, declaration_new, "worker WG-7B arrays")
return_anchor = "      effectiveDischargeM3S, channelSlope, channelWidthM, erodibilityIndex, streamPowerIndex, incisionPotentialMPerYear, localSedimentSupplyKgS, sedimentTransportCapacityKgS, sedimentLoadKgS, sedimentDepositionKgS,\n    };"
return_new = '''      effectiveDischargeM3S, channelSlope, channelWidthM, erodibilityIndex, streamPowerIndex, incisionPotentialMPerYear, localSedimentSupplyKgS, sedimentTransportCapacityKgS, sedimentLoadKgS, sedimentDepositionKgS,
      evolutionStage: { id: output.evolution_stage_id(), version: output.evolution_stage_version(), stageSeed: output.evolution_stage_seed_hex(), durationMs: 0 },
      evolutionMetrics: { sampleCount: output.fine_sample_count(), geomorphicDurationYears: output.geomorphic_duration_years(), erodedSampleCount: output.evolved_eroded_sample_count(), depositionalSampleCount: output.evolved_depositional_sample_count(), receiverChangedSampleCount: output.receiver_changed_sample_count(), receiverChangedFraction: output.receiver_changed_fraction(), maximumAppliedErosionM: output.maximum_applied_erosion_m(), maximumAppliedDepositionM: output.maximum_applied_deposition_m(), maximumAbsoluteTerrainChangeM: output.maximum_absolute_terrain_change_m(), meanLandAbsoluteTerrainChangeM: output.mean_land_absolute_terrain_change_m(), totalAppliedSedimentGeneratedKgS: output.total_applied_sediment_generated_kg_s(), totalLandDepositionKgS: output.evolution_total_land_deposition_kg_s(), totalLakeSinkKgS: output.total_lake_sink_kg_s(), totalTerminalOceanSinkKgS: output.total_terminal_ocean_sink_kg_s(), sedimentConservationRelativeError: output.evolution_sediment_conservation_relative_error(), maximumPostErosionPotentialDischargeM3S: output.maximum_post_erosion_potential_discharge_m3_s(), postErosionRunoffConservationRelativeError: output.post_erosion_runoff_conservation_relative_error(), evolutionParameterHash: output.evolution_parameter_hash_hex(), topographyHash: output.evolution_topography_hash_hex(), drainageHash: output.evolution_drainage_hash_hex(), runoffHash: output.evolution_runoff_hash_hex(), lakeHash: output.evolution_lake_hash_hex(), fluvialErosionHash: output.evolution_fluvial_erosion_hash_hex(), evolvedSurfaceHash: output.evolved_surface_hash_hex(), postErosionDrainageHash: output.post_erosion_drainage_hash_hex(), terrainEvolutionHash: output.terrain_evolution_hash_hex() },
      evolvedSolidElevationM, terrainDeltaM, appliedErosionM, appliedDepositionM, receiverChangedMask, postErosionContributingAreaM2, postErosionPotentialDischargeM3S,
    };'''
text = replace_once(text, return_anchor, return_new, "worker WG-7B result")
text = replace_once(text, "progress('packaging', 14, 15, 1, 1);", "progress('packaging', 15, 16, 1, 1);", "worker packaging end")
transfer_anchor = "result.sedimentLoadKgS.buffer, result.sedimentDepositionKgS.buffer]); return;"
transfer_new = "result.sedimentLoadKgS.buffer, result.sedimentDepositionKgS.buffer, result.evolvedSolidElevationM.buffer, result.terrainDeltaM.buffer, result.appliedErosionM.buffer, result.appliedDepositionM.buffer, result.receiverChangedMask.buffer, result.postErosionContributingAreaM2.buffer, result.postErosionPotentialDischargeM3S.buffer]); return;"
text = replace_once(text, transfer_anchor, transfer_new, "worker WG-7B transfer list")
path.write_text(text)
