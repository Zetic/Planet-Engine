import {
  WORLDGEN_CRUST_CONTINENTAL,
  WORLDGEN_INVALID_SAMPLE_ID,
  type WorldgenClimateResult,
} from './protocol.js';

export const WORLD_CALIBRATION_SCHEMA = 'planet-engine-calibration@1';
const RANKED_LIMIT = 8;
const SIGNIFICANT_CONTINENT_AREA_FRACTION = 0.0025;

export type WorldCalibrationPacket = ReturnType<typeof buildWorldCalibrationPacket>;

type RawComponent = {
  anchorSample: number;
  sampleCount: number;
  areaSr: number;
  perimeterRad: number;
  diameterRad: number;
  plateCount: number;
};

function position(result: WorldgenClimateResult, sample: number): [number, number, number] {
  const offset = sample * 3;
  return [result.positions[offset]!, result.positions[offset + 1]!, result.positions[offset + 2]!];
}

function arc(a: [number, number, number], b: [number, number, number]): number {
  return Math.acos(Math.max(-1, Math.min(1, a[0] * b[0] + a[1] * b[1] + a[2] * b[2])));
}

function sampleArc(result: WorldgenClimateResult, a: number, b: number): number {
  return arc(position(result, a), position(result, b));
}

function percentile(values: ArrayLike<number>, fraction: number): number {
  if (values.length === 0) return 0;
  const sorted = Array.from(values).sort((a, b) => a - b);
  const index = Math.max(0, Math.min(sorted.length - 1, Math.round((sorted.length - 1) * fraction)));
  return sorted[index]!;
}

function continentalComponents(result: WorldgenClimateResult): RawComponent[] {
  const count = result.metrics.fineSampleCount;
  const visited = new Uint8Array(count);
  const meanAreaSr = 4 * Math.PI / Math.max(1, count);
  const components: RawComponent[] = [];
  for (let start = 0; start < count; start += 1) {
    if (visited[start] || result.crustKind[start] !== WORLDGEN_CRUST_CONTINENTAL) continue;
    visited[start] = 1;
    const queue: number[] = [start];
    let queueIndex = 0;
    const samples: number[] = [];
    while (queueIndex < queue.length) {
      const sample = queue[queueIndex++]!;
      samples.push(sample);
      for (let cursor = result.neighborOffsets[sample]!; cursor < result.neighborOffsets[sample + 1]!; cursor += 1) {
        const neighbor = result.neighbors[cursor]!;
        if (!visited[neighbor] && result.crustKind[neighbor] === WORLDGEN_CRUST_CONTINENTAL) {
          visited[neighbor] = 1;
          queue.push(neighbor);
        }
      }
    }
    let perimeterRad = 0;
    const plateIds = new Set<number>();
    for (const sample of samples) {
      plateIds.add(result.plateIds[sample]!);
      for (let cursor = result.neighborOffsets[sample]!; cursor < result.neighborOffsets[sample + 1]!; cursor += 1) {
        const neighbor = result.neighbors[cursor]!;
        if (result.crustKind[neighbor] !== WORLDGEN_CRUST_CONTINENTAL) perimeterRad += sampleArc(result, sample, neighbor);
      }
    }
    const first = samples[0]!;
    let farthestFromFirst = first;
    let farthestArc = -1;
    for (const sample of samples) {
      const distance = sampleArc(result, first, sample);
      if (distance > farthestArc) { farthestArc = distance; farthestFromFirst = sample; }
    }
    let farthest = farthestFromFirst;
    farthestArc = -1;
    for (const sample of samples) {
      const distance = sampleArc(result, farthestFromFirst, sample);
      if (distance > farthestArc) { farthestArc = distance; farthest = sample; }
    }
    components.push({
      anchorSample: start,
      sampleCount: samples.length,
      areaSr: samples.length * meanAreaSr,
      perimeterRad,
      diameterRad: sampleArc(result, farthestFromFirst, farthest),
      plateCount: plateIds.size,
    });
  }
  return components;
}

