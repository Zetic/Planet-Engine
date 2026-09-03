import {
  WORLDGEN_PROTOCOL_VERSION,
  validateGeologyRequest,
  validateInheritanceRequest,
  validateLithosphereRequest,
  validateSyntheticRequest,
  validateTopographyRequest,
  validateTectonicsRequest,
  validateTopologyRequest,
  type WorldgenCommand,
  type WorldgenErrorEvent,
  type WorldgenGeneratedGeologyEvent,
  type WorldgenGeneratedInheritanceEvent,
  type WorldgenGeneratedLithosphereEvent,
  type WorldgenGeneratedSyntheticEvent,
  type WorldgenGeneratedTopographyEvent,
  type WorldgenGeneratedTectonicsEvent,
  type WorldgenGeneratedTopologyEvent,
  type WorldgenGeologyResult,
  type WorldgenInheritanceResult,
  type WorldgenLithosphereResult,
  type WorldgenSyntheticResult,
  type WorldgenTopographyResult,
  type WorldgenTectonicsResult,
  type WorldgenTopologyResult,
} from './protocol.js';

interface WasmDiagnostic { generator_version(): number; stage_id(): string; stage_version(): number; stage_seed_hex(): string; width(): number; height(): number; sample_count(): bigint; minimum(): number; maximum(): number; mean(): number; field_hash_hex(): string; values(): Uint16Array; free(): void; }
interface WasmTopology { generator_version(): number; level(): number; sample_count(): number; edge_count(): number; face_count(): number; five_neighbor_count(): number; six_neighbor_count(): number; total_area_steradians(): number; minimum_area_steradians(): number; maximum_area_steradians(): number; mean_area_steradians(): number; area_coefficient_of_variation(): number; minimum_edge_arc_radians(): number; maximum_edge_arc_radians(): number; mean_edge_arc_radians(): number; edge_coefficient_of_variation(): number; minimum_interface_arc_radians(): number; maximum_interface_arc_radians(): number; mean_interface_arc_radians(): number; interface_coefficient_of_variation(): number; topology_hash_hex(): string; positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array; neighbor_arc_lengths_rad(): Float64Array; neighbor_interface_arc_lengths_rad(): Float64Array; area_steradians(): Float64Array; birth_levels(): Uint8Array; parent_edges(): Uint32Array; free(): void; }
interface WasmTectonics { generator_version(): number; stage_id(): string; stage_version(): number; stage_seed_hex(): string; level(): number; sample_count(): number; plate_count(): number; boundary_edge_count(): number; convergent_edge_count(): number; divergent_edge_count(): number; transform_edge_count(): number; minimum_plate_area_fraction(): number; maximum_plate_area_fraction(): number; mean_plate_area_fraction(): number; minimum_seed_separation_rad(): number; mean_reference_speed_mm_per_year(): number; tectonic_hash_hex(): string; topology_hash_hex(): string; positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array; plate_ids(): Uint16Array; plate_seed_samples(): Uint32Array; plate_euler_poles(): Float64Array; plate_angular_velocities_rad_per_myr(): Float64Array; plate_area_steradians(): Float64Array; boundary_samples(): Uint32Array; boundary_plate_ids(): Uint16Array; boundary_kinds(): Uint8Array; boundary_normal_rates_m_per_year(): Float64Array; boundary_shear_rates_m_per_year(): Float64Array; free(): void; }
interface WasmGeology {
  generator_version(): number; stage_id(): string; stage_version(): number; stage_seed_hex(): string; province_seed_hex(): string; property_seed_hex(): string; history_seed_hex(): string;
  level(): number; sample_count(): number; plate_count(): number; boundary_edge_count(): number; geology_hash_hex(): string; tectonic_hash_hex(): string; topology_hash_hex(): string;
  continental_area_fraction(): number; transitional_area_fraction(): number; oceanic_area_fraction(): number; mean_continental_age_myr(): number; mean_oceanic_age_myr(): number; mean_continental_thickness_km(): number; mean_oceanic_thickness_km(): number;
  oceanic_subduction_edges(): number; ocean_continent_subduction_edges(): number; continental_collision_edges(): number; oceanic_ridge_edges(): number; continental_rift_edges(): number; transitional_divergence_edges(): number; transform_edges(): number;
  positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array; plate_ids(): Uint16Array; boundary_samples(): Uint32Array; boundary_plate_ids(): Uint16Array; boundary_kinds(): Uint8Array;
  crust_kind(): Uint8Array; crust_province_id(): Uint16Array; crust_age_myr(): Float32Array; crust_thickness_km(): Float32Array; crust_density_kg_per_m3(): Float32Array; buoyancy_index(): Float32Array;
  orogenic_history(): Float32Array; rift_history(): Float32Array; ridge_history(): Float32Array; subduction_history(): Float32Array; trench_history(): Float32Array; volcanic_arc_history(): Float32Array; transform_history(): Float32Array; subsidence_history(): Float32Array; basin_potential(): Float32Array; crustal_strain(): Float32Array;
  geological_boundary_regimes(): Uint8Array; subduction_polarities(): Uint8Array; plate_scale_classes(): Uint8Array; plate_continental_fractions(): Float64Array; plate_transitional_fractions(): Float64Array; plate_oceanic_fractions(): Float64Array; plate_mean_crust_age_myr(): Float64Array; plate_mean_crust_thickness_km(): Float64Array;
  free(): void;
}
interface WasmLithosphere {
  generator_version(): number; stage_id(): string; stage_version(): number; stage_seed_hex(): string; mechanical_seed_hex(): string; mantle_seed_hex(): string; refinement_seed_hex(): string;
  level(): number; sample_count(): number; plate_count(): number; boundary_edge_count(): number; topology_hash_hex(): string; tectonic_hash_hex(): string; geology_hash_hex(): string; lithosphere_hash_hex(): string;
  mean_strength_index(): number; mean_weakness_index(): number; mean_effective_elastic_thickness_km(): number; mean_mantle_upwelling_index(): number; mean_dynamic_support_index(): number;
  suture_sample_count(): number; rift_zone_sample_count(): number; transform_zone_sample_count(): number; continental_margin_sample_count(): number; tectonic_fragment_count(): number; microplate_count(): number; terrane_count(): number; fragmented_area_fraction(): number;
  positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array; plate_ids(): Uint16Array; boundary_samples(): Uint32Array; boundary_kinds(): Uint8Array; crust_kind(): Uint8Array; geological_boundary_regimes(): Uint8Array;
  orogenic_history(): Float32Array; rift_history(): Float32Array; ridge_history(): Float32Array; subduction_history(): Float32Array; transform_history(): Float32Array; crustal_strain(): Float32Array;
  strength_index(): Float32Array; weakness_index(): Float32Array; effective_elastic_thickness_km(): Float32Array; thermal_anomaly_index(): Float32Array; mantle_upwelling_index(): Float32Array; mantle_dynamic_support_index(): Float32Array; compensated_buoyancy_index(): Float32Array; structural_fabric_strength(): Float32Array; structural_zone_kind(): Uint8Array; fragmentation_propensity(): Float32Array; fragment_ids(): Uint16Array; kinematic_domain_ids(): Uint16Array;
  fragment_parent_plate_ids(): Uint16Array; fragment_kinds(): Uint8Array; fragment_seed_samples(): Uint32Array; fragment_area_steradians(): Float64Array; fragment_area_fractions_of_parent(): Float64Array; fragment_mean_weakness(): Float64Array; fragment_mean_propensity(): Float64Array; fragment_angular_velocities_rad_per_myr(): Float64Array;
  free(): void;
}
interface WasmInheritance {
  generator_version(): number; stage_id(): string; stage_version(): number;
  coarse_level(): number; fine_level(): number; coarse_sample_count(): number; fine_sample_count(): number; added_sample_count(): number; plate_count(): number; fine_boundary_edge_count(): number;
  provenance_hash_hex(): string; parameter_hash_hex(): string; inheritance_hash_hex(): string; boundary_hash_hex(): string; coarse_topology_hash_hex(): string; fine_topology_hash_hex(): string; tectonic_hash_hex(): string; geology_hash_hex(): string; lithosphere_hash_hex(): string;
  radius_m(): number; surface_gravity_m_s2(): number; surface_water_mass_kg(): number; equivalent_global_water_depth_m(): number; ocean_water_density_kg_per_m3(): number; isostatic_mantle_density_kg_per_m3(): number; internal_heat_flux_w_per_m2(): number; mantle_thermal_expansivity_per_k(): number;
  positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array; nearest_coarse_source(): Uint32Array; inherited_sample_mask(): Uint8Array;
  plate_ids(): Uint16Array; crust_kind(): Uint8Array; crust_province_id(): Uint16Array; crust_age_myr(): Float32Array; crust_thickness_km(): Float32Array; crust_density_kg_per_m3(): Float32Array; buoyancy_index(): Float32Array;
  orogenic_history(): Float32Array; rift_history(): Float32Array; ridge_history(): Float32Array; subduction_history(): Float32Array; trench_history(): Float32Array; volcanic_arc_history(): Float32Array; transform_history(): Float32Array; subsidence_history(): Float32Array; basin_potential(): Float32Array; crustal_strain(): Float32Array;
  strength_index(): Float32Array; weakness_index(): Float32Array; effective_elastic_thickness_km(): Float32Array; thermal_anomaly_index(): Float32Array; mantle_upwelling_index(): Float32Array; mantle_dynamic_support_index(): Float32Array; compensated_buoyancy_index(): Float32Array; structural_fabric_strength(): Float32Array; structural_zone_kind(): Uint8Array; fragmentation_propensity(): Float32Array; fragment_ids(): Uint16Array; kinematic_domain_ids(): Uint16Array;
  boundary_samples(): Uint32Array; boundary_kinds(): Uint8Array; geological_boundary_regimes(): Uint8Array; subduction_polarities(): Uint8Array; boundary_normal_rates_m_per_year(): Float64Array; boundary_shear_rates_m_per_year(): Float64Array; boundary_coarse_source_indices(): Uint32Array;
  free(): void;
}

