from pathlib import Path
import re


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    if old not in text:
        raise SystemExit(f"marker not found in {path}: {old[:180]!r}")
    write(path, text.replace(old, new, 1))


# Protocol v12: the cumulative climate transport now also carries accepted
# WG-6A topology and WG-6B annual water-balance/discharge fields.
replace_once("src/worldgen/protocol.ts", "export const WORLDGEN_PROTOCOL_VERSION = 11;", "export const WORLDGEN_PROTOCOL_VERSION = 12;")

replace_once(
    "src/worldgen/protocol.ts",
    "  seaIcePotential: Float32Array;\n}\n\n\nexport interface WorldgenDrainageMetrics",
    """  seaIcePotential: Float32Array;
  drainageStage: WorldgenStageMetadata;
  drainageMetrics: WorldgenDrainageMetrics;
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
  runoffStage: WorldgenStageMetadata;
  runoffMetrics: WorldgenRunoffMetrics;
  actualEvapotranspirationMm: Float32Array;
  localRunoffMm: Float32Array;
  runoffFraction: Float32Array;
  localRunoffM3S: Float32Array;
  potentialDischargeM3S: Float32Array;
}


export interface WorldgenDrainageMetrics""",
)

replace_once(
    "src/worldgen/protocol.ts",
    "export interface WorldgenDrainageResult {",
    """export interface WorldgenRunoffMetrics {
  sampleCount: number;
  landSampleCount: number;
  landAreaM2: number;
  meanLandPrecipitationMm: number;
  meanLandActualEvapotranspirationMm: number;
  meanLandRunoffMm: number;
  maximumLandRunoffMm: number;
  landRunoffFraction: number;
  totalLocalRunoffM3S: number;
  terminalDischargeM3S: number;
  dischargeConservationRelativeError: number;
  maximumPotentialDischargeM3S: number;
  runoffParameterHash: string;
  climateHash: string;
  drainageHash: string;
  runoffHash: string;
}

export interface WorldgenDrainageResult {""",
)

