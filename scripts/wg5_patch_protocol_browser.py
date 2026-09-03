from pathlib import Path


def require_replace(text: str, old: str, new: str, label: str) -> str:
    if old not in text:
        raise SystemExit(f'{label} marker not found')
    return text.replace(old, new, 1)

# Core engine + WASM protocol version move together.
lib = Path('rust/interlink-worldgen/src/lib.rs')
text = lib.read_text()
text = require_replace(text, 'pub const WORLDGEN_ENGINE_VERSION: u32 = 7;', 'pub const WORLDGEN_ENGINE_VERSION: u32 = 8;', 'engine version')
lib.write_text(text)

wasm_lib = Path('rust/interlink-worldgen-wasm/src/lib.rs')
text = wasm_lib.read_text()
text = require_replace(text, 'pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 7;', 'pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 8;', 'wasm protocol version')
wasm_lib.write_text(text)

# Browser protocol.
protocol = Path('src/worldgen/protocol.ts')
text = protocol.read_text()
text = require_replace(text, 'export const WORLDGEN_PROTOCOL_VERSION = 7;', 'export const WORLDGEN_PROTOCOL_VERSION = 8;', 'browser protocol version')
text = require_replace(
    text,
    'export const WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL = 7;\n',
    'export const WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL = 7;\nexport const WORLDGEN_CLIMATE_COARSE_MAX_LEVEL = 6;\nexport const WORLDGEN_CLIMATE_FINE_MAX_LEVEL = 7;\n',
    'climate level constants',
)
text = require_replace(
    text,
    'export interface WorldgenTopographyRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }\n',
    'export interface WorldgenTopographyRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }\nexport interface WorldgenClimateRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }\n',
    'climate request',
)
climate_types = r'''
export interface WorldgenClimateMetrics {
  coarseSampleCount: number;
  fineSampleCount: number;
  plateCount: number;
  fineBoundaryEdgeCount: number;
  orbitalPhaseCount: number;
  spinupYears: number;
  meanTemperatureK: number;
  minimumTemperatureK: number;
  maximumTemperatureK: number;
  meanLandTemperatureK: number;
  meanOceanTemperatureK: number;
  meanWindSpeedMS: number;
  maximumWindSpeedMS: number;
  meanSurfaceCurrentMS: number;
  maximumSurfaceCurrentMS: number;
  oceanDivergenceResidualMS: number;
  meanSeaSurfaceTemperatureK: number;
  meanAnnualPrecipitationMm: number;
  p95AnnualPrecipitationMm: number;
  globalEvaporationKg: number;
  globalPrecipitationKg: number;
  moistureBudgetRelativeError: number;
  persistentSnowAreaFraction: number;
  seaIceAreaFraction: number;
  finalTemperatureRmsChangeK: number;
  hasSeaLevel: boolean;
  seaLevelM: number;
  landAreaFraction: number;
  oceanAreaFraction: number;
  minimumSolidElevationM: number;
  maximumSolidElevationM: number;
  coarseTopologyHash: string;
  fineTopologyHash: string;
  tectonicHash: string;
  geologyHash: string;
  lithosphereHash: string;
  inheritanceHash: string;
  boundaryHash: string;
  planetParameterHash: string;
  topographyHash: string;
  climatePhysicalParameterHash: string;
  climateModelParameterHash: string;
  climateHash: string;
}

export interface WorldgenClimatePlanetProfile {
  radiusM: number;
  surfaceGravityMS2: number;
  rotationPeriodS: number;
  axialTiltRad: number;
  orbitalPeriodS: number;
  stellarFluxWM2: number;
  referenceSurfacePressurePa: number;
  surfaceWaterMassKg: number;
  equivalentGlobalWaterDepthM: number;
  internalHeatFluxWPerM2: number;
}

export interface WorldgenClimatePhysicalProfile {
  orbitalEccentricity: number;
  longitudeOfPeriapsisRad: number;
  atmosphericMeanMolarMassKgPerMol: number;
  atmosphericSpecificHeatJPerKgK: number;
  atmosphericLongwaveOpticalDepth: number;
}

export interface WorldgenClimateResult {
  engineVersion: number;
  coarseLevel: number;
  fineLevel: number;
  stage: WorldgenStageMetadata;
  metrics: WorldgenClimateMetrics;
  planet: WorldgenClimatePlanetProfile;
  climatePhysical: WorldgenClimatePhysicalProfile;
  positions: Float64Array;
  faces: Uint32Array;
  neighborOffsets: Uint32Array;
  neighbors: Uint32Array;
  plateIds: Uint16Array;
  crustKind: Uint8Array;
  nearestCoarseSource: Uint32Array;
  inheritedSampleMask: Uint8Array;
  crustAgeMyr: Float32Array;
  crustThicknessKm: Float32Array;
  orogenicHistory: Float32Array;
  ridgeHistory: Float32Array;
  trenchHistory: Float32Array;
  strengthIndex: Float32Array;
  weaknessIndex: Float32Array;
  mantleDynamicSupportIndex: Float32Array;
  structuralZoneKind: Uint8Array;
  fragmentationPropensity: Float32Array;
  kinematicDomainIds: Uint16Array;
  boundarySamples: Uint32Array;
  boundaryKinds: Uint8Array;
  geologicalBoundaryRegimes: Uint8Array;
  boundaryCoarseSourceIndices: Uint32Array;
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
  annualMeanInsolationWM2: Float32Array;
  seasonalInsolationAmplitudeWM2: Float32Array;
  temperatureMeanK: Float32Array;
  temperatureAnnualCosK: Float32Array;
  temperatureAnnualSinK: Float32Array;
  temperatureMinK: Float32Array;
  temperatureMaxK: Float32Array;
  localPressurePa: Float32Array;
  windEastMeanMS: Float32Array;
  windNorthMeanMS: Float32Array;
  windEastAnnualCosMS: Float32Array;
  windEastAnnualSinMS: Float32Array;
  windNorthAnnualCosMS: Float32Array;
  windNorthAnnualSinMS: Float32Array;
  seaSurfaceTemperatureMeanK: Float32Array;
  currentEastMeanMS: Float32Array;
  currentNorthMeanMS: Float32Array;
  currentSpeedMeanMS: Float32Array;
  oceanHeatTransportIndex: Float32Array;
  specificHumidityMean: Float32Array;
  annualPrecipitationMm: Float32Array;
  precipitationSeasonality: Float32Array;
  potentialEvaporationMm: Float32Array;
  moistureBalanceMm: Float32Array;
  aridityIndex: Float32Array;
  snowfallFraction: Float32Array;
  persistentSnowPotential: Float32Array;
  seaIcePotential: Float32Array;
}

'''
marker = 'export interface WorldgenSyntheticCommand'
if climate_types.strip() not in text:
    if marker not in text:
        raise SystemExit('protocol command marker not found')
    text = text.replace(marker, climate_types + marker, 1)