interface WasmTopography {
  generator_version(): number; stage_id(): string; stage_version(): number; stage_seed_hex(): string;
  coarse_level(): number; fine_level(): number; coarse_sample_count(): number; fine_sample_count(): number; plate_count(): number; fine_boundary_edge_count(): number;
  topography_hash_hex(): string; topography_parameter_hash_hex(): string; inheritance_hash_hex(): string; boundary_hash_hex(): string; planet_parameter_hash_hex(): string; coarse_topology_hash_hex(): string; fine_topology_hash_hex(): string; tectonic_hash_hex(): string; geology_hash_hex(): string; lithosphere_hash_hex(): string;
  minimum_solid_elevation_m(): number; maximum_solid_elevation_m(): number; mean_solid_elevation_m(): number; p05_solid_elevation_m(): number; median_solid_elevation_m(): number; p95_solid_elevation_m(): number;
  has_sea_level(): boolean; sea_level_m(): number; land_area_fraction(): number; ocean_area_fraction(): number; mean_land_elevation_m(): number; mean_water_depth_m(): number; maximum_water_depth_m(): number; target_water_volume_m3(): number; solved_water_volume_m3(): number; water_volume_relative_error(): number; clamped_sample_count(): number;
  radius_m(): number; surface_gravity_m_s2(): number; surface_water_mass_kg(): number; equivalent_global_water_depth_m(): number; ocean_water_density_kg_per_m3(): number; isostatic_mantle_density_kg_per_m3(): number; internal_heat_flux_w_per_m2(): number; mantle_thermal_expansivity_per_k(): number;
  positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array; plate_ids(): Uint16Array; crust_kind(): Uint8Array; nearest_coarse_source(): Uint32Array; inherited_sample_mask(): Uint8Array; crust_age_myr(): Float32Array; crust_thickness_km(): Float32Array; orogenic_history(): Float32Array; ridge_history(): Float32Array; trench_history(): Float32Array; strength_index(): Float32Array; weakness_index(): Float32Array; mantle_dynamic_support_index(): Float32Array; structural_zone_kind(): Uint8Array; fragmentation_propensity(): Float32Array; kinematic_domain_ids(): Uint16Array; boundary_samples(): Uint32Array; boundary_kinds(): Uint8Array; geological_boundary_regimes(): Uint8Array; boundary_coarse_source_indices(): Uint32Array;
  isostatic_elevation_m(): Float32Array; thermal_elevation_m(): Float32Array; orogenic_elevation_m(): Float32Array; ridge_elevation_m(): Float32Array; rift_basin_elevation_m(): Float32Array; trench_elevation_m(): Float32Array; arc_elevation_m(): Float32Array; mantle_dynamic_elevation_m(): Float32Array; solid_elevation_m(): Float32Array; elevation_above_sea_level_m(): Float32Array; water_depth_m(): Float32Array; submerged_mask(): Uint8Array;
  free(): void;
}
interface WorldgenWasmModule {
  default(): Promise<unknown>;
  worldgen_protocol_version(): number;
  worldgen_engine_version(): number;
  WasmWorldgenDiagnostic: new (seed: string, width: number, height: number) => WasmDiagnostic;
  WasmWorldgenTopology: new (level: number) => WasmTopology;
  WasmWorldgenTectonics: new (seed: string, level: number, plateCount: number) => WasmTectonics;
  WasmWorldgenGeology: new (seed: string, level: number, plateCount: number) => WasmGeology;
  WasmWorldgenLithosphere: new (seed: string, level: number, plateCount: number) => WasmLithosphere;
  WasmWorldgenInheritance: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number) => WasmInheritance;
  WasmWorldgenTopography: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number) => WasmTopography;
}
interface WorldgenWorkerScope {
  addEventListener(type: 'message', listener: (event: MessageEvent<WorldgenCommand>) => void): void;
  postMessage(message: WorldgenGeneratedSyntheticEvent | WorldgenGeneratedTopologyEvent | WorldgenGeneratedTectonicsEvent | WorldgenGeneratedGeologyEvent | WorldgenGeneratedLithosphereEvent | WorldgenGeneratedInheritanceEvent | WorldgenGeneratedTopographyEvent | WorldgenErrorEvent, transfer?: Transferable[]): void;
}