# Extend the cumulative WASM interface.
wasm_climate_extra = """
  drainage_stage_id(): string; drainage_stage_version(): number; drainage_stage_seed_hex(): string; drainage_hash_hex(): string;
  drainage_land_sample_count(): number; drainage_ocean_sample_count(): number; drainage_basin_count(): number; drainage_depression_count(): number; drainage_depression_sample_count(): number;
  drainage_land_area_m2(): number; terminal_contributing_area_m2(): number; drainage_area_conservation_relative_error(): number; maximum_contributing_area_m2(): number; maximum_depression_depth_m(): number;
  receiver(): Uint32Array; outlet_sample(): Uint32Array; outlet_kind(): Uint8Array; basin_id(): Uint32Array; depression_id(): Uint32Array; hydrologic_escape_elevation_m(): Float32Array; depression_depth_m(): Float32Array; contributing_area_m2(): Float64Array; drainage_order(): Uint32Array;
  basin_outlet_samples(): Uint32Array; basin_outlet_kinds(): Uint8Array; basin_areas_m2(): Float64Array; depression_floor_samples(): Uint32Array; depression_floor_elevations_m(): Float64Array; depression_spill_elevations_m(): Float64Array; depression_areas_m2(): Float64Array;
  runoff_stage_id(): string; runoff_stage_version(): number; runoff_stage_seed_hex(): string; runoff_hash_hex(): string; runoff_parameter_hash_hex(): string; runoff_climate_hash_hex(): string; runoff_drainage_hash_hex(): string;
  mean_land_runoff_precipitation_mm(): number; mean_land_actual_evapotranspiration_mm(): number; mean_land_runoff_mm(): number; maximum_land_runoff_mm(): number; land_runoff_fraction(): number; total_local_runoff_m3_s(): number; terminal_discharge_m3_s(): number; discharge_conservation_relative_error(): number; maximum_potential_discharge_m3_s(): number;
  actual_evapotranspiration_mm(): Float32Array; local_runoff_mm(): Float32Array; runoff_fraction(): Float32Array; local_runoff_m3_s(): Float32Array; potential_discharge_m3_s(): Float32Array;
"""
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "  annual_mean_insolation_w_m2(): Float32Array; seasonal_insolation_amplitude_w_m2(): Float32Array; temperature_mean_k(): Float32Array; temperature_annual_cos_k(): Float32Array; temperature_annual_sin_k(): Float32Array; temperature_min_k(): Float32Array; temperature_max_k(): Float32Array; local_pressure_pa(): Float32Array; wind_east_mean_m_s(): Float32Array; wind_north_mean_m_s(): Float32Array; wind_east_annual_cos_m_s(): Float32Array; wind_east_annual_sin_m_s(): Float32Array; wind_north_annual_cos_m_s(): Float32Array; wind_north_annual_sin_m_s(): Float32Array; sea_surface_temperature_mean_k(): Float32Array; sea_surface_temperature_annual_cos_k(): Float32Array; sea_surface_temperature_annual_sin_k(): Float32Array; current_east_mean_m_s(): Float32Array; current_north_mean_m_s(): Float32Array; current_east_annual_cos_m_s(): Float32Array; current_east_annual_sin_m_s(): Float32Array; current_north_annual_cos_m_s(): Float32Array; current_north_annual_sin_m_s(): Float32Array; current_speed_mean_m_s(): Float32Array; ocean_heat_transport_index(): Float32Array; specific_humidity_mean(): Float32Array; annual_precipitation_mm(): Float32Array; precipitation_phase_rate_mm_year(): Float32Array; precipitation_seasonality(): Float32Array; potential_evaporation_mm(): Float32Array; moisture_balance_mm(): Float32Array; aridity_index(): Float32Array; snowfall_fraction(): Float32Array; persistent_snow_potential(): Float32Array; sea_ice_potential(): Float32Array;\n  free(): void;",
    "  annual_mean_insolation_w_m2(): Float32Array; seasonal_insolation_amplitude_w_m2(): Float32Array; temperature_mean_k(): Float32Array; temperature_annual_cos_k(): Float32Array; temperature_annual_sin_k(): Float32Array; temperature_min_k(): Float32Array; temperature_max_k(): Float32Array; local_pressure_pa(): Float32Array; wind_east_mean_m_s(): Float32Array; wind_north_mean_m_s(): Float32Array; wind_east_annual_cos_m_s(): Float32Array; wind_east_annual_sin_m_s(): Float32Array; wind_north_annual_cos_m_s(): Float32Array; wind_north_annual_sin_m_s(): Float32Array; sea_surface_temperature_mean_k(): Float32Array; sea_surface_temperature_annual_cos_k(): Float32Array; sea_surface_temperature_annual_sin_k(): Float32Array; current_east_mean_m_s(): Float32Array; current_north_mean_m_s(): Float32Array; current_east_annual_cos_m_s(): Float32Array; current_east_annual_sin_m_s(): Float32Array; current_north_annual_cos_m_s(): Float32Array; current_north_annual_sin_m_s(): Float32Array; current_speed_mean_m_s(): Float32Array; ocean_heat_transport_index(): Float32Array; specific_humidity_mean(): Float32Array; annual_precipitation_mm(): Float32Array; precipitation_phase_rate_mm_year(): Float32Array; precipitation_seasonality(): Float32Array; potential_evaporation_mm(): Float32Array; moisture_balance_mm(): Float32Array; aridity_index(): Float32Array; snowfall_fraction(): Float32Array; persistent_snow_potential(): Float32Array; sea_ice_potential(): Float32Array;\n" + wasm_climate_extra + "  free(): void;",
)