text = require_replace(
    text,
    "export interface WorldgenTopographyCommand { protocolVersion: number; requestId: number; type: 'generate-topography'; payload: WorldgenTopographyRequest; }\nexport type WorldgenCommand = WorldgenSyntheticCommand | WorldgenTopologyCommand | WorldgenTectonicsCommand | WorldgenGeologyCommand | WorldgenLithosphereCommand | WorldgenInheritanceCommand | WorldgenTopographyCommand;",
    "export interface WorldgenTopographyCommand { protocolVersion: number; requestId: number; type: 'generate-topography'; payload: WorldgenTopographyRequest; }\nexport interface WorldgenClimateCommand { protocolVersion: number; requestId: number; type: 'generate-climate'; payload: WorldgenClimateRequest; }\nexport type WorldgenCommand = WorldgenSyntheticCommand | WorldgenTopologyCommand | WorldgenTectonicsCommand | WorldgenGeologyCommand | WorldgenLithosphereCommand | WorldgenInheritanceCommand | WorldgenTopographyCommand | WorldgenClimateCommand;",
    'climate command union',
)
text = require_replace(
    text,
    "export interface WorldgenGeneratedTopographyEvent { protocolVersion: number; requestId: number; type: 'generated-topography'; payload: WorldgenTopographyResult; }\nexport interface WorldgenErrorEvent",
    "export interface WorldgenGeneratedTopographyEvent { protocolVersion: number; requestId: number; type: 'generated-topography'; payload: WorldgenTopographyResult; }\nexport interface WorldgenGeneratedClimateEvent { protocolVersion: number; requestId: number; type: 'generated-climate'; payload: WorldgenClimateResult; }\nexport interface WorldgenErrorEvent",
    'climate generated event',
)
text = require_replace(
    text,
    'export type WorldgenEvent = WorldgenGeneratedSyntheticEvent | WorldgenGeneratedTopologyEvent | WorldgenGeneratedTectonicsEvent | WorldgenGeneratedGeologyEvent | WorldgenGeneratedLithosphereEvent | WorldgenGeneratedInheritanceEvent | WorldgenGeneratedTopographyEvent | WorldgenErrorEvent;',
    'export type WorldgenEvent = WorldgenGeneratedSyntheticEvent | WorldgenGeneratedTopologyEvent | WorldgenGeneratedTectonicsEvent | WorldgenGeneratedGeologyEvent | WorldgenGeneratedLithosphereEvent | WorldgenGeneratedInheritanceEvent | WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenErrorEvent;',
    'climate event union',
)
validate = r'''
export function validateClimateRequest(request: WorldgenClimateRequest): void {
  if (!request.seed.trim()) throw new Error('WG-5 climate seed must not be empty.');
  if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_CLIMATE_COARSE_MAX_LEVEL) throw new Error(`WG-5 coarse level must be an integer from 0 through ${WORLDGEN_CLIMATE_COARSE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_CLIMATE_FINE_MAX_LEVEL) throw new Error(`WG-5 fine level must be an integer from coarse level through ${WORLDGEN_CLIMATE_FINE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES) throw new Error(`WG-5 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
  const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
  if (request.plateCount > coarseSamples) throw new Error('WG-5 plate count cannot exceed coarse topology sample count.');
}

'''
command_marker = 'export function worldgenSyntheticCommand'
if validate.strip() not in text:
    if command_marker not in text:
        raise SystemExit('climate validator insertion marker not found')
    text = text.replace(command_marker, validate + command_marker, 1)
