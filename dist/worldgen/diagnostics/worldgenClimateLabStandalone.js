import { createWorldgenClient } from '../worldgenClient.js';
import { createWorldgenCrashRecorder, installWorldgenGlobalFailureCapture } from '../worldgenCrashReport.js';
import { worldCalibrationJson, worldCalibrationMarkdown } from '../calibrationPacket.js';
import { clampEquirectangularCenterLatitude, equirectangularCameraForWorldDirectionAtScreen, equirectangularScreenToWorldDirection, mapVectorDelta, reconstructAnnualHarmonicFromBasis, wrapLongitudeRad } from './worldgenClimateMath.js';
import { L8GlobeRenderer, buildRgbaColors, cameraForWorldDirectionAtScreen, pickNearestSample, screenToWorldDirection } from './worldgenL8GlobeRenderer.js';
import { WORLDGEN_BOUNDARY_CONVERGENT, WORLDGEN_BOUNDARY_DIVERGENT, WORLDGEN_BOUNDARY_TRANSFORM, WORLDGEN_BEDROCK_ACCRETED_TERRANE, WORLDGEN_BEDROCK_ARC_VOLCANIC, WORLDGEN_BEDROCK_CARBONATE_PLATFORM, WORLDGEN_BEDROCK_CLASTIC_SEDIMENTARY, WORLDGEN_BEDROCK_CRYSTALLINE_BASEMENT, WORLDGEN_BEDROCK_OCEANIC_BASALT, WORLDGEN_BEDROCK_OCEANIC_SEDIMENT, WORLDGEN_BEDROCK_OROGENIC_METAMORPHIC, WORLDGEN_BEDROCK_RIFT_VOLCANIC, WORLDGEN_CRUST_CONTINENTAL, WORLDGEN_CRUST_OCEANIC, WORLDGEN_CRUST_TRANSITIONAL, WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION, WORLDGEN_GEOLOGY_CONTINENTAL_RIFT, WORLDGEN_GEOLOGY_OCEANIC_RIDGE, WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION, WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION, WORLDGEN_GEOLOGY_TRANSFORM, WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE, WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN, WORLDGEN_STRUCTURE_NONE, WORLDGEN_STRUCTURE_RIFT, WORLDGEN_STRUCTURE_SUTURE, WORLDGEN_STRUCTURE_TRANSFORM, WORLDGEN_INVALID_SAMPLE_ID, WORLDGEN_CLIMATE_COARSE_MAX_LEVEL, WORLDGEN_CLIMATE_FINE_MAX_LEVEL, WORLDGEN_PROTOCOL_VERSION, } from '../protocol.js';
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
function historicalIdentityColor(id, offset = 42) { return `hsl(${(id * 137.507764 + offset) % 360} 62% 55%)`; }
function historicalEventColor(kind) {
    if (kind === 1)
        return '#f59e42';
    if (kind === 2)
        return '#50b9e8';
    if (kind === 3)
        return '#7656d6';
    if (kind === 4)
        return '#e94f4f';
    if (kind === 5)
        return '#e8d35a';
    if (kind === 6)
        return '#5dd18b';
    if (kind === 7)
        return '#c178df';
    return '#101923';
}
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
function bedrockColor(kind) {
    if (kind === WORLDGEN_BEDROCK_OCEANIC_BASALT)
        return '#355f7c';
    if (kind === WORLDGEN_BEDROCK_OCEANIC_SEDIMENT)
        return '#768896';
    if (kind === WORLDGEN_BEDROCK_CRYSTALLINE_BASEMENT)
        return '#9c765d';
    if (kind === WORLDGEN_BEDROCK_OROGENIC_METAMORPHIC)
        return '#7b657d';
    if (kind === WORLDGEN_BEDROCK_ARC_VOLCANIC)
        return '#a94c3d';
    if (kind === WORLDGEN_BEDROCK_RIFT_VOLCANIC)
        return '#b97842';
    if (kind === WORLDGEN_BEDROCK_CLASTIC_SEDIMENTARY)
        return '#c0a477';
    if (kind === WORLDGEN_BEDROCK_CARBONATE_PLATFORM)
        return '#ddd5a5';
    if (kind === WORLDGEN_BEDROCK_ACCRETED_TERRANE)
        return '#6f8b68';
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
const INFILL_MODES = new Set(['infill-solid-elevation', 'infill-fill-depth']);
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
const MAP_LAND_RAMP_STEPS_PER_INTERVAL = 8;
const LAND_RELIEF_SHADE_STEPS = 7;
const INITIAL_LAND_RAMP = [
    [0, [67, 112, 60]],
    [250, [93, 136, 73]],
    [750, [130, 161, 84]],
    [1_500, [157, 154, 95]],
    [2_250, [165, 138, 97]],
    [3_000, [148, 118, 87]],
    [3_750, [128, 101, 79]],
    [4_500, [116, 97, 90]],
    [5_500, [119, 113, 109]],
    [7_000, [133, 130, 127]],
    [8_500, [148, 145, 142]],
];
const EVOLVED_LAND_RAMP = [
    [0, [59, 102, 55]],
    [250, [84, 123, 67]],
    [750, [115, 144, 79]],
    [1_500, [144, 141, 88]],
    [2_250, [150, 127, 88]],
    [3_000, [133, 109, 80]],
    [3_750, [115, 93, 75]],
    [4_500, [105, 89, 81]],
    [5_500, [112, 105, 101]],
    [7_000, [126, 122, 119]],
    [8_500, [141, 138, 135]],
];
let reliefShadeCache = { result: null, initial: null, evolved: null };
function clampByte(value) { return Math.max(0, Math.min(255, Math.round(value))); }
function rgbHex(red, green, blue) {
    return `#${clampByte(red).toString(16).padStart(2, '0')}${clampByte(green).toString(16).padStart(2, '0')}${clampByte(blue).toString(16).padStart(2, '0')}`;
}
function interpolateLandRamp(stops, elevationM, bucketed) {
    if (elevationM <= stops[0][0])
        return stops[0][1];
    const last = stops[stops.length - 1];
    if (elevationM >= last[0])
        return last[1];
    for (let index = 1; index < stops.length; index += 1) {
        const upper = stops[index];
        if (elevationM > upper[0])
            continue;
        const lower = stops[index - 1];
        let t = (elevationM - lower[0]) / Math.max(1e-9, upper[0] - lower[0]);
        if (bucketed)
            t = Math.round(t * MAP_LAND_RAMP_STEPS_PER_INTERVAL) / MAP_LAND_RAMP_STEPS_PER_INTERVAL;
        return [
            lower[1][0] + (upper[1][0] - lower[1][0]) * t,
            lower[1][1] + (upper[1][1] - lower[1][1]) * t,
            lower[1][2] + (upper[1][2] - lower[1][2]) * t,
        ];
    }
    return last[1];
}
function landElevationM(result, sample, evolved) {
    return evolved ? result.postInfillSolidElevationM[sample] - result.metrics.seaLevelM : result.elevationAboveSeaLevelM[sample];
}
function buildLandReliefShade(result, evolved) {
    const count = result.metrics.fineSampleCount;
    const shades = new Float32Array(count);
    shades.fill(1);
    const positions = result.positions;
    const radiusM = Math.max(1, result.planet.radiusM);
    const radialLight = 0.82;
    const tangentLight = 0.57;
    const lightNorm = Math.hypot(radialLight, tangentLight);
    const flatLambert = radialLight / lightNorm;
    for (let sample = 0; sample < count; sample += 1) {
        if (result.submergedMask[sample])
            continue;
        const offset = sample * 3;
        const px = positions[offset];
        const py = positions[offset + 1];
        const pz = positions[offset + 2];
        const elevation = landElevationM(result, sample, evolved);
        let gx = 0;
        let gy = 0;
        let gz = 0;
        let neighborCount = 0;
        for (let cursor = result.neighborOffsets[sample]; cursor < result.neighborOffsets[sample + 1]; cursor += 1) {
            const neighbor = result.neighbors[cursor];
            if (result.submergedMask[neighbor])
                continue;
            const neighborOffset = neighbor * 3;
            const qx = positions[neighborOffset];
            const qy = positions[neighborOffset + 1];
            const qz = positions[neighborOffset + 2];
            const dot = Math.max(-1, Math.min(1, px * qx + py * qy + pz * qz));
            const tx = qx - px * dot;
            const ty = qy - py * dot;
            const tz = qz - pz * dot;
            const sinArc = Math.hypot(tx, ty, tz);
            if (sinArc < 1e-12)
                continue;
            const arc = Math.atan2(sinArc, dot);
            const runM = radiusM * arc;
            if (runM <= 0)
                continue;
            const slope = (landElevationM(result, neighbor, evolved) - elevation) / runM;
            gx += tx / sinArc * slope;
            gy += ty / sinArc * slope;
            gz += tz / sinArc * slope;
            neighborCount += 1;
        }
        if (neighborCount === 0)
            continue;
        const gradientScale = 2 / neighborCount;
        gx *= gradientScale;
        gy *= gradientScale;
        gz *= gradientScale;
        let northX = -px * pz;
        let northY = -py * pz;
        let northZ = 1 - pz * pz;
        let northNorm = Math.hypot(northX, northY, northZ);
        if (northNorm < 1e-8) {
            northX = 1 - px * px;
            northY = -px * py;
            northZ = -px * pz;
            northNorm = Math.hypot(northX, northY, northZ);
        }
        if (northNorm < 1e-12)
            continue;
        northX /= northNorm;
        northY /= northNorm;
        northZ /= northNorm;
        let eastX = northY * pz - northZ * py;
        let eastY = northZ * px - northX * pz;
        let eastZ = northX * py - northY * px;
        const eastNorm = Math.hypot(eastX, eastY, eastZ);
        if (eastNorm < 1e-12)
            continue;
        eastX /= eastNorm;
        eastY /= eastNorm;
        eastZ /= eastNorm;
        const northwestX = (northX - eastX) * Math.SQRT1_2;
        const northwestY = (northY - eastY) * Math.SQRT1_2;
        const northwestZ = (northZ - eastZ) * Math.SQRT1_2;
        const lightX = (radialLight * px + tangentLight * northwestX) / lightNorm;
        const lightY = (radialLight * py + tangentLight * northwestY) / lightNorm;
        const lightZ = (radialLight * pz + tangentLight * northwestZ) / lightNorm;
        let normalX = px - gx;
        let normalY = py - gy;
        let normalZ = pz - gz;
        const normalNorm = Math.hypot(normalX, normalY, normalZ);
        if (normalNorm < 1e-12)
            continue;
        normalX /= normalNorm;
        normalY /= normalNorm;
        normalZ /= normalNorm;
        const lambert = normalX * lightX + normalY * lightY + normalZ * lightZ;
        const relief = Math.max(-0.12, Math.min(0.12, (lambert - flatLambert) * 1.65));
        shades[sample] = 1 + relief;
    }
    return shades;
}
function reliefShade(result, sample, evolved) {
    if (reliefShadeCache.result !== result)
        reliefShadeCache = { result, initial: null, evolved: null };
    if (evolved) {
        if (!reliefShadeCache.evolved)
            reliefShadeCache.evolved = buildLandReliefShade(result, true);
        return reliefShadeCache.evolved[sample];
    }
    if (!reliefShadeCache.initial)
        reliefShadeCache.initial = buildLandReliefShade(result, false);
    return reliefShadeCache.initial[sample];
}
function shadedLandColor(stops, elevationM, shade, bucketed) {
    const color = interpolateLandRamp(stops, elevationM, bucketed);
    let appliedShade = shade;
    if (bucketed) {
        const t = Math.max(0, Math.min(1, (shade - 0.88) / 0.24));
        appliedShade = 0.88 + Math.round(t * (LAND_RELIEF_SHADE_STEPS - 1)) / (LAND_RELIEF_SHADE_STEPS - 1) * 0.24;
    }
    return rgbHex(color[0] * appliedShade, color[1] * appliedShade, color[2] * appliedShade);
}
function hypsometricColor(result, sample, bucketed = false) {
    if (result.submergedMask[sample]) {
        const depth = result.waterDepthM[sample];
        if (depth <= 25)
            return '#b7e5e6';
        if (depth <= 50)
            return '#afe1e4';
        if (depth <= 75)
            return '#a4dce1';
        if (depth <= 100)
            return '#99d5df';
        if (depth <= 150)
            return '#87c9d8';
        if (depth <= 250)
            return '#76bfd3';
        if (depth <= 350)
            return '#69b7cf';
        if (depth <= 500)
            return '#5fa9c7';
        if (depth <= 700)
            return '#589fbd';
        if (depth <= 900)
            return '#5196b4';
        if (depth <= 1_200)
            return '#4b8dab';
        if (depth <= 1_500)
            return '#4585a3';
        if (depth <= 1_800)
            return '#407e9c';
        if (depth <= 2_200)
            return '#3b7795';
        if (depth <= 2_600)
            return '#37718f';
        if (depth <= 3_000)
            return '#336b89';
        if (depth <= 3_500)
            return '#306683';
        if (depth <= 4_000)
            return '#2d617e';
        if (depth <= 4_500)
            return '#2a5d79';
        if (depth <= 5_250)
            return '#285a76';
        if (depth <= 6_000)
            return '#265773';
        if (depth <= 6_750)
            return '#245570';
        if (depth <= 7_500)
            return '#22536e';
        return '#20516c';
    }
    const elevation = result.elevationAboveSeaLevelM[sample];
    return shadedLandColor(INITIAL_LAND_RAMP, elevation, reliefShade(result, sample, false), bucketed);
}
function evolvedHypsometricColor(result, sample, bucketed = false) {
    if (result.submergedMask[sample])
        return hypsometricColor(result, sample, bucketed);
    const elevation = result.postInfillSolidElevationM[sample] - result.metrics.seaLevelM;
    return shadedLandColor(EVOLVED_LAND_RAMP, elevation, reliefShade(result, sample, true), bucketed);
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
        case 'historical-crust-birth-age': return { values: result.crustBirthAgeMyr, minimum: 0, maximum: 3_500, lowHue: 205, highHue: 24 };
        case 'rock-strength': return { values: result.rockStrengthIndex, minimum: 0, maximum: 1, lowHue: 95, highHue: 355 };
        case 'lithology-erodibility': return { values: result.lithologyErodibilityIndex, minimum: 0, maximum: 1, lowHue: 160, highHue: 5 };
        case 'permeability': return { values: result.permeabilityIndex, minimum: 0, maximum: 1, lowHue: 35, highHue: 205 };
        case 'weathering-susceptibility': return { values: result.weatheringSusceptibility, minimum: 0, maximum: 1, lowHue: 55, highHue: 300 };
        case 'fines-fraction': return { values: result.finesFraction, minimum: 0, maximum: 1, lowHue: 90, highHue: 25 };
        case 'carbonate-fraction': return { values: result.carbonateFraction, minimum: 0, maximum: 1, lowHue: 210, highHue: 48 };
        case 'historical-event-age': return { values: result.latestHistoricalEventAgeMyr, minimum: 0, maximum: 350, lowHue: 205, highHue: 24 };
        case 'historical-rift': return { values: result.historicalRiftIntensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 25 };
        case 'historical-rift-age': return { values: result.historicalRiftAgeMyr, minimum: 0, maximum: 350, lowHue: 205, highHue: 24 };
        case 'historical-shear': return { values: result.historicalShearIntensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 55 };
        case 'historical-suture': return { values: result.historicalSutureIntensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 350 };
        case 'historical-suture-age': return { values: result.historicalSutureAgeMyr, minimum: 0, maximum: 350, lowHue: 205, highHue: 24 };
        case 'historical-passive-margin': return { values: result.passiveMarginIndex, minimum: 0, maximum: 1, lowHue: 215, highHue: 155 };
        case 'historical-active-orogen': return { values: result.activeOrogenIntensity, minimum: 0, maximum: 1, lowHue: 215, highHue: 15 };
        case 'historical-fossil-orogen': return { values: result.fossilOrogenIntensity, minimum: 0, maximum: 1, lowHue: 215, highHue: 285 };
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
        case 'infill-solid-elevation': return { values: result.postInfillSolidElevationM, minimum: -12_000, maximum: 8_000, lowHue: 225, highHue: 25 };
        case 'infill-fill-depth': return { values: result.lakeFillDepthM, minimum: 0, maximum: Math.max(0.01, result.infillMetrics.maximumFillDepthM), lowHue: 205, highHue: 35 };
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
function sampleColor(result, mode, sample, field, bucketed = false) {
    if (mode === 'physical-world')
        return evolvedHypsometricColor(result, sample, bucketed);
    if (mode === 'physical-elevation' || mode === 'winds' || mode === 'currents')
        return hypsometricColor(result, sample, bucketed);
    if (mode === 'land-water')
        return result.submergedMask[sample] ? '#214d7a' : '#a99b72';
    if (mode === 'historical-origin')
        return historicalIdentityColor(result.originPlateIds[sample], 42);
    if (mode === 'historical-fragments')
        return historicalIdentityColor(result.historicalFragmentIds[sample], 104);
    if (mode === 'historical-current')
        return historicalIdentityColor(result.currentPlateIds[sample], 18);
    if (mode === 'historical-provenance')
        return historicalIdentityColor(result.crustProvinceId[sample] & 0x7fff, 154);
    if (mode === 'historical-event')
        return historicalEventColor(result.latestHistoricalEventKind[sample]);
    if (mode === 'plates' || mode === 'tectonic-boundaries' || mode === 'geological-boundaries' || mode === 'boundary-provenance')
        return plateColor(result.plateIds[sample]);
    if (mode === 'kinematic-domains')
        return plateColor(result.kinematicDomainIds[sample]);
    if (mode === 'crust-type')
        return crustColor(result.crustKind[sample]);
    if (mode === 'bedrock-class')
        return bedrockColor(result.bedrockClass[sample]);
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
function projectSamples(result, projection, yaw, pitch, width, height, buffers, zoom = 1) {
    const count = result.metrics.fineSampleCount;
    const positions = result.positions;
    if (projection === 'map') {
        const safeZoom = Math.max(1, zoom);
        const longitudeSpan = TWO_PI / safeZoom;
        const latitudeSpan = Math.PI / safeZoom;
        for (let sample = 0; sample < count; sample += 1) {
            const offset = sample * 3;
            const px = positions[offset];
            const py = positions[offset + 1];
            const pz = positions[offset + 2];
            const longitude = Math.atan2(py, px);
            const latitude = Math.asin(Math.max(-1, Math.min(1, pz)));
            const longitudeDelta = wrapLongitudeRad(longitude - yaw);
            const latitudeDelta = latitude - pitch;
            const x = width / 2 + longitudeDelta / longitudeSpan * width;
            const y = height / 2 - latitudeDelta / latitudeSpan * height;
            buffers.x[sample] = x;
            buffers.y[sample] = y;
            buffers.visible[sample] = Math.abs(longitudeDelta) <= longitudeSpan / 2 + 1e-9
                && Math.abs(latitudeDelta) <= latitudeSpan / 2 + 1e-9
                && x >= -2 && x <= width + 2 && y >= -2 && y <= height + 2 ? 1 : 0;
        }
        return;
    }
    const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch);
    const radius = Math.min(width, height) * 0.44 * zoom;
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
function screenTangentDelta(position, eastValue, northValue, projection, yaw, pitch, width, height, zoom = 1) {
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
        return mapVectorDelta(eastValue, northValue, lat, width * Math.max(1, zoom), height * Math.max(1, zoom));
    const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch);
    const x1 = cy * tangent[0] - sy * tangent[1];
    const y1 = sy * tangent[0] + cy * tangent[1];
    const rotatedZ = -sp * x1 + cp * tangent[2];
    const radius = Math.min(width, height) * 0.44 * zoom;
    return [y1 * radius * 0.055, -rotatedZ * radius * 0.055];
}
let styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
function buildStyleCache(result, mode, phase) {
    const field = scalarField(result, mode, phase);
    const phaseKey = ['seasonal-temperature', 'seasonal-sst', 'seasonal-precipitation', 'seasonal-realized-discharge', 'seasonal-snow-storage'].includes(mode) ? phase.toFixed(3) : 'mean';
    const key = `${mode}:${phaseKey}`;
    const sampleBuckets = mode === 'mesh' ? [] : bucketize(result.metrics.fineSampleCount, sample => sampleColor(result, mode, sample, field, true));
    let boundaryBuckets = [];
    if (mode === 'tectonic-boundaries')
        boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => tectonicBoundaryColor(result.boundaryKinds[boundary]));
    else if (mode === 'geological-boundaries')
        boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => geologicalBoundaryColor(result.geologicalBoundaryRegimes[boundary]));
    else if (mode === 'boundary-provenance')
        boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => provenanceColor(result.boundaryCoarseSourceIndices[boundary]));
    return { result, key, sampleBuckets, boundaryBuckets };
}
function drawVectors(context, result, mode, phase, projection, yaw, pitch, width, height, buffers, animation, zoom = 1) {
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
        let [dx, dy] = screenTangentDelta(position, east, north, projection, yaw, pitch, width, height, zoom);
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
            const evolvedA = result.postInfillSolidElevationM[a] - result.metrics.seaLevelM;
            const evolvedB = result.postInfillSolidElevationM[b] - result.metrics.seaLevelM;
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
function drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation, zoom = 1) {
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
        drawVectors(context, result, 'winds', phase, projection, yaw, pitch, width, height, buffers, animation, zoom);
    if (overlays.has('currents'))
        drawVectors(context, result, 'currents', phase, projection, yaw, pitch, width, height, buffers, animation, zoom);
}
let gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 };
const mapRasterCanvas = document.createElement('canvas');
let mapRasterCache = {
    result: null,
    lookupKey: '',
    styleKey: '',
    width: 0,
    height: 0,
    sampleIds: new Uint32Array(0),
};
function nearestMapSampleFromSeed(result, directionX, directionY, directionZ, seedSample) {
    let bestSample = Math.max(0, Math.min(result.metrics.fineSampleCount - 1, seedSample));
    let offset = bestSample * 3;
    let bestDot = result.positions[offset] * directionX
        + result.positions[offset + 1] * directionY
        + result.positions[offset + 2] * directionZ;
    while (true) {
        let improved = false;
        const start = result.neighborOffsets[bestSample];
        const end = result.neighborOffsets[bestSample + 1];
        for (let cursor = start; cursor < end; cursor += 1) {
            const neighbor = result.neighbors[cursor];
            offset = neighbor * 3;
            const dot = result.positions[offset] * directionX
                + result.positions[offset + 1] * directionY
                + result.positions[offset + 2] * directionZ;
            if (dot > bestDot + 1e-12) {
                bestDot = dot;
                bestSample = neighbor;
                improved = true;
            }
        }
        if (!improved)
            return bestSample;
    }
}
function ensureMapSampleLookup(result, centerLongitudeRad, centerLatitudeRad, zoom, width, height, interactive) {
    const rasterScale = interactive && result.metrics.fineSampleCount > 100_000 ? 0.5 : 1;
    const rasterWidth = Math.max(1, Math.round(width * rasterScale));
    const rasterHeight = Math.max(1, Math.round(height * rasterScale));
    const lookupKey = `${centerLongitudeRad.toFixed(7)}:${centerLatitudeRad.toFixed(7)}:${zoom.toFixed(5)}:${rasterWidth}:${rasterHeight}`;
    if (mapRasterCache.result === result && mapRasterCache.lookupKey === lookupKey)
        return mapRasterCache;
    const required = rasterWidth * rasterHeight;
    const sampleIds = mapRasterCache.sampleIds.length === required
        ? mapRasterCache.sampleIds
        : new Uint32Array(required);
    const cosLongitude = new Float64Array(rasterWidth);
    const sinLongitude = new Float64Array(rasterWidth);
    const longitudeSpan = TWO_PI / Math.max(1, zoom);
    const latitudeSpan = Math.PI / Math.max(1, zoom);
    for (let x = 0; x < rasterWidth; x += 1) {
        const longitude = centerLongitudeRad + ((x + 0.5) / rasterWidth - 0.5) * longitudeSpan;
        cosLongitude[x] = Math.cos(longitude);
        sinLongitude[x] = Math.sin(longitude);
    }
    let seedSample = 0;
    for (let y = 0; y < rasterHeight; y += 1) {
        const latitude = centerLatitudeRad + (0.5 - (y + 0.5) / rasterHeight) * latitudeSpan;
        const cosLatitude = Math.cos(latitude);
        const directionZ = Math.sin(latitude);
        if (y === 0) {
            const direction = [
                cosLatitude * cosLongitude[0],
                cosLatitude * sinLongitude[0],
                directionZ,
            ];
            seedSample = pickNearestSample(result, direction);
        }
        else {
            seedSample = sampleIds[(y - 1) * rasterWidth];
        }
        for (let x = 0; x < rasterWidth; x += 1) {
            seedSample = nearestMapSampleFromSeed(result, cosLatitude * cosLongitude[x], cosLatitude * sinLongitude[x], directionZ, seedSample);
            sampleIds[y * rasterWidth + x] = seedSample;
        }
    }
    mapRasterCache = {
        result,
        lookupKey,
        styleKey: '',
        width: rasterWidth,
        height: rasterHeight,
        sampleIds,
    };
    return mapRasterCache;
}
function renderEquirectangularRaster(context, result, mode, phase, centerLongitudeRad, centerLatitudeRad, zoom, width, height, interactive, selectedSample) {
    const lookup = ensureMapSampleLookup(result, centerLongitudeRad, centerLatitudeRad, zoom, width, height, interactive);
    const gpu = ensureGpuColorCache(result, mode, phase);
    const styleKey = `${lookup.lookupKey}:${gpu.key}:${gpu.alpha.toFixed(3)}:${selectedSample ?? -1}`;
    if (mapRasterCanvas.width !== lookup.width)
        mapRasterCanvas.width = lookup.width;
    if (mapRasterCanvas.height !== lookup.height)
        mapRasterCanvas.height = lookup.height;
    const rasterContext = mapRasterCanvas.getContext('2d', { alpha: false });
    if (!rasterContext)
        throw new Error('Planet Engine Lab could not acquire the equirectangular raster context.');
    if (lookup.styleKey !== styleKey) {
        const image = rasterContext.createImageData(lookup.width, lookup.height);
        const pixels = image.data;
        const alpha = Math.max(0, Math.min(1, gpu.alpha));
        const inverseAlpha = 1 - alpha;
        for (let pixel = 0; pixel < lookup.sampleIds.length; pixel += 1) {
            const sample = lookup.sampleIds[pixel];
            const colorOffset = sample * 4;
            let red = gpu.colors[colorOffset];
            let green = gpu.colors[colorOffset + 1];
            let blue = gpu.colors[colorOffset + 2];
            if (sample === selectedSample) {
                red = Math.round(red * 0.72 + 93 * 0.28);
                green = Math.round(green * 0.72 + 224 * 0.28);
                blue = Math.round(blue * 0.72 + 255 * 0.28);
            }
            const output = pixel * 4;
            pixels[output] = Math.round(red * alpha + 8 * inverseAlpha);
            pixels[output + 1] = Math.round(green * alpha + 16 * inverseAlpha);
            pixels[output + 2] = Math.round(blue * alpha + 26 * inverseAlpha);
            pixels[output + 3] = 255;
        }
        rasterContext.putImageData(image, 0, 0);
        lookup.styleKey = styleKey;
    }
    context.save();
    context.imageSmoothingEnabled = interactive && (lookup.width !== width || lookup.height !== height);
    context.drawImage(mapRasterCanvas, 0, 0, lookup.width, lookup.height, 0, 0, width, height);
    context.restore();
}
let projectedResult = null;
let projectedKey = '';
const GPU_SEASONAL_MODES = new Set(['seasonal-temperature', 'seasonal-sst', 'seasonal-precipitation', 'seasonal-realized-discharge', 'seasonal-snow-storage']);
function gpuStyleKey(mode, phase) {
    return `${mode}:${GPU_SEASONAL_MODES.has(mode) ? phase.toFixed(3) : 'mean'}`;
}
function ensureGpuColorCache(result, mode, phase) {
    const key = gpuStyleKey(mode, phase);
    if (gpuColorCache.result === result && gpuColorCache.key === key)
        return gpuColorCache;
    const field = scalarField(result, mode, phase);
    const sampler = (sample) => {
        if (mode === 'mesh')
            return '#24445f';
        if (isLakeMode(mode))
            return lakeSampleColor(result, mode, sample);
        if (isRunoffMode(mode))
            return runoffSampleColor(result, mode, sample);
        if (isDrainageMode(mode))
            return drainageSampleColor(result, mode, sample);
        return sampleColor(result, mode, sample, field);
    };
    const boundaryMode = mode === 'tectonic-boundaries' || mode === 'geological-boundaries' || mode === 'boundary-provenance';
    gpuColorCache = {
        result,
        key,
        colors: buildRgbaColors(result.metrics.fineSampleCount, sampler),
        alpha: boundaryMode ? 0.28 : mode === 'mesh' ? 0.34 : 0.94,
    };
    return gpuColorCache;
}
function ensureProjectedSamples(result, projection, yaw, pitch, width, height, buffers, zoom) {
    const key = `${projection}:${yaw.toFixed(6)}:${pitch.toFixed(6)}:${zoom.toFixed(4)}:${width}:${height}`;
    if (projectedResult === result && projectedKey === key)
        return;
    projectSamples(result, projection, yaw, pitch, width, height, buffers, zoom);
    projectedResult = result;
    projectedKey = key;
}
function renderPlanet(surfaceCanvas, canvas, result, projection, mode, overlays, phase, yaw, pitch, zoom, buffers, interactive, animation, selectedSample) {
    const width = 1100;
    const height = projection === 'map' ? 550 : 760;
    if (canvas.width !== width)
        canvas.width = width;
    if (canvas.height !== height)
        canvas.height = height;
    const context = canvas.getContext('2d');
    if (!context)
        throw new Error('Planet Engine Lab could not acquire a 2D canvas context.');
    if (projection === 'globe') {
        const gpu = ensureGpuColorCache(result, mode, phase);
        surfaceCanvas.hidden = false;
        surfaceCanvas.style.display = '';
        const gpuDrawn = globeRenderer.draw(result, gpu.colors, gpu.key, { yaw, pitch, zoom }, width, height, gpu.alpha, overlays.has('cell-boundaries'), selectedSample);
        if (gpuDrawn) {
            context.clearRect(0, 0, width, height);
            const boundaryMode = mode === 'tectonic-boundaries' || mode === 'geological-boundaries' || mode === 'boundary-provenance';
            const cpuOverlayCount = overlays.size - (overlays.has('cell-boundaries') ? 1 : 0);
            const needsProjectedDecoration = cpuOverlayCount > 0
                || mode === 'mesh'
                || mode === 'winds'
                || mode === 'currents'
                || isDrainageMode(mode)
                || boundaryMode;
            if (needsProjectedDecoration)
                ensureProjectedSamples(result, projection, yaw, pitch, width, height, buffers, zoom);
            const globeRadius = Math.min(width, height) * 0.44 * zoom;
            context.beginPath();
            context.arc(width / 2, height / 2, globeRadius, 0, TWO_PI);
            context.strokeStyle = '#5d7890';
            context.lineWidth = 1;
            context.stroke();
            if (mode === 'mesh') {
                context.beginPath();
                context.strokeStyle = '#5d7890';
                context.lineWidth = 0.55;
                for (let sample = 0; sample < result.metrics.fineSampleCount; sample += 1) {
                    if (!buffers.visible[sample])
                        continue;
                    const ax = buffers.x[sample], ay = buffers.y[sample];
                    if (ax < -4 || ax > width + 4 || ay < -4 || ay > height + 4)
                        continue;
                    for (let cursor = result.neighborOffsets[sample]; cursor < result.neighborOffsets[sample + 1]; cursor += 1) {
                        const neighbor = result.neighbors[cursor];
                        if (neighbor <= sample || !buffers.visible[neighbor])
                            continue;
                        context.moveTo(ax, ay);
                        context.lineTo(buffers.x[neighbor], buffers.y[neighbor]);
                    }
                }
                context.stroke();
            }
            else {
                if (isDrainageMode(mode)) {
                    if (mode === 'flow-direction')
                        drawDrainageReceiverOverlay(context, result, projection, width, buffers);
                    if (mode === 'basins')
                        drawDrainageOutlets(context, result, buffers);
                }
                const cacheKey = `${mode}:${GPU_SEASONAL_MODES.has(mode) ? phase.toFixed(3) : 'mean'}`;
                if (styleCache.result !== result || styleCache.key !== cacheKey)
                    styleCache = buildStyleCache(result, mode, phase);
                if (styleCache.boundaryBuckets.length > 0) {
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
                            context.moveTo(buffers.x[a], buffers.y[a]);
                            context.lineTo(buffers.x[b], buffers.y[b]);
                        }
                        context.stroke();
                    }
                }
                if (mode === 'winds' || mode === 'currents')
                    drawVectors(context, result, mode, phase, projection, yaw, pitch, width, height, buffers, animation, zoom);
            }
            if (cpuOverlayCount > 0)
                drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation, zoom);
            return;
        }
    }
    if (projection === 'map') {
        surfaceCanvas.hidden = true;
        surfaceCanvas.style.display = 'none';
        context.fillStyle = '#08101a';
        context.fillRect(0, 0, width, height);
        renderEquirectangularRaster(context, result, mode, phase, yaw, pitch, zoom, width, height, interactive, selectedSample);
        const boundaryMode = mode === 'tectonic-boundaries' || mode === 'geological-boundaries' || mode === 'boundary-provenance';
        const needsProjectedDecoration = overlays.size > 0
            || mode === 'mesh'
            || mode === 'winds'
            || mode === 'currents'
            || isDrainageMode(mode)
            || boundaryMode;
        if (!needsProjectedDecoration)
            return;
        ensureProjectedSamples(result, projection, yaw, pitch, width, height, buffers, zoom);
        if (mode === 'mesh') {
            context.beginPath();
            context.strokeStyle = '#5d7890';
            context.lineWidth = 0.55;
            for (let sample = 0; sample < result.metrics.fineSampleCount; sample += 1) {
                if (!buffers.visible[sample])
                    continue;
                const ax = buffers.x[sample], ay = buffers.y[sample];
                for (let cursor = result.neighborOffsets[sample]; cursor < result.neighborOffsets[sample + 1]; cursor += 1) {
                    const neighbor = result.neighbors[cursor];
                    if (neighbor <= sample || !buffers.visible[neighbor])
                        continue;
                    const bx = buffers.x[neighbor];
                    if (Math.abs(ax - bx) > width * 0.45)
                        continue;
                    context.moveTo(ax, ay);
                    context.lineTo(bx, buffers.y[neighbor]);
                }
            }
            context.stroke();
        }
        else {
            if (isDrainageMode(mode)) {
                if (mode === 'flow-direction')
                    drawDrainageReceiverOverlay(context, result, projection, width, buffers);
                if (mode === 'basins')
                    drawDrainageOutlets(context, result, buffers);
            }
            const cacheKey = `${mode}:${GPU_SEASONAL_MODES.has(mode) ? phase.toFixed(3) : 'mean'}`;
            if (styleCache.result !== result || styleCache.key !== cacheKey)
                styleCache = buildStyleCache(result, mode, phase);
            if (styleCache.boundaryBuckets.length > 0) {
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
                        if (Math.abs(ax - bx) > width * 0.45)
                            continue;
                        context.moveTo(ax, buffers.y[a]);
                        context.lineTo(bx, buffers.y[b]);
                    }
                    context.stroke();
                }
            }
            if (mode === 'winds' || mode === 'currents')
                drawVectors(context, result, mode, phase, projection, yaw, pitch, width, height, buffers, animation, zoom);
        }
        drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation, zoom);
        return;
    }
    surfaceCanvas.hidden = true;
    surfaceCanvas.style.display = 'none';
    context.fillStyle = '#08101a';
    context.fillRect(0, 0, width, height);
    ensureProjectedSamples(result, projection, yaw, pitch, width, height, buffers, projection === 'globe' ? zoom : 1);
    if (projection === 'globe') {
        context.beginPath();
        context.arc(width / 2, height / 2, Math.min(width, height) * 0.44 * zoom, 0, TWO_PI);
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
        drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation, projection === 'globe' ? zoom : 1);
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
        drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation, projection === 'globe' ? zoom : 1);
        return;
    }
    if (isDrainageMode(mode)) {
        renderDrainageDiagnostic(context, result, projection, mode, width, buffers, interactive);
        drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation, projection === 'globe' ? zoom : 1);
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
    const cacheKey = `${mode}:${GPU_SEASONAL_MODES.has(mode) ? phase.toFixed(3) : 'mean'}`;
    if (styleCache.result !== result || styleCache.key !== cacheKey)
        styleCache = buildStyleCache(result, mode, phase);
    const count = result.metrics.fineSampleCount;
    const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
    // Static L8 equirectangular maps can put hundreds of thousands of equal-valued
    // samples into one color bucket. Rasterize those buckets point-by-point instead
    // of constructing one browser-sized Canvas path, which can silently drop fills.
    const fastPoints = (projection === 'map' || interactive) && count > 20_000;
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
        drawVectors(context, result, mode, phase, projection, yaw, pitch, width, height, buffers, animation, projection === 'globe' ? zoom : 1);
    drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation, projection === 'globe' ? zoom : 1);
}
const seed = element('worldgen-seed');
const coarseLevel = element('worldgen-coarse-level');
const fineLevel = element('worldgen-level');
const plates = element('worldgen-plates');
const projection = element('worldgen-projection');
const preset = element('worldgen-preset');
const diagnosticCategory = element('worldgen-diagnostic-category');
const visualization = element('worldgen-visualization');
const diagnosticLegend = element('worldgen-diagnostic-legend');
const overlayLegends = element('worldgen-overlay-legends');
const season = element('worldgen-season');
const seasonValue = element('worldgen-season-value');
const zoomControl = element('worldgen-zoom');
const zoomValue = element('worldgen-zoom-value');
const resetCamera = element('worldgen-reset-camera');
const cellInspector = element('worldgen-cell-inspector');
const overlaySummary = element('worldgen-overlay-summary');
const overlayInputs = Array.from(document.querySelectorAll('input[data-worldgen-overlay]'));
const generate = element('worldgen-generate');
const copyCalibration = element('worldgen-copy-calibration');
const downloadCalibration = element('worldgen-download-calibration');
const copyCrashReport = element('worldgen-copy-crash-report');
const downloadCrashReport = element('worldgen-download-crash-report');
const receiveOnlyDebug = element('worldgen-debug-receive-only');
const debugSummary = element('worldgen-debug-summary');
const status = element('worldgen-status');
const generationProgress = element('worldgen-generation-progress');
const generationStage = element('worldgen-generation-stage');
const generationStep = element('worldgen-generation-step');
const generationTimer = element('worldgen-generation-timer');
const generationProfile = element('worldgen-generation-profile');
const metrics = element('worldgen-metrics');
const surfaceCanvas = element('worldgen-surface');
const canvas = element('worldgen-field');
const globeRenderer = new L8GlobeRenderer(surfaceCanvas);
const crashRecorder = createWorldgenCrashRecorder(WORLDGEN_PROTOCOL_VERSION);
const removeGlobalFailureCapture = installWorldgenGlobalFailureCapture(crashRecorder);
const client = createWorldgenClient({ onDiagnostic: diagnostic => {
        crashRecorder.record('client', diagnostic.event, { requestId: diagnostic.requestId, commandType: diagnostic.commandType, ...diagnostic.details });
        refreshCrashDebugSummary();
    } });