const workerScope = self as unknown as WorldgenWorkerScope;
let wasmModulePromise: Promise<WorldgenWasmModule> | null = null;
function nowMs(): number { return globalThis.performance?.now?.() ?? Date.now(); }

async function loadWorldgenWasm(): Promise<WorldgenWasmModule> {
  if (wasmModulePromise) return wasmModulePromise;
  wasmModulePromise = (async () => {
    const moduleUrl = new URL('../../src/wasm-worldgen/interlink_worldgen_wasm.js', import.meta.url).href;
    let module: WorldgenWasmModule;
    try { module = await import(moduleUrl) as unknown as WorldgenWasmModule; } catch (error) { const detail = error instanceof Error ? error.message : String(error); throw new Error(`Planet Engine WASM package is not available. ${detail}`); }
    await module.default();
    const actual = module.worldgen_protocol_version();
    if (actual !== WORLDGEN_PROTOCOL_VERSION) throw new Error(`Planet Engine WASM protocol ${actual} does not match browser protocol ${WORLDGEN_PROTOCOL_VERSION}.`);
    return module;
  })();
  return wasmModulePromise;
}

async function generateSynthetic(command: Extract<WorldgenCommand, { type: 'generate-synthetic' }>): Promise<WorldgenSyntheticResult> {
  validateSyntheticRequest(command.payload);
  const module = await loadWorldgenWasm();
  const startedAt = nowMs();
  const output = new module.WasmWorldgenDiagnostic(command.payload.seed, command.payload.width, command.payload.height);
  try {
    const values = output.values();
    return { engineVersion: output.generator_version(), width: output.width(), height: output.height(), values, statistics: { sampleCount: Number(output.sample_count()), minimum: output.minimum(), maximum: output.maximum(), mean: output.mean(), fieldHash: output.field_hash_hex() }, stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) } };
  } finally { output.free(); }
}