text += "\nexport function worldgenClimateCommand(requestId: number, payload: WorldgenClimateRequest): WorldgenClimateCommand { validateClimateRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-climate', payload }; }\n"
protocol.write_text(text)

# Client.
client = Path('src/worldgen/worldgenClient.ts')
text = client.read_text()
text = require_replace(text, '  validateGeologyRequest,\n', '  validateClimateRequest,\n  validateGeologyRequest,\n', 'client validator import')
text = require_replace(text, '  worldgenGeologyCommand,\n', '  worldgenClimateCommand,\n  worldgenGeologyCommand,\n', 'client command import')
text = require_replace(text, '  type WorldgenEvent,\n', '  type WorldgenClimateRequest,\n  type WorldgenClimateResult,\n  type WorldgenEvent,\n', 'client climate type imports')
text = require_replace(
    text,
    'type WorldgenResult = WorldgenSyntheticResult | WorldgenTopologyResult | WorldgenTectonicsResult | WorldgenGeologyResult | WorldgenLithosphereResult | WorldgenInheritanceResult | WorldgenTopographyResult;',
    'type WorldgenResult = WorldgenSyntheticResult | WorldgenTopologyResult | WorldgenTectonicsResult | WorldgenGeologyResult | WorldgenLithosphereResult | WorldgenInheritanceResult | WorldgenTopographyResult | WorldgenClimateResult;',
    'client result union',
)
text = require_replace(
    text,
    'type WorldgenRequestCommand = ReturnType<typeof worldgenSyntheticCommand> | ReturnType<typeof worldgenTopologyCommand> | ReturnType<typeof worldgenTectonicsCommand> | ReturnType<typeof worldgenGeologyCommand> | ReturnType<typeof worldgenLithosphereCommand> | ReturnType<typeof worldgenInheritanceCommand> | ReturnType<typeof worldgenTopographyCommand>;',
    'type WorldgenRequestCommand = ReturnType<typeof worldgenSyntheticCommand> | ReturnType<typeof worldgenTopologyCommand> | ReturnType<typeof worldgenTectonicsCommand> | ReturnType<typeof worldgenGeologyCommand> | ReturnType<typeof worldgenLithosphereCommand> | ReturnType<typeof worldgenInheritanceCommand> | ReturnType<typeof worldgenTopographyCommand> | ReturnType<typeof worldgenClimateCommand>;',
    'client command union',
)
text = require_replace(text, '  generateTopography(request: WorldgenTopographyRequest): Promise<WorldgenTopographyResult>;\n', '  generateTopography(request: WorldgenTopographyRequest): Promise<WorldgenTopographyResult>;\n  generateClimate(request: WorldgenClimateRequest): Promise<WorldgenClimateResult>;\n', 'client interface')
text = require_replace(text, '    generateTopography(input) { validateTopographyRequest(input); return request<WorldgenTopographyResult>(worldgenTopographyCommand(nextRequestId++, input)); },\n', '    generateTopography(input) { validateTopographyRequest(input); return request<WorldgenTopographyResult>(worldgenTopographyCommand(nextRequestId++, input)); },\n    generateClimate(input) { validateClimateRequest(input); return request<WorldgenClimateResult>(worldgenClimateCommand(nextRequestId++, input)); },\n', 'client method')
client.write_text(text)

