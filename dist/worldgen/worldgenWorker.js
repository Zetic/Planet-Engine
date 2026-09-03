import { WORLDGEN_PROTOCOL_VERSION, validateClimateRequest, validateGeologyRequest, validateInheritanceRequest, validateLithosphereRequest, validateSyntheticRequest, validateTopographyRequest, validateTectonicsRequest, validateTopologyRequest, } from './protocol.js';
const workerScope = self;
let wasmModulePromise = null;
function nowMs() { return globalThis.performance?.now?.() ?? Date.now(); }
async function loadWorldgenWasm() {
    if (wasmModulePromise)
        return wasmModulePromise;
    wasmModulePromise = (async () => {
        const moduleUrl = new URL('../../src/wasm-worldgen/interlink_worldgen_wasm.js', import.meta.url).href;
        let module;
        try {
            module = await import(moduleUrl);
        }
        catch (error) {
            const detail = error instanceof Error ? error.message : String(error);
            throw new Error(`Planet Engine WASM package is not available. ${detail}`);
        }
        await module.default();
        const actual = module.worldgen_protocol_version();
        if (actual !== WORLDGEN_PROTOCOL_VERSION)
            throw new Error(`Planet Engine WASM protocol ${actual} does not match browser protocol ${WORLDGEN_PROTOCOL_VERSION}.`);
        return module;
    })();
    return wasmModulePromise;
}
async function generateSynthetic(command) {
    validateSyntheticRequest(command.payload);
    const module = await loadWorldgenWasm();
    const startedAt = nowMs();
    const output = new module.WasmWorldgenDiagnostic(command.payload.seed, command.payload.width, command.payload.height);
    try {
        const values = output.values();
        return { engineVersion: output.generator_version(), width: output.width(), height: output.height(), values, statistics: { sampleCount: Number(output.sample_count()), minimum: output.minimum(), maximum: output.maximum(), mean: output.mean(), fieldHash: output.field_hash_hex() }, stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) } };
    }
    finally {
        output.free();
    }
}
async function generateTopology(command) {
    validateTopologyRequest(command.payload);
    const module = await loadWorldgenWasm();
    const startedAt = nowMs();
    const output = new module.WasmWorldgenTopology(command.payload.level);
    try {
        const positions = output.positions();
        const faces = output.faces();
        const neighborOffsets = output.neighbor_offsets();
        const neighbors = output.neighbors();
        const neighborArcLengthsRad = output.neighbor_arc_lengths_rad();
        const neighborInterfaceArcLengthsRad = output.neighbor_interface_arc_lengths_rad();
        const areaSteradians = output.area_steradians();
        const birthLevels = output.birth_levels();
        const parentEdges = output.parent_edges();
        return { engineVersion: output.generator_version(), level: output.level(), durationMs: Math.max(0, nowMs() - startedAt), metrics: { sampleCount: output.sample_count(), edgeCount: output.edge_count(), faceCount: output.face_count(), fiveNeighborCount: output.five_neighbor_count(), sixNeighborCount: output.six_neighbor_count(), totalAreaSteradians: output.total_area_steradians(), minimumAreaSteradians: output.minimum_area_steradians(), maximumAreaSteradians: output.maximum_area_steradians(), meanAreaSteradians: output.mean_area_steradians(), areaCoefficientOfVariation: output.area_coefficient_of_variation(), minimumEdgeArcRadians: output.minimum_edge_arc_radians(), maximumEdgeArcRadians: output.maximum_edge_arc_radians(), meanEdgeArcRadians: output.mean_edge_arc_radians(), edgeCoefficientOfVariation: output.edge_coefficient_of_variation(), minimumInterfaceArcRadians: output.minimum_interface_arc_radians(), maximumInterfaceArcRadians: output.maximum_interface_arc_radians(), meanInterfaceArcRadians: output.mean_interface_arc_radians(), interfaceCoefficientOfVariation: output.interface_coefficient_of_variation(), topologyHash: output.topology_hash_hex() }, positions, faces, neighborOffsets, neighbors, neighborArcLengthsRad, neighborInterfaceArcLengthsRad, areaSteradians, birthLevels, parentEdges };
    }
    finally {
        output.free();
    }
}
async function generateTectonics(command) {
    validateTectonicsRequest(command.payload);
    const module = await loadWorldgenWasm();
    const startedAt = nowMs();
    const output = new module.WasmWorldgenTectonics(command.payload.seed, command.payload.level, command.payload.plateCount);
    try {
        const positions = output.positions();
        const faces = output.faces();
        const neighborOffsets = output.neighbor_offsets();
        const neighbors = output.neighbors();
        const plateIds = output.plate_ids();
        const plateSeedSamples = output.plate_seed_samples();
        const plateEulerPoles = output.plate_euler_poles();
        const plateAngularVelocitiesRadPerMyr = output.plate_angular_velocities_rad_per_myr();
        const plateAreaSteradians = output.plate_area_steradians();
        const boundarySamples = output.boundary_samples();
        const boundaryPlateIds = output.boundary_plate_ids();
        const boundaryKinds = output.boundary_kinds();
        const boundaryNormalRatesMPerYear = output.boundary_normal_rates_m_per_year();
        const boundaryShearRatesMPerYear = output.boundary_shear_rates_m_per_year();
        return { engineVersion: output.generator_version(), level: output.level(), topologyHash: output.topology_hash_hex(), stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) }, metrics: { sampleCount: output.sample_count(), plateCount: output.plate_count(), boundaryEdgeCount: output.boundary_edge_count(), convergentEdgeCount: output.convergent_edge_count(), divergentEdgeCount: output.divergent_edge_count(), transformEdgeCount: output.transform_edge_count(), minimumPlateAreaFraction: output.minimum_plate_area_fraction(), maximumPlateAreaFraction: output.maximum_plate_area_fraction(), meanPlateAreaFraction: output.mean_plate_area_fraction(), minimumSeedSeparationRad: output.minimum_seed_separation_rad(), meanReferenceSpeedMmPerYear: output.mean_reference_speed_mm_per_year(), tectonicHash: output.tectonic_hash_hex() }, positions, faces, neighborOffsets, neighbors, plateIds, plateSeedSamples, plateEulerPoles, plateAngularVelocitiesRadPerMyr, plateAreaSteradians, boundarySamples, boundaryPlateIds, boundaryKinds, boundaryNormalRatesMPerYear, boundaryShearRatesMPerYear };
    }
    finally {
        output.free();
    }
}
async function generateGeology(command) {
    validateGeologyRequest(command.payload);
    const module = await loadWorldgenWasm();
    const startedAt = nowMs();
    const output = new module.WasmWorldgenGeology(command.payload.seed, command.payload.level, command.payload.plateCount);
    try {
        const positions = output.positions();
        const faces = output.faces();
        const neighborOffsets = output.neighbor_offsets();
        const neighbors = output.neighbors();
        const plateIds = output.plate_ids();
        const boundarySamples = output.boundary_samples();
        const boundaryPlateIds = output.boundary_plate_ids();
        const boundaryKinds = output.boundary_kinds();
        const crustKind = output.crust_kind();
        const crustProvinceId = output.crust_province_id();
        const crustAgeMyr = output.crust_age_myr();
        const crustThicknessKm = output.crust_thickness_km();
        const crustDensityKgPerM3 = output.crust_density_kg_per_m3();
        const buoyancyIndex = output.buoyancy_index();
        const orogenicHistory = output.orogenic_history();
        const riftHistory = output.rift_history();
        const ridgeHistory = output.ridge_history();
        const subductionHistory = output.subduction_history();
        const trenchHistory = output.trench_history();
        const volcanicArcHistory = output.volcanic_arc_history();
        const transformHistory = output.transform_history();
        const subsidenceHistory = output.subsidence_history();
        const basinPotential = output.basin_potential();
        const crustalStrain = output.crustal_strain();
        const geologicalBoundaryRegimes = output.geological_boundary_regimes();
        const subductionPolarities = output.subduction_polarities();
        const plateScaleClasses = output.plate_scale_classes();
        const plateContinentalFractions = output.plate_continental_fractions();
        const plateTransitionalFractions = output.plate_transitional_fractions();
        const plateOceanicFractions = output.plate_oceanic_fractions();
        const plateMeanCrustAgeMyr = output.plate_mean_crust_age_myr();
        const plateMeanCrustThicknessKm = output.plate_mean_crust_thickness_km();
        return { engineVersion: output.generator_version(), level: output.level(), topologyHash: output.topology_hash_hex(), plateCount: output.plate_count(), boundaryEdgeCount: output.boundary_edge_count(), stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) }, provinceSeed: output.province_seed_hex(), propertySeed: output.property_seed_hex(), historySeed: output.history_seed_hex(), metrics: { sampleCount: output.sample_count(), continentalAreaFraction: output.continental_area_fraction(), transitionalAreaFraction: output.transitional_area_fraction(), oceanicAreaFraction: output.oceanic_area_fraction(), meanContinentalAgeMyr: output.mean_continental_age_myr(), meanOceanicAgeMyr: output.mean_oceanic_age_myr(), meanContinentalThicknessKm: output.mean_continental_thickness_km(), meanOceanicThicknessKm: output.mean_oceanic_thickness_km(), oceanicSubductionEdges: output.oceanic_subduction_edges(), oceanContinentSubductionEdges: output.ocean_continent_subduction_edges(), continentalCollisionEdges: output.continental_collision_edges(), oceanicRidgeEdges: output.oceanic_ridge_edges(), continentalRiftEdges: output.continental_rift_edges(), transitionalDivergenceEdges: output.transitional_divergence_edges(), transformEdges: output.transform_edges(), geologyHash: output.geology_hash_hex(), tectonicHash: output.tectonic_hash_hex() }, positions, faces, neighborOffsets, neighbors, plateIds, boundarySamples, boundaryPlateIds, boundaryKinds, crustKind, crustProvinceId, crustAgeMyr, crustThicknessKm, crustDensityKgPerM3, buoyancyIndex, orogenicHistory, riftHistory, ridgeHistory, subductionHistory, trenchHistory, volcanicArcHistory, transformHistory, subsidenceHistory, basinPotential, crustalStrain, geologicalBoundaryRegimes, subductionPolarities, plateScaleClasses, plateContinentalFractions, plateTransitionalFractions, plateOceanicFractions, plateMeanCrustAgeMyr, plateMeanCrustThicknessKm };
    }
    finally {
        output.free();
    }
}
async function generateLithosphere(command) {
    validateLithosphereRequest(command.payload);
    const module = await loadWorldgenWasm();
    const startedAt = nowMs();
    const output = new module.WasmWorldgenLithosphere(command.payload.seed, command.payload.level, command.payload.plateCount);
    try {
        const positions = output.positions();
        const faces = output.faces();
        const neighborOffsets = output.neighbor_offsets();
        const neighbors = output.neighbors();
        const plateIds = output.plate_ids();
        const boundarySamples = output.boundary_samples();
        const boundaryKinds = output.boundary_kinds();
        const crustKind = output.crust_kind();
        const geologicalBoundaryRegimes = output.geological_boundary_regimes();
        const orogenicHistory = output.orogenic_history();
        const riftHistory = output.rift_history();
        const ridgeHistory = output.ridge_history();
        const subductionHistory = output.subduction_history();
        const transformHistory = output.transform_history();
        const crustalStrain = output.crustal_strain();
        const strengthIndex = output.strength_index();
        const weaknessIndex = output.weakness_index();
        const effectiveElasticThicknessKm = output.effective_elastic_thickness_km();
        const thermalAnomalyIndex = output.thermal_anomaly_index();
        const mantleUpwellingIndex = output.mantle_upwelling_index();
        const mantleDynamicSupportIndex = output.mantle_dynamic_support_index();
        const compensatedBuoyancyIndex = output.compensated_buoyancy_index();
        const structuralFabricStrength = output.structural_fabric_strength();
        const structuralZoneKind = output.structural_zone_kind();
        const fragmentationPropensity = output.fragmentation_propensity();
        const fragmentIds = output.fragment_ids();
        const kinematicDomainIds = output.kinematic_domain_ids();
        const fragmentParentPlateIds = output.fragment_parent_plate_ids();
        const fragmentKinds = output.fragment_kinds();
        const fragmentSeedSamples = output.fragment_seed_samples();
        const fragmentAreaSteradians = output.fragment_area_steradians();
        const fragmentAreaFractionsOfParent = output.fragment_area_fractions_of_parent();
        const fragmentMeanWeakness = output.fragment_mean_weakness();
        const fragmentMeanPropensity = output.fragment_mean_propensity();
        const fragmentAngularVelocitiesRadPerMyr = output.fragment_angular_velocities_rad_per_myr();
        return {
            engineVersion: output.generator_version(), level: output.level(), topologyHash: output.topology_hash_hex(), plateCount: output.plate_count(), boundaryEdgeCount: output.boundary_edge_count(),
            stage: { id: output.stage_id(), version: output.stage_version(), stageSeed: output.stage_seed_hex(), durationMs: Math.max(0, nowMs() - startedAt) }, mechanicalSeed: output.mechanical_seed_hex(), mantleSeed: output.mantle_seed_hex(), refinementSeed: output.refinement_seed_hex(),
            metrics: { sampleCount: output.sample_count(), meanStrengthIndex: output.mean_strength_index(), meanWeaknessIndex: output.mean_weakness_index(), meanEffectiveElasticThicknessKm: output.mean_effective_elastic_thickness_km(), meanMantleUpwellingIndex: output.mean_mantle_upwelling_index(), meanDynamicSupportIndex: output.mean_dynamic_support_index(), sutureSampleCount: output.suture_sample_count(), riftZoneSampleCount: output.rift_zone_sample_count(), transformZoneSampleCount: output.transform_zone_sample_count(), continentalMarginSampleCount: output.continental_margin_sample_count(), tectonicFragmentCount: output.tectonic_fragment_count(), microplateCount: output.microplate_count(), terraneCount: output.terrane_count(), fragmentedAreaFraction: output.fragmented_area_fraction(), lithosphereHash: output.lithosphere_hash_hex(), geologyHash: output.geology_hash_hex(), tectonicHash: output.tectonic_hash_hex() },
            positions, faces, neighborOffsets, neighbors, plateIds, boundarySamples, boundaryKinds, crustKind, geologicalBoundaryRegimes, orogenicHistory, riftHistory, ridgeHistory, subductionHistory, transformHistory, crustalStrain, strengthIndex, weaknessIndex, effectiveElasticThicknessKm, thermalAnomalyIndex, mantleUpwellingIndex, mantleDynamicSupportIndex, compensatedBuoyancyIndex, structuralFabricStrength, structuralZoneKind, fragmentationPropensity, fragmentIds, kinematicDomainIds, fragmentParentPlateIds, fragmentKinds, fragmentSeedSamples, fragmentAreaSteradians, fragmentAreaFractionsOfParent, fragmentMeanWeakness, fragmentMeanPropensity, fragmentAngularVelocitiesRadPerMyr,
        };
    }
    finally {
        output.free();
    }
}
async function generateInheritance(command) {
    validateInheritanceRequest(command.payload);
    const module = await loadWorldgenWasm();
    const startedAt = nowMs();
    const output = new module.WasmWorldgenInheritance(command.payload.seed, command.payload.coarseLevel, command.payload.fineLevel, command.payload.plateCount);
    try {
        const positions = output.positions();
        const faces = output.faces();
        const neighborOffsets = output.neighbor_offsets();
        const neighbors = output.neighbors();
        const nearestCoarseSource = output.nearest_coarse_source();
        const inheritedSampleMask = output.inherited_sample_mask();
        const plateIds = output.plate_ids();
        const crustKind = output.crust_kind();
        const crustProvinceId = output.crust_province_id();
        const crustAgeMyr = output.crust_age_myr();
        const crustThicknessKm = output.crust_thickness_km();
        const crustDensityKgPerM3 = output.crust_density_kg_per_m3();
        const buoyancyIndex = output.buoyancy_index();
        const orogenicHistory = output.orogenic_history();
        const riftHistory = output.rift_history();
        const ridgeHistory = output.ridge_history();
        const subductionHistory = output.subduction_history();
        const trenchHistory = output.trench_history();
        const volcanicArcHistory = output.volcanic_arc_history();
        const transformHistory = output.transform_history();
        const subsidenceHistory = output.subsidence_history();
        const basinPotential = output.basin_potential();
        const crustalStrain = output.crustal_strain();
        const strengthIndex = output.strength_index();
        const weaknessIndex = output.weakness_index();
        const effectiveElasticThicknessKm = output.effective_elastic_thickness_km();
        const thermalAnomalyIndex = output.thermal_anomaly_index();
        const mantleUpwellingIndex = output.mantle_upwelling_index();
        const mantleDynamicSupportIndex = output.mantle_dynamic_support_index();
        const compensatedBuoyancyIndex = output.compensated_buoyancy_index();
        const structuralFabricStrength = output.structural_fabric_strength();
        const structuralZoneKind = output.structural_zone_kind();
        const fragmentationPropensity = output.fragmentation_propensity();
        const fragmentIds = output.fragment_ids();
        const kinematicDomainIds = output.kinematic_domain_ids();
        const boundarySamples = output.boundary_samples();
        const boundaryKinds = output.boundary_kinds();
        const geologicalBoundaryRegimes = output.geological_boundary_regimes();
        const subductionPolarities = output.subduction_polarities();
        const boundaryNormalRatesMPerYear = output.boundary_normal_rates_m_per_year();
        const boundaryShearRatesMPerYear = output.boundary_shear_rates_m_per_year();
        const boundaryCoarseSourceIndices = output.boundary_coarse_source_indices();
        return {
            engineVersion: output.generator_version(), coarseLevel: output.coarse_level(), fineLevel: output.fine_level(),
            stage: { id: output.stage_id(), version: output.stage_version(), durationMs: Math.max(0, nowMs() - startedAt) },
            metrics: { coarseSampleCount: output.coarse_sample_count(), fineSampleCount: output.fine_sample_count(), addedSampleCount: output.added_sample_count(), plateCount: output.plate_count(), fineBoundaryEdgeCount: output.fine_boundary_edge_count(), coarseTopologyHash: output.coarse_topology_hash_hex(), fineTopologyHash: output.fine_topology_hash_hex(), tectonicHash: output.tectonic_hash_hex(), geologyHash: output.geology_hash_hex(), lithosphereHash: output.lithosphere_hash_hex(), provenanceHash: output.provenance_hash_hex(), parameterHash: output.parameter_hash_hex(), inheritanceHash: output.inheritance_hash_hex(), boundaryHash: output.boundary_hash_hex() },
            parameters: { radiusM: output.radius_m(), surfaceGravityMS2: output.surface_gravity_m_s2(), surfaceWaterMassKg: output.surface_water_mass_kg(), equivalentGlobalWaterDepthM: output.equivalent_global_water_depth_m(), oceanWaterDensityKgPerM3: output.ocean_water_density_kg_per_m3(), isostaticMantleDensityKgPerM3: output.isostatic_mantle_density_kg_per_m3(), internalHeatFluxWPerM2: output.internal_heat_flux_w_per_m2(), mantleThermalExpansivityPerK: output.mantle_thermal_expansivity_per_k() },
            positions, faces, neighborOffsets, neighbors, nearestCoarseSource, inheritedSampleMask, plateIds, crustKind, crustProvinceId, crustAgeMyr, crustThicknessKm, crustDensityKgPerM3, buoyancyIndex, orogenicHistory, riftHistory, ridgeHistory, subductionHistory, trenchHistory, volcanicArcHistory, transformHistory, subsidenceHistory, basinPotential, crustalStrain, strengthIndex, weaknessIndex, effectiveElasticThicknessKm, thermalAnomalyIndex, mantleUpwellingIndex, mantleDynamicSupportIndex, compensatedBuoyancyIndex, structuralFabricStrength, structuralZoneKind, fragmentationPropensity, fragmentIds, kinematicDomainIds, boundarySamples, boundaryKinds, geologicalBoundaryRegimes, subductionPolarities, boundaryNormalRatesMPerYear, boundaryShearRatesMPerYear, boundaryCoarseSourceIndices,
        };
    }
    finally {
        output.free();
    }
}
async function generateTopography(command) {
    validateTopographyRequest(command.payload);
    const module = await loadWorldgenWasm();
    const startedAt = nowMs();
    const output = new module.WasmWorldgenTopography(command.payload.seed, command.payload.coarseLevel, command.payload.fineLevel, command.payload.plateCount);
    try {
        const positions = output.positions();
        const faces = output.faces();
        const neighborOffsets = output.neighbor_offsets();
        const neighbors = output.neighbors();
        const plateIds = output.plate_ids();
        const crustKind = output.crust_kind();
        const nearestCoarseSource = output.nearest_coarse_source();
        const inheritedSampleMask = output.inherited_sample_mask();
        const crustAgeMyr = output.crust_age_myr();
        const crustThicknessKm = output.crust_thickness_km();
        const orogenicHistory = output.orogenic_history();
        const ridgeHistory = output.ridge_history();
        const trenchHistory = output.trench_history();
        const strengthIndex = output.strength_index();
        const weaknessIndex = output.weakness_index();
        const mantleDynamicSupportIndex = output.mantle_dynamic_support_index();
        const structuralZoneKind = output.structural_zone_kind();
        const fragmentationPropensity = output.fragmentation_propensity();
        const kinematicDomainIds = output.kinematic_domain_ids();
        const boundarySamples = output.boundary_samples();
        const boundaryKinds = output.boundary_kinds();
        const geologicalBoundaryRegimes = output.geological_boundary_regimes();
        const boundaryCoarseSourceIndices = output.boundary_coarse_source_indices();
        const isostaticElevationM = output.isostatic_elevation_m();
        const thermalElevationM = output.thermal_elevation_m();
        const orogenicElevationM = output.orogenic_elevation_m();
        const ridgeElevationM = output.ridge_elevation_m();
        const riftBasinElevationM = output.rift_basin_elevation_m();
        const trenchElevationM = output.trench_elevation_m();
        const arcElevationM = output.arc_elevation_m();
        const mantleDynamicElevationM = output.mantle_dynamic_elevation_m();
        const solidElevationM = output.solid_elevation_m();
        const elevationAboveSeaLevelM = output.elevation_above_sea_level_m();
        const waterDepthM = output.water_depth_m();
        const submergedMask = output.submerged_mask();
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
    }
    finally {
        output.free();
    }
}
async function generateClimate(command) {
    validateClimateRequest(command.payload);
    const module = await loadWorldgenWasm();
    const startedAt = nowMs();
    const output = new module.WasmWorldgenClimate(command.payload.seed, command.payload.coarseLevel, command.payload.fineLevel, command.payload.plateCount);
    try {
        const positions = output.positions();
        const faces = output.faces();
        const neighborOffsets = output.neighbor_offsets();
        const neighbors = output.neighbors();
        const plateIds = output.plate_ids();
        const crustKind = output.crust_kind();
        const nearestCoarseSource = output.nearest_coarse_source();
        const inheritedSampleMask = output.inherited_sample_mask();
        const crustAgeMyr = output.crust_age_myr();
        const crustThicknessKm = output.crust_thickness_km();
        const orogenicHistory = output.orogenic_history();
        const ridgeHistory = output.ridge_history();
        const trenchHistory = output.trench_history();
        const strengthIndex = output.strength_index();
        const weaknessIndex = output.weakness_index();
        const mantleDynamicSupportIndex = output.mantle_dynamic_support_index();
        const structuralZoneKind = output.structural_zone_kind();
        const fragmentationPropensity = output.fragmentation_propensity();
        const kinematicDomainIds = output.kinematic_domain_ids();
        const boundarySamples = output.boundary_samples();
        const boundaryKinds = output.boundary_kinds();
        const geologicalBoundaryRegimes = output.geological_boundary_regimes();
        const boundaryCoarseSourceIndices = output.boundary_coarse_source_indices();
        const isostaticElevationM = output.isostatic_elevation_m();
        const thermalElevationM = output.thermal_elevation_m();
        const orogenicElevationM = output.orogenic_elevation_m();
        const ridgeElevationM = output.ridge_elevation_m();
        const riftBasinElevationM = output.rift_basin_elevation_m();
        const trenchElevationM = output.trench_elevation_m();
        const arcElevationM = output.arc_elevation_m();
        const mantleDynamicElevationM = output.mantle_dynamic_elevation_m();
        const solidElevationM = output.solid_elevation_m();
        const elevationAboveSeaLevelM = output.elevation_above_sea_level_m();
        const waterDepthM = output.water_depth_m();
        const submergedMask = output.submerged_mask();
        const annualMeanInsolationWM2 = output.annual_mean_insolation_w_m2();
        const seasonalInsolationAmplitudeWM2 = output.seasonal_insolation_amplitude_w_m2();
        const temperatureMeanK = output.temperature_mean_k();
        const temperatureAnnualCosK = output.temperature_annual_cos_k();
        const temperatureAnnualSinK = output.temperature_annual_sin_k();
        const temperatureMinK = output.temperature_min_k();
        const temperatureMaxK = output.temperature_max_k();
        const localPressurePa = output.local_pressure_pa();
        const windEastMeanMS = output.wind_east_mean_m_s();
        const windNorthMeanMS = output.wind_north_mean_m_s();
        const windEastAnnualCosMS = output.wind_east_annual_cos_m_s();
        const windEastAnnualSinMS = output.wind_east_annual_sin_m_s();
        const windNorthAnnualCosMS = output.wind_north_annual_cos_m_s();
        const windNorthAnnualSinMS = output.wind_north_annual_sin_m_s();
        const seaSurfaceTemperatureMeanK = output.sea_surface_temperature_mean_k();
        const seaSurfaceTemperatureAnnualCosK = output.sea_surface_temperature_annual_cos_k();
        const seaSurfaceTemperatureAnnualSinK = output.sea_surface_temperature_annual_sin_k();
        const currentEastMeanMS = output.current_east_mean_m_s();
        const currentNorthMeanMS = output.current_north_mean_m_s();
        const currentEastAnnualCosMS = output.current_east_annual_cos_m_s();
        const currentEastAnnualSinMS = output.current_east_annual_sin_m_s();
        const currentNorthAnnualCosMS = output.current_north_annual_cos_m_s();
        const currentNorthAnnualSinMS = output.current_north_annual_sin_m_s();
        const currentSpeedMeanMS = output.current_speed_mean_m_s();
        const oceanHeatTransportIndex = output.ocean_heat_transport_index();
        const specificHumidityMean = output.specific_humidity_mean();
        const annualPrecipitationMm = output.annual_precipitation_mm();
        const precipitationSeasonality = output.precipitation_seasonality();
        const potentialEvaporationMm = output.potential_evaporation_mm();
        const moistureBalanceMm = output.moisture_balance_mm();
        const aridityIndex = output.aridity_index();
        const snowfallFraction = output.snowfall_fraction();
        const persistentSnowPotential = output.persistent_snow_potential();
        const seaIcePotential = output.sea_ice_potential();
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
            annualMeanInsolationWM2, seasonalInsolationAmplitudeWM2, temperatureMeanK, temperatureAnnualCosK, temperatureAnnualSinK, temperatureMinK, temperatureMaxK, localPressurePa, windEastMeanMS, windNorthMeanMS, windEastAnnualCosMS, windEastAnnualSinMS, windNorthAnnualCosMS, windNorthAnnualSinMS, seaSurfaceTemperatureMeanK, seaSurfaceTemperatureAnnualCosK, seaSurfaceTemperatureAnnualSinK, currentEastMeanMS, currentNorthMeanMS, currentEastAnnualCosMS, currentEastAnnualSinMS, currentNorthAnnualCosMS, currentNorthAnnualSinMS, currentSpeedMeanMS, oceanHeatTransportIndex, specificHumidityMean, annualPrecipitationMm, precipitationSeasonality, potentialEvaporationMm, moistureBalanceMm, aridityIndex, snowfallFraction, persistentSnowPotential, seaIcePotential,
        };
    }
    finally {
        output.free();
    }
}
workerScope.addEventListener('message', async (messageEvent) => {
    const command = messageEvent.data;
    try {
        if (!command || command.protocolVersion !== WORLDGEN_PROTOCOL_VERSION)
            throw new Error(`Worldgen protocol must be ${WORLDGEN_PROTOCOL_VERSION}.`);
        if (command.type === 'generate-synthetic') {
            const result = await generateSynthetic(command);
            workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-synthetic', payload: result }, [result.values.buffer]);
            return;
        }
        if (command.type === 'generate-topology') {
            const result = await generateTopology(command);
            workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-topology', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.neighborArcLengthsRad.buffer, result.neighborInterfaceArcLengthsRad.buffer, result.areaSteradians.buffer, result.birthLevels.buffer, result.parentEdges.buffer]);
            return;
        }
        if (command.type === 'generate-tectonics') {
            const result = await generateTectonics(command);
            workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-tectonics', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.plateSeedSamples.buffer, result.plateEulerPoles.buffer, result.plateAngularVelocitiesRadPerMyr.buffer, result.plateAreaSteradians.buffer, result.boundarySamples.buffer, result.boundaryPlateIds.buffer, result.boundaryKinds.buffer, result.boundaryNormalRatesMPerYear.buffer, result.boundaryShearRatesMPerYear.buffer]);
            return;
        }
        if (command.type === 'generate-geology') {
            const result = await generateGeology(command);
            workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-geology', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.boundarySamples.buffer, result.boundaryPlateIds.buffer, result.boundaryKinds.buffer, result.crustKind.buffer, result.crustProvinceId.buffer, result.crustAgeMyr.buffer, result.crustThicknessKm.buffer, result.crustDensityKgPerM3.buffer, result.buoyancyIndex.buffer, result.orogenicHistory.buffer, result.riftHistory.buffer, result.ridgeHistory.buffer, result.subductionHistory.buffer, result.trenchHistory.buffer, result.volcanicArcHistory.buffer, result.transformHistory.buffer, result.subsidenceHistory.buffer, result.basinPotential.buffer, result.crustalStrain.buffer, result.geologicalBoundaryRegimes.buffer, result.subductionPolarities.buffer, result.plateScaleClasses.buffer, result.plateContinentalFractions.buffer, result.plateTransitionalFractions.buffer, result.plateOceanicFractions.buffer, result.plateMeanCrustAgeMyr.buffer, result.plateMeanCrustThicknessKm.buffer]);
            return;
        }
        if (command.type === 'generate-lithosphere') {
            const result = await generateLithosphere(command);
            workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-lithosphere', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.boundarySamples.buffer, result.boundaryKinds.buffer, result.crustKind.buffer, result.geologicalBoundaryRegimes.buffer, result.orogenicHistory.buffer, result.riftHistory.buffer, result.ridgeHistory.buffer, result.subductionHistory.buffer, result.transformHistory.buffer, result.crustalStrain.buffer, result.strengthIndex.buffer, result.weaknessIndex.buffer, result.effectiveElasticThicknessKm.buffer, result.thermalAnomalyIndex.buffer, result.mantleUpwellingIndex.buffer, result.mantleDynamicSupportIndex.buffer, result.compensatedBuoyancyIndex.buffer, result.structuralFabricStrength.buffer, result.structuralZoneKind.buffer, result.fragmentationPropensity.buffer, result.fragmentIds.buffer, result.kinematicDomainIds.buffer, result.fragmentParentPlateIds.buffer, result.fragmentKinds.buffer, result.fragmentSeedSamples.buffer, result.fragmentAreaSteradians.buffer, result.fragmentAreaFractionsOfParent.buffer, result.fragmentMeanWeakness.buffer, result.fragmentMeanPropensity.buffer, result.fragmentAngularVelocitiesRadPerMyr.buffer]);
            return;
        }
        if (command.type === 'generate-inheritance') {
            const result = await generateInheritance(command);
            workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-inheritance', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.nearestCoarseSource.buffer, result.inheritedSampleMask.buffer, result.plateIds.buffer, result.crustKind.buffer, result.crustProvinceId.buffer, result.crustAgeMyr.buffer, result.crustThicknessKm.buffer, result.crustDensityKgPerM3.buffer, result.buoyancyIndex.buffer, result.orogenicHistory.buffer, result.riftHistory.buffer, result.ridgeHistory.buffer, result.subductionHistory.buffer, result.trenchHistory.buffer, result.volcanicArcHistory.buffer, result.transformHistory.buffer, result.subsidenceHistory.buffer, result.basinPotential.buffer, result.crustalStrain.buffer, result.strengthIndex.buffer, result.weaknessIndex.buffer, result.effectiveElasticThicknessKm.buffer, result.thermalAnomalyIndex.buffer, result.mantleUpwellingIndex.buffer, result.mantleDynamicSupportIndex.buffer, result.compensatedBuoyancyIndex.buffer, result.structuralFabricStrength.buffer, result.structuralZoneKind.buffer, result.fragmentationPropensity.buffer, result.fragmentIds.buffer, result.kinematicDomainIds.buffer, result.boundarySamples.buffer, result.boundaryKinds.buffer, result.geologicalBoundaryRegimes.buffer, result.subductionPolarities.buffer, result.boundaryNormalRatesMPerYear.buffer, result.boundaryShearRatesMPerYear.buffer, result.boundaryCoarseSourceIndices.buffer]);
            return;
        }
        if (command.type === 'generate-topography') {
            const result = await generateTopography(command);
            workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-topography', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.crustKind.buffer, result.nearestCoarseSource.buffer, result.inheritedSampleMask.buffer, result.crustAgeMyr.buffer, result.crustThicknessKm.buffer, result.orogenicHistory.buffer, result.ridgeHistory.buffer, result.trenchHistory.buffer, result.strengthIndex.buffer, result.weaknessIndex.buffer, result.mantleDynamicSupportIndex.buffer, result.structuralZoneKind.buffer, result.fragmentationPropensity.buffer, result.kinematicDomainIds.buffer, result.boundarySamples.buffer, result.boundaryKinds.buffer, result.geologicalBoundaryRegimes.buffer, result.boundaryCoarseSourceIndices.buffer, result.isostaticElevationM.buffer, result.thermalElevationM.buffer, result.orogenicElevationM.buffer, result.ridgeElevationM.buffer, result.riftBasinElevationM.buffer, result.trenchElevationM.buffer, result.arcElevationM.buffer, result.mantleDynamicElevationM.buffer, result.solidElevationM.buffer, result.elevationAboveSeaLevelM.buffer, result.waterDepthM.buffer, result.submergedMask.buffer]);
            return;
        }
        if (command.type === 'generate-climate') {
            const result = await generateClimate(command);
            workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-climate', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.crustKind.buffer, result.nearestCoarseSource.buffer, result.inheritedSampleMask.buffer, result.crustAgeMyr.buffer, result.crustThicknessKm.buffer, result.orogenicHistory.buffer, result.ridgeHistory.buffer, result.trenchHistory.buffer, result.strengthIndex.buffer, result.weaknessIndex.buffer, result.mantleDynamicSupportIndex.buffer, result.structuralZoneKind.buffer, result.fragmentationPropensity.buffer, result.kinematicDomainIds.buffer, result.boundarySamples.buffer, result.boundaryKinds.buffer, result.geologicalBoundaryRegimes.buffer, result.boundaryCoarseSourceIndices.buffer, result.isostaticElevationM.buffer, result.thermalElevationM.buffer, result.orogenicElevationM.buffer, result.ridgeElevationM.buffer, result.riftBasinElevationM.buffer, result.trenchElevationM.buffer, result.arcElevationM.buffer, result.mantleDynamicElevationM.buffer, result.solidElevationM.buffer, result.elevationAboveSeaLevelM.buffer, result.waterDepthM.buffer, result.submergedMask.buffer, result.annualMeanInsolationWM2.buffer, result.seasonalInsolationAmplitudeWM2.buffer, result.temperatureMeanK.buffer, result.temperatureAnnualCosK.buffer, result.temperatureAnnualSinK.buffer, result.temperatureMinK.buffer, result.temperatureMaxK.buffer, result.localPressurePa.buffer, result.windEastMeanMS.buffer, result.windNorthMeanMS.buffer, result.windEastAnnualCosMS.buffer, result.windEastAnnualSinMS.buffer, result.windNorthAnnualCosMS.buffer, result.windNorthAnnualSinMS.buffer, result.seaSurfaceTemperatureMeanK.buffer, result.seaSurfaceTemperatureAnnualCosK.buffer, result.seaSurfaceTemperatureAnnualSinK.buffer, result.currentEastMeanMS.buffer, result.currentNorthMeanMS.buffer, result.currentEastAnnualCosMS.buffer, result.currentEastAnnualSinMS.buffer, result.currentNorthAnnualCosMS.buffer, result.currentNorthAnnualSinMS.buffer, result.currentSpeedMeanMS.buffer, result.oceanHeatTransportIndex.buffer, result.specificHumidityMean.buffer, result.annualPrecipitationMm.buffer, result.precipitationSeasonality.buffer, result.potentialEvaporationMm.buffer, result.moistureBalanceMm.buffer, result.aridityIndex.buffer, result.snowfallFraction.buffer, result.persistentSnowPotential.buffer, result.seaIcePotential.buffer]);
            return;
        }
        throw new Error(`Unsupported worldgen command '${String(command.type)}'.`);
    }
    catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command?.requestId ?? -1, type: 'error', payload: { message } });
    }
});