async function generateTopology(command: Extract<WorldgenCommand, { type: 'generate-topology' }>): Promise<WorldgenTopologyResult> {
  validateTopologyRequest(command.payload);
  const module = await loadWorldgenWasm();
  const startedAt = nowMs();
  const output = new module.WasmWorldgenTopology(command.payload.level);
  try {
    const positions = output.positions(); const faces = output.faces(); const neighborOffsets = output.neighbor_offsets(); const neighbors = output.neighbors(); const neighborArcLengthsRad = output.neighbor_arc_lengths_rad(); const neighborInterfaceArcLengthsRad = output.neighbor_interface_arc_lengths_rad(); const areaSteradians = output.area_steradians(); const birthLevels = output.birth_levels(); const parentEdges = output.parent_edges();
    return { engineVersion: output.generator_version(), level: output.level(), durationMs: Math.max(0, nowMs() - startedAt), metrics: { sampleCount: output.sample_count(), edgeCount: output.edge_count(), faceCount: output.face_count(), fiveNeighborCount: output.five_neighbor_count(), sixNeighborCount: output.six_neighbor_count(), totalAreaSteradians: output.total_area_steradians(), minimumAreaSteradians: output.minimum_area_steradians(), maximumAreaSteradians: output.maximum_area_steradians(), meanAreaSteradians: output.mean_area_steradians(), areaCoefficientOfVariation: output.area_coefficient_of_variation(), minimumEdgeArcRadians: output.minimum_edge_arc_radians(), maximumEdgeArcRadians: output.maximum_edge_arc_radians(), meanEdgeArcRadians: output.mean_edge_arc_radians(), edgeCoefficientOfVariation: output.edge_coefficient_of_variation(), minimumInterfaceArcRadians: output.minimum_interface_arc_radians(), maximumInterfaceArcRadians: output.maximum_interface_arc_radians(), meanInterfaceArcRadians: output.mean_interface_arc_radians(), interfaceCoefficientOfVariation: output.interface_coefficient_of_variation(), topologyHash: output.topology_hash_hex() }, positions, faces, neighborOffsets, neighbors, neighborArcLengthsRad, neighborInterfaceArcLengthsRad, areaSteradians, birthLevels, parentEdges };
  } finally { output.free(); }
}

async function generateTectonics(command: Extract<WorldgenCommand, { type: 'generate-tectonics' }>): Promise<WorldgenTectonicsResult> {
  validateTectonicsRequest(command.payload);
  const module = await loadWorldgenWasm();
  const startedAt = nowMs();
  const output = new module.WasmWorldgenTectonics(command.payload.seed, command.payload.level, command.payload.plateCount);
  try {
    const positions = output.positions(); const faces = output.faces(); const neighborOffsets = output.neighbor_offsets(); const neighbors = output.neighbors(); const plateIds = output.plate_ids(); const plateSeedSamples = output.plate_seed_samples(); const plateEulerPoles = output.plate_euler_poles(); const plateAngularVelocitiesRadPerMyr = output.plate_angular_velocities_rad_per_myr(); const plateAreaSteradians = output.plate_area_steradians(); const boundarySamples = output.boundary_samples(); const boundaryPlateIds = output.boundary_plate_ids(); const boundaryKinds = output.boundary_kinds(); const boundaryNormalRatesMPerYear = output.boundary_normal_rates_m_per_year(); const boundaryShearRatesMPerYear = output.boundary_shear_rates_m_per_year();
    return { engineVersion: output.generator_version(), level: output.level(), topologyHash: output.topology_hash_hex(), stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) }, metrics: { sampleCount: output.sample_count(), plateCount: output.plate_count(), boundaryEdgeCount: output.boundary_edge_count(), convergentEdgeCount: output.convergent_edge_count(), divergentEdgeCount: output.divergent_edge_count(), transformEdgeCount: output.transform_edge_count(), minimumPlateAreaFraction: output.minimum_plate_area_fraction(), maximumPlateAreaFraction: output.maximum_plate_area_fraction(), meanPlateAreaFraction: output.mean_plate_area_fraction(), minimumSeedSeparationRad: output.minimum_seed_separation_rad(), meanReferenceSpeedMmPerYear: output.mean_reference_speed_mm_per_year(), tectonicHash: output.tectonic_hash_hex() }, positions, faces, neighborOffsets, neighbors, plateIds, plateSeedSamples, plateEulerPoles, plateAngularVelocitiesRadPerMyr, plateAreaSteradians, boundarySamples, boundaryPlateIds, boundaryKinds, boundaryNormalRatesMPerYear, boundaryShearRatesMPerYear };
  } finally { output.free(); }
}