# Worker.
worker = Path('src/worldgen/worldgenWorker.ts')
text = worker.read_text()
text = require_replace(text, '  validateGeologyRequest,\n', '  validateClimateRequest,\n  validateGeologyRequest,\n', 'worker validator import')
text = require_replace(text, '  type WorldgenCommand,\n', '  type WorldgenClimateResult,\n  type WorldgenCommand,\n', 'worker climate result import')
text = require_replace(text, '  type WorldgenGeneratedGeologyEvent,\n', '  type WorldgenGeneratedClimateEvent,\n  type WorldgenGeneratedGeologyEvent,\n', 'worker climate event import')
wasm_climate = r'''
interface WasmClimate {
  generator_version(): number; stage_id(): string; stage_version(): number; stage_seed_hex(): string;
  coarse_level(): number; fine_level(): number; coarse_sample_count(): number; fine_sample_count(): number; plate_count(): number; fine_boundary_edge_count(): number;
  climate_hash_hex(): string; climate_physical_parameter_hash_hex(): string; climate_model_parameter_hash_hex(): string; topography_hash_hex(): string; inheritance_hash_hex(): string; boundary_hash_hex(): string; planet_parameter_hash_hex(): string; coarse_topology_hash_hex(): string; fine_topology_hash_hex(): string; tectonic_hash_hex(): string; geology_hash_hex(): string; lithosphere_hash_hex(): string;
  orbital_phase_count(): number; spinup_years(): number; mean_temperature_k(): number; minimum_temperature_k(): number; maximum_temperature_k(): number; mean_land_temperature_k(): number; mean_ocean_temperature_k(): number; mean_wind_speed_m_s(): number; maximum_wind_speed_m_s(): number; mean_surface_current_m_s(): number; maximum_surface_current_m_s(): number; ocean_divergence_residual_m_s(): number; mean_sea_surface_temperature_k(): number; mean_annual_precipitation_mm(): number; p95_annual_precipitation_mm(): number; global_evaporation_kg(): number; global_precipitation_kg(): number; moisture_budget_relative_error(): number; persistent_snow_area_fraction(): number; sea_ice_area_fraction(): number; final_temperature_rms_change_k(): number;
  has_sea_level(): boolean; sea_level_m(): number; land_area_fraction(): number; ocean_area_fraction(): number; minimum_solid_elevation_m(): number; maximum_solid_elevation_m(): number;
  radius_m(): number; surface_gravity_m_s2(): number; rotation_period_s(): number; axial_tilt_rad(): number; orbital_period_s(): number; stellar_flux_w_m2(): number; reference_surface_pressure_pa(): number; surface_water_mass_kg(): number; equivalent_global_water_depth_m(): number; internal_heat_flux_w_per_m2(): number;
  orbital_eccentricity(): number; longitude_of_periapsis_rad(): number; atmospheric_mean_molar_mass_kg_per_mol(): number; atmospheric_specific_heat_j_per_kg_k(): number; atmospheric_longwave_optical_depth(): number;
  positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array;
  plate_ids(): Uint16Array; crust_kind(): Uint8Array; nearest_coarse_source(): Uint32Array; inherited_sample_mask(): Uint8Array; crust_age_myr(): Float32Array; crust_thickness_km(): Float32Array; orogenic_history(): Float32Array; ridge_history(): Float32Array; trench_history(): Float32Array; strength_index(): Float32Array; weakness_index(): Float32Array; mantle_dynamic_support_index(): Float32Array; structural_zone_kind(): Uint8Array; fragmentation_propensity(): Float32Array; kinematic_domain_ids(): Uint16Array; boundary_samples(): Uint32Array; boundary_kinds(): Uint8Array; geological_boundary_regimes(): Uint8Array; boundary_coarse_source_indices(): Uint32Array;
  isostatic_elevation_m(): Float32Array; thermal_elevation_m(): Float32Array; orogenic_elevation_m(): Float32Array; ridge_elevation_m(): Float32Array; rift_basin_elevation_m(): Float32Array; trench_elevation_m(): Float32Array; arc_elevation_m(): Float32Array; mantle_dynamic_elevation_m(): Float32Array; solid_elevation_m(): Float32Array; elevation_above_sea_level_m(): Float32Array; water_depth_m(): Float32Array; submerged_mask(): Uint8Array;
  annual_mean_insolation_w_m2(): Float32Array; seasonal_insolation_amplitude_w_m2(): Float32Array; temperature_mean_k(): Float32Array; temperature_annual_cos_k(): Float32Array; temperature_annual_sin_k(): Float32Array; temperature_min_k(): Float32Array; temperature_max_k(): Float32Array; local_pressure_pa(): Float32Array; wind_east_mean_m_s(): Float32Array; wind_north_mean_m_s(): Float32Array; wind_east_annual_cos_m_s(): Float32Array; wind_east_annual_sin_m_s(): Float32Array; wind_north_annual_cos_m_s(): Float32Array; wind_north_annual_sin_m_s(): Float32Array; sea_surface_temperature_mean_k(): Float32Array; current_east_mean_m_s(): Float32Array; current_north_mean_m_s(): Float32Array; current_speed_mean_m_s(): Float32Array; ocean_heat_transport_index(): Float32Array; specific_humidity_mean(): Float32Array; annual_precipitation_mm(): Float32Array; precipitation_seasonality(): Float32Array; potential_evaporation_mm(): Float32Array; moisture_balance_mm(): Float32Array; aridity_index(): Float32Array; snowfall_fraction(): Float32Array; persistent_snow_potential(): Float32Array; sea_ice_potential(): Float32Array;
  free(): void;
}
'''
module_marker = 'interface WorldgenWasmModule {'
if wasm_climate.strip() not in text:
    if module_marker not in text:
        raise SystemExit('worker wasm module marker not found')
    text = text.replace(module_marker, wasm_climate + module_marker, 1)