let current = null;
let currentCalibrationRequest = null;
let buffers = null;
let yaw = -0.65;
let pitch = 0.25;
let mapCenterLongitude = 0;
let mapCenterLatitude = 0;
let zoom = 1;
let selectedTile = null;
let drag = null;
let cameraSettleHandle = null;
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
    'lithology-substrate': 'Lithology / substrate',
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
    'lake-sediment-infill': 'Lake sediment infill / final hydrology',
    packaging: 'Packaging / transfer',
    'transport-ready': 'Worker result ready for transfer',
};
let generationStartedAt = 0;
let generationTimerHandle = null;
function refreshCrashDebugSummary() {
    const snapshot = crashRecorder.snapshot();
    copyCrashReport.disabled = !crashRecorder.hasAttempt();
    downloadCrashReport.disabled = !crashRecorder.hasAttempt();
    if (!crashRecorder.hasAttempt()) {
        debugSummary.textContent = 'No generation attempt recorded yet.';
        return;
    }
    const transport = snapshot.transport ? ` · packet ${snapshot.transport.totalMiB.toFixed(1)} MiB / ${snapshot.transport.arrayCount} arrays` : '';
    const failure = snapshot.failure ? ` · ${snapshot.failure.name}: ${snapshot.failure.message}` : '';
    debugSummary.textContent = `${snapshot.status} · last ${snapshot.lastCheckpoint ?? 'none'} · ${snapshot.events.length} events${transport}${failure}`;
}
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
    refreshOverlayLegends();
}
const DIAGNOSTIC_SUMMARIES = {
    "physical-world": "Shows the final post-infill physical surface: evolved land relief, the fixed WG-4 ocean mask, and bathymetric depth. Use it as the closest current view of the finished physical planet.",
    "physical-elevation": "Shows the accepted WG-4 elevation and bathymetry before WG-7 terrain evolution and lake-sediment infill. It is the baseline surface inherited by climate and the first hydrology solve.",
    "relative-elevation": "Maps solid-surface height relative to the solved sea level, making positive land relief and negative submerged terrain directly comparable.",
    "solid-elevation": "Maps absolute solid elevation relative to the model datum before subtracting sea level. This separates terrain construction from the hydrostatic ocean solution.",
    "land-water": "Shows the binary WG-4 land/ocean partition used by downstream climate and hydrology. It answers whether a sample is submerged, not what crust type lies beneath it.",
    "water-depth": "Maps solved ocean water depth over submerged samples, exposing shelves, slopes, abyssal basins, trenches, and the consequences of the fixed ocean volume.",
    "plates": "Shows modern macro-plate ownership on the final mesh. Colors identify present kinematic plates, not crust type or continent membership.",
    "kinematic-domains": "Shows the refined modern kinematic domains carried into the fine mesh, making coarse-to-fine plate ownership visible.",
    "historical-origin": "Shows each sample's ancestral plate of origin before later fragmentation, capture, and modern plate reorganization.",
    "historical-fragments": "Shows persistent crust fragments produced by the historical lithosphere model. Several fragments may occupy one modern plate.",
    "historical-current": "Shows current plate ownership as recorded by the historical material system, allowing inherited material identity to be compared with present kinematics.",
    "historical-provenance": "Shows the WG-3 crust-province identity inherited by each fine sample. Colors distinguish provenance blocks rather than physical magnitude.",
    "historical-crust-birth-age": "Maps the modeled formation age of inherited crustal material. Young values mark recently created material; large values mark old surviving crust.",
    "historical-event": "Shows the most recent recorded historical tectonic event affecting each material sample, such as rifting, spreading, collision, accretion, or subduction.",
    "historical-event-age": "Maps time since the latest recorded historical event. Small ages are geologically recent; large ages indicate older inherited events.",
    "historical-rift": "Shows accumulated historical rift influence. Higher values identify material more strongly affected by extension and rifting.",
    "historical-rift-age": "Maps the age of inherited rifting. Low values indicate recent extension; high values indicate old rift inheritance.",
    "historical-shear": "Shows inherited shear/transform influence. Higher values mark material with a stronger history of lateral tectonic deformation.",
    "historical-suture": "Shows inherited suture intensity from collision and assembly. Higher values mark stronger fossil joins between previously separate material blocks.",
    "historical-suture-age": "Maps the age of inherited sutures. Low values identify recent joins; high values identify older assembly boundaries.",
    "historical-passive-margin": "Shows how strongly each sample belongs to a passive-margin setting inherited from rifting and continental breakup.",
    "historical-active-orogen": "Shows present or young orogenic intensity derived from active tectonic shortening and mountain-building history.",
    "historical-fossil-orogen": "Shows inherited but no longer active orogenic structure. Higher values preserve stronger fossil mountain-belt ancestry.",
    "crust-type": "Classifies crust as continental, transitional, or oceanic. This is material identity, not a land/ocean mask: continental crust may be submerged.",
    "crust-age": "Maps the modeled age of current crustal material, especially useful for reading ocean-basin spreading patterns and old continental interiors.",
    "crust-thickness": "Maps modeled crustal thickness. Thick values generally mark continental or orogenic crust, while thin values generally mark oceanic lithosphere.",
    "orogeny-history": "Shows cumulative inherited mountain-building influence retained in the lithosphere. Higher values indicate stronger orogenic ancestry.",
    "ridge-history": "Shows cumulative inherited spreading-ridge influence. Higher values identify crust more strongly associated with ridge creation or spreading.",
    "trench-history": "Shows cumulative inherited trench/subduction influence. Higher values identify crust more strongly shaped by convergent-margin history.",
    "strength": "Shows the lithosphere-scale mechanical strength index inherited by topography and structural refinement.",
    "weakness": "Shows the lithospheric weakness field used to identify mechanically susceptible zones.",
    "dynamic-support": "Shows the signed mantle-dynamic support index. Positive values favor broad support or uplift; negative values favor broad downward support.",
    "structural-zones": "Classifies inherited structural zones such as sutures, rifts, transforms, and continental margins.",
    "fragmentation": "Shows the modeled propensity for lithosphere to fragment under tectonic history and inherited weakness. Higher values indicate greater susceptibility.",
    "bedrock-class": "Shows the persistent WG-4.5 bedrock class derived from crust type, tectonic history, structure, and fragment provenance.",
    "rock-strength": "Shows bedrock-scale mechanical resistance intended for later surface-process work. Higher values mean more resistant substrate.",
    "lithology-erodibility": "Shows the substrate's intrinsic tendency to be eroded. Higher values mean the material is easier to remove under otherwise similar forcing.",
    "permeability": "Shows the relative ability of the substrate to transmit water through the material. Higher values represent more permeable substrate.",
    "weathering-susceptibility": "Shows how readily bedrock is expected to weather under suitable climate forcing. Higher values indicate more weathering-prone material.",
    "fines-fraction": "Shows the relative fine-grained fraction of substrate available to later sediment, regolith, and soil processes.",
    "carbonate-fraction": "Shows the modeled carbonate content of the substrate. High values identify carbonate-rich platform or sedimentary material.",
    "isostatic": "Shows the WG-4 isostatic elevation contribution from crustal buoyancy and compensation before all terrain terms are summed.",
    "thermal": "Shows oceanic thermal-subsidence relief. More negative values correspond to stronger cooling-related subsidence of oceanic lithosphere.",
    "orogenic-relief": "Shows elevation added by collision and orogenic processes. Larger positive values mark stronger tectonic mountain support.",
    "ridge-relief": "Shows elevation added around spreading ridges. Larger positive values identify stronger ridge-related topographic support.",
    "rift-basin": "Shows signed relief associated with rifting and basin formation. More negative values indicate stronger subsidence.",
    "trench-relief": "Shows trench-related topographic depression generated at subduction systems. More negative values indicate deeper trench forcing.",
    "arc-relief": "Shows positive volcanic-arc relief generated by subduction-related magmatic systems.",
    "mantle-relief": "Shows the signed elevation contribution from broad mantle-dynamic support after conversion from the lithospheric support index.",
    "temperature": "Maps annual-mean near-surface temperature retained by WG-5 after climate spin-up.",
    "seasonal-temperature": "Reconstructs near-surface temperature at the selected orbital phase from the stored annual harmonic; the season slider changes the view without rerunning climate.",
    "temperature-range": "Maps the difference between modeled annual maximum and minimum temperature, highlighting climates with strong seasonal thermal swings.",
    "annual-insolation": "Maps annual-mean incoming stellar energy at the top-of-atmosphere forcing used by the climate model.",
    "seasonal-insolation": "Maps the amplitude of the annual insolation cycle. Higher values indicate stronger seasonal variation in received stellar energy.",
    "sst": "Maps annual-mean sea-surface temperature over the accepted ocean surface.",
    "seasonal-sst": "Reconstructs sea-surface temperature at the selected orbital phase from the stored seasonal harmonic.",
    "wind-speed": "Maps annual-mean horizontal wind-speed magnitude, independent of direction.",
    "surface-pressure": "Maps local surface atmospheric pressure after elevation and climate-state effects.",
    "current-speed": "Maps annual-mean surface-ocean current-speed magnitude over the ocean.",
    "ocean-heat": "Shows the signed ocean heat-transport index used to diagnose where surface circulation exports or imports heat relative to the local mean.",
    "humidity": "Maps annual-mean atmospheric specific humidity: the mass fraction of water vapor in air.",
    "precipitation": "Maps annual precipitation delivered by WG-5. It shows climate water input before runoff, lake storage, or river routing.",
    "seasonal-precipitation": "Shows precipitation rate at the selected orbital phase using the retained phase climatology.",
    "precip-seasonality": "Shows how strongly precipitation is concentrated into part of the year rather than distributed evenly.",
    "potential-evaporation": "Maps the atmosphere's annual potential evaporative demand before water availability limits actual evapotranspiration.",
    "moisture-balance": "Shows precipitation minus potential evaporation. Positive values indicate climatic moisture surplus; negative values indicate climatic deficit.",
    "aridity": "Shows the dimensionless aridity diagnostic used by WG-5 to distinguish humid from water-limited climates.",
    "snowfall": "Shows the fraction of precipitation expected to fall as snow under the modeled temperature regime.",
    "persistent-snow": "Shows modeled potential for snow to persist through the annual cycle on land.",
    "sea-ice": "Shows modeled potential for persistent or recurrent sea ice over ocean samples.",
    "contributing-area": "Maps the upstream land area draining through each sample in the canonical drainage graph. Large values identify major trunk channels and basin outlets.",
    "basins": "Colors each drainage basin by basin ID and marks basin outlets. Color is categorical and exists to separate neighboring catchments.",
    "flow-direction": "Colors drainage basins and overlays a sampled set of receiver links, showing downstream routing direction in the drainage graph.",
    "depression-depth": "Maps the vertical depth of closed topographic depressions relative to their spill elevation.",
    "depressions": "Colors each identified closed depression by depression ID, separating distinct potential lake or storage basins.",
    "escape-elevation": "Maps the minimum hydrologic escape elevation a sample must reach along its drainage path to leave local containment.",
    "potential-discharge": "Maps annual discharge implied by runoff production and drainage accumulation before equilibrium lake storage and overflow alter realized flow.",
    "annual-runoff": "Maps locally generated annual runoff depth after precipitation and actual evapotranspiration are reconciled.",
    "runoff-fraction": "Shows the fraction of local precipitation that becomes runoff rather than actual evapotranspiration.",
    "actual-et": "Maps annual actual evapotranspiration after water availability limits atmospheric evaporative demand.",
    "realized-discharge": "Maps annual river discharge after equilibrium lakes retain water or release solved overflow.",
    "lake-depth": "Maps equilibrium lake-water depth for land samples occupied by solved lakes.",
    "lake-state": "Classifies solved lake state as absent, endorheic, overflowing, or terminal storage.",
    "lake-fraction": "Shows the fraction of a sample's surface occupied by the solved equilibrium lake.",
    "seasonal-realized-discharge": "Maps realized river discharge at the selected orbital phase after seasonal runoff timing, snowmelt, routing, and lake storage.",
    "seasonal-flow-presence": "Shows the fraction of orbital phases in which realized flow is present at each land sample.",
    "seasonal-flow-regime": "Classifies final flow as dry, intermittent, or perennial from modeled seasonal presence.",
    "seasonal-snow-storage": "Maps snow-water-equivalent storage at the selected orbital phase after snowfall accumulation and degree-day melt.",
    "reconciliation-lake-depth-delta": "Shows the change in lake depth caused by rebuilding hydrology on the WG-7B evolved terrain.",
    "reconciliation-lake-change": "Marks samples where lake state changed when hydrology was reconciled to evolved terrain.",
    "reconciliation-realized-discharge-delta": "Shows the signed change in annual realized discharge after post-erosion hydrology reconciliation.",
    "reconciliation-flow-presence-delta": "Shows the signed change in seasonal flow-presence fraction after terrain evolution and hydrology reconciliation.",
    "reconciliation-flow-regime-change": "Marks samples whose dry, intermittent, or perennial flow regime changed after post-erosion reconciliation.",
    "erosion-effective-discharge": "Maps discharge actually used by the WG-7A erosion diagnostic after seasonal and lake constraints are applied.",
    "erosion-channel-slope": "Maps the downstream channel slope used by the fluvial erosion calculation.",
    "erosion-channel-width": "Maps the hydraulic channel-width estimate used to distribute erosive forcing.",
    "erosion-erodibility": "Shows the inherited WG-7A erodibility field used by the current erosion model before the planned lithology-aware retrofit.",
    "erosion-incision-potential": "Maps the bounded stream-power incision rate that WG-7A predicts before terrain mutation.",
    "erosion-sediment-supply": "Maps local sediment mass generated by erosion before routing downstream.",
    "erosion-sediment-load": "Maps sediment mass flux being routed through the drainage network.",
    "erosion-sediment-deposition": "Maps sediment mass deposited locally by the conservative sediment-routing diagnostic.",
    "evolution-solid-elevation": "Shows the WG-7B evolved solid surface after bounded erosion and deposition have been applied.",
    "evolution-terrain-delta": "Shows signed WG-7B terrain change relative to the accepted WG-4 surface. Negative values are erosion; positive values are deposition.",
    "evolution-applied-erosion": "Maps the depth of erosion actually applied by bounded terrain evolution.",
    "evolution-applied-deposition": "Maps the depth of land deposition actually applied by bounded terrain evolution.",
    "evolution-receiver-change": "Marks land samples whose drainage receiver changed after terrain evolution rebuilt the drainage graph.",
    "evolution-contributing-area": "Maps contributing drainage area after WG-7B terrain evolution and drainage reconstruction.",
    "evolution-potential-discharge": "Maps potential annual discharge on the post-erosion drainage network.",
    "infill-solid-elevation": "Shows the final post-WG-7D solid surface after bounded lake-sediment infill has modified eligible basin floors.",
    "infill-fill-depth": "Maps the depth of lake-sediment fill applied by WG-7D to historical lake depressions.",
    "inherited-mask": "Marks which fine samples are exact inherited coarse samples versus samples introduced by refinement.",
    "provenance": "Colors each fine sample by its nearest coarse source, exposing the spatial footprint of multiresolution inheritance.",
    "boundary-provenance": "Colors fine tectonic boundary segments by the coarse boundary or source provenance that produced them.",
    "mesh": "Shows the fine physical-topology neighbor mesh used by the L8 planet surface. It is a structural diagnostic rather than a physical field."
};
const DIAGNOSTIC_SCALE_HELP = {
    'physical-world': 'Blue bands encode water depth; land uses a hypsometric elevation ramp plus bounded hillshade. The colors are descriptive terrain bands rather than a single linear scalar ramp.',
    'physical-elevation': 'Blue bands encode water depth and land colors encode WG-4 elevation above sea level. Hillshade changes brightness but not the underlying elevation value.',
    'land-water': 'This is categorical: tan means land and blue means submerged. There is no numeric ordering between the two colors.',
    'plates': 'Colors are deterministic plate identifiers. Hue has no ordinal or physical magnitude meaning.',
    'kinematic-domains': 'Colors are deterministic domain identifiers. Hue has no ordinal or physical magnitude meaning.',
    'historical-origin': 'Colors are deterministic ancestral-plate identifiers. Hue has no ordinal meaning.',
    'historical-fragments': 'Colors are deterministic fragment identifiers. Hue has no ordinal meaning.',
    'historical-current': 'Colors are deterministic current-owner identifiers. Hue has no ordinal meaning.',
    'historical-provenance': 'Colors are deterministic provenance identifiers. Hue has no ordinal meaning.',
    'historical-event': 'Colors are event classes, not a magnitude scale.',
    'crust-type': 'Colors are continental, transitional, and oceanic crust classes; they are categorical rather than ordered.',
    'structural-zones': 'Colors identify structural-zone classes. They do not encode intensity.',
    'bedrock-class': 'Colors identify bedrock classes. They do not imply that one rock class is numerically greater than another.',
    'basins': 'Colors are basin IDs; adjacent colors only distinguish catchments. White points mark basin outlets.',
    'flow-direction': 'Colors are basin IDs; the light receiver segments show downstream direction. Basin hue is not a magnitude.',
    'depressions': 'Colors are depression IDs used to separate closed basins. Hue is categorical.',
    'lake-state': 'Colors identify discrete lake states rather than a numeric magnitude.',
    'seasonal-flow-regime': 'Colors identify dry, intermittent, perennial, and ocean states rather than a numeric magnitude.',
    'reconciliation-lake-change': 'The scale is binary: 0 means unchanged and 1 means the lake state changed.',
    'reconciliation-flow-regime-change': 'The scale is binary: 0 means unchanged and 1 means the seasonal flow regime changed.',
    'evolution-receiver-change': 'The scale is binary: 0 means the receiver is unchanged and 1 means the drainage receiver changed.',
    'inherited-mask': 'The scale is binary/categorical: one color marks inherited coarse samples and the other marks samples introduced by refinement.',
    'provenance': 'Colors are deterministic coarse-source identifiers. Hue has no numeric meaning.',
    'boundary-provenance': 'Colors are deterministic coarse-boundary/source identifiers. Hue has no numeric meaning.',
    'mesh': 'The lines are topology edges only. There is no scalar value or ordered color scale.',
    'dynamic-support': 'The index is signed from -1 to +1: negative values indicate downward dynamic support and positive values indicate upward support.',
    'mantle-relief': 'Values are signed meters: negative values lower the surface and positive values raise it.',
    'ocean-heat': 'The index is signed: negative and positive values indicate opposite directions of local heat-transport tendency relative to zero.',
    'moisture-balance': 'Values are precipitation minus potential evaporation in mm/yr: negative means climatic moisture deficit and positive means surplus.',
    'reconciliation-lake-depth-delta': 'Values are signed meters: negative means shallower after reconciliation and positive means deeper.',
    'reconciliation-realized-discharge-delta': 'Values are signed m³/s: negative means less realized flow after reconciliation and positive means more.',
    'reconciliation-flow-presence-delta': 'Values are signed fractions: negative means flow occurs in fewer orbital phases and positive means it occurs in more.',
    'evolution-terrain-delta': 'Values are signed meters: negative is net erosion and positive is net deposition.',
    'runoff-fraction': 'Values are fractions from 0 to 1: 0 means none of local precipitation becomes runoff and 1 means all of it does.',
    'lake-fraction': 'Values are fractions from 0 to 1: 0 means no lake-covered area in the sample and 1 means full coverage.',
    'seasonal-flow-presence': 'Values are fractions from 0 to 1: 0 means no modeled orbital phase has realized flow and 1 means every phase does.',
    'snowfall': 'Values are fractions from 0 to 1: 0 means precipitation is rain-dominated and 1 means it is snow-dominated.',
    'persistent-snow': 'Values are normalized from 0 to 1: higher values indicate greater persistent-snow potential.',
    'sea-ice': 'Values are normalized from 0 to 1: higher values indicate greater sea-ice potential.',
    'historical-rift': 'Values are normalized from 0 to 1: higher values mean stronger inherited rift influence.',
    'historical-shear': 'Values are normalized from 0 to 1: higher values mean stronger inherited shear influence.',
    'historical-suture': 'Values are normalized from 0 to 1: higher values mean stronger inherited suture influence.',
    'historical-passive-margin': 'Values are normalized from 0 to 1: higher values mean stronger passive-margin character.',
    'historical-active-orogen': 'Values are normalized from 0 to 1: higher values mean stronger active-orogen influence.',
    'historical-fossil-orogen': 'Values are normalized from 0 to 1: higher values mean stronger fossil-orogen inheritance.',
    'orogeny-history': 'Values are normalized from 0 to 1: higher values mean stronger cumulative orogenic history.',
    'ridge-history': 'Values are normalized from 0 to 1: higher values mean stronger cumulative ridge history.',
    'trench-history': 'Values are normalized from 0 to 1: higher values mean stronger cumulative trench/subduction history.',
    'strength': 'Values are normalized from 0 to 1: higher values mean a stronger lithosphere.',
    'weakness': 'Values are normalized from 0 to 1: higher values mean a weaker lithosphere.',
    'fragmentation': 'Values are normalized from 0 to 1: higher values mean greater fragmentation propensity.',
    'rock-strength': 'Values are normalized from 0 to 1: higher values mean more mechanically resistant bedrock.',
    'lithology-erodibility': 'Values are normalized from 0 to 1: higher values mean easier erosion.',
    'permeability': 'Values are normalized from 0 to 1: higher values mean greater relative permeability.',
    'weathering-susceptibility': 'Values are normalized from 0 to 1: higher values mean greater weathering susceptibility.',
    'fines-fraction': 'Values are normalized from 0 to 1: higher values mean a greater fine-grained material fraction.',
    'carbonate-fraction': 'Values are normalized from 0 to 1: higher values mean a greater carbonate fraction.',
    'precip-seasonality': 'The dimensionless display runs from 0 to 5. Values near 0 indicate precipitation distributed more evenly through the year; larger values indicate stronger seasonal concentration.',
    'aridity': 'The dimensionless display runs from 0 to 2. Lower values indicate wetter conditions relative to atmospheric demand; larger values indicate stronger water limitation.',
    'erosion-erodibility': 'This is the current WG-7A dimensionless erodibility index. Larger values increase erosive response under comparable flow and slope.',
    'erosion-channel-slope': 'This is dimensionless rise/run. Zero is flat; larger values represent steeper downstream channel gradients.',
    'contributing-area': 'Color uses a logarithmic transform of upstream area so small and continental-scale catchments can be seen together; larger values mean more upstream area.',
    'depression-depth': 'Color uses a logarithmic transform of depression depth; larger values mean a deeper closed basin below its spill level.',
    'escape-elevation': 'Color maps hydrologic escape elevation over a fixed display range; larger values mean water must reach a higher elevation to escape local containment.',
    'potential-discharge': 'Color is logarithmic in m³/s so low-flow and major-river values remain visible together; larger values mean more potential annual flow.',
    'annual-runoff': 'Color is logarithmic in annual runoff depth; larger values mean more locally generated runoff.',
    'actual-et': 'Color uses a saturating transform of annual actual evapotranspiration, so differences remain visible across both low- and high-ET climates.',
    'realized-discharge': 'Color is logarithmic in m³/s after lake storage and overflow; larger values mean greater annual realized river flow.',
    'lake-depth': 'Color is logarithmic in lake depth; larger values mean deeper equilibrium lakes.',
    'seasonal-realized-discharge': 'Color is logarithmic in m³/s at the selected orbital phase; larger values mean greater realized seasonal flow.',
    'seasonal-snow-storage': 'The displayed 0-1 color value is a bounded transform of snow-water storage; higher colors mean more stored snow water.',
    'evolution-contributing-area': 'Color is logarithmic in post-erosion contributing area; larger values identify larger reconstructed catchments.',
    'evolution-potential-discharge': 'Color is logarithmic in post-erosion potential discharge; larger values mean more accumulated annual flow.',
    'erosion-effective-discharge': 'Color is logarithmic in effective erosive discharge; larger values mean stronger flow available to drive incision.',
    'erosion-sediment-supply': 'Color is logarithmic in local sediment production (kg/s); larger values mean more sediment generated at that sample.',
    'erosion-sediment-load': 'Color is logarithmic in routed sediment load (kg/s); larger values mean more sediment carried through the channel network.',
    'erosion-sediment-deposition': 'Color is logarithmic in local deposition rate (kg/s); larger values mean more routed sediment is deposited.'
};
function diagnosticScaleMeaning(mode) {
    const specific = DIAGNOSTIC_SCALE_HELP[mode];
    if (specific)
        return specific;
    const categorical = CATEGORICAL_LEGENDS[mode];
    if (categorical)
        return 'Colors identify discrete classes; they are categorical rather than an ordered numeric scale.';
    const unit = DIAGNOSTIC_UNITS[mode];
    if (unit)
        return 'Legend values are shown directly in ' + unit + '. Colors progress from the displayed minimum to maximum, with values outside the display range clamped to an endpoint color.';
    if (current) {
        const field = scalarField(current, mode, orbitalPhase());
        if (field)
            return 'The numeric legend runs from ' + formatLegendNumber(mode, field.minimum) + ' to ' + formatLegendNumber(mode, field.maximum) + '; larger legend values correspond to larger values of this diagnostic.';
    }
    return 'The legend shows the ordering used by this diagnostic; generate a planet to see any data-dependent numeric bounds.';
}
function addDiagnosticHelp(mode) {
    const details = document.createElement('details');
    details.className = 'worldgen-diagnostic-help';
    const toggle = document.createElement('summary');
    toggle.textContent = 'About this diagnostic';
    const what = document.createElement('p');
    what.textContent = DIAGNOSTIC_SUMMARIES[mode] ?? diagnosticLabel(mode);
    const scale = document.createElement('p');
    const label = document.createElement('strong');
    label.textContent = 'How to read the scale: ';
    scale.append(label, document.createTextNode(diagnosticScaleMeaning(mode)));
    details.append(toggle, what, scale);
    diagnosticLegend.append(details);
}
const DIAGNOSTIC_UNITS = {
    'solid-elevation': 'm', 'relative-elevation': 'm', 'water-depth': 'm', 'isostatic': 'm', 'thermal': 'm', 'orogenic-relief': 'm',
    'ridge-relief': 'm', 'rift-basin': 'm', 'trench-relief': 'm', 'arc-relief': 'm', 'mantle-relief': 'm', 'historical-crust-birth-age': 'Myr',
    'historical-event-age': 'Myr', 'historical-rift-age': 'Myr', 'historical-suture-age': 'Myr', 'crust-age': 'Myr', 'crust-thickness': 'km',
    'annual-insolation': 'W/m²', 'seasonal-insolation': 'W/m²', 'temperature': 'K', 'seasonal-temperature': 'K', 'temperature-range': 'K',
    'sst': 'K', 'seasonal-sst': 'K', 'surface-pressure': 'Pa', 'wind-speed': 'm/s', 'current-speed': 'm/s', 'humidity': 'kg/kg',
    'precipitation': 'mm/yr', 'seasonal-precipitation': 'mm/yr', 'potential-evaporation': 'mm/yr', 'moisture-balance': 'mm/yr',
    'reconciliation-lake-depth-delta': 'm', 'infill-solid-elevation': 'm', 'infill-fill-depth': 'm', 'evolution-solid-elevation': 'm',
    'evolution-terrain-delta': 'm', 'evolution-applied-erosion': 'm', 'evolution-applied-deposition': 'm', 'erosion-channel-width': 'm',
    'erosion-incision-potential': 'm/yr', 'erosion-effective-discharge': 'm³/s', 'seasonal-realized-discharge': 'm³/s',
    'reconciliation-realized-discharge-delta': 'm³/s', 'evolution-potential-discharge': 'm³/s', 'erosion-sediment-supply': 'kg/s',
    'erosion-sediment-load': 'kg/s', 'erosion-sediment-deposition': 'kg/s'
};
const CATEGORICAL_LEGENDS = {
    'land-water': [{ label: 'Land', color: '#a99b72' }, { label: 'Ocean / water', color: '#214d7a' }],
    'crust-type': [{ label: 'Continental', color: '#b79a72' }, { label: 'Transitional', color: '#9aab87' }, { label: 'Oceanic', color: '#477aa3' }],
    'historical-event': [{ label: 'Rift', color: '#f59e42' }, { label: 'Spreading', color: '#50b9e8' }, { label: 'Shear / transform', color: '#7656d6' }, { label: 'Collision', color: '#e94f4f' }, { label: 'Accretion / capture', color: '#e8d35a' }, { label: 'Subduction', color: '#5dd18b' }, { label: 'Other inherited event', color: '#c178df' }],
    'bedrock-class': [
        { label: 'Oceanic basalt', color: '#355f7c' }, { label: 'Oceanic sediment', color: '#768896' }, { label: 'Crystalline basement', color: '#9c765d' },
        { label: 'Orogenic metamorphic', color: '#7b657d' }, { label: 'Arc volcanic', color: '#a94c3d' }, { label: 'Rift volcanic', color: '#b97842' },
        { label: 'Clastic sedimentary', color: '#c0a477' }, { label: 'Carbonate platform', color: '#ddd5a5' }, { label: 'Accreted terrane', color: '#6f8b68' }
    ],
    'structural-zones': [{ label: 'None', color: '#425362' }, { label: 'Suture', color: '#ff7466' }, { label: 'Rift', color: '#ffb45d' }, { label: 'Transform', color: '#c690ff' }, { label: 'Continental margin', color: '#65d7ac' }],
    'seasonal-flow-regime': [{ label: 'Dry', color: '#31423c' }, { label: 'Intermittent', color: '#e3a54f' }, { label: 'Perennial', color: '#4ea7dd' }, { label: 'Ocean', color: '#102c43' }],
    'lake-state': [{ label: 'No lake', color: '#31423c' }, { label: 'Endorheic', color: '#3aa7c9' }, { label: 'Overflowing', color: '#63d0a5' }, { label: 'Terminal storage', color: '#9b78d0' }, { label: 'Ocean', color: '#102c43' }],
    'inherited-mask': [{ label: 'Inherited coarse sample', color: '#f4e27a' }, { label: 'Fine-only sample', color: '#5794c8' }],
    'reconciliation-lake-change': [{ label: 'Unchanged', color: 'hsl(210 68% 37%)' }, { label: 'Changed', color: 'hsl(5 68% 60%)' }],
    'reconciliation-flow-regime-change': [{ label: 'Unchanged', color: 'hsl(210 68% 37%)' }, { label: 'Changed', color: 'hsl(5 68% 60%)' }],
    'evolution-receiver-change': [{ label: 'Unchanged', color: 'hsl(210 68% 37%)' }, { label: 'Changed', color: 'hsl(5 68% 60%)' }]
};
const IDENTITY_MODES = new Set(['plates', 'kinematic-domains', 'historical-origin', 'historical-fragments', 'historical-current', 'historical-provenance', 'provenance', 'boundary-provenance', 'basins', 'flow-direction', 'depressions']);
const LOG_DISPLAY_MODES = new Set(['seasonal-realized-discharge', 'evolution-contributing-area', 'evolution-potential-discharge', 'erosion-effective-discharge', 'erosion-sediment-load', 'erosion-sediment-supply', 'erosion-sediment-deposition']);
function diagnosticOption(mode = visualization.value) {
    return Array.from(visualization.options).find(option => option.value === mode) ?? null;
}
function categoryForDiagnostic(mode) {
    const group = diagnosticOption(mode)?.parentElement;
    return group instanceof HTMLOptGroupElement ? group.dataset.diagnosticCategory ?? null : null;
}
function setDiagnosticCategory(categoryId, preferredMode) {
    const groups = Array.from(visualization.querySelectorAll('optgroup[data-diagnostic-category]'));
    let first = null;
    for (const group of groups) {
        const active = group.dataset.diagnosticCategory === categoryId;
        group.hidden = !active;
        group.disabled = !active;
        if (active && !first)
            first = group.querySelector('option');
    }
    if (preferredMode && categoryForDiagnostic(preferredMode) === categoryId)
        visualization.value = preferredMode;
    else if (first)
        visualization.value = first.value;
}
function selectDiagnostic(mode) {
    const categoryId = categoryForDiagnostic(mode);
    if (categoryId) {
        diagnosticCategory.value = categoryId;
        setDiagnosticCategory(categoryId, mode);
    }
    else
        visualization.value = mode;
}
function diagnosticLabel(mode = visualization.value) {
    return diagnosticOption(mode)?.textContent?.trim() || mode;
}
function formatLegendNumber(mode, value) {
    let display = LOG_DISPLAY_MODES.has(mode) ? Math.expm1(value) : value;
    if (!Number.isFinite(display))
        display = 0;
    const abs = Math.abs(display);
    const digits = abs >= 1000 ? 0 : abs >= 100 ? 1 : abs >= 10 ? 2 : abs >= 1 ? 2 : 3;
    const unit = DIAGNOSTIC_UNITS[mode];
    return display.toFixed(digits) + (unit ? ' ' + unit : '');
}
function addLegendSwatches(items, target = diagnosticLegend) {
    const list = document.createElement('div');
    list.className = 'worldgen-legend-swatches';
    for (const item of items) {
        const row = document.createElement('span');
        const swatch = document.createElement('i');
        swatch.className = item.kind === 'line' ? 'worldgen-legend-line' : item.kind === 'vector' ? 'worldgen-legend-line worldgen-legend-vector' : '';
        swatch.style.background = item.color;
        row.append(swatch, document.createTextNode(item.label));
        list.append(row);
    }
    target.append(list);
}
function addLegendGradient(lowColor, highColor, lowLabel, middleLabel, highLabel) {
    const ramp = document.createElement('div');
    ramp.className = 'worldgen-legend-ramp';
    ramp.style.background = 'linear-gradient(90deg, ' + lowColor + ', ' + highColor + ')';
    const labels = document.createElement('div');
    labels.className = 'worldgen-legend-scale';
    for (const value of [lowLabel, middleLabel, highLabel]) {
        const span = document.createElement('span');
        span.textContent = value;
        labels.append(span);
    }
    diagnosticLegend.append(ramp, labels);
}
function addScalarLegendGradient(mode, field) {
    const ramp = document.createElement('div');
    ramp.className = 'worldgen-legend-ramp';
    const stops = [];
    for (let index = 0; index <= 8; index += 1) {
        const t = index / 8;
        const value = field.minimum + (field.maximum - field.minimum) * t;
        stops.push(scalarColor(value, field) + ' ' + (t * 100).toFixed(1) + '%');
    }
    ramp.style.background = 'linear-gradient(90deg, ' + stops.join(', ') + ')';
    const labels = document.createElement('div');
    labels.className = 'worldgen-legend-scale';
    const middle = (field.minimum + field.maximum) * 0.5;
    for (const value of [formatLegendNumber(mode, field.minimum), formatLegendNumber(mode, middle), formatLegendNumber(mode, field.maximum)]) {
        const span = document.createElement('span');
        span.textContent = value;
        labels.append(span);
    }
    diagnosticLegend.append(ramp, labels);
}
function customGradientForMode(mode) {
    const gradients = {
        'runoff-fraction': [48, 205, 'fraction'], 'actual-et': [42, 168, 'mm/yr'], 'annual-runoff': [44, 218, 'mm/yr'],
        'potential-discharge': [215, 18, 'm³/s'], 'realized-discharge': [205, 35, 'm³/s'], 'lake-fraction': [210, 175, 'fraction'],
        'lake-depth': [220, 175, 'm'], 'depression-depth': [55, 270, 'm'], 'escape-elevation': [220, 20, 'm'], 'contributing-area': [225, 42, 'km²']
    };
    const entry = gradients[mode];
    if (!entry)
        return null;
    return [drainageScalarColor(0, entry[0], entry[1]), drainageScalarColor(1, entry[0], entry[1]), 'low ' + entry[2], 'mid', 'high ' + entry[2]];
}
function refreshDiagnosticLegend() {
    diagnosticLegend.replaceChildren();
    const mode = visualization.value;
    const heading = document.createElement('strong');
    heading.textContent = diagnosticLabel(mode);
    diagnosticLegend.append(heading);
    addDiagnosticHelp(mode);
    const categorical = CATEGORICAL_LEGENDS[mode];
    if (categorical) {
        addLegendSwatches(categorical);
        return;
    }
    if (IDENTITY_MODES.has(mode)) {
        const colorAt = mode === 'plates' || mode === 'kinematic-domains'
            ? plateColor
            : mode === 'basins' || mode === 'flow-direction' || mode === 'depressions'
                ? discreteDrainageColor
                : mode === 'provenance' || mode === 'boundary-provenance'
                    ? provenanceColor
                    : (id) => historicalIdentityColor(id, mode === 'historical-fragments' ? 104 : mode === 'historical-current' ? 18 : mode === 'historical-provenance' ? 154 : 42);
        addLegendSwatches([{ label: 'ID / domain A', color: colorAt(1) }, { label: 'ID / domain B', color: colorAt(2) }, { label: 'ID / domain C', color: colorAt(3) }]);
        const note = document.createElement('small');
        note.textContent = 'Hue identifies a deterministic categorical ID; color ordering is not numeric.';
        diagnosticLegend.append(note);
        return;
    }
    if (mode === 'physical-world' || mode === 'physical-elevation') {
        addLegendSwatches([{ label: 'Deep ocean', color: '#20516c' }, { label: 'Shelf / shallow sea', color: '#a4dce1' }, { label: 'Lowland', color: '#7faa55' }, { label: 'Highland', color: '#b49b63' }]);
        return;
    }
    if (mode === 'mesh') {
        addLegendSwatches([{ label: 'Fine topology edge', color: '#5d7890' }]);
        return;
    }
    if (current) {
        const field = scalarField(current, mode, orbitalPhase());
        if (field) {
            addScalarLegendGradient(mode, field);
            return;
        }
    }
    const custom = customGradientForMode(mode);
    if (custom)
        addLegendGradient(custom[0], custom[1], custom[2], custom[3], custom[4]);
    else {
        const note = document.createElement('small');
        note.textContent = 'Legend becomes data-scaled after a planet has been generated.';
        diagnosticLegend.append(note);
    }
}
function bedrockLabel(kind) {
    if (kind === WORLDGEN_BEDROCK_OCEANIC_BASALT)
        return 'Oceanic basalt';
    if (kind === WORLDGEN_BEDROCK_OCEANIC_SEDIMENT)
        return 'Oceanic sediment';
    if (kind === WORLDGEN_BEDROCK_CRYSTALLINE_BASEMENT)
        return 'Crystalline basement';
    if (kind === WORLDGEN_BEDROCK_OROGENIC_METAMORPHIC)
        return 'Orogenic metamorphic';
    if (kind === WORLDGEN_BEDROCK_ARC_VOLCANIC)
        return 'Arc volcanic';
    if (kind === WORLDGEN_BEDROCK_RIFT_VOLCANIC)
        return 'Rift volcanic';
    if (kind === WORLDGEN_BEDROCK_CLASTIC_SEDIMENTARY)
        return 'Clastic sedimentary';
    if (kind === WORLDGEN_BEDROCK_CARBONATE_PLATFORM)
        return 'Carbonate platform';
    if (kind === WORLDGEN_BEDROCK_ACCRETED_TERRANE)
        return 'Accreted terrane';
    return 'Bedrock ' + kind;
}
function selectedDiagnosticSampleText(result, sample) {
    const mode = visualization.value;
    if (mode === 'land-water')
        return result.submergedMask[sample] ? 'Water' : 'Land';
    if (mode === 'bedrock-class')
        return bedrockLabel(result.bedrockClass[sample]);
    if (mode === 'crust-type') {
        const kind = result.crustKind[sample];
        return kind === WORLDGEN_CRUST_CONTINENTAL ? 'Continental crust' : kind === WORLDGEN_CRUST_TRANSITIONAL ? 'Transitional crust' : kind === WORLDGEN_CRUST_OCEANIC ? 'Oceanic crust' : 'Crust ' + kind;
    }
    if (mode === 'historical-event') {
        const kind = result.latestHistoricalEventKind[sample];
        return kind === 1 ? 'Rift' : kind === 2 ? 'Spreading' : kind === 3 ? 'Shear / transform' : kind === 4 ? 'Collision' : kind === 5 ? 'Accretion / capture' : kind === 6 ? 'Subduction' : kind === 7 ? 'Other inherited event' : 'No recent event';
    }
    if (mode === 'structural-zones') {
        const kind = result.structuralZoneKind[sample];
        return kind === WORLDGEN_STRUCTURE_SUTURE ? 'Suture' : kind === WORLDGEN_STRUCTURE_RIFT ? 'Rift' : kind === WORLDGEN_STRUCTURE_TRANSFORM ? 'Transform' : kind === WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN ? 'Continental margin' : 'No structural zone';
    }
    if (mode === 'plates')
        return 'Plate ' + result.plateIds[sample].toLocaleString();
    if (mode === 'kinematic-domains')
        return 'Kinematic domain ' + result.kinematicDomainIds[sample].toLocaleString();
    if (mode === 'historical-origin')
        return 'Ancestral plate ' + result.originPlateIds[sample].toLocaleString();
    if (mode === 'historical-fragments')
        return 'Fragment ' + result.historicalFragmentIds[sample].toLocaleString();
    if (mode === 'historical-current')
        return 'Current plate ' + result.currentPlateIds[sample].toLocaleString();
    if (mode === 'annual-runoff')
        return result.localRunoffMm[sample].toFixed(1) + ' mm/yr runoff';
    if (mode === 'runoff-fraction')
        return (result.runoffFraction[sample] * 100).toFixed(1) + '% runoff';
    if (mode === 'actual-et')
        return result.actualEvapotranspirationMm[sample].toFixed(1) + ' mm/yr AET';
    if (mode === 'potential-discharge')
        return result.potentialDischargeM3S[sample].toFixed(1) + ' m³/s potential discharge';
    if (mode === 'realized-discharge')
        return result.realizedDischargeM3S[sample].toFixed(1) + ' m³/s realized discharge';
    if (mode === 'lake-depth')
        return result.lakeDepthM[sample].toFixed(1) + ' m lake depth';
    if (mode === 'lake-fraction')
        return (result.lakeFraction[sample] * 100).toFixed(1) + '% lake fraction';
    if (mode === 'contributing-area')
        return (result.contributingAreaM2[sample] / 1e6).toFixed(1) + ' km² contributing area';
    const field = scalarField(result, mode, orbitalPhase());
    if (field)
        return diagnosticLabel(mode) + ' ' + formatLegendNumber(mode, field.values[sample]);
    return diagnosticLabel(mode);
}
const OVERLAY_LEGENDS = {
    'cell-boundaries': { description: 'Draws the canonical L8 dual-cell perimeter when the globe is zoomed far enough for individual physical cells to be screen-resolved.', items: () => [{ label: 'Physical cell edge', color: 'rgb(225,236,246)', kind: 'line' }], note: 'The border fades in with zoom; it is topology, not a geological boundary.' },
    'evolved-topography': { description: 'Draws elevation contours on the final post-WG-7D solid surface.', items: () => [{ label: '500 / 1000 / 2000 / 3000 / 4500 m', color: 'rgba(238,242,235,0.64)', kind: 'line' }], note: 'Higher contour levels are rendered slightly stronger and thicker.' },
    'final-rivers': { description: 'Draws final WG-7D realized river routing. Line width increases with realized annual discharge.', items: () => [{ label: 'Intermittent flow', color: 'rgba(99,188,224,0.74)', kind: 'line' }, { label: 'Perennial flow', color: 'rgba(65,177,236,0.92)', kind: 'line' }], note: 'Dry reaches and flows below the display threshold are omitted.' },
    'final-lakes': { description: 'Marks solved final lakes on land after WG-7D reconciliation and infill.', items: () => [{ label: 'Lake surface', color: 'rgba(65,174,224,0.82)' }], note: 'Opacity increases with modeled lake depth.' },
    'basin-divides': { description: 'Draws boundaries between neighboring final drainage basins.', items: () => [{ label: 'Drainage divide', color: 'rgba(236,207,132,0.72)', kind: 'line' }] },
    'cryosphere': { description: 'Overlays WG-5 persistent-snow potential on land and sea-ice potential over ocean.', items: () => [{ label: 'Persistent snow', color: 'rgba(245,248,250,0.78)' }, { label: 'Sea ice', color: 'rgba(190,229,244,0.78)' }], note: 'Samples below 0.2 potential are not drawn; opacity rises with potential.' },
    'topography': { description: 'Draws elevation contours on the original accepted WG-4 surface before WG-7 terrain evolution.', items: () => [{ label: '500 / 1000 / 2000 / 3000 / 4500 m', color: 'rgba(245,248,252,0.68)', kind: 'line' }], note: 'Compare with Final topographic contours to see where later geomorphology changed relief.' },
    'coastline': { description: 'Draws the fixed WG-4 land/ocean boundary used by the current climate and downstream surface pipeline.', items: () => [{ label: 'Coastline', color: 'rgba(225,236,246,0.84)', kind: 'line' }] },
    'winds': { description: 'Draws prevailing wind vectors reconstructed for the selected orbital phase.', items: () => [{ label: 'Wind direction / speed', color: 'rgba(245,249,255,0.82)', kind: 'vector' }], note: 'Vector direction follows reconstructed ENU wind; displayed length increases with speed and is bounded for readability.' },
    'currents': { description: 'Draws surface-ocean current vectors reconstructed for the selected orbital phase.', items: () => [{ label: 'Current direction / speed', color: 'rgba(91,220,255,0.92)', kind: 'vector' }], note: 'Vectors appear only over ocean; displayed length increases with current speed and is bounded for readability.' },
    'tectonic-boundaries': { description: 'Draws fine modern plate boundaries classified from relative kinematic motion.', items: () => [{ label: 'Convergent', color: tectonicBoundaryColor(WORLDGEN_BOUNDARY_CONVERGENT), kind: 'line' }, { label: 'Divergent', color: tectonicBoundaryColor(WORLDGEN_BOUNDARY_DIVERGENT), kind: 'line' }, { label: 'Transform', color: tectonicBoundaryColor(WORLDGEN_BOUNDARY_TRANSFORM), kind: 'line' }] },
    'geological-boundaries': { description: 'Draws fine boundary segments classified by crustal setting and tectonic regime.', items: () => [{ label: 'Oceanic subduction', color: geologicalBoundaryColor(WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION), kind: 'line' }, { label: 'Ocean-continent subduction', color: geologicalBoundaryColor(WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION), kind: 'line' }, { label: 'Continental collision', color: geologicalBoundaryColor(WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION), kind: 'line' }, { label: 'Oceanic ridge', color: geologicalBoundaryColor(WORLDGEN_GEOLOGY_OCEANIC_RIDGE), kind: 'line' }, { label: 'Continental rift', color: geologicalBoundaryColor(WORLDGEN_GEOLOGY_CONTINENTAL_RIFT), kind: 'line' }, { label: 'Transitional divergence', color: geologicalBoundaryColor(WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE), kind: 'line' }, { label: 'Transform', color: geologicalBoundaryColor(WORLDGEN_GEOLOGY_TRANSFORM), kind: 'line' }] }
};
function refreshOverlayLegends() {
    overlayLegends.replaceChildren();
    const selected = overlayInputs.filter(input => input.checked);
    overlayLegends.hidden = selected.length === 0;
    for (const input of selected) {
        const definition = OVERLAY_LEGENDS[input.value];
        if (!definition)
            continue;
        const card = document.createElement('section');
        card.className = 'worldgen-overlay-legend';
        const heading = document.createElement('strong');
        heading.textContent = input.dataset.label ?? input.value;
        const description = document.createElement('p');
        description.textContent = definition.description;
        card.append(heading, description);
        addLegendSwatches(definition.items(), card);
        if (definition.note) {
            const note = document.createElement('small');
            note.textContent = definition.note;
            card.append(note);
        }
        overlayLegends.append(card);
    }
}
const VIEW_PRESETS = {
    'custom': { mode: 'physical-elevation', overlays: [] },
    'physical-world': { mode: 'physical-world', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'cryosphere'] },
    'tectonic-history': { mode: 'historical-current', overlays: ['coastline', 'tectonic-boundaries', 'geological-boundaries'] },
    'crust-lithology': { mode: 'bedrock-class', overlays: ['coastline', 'tectonic-boundaries'] },
    'climate': { mode: 'precipitation', overlays: ['coastline', 'winds'] },
    'hydrologic-atlas': { mode: 'physical-world', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'basin-divides'] },
    'seasonal-world': { mode: 'seasonal-realized-discharge', overlays: ['evolved-topography', 'coastline', 'final-lakes', 'winds'] },
    'geomorphic-processes': { mode: 'evolution-terrain-delta', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'tectonic-boundaries'] },
};
function applyViewPreset(name) {
    const definition = VIEW_PRESETS[name];
    if (!definition)
        return;
    selectDiagnostic(definition.mode);
    const wanted = new Set(definition.overlays);
    for (const input of overlayInputs)
        input.checked = wanted.has(input.value);
    styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
    updateOverlaySummary();
    refreshDiagnosticLegend();
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
    crashRecorder.recordProgress(progress);
    refreshCrashDebugSummary();
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
    generationStep.textContent = `${result.metrics.spinupYears} climate spin-up years${result.metrics.spinupConverged ? '' : ' · bounded fallback accepted'}`;
    generationTimer.textContent = formatDuration(result.stage.durationMs);
    showGenerationProfile(result);
}
function orbitalPhase() { return Number(season.value) / 1000; }
function updateSeasonLabel() { seasonValue.textContent = `${(orbitalPhase() * 100).toFixed(1)}% orbit`; }
function redraw(interactive = false) {
    if (!current || !buffers)
        return;
    const map = projection.value === 'map';
    renderPlanet(surfaceCanvas, canvas, current, projection.value, visualization.value, selectedOverlays(), orbitalPhase(), map ? mapCenterLongitude : yaw, map ? mapCenterLatitude : pitch, zoom, buffers, interactive, animationPhase, selectedTile);
}
function scheduleRedraw(interactive) {
    if (frameRequest)
        return;
    frameRequest = requestAnimationFrame(() => { frameRequest = 0; redraw(interactive); });
}
function scheduleSettledCameraRedraw() {
    if (cameraSettleHandle)
        clearTimeout(cameraSettleHandle);
    cameraSettleHandle = setTimeout(() => { cameraSettleHandle = null; redraw(false); }, 90);
}
function updateZoomLabel() { zoomValue.textContent = `${zoom.toFixed(1)}×`; }
function setZoom(next, interactive = true) {
    zoom = Math.max(1, Math.min(24, next));
    if (projection.value === 'map')
        mapCenterLatitude = clampEquirectangularCenterLatitude(mapCenterLatitude, zoom);
    zoomControl.value = zoom.toFixed(1);
    updateZoomLabel();
    if (interactive) {
        scheduleRedraw(true);
        scheduleSettledCameraRedraw();
    }
    else
        redraw(false);
}
function updateCameraControls() {
    zoomControl.disabled = false;
    resetCamera.disabled = false;
}
function inspectTile(sample) {
    if (!current || sample === null) {
        cellInspector.textContent = 'Click the globe or map to inspect an L8 physical cell.';
        return;
    }
    const degree = current.neighborOffsets[sample + 1] - current.neighborOffsets[sample];
    const relativeElevation = current.postInfillSolidElevationM[sample] - current.metrics.seaLevelM;
    const surface = current.submergedMask[sample] ? `${current.waterDepthM[sample].toFixed(0)} m water depth` : `${relativeElevation.toFixed(0)} m final elevation`;
    const basin = current.basinId[sample] === WORLDGEN_INVALID_SAMPLE_ID ? 'no basin' : `basin ${current.basinId[sample].toLocaleString()}`;
    const diagnosticValue = selectedDiagnosticSampleText(current, sample);
    cellInspector.textContent = 'Cell ' + sample.toLocaleString() + ' · ' + (degree === 5 ? 'pentagon' : 'hexagon') + ' · plate ' + current.plateIds[sample].toLocaleString() + ' · ' + surface + ' · ' + basin + ' · ' + diagnosticValue;
}
function pickTileAtPointer(event) {
    if (!current)
        return null;
    const rect = canvas.getBoundingClientRect();
    const x = (event.clientX - rect.left) * canvas.width / Math.max(1, rect.width);
    const y = (event.clientY - rect.top) * canvas.height / Math.max(1, rect.height);
    const direction = projection.value === 'map'
        ? equirectangularScreenToWorldDirection(x, y, canvas.width, canvas.height, {
            centerLongitudeRad: mapCenterLongitude,
            centerLatitudeRad: mapCenterLatitude,
            zoom,
        })
        : screenToWorldDirection(x, y, canvas.width, canvas.height, { yaw, pitch, zoom });
    return direction ? pickNearestSample(current, direction) : null;
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
            redraw(false);
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
    metric(metrics, 'Spin-up', `${result.metrics.spinupYears} model years · ΔT ${result.metrics.finalTemperatureRmsChangeK.toFixed(3)} K RMS · ${result.metrics.spinupConverged ? 'converged' : `bounded fallback (target ≤ ${result.metrics.convergenceTemperatureRmsK.toFixed(3)} K)`}`);
    metric(metrics, 'Planet forcing', `${result.planet.stellarFluxWM2.toFixed(0)} W/m² · tilt ${(result.planet.axialTiltRad * 180 / Math.PI).toFixed(2)}° · e ${result.climatePhysical.orbitalEccentricity.toFixed(4)}`);
    metric(metrics, 'Land / ocean', `${(result.metrics.landAreaFraction * 100).toFixed(1)}% / ${(result.metrics.oceanAreaFraction * 100).toFixed(1)}%`);
    metric(metrics, 'Climate duration', `${result.stage.durationMs.toFixed(1)} ms`);
    metric(metrics, 'Final hydrology / drainage stage', `v${result.engineVersion} · ${result.drainageStage.id}@${result.drainageStage.version}`);
    metric(metrics, 'Final drainage topology', `${result.drainageMetrics.basinCount.toLocaleString()} basins · ${result.drainageMetrics.depressionCount.toLocaleString()} depressions`);
    metric(metrics, 'Final largest contributing area', `${(result.drainageMetrics.maximumContributingAreaM2 / 1e12).toFixed(3)} million km²`);
    metric(metrics, 'Final deepest depression', `${result.drainageMetrics.maximumDepressionDepthM.toFixed(1)} m`);
    metric(metrics, 'Final drainage area closure', result.drainageMetrics.areaConservationRelativeError.toExponential(2));
    metric(metrics, 'Final drainage hash', result.drainageMetrics.drainageHash);
    metric(metrics, 'Pre-erosion WG-6A drainage hash', result.reconciliationMetrics.preErosionDrainageHash);
    metric(metrics, 'Final hydrology identity', result.runoffMetrics.climateHash === result.metrics.climateHash && result.runoffMetrics.drainageHash === result.drainageMetrics.drainageHash ? 'WG-7D canonical drainage / runoff match' : 'MISMATCH');
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
    metric(metrics, 'WG-7D / stage', `v${result.engineVersion} · ${result.infillStage.id}@${result.infillStage.version}`);
    metric(metrics, 'WG-7D infill horizon', `${result.infillMetrics.geomorphicDurationYears.toFixed(0)} y · ${result.infillMetrics.historicalLakeTrapCount.toLocaleString()} historical traps`);
    metric(metrics, 'WG-7D filled depressions / samples', `${result.infillMetrics.filledDepressionCount.toLocaleString()} / ${result.infillMetrics.filledSampleCount.toLocaleString()} · ${result.infillMetrics.capacityLimitedDepressionCount.toLocaleString()} capacity-limited`);
    metric(metrics, 'WG-7D max fill', `${result.infillMetrics.maximumFillDepthM.toFixed(3)} m`);
    metric(metrics, 'WG-7D sediment delivery', `${result.infillMetrics.totalHistoricalLakeDeliveryKgS.toFixed(1)} kg/s · applied ${result.infillMetrics.totalAppliedLakeFillEquivalentKgS.toFixed(1)} · unapplied ${result.infillMetrics.totalUnappliedLakeSedimentKgS.toFixed(1)}`);
    metric(metrics, 'WG-7D lake count', `${result.infillMetrics.preInfillLakeCount.toLocaleString()} → ${result.infillMetrics.postInfillLakeCount.toLocaleString()}`);
    metric(metrics, 'WG-7D sediment closure', result.infillMetrics.sedimentConservationRelativeError.toExponential(2));
    metric(metrics, 'WG-7D final hydro closure', `runoff ${result.infillMetrics.postInfillRunoffConservationRelativeError.toExponential(2)} · lake ${result.infillMetrics.postInfillLakeWaterBalanceRelativeError.toExponential(2)} · seasonal ${result.infillMetrics.postInfillSeasonalWaterBalanceRelativeError.toExponential(2)}`);
    metric(metrics, 'WG-7D surface / drainage hash', `${result.infillMetrics.postInfillSurfaceHash} / ${result.infillMetrics.postInfillDrainageHash}`);
    metric(metrics, 'WG-7D infill hash', result.infillMetrics.lakeSedimentInfillHash);
}
function calibrationFileStem(value) {
    const normalized = value.trim().replace(/[^a-zA-Z0-9._-]+/g, '-').replace(/^-+|-+$/g, '');
    return normalized || 'planet';
}
async function copyCrashReportToClipboard() {
    if (!crashRecorder.hasAttempt())
        return;
    try {
        await navigator.clipboard.writeText(crashRecorder.toMarkdown());
        status.textContent = 'Copied the full worldgen crash/debug report to the clipboard.';
    }
    catch (error) {
        status.textContent = `Could not copy crash report: ${error instanceof Error ? error.message : String(error)}`;
    }
}
function downloadCrashReportJson() {
    if (!crashRecorder.hasAttempt())
        return;
    const snapshot = crashRecorder.snapshot();
    const blob = new Blob([crashRecorder.toJson()], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `planet-crash-report-${calibrationFileStem(snapshot.request?.seed ?? 'worldgen')}-${snapshot.runId.slice(0, 8)}.json`;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    URL.revokeObjectURL(url);
    status.textContent = 'Downloaded structured worldgen crash/debug report.';
}
async function copyCalibrationReport() {
    if (!current || !currentCalibrationRequest)
        return;
    try {
        await navigator.clipboard.writeText(worldCalibrationMarkdown(current, currentCalibrationRequest.seed, currentCalibrationRequest.plateCount));
        status.textContent = 'Copied compact LLM calibration summary to the clipboard.';
    }
    catch (error) {
        status.textContent = `Could not copy calibration summary: ${error instanceof Error ? error.message : String(error)}`;
    }
}
function downloadCalibrationReport() {
    if (!current || !currentCalibrationRequest)
        return;
    const blob = new Blob([worldCalibrationJson(current, currentCalibrationRequest.seed, currentCalibrationRequest.plateCount)], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `planet-calibration-${calibrationFileStem(currentCalibrationRequest.seed)}.json`;
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    URL.revokeObjectURL(url);
    status.textContent = 'Downloaded structured calibration packet.';
}
async function generatePlanet() {
    const request = { seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) };
    crashRecorder.startRun(request, receiveOnlyDebug.checked);
    refreshCrashDebugSummary();
    generate.disabled = true;
    copyCalibration.disabled = true;
    downloadCalibration.disabled = true;
    startGenerationTelemetry();
    status.textContent = receiveOnlyDebug.checked
        ? 'Diagnostic receive-only run: generating through WG-7D and stopping immediately after transport / validation…'
        : 'Generating one physical planet through WG-7D lake sediment infill in Rust/WASM…';
    try {
        crashRecorder.record('lab', 'generate-climate-await-begin', { receiveOnly: receiveOnlyDebug.checked });
        const loaded = await client.generateClimate(request, handleGenerationProgress);
        crashRecorder.record('lab', 'generate-climate-promise-resolved', { fineSampleCount: loaded.metrics.fineSampleCount });
        crashRecorder.recordResult(loaded);
        crashRecorder.record('lab', 'identity-validation-begin');
        refreshCrashDebugSummary();
        if (loaded.runoffMetrics.climateHash !== loaded.metrics.climateHash)
            throw new Error('WG-6B climate identity does not match accepted WG-5 forcing.');
        if (loaded.runoffMetrics.drainageHash !== loaded.drainageMetrics.drainageHash)
            throw new Error('Final runoff drainage identity does not match canonical WG-7D drainage.');
        if (loaded.lakeMetrics.climateHash !== loaded.metrics.climateHash)
            throw new Error('WG-6C climate identity does not match accepted WG-5 forcing.');
        if (loaded.lakeMetrics.drainageHash !== loaded.drainageMetrics.drainageHash)
            throw new Error('Final lake drainage identity does not match canonical WG-7D drainage.');
        if (loaded.lakeMetrics.runoffHash !== loaded.runoffMetrics.runoffHash)
            throw new Error('Final lake runoff identity does not match canonical WG-7D runoff.');
        if (loaded.seasonalMetrics.climateHash !== loaded.metrics.climateHash)
            throw new Error('WG-6D climate identity does not match accepted WG-5 forcing.');
        if (loaded.seasonalMetrics.drainageHash !== loaded.drainageMetrics.drainageHash)
            throw new Error('Final seasonal drainage identity does not match canonical WG-7D drainage.');
        if (loaded.seasonalMetrics.runoffHash !== loaded.runoffMetrics.runoffHash)
            throw new Error('Final seasonal runoff identity does not match canonical WG-7D runoff.');
        if (loaded.seasonalMetrics.lakeHash !== loaded.lakeMetrics.lakeHash)
            throw new Error('Final seasonal lake identity does not match canonical WG-7D lake state.');
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
        if (loaded.reconciliationMetrics.postErosionDrainageHash !== loaded.infillMetrics.preInfillDrainageHash)
            throw new Error('WG-7C drainage identity does not match WG-7D pre-infill ancestry.');
        if (loaded.reconciliationMetrics.reconciledRunoffHash !== loaded.infillMetrics.preInfillRunoffHash)
            throw new Error('WG-7C runoff identity does not match WG-7D pre-infill ancestry.');
        if (loaded.reconciliationMetrics.reconciledLakeHash !== loaded.infillMetrics.preInfillLakeHash)
            throw new Error('WG-7C lake identity does not match WG-7D pre-infill ancestry.');
        if (loaded.reconciliationMetrics.reconciledSeasonalHash !== loaded.infillMetrics.preInfillSeasonalHash)
            throw new Error('WG-7C seasonal identity does not match WG-7D pre-infill ancestry.');
        if (loaded.infillMetrics.topographyHash !== loaded.metrics.topographyHash || loaded.infillMetrics.climateHash !== loaded.metrics.climateHash)
            throw new Error('WG-7D immutable WG-4/WG-5 ancestry mismatch.');
        if (loaded.infillMetrics.preErosionDrainageHash !== loaded.reconciliationMetrics.preErosionDrainageHash || loaded.infillMetrics.preErosionLakeHash !== loaded.reconciliationMetrics.preErosionLakeHash)
            throw new Error('WG-7D pre-erosion hydrology ancestry mismatch.');
        if (loaded.infillMetrics.fluvialErosionHash !== loaded.erosionMetrics.fluvialErosionHash || loaded.infillMetrics.terrainEvolutionHash !== loaded.evolutionMetrics.terrainEvolutionHash)
            throw new Error('WG-7D WG-7A/WG-7B geomorphic ancestry mismatch.');
        if (loaded.infillMetrics.postErosionHydrologyHash !== loaded.reconciliationMetrics.postErosionHydrologyHash || loaded.infillMetrics.inputEvolvedSurfaceHash !== loaded.evolutionMetrics.evolvedSurfaceHash)
            throw new Error('WG-7D WG-7C/evolved-surface ancestry mismatch.');
        if (loaded.infillMetrics.postInfillDrainageHash !== loaded.drainageMetrics.drainageHash)
            throw new Error('WG-7D final drainage identity mismatch.');
        if (loaded.infillMetrics.postInfillRunoffHash !== loaded.runoffMetrics.runoffHash)
            throw new Error('WG-7D final runoff identity mismatch.');
        if (loaded.infillMetrics.postInfillLakeHash !== loaded.lakeMetrics.lakeHash)
            throw new Error('WG-7D final lake identity mismatch.');
        if (loaded.infillMetrics.postInfillSeasonalHash !== loaded.seasonalMetrics.seasonalHydrologyHash)
            throw new Error('WG-7D final seasonal identity mismatch.');
        crashRecorder.record('lab', 'identity-validation-complete');
        refreshCrashDebugSummary();
        if (receiveOnlyDebug.checked) {
            crashRecorder.record('lab', 'receive-only-viewer-skipped', { transportMiB: crashRecorder.snapshot().transport?.totalMiB });
            finishGenerationTelemetry(loaded);
            generationStage.textContent = 'Receive-only complete';
            generationStep.textContent = 'Transport + identity validation succeeded; viewer allocation/render intentionally skipped';
            generationTimer.textContent = formatDuration(performance.now() - generationStartedAt);
            status.textContent = `Receive-only diagnostic succeeded: ${loaded.metrics.fineSampleCount.toLocaleString()} samples crossed the worker boundary. Viewer setup was skipped.`;
            crashRecorder.complete('receive-only-complete');
            refreshCrashDebugSummary();
            return;
        }
        crashRecorder.record('lab', 'viewer-state-install-begin');
        current = loaded;
        currentCalibrationRequest = { seed: request.seed, plateCount: request.plateCount };
        copyCalibration.disabled = false;
        downloadCalibration.disabled = false;
        crashRecorder.record('lab', 'viewer-projection-buffer-allocation-begin', { fineSampleCount: loaded.metrics.fineSampleCount });
        buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };
        crashRecorder.record('lab', 'viewer-projection-buffer-allocation-complete', { bytes: buffers.x.byteLength + buffers.y.byteLength + buffers.visible.byteLength });
        styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
        edgeOverlayCache = { result: null, coastline: new Uint32Array(0), contours: [], evolvedContours: [], basinDivides: new Uint32Array(0), riverBuckets: [] };
        gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 };
        mapRasterCache = { result: null, lookupKey: '', styleKey: '', width: 0, height: 0, sampleIds: new Uint32Array(0) };
        projectedResult = null;
        projectedKey = '';
        selectedTile = null;
        inspectTile(null);
        crashRecorder.record('lab', 'viewer-cache-reset-complete');
        crashRecorder.record('lab', 'viewer-metrics-render-begin');
        showMetrics(loaded);
        refreshDiagnosticLegend();
        crashRecorder.record('lab', 'viewer-metrics-render-complete');
        crashRecorder.record('lab', 'viewer-first-render-begin');
        redraw(false);
        crashRecorder.record('lab', 'viewer-first-render-complete');
        updateAnimation();
        crashRecorder.record('lab', 'viewer-animation-armed');
        finishGenerationTelemetry(loaded);
        crashRecorder.record('lab', 'viewer-state-install-complete');
        generationStep.textContent = `${loaded.metrics.spinupYears} climate spin-up years · ${loaded.drainageMetrics.basinCount.toLocaleString()} basins · ${loaded.lakeMetrics.lakeCount.toLocaleString()} equilibrium lakes · ${loaded.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} receivers changed after evolution · ${loaded.infillMetrics.filledDepressionCount.toLocaleString()} lake basins infilled`;
        generationTimer.textContent = formatDuration(performance.now() - generationStartedAt);
        status.textContent = `Planet ready through WG-7D: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.evolutionMetrics.erodedSampleCount.toLocaleString()} evolved erosion cells, ${loaded.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} drainage receivers changed, mean land |Δz| ${loaded.evolutionMetrics.meanLandAbsoluteTerrainChangeM.toFixed(3)} m, sediment closure ${loaded.evolutionMetrics.sedimentConservationRelativeError.toExponential(2)}.`;
        crashRecorder.complete('completed');
        refreshCrashDebugSummary();
    }
    catch (error) {
        if (generationTimerHandle) {
            clearInterval(generationTimerHandle);
            generationTimerHandle = null;
        }
        crashRecorder.recordError(error, 'generate-planet');
        const snapshot = crashRecorder.snapshot();
        generationStage.textContent = 'Generation failed';
        generationStep.textContent = `Last checkpoint: ${snapshot.lastCheckpoint ?? 'unknown'}`;
        status.textContent = error instanceof Error ? error.message : String(error);
        refreshCrashDebugSummary();
    }
    finally {
        generate.disabled = false;
    }
}
generate.addEventListener('click', () => void generatePlanet());
copyCalibration.addEventListener('click', () => void copyCalibrationReport());
downloadCalibration.addEventListener('click', downloadCalibrationReport);
copyCrashReport.addEventListener('click', () => void copyCrashReportToClipboard());
downloadCrashReport.addEventListener('click', downloadCrashReportJson);
projection.addEventListener('change', () => { updateCameraControls(); redraw(false); });
preset.addEventListener('change', () => applyViewPreset(preset.value));
diagnosticCategory.addEventListener('change', () => {
    preset.value = 'custom';
    setDiagnosticCategory(diagnosticCategory.value);
    styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
    gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 };
    refreshDiagnosticLegend();
    selectedTile = null;
    inspectTile(null);
    redraw(false);
    updateAnimation();
});
visualization.addEventListener('change', () => {
    preset.value = 'custom';
    const categoryId = categoryForDiagnostic(visualization.value);
    if (categoryId)
        diagnosticCategory.value = categoryId;
    styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
    gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 };
    refreshDiagnosticLegend();
    if (selectedTile !== null)
        inspectTile(selectedTile);
    redraw(false);
    updateAnimation();
});
overlayInputs.forEach(input => input.addEventListener('change', () => { preset.value = 'custom'; updateOverlaySummary(); redraw(false); updateAnimation(); }));
season.addEventListener('input', () => { updateSeasonLabel(); styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 }; refreshDiagnosticLegend(); if (selectedTile !== null)
    inspectTile(selectedTile); redraw(false); });
