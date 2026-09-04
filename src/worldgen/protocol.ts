export const WORLDGEN_PROTOCOL_VERSION = 14;
export const WORLDGEN_SYNTHETIC_MAX_SAMPLES = 4_194_304;
export const WORLDGEN_TOPOLOGY_MAX_LEVEL = 7;
export const WORLDGEN_TECTONICS_MAX_LEVEL = 6;
export const WORLDGEN_GEOLOGY_MAX_LEVEL = 6;
export const WORLDGEN_LITHOSPHERE_MAX_LEVEL = 6;
export const WORLDGEN_INHERITANCE_COARSE_MAX_LEVEL = 6;
export const WORLDGEN_INHERITANCE_FINE_MAX_LEVEL = 7;
export const WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL = 6;
export const WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL = 7;
export const WORLDGEN_CLIMATE_COARSE_MAX_LEVEL = 6;
export const WORLDGEN_CLIMATE_FINE_MAX_LEVEL = 7;
export const WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL = 6;
export const WORLDGEN_DRAINAGE_FINE_MAX_LEVEL = 7;
export const WORLDGEN_INVALID_SAMPLE_ID = 0xffff_ffff;
export const WORLDGEN_DRAINAGE_OUTLET_NONE = 0;
export const WORLDGEN_DRAINAGE_OUTLET_OCEAN = 1;
export const WORLDGEN_DRAINAGE_OUTLET_INTERNAL = 2;
export const WORLDGEN_TECTONICS_MIN_PLATES = 4;
export const WORLDGEN_TECTONICS_MAX_PLATES = 48;

export interface WorldgenSyntheticRequest { seed: string; width: number; height: number; }
export interface WorldgenTopologyRequest { level: number; }
export interface WorldgenTectonicsRequest { seed: string; level: number; plateCount: number; }
export interface WorldgenGeologyRequest { seed: string; level: number; plateCount: number; }
export interface WorldgenLithosphereRequest { seed: string; level: number; plateCount: number; }
export interface WorldgenInheritanceRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }
export interface WorldgenTopographyRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }
export interface WorldgenClimateRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }
export interface WorldgenDrainageRequest { seed: string; coarseLevel: number; fineLevel: number; plateCount: number; }

export interface WorldgenFieldStatistics { sampleCount: number; minimum: number; maximum: number; mean: number; fieldHash: string; }
export interface WorldgenStageMetadata { id: string; version: number; stageSeed: string; durationMs: number; }
export interface WorldgenUnseededStageMetadata { id: string; version: number; durationMs: number; }
export interface WorldgenGenerationTiming { stageId: string; durationMs: number; }
export interface WorldgenGenerationProgress {
  stageId: string;
  stageIndex: number;
  stageCount: number;
  completed: number;
  total: number;
  elapsedMs: number;
  stageElapsedMs: number;
}
export interface WorldgenSyntheticResult { engineVersion: number; width: number; height: number; values: Uint16Array; statistics: WorldgenFieldStatistics; stage: WorldgenStageMetadata; }

export interface WorldgenTopologyMetrics { sampleCount: number; edgeCount: number; faceCount: number; fiveNeighborCount: number; sixNeighborCount: number; totalAreaSteradians: number; minimumAreaSteradians: number; maximumAreaSteradians: number; meanAreaSteradians: number; areaCoefficientOfVariation: number; minimumEdgeArcRadians: number; maximumEdgeArcRadians: number; meanEdgeArcRadians: number; edgeCoefficientOfVariation: number; minimumInterfaceArcRadians: number; maximumInterfaceArcRadians: number; meanInterfaceArcRadians: number; interfaceCoefficientOfVariation: number; topologyHash: string; }
export interface WorldgenTopologyResult { engineVersion: number; level: number; metrics: WorldgenTopologyMetrics; durationMs: number; positions: Float64Array; faces: Uint32Array; neighborOffsets: Uint32Array; neighbors: Uint32Array; neighborArcLengthsRad: Float64Array; neighborInterfaceArcLengthsRad: Float64Array; areaSteradians: Float64Array; birthLevels: Uint8Array; parentEdges: Uint32Array; }