async function generateGeology(command: Extract<WorldgenCommand, { type: 'generate-geology' }>): Promise<WorldgenGeologyResult> {
  validateGeologyRequest(command.payload);
  const module = await loadWorldgenWasm();
  const startedAt = nowMs();
  const output = new module.WasmWorldgenGeology(command.payload.seed, command.payload.level, command.payload.plateCount);
  try {
    const positions = output.positions(); const faces = output.faces(); const neighborOffsets = output.neighbor_offsets(); const neighbors = output.neighbors(); const plateIds = output.plate_ids(); const boundarySamples = output.boundary_samples(); const boundaryPlateIds = output.boundary_plate_ids(); const boundaryKinds = output.boundary_kinds(); const crustKind = output.crust_kind(); const crustProvinceId = output.crust_province_id(); const crustAgeMyr = output.crust_age_myr(); const crustThicknessKm = output.crust_thickness_km(); const crustDensityKgPerM3 = output.crust_density_kg_per_m3(); const buoyancyIndex = output.buoyancy_index(); const orogenicHistory = output.orogenic_history(); const riftHistory = output.rift_history(); const ridgeHistory = output.ridge_history(); const subductionHistory = output.subduction_history(); const trenchHistory = output.trench_history(); const volcanicArcHistory = output.volcanic_arc_history(); const transformHistory = output.transform_history(); const subsidenceHistory = output.subsidence_history(); const basinPotential = output.basin_potential(); const crustalStrain = output.crustal_strain(); const geologicalBoundaryRegimes = output.geological_boundary_regimes(); const subductionPolarities = output.subduction_polarities(); const plateScaleClasses = output.plate_scale_classes(); const plateContinentalFractions = output.plate_continental_fractions(); const plateTransitionalFractions = output.plate_transitional_fractions(); const plateOceanicFractions = output.plate_oceanic_fractions(); const plateMeanCrustAgeMyr = output.plate_mean_crust_age_myr(); const plateMeanCrustThicknessKm = output.plate_mean_crust_thickness_km();
    return { engineVersion: output.generator_version(), level: output.level(), topologyHash: output.topology_hash_hex(), plateCount: output.plate_count(), boundaryEdgeCount: output.boundary_edge_count(), stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) }, provinceSeed: output.province_seed_hex(), propertySeed: output.property_seed_hex(), historySeed: output.history_seed_hex(), metrics: { sampleCount: output.sample_count(), continentalAreaFraction: output.continental_area_fraction(), transitionalAreaFraction: output.transitional_area_fraction(), oceanicAreaFraction: output.oceanic_area_fraction(), meanContinentalAgeMyr: output.mean_continental_age_myr(), meanOceanicAgeMyr: output.mean_oceanic_age_myr(), meanContinentalThicknessKm: output.mean_continental_thickness_km(), meanOceanicThicknessKm: output.mean_oceanic_thickness_km(), oceanicSubductionEdges: output.oceanic_subduction_edges(), oceanContinentSubductionEdges: output.ocean_continent_subduction_edges(), continentalCollisionEdges: output.continental_collision_edges(), oceanicRidgeEdges: output.oceanic_ridge_edges(), continentalRiftEdges: output.continental_rift_edges(), transitionalDivergenceEdges: output.transitional_divergence_edges(), transformEdges: output.transform_edges(), geologyHash: output.geology_hash_hex(), tectonicHash: output.tectonic_hash_hex() }, positions, faces, neighborOffsets, neighbors, plateIds, boundarySamples, boundaryPlateIds, boundaryKinds, crustKind, crustProvinceId, crustAgeMyr, crustThicknessKm, crustDensityKgPerM3, buoyancyIndex, orogenicHistory, riftHistory, ridgeHistory, subductionHistory, trenchHistory, volcanicArcHistory, transformHistory, subsidenceHistory, basinPotential, crustalStrain, geologicalBoundaryRegimes, subductionPolarities, plateScaleClasses, plateContinentalFractions, plateTransitionalFractions, plateOceanicFractions, plateMeanCrustAgeMyr, plateMeanCrustThicknessKm };
  } finally { output.free(); }
}

async function generateLithosphere(command: Extract<WorldgenCommand, { type: 'generate-lithosphere' }>): Promise<WorldgenLithosphereResult> {
  validateLithosphereRequest(command.payload);
  const module = await loadWorldgenWasm();
  const startedAt = nowMs();
  const output = new module.WasmWorldgenLithosphere(command.payload.seed, command.payload.level, command.payload.plateCount);
  try {
    const positions = output.positions(); const faces = output.faces(); const neighborOffsets = output.neighbor_offsets(); const neighbors = output.neighbors(); const plateIds = output.plate_ids(); const boundarySamples = output.boundary_samples(); const boundaryKinds = output.boundary_kinds(); const crustKind = output.crust_kind(); const geologicalBoundaryRegimes = output.geological_boundary_regimes();
    const orogenicHistory = output.orogenic_history(); const riftHistory = output.rift_history(); const ridgeHistory = output.ridge_history(); const subductionHistory = output.subduction_history(); const transformHistory = output.transform_history(); const crustalStrain = output.crustal_strain();
    const strengthIndex = output.strength_index(); const weaknessIndex = output.weakness_index(); const effectiveElasticThicknessKm = output.effective_elastic_thickness_km(); const thermalAnomalyIndex = output.thermal_anomaly_index(); const mantleUpwellingIndex = output.mantle_upwelling_index(); const mantleDynamicSupportIndex = output.mantle_dynamic_support_index(); const compensatedBuoyancyIndex = output.compensated_buoyancy_index(); const structuralFabricStrength = output.structural_fabric_strength(); const structuralZoneKind = output.structural_zone_kind(); const fragmentationPropensity = output.fragmentation_propensity(); const fragmentIds = output.fragment_ids(); const kinematicDomainIds = output.kinematic_domain_ids();
    const fragmentParentPlateIds = output.fragment_parent_plate_ids(); const fragmentKinds = output.fragment_kinds(); const fragmentSeedSamples = output.fragment_seed_samples(); const fragmentAreaSteradians = output.fragment_area_steradians(); const fragmentAreaFractionsOfParent = output.fragment_area_fractions_of_parent(); const fragmentMeanWeakness = output.fragment_mean_weakness(); const fragmentMeanPropensity = output.fragment_mean_propensity(); const fragmentAngularVelocitiesRadPerMyr = output.fragment_angular_velocities_rad_per_myr();
    return {
      engineVersion: output.generator_version(), level: output.level(), topologyHash: output.topology_hash_hex(), plateCount: output.plate_count(), boundaryEdgeCount: output.boundary_edge_count(),
      stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) }, mechanicalSeed: output.mechanical_seed_hex(), mantleSeed: output.mantle_seed_hex(), refinementSeed: output.refinement_seed_hex(),
      metrics: { sampleCount: output.sample_count(), meanStrengthIndex: output.mean_strength_index(), meanWeaknessIndex: output.mean_weakness_index(), meanEffectiveElasticThicknessKm: output.mean_effective_elastic_thickness_km(), meanMantleUpwellingIndex: output.mean_mantle_upwelling_index(), meanDynamicSupportIndex: output.mean_dynamic_support_index(), sutureSampleCount: output.suture_sample_count(), riftZoneSampleCount: output.rift_zone_sample_count(), transformZoneSampleCount: output.transform_zone_sample_count(), continentalMarginSampleCount: output.continental_margin_sample_count(), tectonicFragmentCount: output.tectonic_fragment_count(), microplateCount: output.microplate_count(), terraneCount: output.terrane_count(), fragmentedAreaFraction: output.fragmented_area_fraction(), lithosphereHash: output.lithosphere_hash_hex(), geologyHash: output.geology_hash_hex(), tectonicHash: output.tectonic_hash_hex() },
      positions, faces, neighborOffsets, neighbors, plateIds, boundarySamples, boundaryKinds, crustKind, geologicalBoundaryRegimes, orogenicHistory, riftHistory, ridgeHistory, subductionHistory, transformHistory, crustalStrain, strengthIndex, weaknessIndex, effectiveElasticThicknessKm, thermalAnomalyIndex, mantleUpwellingIndex, mantleDynamicSupportIndex, compensatedBuoyancyIndex, structuralFabricStrength, structuralZoneKind, fragmentationPropensity, fragmentIds, kinematicDomainIds, fragmentParentPlateIds, fragmentKinds, fragmentSeedSamples, fragmentAreaSteradians, fragmentAreaFractionsOfParent, fragmentMeanWeakness, fragmentMeanPropensity, fragmentAngularVelocitiesRadPerMyr,
    };
  } finally { output.free(); }
}