# Package WG-6A/WG-6B fields from the same cumulative WASM object.
replace_once("src/worldgen/worldgenWorker.ts", "    progress('packaging', 9, 10, 0, 1);", "    progress('packaging', 11, 12, 0, 1);")
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "    const annualMeanInsolationWM2 = output.annual_mean_insolation_w_m2(); const seasonalInsolationAmplitudeWM2 = output.seasonal_insolation_amplitude_w_m2(); const temperatureMeanK = output.temperature_mean_k(); const temperatureAnnualCosK = output.temperature_annual_cos_k(); const temperatureAnnualSinK = output.temperature_annual_sin_k(); const temperatureMinK = output.temperature_min_k(); const temperatureMaxK = output.temperature_max_k(); const localPressurePa = output.local_pressure_pa(); const windEastMeanMS = output.wind_east_mean_m_s(); const windNorthMeanMS = output.wind_north_mean_m_s(); const windEastAnnualCosMS = output.wind_east_annual_cos_m_s(); const windEastAnnualSinMS = output.wind_east_annual_sin_m_s(); const windNorthAnnualCosMS = output.wind_north_annual_cos_m_s(); const windNorthAnnualSinMS = output.wind_north_annual_sin_m_s(); const seaSurfaceTemperatureMeanK = output.sea_surface_temperature_mean_k(); const seaSurfaceTemperatureAnnualCosK = output.sea_surface_temperature_annual_cos_k(); const seaSurfaceTemperatureAnnualSinK = output.sea_surface_temperature_annual_sin_k(); const currentEastMeanMS = output.current_east_mean_m_s(); const currentNorthMeanMS = output.current_north_mean_m_s(); const currentEastAnnualCosMS = output.current_east_annual_cos_m_s(); const currentEastAnnualSinMS = output.current_east_annual_sin_m_s(); const currentNorthAnnualCosMS = output.current_north_annual_cos_m_s(); const currentNorthAnnualSinMS = output.current_north_annual_sin_m_s(); const currentSpeedMeanMS = output.current_speed_mean_m_s(); const oceanHeatTransportIndex = output.ocean_heat_transport_index(); const specificHumidityMean = output.specific_humidity_mean(); const annualPrecipitationMm = output.annual_precipitation_mm(); const precipitationPhaseRateMmYear = output.precipitation_phase_rate_mm_year(); const precipitationSeasonality = output.precipitation_seasonality(); const potentialEvaporationMm = output.potential_evaporation_mm(); const moistureBalanceMm = output.moisture_balance_mm(); const aridityIndex = output.aridity_index(); const snowfallFraction = output.snowfall_fraction(); const persistentSnowPotential = output.persistent_snow_potential(); const seaIcePotential = output.sea_ice_potential();",
    """    const annualMeanInsolationWM2 = output.annual_mean_insolation_w_m2(); const seasonalInsolationAmplitudeWM2 = output.seasonal_insolation_amplitude_w_m2(); const temperatureMeanK = output.temperature_mean_k(); const temperatureAnnualCosK = output.temperature_annual_cos_k(); const temperatureAnnualSinK = output.temperature_annual_sin_k(); const temperatureMinK = output.temperature_min_k(); const temperatureMaxK = output.temperature_max_k(); const localPressurePa = output.local_pressure_pa(); const windEastMeanMS = output.wind_east_mean_m_s(); const windNorthMeanMS = output.wind_north_mean_m_s(); const windEastAnnualCosMS = output.wind_east_annual_cos_m_s(); const windEastAnnualSinMS = output.wind_east_annual_sin_m_s(); const windNorthAnnualCosMS = output.wind_north_annual_cos_m_s(); const windNorthAnnualSinMS = output.wind_north_annual_sin_m_s(); const seaSurfaceTemperatureMeanK = output.sea_surface_temperature_mean_k(); const seaSurfaceTemperatureAnnualCosK = output.sea_surface_temperature_annual_cos_k(); const seaSurfaceTemperatureAnnualSinK = output.sea_surface_temperature_annual_sin_k(); const currentEastMeanMS = output.current_east_mean_m_s(); const currentNorthMeanMS = output.current_north_mean_m_s(); const currentEastAnnualCosMS = output.current_east_annual_cos_m_s(); const currentEastAnnualSinMS = output.current_east_annual_sin_m_s(); const currentNorthAnnualCosMS = output.current_north_annual_cos_m_s(); const currentNorthAnnualSinMS = output.current_north_annual_sin_m_s(); const currentSpeedMeanMS = output.current_speed_mean_m_s(); const oceanHeatTransportIndex = output.ocean_heat_transport_index(); const specificHumidityMean = output.specific_humidity_mean(); const annualPrecipitationMm = output.annual_precipitation_mm(); const precipitationPhaseRateMmYear = output.precipitation_phase_rate_mm_year(); const precipitationSeasonality = output.precipitation_seasonality(); const potentialEvaporationMm = output.potential_evaporation_mm(); const moistureBalanceMm = output.moisture_balance_mm(); const aridityIndex = output.aridity_index(); const snowfallFraction = output.snowfall_fraction(); const persistentSnowPotential = output.persistent_snow_potential(); const seaIcePotential = output.sea_ice_potential();
    const receiver = output.receiver(); const outletSample = output.outlet_sample(); const outletKind = output.outlet_kind(); const basinId = output.basin_id(); const depressionId = output.depression_id(); const hydrologicEscapeElevationM = output.hydrologic_escape_elevation_m(); const depressionDepthM = output.depression_depth_m(); const contributingAreaM2 = output.contributing_area_m2(); const drainageOrder = output.drainage_order(); const basinOutletSamples = output.basin_outlet_samples(); const basinOutletKinds = output.basin_outlet_kinds(); const basinAreasM2 = output.basin_areas_m2(); const depressionFloorSamples = output.depression_floor_samples(); const depressionFloorElevationsM = output.depression_floor_elevations_m(); const depressionSpillElevationsM = output.depression_spill_elevations_m(); const depressionAreasM2 = output.depression_areas_m2();
    const actualEvapotranspirationMm = output.actual_evapotranspiration_mm(); const localRunoffMm = output.local_runoff_mm(); const runoffFraction = output.runoff_fraction(); const localRunoffM3S = output.local_runoff_m3_s(); const potentialDischargeM3S = output.potential_discharge_m3_s();""",
)
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "      annualMeanInsolationWM2, seasonalInsolationAmplitudeWM2, temperatureMeanK, temperatureAnnualCosK, temperatureAnnualSinK, temperatureMinK, temperatureMaxK, localPressurePa, windEastMeanMS, windNorthMeanMS, windEastAnnualCosMS, windEastAnnualSinMS, windNorthAnnualCosMS, windNorthAnnualSinMS, seaSurfaceTemperatureMeanK, seaSurfaceTemperatureAnnualCosK, seaSurfaceTemperatureAnnualSinK, currentEastMeanMS, currentNorthMeanMS, currentEastAnnualCosMS, currentEastAnnualSinMS, currentNorthAnnualCosMS, currentNorthAnnualSinMS, currentSpeedMeanMS, oceanHeatTransportIndex, specificHumidityMean, annualPrecipitationMm, precipitationPhaseRateMmYear, precipitationSeasonality, potentialEvaporationMm, moistureBalanceMm, aridityIndex, snowfallFraction, persistentSnowPotential, seaIcePotential,",
    """      annualMeanInsolationWM2, seasonalInsolationAmplitudeWM2, temperatureMeanK, temperatureAnnualCosK, temperatureAnnualSinK, temperatureMinK, temperatureMaxK, localPressurePa, windEastMeanMS, windNorthMeanMS, windEastAnnualCosMS, windEastAnnualSinMS, windNorthAnnualCosMS, windNorthAnnualSinMS, seaSurfaceTemperatureMeanK, seaSurfaceTemperatureAnnualCosK, seaSurfaceTemperatureAnnualSinK, currentEastMeanMS, currentNorthMeanMS, currentEastAnnualCosMS, currentEastAnnualSinMS, currentNorthAnnualCosMS, currentNorthAnnualSinMS, currentSpeedMeanMS, oceanHeatTransportIndex, specificHumidityMean, annualPrecipitationMm, precipitationPhaseRateMmYear, precipitationSeasonality, potentialEvaporationMm, moistureBalanceMm, aridityIndex, snowfallFraction, persistentSnowPotential, seaIcePotential,
      drainageStage: { id: output.drainage_stage_id(), version: output.drainage_stage_version(), stageSeed: output.drainage_stage_seed_hex(), durationMs: 0 },
      drainageMetrics: { sampleCount: output.fine_sample_count(), landSampleCount: output.drainage_land_sample_count(), oceanSampleCount: output.drainage_ocean_sample_count(), basinCount: output.drainage_basin_count(), depressionCount: output.drainage_depression_count(), depressionSampleCount: output.drainage_depression_sample_count(), landAreaM2: output.drainage_land_area_m2(), terminalContributingAreaM2: output.terminal_contributing_area_m2(), areaConservationRelativeError: output.drainage_area_conservation_relative_error(), maximumContributingAreaM2: output.maximum_contributing_area_m2(), maximumDepressionDepthM: output.maximum_depression_depth_m(), drainageHash: output.drainage_hash_hex() },
      receiver, outletSample, outletKind, basinId, depressionId, hydrologicEscapeElevationM, depressionDepthM, contributingAreaM2, drainageOrder, basinOutletSamples, basinOutletKinds, basinAreasM2, depressionFloorSamples, depressionFloorElevationsM, depressionSpillElevationsM, depressionAreasM2,
      runoffStage: { id: output.runoff_stage_id(), version: output.runoff_stage_version(), stageSeed: output.runoff_stage_seed_hex(), durationMs: 0 },
      runoffMetrics: { sampleCount: output.fine_sample_count(), landSampleCount: output.drainage_land_sample_count(), landAreaM2: output.drainage_land_area_m2(), meanLandPrecipitationMm: output.mean_land_runoff_precipitation_mm(), meanLandActualEvapotranspirationMm: output.mean_land_actual_evapotranspiration_mm(), meanLandRunoffMm: output.mean_land_runoff_mm(), maximumLandRunoffMm: output.maximum_land_runoff_mm(), landRunoffFraction: output.land_runoff_fraction(), totalLocalRunoffM3S: output.total_local_runoff_m3_s(), terminalDischargeM3S: output.terminal_discharge_m3_s(), dischargeConservationRelativeError: output.discharge_conservation_relative_error(), maximumPotentialDischargeM3S: output.maximum_potential_discharge_m3_s(), runoffParameterHash: output.runoff_parameter_hash_hex(), climateHash: output.runoff_climate_hash_hex(), drainageHash: output.runoff_drainage_hash_hex(), runoffHash: output.runoff_hash_hex() },
      actualEvapotranspirationMm, localRunoffMm, runoffFraction, localRunoffM3S, potentialDischargeM3S,""",
)
replace_once("src/worldgen/worldgenWorker.ts", "    progress('packaging', 9, 10, 1, 1);", "    progress('packaging', 11, 12, 1, 1);")

