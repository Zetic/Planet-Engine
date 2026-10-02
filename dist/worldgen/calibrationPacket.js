import { WORLDGEN_CRUST_CONTINENTAL, WORLDGEN_CRUST_TRANSITIONAL, WORLDGEN_INVALID_SAMPLE_ID, WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN, WORLDGEN_STRUCTURE_NONE, WORLDGEN_STRUCTURE_RIFT, WORLDGEN_STRUCTURE_SUTURE, WORLDGEN_STRUCTURE_TRANSFORM, } from './protocol.js';
export const WORLD_CALIBRATION_SCHEMA = 'planet-engine-calibration@1';
const RANKED_LIMIT = 8;
const SIGNIFICANT_CONTINENT_AREA_FRACTION = 0.0025;
function position(result, sample) {
    const offset = sample * 3;
    return [result.positions[offset], result.positions[offset + 1], result.positions[offset + 2]];
}
function arc(a, b) {
    return Math.acos(Math.max(-1, Math.min(1, a[0] * b[0] + a[1] * b[1] + a[2] * b[2])));
}
function sampleArc(result, a, b) {
    return arc(position(result, a), position(result, b));
}
function continentalComponents(result) {
    const count = result.metrics.fineSampleCount;
    const visited = new Uint8Array(count);
    const meanAreaSr = 4 * Math.PI / Math.max(1, count);
    const components = [];
    for (let start = 0; start < count; start += 1) {
        if (visited[start] || result.crustKind[start] !== WORLDGEN_CRUST_CONTINENTAL)
            continue;
        visited[start] = 1;
        const queue = [start];
        let queueIndex = 0;
        const samples = [];
        while (queueIndex < queue.length) {
            const sample = queue[queueIndex++];
            samples.push(sample);
            for (let cursor = result.neighborOffsets[sample]; cursor < result.neighborOffsets[sample + 1]; cursor += 1) {
                const neighbor = result.neighbors[cursor];
                if (!visited[neighbor] && result.crustKind[neighbor] === WORLDGEN_CRUST_CONTINENTAL) {
                    visited[neighbor] = 1;
                    queue.push(neighbor);
                }
            }
        }
        let perimeterRad = 0;
        const plateIds = new Set();
        for (const sample of samples) {
            plateIds.add(result.plateIds[sample]);
            for (let cursor = result.neighborOffsets[sample]; cursor < result.neighborOffsets[sample + 1]; cursor += 1) {
                const neighbor = result.neighbors[cursor];
                if (result.crustKind[neighbor] !== WORLDGEN_CRUST_CONTINENTAL)
                    perimeterRad += sampleArc(result, sample, neighbor);
            }
        }
        const first = samples[0];
        let farthestFromFirst = first;
        let farthestArc = -1;
        for (const sample of samples) {
            const distance = sampleArc(result, first, sample);
            if (distance > farthestArc) {
                farthestArc = distance;
                farthestFromFirst = sample;
            }
        }
        let farthest = farthestFromFirst;
        farthestArc = -1;
        for (const sample of samples) {
            const distance = sampleArc(result, farthestFromFirst, sample);
            if (distance > farthestArc) {
                farthestArc = distance;
                farthest = sample;
            }
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
function continentSummary(result) {
    const totalAreaSr = 4 * Math.PI;
    const components = continentalComponents(result)
        .filter(component => component.areaSr >= totalAreaSr * SIGNIFICANT_CONTINENT_AREA_FRACTION)
        .sort((a, b) => b.areaSr - a.areaSr);
    const areas = components.map(component => component.areaSr);
    const mean = areas.length ? areas.reduce((sum, value) => sum + value, 0) / areas.length : 0;
    const variance = areas.length ? areas.reduce((sum, value) => sum + (value - mean) ** 2, 0) / areas.length : 0;
    const cv = mean > 0 ? Math.sqrt(variance) / mean : 0;
    const median = areas.length ? areas[Math.floor(areas.length / 2)] : 0;
    const hierarchy = areas.length ? areas[0] / Math.max(1e-18, median) : 0;
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
        if (component.areaSr >= totalAreaSr * 0.015 && component.plateCount >= 2)
            hasMajorMultiplateComponent = true;
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
function topographySummary(result) {
    return {
        minimum_solid_elevation_m: result.metrics.minimumSolidElevationM,
        mean_solid_elevation_m: result.metrics.meanSolidElevationM,
        p05_solid_elevation_m: result.metrics.p05SolidElevationM,
        median_solid_elevation_m: result.metrics.medianSolidElevationM,
        p95_solid_elevation_m: result.metrics.p95SolidElevationM,
        maximum_solid_elevation_m: result.metrics.maximumSolidElevationM,
        sea_level_m: result.metrics.hasSeaLevel ? result.metrics.seaLevelM : null,
        land_area_fraction: result.metrics.landAreaFraction,
        ocean_area_fraction: result.metrics.oceanAreaFraction,
        mean_land_elevation_m: result.metrics.meanLandElevationM,
        mean_water_depth_m: result.metrics.meanWaterDepthM,
        maximum_water_depth_m: result.metrics.maximumWaterDepthM,
        target_water_volume_m3: result.metrics.targetWaterVolumeM3,
        solved_water_volume_m3: result.metrics.solvedWaterVolumeM3,
        water_volume_relative_error: result.metrics.waterVolumeRelativeError,
        clamped_sample_count: result.metrics.clampedSampleCount,
    };
}
function freeboardAccumulator() {
    return {
        sampleCount: 0,
        submergedCount: 0,
        submergedDepthSum: 0,
        shallow200: 0,
        shallow500: 0,
        shallow1000: 0,
        thicknessSum: 0,
        stabilitySum: 0,
        passiveMarginSum: 0,
        historicalRiftSum: 0,
        isostaticSum: 0,
        thermalSum: 0,
        orogenicSum: 0,
        ridgeSum: 0,
        riftBasinSum: 0,
        trenchSum: 0,
        arcSum: 0,
        mantleSum: 0,
        solidSum: 0,
        relativeElevationSum: 0,
    };
}
function addFreeboardSample(bucket, result, sample) {
    bucket.sampleCount += 1;
    const submerged = result.submergedMask[sample] !== 0;
    if (submerged) {
        const depth = Math.max(0, result.waterDepthM[sample]);
        bucket.submergedCount += 1;
        bucket.submergedDepthSum += depth;
        if (depth <= 200)
            bucket.shallow200 += 1;
        if (depth <= 500)
            bucket.shallow500 += 1;
        if (depth <= 1000)
            bucket.shallow1000 += 1;
    }
    bucket.thicknessSum += result.crustThicknessKm[sample];
    bucket.stabilitySum += result.continentalStabilityIndex[sample];
    bucket.passiveMarginSum += result.passiveMarginIndex[sample];
    bucket.historicalRiftSum += result.historicalRiftIntensity[sample];
    bucket.isostaticSum += result.isostaticElevationM[sample];
    bucket.thermalSum += result.thermalElevationM[sample];
    bucket.orogenicSum += result.orogenicElevationM[sample];
    bucket.ridgeSum += result.ridgeElevationM[sample];
    bucket.riftBasinSum += result.riftBasinElevationM[sample];
    bucket.trenchSum += result.trenchElevationM[sample];
    bucket.arcSum += result.arcElevationM[sample];
    bucket.mantleSum += result.mantleDynamicElevationM[sample];
    bucket.solidSum += result.solidElevationM[sample];
    bucket.relativeElevationSum += result.elevationAboveSeaLevelM[sample];
}
const FREEBOARD_BUCKET_ALL_CONTINENTAL = 0;
const FREEBOARD_BUCKET_EMERGENT_CONTINENTAL = 1;
const FREEBOARD_BUCKET_SUBMERGED_CONTINENTAL = 2;
const FREEBOARD_BUCKET_RESTORED_MARGIN_CONTINENTAL = 3;
const FREEBOARD_BUCKET_OTHER_CONTINENTAL = 4;
const FREEBOARD_BUCKET_COUNT = 5;
function validateFreeboardCausalSummary(result) {
    const causal = result.freeboardCausal;
    const fields = [
        causal.sampleCounts,
        causal.meanCrustDensityKgPerM3,
        causal.meanRiftHistory,
        causal.meanSubsidenceHistory,
        causal.meanBasinPotential,
        causal.meanCrustalStrain,
        causal.meanCompensatedBuoyancyIndex,
        causal.meanEffectiveElasticThicknessKm,
        causal.meanStructuralFabricStrength,
    ];
    if (fields.some(values => values.length !== FREEBOARD_BUCKET_COUNT) || causal.continentalStateCounts.length !== 3) {
        throw new Error('Freeboard causal observability packet has an invalid bucket shape.');
    }
    for (let bucket = 0; bucket < FREEBOARD_BUCKET_COUNT; bucket += 1) {
        if (causal.sampleCounts[bucket] === 0)
            continue;
        for (const values of fields.slice(1)) {
            if (!Number.isFinite(values[bucket])) {
                throw new Error(`Freeboard causal observability contains a non-finite mean in bucket ${bucket}.`);
            }
        }
    }
}
function causalMean(values, sampleCount, bucket) {
    return sampleCount > 0 ? values[bucket] : null;
}
function finalizeFreeboardBucket(bucket, result, causalBucket) {
    const count = Math.max(1, bucket.sampleCount);
    const submerged = Math.max(1, bucket.submergedCount);
    const causal = result.freeboardCausal;
    const causalCount = causal.sampleCounts[causalBucket];
    return {
        sample_count: bucket.sampleCount,
        causal_snapshot_sample_count: causalCount,
        submerged_fraction: bucket.submergedCount / count,
        mean_submerged_depth_m: bucket.submergedDepthSum / submerged,
        submerged_within_200m_fraction: bucket.shallow200 / submerged,
        submerged_within_500m_fraction: bucket.shallow500 / submerged,
        submerged_within_1000m_fraction: bucket.shallow1000 / submerged,
        mean_crust_thickness_km: bucket.thicknessSum / count,
        mean_crust_density_kg_per_m3: causalMean(causal.meanCrustDensityKgPerM3, causalCount, causalBucket),
        mean_continental_stability_index: bucket.stabilitySum / count,
        mean_rift_history: causalMean(causal.meanRiftHistory, causalCount, causalBucket),
        mean_subsidence_history: causalMean(causal.meanSubsidenceHistory, causalCount, causalBucket),
        mean_basin_potential: causalMean(causal.meanBasinPotential, causalCount, causalBucket),
        mean_passive_margin_index: bucket.passiveMarginSum / count,
        mean_historical_rift_intensity: bucket.historicalRiftSum / count,
        mean_crustal_strain: causalMean(causal.meanCrustalStrain, causalCount, causalBucket),
        mean_compensated_buoyancy_index: causalMean(causal.meanCompensatedBuoyancyIndex, causalCount, causalBucket),
        mean_effective_elastic_thickness_km: causalMean(causal.meanEffectiveElasticThicknessKm, causalCount, causalBucket),
        mean_structural_fabric_strength: causalMean(causal.meanStructuralFabricStrength, causalCount, causalBucket),
        elevation_budget_m: {
            isostatic: bucket.isostaticSum / count,
            thermal: bucket.thermalSum / count,
            orogenic: bucket.orogenicSum / count,
            ridge: bucket.ridgeSum / count,
            rift_basin: bucket.riftBasinSum / count,
            trench: bucket.trenchSum / count,
            arc: bucket.arcSum / count,
            mantle_dynamic: bucket.mantleSum / count,
            solid: bucket.solidSum / count,
            relative_to_sea_level: bucket.relativeElevationSum / count,
        },
    };
}
function crustFreeboardSummary(result) {
    validateFreeboardCausalSummary(result);
    const count = result.metrics.fineSampleCount;
    const crustCounts = { continental: 0, transitional: 0, oceanic: 0 };
    const crustSubmerged = { continental: 0, transitional: 0, oceanic: 0 };
    const crustDepth = { continental: 0, transitional: 0, oceanic: 0 };
    const structures = { none: 0, suture: 0, rift: 0, transform: 0, continental_margin: 0, other: 0 };
    const continental = freeboardAccumulator();
    const emergentContinental = freeboardAccumulator();
    const submergedContinental = freeboardAccumulator();
    const restoredMarginContinental = freeboardAccumulator();
    const otherContinental = freeboardAccumulator();
    let thinnedBelow32 = 0;
    let thinned32To36 = 0;
    let normal36To42 = 0;
    let thick42Plus = 0;
    for (let sample = 0; sample < count; sample += 1) {
        const crust = result.crustKind[sample];
        const submerged = result.submergedMask[sample] !== 0;
        const depth = submerged ? Math.max(0, result.waterDepthM[sample]) : 0;
        let crustKey;
        if (crust === WORLDGEN_CRUST_CONTINENTAL)
            crustKey = 'continental';
        else if (crust === WORLDGEN_CRUST_TRANSITIONAL)
            crustKey = 'transitional';
        else
            crustKey = 'oceanic';
        crustCounts[crustKey] += 1;
        if (submerged) {
            crustSubmerged[crustKey] += 1;
            crustDepth[crustKey] += depth;
        }
        const structure = result.structuralZoneKind[sample];
        if (structure === WORLDGEN_STRUCTURE_NONE)
            structures.none += 1;
        else if (structure === WORLDGEN_STRUCTURE_SUTURE)
            structures.suture += 1;
        else if (structure === WORLDGEN_STRUCTURE_RIFT)
            structures.rift += 1;
        else if (structure === WORLDGEN_STRUCTURE_TRANSFORM)
            structures.transform += 1;
        else if (structure === WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN)
            structures.continental_margin += 1;
        else
            structures.other += 1;
        if (crust !== WORLDGEN_CRUST_CONTINENTAL)
            continue;
        addFreeboardSample(continental, result, sample);
        addFreeboardSample(submerged ? submergedContinental : emergentContinental, result, sample);
        const restoredMargin = result.continentalMarginMaterial[sample] !== 0;
        addFreeboardSample(restoredMargin ? restoredMarginContinental : otherContinental, result, sample);
        const thickness = result.crustThicknessKm[sample];
        if (thickness < 32)
            thinnedBelow32 += 1;
        else if (thickness < 36)
            thinned32To36 += 1;
        else if (thickness < 42)
            normal36To42 += 1;
        else
            thick42Plus += 1;
    }
    const continentalCount = Math.max(1, crustCounts.continental);
    const crustSection = (key) => ({
        surface_fraction: crustCounts[key] / Math.max(1, count),
        submerged_fraction: crustSubmerged[key] / Math.max(1, crustCounts[key]),
        mean_submerged_depth_m: crustDepth[key] / Math.max(1, crustSubmerged[key]),
    });
    const structureFractions = Object.fromEntries(Object.entries(structures).map(([key, value]) => [key, value / Math.max(1, count)]));
    const causalSnapshotConsistent = result.freeboardCausal.sampleCounts[FREEBOARD_BUCKET_ALL_CONTINENTAL] === continental.sampleCount
        && result.freeboardCausal.sampleCounts[FREEBOARD_BUCKET_EMERGENT_CONTINENTAL] === emergentContinental.sampleCount
        && result.freeboardCausal.sampleCounts[FREEBOARD_BUCKET_SUBMERGED_CONTINENTAL] === submergedContinental.sampleCount
        && result.freeboardCausal.sampleCounts[FREEBOARD_BUCKET_RESTORED_MARGIN_CONTINENTAL] === restoredMarginContinental.sampleCount
        && result.freeboardCausal.sampleCounts[FREEBOARD_BUCKET_OTHER_CONTINENTAL] === otherContinental.sampleCount;
    if (!causalSnapshotConsistent) {
        throw new Error('Freeboard causal snapshot counts do not match the transported WG-4 classification fields.');
    }
    return {
        causal_snapshot_source: 'wg4-pre-scratch-release',
        water_inventory: {
            surface_water_mass_kg: result.planet.surfaceWaterMassKg,
            equivalent_global_water_depth_m: result.planet.equivalentGlobalWaterDepthM,
            solved_sea_level_m: result.metrics.hasSeaLevel ? result.metrics.seaLevelM : null,
            target_water_volume_m3: result.metrics.targetWaterVolumeM3,
            solved_water_volume_m3: result.metrics.solvedWaterVolumeM3,
            water_volume_relative_error: result.metrics.waterVolumeRelativeError,
        },
        crust: {
            continental: crustSection('continental'),
            transitional: crustSection('transitional'),
            oceanic: crustSection('oceanic'),
        },
        structural_zone_surface_fraction: structureFractions,
        continental_state: {
            quiet_fraction_of_continental: result.freeboardCausal.continentalStateCounts[0] / continentalCount,
            margin_fraction_of_continental: result.freeboardCausal.continentalStateCounts[1] / continentalCount,
            rift_fraction_of_continental: result.freeboardCausal.continentalStateCounts[2] / continentalCount,
            restored_margin_material_fraction_of_continental: restoredMarginContinental.sampleCount / continentalCount,
            thickness_fraction_of_continental: {
                below_32_km: thinnedBelow32 / continentalCount,
                from_32_to_36_km: thinned32To36 / continentalCount,
                from_36_to_42_km: normal36To42 / continentalCount,
                at_or_above_42_km: thick42Plus / continentalCount,
            },
        },
        causal_snapshot_consistent: causalSnapshotConsistent,
        all_continental: finalizeFreeboardBucket(continental, result, FREEBOARD_BUCKET_ALL_CONTINENTAL),
        emergent_continental: finalizeFreeboardBucket(emergentContinental, result, FREEBOARD_BUCKET_EMERGENT_CONTINENTAL),
        submerged_continental: finalizeFreeboardBucket(submergedContinental, result, FREEBOARD_BUCKET_SUBMERGED_CONTINENTAL),
        restored_margin_continental: finalizeFreeboardBucket(restoredMarginContinental, result, FREEBOARD_BUCKET_RESTORED_MARGIN_CONTINENTAL),
        other_continental: finalizeFreeboardBucket(otherContinental, result, FREEBOARD_BUCKET_OTHER_CONTINENTAL),
    };
}
function rankedBasins(result) {
    return Array.from(result.basinAreasM2, (area, index) => ({
        basin_id: index,
        outlet_sample: result.basinOutletSamples[index],
        outlet_kind: result.basinOutletKinds[index],
        area_km2: area / 1e6,
    }))
        .sort((a, b) => b.area_km2 - a.area_km2)
        .slice(0, RANKED_LIMIT);
}
function rankedDepressions(result) {
    return Array.from(result.depressionAreasM2, (area, index) => ({
        depression_id: index,
        floor_sample: result.depressionFloorSamples[index],
        area_km2: area / 1e6,
        maximum_depth_m: Math.max(0, result.depressionSpillElevationsM[index] - result.depressionFloorElevationsM[index]),
        floor_elevation_m: result.depressionFloorElevationsM[index],
        spill_elevation_m: result.depressionSpillElevationsM[index],
    }))
        .sort((a, b) => b.area_km2 - a.area_km2)
        .slice(0, RANKED_LIMIT);
}
function rankedLakes(result) {
    const maximumDepthById = new Map();
    for (let sample = 0; sample < result.lakeId.length; sample += 1) {
        const lakeId = result.lakeId[sample];
        if (lakeId === WORLDGEN_INVALID_SAMPLE_ID)
            continue;
        maximumDepthById.set(lakeId, Math.max(maximumDepthById.get(lakeId) ?? 0, result.lakeDepthM[sample]));
    }
    return Array.from(result.lakeAreasM2, (area, index) => ({
        lake_id: index,
        depression_id: result.lakeDepressionIds[index],
        kind: result.lakeKinds[index],
        area_km2: area / 1e6,
        volume_km3: result.lakeVolumesM3[index] / 1e9,
        maximum_depth_m: maximumDepthById.get(index) ?? 0,
        surface_elevation_m: result.lakeSurfaceElevationsM[index],
        gross_land_inflow_m3_s: null,
        lake_evaporation_m3_s: null,
        outflow_m3_s: result.lakeOutflowsM3S[index],
    }))
        .sort((a, b) => b.area_km2 - a.area_km2)
        .slice(0, RANKED_LIMIT);
}
export function buildWorldCalibrationPacket(result, seed, plateCount) {
    const largestBasins = rankedBasins(result);
    const largestDepressions = rankedDepressions(result);
    const largestLakes = rankedLakes(result);
    const landAreaKm2 = Math.max(1e-12, result.drainageMetrics.landAreaM2 / 1e6);
    return {
        schema: WORLD_CALIBRATION_SCHEMA,
        fidelity: {
            source: 'github-pages',
            canonical_dual_cell_area: false,
            complete_internal_lake_budget: false,
            approximation_notes: [
                'continental component and crust/freeboard area fractions use equal-sample weighting because the Pages cumulative result does not transport fine dual-cell area',
                'topography summary scalars come from the canonical Rust WG-4 metrics rather than browser recomputation',
                'freeboard causal means and continental-state counts are snapshotted in Rust at WG-4 before memory-only scratch fields are released',
                'per-lake gross inflow and evaporation are unavailable in the cumulative browser result and remain null',
            ],
        },
        run: {
            seed,
            engine_version: result.engineVersion,
            coarse_level: result.coarseLevel,
            fine_level: result.fineLevel,
            sample_count: result.metrics.fineSampleCount,
            plate_count: plateCount,
        },
        planet: {
            radius_m: result.planet.radiusM,
            surface_gravity_m_s2: result.planet.surfaceGravityMS2,
            rotation_period_s: result.planet.rotationPeriodS,
            axial_tilt_rad: result.planet.axialTiltRad,
            orbital_period_s: result.planet.orbitalPeriodS,
            stellar_flux_w_m2: result.planet.stellarFluxWM2,
            reference_surface_pressure_pa: result.planet.referenceSurfacePressurePa,
            surface_water_mass_kg: result.planet.surfaceWaterMassKg,
            equivalent_global_water_depth_m: result.planet.equivalentGlobalWaterDepthM,
            internal_heat_flux_w_per_m2: result.planet.internalHeatFluxWPerM2,
        },
        hashes: {
            coarse_topology: result.metrics.coarseTopologyHash,
            fine_topology: result.metrics.fineTopologyHash,
            tectonic: result.metrics.tectonicHash,
            geology: result.metrics.geologyHash,
            lithosphere: result.metrics.lithosphereHash,
            historical_identity: result.historicalIdentityHash,
            historical_morphology: result.historicalMorphologyHash,
            lithology: result.lithologyHash,
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
        crust_freeboard: crustFreeboardSummary(result),
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
            spinup_converged: result.metrics.spinupConverged,
            convergence_temperature_rms_k: result.metrics.convergenceTemperatureRmsK,
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
export function worldCalibrationJson(result, seed, plateCount) {
    return `${JSON.stringify(buildWorldCalibrationPacket(result, seed, plateCount), null, 2)}\n`;
}
export function worldCalibrationMarkdown(result, seed, plateCount) {
    const packet = buildWorldCalibrationPacket(result, seed, plateCount);
    const c = packet.continents;
    const t = packet.topography;
    const cf = packet.crust_freeboard;
    const climate = packet.climate;
    const h = packet.hydrology;
    const g = packet.geomorphology;
    const fixed = (value, digits) => value === null ? 'n/a' : value.toFixed(digits);
    const lines = [
        '# Planet Engine calibration report',
        '',
        `Schema: \`${packet.schema}\``,
        'Fidelity: GitHub Pages packet · equal-sample morphology/freeboard areas · canonical WG-4 topography summaries · partial per-lake budget',
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
        '## Crust / freeboard',
        `Water inventory: ${(cf.water_inventory.surface_water_mass_kg / 1e21).toFixed(3)}e21 kg · global equivalent depth ${cf.water_inventory.equivalent_global_water_depth_m.toFixed(0)} m · solved sea level ${cf.water_inventory.solved_sea_level_m?.toFixed(0) ?? 'none'} m`,
        `Crust C/T/O: ${(cf.crust.continental.surface_fraction * 100).toFixed(1)}%/${(cf.crust.transitional.surface_fraction * 100).toFixed(1)}%/${(cf.crust.oceanic.surface_fraction * 100).toFixed(1)}% · submerged ${(cf.crust.continental.submerged_fraction * 100).toFixed(1)}%/${(cf.crust.transitional.submerged_fraction * 100).toFixed(1)}%/${(cf.crust.oceanic.submerged_fraction * 100).toFixed(1)}%`,
        `Continental state: quiet ${(cf.continental_state.quiet_fraction_of_continental * 100).toFixed(1)}% · margin ${(cf.continental_state.margin_fraction_of_continental * 100).toFixed(1)}% · rift ${(cf.continental_state.rift_fraction_of_continental * 100).toFixed(1)}% · restored margin material ${(cf.continental_state.restored_margin_material_fraction_of_continental * 100).toFixed(1)}% · <36 km crust ${((cf.continental_state.thickness_fraction_of_continental.below_32_km + cf.continental_state.thickness_fraction_of_continental.from_32_to_36_km) * 100).toFixed(1)}%`,
        `Emergent/submerged continental: thickness ${cf.emergent_continental.mean_crust_thickness_km.toFixed(1)}/${cf.submerged_continental.mean_crust_thickness_km.toFixed(1)} km · density ${fixed(cf.emergent_continental.mean_crust_density_kg_per_m3, 0)}/${fixed(cf.submerged_continental.mean_crust_density_kg_per_m3, 0)} kg/m³ · shallow submerged ≤500 m ${(cf.submerged_continental.submerged_within_500m_fraction * 100).toFixed(1)}% · restored-margin submerged ${(cf.restored_margin_continental.submerged_fraction * 100).toFixed(1)}%`,
        `Submerged drivers: rift/subsidence/basin ${fixed(cf.submerged_continental.mean_rift_history, 3)}/${fixed(cf.submerged_continental.mean_subsidence_history, 3)}/${fixed(cf.submerged_continental.mean_basin_potential, 3)} · passive margin ${cf.submerged_continental.mean_passive_margin_index.toFixed(3)} · isostatic/rift-basin/orogen ${cf.submerged_continental.elevation_budget_m.isostatic.toFixed(0)}/${cf.submerged_continental.elevation_budget_m.rift_basin.toFixed(0)}/${cf.submerged_continental.elevation_budget_m.orogenic.toFixed(0)} m`,
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
        `\`tectonic ${packet.hashes.tectonic}\` → \`geology ${packet.hashes.geology}\` → \`lithosphere ${packet.hashes.lithosphere}\` → \`topography ${packet.hashes.topography}\` → \`lithology ${packet.hashes.lithology}\` → \`climate ${packet.hashes.climate}\``,
        `\`drainage ${packet.hashes.final_drainage}\` → \`runoff ${packet.hashes.final_runoff}\` → \`lakes ${packet.hashes.final_lake}\` → \`seasonal ${packet.hashes.final_seasonal}\` → \`erosion ${packet.hashes.erosion}\` → \`evolution ${packet.hashes.evolution}\` → \`infill ${packet.hashes.infill}\``,
        '',
    ];
    return lines.join('\n');
}