function continentSummary(result: WorldgenClimateResult) {
  const totalAreaSr = 4 * Math.PI;
  const components = continentalComponents(result)
    .filter(component => component.areaSr >= totalAreaSr * SIGNIFICANT_CONTINENT_AREA_FRACTION)
    .sort((a, b) => b.areaSr - a.areaSr);
  const areas = components.map(component => component.areaSr);
  const mean = areas.length ? areas.reduce((sum, value) => sum + value, 0) / areas.length : 0;
  const variance = areas.length ? areas.reduce((sum, value) => sum + (value - mean) ** 2, 0) / areas.length : 0;
  const cv = mean > 0 ? Math.sqrt(variance) / mean : 0;
  const median = areas.length ? areas[Math.floor(areas.length / 2)]! : 0;
  const hierarchy = areas.length ? areas[0]! / Math.max(1e-18, median) : 0;
  const radiusKm = result.planet.radiusM / 1000;
  const areaScaleKm2 = radiusKm * radiusKm;
  let maximumElongation = 0;
  let maximumCompactness = 0;
  let hasMajorMultiplateComponent = false;
  const ranked = components.slice(0, RANKED_LIMIT).map(component => {
    const equivalentRadius = Math.max(1e-9, Math.sqrt(component.areaSr / Math.PI));
    const elongation = component.diameterRad / (2 * equivalentRadius);
    const compactness = component.perimeterRad ** 2 / (4 * Math.PI * Math.max(1e-18, component.areaSr));
    maximumElongation = Math.max(maximumElongation, elongation);
    maximumCompactness = Math.max(maximumCompactness, compactness);
    return {
      anchor_sample: component.anchorSample,
      sample_count: component.sampleCount,
      area_km2: component.areaSr * areaScaleKm2,
      perimeter_km: component.perimeterRad * radiusKm,
      diameter_km: component.diameterRad * radiusKm,
      plate_count: component.plateCount,
      elongation,
      compactness,
    };
  });
  for (const component of components) {
    const equivalentRadius = Math.max(1e-9, Math.sqrt(component.areaSr / Math.PI));
    maximumElongation = Math.max(maximumElongation, component.diameterRad / (2 * equivalentRadius));
    maximumCompactness = Math.max(maximumCompactness, component.perimeterRad ** 2 / (4 * Math.PI * Math.max(1e-18, component.areaSr)));
    if (component.areaSr >= totalAreaSr * 0.015 && component.plateCount >= 2) hasMajorMultiplateComponent = true;
  }
  return {
    significant_component_count: components.length,
    component_area_cv: cv,
    largest_to_median_area_ratio: hierarchy,
    maximum_elongation: maximumElongation,
    maximum_compactness: maximumCompactness,
    largest_component_plate_count: components[0]?.plateCount ?? 0,
    has_major_multiplate_component: hasMajorMultiplateComponent,
    ranked_components: ranked,
  };
}

function topographySummary(result: WorldgenClimateResult) {
  const land: number[] = [];
  const waterDepth: number[] = [];
  for (let sample = 0; sample < result.metrics.fineSampleCount; sample += 1) {
    if (result.submergedMask[sample]) waterDepth.push(Math.max(0, result.waterDepthM[sample]!));
    else land.push(result.elevationAboveSeaLevelM[sample]!);
  }
  const solid = result.solidElevationM;
  const mean = (values: number[]) => values.length ? values.reduce((sum, value) => sum + value, 0) / values.length : 0;
  return {
    minimum_solid_elevation_m: result.metrics.minimumSolidElevationM,
    p05_solid_elevation_m: percentile(solid, 0.05),
    median_solid_elevation_m: percentile(solid, 0.50),
    p95_solid_elevation_m: percentile(solid, 0.95),
    maximum_solid_elevation_m: result.metrics.maximumSolidElevationM,
    sea_level_m: result.metrics.hasSeaLevel ? result.metrics.seaLevelM : null,
    land_area_fraction: result.metrics.landAreaFraction,
    ocean_area_fraction: result.metrics.oceanAreaFraction,
    mean_land_elevation_m: mean(land),
    mean_water_depth_m: mean(waterDepth),
    maximum_water_depth_m: waterDepth.reduce((maximum, value) => Math.max(maximum, value), 0),
    water_volume_relative_error: null,
    clamped_sample_count: null,
  };
}