# Add WG-6A/WG-6B transfer buffers to the cumulative event.
replace_once(
    "src/worldgen/worldgenWorker.ts",
    "result.snowfallFraction.buffer, result.persistentSnowPotential.buffer, result.seaIcePotential.buffer]); return;",
    "result.snowfallFraction.buffer, result.persistentSnowPotential.buffer, result.seaIcePotential.buffer, result.receiver.buffer, result.outletSample.buffer, result.outletKind.buffer, result.basinId.buffer, result.depressionId.buffer, result.hydrologicEscapeElevationM.buffer, result.depressionDepthM.buffer, result.contributingAreaM2.buffer, result.drainageOrder.buffer, result.basinOutletSamples.buffer, result.basinOutletKinds.buffer, result.basinAreasM2.buffer, result.depressionFloorSamples.buffer, result.depressionFloorElevationsM.buffer, result.depressionSpillElevationsM.buffer, result.depressionAreasM2.buffer, result.actualEvapotranspirationMm.buffer, result.localRunoffMm.buffer, result.runoffFraction.buffer, result.localRunoffM3S.buffer, result.potentialDischargeM3S.buffer]); return;",
)

# Main Lab becomes single-request cumulative WG-6B UI.
lab = "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts"
replace_once(lab, "  type WorldgenDrainageResult,\n", "")
text = read(lab)
text = text.replace("WorldgenDrainageResult", "WorldgenClimateResult")
write(lab, text)