export const WORLDGEN_BOUNDARY_CONVERGENT = 1;
export const WORLDGEN_BOUNDARY_DIVERGENT = 2;
export const WORLDGEN_BOUNDARY_TRANSFORM = 3;
export interface WorldgenTectonicMetrics { sampleCount: number; plateCount: number; boundaryEdgeCount: number; convergentEdgeCount: number; divergentEdgeCount: number; transformEdgeCount: number; minimumPlateAreaFraction: number; maximumPlateAreaFraction: number; meanPlateAreaFraction: number; minimumSeedSeparationRad: number; meanReferenceSpeedMmPerYear: number; tectonicHash: string; }
export interface WorldgenTectonicsResult {
  engineVersion: number;
  level: number;
  topologyHash: string;
  metrics: WorldgenTectonicMetrics;
  stage: WorldgenStageMetadata;
  positions: Float64Array;
  faces: Uint32Array;
  neighborOffsets: Uint32Array;
  neighbors: Uint32Array;
  plateIds: Uint16Array;
  plateSeedSamples: Uint32Array;
  plateEulerPoles: Float64Array;
  plateAngularVelocitiesRadPerMyr: Float64Array;
  plateAreaSteradians: Float64Array;
  boundarySamples: Uint32Array;
  boundaryPlateIds: Uint16Array;
  boundaryKinds: Uint8Array;
  boundaryNormalRatesMPerYear: Float64Array;
  boundaryShearRatesMPerYear: Float64Array;
}

export const WORLDGEN_CRUST_OCEANIC = 1;
export const WORLDGEN_CRUST_TRANSITIONAL = 2;
export const WORLDGEN_CRUST_CONTINENTAL = 3;
export const WORLDGEN_PLATE_MAJOR = 1;
export const WORLDGEN_PLATE_INTERMEDIATE = 2;
export const WORLDGEN_PLATE_MINOR = 3;
export const WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION = 1;
export const WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION = 2;
export const WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION = 3;
export const WORLDGEN_GEOLOGY_OCEANIC_RIDGE = 4;
export const WORLDGEN_GEOLOGY_CONTINENTAL_RIFT = 5;
export const WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE = 6;
export const WORLDGEN_GEOLOGY_TRANSFORM = 7;
export const WORLDGEN_SUBDUCTION_NONE = 0;
export const WORLDGEN_SUBDUCTION_PLATE_A = 1;
export const WORLDGEN_SUBDUCTION_PLATE_B = 2;

export interface WorldgenGeologyMetrics {
  sampleCount: number;
  continentalAreaFraction: number;
  transitionalAreaFraction: number;
  oceanicAreaFraction: number;
  meanContinentalAgeMyr: number;
  meanOceanicAgeMyr: number;
  meanContinentalThicknessKm: number;
  meanOceanicThicknessKm: number;
  oceanicSubductionEdges: number;
  oceanContinentSubductionEdges: number;
  continentalCollisionEdges: number;
  oceanicRidgeEdges: number;
  continentalRiftEdges: number;
  transitionalDivergenceEdges: number;
  transformEdges: number;
  geologyHash: string;
  tectonicHash: string;
}

export interface WorldgenGeologyResult {
  engineVersion: number;
  level: number;
  topologyHash: string;
  plateCount: number;
  boundaryEdgeCount: number;
  stage: WorldgenStageMetadata;
  provinceSeed: string;
  propertySeed: string;
  historySeed: string;
  metrics: WorldgenGeologyMetrics;
  positions: Float64Array;
  faces: Uint32Array;
  neighborOffsets: Uint32Array;
  neighbors: Uint32Array;
  plateIds: Uint16Array;
  boundarySamples: Uint32Array;
  boundaryPlateIds: Uint16Array;
  boundaryKinds: Uint8Array;
  crustKind: Uint8Array;
  crustProvinceId: Uint16Array;
  crustAgeMyr: Float32Array;
  crustThicknessKm: Float32Array;
  crustDensityKgPerM3: Float32Array;
  buoyancyIndex: Float32Array;
  orogenicHistory: Float32Array;
  riftHistory: Float32Array;
  ridgeHistory: Float32Array;
  subductionHistory: Float32Array;
  trenchHistory: Float32Array;
  volcanicArcHistory: Float32Array;
  transformHistory: Float32Array;
  subsidenceHistory: Float32Array;
  basinPotential: Float32Array;
  crustalStrain: Float32Array;
  geologicalBoundaryRegimes: Uint8Array;
  subductionPolarities: Uint8Array;
  plateScaleClasses: Uint8Array;
  plateContinentalFractions: Float64Array;
  plateTransitionalFractions: Float64Array;
  plateOceanicFractions: Float64Array;
  plateMeanCrustAgeMyr: Float64Array;
  plateMeanCrustThicknessKm: Float64Array;
}