text = require_replace(text, '  WasmWorldgenTopography: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number) => WasmTopography;\n', '  WasmWorldgenTopography: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number) => WasmTopography;\n  WasmWorldgenClimate: new (seed: string, coarseLevel: number, fineLevel: number, plateCount: number) => WasmClimate;\n', 'worker wasm constructor')
text = require_replace(text, 'WorldgenGeneratedTopographyEvent | WorldgenErrorEvent', 'WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenErrorEvent', 'worker scope union')
climate_function = r'''
async function generateClimate(command: Extract<WorldgenCommand, { type: 'generate-climate' }>): Promise<WorldgenClimateResult> {
  validateClimateRequest(command.payload);
  const module = await loadWorldgenWasm();
  const startedAt = nowMs();
  const output = new module.WasmWorldgenClimate(command.payload.seed, command.payload.coarseLevel, command.payload.fineLevel, command.payload.plateCount);
  try {
    const positions = output.positions(); const faces = output.faces(); const neighborOffsets = output.neighbor_offsets(); const neighbors = output.neighbors();
    const plateIds = output.plate_ids(); const crustKind = output.crust_kind(); const nearestCoarseSource = output.nearest_coarse_source(); const inheritedSampleMask = output.inherited_sample_mask(); const crustAgeMyr = output.crust_age_myr(); const crustThicknessKm = output.crust_thickness_km(); const orogenicHistory = output.orogenic_history(); const ridgeHistory = output.ridge_history(); const trenchHistory = output.trench_history(); const strengthIndex = output.strength_index(); const weaknessIndex = output.weakness_index(); const mantleDynamicSupportIndex = output.mantle_dynamic_support_index(); const structuralZoneKind = output.structural_zone_kind(); const fragmentationPropensity = output.fragmentation_propensity(); const kinematicDomainIds = output.kinematic_domain_ids(); const boundarySamples = output.boundary_samples(); const boundaryKinds = output.boundary_kinds(); const geologicalBoundaryRegimes = output.geological_boundary_regimes(); const boundaryCoarseSourceIndices = output.boundary_coarse_source_indices();
    const isostaticElevationM = output.isostatic_elevation_m(); const thermalElevationM = output.thermal_elevation_m(); const orogenicElevationM = output.orogenic_elevation_m(); const ridgeElevationM = output.ridge_elevation_m(); const riftBasinElevationM = output.rift_basin_elevation_m(); const trenchElevationM = output.trench_elevation_m(); const arcElevationM = output.arc_elevation_m(); const mantleDynamicElevationM = output.mantle_dynamic_elevation_m(); const solidElevationM = output.solid_elevation_m(); const elevationAboveSeaLevelM = output.elevation_above_sea_level_m(); const waterDepthM = output.water_depth_m(); const submergedMask = output.submerged_mask();
    const annualMeanInsolationWM2 = output.annual_mean_insolation_w_m2(); const seasonalInsolationAmplitudeWM2 = output.seasonal_insolation_amplitude_w_m2(); const temperatureMeanK = output.temperature_mean_k(); const temperatureAnnualCosK = output.temperature_annual_cos_k(); const temperatureAnnualSinK = output.temperature_annual_sin_k(); const temperatureMinK = output.temperature_min_k(); const temperatureMaxK = output.temperature_max_k(); const localPressurePa = output.local_pressure_pa(); const windEastMeanMS = output.wind_east_mean_m_s(); const windNorthMeanMS = output.wind_north_mean_m_s(); const windEastAnnualCosMS = output.wind_east_annual_cos_m_s(); const windEastAnnualSinMS = output.wind_east_annual_sin_m_s(); const windNorthAnnualCosMS = output.wind_north_annual_cos_m_s(); const windNorthAnnualSinMS = output.wind_north_annual_sin_m_s(); const seaSurfaceTemperatureMeanK = output.sea_surface_temperature_mean_k(); const currentEastMeanMS = output.current_east_mean_m_s(); const currentNorthMeanMS = output.current_north_mean_m_s(); const currentSpeedMeanMS = output.current_speed_mean_m_s(); const oceanHeatTransportIndex = output.ocean_heat_transport_index(); const specificHumidityMean = output.specific_humidity_mean(); const annualPrecipitationMm = output.annual_precipitation_mm(); const precipitationSeasonality = output.precipitation_seasonality(); const potentialEvaporationMm = output.potential_evaporation_mm(); const moistureBalanceMm = output.moisture_balance_mm(); const aridityIndex = output.aridity_index(); const snowfallFraction = output.snowfall_fraction(); const persistentSnowPotential = output.persistent_snow_potential(); const seaIcePotential = output.sea_ice_potential();
    return {
      engineVersion: output.generator_version(), coarseLevel: output.coarse_level(), fineLevel: output.fine_level(),
      stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) },
      metrics: {
        coarseSampleCount: output.coarse_sample_count(), fineSampleCount: output.fine_sample_count(), plateCount: output.plate_count(), fineBoundaryEdgeCount: output.fine_boundary_edge_count(), orbitalPhaseCount: output.orbital_phase_count(), spinupYears: output.spinup_years(),
        meanTemperatureK: output.mean_temperature_k(), minimumTemperatureK: output.minimum_temperature_k(), maximumTemperatureK: output.maximum_temperature_k(), meanLandTemperatureK: output.mean_land_temperature_k(), meanOceanTemperatureK: output.mean_ocean_temperature_k(), meanWindSpeedMS: output.mean_wind_speed_m_s(), maximumWindSpeedMS: output.maximum_wind_speed_m_s(), meanSurfaceCurrentMS: output.mean_surface_current_m_s(), maximumSurfaceCurrentMS: output.maximum_surface_current_m_s(), oceanDivergenceResidualMS: output.ocean_divergence_residual_m_s(), meanSeaSurfaceTemperatureK: output.mean_sea_surface_temperature_k(), meanAnnualPrecipitationMm: output.mean_annual_precipitation_mm(), p95AnnualPrecipitationMm: output.p95_annual_precipitation_mm(), globalEvaporationKg: output.global_evaporation_kg(), globalPrecipitationKg: output.global_precipitation_kg(), moistureBudgetRelativeError: output.moisture_budget_relative_error(), persistentSnowAreaFraction: output.persistent_snow_area_fraction(), seaIceAreaFraction: output.sea_ice_area_fraction(), finalTemperatureRmsChangeK: output.final_temperature_rms_change_k(),
        hasSeaLevel: output.has_sea_level(), seaLevelM: output.sea_level_m(), landAreaFraction: output.land_area_fraction(), oceanAreaFraction: output.ocean_area_fraction(), minimumSolidElevationM: output.minimum_solid_elevation_m(), maximumSolidElevationM: output.maximum_solid_elevation_m(),
        coarseTopologyHash: output.coarse_topology_hash_hex(), fineTopologyHash: output.fine_topology_hash_hex(), tectonicHash: output.tectonic_hash_hex(), geologyHash: output.geology_hash_hex(), lithosphereHash: output.lithosphere_hash_hex(), inheritanceHash: output.inheritance_hash_hex(), boundaryHash: output.boundary_hash_hex(), planetParameterHash: output.planet_parameter_hash_hex(), topographyHash: output.topography_hash_hex(), climatePhysicalParameterHash: output.climate_physical_parameter_hash_hex(), climateModelParameterHash: output.climate_model_parameter_hash_hex(), climateHash: output.climate_hash_hex(),
      },
      planet: { radiusM: output.radius_m(), surfaceGravityMS2: output.surface_gravity_m_s2(), rotationPeriodS: output.rotation_period_s(), axialTiltRad: output.axial_tilt_rad(), orbitalPeriodS: output.orbital_period_s(), stellarFluxWM2: output.stellar_flux_w_m2(), referenceSurfacePressurePa: output.reference_surface_pressure_pa(), surfaceWaterMassKg: output.surface_water_mass_kg(), equivalentGlobalWaterDepthM: output.equivalent_global_water_depth_m(), internalHeatFluxWPerM2: output.internal_heat_flux_w_per_m2() },
      climatePhysical: { orbitalEccentricity: output.orbital_eccentricity(), longitudeOfPeriapsisRad: output.longitude_of_periapsis_rad(), atmosphericMeanMolarMassKgPerMol: output.atmospheric_mean_molar_mass_kg_per_mol(), atmosphericSpecificHeatJPerKgK: output.atmospheric_specific_heat_j_per_kg_k(), atmosphericLongwaveOpticalDepth: output.atmospheric_longwave_optical_depth() },
      positions, faces, neighborOffsets, neighbors, plateIds, crustKind, nearestCoarseSource, inheritedSampleMask, crustAgeMyr, crustThicknessKm, orogenicHistory, ridgeHistory, trenchHistory, strengthIndex, weaknessIndex, mantleDynamicSupportIndex, structuralZoneKind, fragmentationPropensity, kinematicDomainIds, boundarySamples, boundaryKinds, geologicalBoundaryRegimes, boundaryCoarseSourceIndices,
      isostaticElevationM, thermalElevationM, orogenicElevationM, ridgeElevationM, riftBasinElevationM, trenchElevationM, arcElevationM, mantleDynamicElevationM, solidElevationM, elevationAboveSeaLevelM, waterDepthM, submergedMask,
      annualMeanInsolationWM2, seasonalInsolationAmplitudeWM2, temperatureMeanK, temperatureAnnualCosK, temperatureAnnualSinK, temperatureMinK, temperatureMaxK, localPressurePa, windEastMeanMS, windNorthMeanMS, windEastAnnualCosMS, windEastAnnualSinMS, windNorthAnnualCosMS, windNorthAnnualSinMS, seaSurfaceTemperatureMeanK, currentEastMeanMS, currentNorthMeanMS, currentSpeedMeanMS, oceanHeatTransportIndex, specificHumidityMean, annualPrecipitationMm, precipitationSeasonality, potentialEvaporationMm, moistureBalanceMm, aridityIndex, snowfallFraction, persistentSnowPotential, seaIcePotential,
    };
  } finally { output.free(); }
}

'''
listener_marker = "workerScope.addEventListener('message'"
if climate_function.strip() not in text:
    if listener_marker not in text:
        raise SystemExit('worker listener marker not found')
    text = text.replace(listener_marker, climate_function + listener_marker, 1)