replace_once(
    lab,
    "const DRAINAGE_MODES = new Set([",
    """const RUNOFF_MODES = new Set(['annual-runoff', 'runoff-fraction', 'actual-et', 'potential-discharge']);
function isRunoffMode(mode: string): boolean { return RUNOFF_MODES.has(mode); }
function runoffSampleColor(result: WorldgenClimateResult, mode: string, sample: number): string {
  if (result.submergedMask[sample]) return '#102c43';
  if (mode === 'runoff-fraction') return drainageScalarColor(result.runoffFraction[sample]!, 48, 205);
  if (mode === 'actual-et') {
    const value = Math.max(0, result.actualEvapotranspirationMm[sample]!);
    return drainageScalarColor(value / (value + 850), 42, 168);
  }
  if (mode === 'annual-runoff') {
    const maxValue = Math.max(1, result.runoffMetrics.maximumLandRunoffMm);
    return drainageScalarColor(Math.log1p(Math.max(0, result.localRunoffMm[sample]!)) / Math.log1p(maxValue), 44, 218);
  }
  const maxValue = Math.max(1e-6, result.runoffMetrics.maximumPotentialDischargeM3S);
  return drainageScalarColor(Math.log1p(Math.max(0, result.potentialDischargeM3S[sample]!)) / Math.log1p(maxValue), 215, 18);
}

const DRAINAGE_MODES = new Set([""",
)

