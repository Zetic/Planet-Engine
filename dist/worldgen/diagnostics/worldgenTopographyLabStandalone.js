import { createWorldgenClient } from '../worldgenClient.js';
import { WORLDGEN_BOUNDARY_CONVERGENT, WORLDGEN_BOUNDARY_DIVERGENT, WORLDGEN_BOUNDARY_TRANSFORM, WORLDGEN_CRUST_CONTINENTAL, WORLDGEN_CRUST_OCEANIC, WORLDGEN_CRUST_TRANSITIONAL, WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION, WORLDGEN_GEOLOGY_CONTINENTAL_RIFT, WORLDGEN_GEOLOGY_OCEANIC_RIDGE, WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION, WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION, WORLDGEN_GEOLOGY_TRANSFORM, WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE, WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN, WORLDGEN_STRUCTURE_NONE, WORLDGEN_STRUCTURE_RIFT, WORLDGEN_STRUCTURE_SUTURE, WORLDGEN_STRUCTURE_TRANSFORM, } from '../protocol.js';
const PALETTE_STEPS = 256;
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
function scalarField(result, mode) {
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
        case 'crust-age': return { values: result.crustAgeMyr, minimum: 0, maximum: 3500, lowHue: 205, highHue: 24 };
        case 'crust-thickness': return { values: result.crustThicknessKm, minimum: 5, maximum: 56, lowHue: 205, highHue: 350 };
        case 'orogeny-history': return { values: result.orogenicHistory, minimum: 0, maximum: 1, lowHue: 50, highHue: 350 };
        case 'ridge-history': return { values: result.ridgeHistory, minimum: 0, maximum: 1, lowHue: 225, highHue: 170 };
        case 'trench-history': return { values: result.trenchHistory, minimum: 0, maximum: 1, lowHue: 200, highHue: 260 };
        case 'strength': return { values: result.strengthIndex, minimum: 0, maximum: 1, lowHue: 0, highHue: 135 };
        case 'weakness': return { values: result.weaknessIndex, minimum: 0, maximum: 1, lowHue: 205, highHue: 15 };
        case 'dynamic-support': return { values: result.mantleDynamicSupportIndex, minimum: -1, maximum: 1, lowHue: 245, highHue: 25 };
        case 'fragmentation': return { values: result.fragmentationPropensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 0 };
        default: return null;
    }
}
function scalarColor(value, field) {
    const t = Math.max(0, Math.min(1, (value - field.minimum) / Math.max(1e-12, field.maximum - field.minimum)));
    const quantized = Math.round(t * (PALETTE_STEPS - 1)) / (PALETTE_STEPS - 1);
    const hue = field.lowHue + (field.highHue - field.lowHue) * quantized;
    return `hsl(${hue} 68% ${38 + quantized * 22}%)`;
}
function bucketize(count, colorAt) {
    const buckets = new Map();
    for (let index = 0; index < count; index += 1) {
        const color = colorAt(index);
        const entries = buckets.get(color);
        if (entries)
            entries.push(index);
        else
            buckets.set(color, [index]);
    }
    return Array.from(buckets, ([color, indices]) => ({ color, indices: Uint32Array.from(indices) }));
}
function sampleColor(result, mode, sample, field) {
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
function buildStyleCache(result, mode) {
    const field = scalarField(result, mode);
    const sampleBuckets = mode === 'mesh' ? [] : bucketize(result.metrics.fineSampleCount, sample => sampleColor(result, mode, sample, field));
    let boundaryBuckets = [];
    if (mode === 'tectonic-boundaries')
        boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => tectonicBoundaryColor(result.boundaryKinds[boundary]));
    else if (mode === 'geological-boundaries')
        boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => geologicalBoundaryColor(result.geologicalBoundaryRegimes[boundary]));
    else if (mode === 'boundary-provenance')
        boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => provenanceColor(result.boundaryCoarseSourceIndices[boundary]));
    return { result, mode, sampleBuckets, boundaryBuckets };
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
            buffers.x[sample] = (Math.atan2(py, px) + Math.PI) / (2 * Math.PI) * width;
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
let styleCache = { result: null, mode: '', sampleBuckets: [], boundaryBuckets: [] };
function renderPlanet(canvas, result, projection, mode, yaw, pitch, buffers, interactive) {
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
        context.arc(width / 2, height / 2, Math.min(width, height) * 0.44, 0, Math.PI * 2);
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
            const ax = buffers.x[sample];
            const ay = buffers.y[sample];
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
    if (styleCache.result !== result || styleCache.mode !== mode)
        styleCache = buildStyleCache(result, mode);
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
            const x = buffers.x[sample];
            const y = buffers.y[sample];
            if (fastPoints)
                context.fillRect(x - 0.75, y - 0.75, 1.5, 1.5);
            else {
                context.moveTo(x + pointRadius, y);
                context.arc(x, y, pointRadius, 0, Math.PI * 2);
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
                const a = result.boundarySamples[boundary * 2];
                const b = result.boundarySamples[boundary * 2 + 1];
                if (!buffers.visible[a] || !buffers.visible[b])
                    continue;
                const ax = buffers.x[a];
                const bx = buffers.x[b];
                if (projection === 'map' && Math.abs(ax - bx) > width / 2)
                    continue;
                context.moveTo(ax, buffers.y[a]);
                context.lineTo(bx, buffers.y[b]);
            }
            context.stroke();
        }
    }
}
const seed = element('worldgen-seed');
const coarseLevel = element('worldgen-coarse-level');
const fineLevel = element('worldgen-level');
const plates = element('worldgen-plates');
const projection = element('worldgen-projection');
const visualization = element('worldgen-visualization');
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
function redraw(interactive = false) {
    if (!current || !buffers)
        return;
    renderPlanet(canvas, current, projection.value, visualization.value, yaw, pitch, buffers, interactive);
}
function scheduleRedraw(interactive) {
    if (frameRequest)
        return;
    frameRequest = requestAnimationFrame(() => { frameRequest = 0; redraw(interactive && drag !== null); });
}
function showMetrics(result) {
    metrics.replaceChildren();
    metric(metrics, 'Engine / stage', `v${result.engineVersion} · ${result.stage.id}@${result.stage.version}`);
    metric(metrics, 'Resolution', `L${result.coarseLevel} → L${result.fineLevel}`);
    metric(metrics, 'Samples', result.metrics.fineSampleCount.toLocaleString());
    metric(metrics, 'Topography hash', result.metrics.topographyHash);
    metric(metrics, 'Elevation', `${result.metrics.minimumSolidElevationM.toFixed(0)} m → ${result.metrics.maximumSolidElevationM.toFixed(0)} m`);
    metric(metrics, 'Hypsometry', `P05 ${result.metrics.p05SolidElevationM.toFixed(0)} · P50 ${result.metrics.medianSolidElevationM.toFixed(0)} · P95 ${result.metrics.p95SolidElevationM.toFixed(0)} m`);
    metric(metrics, 'Sea level', result.metrics.hasSeaLevel ? `${result.metrics.seaLevelM.toFixed(1)} m` : 'dry profile');
    metric(metrics, 'Land / ocean', `${(result.metrics.landAreaFraction * 100).toFixed(1)}% / ${(result.metrics.oceanAreaFraction * 100).toFixed(1)}%`);
    metric(metrics, 'Mean land / water', `${result.metrics.meanLandElevationM.toFixed(0)} m / ${result.metrics.meanWaterDepthM.toFixed(0)} m`);
    metric(metrics, 'Deepest water', `${result.metrics.maximumWaterDepthM.toFixed(0)} m`);
    metric(metrics, 'Water volume error', result.metrics.waterVolumeRelativeError.toExponential(2));
    metric(metrics, 'Planet thermal', `${result.parameters.internalHeatFluxWPerM2.toFixed(3)} W/m² · α ${result.parameters.mantleThermalExpansivityPerK.toExponential(2)} K⁻¹`);
    metric(metrics, 'Safety clamps', result.metrics.clampedSampleCount.toLocaleString());
    metric(metrics, 'Upstream', `I ${result.metrics.inheritanceHash} · B ${result.metrics.boundaryHash}`);
    metric(metrics, 'Duration', `${result.stage.durationMs.toFixed(1)} ms`);
}
async function generatePlanet() {
    generate.disabled = true;
    status.textContent = 'Generating one physical planet through WG-4 in Rust/WASM…';
    try {
        const loaded = await client.generateTopography({ seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) });
        current = loaded;
        buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };
        styleCache = { result: null, mode: '', sampleBuckets: [], boundaryBuckets: [] };
        showMetrics(loaded);
        redraw(false);
        status.textContent = `Planet ready through WG-4: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${(loaded.metrics.landAreaFraction * 100).toFixed(1)}% land, ${(loaded.metrics.oceanAreaFraction * 100).toFixed(1)}% ocean.`;
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
visualization.addEventListener('change', () => redraw(false));
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
window.addEventListener('beforeunload', () => { if (frameRequest)
    cancelAnimationFrame(frameRequest); client.dispose(); });
void generatePlanet();