async function generateInheritance(command: Extract<WorldgenCommand, { type: 'generate-inheritance' }>): Promise<WorldgenInheritanceResult> {
  validateInheritanceRequest(command.payload);
  const module = await loadWorldgenWasm();
  const startedAt = nowMs();
  const output = new module.WasmWorldgenInheritance(command.payload.seed, command.payload.coarseLevel, command.payload.fineLevel, command.payload.plateCount);
  try {
    const positions = output.positions(); const faces = output.faces(); const neighborOffsets = output.neighbor_offsets(); const neighbors = output.neighbors(); const nearestCoarseSource = output.nearest_coarse_source(); const inheritedSampleMask = output.inherited_sample_mask();
    const plateIds = output.plate_ids(); const crustKind = output.crust_kind(); const crustProvinceId = output.crust_province_id(); const crustAgeMyr = output.crust_age_myr(); const crustThicknessKm = output.crust_thickness_km(); const crustDensityKgPerM3 = output.crust_density_kg_per_m3(); const buoyancyIndex = output.buoyancy_index();
    const orogenicHistory = output.orogenic_history(); const riftHistory = output.rift_history(); const ridgeHistory = output.ridge_history(); const subductionHistory = output.subduction_history(); const trenchHistory = output.trench_history(); const volcanicArcHistory = output.volcanic_arc_history(); const transformHistory = output.transform_history(); const subsidenceHistory = output.subsidence_history(); const basinPotential = output.basin_potential(); const crustalStrain = output.crustal_strain();
    const strengthIndex = output.strength_index(); const weaknessIndex = output.weakness_index(); const effectiveElasticThicknessKm = output.effective_elastic_thickness_km(); const thermalAnomalyIndex = output.thermal_anomaly_index(); const mantleUpwellingIndex = output.mantle_upwelling_index(); const mantleDynamicSupportIndex = output.mantle_dynamic_support_index(); const compensatedBuoyancyIndex = output.compensated_buoyancy_index(); const structuralFabricStrength = output.structural_fabric_strength(); const structuralZoneKind = output.structural_zone_kind(); const fragmentationPropensity = output.fragmentation_propensity(); const fragmentIds = output.fragment_ids(); const kinematicDomainIds = output.kinematic_domain_ids();
    const boundarySamples = output.boundary_samples(); const boundaryKinds = output.boundary_kinds(); const geologicalBoundaryRegimes = output.geological_boundary_regimes(); const subductionPolarities = output.subduction_polarities(); const boundaryNormalRatesMPerYear = output.boundary_normal_rates_m_per_year(); const boundaryShearRatesMPerYear = output.boundary_shear_rates_m_per_year(); const boundaryCoarseSourceIndices = output.boundary_coarse_source_indices();
    return {
      engineVersion: output.generator_version(), coarseLevel: output.coarse_level(), fineLevel: output.fine_level(),
      stage: { id: output.stage_id(), version: output.stage_version(), durationMs: Math.max(0, nowMs() - startedAt) },
      metrics: { coarseSampleCount: output.coarse_sample_count(), fineSampleCount: output.fine_sample_count(), addedSampleCount: output.added_sample_count(), plateCount: output.plate_count(), fineBoundaryEdgeCount: output.fine_boundary_edge_count(), coarseTopologyHash: output.coarse_topology_hash_hex(), fineTopologyHash: output.fine_topology_hash_hex(), tectonicHash: output.tectonic_hash_hex(), geologyHash: output.geology_hash_hex(), lithosphereHash: output.lithosphere_hash_hex(), provenanceHash: output.provenance_hash_hex(), parameterHash: output.parameter_hash_hex(), inheritanceHash: output.inheritance_hash_hex(), boundaryHash: output.boundary_hash_hex() },
      parameters: { radiusM: output.radius_m(), surfaceGravityMS2: output.surface_gravity_m_s2(), surfaceWaterMassKg: output.surface_water_mass_kg(), equivalentGlobalWaterDepthM: output.equivalent_global_water_depth_m(), oceanWaterDensityKgPerM3: output.ocean_water_density_kg_per_m3(), isostaticMantleDensityKgPerM3: output.isostatic_mantle_density_kg_per_m3(), internalHeatFluxWPerM2: output.internal_heat_flux_w_per_m2(), mantleThermalExpansivityPerK: output.mantle_thermal_expansivity_per_k() },
      positions, faces, neighborOffsets, neighbors, nearestCoarseSource, inheritedSampleMask, plateIds, crustKind, crustProvinceId, crustAgeMyr, crustThicknessKm, crustDensityKgPerM3, buoyancyIndex, orogenicHistory, riftHistory, ridgeHistory, subductionHistory, trenchHistory, volcanicArcHistory, transformHistory, subsidenceHistory, basinPotential, crustalStrain, strengthIndex, weaknessIndex, effectiveElasticThicknessKm, thermalAnomalyIndex, mantleUpwellingIndex, mantleDynamicSupportIndex, compensatedBuoyancyIndex, structuralFabricStrength, structuralZoneKind, fragmentationPropensity, fragmentIds, kinematicDomainIds, boundarySamples, boundaryKinds, geologicalBoundaryRegimes, subductionPolarities, boundaryNormalRatesMPerYear, boundaryShearRatesMPerYear, boundaryCoarseSourceIndices,
    };
  } finally { output.free(); }
}