replace_once(
    lab,
    "function renderPlanet(canvas: HTMLCanvasElement, result: WorldgenClimateResult, drainage: WorldgenClimateResult | null, projection: string, mode: string, overlays: ReadonlySet<string>, phase: number, yaw: number, pitch: number, buffers: ProjectionBuffers, interactive: boolean, animation: number): void {",
    "function renderPlanet(canvas: HTMLCanvasElement, result: WorldgenClimateResult, projection: string, mode: string, overlays: ReadonlySet<string>, phase: number, yaw: number, pitch: number, buffers: ProjectionBuffers, interactive: boolean, animation: number): void {",
)
replace_once(
    lab,
    "  if (isDrainageMode(mode)) {\n    if (!drainage) return;\n    renderDrainageDiagnostic(context, drainage, projection, mode, width, buffers, interactive);",
    """  if (isRunoffMode(mode)) {
    const count = result.metrics.fineSampleCount;
    const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
    const fastPoints = interactive && count > 20_000;
    context.globalAlpha = 0.94;
    for (let sample = 0; sample < count; sample += 1) {
      if (!buffers.visible[sample]) continue;
      context.fillStyle = runoffSampleColor(result, mode, sample);
      const x = buffers.x[sample]!, y = buffers.y[sample]!;
      if (fastPoints) context.fillRect(x - 0.75, y - 0.75, 1.5, 1.5);
      else { context.beginPath(); context.arc(x, y, pointRadius, 0, TWO_PI); context.fill(); }
    }
    context.globalAlpha = 1;
    drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation);
    return;
  }
  if (isDrainageMode(mode)) {
    renderDrainageDiagnostic(context, result, projection, mode, width, buffers, interactive);""",
)
replace_once(lab, "let currentDrainage: WorldgenClimateResult | null = null;\n", "")
replace_once(
    lab,
    "  renderPlanet(canvas, current, currentDrainage, projection.value, visualization.value, selectedOverlays(), orbitalPhase(), yaw, pitch, buffers, interactive, animationPhase);",
    "  renderPlanet(canvas, current, projection.value, visualization.value, selectedOverlays(), orbitalPhase(), yaw, pitch, buffers, interactive, animationPhase);",
)
replace_once(lab, "function showMetrics(result: WorldgenClimateResult, drainage: WorldgenClimateResult): void {", "function showMetrics(result: WorldgenClimateResult): void {")
text = read(lab)
text = text.replace("drainage.stage.id", "result.drainageStage.id").replace("drainage.stage.version", "result.drainageStage.version")
text = text.replace("drainage.metrics.", "result.drainageMetrics.").replace("drainage.topographyHash === result.metrics.topographyHash", "result.runoffMetrics.climateHash === result.metrics.climateHash && result.runoffMetrics.drainageHash === result.drainageMetrics.drainageHash")
text = text.replace("metric(metrics, 'WG-4 surface identity', result.runoffMetrics.climateHash === result.metrics.climateHash && result.runoffMetrics.drainageHash === result.drainageMetrics.drainageHash ? 'Climate / drainage match' : 'MISMATCH');", "metric(metrics, 'Hydrology identity', result.runoffMetrics.climateHash === result.metrics.climateHash && result.runoffMetrics.drainageHash === result.drainageMetrics.drainageHash ? 'WG-5 / WG-6A / WG-6B match' : 'MISMATCH');\n  metric(metrics, 'WG-6B water balance', `${result.runoffMetrics.meanLandPrecipitationMm.toFixed(1)} P · ${result.runoffMetrics.meanLandActualEvapotranspirationMm.toFixed(1)} AET · ${result.runoffMetrics.meanLandRunoffMm.toFixed(1)} runoff mm/yr`);\n  metric(metrics, 'WG-6B runoff fraction', `${(result.runoffMetrics.landRunoffFraction * 100).toFixed(1)}% of land precipitation`);\n  metric(metrics, 'WG-6B max potential discharge', `${result.runoffMetrics.maximumPotentialDischargeM3S.toFixed(1)} m³/s`);\n  metric(metrics, 'WG-6B discharge closure', result.runoffMetrics.dischargeConservationRelativeError.toExponential(2));\n  metric(metrics, 'WG-6B runoff hash', result.runoffMetrics.runoffHash);")
write(lab, text)