function rankedBasins(result: WorldgenClimateResult) {
  return Array.from(result.basinAreasM2, (area, index) => ({
    basin_id: index,
    outlet_sample: result.basinOutletSamples[index]!,
    outlet_kind: result.basinOutletKinds[index]!,
    area_km2: area / 1e6,
  }))
    .sort((a, b) => b.area_km2 - a.area_km2)
    .slice(0, RANKED_LIMIT);
}

function rankedDepressions(result: WorldgenClimateResult) {
  return Array.from(result.depressionAreasM2, (area, index) => ({
    depression_id: index,
    floor_sample: result.depressionFloorSamples[index]!,
    area_km2: area / 1e6,
    maximum_depth_m: Math.max(0, result.depressionSpillElevationsM[index]! - result.depressionFloorElevationsM[index]!),
    floor_elevation_m: result.depressionFloorElevationsM[index]!,
    spill_elevation_m: result.depressionSpillElevationsM[index]!,
  }))
    .sort((a, b) => b.area_km2 - a.area_km2)
    .slice(0, RANKED_LIMIT);
}

function rankedLakes(result: WorldgenClimateResult) {
  const maximumDepthById = new Map<number, number>();
  for (let sample = 0; sample < result.lakeId.length; sample += 1) {
    const lakeId = result.lakeId[sample]!;
    if (lakeId === WORLDGEN_INVALID_SAMPLE_ID) continue;
    maximumDepthById.set(lakeId, Math.max(maximumDepthById.get(lakeId) ?? 0, result.lakeDepthM[sample]!));
  }
  return Array.from(result.lakeAreasM2, (area, index) => ({
    lake_id: index,
    depression_id: result.lakeDepressionIds[index]!,
    kind: result.lakeKinds[index]!,
    area_km2: area / 1e6,
    volume_km3: result.lakeVolumesM3[index]! / 1e9,
    maximum_depth_m: maximumDepthById.get(index) ?? 0,
    surface_elevation_m: result.lakeSurfaceElevationsM[index]!,
    gross_land_inflow_m3_s: null,
    lake_evaporation_m3_s: null,
    outflow_m3_s: result.lakeOutflowsM3S[index]!,
  }))
    .sort((a, b) => b.area_km2 - a.area_km2)
    .slice(0, RANKED_LIMIT);
}

