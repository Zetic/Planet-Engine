import { createWorldgenClient } from '../worldgenClient.js';
import { WORLDGEN_BOUNDARY_CONVERGENT, WORLDGEN_BOUNDARY_DIVERGENT, WORLDGEN_BOUNDARY_TRANSFORM, WORLDGEN_CRUST_CONTINENTAL, WORLDGEN_CRUST_OCEANIC, WORLDGEN_CRUST_TRANSITIONAL, WORLDGEN_FRAGMENT_MICROPLATE, WORLDGEN_FRAGMENT_TERRANE, WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION, WORLDGEN_GEOLOGY_CONTINENTAL_RIFT, WORLDGEN_GEOLOGY_OCEANIC_RIDGE, WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION, WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION, WORLDGEN_GEOLOGY_TRANSFORM, WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE, WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN, WORLDGEN_STRUCTURE_NONE, WORLDGEN_STRUCTURE_RIFT, WORLDGEN_STRUCTURE_SUTURE, WORLDGEN_STRUCTURE_TRANSFORM, } from '../protocol.js';
function element(id) { const target = document.getElementById(id); if (!target)
    throw new Error(`Worldgen Lab is missing #${id}.`); return target; }
function metric(container, label, value) { const item = document.createElement('div'); const key = document.createElement('strong'); key.textContent = label; const detail = document.createElement('span'); detail.textContent = value; item.append(key, detail); container.appendChild(item); }
function rotate(position, yaw, pitch) { const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch); const x1 = cy * position[0] - sy * position[1]; const y1 = sy * position[0] + cy * position[1]; return [cp * x1 + sp * position[2], y1, -sp * x1 + cp * position[2]]; }
function cross(a, b) { return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]; }
function add(a, b, factor = 1) { return [a[0] + b[0] * factor, a[1] + b[1] * factor, a[2] + b[2] * factor]; }
function normalize(value) { const magnitude = Math.hypot(value[0], value[1], value[2]); return magnitude > 0 ? [value[0] / magnitude, value[1] / magnitude, value[2] / magnitude] : [0, 0, 0]; }
function samplePosition(result, sample) { const offset = sample * 3; return [result.positions[offset], result.positions[offset + 1], result.positions[offset + 2]]; }
function plateVector(values, plate) { const offset = plate * 3; return [values[offset], values[offset + 1], values[offset + 2]]; }
function plateColor(plate) { return `hsl(${(plate * 137.507764 + 18) % 360} 60% 55%)`; }
function tectonicBoundaryColor(kind) { if (kind === WORLDGEN_BOUNDARY_CONVERGENT)
    return '#ff7272'; if (kind === WORLDGEN_BOUNDARY_DIVERGENT)
    return '#64d7ff'; if (kind === WORLDGEN_BOUNDARY_TRANSFORM)
    return '#ffd36a'; return '#d7e2ef'; }
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
function crustColor(kind) { if (kind === WORLDGEN_CRUST_CONTINENTAL)
    return '#b79a72'; if (kind === WORLDGEN_CRUST_TRANSITIONAL)
    return '#9aab87'; if (kind === WORLDGEN_CRUST_OCEANIC)
    return '#477aa3'; return '#d7e2ef'; }
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
function fragmentColor(result, sample) {
    const fragmentId = result.fragmentIds[sample];
    if (fragmentId === 0)
        return '#263541';
    const kind = result.fragmentKinds[fragmentId - 1];
    const hue = (fragmentId * 137.507764 + (kind === WORLDGEN_FRAGMENT_MICROPLATE ? 350 : 55)) % 360;
    const saturation = kind === WORLDGEN_FRAGMENT_MICROPLATE ? 78 : 55;
    const lightness = kind === WORLDGEN_FRAGMENT_TERRANE ? 48 : 58;
    return `hsl(${hue} ${saturation}% ${lightness}%)`;
}
function scalarColor(value, minimum, maximum, lowHue, highHue) {
    const span = Math.max(1e-12, maximum - minimum);
    const t = Math.max(0, Math.min(1, (value - minimum) / span));
    const hue = lowHue + (highHue - lowHue) * t;
    return `hsl(${hue} 68% ${38 + t * 22}%)`;
}
function geologyScalar(result, geology, mode) {
    switch (mode) {
        case 'orogeny': return { values: result.orogenicHistory, minimum: 0, maximum: 1, lowHue: 50, highHue: 350 };
        case 'rift': return { values: result.riftHistory, minimum: 0, maximum: 1, lowHue: 210, highHue: 26 };
        case 'ridge': return { values: result.ridgeHistory, minimum: 0, maximum: 1, lowHue: 225, highHue: 170 };
        case 'subduction': return { values: result.subductionHistory, minimum: 0, maximum: 1, lowHue: 210, highHue: 285 };
        case 'transform-history': return { values: result.transformHistory, minimum: 0, maximum: 1, lowHue: 210, highHue: 300 };
        case 'strain': return { values: result.crustalStrain, minimum: 0, maximum: 1, lowHue: 210, highHue: 0 };
    }
    if (!geology)
        return null;
    switch (mode) {
        case 'crust-age': return { values: geology.crustAgeMyr, minimum: 0, maximum: 3500, lowHue: 205, highHue: 24 };
        case 'crust-thickness': return { values: geology.crustThicknessKm, minimum: 5, maximum: 56, lowHue: 205, highHue: 350 };
        case 'crust-density': return { values: geology.crustDensityKgPerM3, minimum: 2670, maximum: 3010, lowHue: 48, highHue: 258 };
        case 'buoyancy': return { values: geology.buoyancyIndex, minimum: -1, maximum: 1, lowHue: 250, highHue: 42 };
        case 'trench': return { values: geology.trenchHistory, minimum: 0, maximum: 1, lowHue: 200, highHue: 260 };
        case 'arc': return { values: geology.volcanicArcHistory, minimum: 0, maximum: 1, lowHue: 50, highHue: 8 };
        case 'subsidence': return { values: geology.subsidenceHistory, minimum: 0, maximum: 1, lowHue: 45, highHue: 230 };
        case 'basin': return { values: geology.basinPotential, minimum: 0, maximum: 1, lowHue: 220, highHue: 90 };
        default: return null;
    }
}
function lithosphereScalar(result, mode) {
    switch (mode) {
        case 'strength': return { values: result.strengthIndex, minimum: 0, maximum: 1, lowHue: 0, highHue: 135 };
        case 'weakness': return { values: result.weaknessIndex, minimum: 0, maximum: 1, lowHue: 205, highHue: 15 };
        case 'elastic-thickness': return { values: result.effectiveElasticThicknessKm, minimum: 4, maximum: 86, lowHue: 205, highHue: 35 };
        case 'thermal-anomaly': return { values: result.thermalAnomalyIndex, minimum: -1, maximum: 1, lowHue: 225, highHue: 8 };
        case 'mantle-upwelling': return { values: result.mantleUpwellingIndex, minimum: 0, maximum: 1, lowHue: 230, highHue: 350 };
        case 'dynamic-support': return { values: result.mantleDynamicSupportIndex, minimum: -1, maximum: 1, lowHue: 245, highHue: 25 };
        case 'compensated-buoyancy': return { values: result.compensatedBuoyancyIndex, minimum: -1, maximum: 1, lowHue: 250, highHue: 42 };
        case 'structural-fabric': return { values: result.structuralFabricStrength, minimum: 0, maximum: 1, lowHue: 210, highHue: 330 };
        case 'fragmentation': return { values: result.fragmentationPropensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 0 };
        default: return null;
    }
}
function renderPlanet(canvas, result, geology, tectonics, projection, mode, yaw, pitch) {
    const width = 1100;
    const height = projection === 'map' ? 550 : 760;
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext('2d');
    if (!context)
        throw new Error('Worldgen Lab could not acquire a 2D canvas context.');
    context.fillStyle = '#08101a';
    context.fillRect(0, 0, width, height);
    function projectVector(raw) {
        if (projection === 'map') {
            const lon = Math.atan2(raw[1], raw[0]);
            const lat = Math.asin(Math.max(-1, Math.min(1, raw[2])));
            return [(lon + Math.PI) / (2 * Math.PI) * width, (Math.PI / 2 - lat) / Math.PI * height, true];
        }
        const position = rotate(raw, yaw, pitch);
        const radius = Math.min(width, height) * 0.44;
        return [width / 2 + position[1] * radius, height / 2 - position[2] * radius, position[0] >= 0];
    }
    function projectSample(sample) { return projectVector(samplePosition(result, sample)); }
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
        for (let sample = 0; sample < result.metrics.sampleCount; sample += 1) {
            const [ax, ay, av] = projectSample(sample);
            if (!av)
                continue;
            for (let cursor = result.neighborOffsets[sample]; cursor < result.neighborOffsets[sample + 1]; cursor += 1) {
                const neighbor = result.neighbors[cursor];
                if (neighbor <= sample)
                    continue;
                const [bx, by, bv] = projectSample(neighbor);
                if (!bv || (projection === 'map' && Math.abs(ax - bx) > width / 2))
                    continue;
                context.moveTo(ax, ay);
                context.lineTo(bx, by);
            }
        }
        context.stroke();
        return;
    }
    const pointRadius = result.metrics.sampleCount > 30_000 ? 1.15 : result.metrics.sampleCount > 5_000 ? 2.05 : 3.0;
    const categoricalBoundaryMode = mode === 'tectonic-boundaries' || mode === 'geological-boundaries';
    const scalar = lithosphereScalar(result, mode) ?? geologyScalar(result, geology, mode);
    context.globalAlpha = categoricalBoundaryMode || mode === 'motion' ? 0.30 : 0.94;
    for (let sample = 0; sample < result.metrics.sampleCount; sample += 1) {
        const [x, y, visible] = projectSample(sample);
        if (!visible)
            continue;
        if (mode === 'plates' || mode === 'motion' || mode === 'tectonic-boundaries')
            context.fillStyle = plateColor(result.plateIds[sample]);
        else if (mode === 'kinematic-domains')
            context.fillStyle = plateColor(result.kinematicDomainIds[sample]);
        else if (mode === 'fragments')
            context.fillStyle = fragmentColor(result, sample);
        else if (mode === 'crust-type' || mode === 'geological-boundaries')
            context.fillStyle = crustColor(result.crustKind[sample]);
        else if (mode === 'structural-zones')
            context.fillStyle = structuralColor(result.structuralZoneKind[sample]);
        else if (scalar)
            context.fillStyle = scalarColor(scalar.values[sample], scalar.minimum, scalar.maximum, scalar.lowHue, scalar.highHue);
        else
            context.fillStyle = '#8297aa';
        context.beginPath();
        context.arc(x, y, pointRadius, 0, Math.PI * 2);
        context.fill();
    }
    context.globalAlpha = 1;
    if (mode === 'tectonic-boundaries' || mode === 'geological-boundaries') {
        context.lineCap = 'round';
        for (let boundary = 0; boundary < result.boundaryEdgeCount; boundary += 1) {
            const sampleA = result.boundarySamples[boundary * 2];
            const sampleB = result.boundarySamples[boundary * 2 + 1];
            const [ax, ay, av] = projectSample(sampleA);
            const [bx, by, bv] = projectSample(sampleB);
            if (!av || !bv || (projection === 'map' && Math.abs(ax - bx) > width / 2))
                continue;
            context.strokeStyle = mode === 'tectonic-boundaries' ? tectonicBoundaryColor(result.boundaryKinds[boundary]) : geologicalBoundaryColor(result.geologicalBoundaryRegimes[boundary]);
            context.lineWidth = 1.8;
            context.beginPath();
            context.moveTo(ax, ay);
            context.lineTo(bx, by);
            context.stroke();
        }
    }
    if (mode === 'motion' && tectonics) {
        context.lineWidth = 1.5;
        for (let plate = 0; plate < tectonics.metrics.plateCount; plate += 1) {
            const seed = tectonics.plateSeedSamples[plate];
            const start = samplePosition(tectonics, seed);
            const omega = plateVector(tectonics.plateAngularVelocitiesRadPerMyr, plate);
            const tangentVelocity = normalize(cross(omega, start));
            const end = normalize(add(start, tangentVelocity, 0.13));
            const [ax, ay, av] = projectVector(start);
            const [bx, by, bv] = projectVector(end);
            if (!av || !bv || (projection === 'map' && Math.abs(ax - bx) > width / 2))
                continue;
            context.strokeStyle = '#f4f7fb';
            context.fillStyle = '#f4f7fb';
            context.beginPath();
            context.moveTo(ax, ay);
            context.lineTo(bx, by);
            context.stroke();
            const angle = Math.atan2(by - ay, bx - ax);
            context.beginPath();
            context.moveTo(bx, by);
            context.lineTo(bx - 6 * Math.cos(angle - 0.55), by - 6 * Math.sin(angle - 0.55));
            context.lineTo(bx - 6 * Math.cos(angle + 0.55), by - 6 * Math.sin(angle + 0.55));
            context.closePath();
            context.fill();
        }
    }
}
function renderMetrics(container, result) {
    container.replaceChildren();
    metric(container, 'Engine / stage', `v${result.engineVersion} · ${result.stage.id}@${result.stage.version}`);
    metric(container, 'Topology', `icosphere L${result.level} · ${result.metrics.sampleCount.toLocaleString()} samples`);
    metric(container, 'Macro plates', result.plateCount.toLocaleString());
    metric(container, 'Strength / weakness', `${result.metrics.meanStrengthIndex.toFixed(3)} / ${result.metrics.meanWeaknessIndex.toFixed(3)}`);
    metric(container, 'Mean elastic thickness', `${result.metrics.meanEffectiveElasticThicknessKm.toFixed(1)} km`);
    metric(container, 'Mantle upwelling / support', `${result.metrics.meanMantleUpwellingIndex.toFixed(3)} / ${result.metrics.meanDynamicSupportIndex.toFixed(3)}`);
    metric(container, 'Structural samples', `${result.metrics.sutureSampleCount.toLocaleString()} suture · ${result.metrics.riftZoneSampleCount.toLocaleString()} rift · ${result.metrics.transformZoneSampleCount.toLocaleString()} transform · ${result.metrics.continentalMarginSampleCount.toLocaleString()} margin`);
    metric(container, 'Tectonic refinement', `${result.metrics.microplateCount} microplates · ${result.metrics.terraneCount} terranes`);
    metric(container, 'Fragmented area', `${(result.metrics.fragmentedAreaFraction * 100).toFixed(2)}%`);
    metric(container, 'Topology hash', result.topologyHash);
    metric(container, 'Tectonic hash', result.metrics.tectonicHash);
    metric(container, 'Geology hash', result.metrics.geologyHash);
    metric(container, 'Lithosphere hash', result.metrics.lithosphereHash);
    metric(container, 'Rust + WASM', `${result.stage.durationMs.toFixed(2)} ms`);
}
const GEOLOGY_ONLY_MODES = new Set(['crust-age', 'crust-thickness', 'crust-density', 'buoyancy', 'trench', 'arc', 'subsidence', 'basin']);
function install() {
    const seed = element('worldgen-seed');
    const level = element('worldgen-level');
    const plates = element('worldgen-plates');
    const projection = element('worldgen-projection');
    const visualization = element('worldgen-visualization');
    const generate = element('worldgen-generate');
    const status = element('worldgen-status');
    const metrics = element('worldgen-metrics');
    const canvas = element('worldgen-field');
    const client = createWorldgenClient();
    let current = null;
    let currentGeology = null;
    let currentTectonics = null;
    let currentIdentity = '';
    let yaw = 0.0, pitch = 0.15, dragging = false, lastX = 0, lastY = 0;
    function identity() { return `${seed.value}\u0000${level.value}\u0000${plates.value}`; }
    function redraw() { if (current)
        renderPlanet(canvas, current, currentGeology, currentTectonics, projection.value, visualization.value, yaw, pitch); }
    async function ensureAuxiliaryData() {
        if (!current || currentIdentity !== identity())
            return;
        const requestedIdentity = currentIdentity;
        const request = { seed: seed.value, level: Number(level.value), plateCount: Number(plates.value) };
        if (visualization.value === 'motion' && !currentTectonics) {
            const loaded = await client.generateTectonics(request);
            if (requestedIdentity === currentIdentity && requestedIdentity === identity())
                currentTectonics = loaded;
        }
        if (GEOLOGY_ONLY_MODES.has(visualization.value) && !currentGeology) {
            const loaded = await client.generateGeology(request);
            if (requestedIdentity === currentIdentity && requestedIdentity === identity())
                currentGeology = loaded;
        }
        if (requestedIdentity === currentIdentity && requestedIdentity === identity())
            redraw();
    }
    async function run() {
        generate.disabled = true;
        status.className = 'worldgen-lab-status';
        status.textContent = 'Building lithospheric mechanics, structural fabric, mantle support, and tectonic refinement in Rust/WASM…';
        try {
            const requestedIdentity = identity();
            const loaded = await client.generateLithosphere({ seed: seed.value, level: Number(level.value), plateCount: Number(plates.value) });
            if (requestedIdentity !== identity())
                return;
            current = loaded;
            currentGeology = null;
            currentTectonics = null;
            currentIdentity = requestedIdentity;
            renderMetrics(metrics, current);
            redraw();
            await ensureAuxiliaryData();
            status.className = 'worldgen-lab-status worldgen-lab-status--ok';
            status.textContent = 'WG-3.5 lithospheric mechanics and selective tectonic refinement active. No topography is generated.';
        }
        catch (error) {
            status.className = 'worldgen-lab-status worldgen-lab-status--error';
            status.textContent = error instanceof Error ? error.message : String(error);
        }
        finally {
            generate.disabled = false;
        }
    }
    generate.addEventListener('click', () => void run());
    for (const input of [seed, level, plates])
        input.addEventListener('keydown', event => { if (event.key === 'Enter')
            void run(); });
    projection.addEventListener('change', redraw);
    visualization.addEventListener('change', () => { redraw(); void ensureAuxiliaryData(); });
    canvas.addEventListener('pointerdown', event => { if (projection.value !== 'globe')
        return; dragging = true; lastX = event.clientX; lastY = event.clientY; canvas.setPointerCapture(event.pointerId); });
    canvas.addEventListener('pointermove', event => { if (!dragging || projection.value !== 'globe')
        return; yaw += (event.clientX - lastX) * 0.01; pitch = Math.max(-Math.PI / 2, Math.min(Math.PI / 2, pitch + (event.clientY - lastY) * 0.01)); lastX = event.clientX; lastY = event.clientY; redraw(); });
    canvas.addEventListener('pointerup', () => { dragging = false; });
    canvas.addEventListener('pointercancel', () => { dragging = false; });
    window.addEventListener('beforeunload', () => client.dispose(), { once: true });
    void run();
}
document.addEventListener('DOMContentLoaded', install);
