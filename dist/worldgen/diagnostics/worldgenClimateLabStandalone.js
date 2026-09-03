import { createWorldgenClient } from '../worldgenClient.js';
import { WORLDGEN_BOUNDARY_CONVERGENT, WORLDGEN_BOUNDARY_DIVERGENT, WORLDGEN_BOUNDARY_TRANSFORM, WORLDGEN_CRUST_CONTINENTAL, WORLDGEN_CRUST_OCEANIC, WORLDGEN_CRUST_TRANSITIONAL, WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION, WORLDGEN_GEOLOGY_CONTINENTAL_RIFT, WORLDGEN_GEOLOGY_OCEANIC_RIDGE, WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION, WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION, WORLDGEN_GEOLOGY_TRANSFORM, WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE, WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN, WORLDGEN_STRUCTURE_NONE, WORLDGEN_STRUCTURE_RIFT, WORLDGEN_STRUCTURE_SUTURE, WORLDGEN_STRUCTURE_TRANSFORM, } from '../protocol.js';
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
    return mean + cosine * Math.cos(angle) + sine * Math.sin(angle);
}
function seasonalScalar(mean, cosine, sine, phase, scratch) {
    const angle = phase * TWO_PI;
    const c = Math.cos(angle);
    const s = Math.sin(angle);
    for (let index = 0; index < scratch.length; index += 1)
        scratch[index] = mean[index] + cosine[index] * c + sine[index] * s;
    return scratch;
}
function magnitudeField(east, north, scratch) {
    for (let index = 0; index < scratch.length; index += 1)
        scratch[index] = Math.hypot(east[index], north[index]);
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
    if (mode === 'provenance')
        return provenanceColor(result.nearestCoarseSource[sample]);
    if (mode === 'inherited-mask')
        return result.inheritedSampleMask[sample] ? '#f4e27a' : '#5794c8';
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
    if (projection === 'map') {
        const cosLat = Math.max(0.18, Math.cos(lat));
        return [tangent[0] * width * 0.014 / cosLat, -tangent[2] * height * 0.026];
    }
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
    const phaseKey = ['seasonal-temperature', 'seasonal-sst'].includes(mode) ? phase.toFixed(3) : 'mean';
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
function renderPlanet(canvas, result, projection, mode, phase, yaw, pitch, buffers, interactive, animation) {
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
    const cacheKey = `${mode}:${['seasonal-temperature', 'seasonal-sst'].includes(mode) ? phase.toFixed(3) : 'mean'}`;
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
}
const seed = element('worldgen-seed');
const coarseLevel = element('worldgen-coarse-level');
const fineLevel = element('worldgen-level');
const plates = element('worldgen-plates');
const projection = element('worldgen-projection');
const visualization = element('worldgen-visualization');
const season = element('worldgen-season');
const seasonValue = element('worldgen-season-value');
const generate = element('worldgen-generate');
const status = element('worldgen-status');
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
function orbitalPhase() { return Number(season.value) / 1000; }
function updateSeasonLabel() { seasonValue.textContent = `${(orbitalPhase() * 100).toFixed(1)}% orbit`; }
function redraw(interactive = false) {
    if (!current || !buffers)
        return;
    renderPlanet(canvas, current, projection.value, visualization.value, orbitalPhase(), yaw, pitch, buffers, interactive, animationPhase);
}
function scheduleRedraw(interactive) {
    if (frameRequest)
        return;
    frameRequest = requestAnimationFrame(() => { frameRequest = 0; redraw(interactive && drag !== null); });
}
function vectorAnimationFrame(timestampMs) {
    animationRequest = 0;
    if (visualization.value === 'winds' || visualization.value === 'currents') {
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
    if (visualization.value === 'winds' || visualization.value === 'currents')
        animationRequest = requestAnimationFrame(vectorAnimationFrame);
}
function showMetrics(result) {
    metrics.replaceChildren();
    metric(metrics, 'Engine / stage', `v${result.engineVersion} · ${result.stage.id}@${result.stage.version}`);
    metric(metrics, 'Resolution', `L${result.coarseLevel} → L${result.fineLevel}`);
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
    metric(metrics, 'Snow / sea ice potential', `${(result.metrics.persistentSnowAreaFraction * 100).toFixed(1)}% / ${(result.metrics.seaIceAreaFraction * 100).toFixed(1)}% area`);
    metric(metrics, 'Spin-up', `${result.metrics.spinupYears} model years · ΔT ${result.metrics.finalTemperatureRmsChangeK.toFixed(3)} K RMS`);
    metric(metrics, 'Planet forcing', `${result.planet.stellarFluxWM2.toFixed(0)} W/m² · tilt ${(result.planet.axialTiltRad * 180 / Math.PI).toFixed(2)}° · e ${result.climatePhysical.orbitalEccentricity.toFixed(4)}`);
    metric(metrics, 'Land / ocean', `${(result.metrics.landAreaFraction * 100).toFixed(1)}% / ${(result.metrics.oceanAreaFraction * 100).toFixed(1)}%`);
    metric(metrics, 'Duration', `${result.stage.durationMs.toFixed(1)} ms`);
}
async function generatePlanet() {
    generate.disabled = true;
    status.textContent = 'Generating one physical planet through WG-5 coupled climate in Rust/WASM…';
    try {
        const loaded = await client.generateClimate({ seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) });
        current = loaded;
        buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };
        styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
        showMetrics(loaded);
        redraw(false);
        updateAnimation();
        status.textContent = `Planet ready through WG-5: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.metrics.spinupYears} climate spin-up years, ${loaded.metrics.meanTemperatureK.toFixed(1)} K global mean.`;
    }
    catch (error) {
        status.textContent = error instanceof Error ? error.message : String(error);
    }
    finally {
        generate.disabled = false;
    }
}
generate.addEventListener('click', () => void generatePlanet());
projection.addEventListener('change', () => redraw(false));
visualization.addEventListener('change', () => { styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; redraw(false); updateAnimation(); });
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
void generatePlanet();