async function generateTopography(command: Extract<WorldgenCommand, { type: 'generate-topography' }>): Promise<WorldgenTopographyResult> {
  validateTopographyRequest(command.payload);
  const module = await loadWorldgenWasm();
  const startedAt = nowMs();
  const output = new module.WasmWorldgenTopography(command.payload.seed, command.payload.coarseLevel, command.payload.fineLevel, command.payload.plateCount);
  try {
    const positions = output.positions(); const faces = output.faces(); const neighborOffsets = output.neighbor_offsets(); const neighbors = output.neighbors();
    const plateIds = output.plate_ids(); const crustKind = output.crust_kind(); const nearestCoarseSource = output.nearest_coarse_source(); const inheritedSampleMask = output.inherited_sample_mask(); const crustAgeMyr = output.crust_age_myr(); const crustThicknessKm = output.crust_thickness_km();
    const orogenicHistory = output.orogenic_history(); const ridgeHistory = output.ridge_history(); const trenchHistory = output.trench_history(); const strengthIndex = output.strength_index(); const weaknessIndex = output.weakness_index(); const mantleDynamicSupportIndex = output.mantle_dynamic_support_index(); const structuralZoneKind = output.structural_zone_kind(); const fragmentationPropensity = output.fragmentation_propensity(); const kinematicDomainIds = output.kinematic_domain_ids();
    const boundarySamples = output.boundary_samples(); const boundaryKinds = output.boundary_kinds(); const geologicalBoundaryRegimes = output.geological_boundary_regimes(); const boundaryCoarseSourceIndices = output.boundary_coarse_source_indices();
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
      parameters: { radiusM: output.radius_m(), surfaceGravityMS2: output.surface_gravity_m_s2(), surfaceWaterMassKg: output.surface_water_mass_kg(), equivalentGlobalWaterDepthM: output.equivalent_global_water_depth_m(), oceanWaterDensityKgPerM3: output.ocean_water_density_kg_per_m3(), isostaticMantleDensityKgPerM3: output.isostatic_mantle_density_kg_per_m3(), internalHeatFluxWPerM2: output.internal_heat_flux_w_per_m2(), mantleThermalExpansivityPerK: output.mantle_thermal_expansivity_per_k() },
      positions, faces, neighborOffsets, neighbors, plateIds, crustKind, nearestCoarseSource, inheritedSampleMask, crustAgeMyr, crustThicknessKm, orogenicHistory, ridgeHistory, trenchHistory, strengthIndex, weaknessIndex, mantleDynamicSupportIndex, structuralZoneKind, fragmentationPropensity, kinematicDomainIds, boundarySamples, boundaryKinds, geologicalBoundaryRegimes, boundaryCoarseSourceIndices,
      isostaticElevationM, thermalElevationM, orogenicElevationM, ridgeElevationM, riftBasinElevationM, trenchElevationM, arcElevationM, mantleDynamicElevationM, solidElevationM, elevationAboveSeaLevelM, waterDepthM, submergedMask,
    };
  } finally { output.free(); }
}