# Replace the two-request generation block with a single cumulative call.
text = read(lab)
pattern = re.compile(r"  currentDrainage = null;\n  try \{\n    const request = \{ seed: seed\.value, coarseLevel: Number\(coarseLevel\.value\), fineLevel: Number\(fineLevel\.value\), plateCount: Number\(plates\.value\) \};\n    const loaded = await client\.generateClimate\(request, handleGenerationProgress\);\n    generationStage\.textContent = 'WG-6A drainage topology';[\s\S]*?status\.textContent = `Planet ready through WG-6A:[^`]*`;", re.M)
replacement = """  try {
    const request = { seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) };
    const loaded = await client.generateClimate(request, handleGenerationProgress);
    if (loaded.runoffMetrics.climateHash !== loaded.metrics.climateHash) throw new Error('WG-6B climate identity does not match accepted WG-5 forcing.');
    if (loaded.runoffMetrics.drainageHash !== loaded.drainageMetrics.drainageHash) throw new Error('WG-6B drainage identity does not match accepted WG-6A topology.');
    current = loaded;
    buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };
    styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
    edgeOverlayCache = { result: null, coastline: new Uint32Array(0), contours: [] };
    showMetrics(loaded); redraw(false); updateAnimation(); finishGenerationTelemetry(loaded);
    generationStep.textContent = `${loaded.metrics.spinupYears} climate spin-up years · ${loaded.drainageMetrics.basinCount.toLocaleString()} basins · ${loaded.runoffMetrics.meanLandRunoffMm.toFixed(1)} mm/yr land runoff`;
    generationTimer.textContent = formatDuration(performance.now() - generationStartedAt);
    status.textContent = `Planet ready through WG-6B: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.drainageMetrics.basinCount.toLocaleString()} drainage basins, max potential discharge ${loaded.runoffMetrics.maximumPotentialDischargeM3S.toFixed(1)} m³/s.`;"""
text, count = pattern.subn(replacement, text, count=1)
if count != 1:
    raise SystemExit("failed to replace cumulative main Lab generation block")
write(lab, text)

# Main HTML: WG-6B is the public frontier and only index.html remains public.
html = read("index.html")
html = html.replace("Through WG-6A", "Through WG-6B").replace("THROUGH WG-6A", "THROUGH WG-6B")
html = html.replace("through WG-6A", "through WG-6B").replace("interlink-wg6a", "interlink-wg6b")
runoff_group = '''          <optgroup label="Hydrology · Runoff / discharge (WG-6B)">
            <option value="potential-discharge">Potential annual discharge</option>
            <option value="annual-runoff">Annual runoff depth</option>
            <option value="runoff-fraction">Runoff fraction</option>
            <option value="actual-et">Actual evapotranspiration</option>
          </optgroup>
'''
marker = '          <optgroup label="Hydrology · Drainage topology (WG-6A)">'
if marker not in html:
    raise SystemExit("hydrology group marker missing from index.html")