export const WORLDGEN_STRUCTURE_NONE = 0;
export const WORLDGEN_STRUCTURE_SUTURE = 1;
export const WORLDGEN_STRUCTURE_RIFT = 2;
export const WORLDGEN_STRUCTURE_TRANSFORM = 3;
export const WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN = 4;
export const WORLDGEN_FRAGMENT_TERRANE = 1;
export const WORLDGEN_FRAGMENT_MICROPLATE = 2;

export interface WorldgenLithosphereMetrics {
  sampleCount: number;
  meanStrengthIndex: number;
  meanWeaknessIndex: number;
  meanEffectiveElasticThicknessKm: number;
  meanMantleUpwellingIndex: number;
  meanDynamicSupportIndex: number;
  sutureSampleCount: number;
  riftZoneSampleCount: number;
  transformZoneSampleCount: number;
  continentalMarginSampleCount: number;
  tectonicFragmentCount: number;
  microplateCount: number;
  terraneCount: number;
  fragmentedAreaFraction: number;
  lithosphereHash: string;
  geologyHash: string;
  tectonicHash: string;
}

export interface WorldgenLithosphereResult {
  engineVersion: number;
  level: number;
  topologyHash: string;
  plateCount: number;
  boundaryEdgeCount: number;
  stage: WorldgenStageMetadata;
  mechanicalSeed: string;
  mantleSeed: string;
  refinementSeed: string;
  metrics: WorldgenLithosphereMetrics;
  positions: Float64Array;
  faces: Uint32Array;
  neighborOffsets: Uint32Array;
  neighbors: Uint32Array;
  plateIds: Uint16Array;
  boundarySamples: Uint32Array;
  boundaryKinds: Uint8Array;
  crustKind: Uint8Array;
  geologicalBoundaryRegimes: Uint8Array;
  orogenicHistory: Float32Array;
  riftHistory: Float32Array;
  ridgeHistory: Float32Array;
  subductionHistory: Float32Array;
  transformHistory: Float32Array;
  crustalStrain: Float32Array;
  strengthIndex: Float32Array;
  weaknessIndex: Float32Array;
  effectiveElasticThicknessKm: Float32Array;
  thermalAnomalyIndex: Float32Array;
  mantleUpwellingIndex: Float32Array;
  mantleDynamicSupportIndex: Float32Array;
  compensatedBuoyancyIndex: Float32Array;
  structuralFabricStrength: Float32Array;
  structuralZoneKind: Uint8Array;
  fragmentationPropensity: Float32Array;
  fragmentIds: Uint16Array;
  kinematicDomainIds: Uint16Array;
  fragmentParentPlateIds: Uint16Array;
  fragmentKinds: Uint8Array;
  fragmentSeedSamples: Uint32Array;
  fragmentAreaSteradians: Float64Array;
  fragmentAreaFractionsOfParent: Float64Array;
  fragmentMeanWeakness: Float64Array;
  fragmentMeanPropensity: Float64Array;
  fragmentAngularVelocitiesRadPerMyr: Float64Array;
}

export interface WorldgenInheritanceMetrics {
  coarseSampleCount: number;
  fineSampleCount: number;
  addedSampleCount: number;
  plateCount: number;
  fineBoundaryEdgeCount: number;
  coarseTopologyHash: string;
  fineTopologyHash: string;
  tectonicHash: string;
  geologyHash: string;
  lithosphereHash: string;
  provenanceHash: string;
  parameterHash: string;
  inheritanceHash: string;
  boundaryHash: string;
}

export interface WorldgenPlanetPhysicalProfile {
  radiusM: number;
  surfaceGravityMS2: number;
  surfaceWaterMassKg: number;
  equivalentGlobalWaterDepthM: number;
  oceanWaterDensityKgPerM3: number;
  isostaticMantleDensityKgPerM3: number;
  internalHeatFluxWPerM2: number;
  mantleThermalExpansivityPerK: number;
}