workerScope.addEventListener('message', async messageEvent => {
  const command = messageEvent.data;
  try {
    if (!command || command.protocolVersion !== WORLDGEN_PROTOCOL_VERSION) throw new Error(`Worldgen protocol must be ${WORLDGEN_PROTOCOL_VERSION}.`);
    if (command.type === 'generate-synthetic') { const result = await generateSynthetic(command); workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-synthetic', payload: result }, [result.values.buffer]); return; }
    if (command.type === 'generate-topology') { const result = await generateTopology(command); workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-topology', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.neighborArcLengthsRad.buffer, result.neighborInterfaceArcLengthsRad.buffer, result.areaSteradians.buffer, result.birthLevels.buffer, result.parentEdges.buffer]); return; }
    if (command.type === 'generate-tectonics') { const result = await generateTectonics(command); workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-tectonics', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.plateSeedSamples.buffer, result.plateEulerPoles.buffer, result.plateAngularVelocitiesRadPerMyr.buffer, result.plateAreaSteradians.buffer, result.boundarySamples.buffer, result.boundaryPlateIds.buffer, result.boundaryKinds.buffer, result.boundaryNormalRatesMPerYear.buffer, result.boundaryShearRatesMPerYear.buffer]); return; }
    if (command.type === 'generate-geology') {
      const result = await generateGeology(command);
      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-geology', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.boundarySamples.buffer, result.boundaryPlateIds.buffer, result.boundaryKinds.buffer, result.crustKind.buffer, result.crustProvinceId.buffer, result.crustAgeMyr.buffer, result.crustThicknessKm.buffer, result.crustDensityKgPerM3.buffer, result.buoyancyIndex.buffer, result.orogenicHistory.buffer, result.riftHistory.buffer, result.ridgeHistory.buffer, result.subductionHistory.buffer, result.trenchHistory.buffer, result.volcanicArcHistory.buffer, result.transformHistory.buffer, result.subsidenceHistory.buffer, result.basinPotential.buffer, result.crustalStrain.buffer, result.geologicalBoundaryRegimes.buffer, result.subductionPolarities.buffer, result.plateScaleClasses.buffer, result.plateContinentalFractions.buffer, result.plateTransitionalFractions.buffer, result.plateOceanicFractions.buffer, result.plateMeanCrustAgeMyr.buffer, result.plateMeanCrustThicknessKm.buffer]); return;
    }
    if (command.type === 'generate-lithosphere') {
      const result = await generateLithosphere(command);
      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-lithosphere', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.boundarySamples.buffer, result.boundaryKinds.buffer, result.crustKind.buffer, result.geologicalBoundaryRegimes.buffer, result.orogenicHistory.buffer, result.riftHistory.buffer, result.ridgeHistory.buffer, result.subductionHistory.buffer, result.transformHistory.buffer, result.crustalStrain.buffer, result.strengthIndex.buffer, result.weaknessIndex.buffer, result.effectiveElasticThicknessKm.buffer, result.thermalAnomalyIndex.buffer, result.mantleUpwellingIndex.buffer, result.mantleDynamicSupportIndex.buffer, result.compensatedBuoyancyIndex.buffer, result.structuralFabricStrength.buffer, result.structuralZoneKind.buffer, result.fragmentationPropensity.buffer, result.fragmentIds.buffer, result.kinematicDomainIds.buffer, result.fragmentParentPlateIds.buffer, result.fragmentKinds.buffer, result.fragmentSeedSamples.buffer, result.fragmentAreaSteradians.buffer, result.fragmentAreaFractionsOfParent.buffer, result.fragmentMeanWeakness.buffer, result.fragmentMeanPropensity.buffer, result.fragmentAngularVelocitiesRadPerMyr.buffer]); return;
    }
    if (command.type === 'generate-inheritance') {
      const result = await generateInheritance(command);
      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-inheritance', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.nearestCoarseSource.buffer, result.inheritedSampleMask.buffer, result.plateIds.buffer, result.crustKind.buffer, result.crustProvinceId.buffer, result.crustAgeMyr.buffer, result.crustThicknessKm.buffer, result.crustDensityKgPerM3.buffer, result.buoyancyIndex.buffer, result.orogenicHistory.buffer, result.riftHistory.buffer, result.ridgeHistory.buffer, result.subductionHistory.buffer, result.trenchHistory.buffer, result.volcanicArcHistory.buffer, result.transformHistory.buffer, result.subsidenceHistory.buffer, result.basinPotential.buffer, result.crustalStrain.buffer, result.strengthIndex.buffer, result.weaknessIndex.buffer, result.effectiveElasticThicknessKm.buffer, result.thermalAnomalyIndex.buffer, result.mantleUpwellingIndex.buffer, result.mantleDynamicSupportIndex.buffer, result.compensatedBuoyancyIndex.buffer, result.structuralFabricStrength.buffer, result.structuralZoneKind.buffer, result.fragmentationPropensity.buffer, result.fragmentIds.buffer, result.kinematicDomainIds.buffer, result.boundarySamples.buffer, result.boundaryKinds.buffer, result.geologicalBoundaryRegimes.buffer, result.subductionPolarities.buffer, result.boundaryNormalRatesMPerYear.buffer, result.boundaryShearRatesMPerYear.buffer, result.boundaryCoarseSourceIndices.buffer]); return;
    }
    if (command.type === 'generate-topography') {
      const result = await generateTopography(command);
      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-topography', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.crustKind.buffer, result.nearestCoarseSource.buffer, result.inheritedSampleMask.buffer, result.crustAgeMyr.buffer, result.crustThicknessKm.buffer, result.orogenicHistory.buffer, result.ridgeHistory.buffer, result.trenchHistory.buffer, result.strengthIndex.buffer, result.weaknessIndex.buffer, result.mantleDynamicSupportIndex.buffer, result.structuralZoneKind.buffer, result.fragmentationPropensity.buffer, result.kinematicDomainIds.buffer, result.boundarySamples.buffer, result.boundaryKinds.buffer, result.geologicalBoundaryRegimes.buffer, result.boundaryCoarseSourceIndices.buffer, result.isostaticElevationM.buffer, result.thermalElevationM.buffer, result.orogenicElevationM.buffer, result.ridgeElevationM.buffer, result.riftBasinElevationM.buffer, result.trenchElevationM.buffer, result.arcElevationM.buffer, result.mantleDynamicElevationM.buffer, result.solidElevationM.buffer, result.elevationAboveSeaLevelM.buffer, result.waterDepthM.buffer, result.submergedMask.buffer]); return;
    }
    throw new Error(`Unsupported worldgen command '${String((command as { type?: unknown }).type)}'.`);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command?.requestId ?? -1, type: 'error', payload: { message } });
  }
});