html = html.replace(marker, runoff_group + marker, 1)
html = html.replace("<strong>Current physical frontier: WG-6A</strong>", "<strong>Current physical frontier: WG-6B</strong>")
html = html.replace("WG-5 coupled climate, and WG-6A drainage topology. Climate and drainage are required to resolve to the same deterministic WG-4 topography identity before the Lab accepts the result.", "WG-5 coupled climate, WG-6A drainage topology, and WG-6B annual runoff/discharge in one cumulative Rust/WASM result. WG-6B is required to reference the exact accepted WG-5 climate and WG-6A drainage identities.")
html = html.replace("WG-6A itself remains terrain-only and does not consume rainfall yet.", "WG-6A remains terrain-only; WG-6B consumes the accepted annual precipitation and PET forcing to produce actual evapotranspiration, runoff, and potential routed discharge.")
html = html.replace("Runoff and river discharge, lake water balance, river incision, sediment transport", "Lake water balance and spill activation, seasonal discharge, river incision, sediment transport")
write("index.html", html)

# Canonical Pages contract: index.html only.
pages = read("tests/pages.test.ts")
pages = re.sub(r"test\('GitHub Pages root serves the cumulative Planet Engine Lab through WG-6A',[\s\S]*?\n\}\);", """test('GitHub Pages root is the single cumulative Planet Engine Lab through WG-6B', () => {
  assert.ok(fs.existsSync('index.html'), 'Pages root requires index.html');
  assert.ok(!fs.existsSync('worldgen-lab.html'), 'secondary Lab HTML entrypoint must not exist');
  assert.ok(!fs.existsSync('drainage.html'), 'standalone drainage HTML entrypoint must not exist');
  assert.ok(fs.existsSync('styles/base.css'));
  assert.ok(fs.existsSync('styles/worldgenLab.css'));
  const html = fs.readFileSync('index.html', 'utf8');
  assert.match(html, /PLANET ENGINE LAB/);
  assert.match(html, /PLANET ENGINE · THROUGH WG-6B/);
  assert.match(html, /Potential annual discharge/);
  assert.match(html, /Contributing drainage area/);
  assert.match(html, /dist\\/worldgen\\/diagnostics\\/worldgenClimateLabStandalone\\.js/);
  assert.doesNotMatch(html, /Return to game/i);
});""", pages, count=1)
write("tests/pages.test.ts", pages)

# Update stable protocol/frontier assertions and add a dedicated WG-6B cumulative contract test.
for path in ["tests/wg4Topography.test.ts", "tests/wg5Climate.test.ts", "tests/wg6Drainage.test.ts", "tests/worldgenRewrite.test.ts"]:
    text = read(path)
    text = text.replace("protocol v11", "protocol v12").replace("Protocol v11", "Protocol v12")
    text = text.replace("WORLDGEN_PROTOCOL_VERSION, 11", "WORLDGEN_PROTOCOL_VERSION, 12")
    text = text.replace("const PROTOCOL = 11", "const PROTOCOL = 12")
    text = text.replace("THROUGH WG-6A", "THROUGH WG-6B")
    write(path, text)

wg6 = read("tests/wg6Drainage.test.ts")
wg6 = wg6.replace("  assert.match(source, /generateDrainage/);", "  assert.doesNotMatch(source, /client\\.generateDrainage\\(/, 'primary Lab must consume cumulative WG-6A/WG-6B output from one request');")
wg6 = wg6.replace("  assert.match(source, /currentDrainage/);", "  assert.match(source, /drainageMetrics/);")
write("tests/wg6Drainage.test.ts", wg6)

Path("tests/wg6Runoff.test.ts").write_text("""import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-6B cumulative browser contract is protocol v12 and single-request', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 12);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  for (const field of ['actualEvapotranspirationMm', 'localRunoffMm', 'runoffFraction', 'potentialDischargeM3S', 'runoffMetrics', 'drainageMetrics']) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  assert.match(lab, /potential-discharge/);
  assert.match(lab, /annual-runoff/);
  assert.match(lab, /runoff-fraction/);
  assert.match(lab, /actual-et/);
  assert.doesNotMatch(lab, /client\\.generateDrainage\\(/);
  assert.match(lab, /client\\.generateClimate\\(request, handleGenerationProgress\\)/);
});
""")

# Remove obsolete standalone public pages and their standalone UI controller.
for path in ["drainage.html", "worldgen-lab.html", "src/worldgen/diagnostics/worldgenDrainageLabStandalone.ts"]:
    p = Path(path)
    if p.exists():
        p.unlink()