zoomControl.addEventListener('input', () => setZoom(Number(zoomControl.value), true));
resetCamera.addEventListener('click', () => {
    if (projection.value === 'map') {
        mapCenterLongitude = 0;
        mapCenterLatitude = 0;
    }
    else {
        yaw = -0.65;
        pitch = 0.25;
    }
    selectedTile = null;
    inspectTile(null);
    setZoom(1, false);
});
canvas.addEventListener('wheel', event => {
    if (!current)
        return;
    event.preventDefault();
    const rect = canvas.getBoundingClientRect();
    const canvasX = (event.clientX - rect.left) * canvas.width / Math.max(1, rect.width);
    const canvasY = (event.clientY - rect.top) * canvas.height / Math.max(1, rect.height);
    const nextZoom = Math.max(1, Math.min(24, zoom * Math.exp(-event.deltaY * 0.0015)));
    if (projection.value === 'map') {
        const anchorDirection = equirectangularScreenToWorldDirection(canvasX, canvasY, canvas.width, canvas.height, {
            centerLongitudeRad: mapCenterLongitude,
            centerLatitudeRad: mapCenterLatitude,
            zoom,
        });
        const camera = equirectangularCameraForWorldDirectionAtScreen(anchorDirection, canvasX, canvasY, canvas.width, canvas.height, nextZoom);
        mapCenterLongitude = camera.centerLongitudeRad;
        mapCenterLatitude = camera.centerLatitudeRad;
    }
    else {
        const anchorDirection = screenToWorldDirection(canvasX, canvasY, canvas.width, canvas.height, { yaw, pitch, zoom });
        if (anchorDirection) {
            const camera = cameraForWorldDirectionAtScreen(anchorDirection, canvasX, canvasY, canvas.width, canvas.height, nextZoom, { yaw, pitch, zoom });
            yaw = camera.yaw;
            pitch = camera.pitch;
        }
    }
    setZoom(nextZoom, true);
}, { passive: false });
canvas.addEventListener('pointerdown', event => {
    if (!current)
        return;
    drag = {
        x: event.clientX,
        y: event.clientY,
        yaw,
        pitch,
        mapLongitude: mapCenterLongitude,
        mapLatitude: mapCenterLatitude,
        projection: projection.value,
    };
    canvas.setPointerCapture(event.pointerId);
});
canvas.addEventListener('pointermove', event => {
    if (!drag || projection.value !== drag.projection)
        return;
    if (projection.value === 'map') {
        const rect = canvas.getBoundingClientRect();
        const longitudeSpan = TWO_PI / Math.max(1, zoom);
        const latitudeSpan = Math.PI / Math.max(1, zoom);
        mapCenterLongitude = wrapLongitudeRad(drag.mapLongitude - (event.clientX - drag.x) / Math.max(1, rect.width) * longitudeSpan);
        mapCenterLatitude = clampEquirectangularCenterLatitude(drag.mapLatitude + (event.clientY - drag.y) / Math.max(1, rect.height) * latitudeSpan, zoom);
    }
    else {
        const sensitivity = 0.007 / Math.sqrt(zoom);
        yaw = drag.yaw + (event.clientX - drag.x) * sensitivity;
        pitch = Math.max(-1.45, Math.min(1.45, drag.pitch + (event.clientY - drag.y) * sensitivity));
    }
    scheduleRedraw(true);
    scheduleSettledCameraRedraw();
});
canvas.addEventListener('pointerup', event => {
    const finishedDrag = drag;
    drag = null;
    if (canvas.hasPointerCapture(event.pointerId))
        canvas.releasePointerCapture(event.pointerId);
    if (finishedDrag && Math.hypot(event.clientX - finishedDrag.x, event.clientY - finishedDrag.y) < 4) {
        selectedTile = pickTileAtPointer(event);
        inspectTile(selectedTile);
    }
    scheduleRedraw(false);
});
canvas.addEventListener('pointercancel', () => { drag = null; scheduleRedraw(false); });
window.addEventListener('beforeunload', () => {
    if (frameRequest)
        cancelAnimationFrame(frameRequest);
    if (animationRequest)
        cancelAnimationFrame(animationRequest);
    if (cameraSettleHandle)
        clearTimeout(cameraSettleHandle);
    crashRecorder.record('browser', 'beforeunload');
    removeGlobalFailureCapture();
    globeRenderer.dispose();
    client.dispose();
});
coarseLevel.max = String(WORLDGEN_CLIMATE_COARSE_MAX_LEVEL);
coarseLevel.value = String(WORLDGEN_CLIMATE_COARSE_MAX_LEVEL);
fineLevel.max = String(WORLDGEN_CLIMATE_FINE_MAX_LEVEL);
fineLevel.value = String(WORLDGEN_CLIMATE_FINE_MAX_LEVEL);
updateSeasonLabel();
updateZoomLabel();
updateCameraControls();
updateOverlaySummary();
refreshCrashDebugSummary();
setDiagnosticCategory('world', 'physical-elevation');
applyViewPreset(preset.value);
refreshDiagnosticLegend();
generationStage.textContent = 'Ready for canonical L8 generation · maximum fidelity';
generationStep.textContent = 'L' + WORLDGEN_CLIMATE_COARSE_MAX_LEVEL + ' coarse physical state → L' + WORLDGEN_CLIMATE_FINE_MAX_LEVEL + ' final physical planet';
status.textContent = 'Ready. Generate the canonical L' + WORLDGEN_CLIMATE_COARSE_MAX_LEVEL + ' → L' + WORLDGEN_CLIMATE_FINE_MAX_LEVEL + ' physical world when you want to allocate the full-resolution state.';