clause_marker = "    if (command.type === 'generate-topography') {\n      const result = await generateTopography(command);"
if clause_marker not in text:
    raise SystemExit('worker topography handler marker not found')
# Put climate handler after complete topography handler by locating its return block.
topo_end = "      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-topography', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.crustKind.buffer, result.nearestCoarseSource.buffer, result.inheritedSampleMask.buffer, result.crustAgeMyr.buffer, result.crustThicknessKm.buffer, result.orogenicHistory.buffer, result.ridgeHistory.buffer, result.trenchHistory.buffer, result.strengthIndex.buffer, result.weaknessIndex.buffer, result.mantleDynamicSupportIndex.buffer, result.structuralZoneKind.buffer, result.fragmentationPropensity.buffer, result.kinematicDomainIds.buffer, result.boundarySamples.buffer, result.boundaryKinds.buffer, result.geologicalBoundaryRegimes.buffer, result.boundaryCoarseSourceIndices.buffer, result.isostaticElevationM.buffer, result.thermalElevationM.buffer, result.orogenicElevationM.buffer, result.ridgeElevationM.buffer, result.riftBasinElevationM.buffer, result.trenchElevationM.buffer, result.arcElevationM.buffer, result.mantleDynamicElevationM.buffer, result.solidElevationM.buffer, result.elevationAboveSeaLevelM.buffer, result.waterDepthM.buffer, result.submergedMask.buffer]); return;\n    }"
if topo_end not in text:
    raise SystemExit('worker topography transfer marker not found')