export function buildWorldCalibrationPacket(result: WorldgenClimateResult, seed: string, plateCount: number) {
  const largestBasins = rankedBasins(result);
  const largestDepressions = rankedDepressions(result);
  const largestLakes = rankedLakes(result);
  const landAreaKm2 = Math.max(1e-12, result.drainageMetrics.landAreaM2 / 1e6);
  return {
    schema: WORLD_CALIBRATION_SCHEMA,
    run: {
      seed,
      engine_version: result.engineVersion,
      coarse_level: result.coarseLevel,
      fine_level: result.fineLevel,
      sample_count: result.metrics.fineSampleCount,
      plate_count: plateCount,
    },
    hashes: {
      coarse_topology: result.metrics.coarseTopologyHash,
      fine_topology: result.metrics.fineTopologyHash,
      tectonic: result.metrics.tectonicHash,
      geology: result.metrics.geologyHash,
      lithosphere: result.metrics.lithosphereHash,
      inheritance: result.metrics.inheritanceHash,
      topography: result.metrics.topographyHash,
      climate: result.metrics.climateHash,
      final_drainage: result.drainageMetrics.drainageHash,
      final_runoff: result.runoffMetrics.runoffHash,
      final_lake: result.lakeMetrics.lakeHash,
      final_seasonal: result.seasonalMetrics.seasonalHydrologyHash,
      erosion: result.erosionMetrics.fluvialErosionHash,
      evolution: result.evolutionMetrics.terrainEvolutionHash,
      post_erosion_hydrology: result.reconciliationMetrics.postErosionHydrologyHash,
      infill: result.infillMetrics.lakeSedimentInfillHash,
    },
    continents: continentSummary(result),
    topography: topographySummary(result),
    climate: {
      global_solver_level: result.metrics.globalSolverLevel,
      global_solver_sample_count: result.metrics.globalSolverSampleCount,
      orbital_phase_count: result.metrics.orbitalPhaseCount,
      spinup_years: result.metrics.spinupYears,
      minimum_temperature_k: result.metrics.minimumTemperatureK,
      mean_temperature_k: result.metrics.meanTemperatureK,
      maximum_temperature_k: result.metrics.maximumTemperatureK,
      mean_land_temperature_k: result.metrics.meanLandTemperatureK,
      mean_ocean_temperature_k: result.metrics.meanOceanTemperatureK,
      mean_wind_speed_m_s: result.metrics.meanWindSpeedMS,
      maximum_wind_speed_m_s: result.metrics.maximumWindSpeedMS,
      mean_surface_current_m_s: result.metrics.meanSurfaceCurrentMS,
      maximum_surface_current_m_s: result.metrics.maximumSurfaceCurrentMS,
      mean_sst_k: result.metrics.meanSeaSurfaceTemperatureK,
      mean_annual_precipitation_mm: result.metrics.meanAnnualPrecipitationMm,
      p95_annual_precipitation_mm: result.metrics.p95AnnualPrecipitationMm,
      precipitation_p95_to_mean_ratio: result.metrics.p95AnnualPrecipitationMm / Math.max(1e-12, result.metrics.meanAnnualPrecipitationMm),
      moisture_budget_relative_error: result.metrics.moistureBudgetRelativeError,
      moisture_transport_limiter_fraction: result.metrics.moistureTransportLimiterFraction,
      maximum_moisture_transport_substeps: result.metrics.maximumMoistureTransportSubsteps,
      persistent_snow_area_fraction: result.metrics.persistentSnowAreaFraction,
      sea_ice_area_fraction: result.metrics.seaIceAreaFraction,
      final_temperature_rms_change_k: result.metrics.finalTemperatureRmsChangeK,
    },
    hydrology: {
      basin_count: result.drainageMetrics.basinCount,
      depression_count: result.drainageMetrics.depressionCount,
      largest_contributing_area_km2: result.drainageMetrics.maximumContributingAreaM2 / 1e6,
      largest_basin_area_fraction_of_land: (largestBasins[0]?.area_km2 ?? 0) / landAreaKm2,
      maximum_depression_depth_m: result.drainageMetrics.maximumDepressionDepthM,
      drainage_area_conservation_relative_error: result.drainageMetrics.areaConservationRelativeError,
      mean_land_precipitation_mm: result.runoffMetrics.meanLandPrecipitationMm,
      mean_land_actual_evapotranspiration_mm: result.runoffMetrics.meanLandActualEvapotranspirationMm,
      mean_land_runoff_mm: result.runoffMetrics.meanLandRunoffMm,
      land_runoff_fraction: result.runoffMetrics.landRunoffFraction,
      maximum_potential_discharge_m3_s: result.runoffMetrics.maximumPotentialDischargeM3S,
      runoff_discharge_conservation_relative_error: result.runoffMetrics.dischargeConservationRelativeError,
      lake_count: result.lakeMetrics.lakeCount,
      endorheic_lake_count: result.lakeMetrics.endorheicLakeCount,
      overflowing_lake_count: result.lakeMetrics.overflowingLakeCount,
      terminal_storage_lake_count: result.lakeMetrics.terminalStorageLakeCount,
      total_lake_area_km2: result.lakeMetrics.totalLakeAreaM2 / 1e6,
      total_lake_volume_km3: result.lakeMetrics.totalLakeVolumeM3 / 1e9,
      largest_lake_area_km2: result.lakeMetrics.maximumLakeAreaM2 / 1e6,
      largest_lake_area_fraction_of_land: (result.lakeMetrics.maximumLakeAreaM2 / 1e6) / landAreaKm2,
      maximum_lake_depth_m: result.lakeMetrics.maximumLakeDepthM,
      lake_water_balance_relative_error: result.lakeMetrics.waterBalanceRelativeError,
      dry_flow_sample_count: result.seasonalMetrics.dryFlowSampleCount,
      intermittent_flow_sample_count: result.seasonalMetrics.intermittentFlowSampleCount,
      perennial_flow_sample_count: result.seasonalMetrics.perennialFlowSampleCount,
      snowmelt_runoff_fraction: result.seasonalMetrics.snowmeltRunoffFraction,
      maximum_phase_realized_discharge_m3_s: result.seasonalMetrics.maximumPhaseRealizedDischargeM3S,
      seasonal_routing_conservation_relative_error: result.seasonalMetrics.seasonalRoutingConservationRelativeError,
      seasonal_water_balance_relative_error: result.seasonalMetrics.seasonalWaterBalanceRelativeError,
      lake_spinup_years: result.seasonalMetrics.lakeSpinupYears,
      final_lake_surface_cycle_change_m: result.seasonalMetrics.finalLakeSurfaceCycleChangeM,
      maximum_seasonal_lake_level_range_m: result.seasonalMetrics.maximumSeasonalLakeLevelRangeM,
      largest_basins: largestBasins,
      largest_depressions: largestDepressions,
      largest_lakes: largestLakes,
    },
    geomorphology: {
      erosive_sample_count: result.erosionMetrics.erosiveSampleCount,
      active_lake_trap_count: result.erosionMetrics.activeLakeTrapCount,
      maximum_effective_discharge_m3_s: result.erosionMetrics.maximumEffectiveDischargeM3S,
      maximum_channel_slope: result.erosionMetrics.maximumChannelSlope,
      maximum_channel_width_m: result.erosionMetrics.maximumChannelWidthM,
      maximum_incision_potential_m_per_year: result.erosionMetrics.maximumIncisionPotentialMPerYear,
      total_sediment_generated_kg_s: result.erosionMetrics.totalSedimentGeneratedKgS,
      total_land_deposition_kg_s: result.erosionMetrics.totalLandDepositionKgS,
      total_lake_deposition_kg_s: result.erosionMetrics.totalLakeDepositionKgS,
      total_terminal_ocean_deposition_kg_s: result.erosionMetrics.totalTerminalOceanDepositionKgS,
      erosion_sediment_conservation_relative_error: result.erosionMetrics.sedimentConservationRelativeError,
      geomorphic_duration_years: result.evolutionMetrics.geomorphicDurationYears,
      eroded_sample_count: result.evolutionMetrics.erodedSampleCount,
      depositional_sample_count: result.evolutionMetrics.depositionalSampleCount,
      receiver_changed_sample_count: result.evolutionMetrics.receiverChangedSampleCount,
      receiver_changed_fraction: result.evolutionMetrics.receiverChangedFraction,
      maximum_applied_erosion_m: result.evolutionMetrics.maximumAppliedErosionM,
      maximum_applied_deposition_m: result.evolutionMetrics.maximumAppliedDepositionM,
      mean_land_absolute_terrain_change_m: result.evolutionMetrics.meanLandAbsoluteTerrainChangeM,
      evolution_sediment_conservation_relative_error: result.evolutionMetrics.sedimentConservationRelativeError,
      post_erosion_runoff_conservation_relative_error: result.evolutionMetrics.postErosionRunoffConservationRelativeError,
      filled_depression_count: result.infillMetrics.filledDepressionCount,
      filled_sample_count: result.infillMetrics.filledSampleCount,
      capacity_limited_depression_count: result.infillMetrics.capacityLimitedDepressionCount,
      maximum_lake_fill_depth_m: result.infillMetrics.maximumFillDepthM,
      total_historical_lake_delivery_kg_s: result.infillMetrics.totalHistoricalLakeDeliveryKgS,
      total_applied_lake_fill_equivalent_kg_s: result.infillMetrics.totalAppliedLakeFillEquivalentKgS,
      total_unapplied_lake_sediment_kg_s: result.infillMetrics.totalUnappliedLakeSedimentKgS,
      infill_sediment_conservation_relative_error: result.infillMetrics.sedimentConservationRelativeError,
      pre_infill_lake_count: result.infillMetrics.preInfillLakeCount,
      post_infill_lake_count: result.infillMetrics.postInfillLakeCount,
    },
  };
}