export interface WorldgenInheritanceResult {
  engineVersion: number;
  coarseLevel: number;
  fineLevel: number;
  stage: WorldgenUnseededStageMetadata;
  metrics: WorldgenInheritanceMetrics;
  parameters: WorldgenPlanetPhysicalProfile;
  positions: Float64Array;
  faces: Uint32Array;
  neighborOffsets: Uint32Array;
  neighbors: Uint32Array;
  nearestCoarseSource: Uint32Array;
  inheritedSampleMask: Uint8Array;
  plateIds: Uint16Array;
  crustKind: Uint8Array;
  crustProvinceId: Uint16Array;
  crustAgeMyr: Float32Array;
  crustThicknessKm: Float32Array;
  crustDensityKgPerM3: Float32Array;
  buoyancyIndex: Float32Array;
  orogenicHistory: Float32Array;
  riftHistory: Float32Array;
  ridgeHistory: Float32Array;
  subductionHistory: Float32Array;
  trenchHistory: Float32Array;
  volcanicArcHistory: Float32Array;
  transformHistory: Float32Array;
  subsidenceHistory: Float32Array;
  basinPotential: Float32Array;
  crustalStrain: Float32Array;
  strengthIndex: Float32Array;
  weaknessIndex: Float32Array;
  effectiveElasticThicknessKm: Float32Array;
  thermalAnomalyIndex: Float32Array;
  mantleUpwellingIndex: Float32Array;
  mantleDynamicSupportIndex: Float32Array;
  compensatedBuoyancyIndex: Float32Array;
  structuralFabricStrength: Float32Array;
  structuralZoneKind: Uint8Array;
  fragmentationPropensity: Float32Array;
  fragmentIds: Uint16Array;
  kinematicDomainIds: Uint16Array;
  boundarySamples: Uint32Array;
  boundaryKinds: Uint8Array;
  geologicalBoundaryRegimes: Uint8Array;
  subductionPolarities: Uint8Array;
  boundaryNormalRatesMPerYear: Float64Array;
  boundaryShearRatesMPerYear: Float64Array;
  boundaryCoarseSourceIndices: Uint32Array;
}


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
}


export interface WorldgenClimateMetrics {
  coarseSampleCount: number;
  fineSampleCount: number;
  plateCount: number;
  fineBoundaryEdgeCount: number;
  orbitalPhaseCount: number;
  globalSolverLevel: number;
  globalSolverSampleCount: number;
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
  moistureTransportLimiterFraction: number;
  maximumMoistureTransportSubsteps: number;
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
  generationTimings: WorldgenGenerationTiming[];
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
  seaSurfaceTemperatureAnnualCosK: Float32Array;
  seaSurfaceTemperatureAnnualSinK: Float32Array;
  currentEastMeanMS: Float32Array;
  currentNorthMeanMS: Float32Array;
  currentEastAnnualCosMS: Float32Array;
  currentEastAnnualSinMS: Float32Array;
  currentNorthAnnualCosMS: Float32Array;
  currentNorthAnnualSinMS: Float32Array;
  currentSpeedMeanMS: Float32Array;
  oceanHeatTransportIndex: Float32Array;
  specificHumidityMean: Float32Array;
  annualPrecipitationMm: Float32Array;
  precipitationPhaseRateMmYear: Float32Array;
  precipitationSeasonality: Float32Array;
  potentialEvaporationMm: Float32Array;
  moistureBalanceMm: Float32Array;
  aridityIndex: Float32Array;
  snowfallFraction: Float32Array;
  persistentSnowPotential: Float32Array;
  seaIcePotential: Float32Array;
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
  lakeStage: WorldgenStageMetadata;
  lakeMetrics: WorldgenLakeMetrics;
  lakeId: Uint32Array;
  lakeKind: Uint8Array;
  lakeFraction: Float32Array;
  lakeDepthM: Float32Array;
  realizedDischargeM3S: Float32Array;
  lakeDepressionIds: Uint32Array;
  lakeKinds: Uint8Array;
  lakeSurfaceElevationsM: Float64Array;
  lakeAreasM2: Float64Array;
  lakeVolumesM3: Float64Array;
  lakeOutflowsM3S: Float64Array;
  lakeSpillSamples: Uint32Array;
  seasonalStage: WorldgenStageMetadata;
  seasonalMetrics: WorldgenSeasonalHydrologyMetrics;
  seasonalPhaseLocalRunoffM3S: Float32Array;
  seasonalPhaseSnowmeltRunoffM3S: Float32Array;
  seasonalPhaseSnowStorageMm: Float32Array;
  seasonalPhasePotentialDischargeM3S: Float32Array;
  seasonalPhaseRealizedDischargeM3S: Float32Array;
  seasonalFlowPresenceFraction: Float32Array;
  seasonalFlowRegime: Uint8Array;
  seasonalPhaseLakeSurfaceElevationM: Float32Array;
  seasonalPhaseLakeAreaM2: Float64Array;
  seasonalPhaseLakeVolumeM3: Float64Array;
}


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

