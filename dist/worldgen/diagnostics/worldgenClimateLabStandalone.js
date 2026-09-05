import { createWorldgenClient } from '../worldgenClient.js';
import { mapVectorDelta, reconstructAnnualHarmonicFromBasis } from './worldgenClimateMath.js';
import { WORLDGEN_BOUNDARY_CONVERGENT, WORLDGEN_BOUNDARY_DIVERGENT, WORLDGEN_BOUNDARY_TRANSFORM, WORLDGEN_CRUST_CONTINENTAL, WORLDGEN_CRUST_OCEANIC, WORLDGEN_CRUST_TRANSITIONAL, WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION, WORLDGEN_GEOLOGY_CONTINENTAL_RIFT, WORLDGEN_GEOLOGY_OCEANIC_RIDGE, WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION, WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION, WORLDGEN_GEOLOGY_TRANSFORM, WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE, WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN, WORLDGEN_STRUCTURE_NONE, WORLDGEN_STRUCTURE_RIFT, WORLDGEN_STRUCTURE_SUTURE, WORLDGEN_STRUCTURE_TRANSFORM, WORLDGEN_INVALID_SAMPLE_ID, } from '../protocol.js';
const PALETTE_STEPS = 256;
const TWO_PI = Math.PI * 2;
function element(id) {
    const target = document.getElementById(id);
    if (!target)
        throw new Error(`Planet Engine Lab is missing #${id}.`);
    return target;
}
function metric(container, label, value) {
    const item = document.createElement('div');
    const key = document.createElement('strong');
    const detail = document.createElement('span');
    key.textContent = label;
    detail.textContent = value;
    item.append(key, detail);
    container.appendChild(item);
}
function plateColor(id) { return `hsl(${(id * 137.507764 + 18) % 360} 60% 55%)`; }
function provenanceColor(source) { return `hsl(${(source * 137.507764 + 42) % 360} 58% 54%)`; }
function crustColor(kind) {
    if (kind === WORLDGEN_CRUST_CONTINENTAL)
        return '#b79a72';
    if (kind === WORLDGEN_CRUST_TRANSITIONAL)
        return '#9aab87';
    if (kind === WORLDGEN_CRUST_OCEANIC)
        return '#477aa3';
    return '#d7e2ef';
}
function structuralColor(kind) {
    if (kind === WORLDGEN_STRUCTURE_SUTURE)
        return '#ff7466';
    if (kind === WORLDGEN_STRUCTURE_RIFT)
        return '#ffb45d';
    if (kind === WORLDGEN_STRUCTURE_TRANSFORM)
        return '#c690ff';
    if (kind === WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN)
        return '#65d7ac';
    if (kind === WORLDGEN_STRUCTURE_NONE)
        return '#425362';
    return '#d7e2ef';
}
function tectonicBoundaryColor(kind) {
    if (kind === WORLDGEN_BOUNDARY_CONVERGENT)
        return '#ff7272';
    if (kind === WORLDGEN_BOUNDARY_DIVERGENT)
        return '#64d7ff';
    if (kind === WORLDGEN_BOUNDARY_TRANSFORM)
        return '#ffd36a';
    return '#d7e2ef';
}
function geologicalBoundaryColor(regime) {
    if (regime === WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION)
        return '#5a8fff';
    if (regime === WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION)
        return '#8a70ff';
    if (regime === WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION)
        return '#ff6969';
    if (regime === WORLDGEN_GEOLOGY_OCEANIC_RIDGE)
        return '#4ee8df';
    if (regime === WORLDGEN_GEOLOGY_CONTINENTAL_RIFT)
        return '#ffb65c';
    if (regime === WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE)
        return '#e8cf66';
    if (regime === WORLDGEN_GEOLOGY_TRANSFORM)
        return '#d59cff';
    return '#d7e2ef';
}
const RECONCILIATION_MODES = new Set(['reconciliation-lake-depth-delta', 'reconciliation-lake-change', 'reconciliation-realized-discharge-delta', 'reconciliation-flow-presence-delta', 'reconciliation-flow-regime-change']);
const EVOLUTION_MODES = new Set(['evolution-solid-elevation', 'evolution-terrain-delta', 'evolution-applied-erosion', 'evolution-applied-deposition', 'evolution-receiver-change', 'evolution-contributing-area', 'evolution-potential-discharge']);
const EROSION_MODES = new Set(['erosion-effective-discharge', 'erosion-channel-slope', 'erosion-channel-width', 'erosion-erodibility', 'erosion-incision-potential', 'erosion-sediment-supply', 'erosion-sediment-load', 'erosion-sediment-deposition']);
const LAKE_MODES = new Set(['realized-discharge', 'lake-depth', 'lake-state', 'lake-fraction']);
function isLakeMode(mode) { return LAKE_MODES.has(mode); }
function lakeSampleColor(result, mode, sample) {
    if (result.submergedMask[sample])
        return '#102c43';
    if (mode === 'lake-state') {
        const kind = result.lakeKind[sample];
        if (kind === 1)
            return '#3aa7c9';
        if (kind === 2)
            return '#63d0a5';
        if (kind === 3)
            return '#9b78d0';
        return '#31423c';
    }
    if (mode === 'lake-fraction')
        return drainageScalarColor(result.lakeFraction[sample], 210, 175);
    if (mode === 'lake-depth') {
        const depth = Math.max(0, result.lakeDepthM[sample]);
        if (depth <= 0)
            return '#31423c';
        const maxDepth = Math.max(1, result.lakeMetrics.maximumLakeDepthM);
        return drainageScalarColor(Math.log1p(depth) / Math.log1p(maxDepth), 220, 175);
    }
    const maxValue = Math.max(1e-6, result.lakeMetrics.maximumRealizedDischargeM3S);
    return drainageScalarColor(Math.log1p(Math.max(0, result.realizedDischargeM3S[sample])) / Math.log1p(maxValue), 205, 35);
}
const RUNOFF_MODES = new Set(['annual-runoff', 'runoff-fraction', 'actual-et', 'potential-discharge']);
function isRunoffMode(mode) { return RUNOFF_MODES.has(mode); }
function runoffSampleColor(result, mode, sample) {
    if (result.submergedMask[sample])
        return '#102c43';
    if (mode === 'runoff-fraction')
        return drainageScalarColor(result.runoffFraction[sample], 48, 205);
    if (mode === 'actual-et') {
        const value = Math.max(0, result.actualEvapotranspirationMm[sample]);
        return drainageScalarColor(value / (value + 850), 42, 168);
    }
    if (mode === 'annual-runoff') {
        const maxValue = Math.max(1, result.runoffMetrics.maximumLandRunoffMm);
        return drainageScalarColor(Math.log1p(Math.max(0, result.localRunoffMm[sample])) / Math.log1p(maxValue), 44, 218);
    }
    const maxValue = Math.max(1e-6, result.runoffMetrics.maximumPotentialDischargeM3S);
    return drainageScalarColor(Math.log1p(Math.max(0, result.potentialDischargeM3S[sample])) / Math.log1p(maxValue), 215, 18);
}
const DRAINAGE_MODES = new Set([
    'contributing-area',
    'basins',
    'flow-direction',
    'depression-depth',
    'depressions',
    'escape-elevation',
]);
function isDrainageMode(mode) { return DRAINAGE_MODES.has(mode); }
function discreteDrainageColor(id, saturation = 62, lightness = 53) {
    return `hsl(${(id * 137.507764 + 32) % 360} ${saturation}% ${lightness}%)`;
}
function drainageScalarColor(t, lowHue, highHue) {
    const clamped = Math.max(0, Math.min(1, t));
    const hue = lowHue + (highHue - lowHue) * clamped;
    return `hsl(${hue} 70% ${34 + 25 * clamped}%)`;
}
function drainageSampleColor(result, mode, sample) {
    if (result.submergedMask[sample])
        return '#102c43';
    if (mode === 'basins' || mode === 'flow-direction') {
        const basin = result.basinId[sample];
        return basin === WORLDGEN_INVALID_SAMPLE_ID ? '#4c5964' : discreteDrainageColor(basin);
    }
    if (mode === 'depressions') {
        const depression = result.depressionId[sample];
        return depression === WORLDGEN_INVALID_SAMPLE_ID ? '#31423c' : discreteDrainageColor(depression, 72, 58);
    }
    if (mode === 'depression-depth') {
        const depth = result.depressionDepthM[sample];
        if (depth <= 0)
            return '#283c34';
        const t = Math.log10(1 + depth) / Math.log10(1 + Math.max(50, result.drainageMetrics.maximumDepressionDepthM));
        return drainageScalarColor(t, 55, 270);
    }
    if (mode === 'escape-elevation') {
        return drainageScalarColor((result.hydrologicEscapeElevationM[sample] + 500) / 5_500, 220, 20);
    }
    const areaKm2 = Math.max(1e-9, result.contributingAreaM2[sample] / 1e6);
    const logArea = Math.log10(areaKm2 + 1);
    const maxLog = Math.log10(Math.max(10, result.drainageMetrics.maximumContributingAreaM2 / 1e6) + 1);
    return drainageScalarColor(logArea / maxLog, 225, 42);
}
function drawDrainageReceiverOverlay(context, result, projection, width, buffers) {
    const targetSegments = 3_500;
    const stride = Math.max(1, Math.floor(result.drainageMetrics.landSampleCount / targetSegments));
    context.save();
    context.strokeStyle = 'rgba(235,247,255,0.68)';
    context.lineWidth = 0.75;
    context.beginPath();
    let accepted = 0;
    for (let sample = 0; sample < result.drainageMetrics.sampleCount; sample += 1) {
        if (result.submergedMask[sample] || !buffers.visible[sample])
            continue;
        if ((accepted++ % stride) !== 0)
            continue;
        const receiver = result.receiver[sample];
        if (receiver === WORLDGEN_INVALID_SAMPLE_ID || !buffers.visible[receiver])
            continue;
        const ax = buffers.x[sample], bx = buffers.x[receiver];
        if (projection === 'map' && Math.abs(ax - bx) > width * 0.45)
            continue;
        context.moveTo(ax, buffers.y[sample]);
        context.lineTo(bx, buffers.y[receiver]);
    }
    context.stroke();
    context.restore();
}
function drawDrainageOutlets(context, result, buffers) {
    context.save();
    context.fillStyle = 'rgba(255,255,255,0.92)';
    for (const outlet of result.basinOutletSamples) {
        if (outlet === WORLDGEN_INVALID_SAMPLE_ID || !buffers.visible[outlet])
            continue;
        context.beginPath();
        context.arc(buffers.x[outlet], buffers.y[outlet], 2.2, 0, TWO_PI);
        context.fill();
    }
    context.restore();
}
function renderDrainageDiagnostic(context, result, projection, mode, width, buffers, interactive) {
    const count = result.drainageMetrics.sampleCount;
    const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
    const fastPoints = interactive && count > 20_000;
    context.globalAlpha = 0.94;
    for (let sample = 0; sample < count; sample += 1) {
        if (!buffers.visible[sample])
            continue;
        context.fillStyle = drainageSampleColor(result, mode, sample);
        const x = buffers.x[sample], y = buffers.y[sample];
        if (fastPoints)
            context.fillRect(x - 0.75, y - 0.75, 1.5, 1.5);
        else {
            context.beginPath();
            context.arc(x, y, pointRadius, 0, TWO_PI);
            context.fill();
        }
    }
    context.globalAlpha = 1;
    if (mode === 'flow-direction')
        drawDrainageReceiverOverlay(context, result, projection, width, buffers);
    if (mode === 'basins')
        drawDrainageOutlets(context, result, buffers);
}
function scalarColor(value, field) {
    const t = Math.max(0, Math.min(1, (value - field.minimum) / Math.max(1e-12, field.maximum - field.minimum)));
    const quantized = Math.round(t * (PALETTE_STEPS - 1)) / (PALETTE_STEPS - 1);
    const hue = field.lowHue + (field.highHue - field.lowHue) * quantized;
    return `hsl(${hue} 68% ${37 + quantized * 23}%)`;
}
function hypsometricColor(result, sample) {
    if (result.submergedMask[sample]) {
        const depth = result.waterDepthM[sample];
        if (depth > 6_000)
            return '#071d3a';
        if (depth > 3_500)
            return '#0b3562';
        if (depth > 1_500)
            return '#15588a';
        if (depth > 500)
            return '#2b83b8';
        if (depth > 100)
            return '#69b7cf';
        return '#a4dce1';
    }
    const elevation = result.elevationAboveSeaLevelM[sample];
    if (elevation < 100)
        return '#507d45';
    if (elevation < 400)
        return '#6d9450';
    if (elevation < 1_000)
        return '#91a85d';
    if (elevation < 2_000)
        return '#aa9463';
    if (elevation < 3_500)
        return '#8f7157';
    if (elevation < 5_000)
        return '#9b9290';
    return '#e6ebed';
}
function evolvedHypsometricColor(result, sample) {
    if (result.submergedMask[sample])
        return hypsometricColor(result, sample);
    const elevation = result.elevationAboveSeaLevelM[sample] + result.terrainDeltaM[sample];
    if (elevation < 100)
        return '#456f3d';
    if (elevation < 400)
        return '#608448';
    if (elevation < 1_000)
        return '#829955';
    if (elevation < 2_000)
        return '#9b875b';
    if (elevation < 3_500)
        return '#80664f';
    if (elevation < 5_000)
        return '#918a88';
    return '#e4e9ec';
}
function bucketize(count, colorAt) {
    const buckets = new Map();
    for (let index = 0; index < count; index += 1) {
        const color = colorAt(index);
        const values = buckets.get(color);
        if (values)
            values.push(index);
        else
            buckets.set(color, [index]);
    }
    return Array.from(buckets, ([color, indices]) => ({ color, indices: Uint32Array.from(indices) }));
}
function seasonalValue(mean, cosine, sine, phase) {
    const angle = phase * TWO_PI;
    return reconstructAnnualHarmonicFromBasis(mean, cosine, sine, Math.cos(angle), Math.sin(angle));
}
function seasonalScalar(mean, cosine, sine, phase, scratch) {
    const angle = phase * TWO_PI;
    const c = Math.cos(angle);
    const s = Math.sin(angle);
    for (let index = 0; index < scratch.length; index += 1)
        scratch[index] = reconstructAnnualHarmonicFromBasis(mean[index], cosine[index], sine[index], c, s);
    return scratch;
}
function magnitudeField(east, north, scratch) {
    for (let index = 0; index < scratch.length; index += 1)
        scratch[index] = Math.hypot(east[index], north[index]);
    return scratch;
}
function seasonalPhaseRate(phases, phaseCount, sampleCount, phase, scratch) {
    if (phaseCount <= 0 || phases.length !== phaseCount * sampleCount) {
        scratch.fill(0);
        return scratch;
    }
    const scaled = ((phase % 1) + 1) % 1 * phaseCount;
    const lower = Math.floor(scaled) % phaseCount;
    const upper = (lower + 1) % phaseCount;
    const t = scaled - Math.floor(scaled);
    const lowerOffset = lower * sampleCount;
    const upperOffset = upper * sampleCount;
    for (let index = 0; index < sampleCount; index += 1) {
        scratch[index] = phases[lowerOffset + index] * (1 - t) + phases[upperOffset + index] * t;
    }
    return scratch;
}
let seasonalScratch = new Float32Array(0);
let scalarScratch = new Float32Array(0);
function ensureScratch(count) {
    if (seasonalScratch.length !== count)
        seasonalScratch = new Float32Array(count);
    if (scalarScratch.length !== count)
        scalarScratch = new Float32Array(count);
}
function scalarField(result, mode, phase) {
    ensureScratch(result.metrics.fineSampleCount);
    switch (mode) {
        case 'solid-elevation': return { values: result.solidElevationM, minimum: -12_000, maximum: 8_000, lowHue: 225, highHue: 25 };
        case 'relative-elevation': return { values: result.elevationAboveSeaLevelM, minimum: -10_000, maximum: 6_000, lowHue: 220, highHue: 35 };
        case 'water-depth': return { values: result.waterDepthM, minimum: 0, maximum: 10_000, lowHue: 195, highHue: 245 };
        case 'isostatic': return { values: result.isostaticElevationM, minimum: 0, maximum: 10_000, lowHue: 210, highHue: 25 };
        case 'thermal': return { values: result.thermalElevationM, minimum: -5_000, maximum: 0, lowHue: 260, highHue: 185 };
        case 'orogenic-relief': return { values: result.orogenicElevationM, minimum: 0, maximum: 6_000, lowHue: 55, highHue: 350 };
        case 'ridge-relief': return { values: result.ridgeElevationM, minimum: 0, maximum: 3_000, lowHue: 220, highHue: 165 };
        case 'rift-basin': return { values: result.riftBasinElevationM, minimum: -3_500, maximum: 0, lowHue: 250, highHue: 35 };
        case 'trench-relief': return { values: result.trenchElevationM, minimum: -7_000, maximum: 0, lowHue: 285, highHue: 210 };
        case 'arc-relief': return { values: result.arcElevationM, minimum: 0, maximum: 3_000, lowHue: 50, highHue: 5 };
        case 'mantle-relief': return { values: result.mantleDynamicElevationM, minimum: -1_200, maximum: 1_200, lowHue: 245, highHue: 25 };
        case 'crust-age': return { values: result.crustAgeMyr, minimum: 0, maximum: 3_500, lowHue: 205, highHue: 24 };
        case 'crust-thickness': return { values: result.crustThicknessKm, minimum: 5, maximum: 56, lowHue: 205, highHue: 350 };
        case 'orogeny-history': return { values: result.orogenicHistory, minimum: 0, maximum: 1, lowHue: 50, highHue: 350 };
        case 'ridge-history': return { values: result.ridgeHistory, minimum: 0, maximum: 1, lowHue: 225, highHue: 170 };
        case 'trench-history': return { values: result.trenchHistory, minimum: 0, maximum: 1, lowHue: 200, highHue: 260 };
        case 'strength': return { values: result.strengthIndex, minimum: 0, maximum: 1, lowHue: 0, highHue: 135 };
        case 'weakness': return { values: result.weaknessIndex, minimum: 0, maximum: 1, lowHue: 205, highHue: 15 };
        case 'dynamic-support': return { values: result.mantleDynamicSupportIndex, minimum: -1, maximum: 1, lowHue: 245, highHue: 25 };
        case 'fragmentation': return { values: result.fragmentationPropensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 0 };
        case 'annual-insolation': return { values: result.annualMeanInsolationWM2, minimum: 0, maximum: 500, lowHue: 240, highHue: 40 };
        case 'seasonal-insolation': return { values: result.seasonalInsolationAmplitudeWM2, minimum: 0, maximum: 700, lowHue: 220, highHue: 0 };
        case 'temperature': return { values: result.temperatureMeanK, minimum: 220, maximum: 315, lowHue: 235, highHue: 0 };
        case 'seasonal-temperature': return { values: seasonalScalar(result.temperatureMeanK, result.temperatureAnnualCosK, result.temperatureAnnualSinK, phase, seasonalScratch), minimum: 210, maximum: 325, lowHue: 240, highHue: 0 };
        case 'temperature-range': {
            for (let index = 0; index < scalarScratch.length; index += 1)
                scalarScratch[index] = result.temperatureMaxK[index] - result.temperatureMinK[index];
            return { values: scalarScratch, minimum: 0, maximum: 70, lowHue: 215, highHue: 10 };
        }
        case 'sst': return { values: result.seaSurfaceTemperatureMeanK, minimum: 265, maximum: 310, lowHue: 235, highHue: 0 };
        case 'seasonal-sst': return { values: seasonalScalar(result.seaSurfaceTemperatureMeanK, result.seaSurfaceTemperatureAnnualCosK, result.seaSurfaceTemperatureAnnualSinK, phase, seasonalScratch), minimum: 260, maximum: 315, lowHue: 235, highHue: 0 };
        case 'surface-pressure': return { values: result.localPressurePa, minimum: 45_000, maximum: 105_000, lowHue: 260, highHue: 35 };
        case 'wind-speed': {
            magnitudeField(result.windEastMeanMS, result.windNorthMeanMS, scalarScratch);
            return { values: scalarScratch, minimum: 0, maximum: 25, lowHue: 220, highHue: 25 };
        }
        case 'current-speed': return { values: result.currentSpeedMeanMS, minimum: 0, maximum: 1.5, lowHue: 225, highHue: 25 };
        case 'ocean-heat': return { values: result.oceanHeatTransportIndex, minimum: -2, maximum: 2, lowHue: 230, highHue: 5 };
        case 'humidity': return { values: result.specificHumidityMean, minimum: 0, maximum: 0.025, lowHue: 35, highHue: 205 };
        case 'precipitation': return { values: result.annualPrecipitationMm, minimum: 0, maximum: 2_500, lowHue: 45, highHue: 205 };
        case 'seasonal-precipitation': return { values: seasonalPhaseRate(result.precipitationPhaseRateMmYear, result.metrics.orbitalPhaseCount, result.metrics.fineSampleCount, phase, seasonalScratch), minimum: 0, maximum: 5_000, lowHue: 45, highHue: 205 };
        case 'seasonal-realized-discharge': {
            seasonalPhaseRate(result.seasonalPhaseRealizedDischargeM3S, result.seasonalMetrics.orbitalPhaseCount, result.seasonalMetrics.sampleCount, phase, seasonalScratch);
            for (let index = 0; index < scalarScratch.length; index += 1)
                scalarScratch[index] = Math.log1p(Math.max(0, seasonalScratch[index]));
            return { values: scalarScratch, minimum: 0, maximum: Math.log1p(Math.max(1e-6, result.seasonalMetrics.maximumPhaseRealizedDischargeM3S)), lowHue: 205, highHue: 25 };
        }
        case 'seasonal-flow-presence': return { values: result.seasonalFlowPresenceFraction, minimum: 0, maximum: 1, lowHue: 42, highHue: 205 };
        case 'seasonal-snow-storage': {
            seasonalPhaseRate(result.seasonalPhaseSnowStorageMm, result.seasonalMetrics.orbitalPhaseCount, result.seasonalMetrics.sampleCount, phase, seasonalScratch);
            for (let index = 0; index < scalarScratch.length; index += 1) {
                const value = Math.max(0, seasonalScratch[index]);
                scalarScratch[index] = value / (value + 500);
            }
            return { values: scalarScratch, minimum: 0, maximum: 1, lowHue: 225, highHue: 175 };
        }
        case 'reconciliation-lake-depth-delta': {
            const bound = Math.max(0.01, result.reconciliationMetrics.maximumAbsoluteLakeDepthChangeM);
            return { values: result.lakeDepthDeltaM, minimum: -bound, maximum: bound, lowHue: 25, highHue: 205 };
        }
        case 'reconciliation-lake-change': {
            for (let index = 0; index < scalarScratch.length; index += 1)
                scalarScratch[index] = result.lakeKindChangedMask[index];
            return { values: scalarScratch, minimum: 0, maximum: 1, lowHue: 210, highHue: 5 };
        }
        case 'reconciliation-realized-discharge-delta': {
            const bound = Math.max(1e-6, result.reconciliationMetrics.maximumAbsoluteAnnualRealizedDischargeChangeM3S);
            return { values: result.annualRealizedDischargeDeltaM3S, minimum: -bound, maximum: bound, lowHue: 25, highHue: 205 };
        }
        case 'reconciliation-flow-presence-delta': {
            const bound = Math.max(1e-6, result.reconciliationMetrics.maximumAbsoluteFlowPresenceChange);
            return { values: result.flowPresenceDelta, minimum: -bound, maximum: bound, lowHue: 25, highHue: 205 };
        }
        case 'reconciliation-flow-regime-change': {
            for (let index = 0; index < scalarScratch.length; index += 1)
                scalarScratch[index] = result.flowRegimeChangedMask[index];
            return { values: scalarScratch, minimum: 0, maximum: 1, lowHue: 210, highHue: 5 };
        }
        case 'evolution-solid-elevation': return { values: result.evolvedSolidElevationM, minimum: -12_000, maximum: 8_000, lowHue: 225, highHue: 25 };
        case 'evolution-terrain-delta': {
            const bound = Math.max(0.01, result.evolutionMetrics.maximumAbsoluteTerrainChangeM);
            return { values: result.terrainDeltaM, minimum: -bound, maximum: bound, lowHue: 225, highHue: 20 };
        }
        case 'evolution-applied-erosion': return { values: result.appliedErosionM, minimum: 0, maximum: Math.max(0.01, result.evolutionMetrics.maximumAppliedErosionM), lowHue: 55, highHue: 345 };
        case 'evolution-applied-deposition': return { values: result.appliedDepositionM, minimum: 0, maximum: Math.max(0.01, result.evolutionMetrics.maximumAppliedDepositionM), lowHue: 205, highHue: 45 };
        case 'evolution-receiver-change': {
            for (let index = 0; index < scalarScratch.length; index += 1)
                scalarScratch[index] = result.receiverChangedMask[index];
            return { values: scalarScratch, minimum: 0, maximum: 1, lowHue: 210, highHue: 5 };
        }
        case 'evolution-contributing-area': {
            let maximum = 0;
            for (let index = 0; index < scalarScratch.length; index += 1) {
                const value = Math.log1p(Math.max(0, result.postErosionContributingAreaM2[index]));
                scalarScratch[index] = value;
                maximum = Math.max(maximum, value);
            }
            return { values: scalarScratch, minimum: 0, maximum: Math.max(1e-6, maximum), lowHue: 55, highHue: 205 };
        }
        case 'evolution-potential-discharge': {
            for (let index = 0; index < scalarScratch.length; index += 1)
                scalarScratch[index] = Math.log1p(Math.max(0, result.postErosionPotentialDischargeM3S[index]));
            return { values: scalarScratch, minimum: 0, maximum: Math.log1p(Math.max(1e-6, result.evolutionMetrics.maximumPostErosionPotentialDischargeM3S)), lowHue: 205, highHue: 20 };
        }
        case 'erosion-effective-discharge': {
            for (let index = 0; index < scalarScratch.length; index += 1)
                scalarScratch[index] = Math.log1p(Math.max(0, result.effectiveDischargeM3S[index]));
            return { values: scalarScratch, minimum: 0, maximum: Math.log1p(Math.max(1e-6, result.erosionMetrics.maximumEffectiveDischargeM3S)), lowHue: 205, highHue: 20 };
        }
        case 'erosion-channel-slope': return { values: result.channelSlope, minimum: 0, maximum: Math.max(1e-6, result.erosionMetrics.maximumChannelSlope), lowHue: 52, highHue: 350 };
        case 'erosion-channel-width': return { values: result.channelWidthM, minimum: 0, maximum: Math.max(1, result.erosionMetrics.maximumChannelWidthM), lowHue: 210, highHue: 30 };
        case 'erosion-erodibility': return { values: result.erodibilityIndex, minimum: 0.05, maximum: 1, lowHue: 125, highHue: 5 };
        case 'erosion-incision-potential': return { values: result.incisionPotentialMPerYear, minimum: 0, maximum: Math.max(1e-9, result.erosionMetrics.maximumIncisionPotentialMPerYear), lowHue: 55, highHue: 345 };
        case 'erosion-sediment-load': {
            for (let index = 0; index < scalarScratch.length; index += 1)
                scalarScratch[index] = Math.log1p(Math.max(0, result.sedimentLoadKgS[index]));
            return { values: scalarScratch, minimum: 0, maximum: Math.log1p(Math.max(1e-6, result.erosionMetrics.maximumSedimentLoadKgS)), lowHue: 45, highHue: 300 };
        }
        case 'erosion-sediment-supply': {
            let maximum = 0;
            for (let index = 0; index < scalarScratch.length; index += 1) {
                const value = Math.log1p(Math.max(0, result.localSedimentSupplyKgS[index]));
                scalarScratch[index] = value;
                maximum = Math.max(maximum, value);
            }
            return { values: scalarScratch, minimum: 0, maximum: Math.max(1e-6, maximum), lowHue: 58, highHue: 325 };
        }
        case 'erosion-sediment-deposition': {
            let maximum = 0;
            for (let index = 0; index < scalarScratch.length; index += 1) {
                const value = Math.log1p(Math.max(0, result.sedimentDepositionKgS[index]));
                scalarScratch[index] = value;
                maximum = Math.max(maximum, value);
            }
            return { values: scalarScratch, minimum: 0, maximum: Math.max(1e-6, maximum), lowHue: 35, highHue: 285 };
        }
        case 'precip-seasonality': return { values: result.precipitationSeasonality, minimum: 0, maximum: 5, lowHue: 205, highHue: 335 };
        case 'potential-evaporation': return { values: result.potentialEvaporationMm, minimum: 0, maximum: 3_000, lowHue: 205, highHue: 20 };
        case 'moisture-balance': return { values: result.moistureBalanceMm, minimum: -2_000, maximum: 2_000, lowHue: 25, highHue: 210 };
        case 'aridity': return { values: result.aridityIndex, minimum: 0, maximum: 2, lowHue: 20, highHue: 165 };
        case 'snowfall': return { values: result.snowfallFraction, minimum: 0, maximum: 1, lowHue: 210, highHue: 190 };
        case 'persistent-snow': return { values: result.persistentSnowPotential, minimum: 0, maximum: 1, lowHue: 220, highHue: 185 };
        case 'sea-ice': return { values: result.seaIcePotential, minimum: 0, maximum: 1, lowHue: 225, highHue: 175 };
        default: return null;
    }
}
function sampleColor(result, mode, sample, field) {
    if (mode === 'physical-world')
        return evolvedHypsometricColor(result, sample);
    if (mode === 'physical-elevation' || mode === 'winds' || mode === 'currents')
        return hypsometricColor(result, sample);
    if (mode === 'land-water')
        return result.submergedMask[sample] ? '#214d7a' : '#a99b72';
    if (mode === 'plates' || mode === 'tectonic-boundaries' || mode === 'geological-boundaries' || mode === 'boundary-provenance')
        return plateColor(result.plateIds[sample]);
    if (mode === 'kinematic-domains')
        return plateColor(result.kinematicDomainIds[sample]);
    if (mode === 'crust-type')
        return crustColor(result.crustKind[sample]);
    if (mode === 'structural-zones')
        return structuralColor(result.structuralZoneKind[sample]);
    if (mode === 'seasonal-flow-regime') {
        if (result.submergedMask[sample])
            return '#102c43';
        const regime = result.seasonalFlowRegime[sample];
        if (regime === 2)
            return '#4ea7dd';
        if (regime === 1)
            return '#e3a54f';
        return '#31423c';
    }
    if (mode === 'provenance')
        return provenanceColor(result.nearestCoarseSource[sample]);
    if (mode === 'inherited-mask')
        return result.inheritedSampleMask[sample] ? '#f4e27a' : '#5794c8';
    if (RECONCILIATION_MODES.has(mode) && result.submergedMask[sample])
        return '#102c43';
    if (EVOLUTION_MODES.has(mode) && result.submergedMask[sample])
        return '#102c43';
    if (EROSION_MODES.has(mode) && mode !== 'erosion-sediment-deposition' && result.submergedMask[sample])
        return '#102c43';
    if (field)
        return scalarColor(field.values[sample], field);
    return '#8297aa';
}
function projectSamples(result, projection, yaw, pitch, width, height, buffers) {
    const count = result.metrics.fineSampleCount;
    const positions = result.positions;
    if (projection === 'map') {
        for (let sample = 0; sample < count; sample += 1) {
            const offset = sample * 3;
            const px = positions[offset];
            const py = positions[offset + 1];
            const pz = positions[offset + 2];
            buffers.x[sample] = (Math.atan2(py, px) + Math.PI) / TWO_PI * width;
            buffers.y[sample] = (Math.PI / 2 - Math.asin(Math.max(-1, Math.min(1, pz)))) / Math.PI * height;
            buffers.visible[sample] = 1;
        }
        return;
    }
    const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch);
    const radius = Math.min(width, height) * 0.44;
    for (let sample = 0; sample < count; sample += 1) {
        const offset = sample * 3;
        const px = positions[offset];
        const py = positions[offset + 1];
        const pz = positions[offset + 2];
        const x1 = cy * px - sy * py;
        const y1 = sy * px + cy * py;
        const rotatedX = cp * x1 + sp * pz;
        const rotatedZ = -sp * x1 + cp * pz;
        buffers.x[sample] = width / 2 + y1 * radius;
        buffers.y[sample] = height / 2 - rotatedZ * radius;
        buffers.visible[sample] = rotatedX >= 0 ? 1 : 0;
    }
}
function screenTangentDelta(position, eastValue, northValue, projection, yaw, pitch, width, height) {
    const [x, y, z] = position;
    const lon = Math.atan2(y, x);
    const lat = Math.asin(Math.max(-1, Math.min(1, z)));
    const east = [-Math.sin(lon), Math.cos(lon), 0];
    const north = [-Math.sin(lat) * Math.cos(lon), -Math.sin(lat) * Math.sin(lon), Math.cos(lat)];
    const speed = Math.hypot(eastValue, northValue);
    if (speed < 1e-9)
        return [0, 0];
    const tangent = [
        (eastValue * east[0] + northValue * north[0]) / speed,
        (eastValue * east[1] + northValue * north[1]) / speed,
        (eastValue * east[2] + northValue * north[2]) / speed,
    ];
    if (projection === 'map')
        return mapVectorDelta(eastValue, northValue, lat, width, height);
    const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch);
    const x1 = cy * tangent[0] - sy * tangent[1];
    const y1 = sy * tangent[0] + cy * tangent[1];
    const rotatedZ = -sp * x1 + cp * tangent[2];
    const radius = Math.min(width, height) * 0.44;
    return [y1 * radius * 0.055, -rotatedZ * radius * 0.055];
}
let styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
function buildStyleCache(result, mode, phase) {
    const field = scalarField(result, mode, phase);
    const phaseKey = ['seasonal-temperature', 'seasonal-sst', 'seasonal-precipitation', 'seasonal-realized-discharge', 'seasonal-snow-storage'].includes(mode) ? phase.toFixed(3) : 'mean';
    const key = `${mode}:${phaseKey}`;
    const sampleBuckets = mode === 'mesh' ? [] : bucketize(result.metrics.fineSampleCount, sample => sampleColor(result, mode, sample, field));
    let boundaryBuckets = [];
    if (mode === 'tectonic-boundaries')
        boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => tectonicBoundaryColor(result.boundaryKinds[boundary]));
    else if (mode === 'geological-boundaries')
        boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => geologicalBoundaryColor(result.geologicalBoundaryRegimes[boundary]));
    else if (mode === 'boundary-provenance')
        boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => provenanceColor(result.boundaryCoarseSourceIndices[boundary]));
    return { result, key, sampleBuckets, boundaryBuckets };
}
function drawVectors(context, result, mode, phase, projection, yaw, pitch, width, height, buffers, animation) {
    const count = result.metrics.fineSampleCount;
    const targetVectors = 1_300;
    const stride = Math.max(1, Math.floor(count / targetVectors));
    const angle = phase * TWO_PI;
    const c = Math.cos(angle), s = Math.sin(angle);
    context.save();
    context.lineWidth = mode === 'winds' ? 1.15 : 1.5;
    context.strokeStyle = mode === 'winds' ? 'rgba(245,249,255,0.82)' : 'rgba(91,220,255,0.92)';
    context.fillStyle = context.strokeStyle;
    context.setLineDash([4, 5]);
    context.lineDashOffset = -animation;
    for (let sample = 0; sample < count; sample += stride) {
        if (!buffers.visible[sample])
            continue;
        if (mode === 'currents' && !result.submergedMask[sample])
            continue;
        let east;
        let north;
        if (mode === 'winds') {
            east = result.windEastMeanMS[sample] + result.windEastAnnualCosMS[sample] * c + result.windEastAnnualSinMS[sample] * s;
            north = result.windNorthMeanMS[sample] + result.windNorthAnnualCosMS[sample] * c + result.windNorthAnnualSinMS[sample] * s;
        }
        else {
            east = result.currentEastMeanMS[sample] + result.currentEastAnnualCosMS[sample] * c + result.currentEastAnnualSinMS[sample] * s;
            north = result.currentNorthMeanMS[sample] + result.currentNorthAnnualCosMS[sample] * c + result.currentNorthAnnualSinMS[sample] * s;
        }
        const speed = Math.hypot(east, north);
        if (speed < (mode === 'winds' ? 0.6 : 0.025))
            continue;
        const offset = sample * 3;
        const position = [result.positions[offset], result.positions[offset + 1], result.positions[offset + 2]];
        let [dx, dy] = screenTangentDelta(position, east, north, projection, yaw, pitch, width, height);
        const scale = mode === 'winds' ? Math.min(2.1, 0.6 + speed / 12) : Math.min(2.4, 0.8 + speed * 1.8);
        dx *= scale;
        dy *= scale;
        const x = buffers.x[sample], y = buffers.y[sample];
        context.beginPath();
        context.moveTo(x - dx * 0.35, y - dy * 0.35);
        context.lineTo(x + dx, y + dy);
        context.stroke();
    }
    context.restore();
}
let edgeOverlayCache = { result: null, coastline: new Uint32Array(0), contours: [], evolvedContours: [], basinDivides: new Uint32Array(0), riverBuckets: [] };
const TOPOGRAPHIC_CONTOURS_M = [500, 1_000, 2_000, 3_000, 4_500];
function ensureEdgeOverlayCache(result) {
    if (edgeOverlayCache.result === result)
        return edgeOverlayCache;
    const coastline = [];
    const basinDivides = [];
    const contourPairs = TOPOGRAPHIC_CONTOURS_M.map(() => []);
    const evolvedContourPairs = TOPOGRAPHIC_CONTOURS_M.map(() => []);
    for (let a = 0; a < result.metrics.fineSampleCount; a += 1) {
        const start = result.neighborOffsets[a];
        const end = result.neighborOffsets[a + 1];
        for (let cursor = start; cursor < end; cursor += 1) {
            const b = result.neighbors[cursor];
            if (b <= a)
                continue;
            if (result.submergedMask[a] !== result.submergedMask[b])
                coastline.push(a, b);
            if (result.submergedMask[a] || result.submergedMask[b])
                continue;
            const basinA = result.basinId[a];
            const basinB = result.basinId[b];
            if (basinA !== WORLDGEN_INVALID_SAMPLE_ID && basinB !== WORLDGEN_INVALID_SAMPLE_ID && basinA !== basinB)
                basinDivides.push(a, b);
            const ea = result.elevationAboveSeaLevelM[a];
            const eb = result.elevationAboveSeaLevelM[b];
            const evolvedA = ea + result.terrainDeltaM[a];
            const evolvedB = eb + result.terrainDeltaM[b];
            for (let levelIndex = 0; levelIndex < TOPOGRAPHIC_CONTOURS_M.length; levelIndex += 1) {
                const level = TOPOGRAPHIC_CONTOURS_M[levelIndex];
                if ((ea < level && eb >= level) || (eb < level && ea >= level))
                    contourPairs[levelIndex].push(a, b);
                if ((evolvedA < level && evolvedB >= level) || (evolvedB < level && evolvedA >= level))
                    evolvedContourPairs[levelIndex].push(a, b);
            }
        }
    }
    const riverBuckets = new Map();
    const maximumFlow = Math.max(1, result.lakeMetrics.maximumRealizedDischargeM3S);
    const maximumLogFlow = Math.log1p(maximumFlow);
    for (let sample = 0; sample < result.metrics.fineSampleCount; sample += 1) {
        if (result.submergedMask[sample])
            continue;
        const downstream = result.receiver[sample];
        if (downstream === WORLDGEN_INVALID_SAMPLE_ID || downstream >= result.metrics.fineSampleCount)
            continue;
        const discharge = Math.max(0, result.realizedDischargeM3S[sample]);
        const regime = result.seasonalFlowRegime[sample];
        if (discharge < 1 || regime === 0)
            continue;
        const normalized = Math.log1p(discharge) / maximumLogFlow;
        const widthBucket = Math.max(0, Math.min(5, Math.floor(normalized * 6)));
        const key = `${regime}:${widthBucket}`;
        let bucket = riverBuckets.get(key);
        if (!bucket) {
            bucket = { regime, widthBucket, pairs: [] };
            riverBuckets.set(key, bucket);
        }
        bucket.pairs.push(sample, downstream);
    }
    edgeOverlayCache = {
        result,
        coastline: Uint32Array.from(coastline),
        contours: TOPOGRAPHIC_CONTOURS_M.map((level, index) => ({ level, pairs: Uint32Array.from(contourPairs[index]) })),
        evolvedContours: TOPOGRAPHIC_CONTOURS_M.map((level, index) => ({ level, pairs: Uint32Array.from(evolvedContourPairs[index]) })),
        basinDivides: Uint32Array.from(basinDivides),
        riverBuckets: Array.from(riverBuckets.values(), bucket => ({ regime: bucket.regime, widthBucket: bucket.widthBucket, pairs: Uint32Array.from(bucket.pairs) })),
    };
    return edgeOverlayCache;
}
function strokeSamplePairs(context, pairs, buffers, projection, width, strokeStyle, lineWidth) {
    context.save();
    context.strokeStyle = strokeStyle;
    context.lineWidth = lineWidth;
    context.lineCap = 'round';
    context.beginPath();
    for (let cursor = 0; cursor < pairs.length; cursor += 2) {
        const a = pairs[cursor], b = pairs[cursor + 1];
        if (!buffers.visible[a] || !buffers.visible[b])
            continue;
        const ax = buffers.x[a], bx = buffers.x[b];
        if (projection === 'map' && Math.abs(ax - bx) > width * 0.45)
            continue;
        context.moveTo(ax, buffers.y[a]);
        context.lineTo(bx, buffers.y[b]);
    }
    context.stroke();
    context.restore();
}
function drawBoundaryOverlay(context, result, kind, projection, width, buffers) {
    const buckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => kind === 'tectonic-boundaries'
        ? tectonicBoundaryColor(result.boundaryKinds[boundary])
        : geologicalBoundaryColor(result.geologicalBoundaryRegimes[boundary]));
    context.save();
    context.lineCap = 'round';
    context.lineWidth = 1.8;
    for (const bucket of buckets) {
        context.strokeStyle = bucket.color;
        context.beginPath();
        for (let cursor = 0; cursor < bucket.indices.length; cursor += 1) {
            const boundary = bucket.indices[cursor];
            const a = result.boundarySamples[boundary * 2], b = result.boundarySamples[boundary * 2 + 1];
            if (!buffers.visible[a] || !buffers.visible[b])
                continue;
            const ax = buffers.x[a], bx = buffers.x[b];
            if (projection === 'map' && Math.abs(ax - bx) > width * 0.45)
                continue;
            context.moveTo(ax, buffers.y[a]);
            context.lineTo(bx, buffers.y[b]);
        }
        context.stroke();
    }
    context.restore();
}
function drawFinalLakeOverlay(context, result, buffers) {
    const count = result.metrics.fineSampleCount;
    const radius = count > 100_000 ? 1.0 : count > 30_000 ? 1.45 : 2.2;
    context.save();
    for (let sample = 0; sample < count; sample += 1) {
        if (!buffers.visible[sample] || result.submergedMask[sample] || result.lakeFraction[sample] <= 0.01)
            continue;
        const depth = Math.max(0, result.lakeDepthM[sample]);
        const alpha = Math.max(0.48, Math.min(0.94, 0.55 + Math.log1p(depth) / 14));
        context.fillStyle = `rgba(65,174,224,${alpha})`;
        context.beginPath();
        context.arc(buffers.x[sample], buffers.y[sample], radius, 0, TWO_PI);
        context.fill();
    }
    context.restore();
}
function drawCryosphereOverlay(context, result, buffers) {
    const count = result.metrics.fineSampleCount;
    const radius = count > 100_000 ? 0.9 : count > 30_000 ? 1.25 : 1.9;
    context.save();
    for (let sample = 0; sample < count; sample += 1) {
        if (!buffers.visible[sample])
            continue;
        const potential = result.submergedMask[sample] ? result.seaIcePotential[sample] : result.persistentSnowPotential[sample];
        if (potential < 0.2)
            continue;
        const alpha = Math.min(0.82, 0.18 + potential * 0.64);
        context.fillStyle = result.submergedMask[sample] ? `rgba(190,229,244,${alpha})` : `rgba(245,248,250,${alpha})`;
        context.beginPath();
        context.arc(buffers.x[sample], buffers.y[sample], radius, 0, TWO_PI);
        context.fill();
    }
    context.restore();
}
function drawFinalRiverOverlay(context, edgeCache, buffers, projection, width) {
    const ordered = [...edgeCache.riverBuckets].sort((a, b) => a.widthBucket - b.widthBucket);
    for (const bucket of ordered) {
        const perennial = bucket.regime === 2;
        const alpha = perennial ? 0.78 + bucket.widthBucket * 0.035 : 0.46 + bucket.widthBucket * 0.04;
        const stroke = perennial ? `rgba(65,177,236,${Math.min(0.98, alpha)})` : `rgba(99,188,224,${Math.min(0.82, alpha)})`;
        strokeSamplePairs(context, bucket.pairs, buffers, projection, width, stroke, 0.55 + bucket.widthBucket * 0.42);
    }
}
function drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation) {
    if (overlays.size === 0)
        return;
    const edgeCache = ensureEdgeOverlayCache(result);
    if (overlays.has('topography')) {
        const alphas = [0.24, 0.32, 0.42, 0.54, 0.68];
        for (let index = 0; index < edgeCache.contours.length; index += 1) {
            strokeSamplePairs(context, edgeCache.contours[index].pairs, buffers, projection, width, `rgba(245,248,252,${alphas[index]})`, index >= 3 ? 1.1 : 0.8);
        }
    }
    if (overlays.has('evolved-topography')) {
        const alphas = [0.20, 0.28, 0.38, 0.50, 0.64];
        for (let index = 0; index < edgeCache.evolvedContours.length; index += 1) {
            strokeSamplePairs(context, edgeCache.evolvedContours[index].pairs, buffers, projection, width, `rgba(238,242,235,${alphas[index]})`, index >= 3 ? 1.05 : 0.78);
        }
    }
    if (overlays.has('coastline'))
        strokeSamplePairs(context, edgeCache.coastline, buffers, projection, width, 'rgba(225,236,246,0.84)', 1.2);
    if (overlays.has('basin-divides'))
        strokeSamplePairs(context, edgeCache.basinDivides, buffers, projection, width, 'rgba(236,207,132,0.34)', 0.7);
    if (overlays.has('cryosphere'))
        drawCryosphereOverlay(context, result, buffers);
    if (overlays.has('final-lakes'))
        drawFinalLakeOverlay(context, result, buffers);
    if (overlays.has('final-rivers'))
        drawFinalRiverOverlay(context, edgeCache, buffers, projection, width);
    if (overlays.has('tectonic-boundaries'))
        drawBoundaryOverlay(context, result, 'tectonic-boundaries', projection, width, buffers);
    if (overlays.has('geological-boundaries'))
        drawBoundaryOverlay(context, result, 'geological-boundaries', projection, width, buffers);
    if (overlays.has('winds'))
        drawVectors(context, result, 'winds', phase, projection, yaw, pitch, width, height, buffers, animation);
    if (overlays.has('currents'))
        drawVectors(context, result, 'currents', phase, projection, yaw, pitch, width, height, buffers, animation);
}
function renderPlanet(canvas, result, projection, mode, overlays, phase, yaw, pitch, buffers, interactive, animation) {
    const width = 1100;
    const height = projection === 'map' ? 550 : 760;
    if (canvas.width !== width)
        canvas.width = width;
    if (canvas.height !== height)
        canvas.height = height;
    const context = canvas.getContext('2d');
    if (!context)
        throw new Error('Planet Engine Lab could not acquire a 2D canvas context.');
    context.fillStyle = '#08101a';
    context.fillRect(0, 0, width, height);
    projectSamples(result, projection, yaw, pitch, width, height, buffers);
    if (projection === 'globe') {
        context.beginPath();
        context.arc(width / 2, height / 2, Math.min(width, height) * 0.44, 0, TWO_PI);
        context.strokeStyle = '#5d7890';
        context.lineWidth = 1;
        context.stroke();
    }
    if (isLakeMode(mode)) {
        const count = result.metrics.fineSampleCount;
        const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
        const fastPoints = interactive && count > 20_000;
        context.globalAlpha = 0.94;
        for (let sample = 0; sample < count; sample += 1) {
            if (!buffers.visible[sample])
                continue;
            context.fillStyle = lakeSampleColor(result, mode, sample);
            const x = buffers.x[sample], y = buffers.y[sample];
            if (fastPoints)
                context.fillRect(x - 0.75, y - 0.75, 1.5, 1.5);
            else {
                context.beginPath();
                context.arc(x, y, pointRadius, 0, TWO_PI);
                context.fill();
            }
        }
        context.globalAlpha = 1;
        drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation);
        return;
    }
    if (isRunoffMode(mode)) {
        const count = result.metrics.fineSampleCount;
        const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
        const fastPoints = interactive && count > 20_000;
        context.globalAlpha = 0.94;
        for (let sample = 0; sample < count; sample += 1) {
            if (!buffers.visible[sample])
                continue;
            context.fillStyle = runoffSampleColor(result, mode, sample);
            const x = buffers.x[sample], y = buffers.y[sample];
            if (fastPoints)
                context.fillRect(x - 0.75, y - 0.75, 1.5, 1.5);
            else {
                context.beginPath();
                context.arc(x, y, pointRadius, 0, TWO_PI);
                context.fill();
            }
        }
        context.globalAlpha = 1;
        drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation);
        return;
    }
    if (isDrainageMode(mode)) {
        renderDrainageDiagnostic(context, result, projection, mode, width, buffers, interactive);
        drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation);
        return;
    }
    if (mode === 'mesh') {
        context.beginPath();
        context.strokeStyle = '#35536d';
        context.lineWidth = 0.65;
        for (let sample = 0; sample < result.metrics.fineSampleCount; sample += 1) {
            if (!buffers.visible[sample])
                continue;
            const ax = buffers.x[sample], ay = buffers.y[sample];
            for (let cursor = result.neighborOffsets[sample]; cursor < result.neighborOffsets[sample + 1]; cursor += 1) {
                const neighbor = result.neighbors[cursor];
                if (neighbor <= sample || !buffers.visible[neighbor])
                    continue;
                const bx = buffers.x[neighbor];
                if (projection === 'map' && Math.abs(ax - bx) > width / 2)
                    continue;
                context.moveTo(ax, ay);
                context.lineTo(bx, buffers.y[neighbor]);
            }
        }
        context.stroke();
        return;
    }
    const cacheKey = `${mode}:${['seasonal-temperature', 'seasonal-sst', 'seasonal-precipitation', 'seasonal-realized-discharge', 'seasonal-snow-storage'].includes(mode) ? phase.toFixed(3) : 'mean'}`;
    if (styleCache.result !== result || styleCache.key !== cacheKey)
        styleCache = buildStyleCache(result, mode, phase);
    const count = result.metrics.fineSampleCount;
    const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
    const fastPoints = interactive && count > 20_000;
    const boundaryMode = styleCache.boundaryBuckets.length > 0;
    context.globalAlpha = boundaryMode ? 0.28 : 0.94;
    for (const bucket of styleCache.sampleBuckets) {
        context.fillStyle = bucket.color;
        if (!fastPoints)
            context.beginPath();
        for (let cursor = 0; cursor < bucket.indices.length; cursor += 1) {
            const sample = bucket.indices[cursor];
            if (!buffers.visible[sample])
                continue;
            const x = buffers.x[sample], y = buffers.y[sample];
            if (fastPoints)
                context.fillRect(x - 0.75, y - 0.75, 1.5, 1.5);
            else {
                context.moveTo(x + pointRadius, y);
                context.arc(x, y, pointRadius, 0, TWO_PI);
            }
        }
        if (!fastPoints)
            context.fill();
    }
    context.globalAlpha = 1;
    if (boundaryMode) {
        context.lineCap = 'round';
        context.lineWidth = mode === 'boundary-provenance' ? 1.4 : 2.0;
        for (const bucket of styleCache.boundaryBuckets) {
            context.strokeStyle = bucket.color;
            context.beginPath();
            for (let cursor = 0; cursor < bucket.indices.length; cursor += 1) {
                const boundary = bucket.indices[cursor];
                const a = result.boundarySamples[boundary * 2], b = result.boundarySamples[boundary * 2 + 1];
                if (!buffers.visible[a] || !buffers.visible[b])
                    continue;
                const ax = buffers.x[a], bx = buffers.x[b];
                if (projection === 'map' && Math.abs(ax - bx) > width / 2)
                    continue;
                context.moveTo(ax, buffers.y[a]);
                context.lineTo(bx, buffers.y[b]);
            }
            context.stroke();
        }
    }
    if (mode === 'winds' || mode === 'currents')
        drawVectors(context, result, mode, phase, projection, yaw, pitch, width, height, buffers, animation);
    drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation);
}
const seed = element('worldgen-seed');
const coarseLevel = element('worldgen-coarse-level');
const fineLevel = element('worldgen-level');
const plates = element('worldgen-plates');
const projection = element('worldgen-projection');
const preset = element('worldgen-preset');
const visualization = element('worldgen-visualization');
const season = element('worldgen-season');
const seasonValue = element('worldgen-season-value');
const overlaySummary = element('worldgen-overlay-summary');
const overlayInputs = Array.from(document.querySelectorAll('input[data-worldgen-overlay]'));
const generate = element('worldgen-generate');
const status = element('worldgen-status');
const generationProgress = element('worldgen-generation-progress');
const generationStage = element('worldgen-generation-stage');
const generationStep = element('worldgen-generation-step');
const generationTimer = element('worldgen-generation-timer');
const generationProfile = element('worldgen-generation-profile');
const metrics = element('worldgen-metrics');
const canvas = element('worldgen-field');
const client = createWorldgenClient();
let current = null;
let buffers = null;
let yaw = -0.65;
let pitch = 0.25;
let drag = null;
let frameRequest = 0;
let animationRequest = 0;
let animationPhase = 0;
let lastVectorAnimationMs = Number.NEGATIVE_INFINITY;
const VECTOR_ANIMATION_INTERVAL_MS = 50;
const GENERATION_STAGE_LABELS = {
    'coarse-topology': 'Coarse topology',
    'fine-topology': 'Fine topology',
    tectonics: 'Tectonics',
    geology: 'Geological history',
    lithosphere: 'Lithosphere',
    inheritance: 'Fine-topology inheritance',
    'boundary-refinement': 'Boundary refinement',
    topography: 'Topography + sea level',
    'climate-spinup': 'Climate spin-up',
    'drainage-topology': 'Drainage topology',
    'runoff-discharge': 'Annual runoff / discharge',
    'lake-equilibrium': 'Lake equilibrium',
    'seasonal-hydrology': 'Seasonal hydrology',
    'fluvial-erosion-sediment': 'Fluvial erosion / sediment',
    'bounded-terrain-evolution': 'Bounded terrain evolution',
    'post-erosion-hydrology': 'Post-erosion hydrology reconciliation',
    packaging: 'Packaging / transfer',
};
let generationStartedAt = 0;
let generationTimerHandle = null;
function selectedOverlays() {
    return new Set(overlayInputs.filter(input => input.checked).map(input => input.value));
}
function updateOverlaySummary() {
    const selected = overlayInputs.filter(input => input.checked);
    if (selected.length === 0)
        overlaySummary.textContent = 'None';
    else if (selected.length === 1)
        overlaySummary.textContent = selected[0].dataset.label ?? selected[0].value;
    else
        overlaySummary.textContent = `${selected.length} selected`;
}
const VIEW_PRESETS = {
    'physical-world': { mode: 'physical-world', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'cryosphere'] },
    'hydrologic-atlas': { mode: 'physical-world', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'basin-divides'] },
    'seasonal-world': { mode: 'seasonal-realized-discharge', overlays: ['evolved-topography', 'coastline', 'final-lakes', 'winds'] },
    'geomorphic-processes': { mode: 'evolution-terrain-delta', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'tectonic-boundaries'] },
};
function applyViewPreset(name) {
    const definition = VIEW_PRESETS[name];
    if (!definition)
        return;
    visualization.value = definition.mode;
    const wanted = new Set(definition.overlays);
    for (const input of overlayInputs)
        input.checked = wanted.has(input.value);
    styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
    updateOverlaySummary();
    redraw(false);
    updateAnimation();
}
function formatDuration(ms) {
    if (ms < 1_000)
        return `${ms.toFixed(0)} ms`;
    return `${(ms / 1_000).toFixed(2)} s`;
}
function startGenerationTelemetry() {
    generationStartedAt = performance.now();
    generationProgress.value = 0;
    generationStage.textContent = 'Starting';
    generationStep.textContent = '';
    generationProfile.replaceChildren();
    if (generationTimerHandle)
        clearInterval(generationTimerHandle);
    const updateTimer = () => { generationTimer.textContent = formatDuration(performance.now() - generationStartedAt); };
    updateTimer();
    generationTimerHandle = setInterval(updateTimer, 100);
}
function handleGenerationProgress(progress) {
    const stageFraction = progress.total > 0 ? Math.max(0, Math.min(1, progress.completed / progress.total)) : 0;
    generationProgress.value = Math.max(0, Math.min(100, (progress.stageIndex + stageFraction) / Math.max(1, progress.stageCount) * 100));
    generationStage.textContent = GENERATION_STAGE_LABELS[progress.stageId] ?? progress.stageId;
    generationStep.textContent = progress.stageId === 'climate-spinup'
        ? `year ${progress.completed} / max ${progress.total}`
        : progress.completed >= progress.total ? 'complete' : 'running';
}
function showGenerationProfile(result) {
    generationProfile.replaceChildren();
    const total = result.generationTimings.reduce((sum, timing) => sum + timing.durationMs, 0);
    for (const timing of result.generationTimings) {
        const row = document.createElement('div');
        const label = document.createElement('span');
        const duration = document.createElement('span');
        const share = document.createElement('span');
        label.textContent = GENERATION_STAGE_LABELS[timing.stageId] ?? timing.stageId;
        duration.textContent = formatDuration(timing.durationMs);
        share.textContent = total > 0 ? `${(timing.durationMs / total * 100).toFixed(1)}%` : '—';
        row.append(label, duration, share);
        generationProfile.append(row);
    }
}
function finishGenerationTelemetry(result) {
    if (generationTimerHandle) {
        clearInterval(generationTimerHandle);
        generationTimerHandle = null;
    }
    generationProgress.value = 100;
    generationStage.textContent = 'Complete';
    generationStep.textContent = `${result.metrics.spinupYears} climate spin-up years`;
    generationTimer.textContent = formatDuration(result.stage.durationMs);
    showGenerationProfile(result);
}
function orbitalPhase() { return Number(season.value) / 1000; }
function updateSeasonLabel() { seasonValue.textContent = `${(orbitalPhase() * 100).toFixed(1)}% orbit`; }
function redraw(interactive = false) {
    if (!current || !buffers)
        return;
    renderPlanet(canvas, current, projection.value, visualization.value, selectedOverlays(), orbitalPhase(), yaw, pitch, buffers, interactive, animationPhase);
}
function scheduleRedraw(interactive) {
    if (frameRequest)
        return;
    frameRequest = requestAnimationFrame(() => { frameRequest = 0; redraw(interactive && drag !== null); });
}
function vectorAnimationFrame(timestampMs) {
    animationRequest = 0;
    const overlays = selectedOverlays();
    const vectorsActive = visualization.value === 'winds' || visualization.value === 'currents'
        || overlays.has('winds') || overlays.has('currents');
    if (vectorsActive) {
        if (timestampMs - lastVectorAnimationMs >= VECTOR_ANIMATION_INTERVAL_MS) {
            animationPhase = (animationPhase + 0.9) % 1000;
            lastVectorAnimationMs = timestampMs;
            redraw(true);
        }
        animationRequest = requestAnimationFrame(vectorAnimationFrame);
    }
}
function updateAnimation() {
    if (animationRequest) {
        cancelAnimationFrame(animationRequest);
        animationRequest = 0;
    }
    lastVectorAnimationMs = Number.NEGATIVE_INFINITY;
    const overlays = selectedOverlays();
    if (visualization.value === 'winds' || visualization.value === 'currents' || overlays.has('winds') || overlays.has('currents'))
        animationRequest = requestAnimationFrame(vectorAnimationFrame);
}
function showMetrics(result) {
    metrics.replaceChildren();
    metric(metrics, 'Engine / stage', `v${result.engineVersion} · ${result.stage.id}@${result.stage.version}`);
    metric(metrics, 'Resolution', `L${result.coarseLevel} → L${result.fineLevel} · climate solved at L${result.metrics.globalSolverLevel} (${result.metrics.globalSolverSampleCount.toLocaleString()} cells)`);
    metric(metrics, 'Samples / phases', `${result.metrics.fineSampleCount.toLocaleString()} · ${result.metrics.orbitalPhaseCount}`);
    metric(metrics, 'Climate hash', result.metrics.climateHash);
    metric(metrics, 'Topography hash', result.metrics.topographyHash);
    metric(metrics, 'Temperature', `${result.metrics.minimumTemperatureK.toFixed(1)} → ${result.metrics.maximumTemperatureK.toFixed(1)} K · mean ${result.metrics.meanTemperatureK.toFixed(1)} K`);
    metric(metrics, 'Land / ocean temp', `${result.metrics.meanLandTemperatureK.toFixed(1)} / ${result.metrics.meanOceanTemperatureK.toFixed(1)} K`);
    metric(metrics, 'Wind', `${result.metrics.meanWindSpeedMS.toFixed(2)} mean · ${result.metrics.maximumWindSpeedMS.toFixed(2)} max m/s`);
    metric(metrics, 'Surface current', `${result.metrics.meanSurfaceCurrentMS.toFixed(3)} mean · ${result.metrics.maximumSurfaceCurrentMS.toFixed(3)} max m/s`);
    metric(metrics, 'Mean SST', `${result.metrics.meanSeaSurfaceTemperatureK.toFixed(1)} K`);
    metric(metrics, 'Precipitation', `${result.metrics.meanAnnualPrecipitationMm.toFixed(0)} mean · P95 ${result.metrics.p95AnnualPrecipitationMm.toFixed(0)} mm/yr`);
    metric(metrics, 'Moisture budget error', result.metrics.moistureBudgetRelativeError.toExponential(2));
    metric(metrics, 'Moisture limiter', `${(result.metrics.moistureTransportLimiterFraction * 100).toFixed(4)}% donor steps`);
    metric(metrics, 'Moisture substeps', `${result.metrics.maximumMoistureTransportSubsteps} maximum`);
    metric(metrics, 'Snow / sea ice potential', `${(result.metrics.persistentSnowAreaFraction * 100).toFixed(1)}% / ${(result.metrics.seaIceAreaFraction * 100).toFixed(1)}% area`);
    metric(metrics, 'Spin-up', `${result.metrics.spinupYears} model years · ΔT ${result.metrics.finalTemperatureRmsChangeK.toFixed(3)} K RMS`);
    metric(metrics, 'Planet forcing', `${result.planet.stellarFluxWM2.toFixed(0)} W/m² · tilt ${(result.planet.axialTiltRad * 180 / Math.PI).toFixed(2)}° · e ${result.climatePhysical.orbitalEccentricity.toFixed(4)}`);
    metric(metrics, 'Land / ocean', `${(result.metrics.landAreaFraction * 100).toFixed(1)}% / ${(result.metrics.oceanAreaFraction * 100).toFixed(1)}%`);
    metric(metrics, 'Climate duration', `${result.stage.durationMs.toFixed(1)} ms`);
    metric(metrics, 'Hydrology / stage', `v${result.engineVersion} · ${result.drainageStage.id}@${result.drainageStage.version}`);
    metric(metrics, 'Drainage topology', `${result.drainageMetrics.basinCount.toLocaleString()} basins · ${result.drainageMetrics.depressionCount.toLocaleString()} depressions`);
    metric(metrics, 'Largest contributing area', `${(result.drainageMetrics.maximumContributingAreaM2 / 1e12).toFixed(3)} million km²`);
    metric(metrics, 'Deepest depression', `${result.drainageMetrics.maximumDepressionDepthM.toFixed(1)} m`);
    metric(metrics, 'Drainage area closure', result.drainageMetrics.areaConservationRelativeError.toExponential(2));
    metric(metrics, 'Drainage hash', result.drainageMetrics.drainageHash);
    metric(metrics, 'Hydrology identity', result.runoffMetrics.climateHash === result.metrics.climateHash && result.runoffMetrics.drainageHash === result.drainageMetrics.drainageHash ? 'WG-5 / WG-6A / WG-6B match' : 'MISMATCH');
    metric(metrics, 'WG-6B water balance', `${result.runoffMetrics.meanLandPrecipitationMm.toFixed(1)} P · ${result.runoffMetrics.meanLandActualEvapotranspirationMm.toFixed(1)} AET · ${result.runoffMetrics.meanLandRunoffMm.toFixed(1)} runoff mm/yr`);
    metric(metrics, 'WG-6B runoff fraction', `${(result.runoffMetrics.landRunoffFraction * 100).toFixed(1)}% of land precipitation`);
    metric(metrics, 'WG-6B max potential discharge', `${result.runoffMetrics.maximumPotentialDischargeM3S.toFixed(1)} m³/s`);
    metric(metrics, 'WG-6B discharge closure', result.runoffMetrics.dischargeConservationRelativeError.toExponential(2));
    metric(metrics, 'WG-6B runoff hash', result.runoffMetrics.runoffHash);
    metric(metrics, 'WG-6C / stage', `v${result.engineVersion} · ${result.lakeStage.id}@${result.lakeStage.version}`);
    metric(metrics, 'WG-6C lakes', `${result.lakeMetrics.lakeCount.toLocaleString()} total · ${result.lakeMetrics.endorheicLakeCount.toLocaleString()} endorheic · ${result.lakeMetrics.overflowingLakeCount.toLocaleString()} overflowing · ${result.lakeMetrics.terminalStorageLakeCount.toLocaleString()} terminal storage`);
    metric(metrics, 'WG-6C lake area / volume', `${(result.lakeMetrics.totalLakeAreaM2 / 1e12).toFixed(3)} million km² · ${(result.lakeMetrics.totalLakeVolumeM3 / 1e12).toFixed(3)} thousand km³`);
    metric(metrics, 'WG-6C deepest lake', `${result.lakeMetrics.maximumLakeDepthM.toFixed(1)} m`);
    metric(metrics, 'WG-6C lake evaporation', `${result.lakeMetrics.totalLakeEvaporationM3S.toFixed(1)} m³/s`);
    metric(metrics, 'WG-6C terminal realized flow', `${result.lakeMetrics.terminalRealizedDischargeM3S.toFixed(1)} m³/s`);
    metric(metrics, 'WG-6C water balance', result.lakeMetrics.waterBalanceRelativeError.toExponential(2));
    metric(metrics, 'WG-6C lake hash', result.lakeMetrics.lakeHash);
    metric(metrics, 'WG-6D / stage', `v${result.engineVersion} · ${result.seasonalStage.id}@${result.seasonalStage.version}`);
    metric(metrics, 'WG-6D flow regimes', `${result.seasonalMetrics.dryFlowSampleCount.toLocaleString()} dry · ${result.seasonalMetrics.intermittentFlowSampleCount.toLocaleString()} intermittent · ${result.seasonalMetrics.perennialFlowSampleCount.toLocaleString()} perennial`);
    metric(metrics, 'WG-6D snowmelt runoff', `${(result.seasonalMetrics.snowmeltRunoffFraction * 100).toFixed(2)}% of seasonal runoff timing`);
    metric(metrics, 'WG-6D max phase realized flow', `${result.seasonalMetrics.maximumPhaseRealizedDischargeM3S.toFixed(1)} m³/s`);
    metric(metrics, 'WG-6D routing / water closure', `${result.seasonalMetrics.seasonalRoutingConservationRelativeError.toExponential(2)} / ${result.seasonalMetrics.seasonalWaterBalanceRelativeError.toExponential(2)}`);
    metric(metrics, 'WG-6D lake cycle', `${result.seasonalMetrics.lakeSpinupYears} years · ${result.seasonalMetrics.finalLakeSurfaceCycleChangeM.toFixed(4)} m surface drift · ${result.seasonalMetrics.maximumSeasonalLakeLevelRangeM.toFixed(3)} m max seasonal range`);
    metric(metrics, 'WG-6D seasonal hash', result.seasonalMetrics.seasonalHydrologyHash);
    metric(metrics, 'WG-7A / stage', `v${result.engineVersion} · ${result.erosionStage.id}@${result.erosionStage.version}`);
    metric(metrics, 'WG-7A erosive samples / lake traps', `${result.erosionMetrics.erosiveSampleCount.toLocaleString()} erosive · ${result.erosionMetrics.activeLakeTrapCount.toLocaleString()} lake traps receiving sediment`);
    metric(metrics, 'WG-7A effective flow / slope / width', `${result.erosionMetrics.maximumEffectiveDischargeM3S.toFixed(1)} m³/s · ${result.erosionMetrics.maximumChannelSlope.toFixed(5)} · ${result.erosionMetrics.maximumChannelWidthM.toFixed(1)} m max`);
    metric(metrics, 'WG-7A max incision potential', `${result.erosionMetrics.maximumIncisionPotentialMPerYear.toFixed(6)} m/yr`);
    metric(metrics, 'WG-7A sediment generation', `${result.erosionMetrics.totalSedimentGeneratedKgS.toFixed(1)} kg/s`);
    metric(metrics, 'WG-7A deposition land / lake / terminal-ocean', `${result.erosionMetrics.totalLandDepositionKgS.toFixed(1)} / ${result.erosionMetrics.totalLakeDepositionKgS.toFixed(1)} / ${result.erosionMetrics.totalTerminalOceanDepositionKgS.toFixed(1)} kg/s`);
    metric(metrics, 'WG-7A sediment closure', result.erosionMetrics.sedimentConservationRelativeError.toExponential(2));
    metric(metrics, 'WG-7A erosion hash', result.erosionMetrics.fluvialErosionHash);
    metric(metrics, 'WG-7B / stage', `v${result.engineVersion} · ${result.evolutionStage.id}@${result.evolutionStage.version}`);
    metric(metrics, 'WG-7B geomorphic horizon', `${result.evolutionMetrics.geomorphicDurationYears.toFixed(0)} years`);
    metric(metrics, 'WG-7B changed terrain samples', `${result.evolutionMetrics.erodedSampleCount.toLocaleString()} eroded · ${result.evolutionMetrics.depositionalSampleCount.toLocaleString()} depositional`);
    metric(metrics, 'WG-7B receiver changes', `${result.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} · ${(result.evolutionMetrics.receiverChangedFraction * 100).toFixed(3)}% of land`);
    metric(metrics, 'WG-7B terrain change', `${result.evolutionMetrics.maximumAppliedErosionM.toFixed(2)} m max erosion · ${result.evolutionMetrics.maximumAppliedDepositionM.toFixed(2)} m max deposition · ${result.evolutionMetrics.meanLandAbsoluteTerrainChangeM.toFixed(3)} m mean |Δz|`);
    metric(metrics, 'WG-7B sediment generation', `${result.evolutionMetrics.totalAppliedSedimentGeneratedKgS.toFixed(1)} kg/s`);
    metric(metrics, 'WG-7B deposition land / lake / terminal-ocean', `${result.evolutionMetrics.totalLandDepositionKgS.toFixed(1)} / ${result.evolutionMetrics.totalLakeSinkKgS.toFixed(1)} / ${result.evolutionMetrics.totalTerminalOceanSinkKgS.toFixed(1)} kg/s`);
    metric(metrics, 'WG-7B sediment closure', result.evolutionMetrics.sedimentConservationRelativeError.toExponential(2));
    metric(metrics, 'WG-7B max post-erosion potential flow', `${result.evolutionMetrics.maximumPostErosionPotentialDischargeM3S.toFixed(1)} m³/s`);
    metric(metrics, 'WG-7B post-erosion runoff closure', result.evolutionMetrics.postErosionRunoffConservationRelativeError.toExponential(2));
    metric(metrics, 'WG-7B evolved surface / drainage hash', `${result.evolutionMetrics.evolvedSurfaceHash} / ${result.evolutionMetrics.postErosionDrainageHash}`);
    metric(metrics, 'WG-7B evolution hash', result.evolutionMetrics.terrainEvolutionHash);
    metric(metrics, 'WG-7C stage', `${result.reconciliationStage.id}@${result.reconciliationStage.version}`);
    metric(metrics, 'WG-7C lakes before / after', `${result.reconciliationMetrics.preErosionLakeCount.toLocaleString()} / ${result.reconciliationMetrics.postErosionLakeCount.toLocaleString()}`);
    metric(metrics, 'WG-7C lake state changes', `${result.reconciliationMetrics.lakeKindChangedSampleCount.toLocaleString()} changed · ${result.reconciliationMetrics.lakeAddedSampleCount.toLocaleString()} added · ${result.reconciliationMetrics.lakeRemovedSampleCount.toLocaleString()} removed`);
    metric(metrics, 'WG-7C max lake depth Δ', `${result.reconciliationMetrics.maximumAbsoluteLakeDepthChangeM.toFixed(3)} m`);
    metric(metrics, 'WG-7C flow regime changes', result.reconciliationMetrics.flowRegimeChangedSampleCount.toLocaleString());
    metric(metrics, 'WG-7C max annual realized-flow Δ', `${result.reconciliationMetrics.maximumAbsoluteAnnualRealizedDischargeChangeM3S.toFixed(1)} m³/s`);
    metric(metrics, 'WG-7C runoff / lake closure', `${result.reconciliationMetrics.reconciledRunoffConservationRelativeError.toExponential(2)} / ${result.reconciliationMetrics.reconciledLakeWaterBalanceRelativeError.toExponential(2)}`);
    metric(metrics, 'WG-7C seasonal routing / water closure', `${result.reconciliationMetrics.reconciledSeasonalRoutingRelativeError.toExponential(2)} / ${result.reconciliationMetrics.reconciledSeasonalWaterBalanceRelativeError.toExponential(2)}`);
    metric(metrics, 'WG-7C reconciled hashes', `${result.reconciliationMetrics.reconciledRunoffHash} / ${result.reconciliationMetrics.reconciledLakeHash} / ${result.reconciliationMetrics.reconciledSeasonalHash}`);
    metric(metrics, 'WG-7C reconciliation hash', result.reconciliationMetrics.postErosionHydrologyHash);
}
async function generatePlanet() {
    generate.disabled = true;
    startGenerationTelemetry();
    status.textContent = 'Generating one physical planet through WG-7B bounded terrain evolution in Rust/WASM…';
    try {
        const request = { seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) };
        const loaded = await client.generateClimate(request, handleGenerationProgress);
        if (loaded.runoffMetrics.climateHash !== loaded.metrics.climateHash)
            throw new Error('WG-6B climate identity does not match accepted WG-5 forcing.');
        if (loaded.runoffMetrics.drainageHash !== loaded.drainageMetrics.drainageHash)
            throw new Error('WG-6B drainage identity does not match accepted WG-6A topology.');
        if (loaded.lakeMetrics.climateHash !== loaded.metrics.climateHash)
            throw new Error('WG-6C climate identity does not match accepted WG-5 forcing.');
        if (loaded.lakeMetrics.drainageHash !== loaded.drainageMetrics.drainageHash)
            throw new Error('WG-6C drainage identity does not match accepted WG-6A topology.');
        if (loaded.lakeMetrics.runoffHash !== loaded.runoffMetrics.runoffHash)
            throw new Error('WG-6C runoff identity does not match accepted WG-6B runoff.');
        if (loaded.seasonalMetrics.climateHash !== loaded.metrics.climateHash)
            throw new Error('WG-6D climate identity does not match accepted WG-5 forcing.');
        if (loaded.seasonalMetrics.drainageHash !== loaded.drainageMetrics.drainageHash)
            throw new Error('WG-6D drainage identity does not match accepted WG-6A topology.');
        if (loaded.seasonalMetrics.runoffHash !== loaded.runoffMetrics.runoffHash)
            throw new Error('WG-6D runoff identity does not match accepted WG-6B runoff.');
        if (loaded.seasonalMetrics.lakeHash !== loaded.lakeMetrics.lakeHash)
            throw new Error('WG-6D lake identity does not match accepted WG-6C state.');
        if (loaded.erosionMetrics.inheritanceHash !== loaded.metrics.inheritanceHash)
            throw new Error('WG-7A inheritance identity does not match accepted fine physical state.');
        if (loaded.erosionMetrics.topographyHash !== loaded.metrics.topographyHash)
            throw new Error('WG-7A topography identity does not match accepted WG-4 terrain.');
        if (loaded.erosionMetrics.drainageHash !== loaded.reconciliationMetrics.preErosionDrainageHash)
            throw new Error('WG-7A drainage identity does not match WG-7C pre-erosion WG-6A ancestry.');
        if (loaded.erosionMetrics.lakeHash !== loaded.reconciliationMetrics.preErosionLakeHash)
            throw new Error('WG-7A lake identity does not match WG-7C pre-erosion WG-6C ancestry.');
        if (loaded.erosionMetrics.seasonalHydrologyHash !== loaded.reconciliationMetrics.preErosionSeasonalHash)
            throw new Error('WG-7A seasonal identity does not match WG-7C pre-erosion WG-6D ancestry.');
        if (loaded.evolutionMetrics.topographyHash !== loaded.metrics.topographyHash)
            throw new Error('WG-7B topography identity does not match accepted WG-4 terrain.');
        if (loaded.evolutionMetrics.drainageHash !== loaded.reconciliationMetrics.preErosionDrainageHash)
            throw new Error('WG-7B drainage identity does not match WG-7C pre-erosion WG-6A ancestry.');
        if (loaded.evolutionMetrics.runoffHash !== loaded.reconciliationMetrics.preErosionRunoffHash)
            throw new Error('WG-7B runoff identity does not match WG-7C pre-erosion WG-6B ancestry.');
        if (loaded.evolutionMetrics.lakeHash !== loaded.reconciliationMetrics.preErosionLakeHash)
            throw new Error('WG-7B lake identity does not match WG-7C pre-erosion WG-6C ancestry.');
        if (loaded.evolutionMetrics.fluvialErosionHash !== loaded.erosionMetrics.fluvialErosionHash)
            throw new Error('WG-7B erosion identity does not match accepted WG-7A forcing.');
        if (loaded.reconciliationMetrics.topographyHash !== loaded.metrics.topographyHash || loaded.reconciliationMetrics.climateHash !== loaded.metrics.climateHash)
            throw new Error('WG-7C immutable WG-4/WG-5 ancestry mismatch.');
        if (loaded.reconciliationMetrics.terrainEvolutionHash !== loaded.evolutionMetrics.terrainEvolutionHash || loaded.reconciliationMetrics.evolvedSurfaceHash !== loaded.evolutionMetrics.evolvedSurfaceHash)
            throw new Error('WG-7C WG-7B terrain ancestry mismatch.');
        if (loaded.reconciliationMetrics.postErosionDrainageHash !== loaded.drainageMetrics.drainageHash)
            throw new Error('WG-7C final drainage identity mismatch.');
        if (loaded.reconciliationMetrics.reconciledRunoffHash !== loaded.runoffMetrics.runoffHash)
            throw new Error('WG-7C final runoff identity mismatch.');
        if (loaded.reconciliationMetrics.reconciledLakeHash !== loaded.lakeMetrics.lakeHash)
            throw new Error('WG-7C final lake identity mismatch.');
        if (loaded.reconciliationMetrics.reconciledSeasonalHash !== loaded.seasonalMetrics.seasonalHydrologyHash)
            throw new Error('WG-7C final seasonal identity mismatch.');
        current = loaded;
        buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };
        styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
        edgeOverlayCache = { result: null, coastline: new Uint32Array(0), contours: [], evolvedContours: [], basinDivides: new Uint32Array(0), riverBuckets: [] };
        showMetrics(loaded);
        redraw(false);
        updateAnimation();
        finishGenerationTelemetry(loaded);
        generationStep.textContent = `${loaded.metrics.spinupYears} climate spin-up years · ${loaded.drainageMetrics.basinCount.toLocaleString()} basins · ${loaded.lakeMetrics.lakeCount.toLocaleString()} equilibrium lakes · ${loaded.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} receivers changed after evolution`;
        generationTimer.textContent = formatDuration(performance.now() - generationStartedAt);
        status.textContent = `Planet ready through WG-7C: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.evolutionMetrics.erodedSampleCount.toLocaleString()} evolved erosion cells, ${loaded.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} drainage receivers changed, mean land |Δz| ${loaded.evolutionMetrics.meanLandAbsoluteTerrainChangeM.toFixed(3)} m, sediment closure ${loaded.evolutionMetrics.sedimentConservationRelativeError.toExponential(2)}.`;
    }
    catch (error) {
        if (generationTimerHandle) {
            clearInterval(generationTimerHandle);
            generationTimerHandle = null;
        }
        generationStage.textContent = 'Generation failed';
        generationStep.textContent = '';
        status.textContent = error instanceof Error ? error.message : String(error);
    }
    finally {
        generate.disabled = false;
    }
}
generate.addEventListener('click', () => void generatePlanet());
projection.addEventListener('change', () => redraw(false));
preset.addEventListener('change', () => applyViewPreset(preset.value));
visualization.addEventListener('change', () => { preset.value = 'custom'; styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; redraw(false); updateAnimation(); });
overlayInputs.forEach(input => input.addEventListener('change', () => { preset.value = 'custom'; updateOverlaySummary(); redraw(false); updateAnimation(); }));
season.addEventListener('input', () => { updateSeasonLabel(); styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; redraw(false); });
canvas.addEventListener('pointerdown', event => {
    if (projection.value !== 'globe')
        return;
    drag = { x: event.clientX, y: event.clientY, yaw, pitch };
    canvas.setPointerCapture(event.pointerId);
});
canvas.addEventListener('pointermove', event => {
    if (!drag || projection.value !== 'globe')
        return;
    yaw = drag.yaw + (event.clientX - drag.x) * 0.007;
    pitch = Math.max(-1.45, Math.min(1.45, drag.pitch + (event.clientY - drag.y) * 0.007));
    scheduleRedraw(true);
});
canvas.addEventListener('pointerup', event => {
    drag = null;
    if (canvas.hasPointerCapture(event.pointerId))
        canvas.releasePointerCapture(event.pointerId);
    scheduleRedraw(false);
});
canvas.addEventListener('pointercancel', () => { drag = null; scheduleRedraw(false); });
window.addEventListener('beforeunload', () => {
    if (frameRequest)
        cancelAnimationFrame(frameRequest);
    if (animationRequest)
        cancelAnimationFrame(animationRequest);
    client.dispose();
});
updateSeasonLabel();
updateOverlaySummary();
applyViewPreset(preset.value);
void generatePlanet();