export function worldCalibrationJson(result: WorldgenClimateResult, seed: string, plateCount: number): string {
  return `${JSON.stringify(buildWorldCalibrationPacket(result, seed, plateCount), null, 2)}\n`;
}

export function worldCalibrationMarkdown(result: WorldgenClimateResult, seed: string, plateCount: number): string {
  const packet = buildWorldCalibrationPacket(result, seed, plateCount);
  const c = packet.continents;
  const t = packet.topography;
  const climate = packet.climate;
  const h = packet.hydrology;
  const g = packet.geomorphology;
  const lines = [
    '# Planet Engine calibration report',
    '',
    `Schema: \`${packet.schema}\``,
    `Seed: \`${seed}\` · L${result.coarseLevel} → L${result.fineLevel} · ${plateCount} plates · ${result.metrics.fineSampleCount.toLocaleString()} samples · engine v${result.engineVersion}`,
    '',
    '## Continental assembly',
    `Significant components: ${c.significant_component_count} · area CV ${c.component_area_cv.toFixed(3)} · largest/median ${c.largest_to_median_area_ratio.toFixed(3)}× · max elongation ${c.maximum_elongation.toFixed(3)} · max compactness ${c.maximum_compactness.toFixed(3)} · major multi-plate component: ${c.has_major_multiplate_component}`,
    ...c.ranked_components.map((component, index) => `${index + 1}. ${(component.area_km2 / 1e6).toFixed(3)}M km² · diameter ${component.diameter_km.toFixed(0)} km · ${component.plate_count} plates · elongation ${component.elongation.toFixed(3)} · compactness ${component.compactness.toFixed(3)}`),
    '',
    '## Topography',
    `Land/ocean: ${(t.land_area_fraction * 100).toFixed(1)}% / ${(t.ocean_area_fraction * 100).toFixed(1)}% · mean land elevation ${t.mean_land_elevation_m.toFixed(0)} m · mean/max water depth ${t.mean_water_depth_m.toFixed(0)}/${t.maximum_water_depth_m.toFixed(0)} m`,
    `Solid elevation P05/P50/P95: ${t.p05_solid_elevation_m.toFixed(0)}/${t.median_solid_elevation_m.toFixed(0)}/${t.p95_solid_elevation_m.toFixed(0)} m · range ${t.minimum_solid_elevation_m.toFixed(0)} → ${t.maximum_solid_elevation_m.toFixed(0)} m`,
    '',
    '## Climate',
    `Temperature ${climate.minimum_temperature_k.toFixed(1)} → ${climate.maximum_temperature_k.toFixed(1)} K · mean ${climate.mean_temperature_k.toFixed(1)} K · land/ocean ${climate.mean_land_temperature_k.toFixed(1)}/${climate.mean_ocean_temperature_k.toFixed(1)} K · SST ${climate.mean_sst_k.toFixed(1)} K`,
    `Precipitation mean/P95 ${climate.mean_annual_precipitation_mm.toFixed(1)}/${climate.p95_annual_precipitation_mm.toFixed(1)} mm/yr · P95/mean ${climate.precipitation_p95_to_mean_ratio.toFixed(2)}× · moisture closure ${climate.moisture_budget_relative_error.toExponential(3)}`,
    '',
    '## Hydrology',
    `Basins/depressions: ${h.basin_count} / ${h.depression_count} · largest contributing area ${(h.largest_contributing_area_km2 / 1e6).toFixed(3)}M km² · deepest depression ${h.maximum_depression_depth_m.toFixed(1)} m · area closure ${h.drainage_area_conservation_relative_error.toExponential(3)}`,
    `Land water balance: P ${h.mean_land_precipitation_mm.toFixed(1)} · AET ${h.mean_land_actual_evapotranspiration_mm.toFixed(1)} · runoff ${h.mean_land_runoff_mm.toFixed(1)} mm/yr · runoff fraction ${(h.land_runoff_fraction * 100).toFixed(1)}%`,
    `Lakes: ${h.lake_count} total (${h.endorheic_lake_count} endorheic / ${h.overflowing_lake_count} overflowing / ${h.terminal_storage_lake_count} terminal) · area ${(h.total_lake_area_km2 / 1e6).toFixed(3)}M km² · largest ${(h.largest_lake_area_km2 / 1e3).toFixed(0)}k km² (${(h.largest_lake_area_fraction_of_land * 100).toFixed(2)}% of land) · max depth ${h.maximum_lake_depth_m.toFixed(1)} m`,
    `Flow regimes: ${h.dry_flow_sample_count} dry / ${h.intermittent_flow_sample_count} intermittent / ${h.perennial_flow_sample_count} perennial · snowmelt ${(h.snowmelt_runoff_fraction * 100).toFixed(2)}% · seasonal closure ${h.seasonal_water_balance_relative_error.toExponential(3)}`,
    '',
    '### Largest lakes',
    ...h.largest_lakes.map(lake => `- lake ${lake.lake_id} / depression ${lake.depression_id}: ${(lake.area_km2 / 1e3).toFixed(0)}k km² · ${(lake.volume_km3 / 1e3).toFixed(1)}k km³ · depth ${lake.maximum_depth_m.toFixed(1)} m · outflow ${lake.outflow_m3_s.toFixed(0)} m³/s`),
    '',
    '### Largest depressions',
    ...h.largest_depressions.map(depression => `- depression ${depression.depression_id}: ${(depression.area_km2 / 1e3).toFixed(0)}k km² · depth ${depression.maximum_depth_m.toFixed(1)} m · floor/spill ${depression.floor_elevation_m.toFixed(1)}/${depression.spill_elevation_m.toFixed(1)} m`),
    '',
    '## Geomorphology',
    `WG-7A: ${g.erosive_sample_count} erosive samples · max Q ${g.maximum_effective_discharge_m3_s.toFixed(0)} m³/s · max slope ${g.maximum_channel_slope.toFixed(5)} · sediment ${g.total_sediment_generated_kg_s.toFixed(1)} kg/s · closure ${g.erosion_sediment_conservation_relative_error.toExponential(3)}`,
    `WG-7B: ${g.geomorphic_duration_years.toFixed(0)} years · ${g.eroded_sample_count} eroded / ${g.depositional_sample_count} depositional samples · receiver changes ${g.receiver_changed_sample_count} (${(g.receiver_changed_fraction * 100).toFixed(3)}%) · max erosion/deposition ${g.maximum_applied_erosion_m.toFixed(1)}/${g.maximum_applied_deposition_m.toFixed(1)} m`,
    `WG-7D: ${g.filled_depression_count} filled depressions / ${g.filled_sample_count} samples · max fill ${g.maximum_lake_fill_depth_m.toFixed(1)} m · lakes ${g.pre_infill_lake_count} → ${g.post_infill_lake_count}`,
    '',
    '## Causal identity',
    `\`tectonic ${packet.hashes.tectonic}\` → \`geology ${packet.hashes.geology}\` → \`lithosphere ${packet.hashes.lithosphere}\` → \`topography ${packet.hashes.topography}\` → \`climate ${packet.hashes.climate}\``,
    `\`drainage ${packet.hashes.final_drainage}\` → \`runoff ${packet.hashes.final_runoff}\` → \`lakes ${packet.hashes.final_lake}\` → \`seasonal ${packet.hashes.final_seasonal}\` → \`erosion ${packet.hashes.erosion}\` → \`evolution ${packet.hashes.evolution}\` → \`infill ${packet.hashes.infill}\``,
    '',
  ];
  return lines.join('\n');
}