export interface WorldgenRunoffMetrics {
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

export interface WorldgenLakeMetrics {
  sampleCount: number;
  lakeCount: number;
  endorheicLakeCount: number;
  overflowingLakeCount: number;
  terminalStorageLakeCount: number;
  lakeSampleCount: number;
  totalLakeAreaM2: number;
  totalLakeVolumeM3: number;
  maximumLakeAreaM2: number;
  maximumLakeDepthM: number;
  totalLakePrecipitationM3S: number;
  totalLakeEvaporationM3S: number;
  terminalRealizedDischargeM3S: number;
  maximumRealizedDischargeM3S: number;
  unreleasedStorageM3S: number;
  waterBalanceRelativeError: number;
  lakeParameterHash: string;
  climateHash: string;
  drainageHash: string;
  runoffHash: string;
  lakeHash: string;
}


export interface WorldgenSeasonalHydrologyMetrics {
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

export interface WorldgenSyntheticCommand { protocolVersion: number; requestId: number; type: 'generate-synthetic'; payload: WorldgenSyntheticRequest; }
export interface WorldgenTopologyCommand { protocolVersion: number; requestId: number; type: 'generate-topology'; payload: WorldgenTopologyRequest; }
export interface WorldgenTectonicsCommand { protocolVersion: number; requestId: number; type: 'generate-tectonics'; payload: WorldgenTectonicsRequest; }
export interface WorldgenGeologyCommand { protocolVersion: number; requestId: number; type: 'generate-geology'; payload: WorldgenGeologyRequest; }
export interface WorldgenLithosphereCommand { protocolVersion: number; requestId: number; type: 'generate-lithosphere'; payload: WorldgenLithosphereRequest; }
export interface WorldgenInheritanceCommand { protocolVersion: number; requestId: number; type: 'generate-inheritance'; payload: WorldgenInheritanceRequest; }
export interface WorldgenTopographyCommand { protocolVersion: number; requestId: number; type: 'generate-topography'; payload: WorldgenTopographyRequest; }
export interface WorldgenClimateCommand { protocolVersion: number; requestId: number; type: 'generate-climate'; payload: WorldgenClimateRequest; }
export interface WorldgenDrainageCommand { protocolVersion: number; requestId: number; type: 'generate-drainage'; payload: WorldgenDrainageRequest; }
export type WorldgenCommand = WorldgenSyntheticCommand | WorldgenTopologyCommand | WorldgenTectonicsCommand | WorldgenGeologyCommand | WorldgenLithosphereCommand | WorldgenInheritanceCommand | WorldgenTopographyCommand | WorldgenClimateCommand | WorldgenDrainageCommand;

export interface WorldgenGeneratedSyntheticEvent { protocolVersion: number; requestId: number; type: 'generated-synthetic'; payload: WorldgenSyntheticResult; }
export interface WorldgenGeneratedTopologyEvent { protocolVersion: number; requestId: number; type: 'generated-topology'; payload: WorldgenTopologyResult; }
export interface WorldgenGeneratedTectonicsEvent { protocolVersion: number; requestId: number; type: 'generated-tectonics'; payload: WorldgenTectonicsResult; }
export interface WorldgenGeneratedGeologyEvent { protocolVersion: number; requestId: number; type: 'generated-geology'; payload: WorldgenGeologyResult; }
export interface WorldgenGeneratedLithosphereEvent { protocolVersion: number; requestId: number; type: 'generated-lithosphere'; payload: WorldgenLithosphereResult; }
export interface WorldgenGeneratedInheritanceEvent { protocolVersion: number; requestId: number; type: 'generated-inheritance'; payload: WorldgenInheritanceResult; }
export interface WorldgenGeneratedTopographyEvent { protocolVersion: number; requestId: number; type: 'generated-topography'; payload: WorldgenTopographyResult; }
export interface WorldgenGeneratedClimateEvent { protocolVersion: number; requestId: number; type: 'generated-climate'; payload: WorldgenClimateResult; }
export interface WorldgenGeneratedDrainageEvent { protocolVersion: number; requestId: number; type: 'generated-drainage'; payload: WorldgenDrainageResult; }
export interface WorldgenGenerationProgressEvent { protocolVersion: number; requestId: number; type: 'progress'; payload: WorldgenGenerationProgress; }
export interface WorldgenErrorEvent { protocolVersion: number; requestId: number; type: 'error'; payload: { message: string }; }
export type WorldgenEvent = WorldgenGeneratedSyntheticEvent | WorldgenGeneratedTopologyEvent | WorldgenGeneratedTectonicsEvent | WorldgenGeneratedGeologyEvent | WorldgenGeneratedLithosphereEvent | WorldgenGeneratedInheritanceEvent | WorldgenGeneratedTopographyEvent | WorldgenGeneratedClimateEvent | WorldgenGeneratedDrainageEvent | WorldgenGenerationProgressEvent | WorldgenErrorEvent;

export function validateSyntheticRequest(request: WorldgenSyntheticRequest): void {
  if (!request.seed.trim()) throw new Error('Worldgen seed must not be empty.');
  for (const [name, value] of [['width', request.width], ['height', request.height]] as const) if (!Number.isInteger(value) || value <= 0) throw new Error(`Worldgen ${name} must be a positive integer.`);
  const samples = request.width * request.height;
  if (!Number.isSafeInteger(samples) || samples > WORLDGEN_SYNTHETIC_MAX_SAMPLES) throw new Error(`WG-0 synthetic diagnostics are limited to ${WORLDGEN_SYNTHETIC_MAX_SAMPLES.toLocaleString()} samples.`);
}

export function validateTopologyRequest(request: WorldgenTopologyRequest): void {
  if (!Number.isInteger(request.level) || request.level < 0 || request.level > WORLDGEN_TOPOLOGY_MAX_LEVEL) throw new Error(`WG-1 browser topology level must be an integer from 0 through ${WORLDGEN_TOPOLOGY_MAX_LEVEL}.`);
}

export function validateTectonicsRequest(request: WorldgenTectonicsRequest): void {
  if (!request.seed.trim()) throw new Error('WG-2 tectonic seed must not be empty.');
  if (!Number.isInteger(request.level) || request.level < 0 || request.level > WORLDGEN_TECTONICS_MAX_LEVEL) throw new Error(`WG-2 browser tectonics level must be an integer from 0 through ${WORLDGEN_TECTONICS_MAX_LEVEL}.`);
  if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES) throw new Error(`WG-2 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
  const samples = 10 * (4 ** request.level) + 2;
  if (request.plateCount > samples) throw new Error('WG-2 plate count cannot exceed topology sample count.');
}

export function validateGeologyRequest(request: WorldgenGeologyRequest): void {
  if (!request.seed.trim()) throw new Error('WG-3 geology seed must not be empty.');
  if (!Number.isInteger(request.level) || request.level < 0 || request.level > WORLDGEN_GEOLOGY_MAX_LEVEL) throw new Error(`WG-3 browser geology level must be an integer from 0 through ${WORLDGEN_GEOLOGY_MAX_LEVEL}.`);
  if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES) throw new Error(`WG-3 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
  const samples = 10 * (4 ** request.level) + 2;
  if (request.plateCount > samples) throw new Error('WG-3 plate count cannot exceed topology sample count.');
}

export function validateLithosphereRequest(request: WorldgenLithosphereRequest): void {
  if (!request.seed.trim()) throw new Error('WG-3.5 lithosphere seed must not be empty.');
  if (!Number.isInteger(request.level) || request.level < 0 || request.level > WORLDGEN_LITHOSPHERE_MAX_LEVEL) throw new Error(`WG-3.5 browser lithosphere level must be an integer from 0 through ${WORLDGEN_LITHOSPHERE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES) throw new Error(`WG-3.5 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
  const samples = 10 * (4 ** request.level) + 2;
  if (request.plateCount > samples) throw new Error('WG-3.5 plate count cannot exceed topology sample count.');
}

export function validateInheritanceRequest(request: WorldgenInheritanceRequest): void {
  if (!request.seed.trim()) throw new Error('WG-3.75 inheritance seed must not be empty.');
  if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_INHERITANCE_COARSE_MAX_LEVEL) throw new Error(`WG-3.75 coarse level must be an integer from 0 through ${WORLDGEN_INHERITANCE_COARSE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_INHERITANCE_FINE_MAX_LEVEL) throw new Error(`WG-3.75 fine level must be an integer from coarse level through ${WORLDGEN_INHERITANCE_FINE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES) throw new Error(`WG-3.75 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
  const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
  if (request.plateCount > coarseSamples) throw new Error('WG-3.75 plate count cannot exceed coarse topology sample count.');
}


export function validateTopographyRequest(request: WorldgenTopographyRequest): void {
  if (!request.seed.trim()) throw new Error('WG-4 topography seed must not be empty.');
  if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL) throw new Error(`WG-4 coarse level must be an integer from 0 through ${WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL) throw new Error(`WG-4 fine level must be an integer from coarse level through ${WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES) throw new Error(`WG-4 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
  const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
  if (request.plateCount > coarseSamples) throw new Error('WG-4 plate count cannot exceed coarse topology sample count.');
}


export function validateClimateRequest(request: WorldgenClimateRequest): void {
  if (!request.seed.trim()) throw new Error('WG-5 climate seed must not be empty.');
  if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_CLIMATE_COARSE_MAX_LEVEL) throw new Error(`WG-5 coarse level must be an integer from 0 through ${WORLDGEN_CLIMATE_COARSE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_CLIMATE_FINE_MAX_LEVEL) throw new Error(`WG-5 fine level must be an integer from coarse level through ${WORLDGEN_CLIMATE_FINE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES) throw new Error(`WG-5 plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
  const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
  if (request.plateCount > coarseSamples) throw new Error('WG-5 plate count cannot exceed coarse topology sample count.');
}


export function validateDrainageRequest(request: WorldgenDrainageRequest): void {
  if (!request.seed.trim()) throw new Error('WG-6A drainage seed must not be empty.');
  if (!Number.isInteger(request.coarseLevel) || request.coarseLevel < 0 || request.coarseLevel > WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL) throw new Error(`WG-6A coarse level must be an integer from 0 through ${WORLDGEN_DRAINAGE_COARSE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.fineLevel) || request.fineLevel < request.coarseLevel || request.fineLevel > WORLDGEN_DRAINAGE_FINE_MAX_LEVEL) throw new Error(`WG-6A fine level must be an integer from coarse level through ${WORLDGEN_DRAINAGE_FINE_MAX_LEVEL}.`);
  if (!Number.isInteger(request.plateCount) || request.plateCount < WORLDGEN_TECTONICS_MIN_PLATES || request.plateCount > WORLDGEN_TECTONICS_MAX_PLATES) throw new Error(`WG-6A plate count must be an integer from ${WORLDGEN_TECTONICS_MIN_PLATES} through ${WORLDGEN_TECTONICS_MAX_PLATES}.`);
  const coarseSamples = 10 * (4 ** request.coarseLevel) + 2;
  if (request.plateCount > coarseSamples) throw new Error('WG-6A plate count cannot exceed coarse topology sample count.');
}

export function worldgenSyntheticCommand(requestId: number, payload: WorldgenSyntheticRequest): WorldgenSyntheticCommand { validateSyntheticRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-synthetic', payload }; }
export function worldgenTopologyCommand(requestId: number, payload: WorldgenTopologyRequest): WorldgenTopologyCommand { validateTopologyRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-topology', payload }; }
export function worldgenTectonicsCommand(requestId: number, payload: WorldgenTectonicsRequest): WorldgenTectonicsCommand { validateTectonicsRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-tectonics', payload }; }
export function worldgenGeologyCommand(requestId: number, payload: WorldgenGeologyRequest): WorldgenGeologyCommand { validateGeologyRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-geology', payload }; }
export function worldgenLithosphereCommand(requestId: number, payload: WorldgenLithosphereRequest): WorldgenLithosphereCommand { validateLithosphereRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-lithosphere', payload }; }
export function worldgenInheritanceCommand(requestId: number, payload: WorldgenInheritanceRequest): WorldgenInheritanceCommand { validateInheritanceRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-inheritance', payload }; }

export function worldgenTopographyCommand(requestId: number, payload: WorldgenTopographyRequest): WorldgenTopographyCommand { validateTopographyRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-topography', payload }; }

export function worldgenClimateCommand(requestId: number, payload: WorldgenClimateRequest): WorldgenClimateCommand { validateClimateRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-climate', payload }; }
export function worldgenDrainageCommand(requestId: number, payload: WorldgenDrainageRequest): WorldgenDrainageCommand { validateDrainageRequest(payload); return { protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId, type: 'generate-drainage', payload }; }