climate_buffers = '[result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.crustKind.buffer, result.nearestCoarseSource.buffer, result.inheritedSampleMask.buffer, result.crustAgeMyr.buffer, result.crustThicknessKm.buffer, result.orogenicHistory.buffer, result.ridgeHistory.buffer, result.trenchHistory.buffer, result.strengthIndex.buffer, result.weaknessIndex.buffer, result.mantleDynamicSupportIndex.buffer, result.structuralZoneKind.buffer, result.fragmentationPropensity.buffer, result.kinematicDomainIds.buffer, result.boundarySamples.buffer, result.boundaryKinds.buffer, result.geologicalBoundaryRegimes.buffer, result.boundaryCoarseSourceIndices.buffer, result.isostaticElevationM.buffer, result.thermalElevationM.buffer, result.orogenicElevationM.buffer, result.ridgeElevationM.buffer, result.riftBasinElevationM.buffer, result.trenchElevationM.buffer, result.arcElevationM.buffer, result.mantleDynamicElevationM.buffer, result.solidElevationM.buffer, result.elevationAboveSeaLevelM.buffer, result.waterDepthM.buffer, result.submergedMask.buffer, result.annualMeanInsolationWM2.buffer, result.seasonalInsolationAmplitudeWM2.buffer, result.temperatureMeanK.buffer, result.temperatureAnnualCosK.buffer, result.temperatureAnnualSinK.buffer, result.temperatureMinK.buffer, result.temperatureMaxK.buffer, result.localPressurePa.buffer, result.windEastMeanMS.buffer, result.windNorthMeanMS.buffer, result.windEastAnnualCosMS.buffer, result.windEastAnnualSinMS.buffer, result.windNorthAnnualCosMS.buffer, result.windNorthAnnualSinMS.buffer, result.seaSurfaceTemperatureMeanK.buffer, result.currentEastMeanMS.buffer, result.currentNorthMeanMS.buffer, result.currentSpeedMeanMS.buffer, result.oceanHeatTransportIndex.buffer, result.specificHumidityMean.buffer, result.annualPrecipitationMm.buffer, result.precipitationSeasonality.buffer, result.potentialEvaporationMm.buffer, result.moistureBalanceMm.buffer, result.aridityIndex.buffer, result.snowfallFraction.buffer, result.persistentSnowPotential.buffer, result.seaIcePotential.buffer]'
climate_handler = "\n    if (command.type === 'generate-climate') {\n      const result = await generateClimate(command);\n      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-climate', payload: result }, " + climate_buffers + "); return;\n    }"
text = text.replace(topo_end, topo_end + climate_handler, 1)
worker.write_text(text)
