import { createWorldgenClient } from '../worldgenClient.js';
import {
  WORLDGEN_BOUNDARY_CONVERGENT,
  WORLDGEN_BOUNDARY_DIVERGENT,
  WORLDGEN_BOUNDARY_TRANSFORM,
  WORLDGEN_CRUST_CONTINENTAL,
  WORLDGEN_CRUST_OCEANIC,
  WORLDGEN_CRUST_TRANSITIONAL,
  WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION,
  WORLDGEN_GEOLOGY_CONTINENTAL_RIFT,
  WORLDGEN_GEOLOGY_OCEANIC_RIDGE,
  WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION,
  WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION,
  WORLDGEN_GEOLOGY_TRANSFORM,
  WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE,
  WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN,
  WORLDGEN_STRUCTURE_NONE,
  WORLDGEN_STRUCTURE_RIFT,
  WORLDGEN_STRUCTURE_SUTURE,
  WORLDGEN_STRUCTURE_TRANSFORM,
  type WorldgenInheritanceResult,
} from '../protocol.js';

type ScalarDescriptor = { values: Float32Array; minimum: number; maximum: number; lowHue: number; highHue: number };
type ProjectionBuffers = { x: Float32Array; y: Float32Array; visible: Uint8Array };
type DrawBucket = { color: string; indices: Uint32Array };
type StyleCache = {
  result: WorldgenInheritanceResult | null;
  mode: string;
  sampleBuckets: DrawBucket[];
  boundaryBuckets: DrawBucket[];
};

const SCALAR_PALETTE_STEPS = 256;

function element<T extends HTMLElement>(id: string): T {
  const target = document.getElementById(id);
  if (!target) throw new Error(`Worldgen Lab is missing #${id}.`);
  return target as T;
}
function metric(container: HTMLElement, label: string, value: string): void {
  const item = document.createElement('div');
  const key = document.createElement('strong');
  const detail = document.createElement('span');
  key.textContent = label;
  detail.textContent = value;
  item.append(key, detail);
  container.appendChild(item);
}
function plateColor(plate: number): string { return `hsl(${(plate * 137.507764 + 18) % 360} 60% 55%)`; }
function provenanceColor(source: number): string { return `hsl(${(source * 137.507764 + 42) % 360} 58% 54%)`; }
function crustColor(kind: number): string {
  if (kind === WORLDGEN_CRUST_CONTINENTAL) return '#b79a72';
  if (kind === WORLDGEN_CRUST_TRANSITIONAL) return '#9aab87';
  if (kind === WORLDGEN_CRUST_OCEANIC) return '#477aa3';
  return '#d7e2ef';
}
function structuralColor(kind: number): string {
  if (kind === WORLDGEN_STRUCTURE_SUTURE) return '#ff7466';
  if (kind === WORLDGEN_STRUCTURE_RIFT) return '#ffb45d';
  if (kind === WORLDGEN_STRUCTURE_TRANSFORM) return '#c690ff';
  if (kind === WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN) return '#65d7ac';
  if (kind === WORLDGEN_STRUCTURE_NONE) return '#425362';
  return '#d7e2ef';
}
function tectonicBoundaryColor(kind: number): string {
  if (kind === WORLDGEN_BOUNDARY_CONVERGENT) return '#ff7272';
  if (kind === WORLDGEN_BOUNDARY_DIVERGENT) return '#64d7ff';
  if (kind === WORLDGEN_BOUNDARY_TRANSFORM) return '#ffd36a';
  return '#d7e2ef';
}
function geologicalBoundaryColor(regime: number): string {
  if (regime === WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION) return '#5a8fff';
  if (regime === WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION) return '#8a70ff';
  if (regime === WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION) return '#ff6969';
  if (regime === WORLDGEN_GEOLOGY_OCEANIC_RIDGE) return '#4ee8df';
  if (regime === WORLDGEN_GEOLOGY_CONTINENTAL_RIFT) return '#ffb65c';
  if (regime === WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE) return '#e8cf66';
  if (regime === WORLDGEN_GEOLOGY_TRANSFORM) return '#d59cff';
  return '#d7e2ef';
}
function scalarColorFromT(t: number, lowHue: number, highHue: number): string {
  const hue = lowHue + (highHue - lowHue) * t;
  return `hsl(${hue} 68% ${38 + t * 22}%)`;
}
function scalar(result: WorldgenInheritanceResult, mode: string): ScalarDescriptor | null {
  switch (mode) {
    case 'crust-age': return { values: result.crustAgeMyr, minimum: 0, maximum: 3500, lowHue: 205, highHue: 24 };
    case 'crust-thickness': return { values: result.crustThicknessKm, minimum: 5, maximum: 56, lowHue: 205, highHue: 350 };
    case 'strength': return { values: result.strengthIndex, minimum: 0, maximum: 1, lowHue: 0, highHue: 135 };
    case 'weakness': return { values: result.weaknessIndex, minimum: 0, maximum: 1, lowHue: 205, highHue: 15 };
    case 'dynamic-support': return { values: result.mantleDynamicSupportIndex, minimum: -1, maximum: 1, lowHue: 245, highHue: 25 };
    case 'fragmentation': return { values: result.fragmentationPropensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 0 };
    case 'orogeny': return { values: result.orogenicHistory, minimum: 0, maximum: 1, lowHue: 50, highHue: 350 };
    case 'ridge': return { values: result.ridgeHistory, minimum: 0, maximum: 1, lowHue: 225, highHue: 170 };
    case 'trench': return { values: result.trenchHistory, minimum: 0, maximum: 1, lowHue: 200, highHue: 260 };
    default: return null;
  }
}

function bucketize(count: number, colorAt: (index: number) => string): DrawBucket[] {
  const buckets = new Map<string, number[]>();
  for (let index = 0; index < count; index += 1) {
    const color = colorAt(index);
    const indices = buckets.get(color);
    if (indices) indices.push(index);
    else buckets.set(color, [index]);
  }
  return Array.from(buckets, ([color, indices]) => ({ color, indices: Uint32Array.from(indices) }));
}

function sampleColor(result: WorldgenInheritanceResult, mode: string, sample: number, scalarDescriptor: ScalarDescriptor | null): string {
  if (mode === 'plates' || mode === 'tectonic-boundaries' || mode === 'geological-boundaries' || mode === 'boundary-provenance') return plateColor(result.plateIds[sample]!);
  if (mode === 'kinematic-domains') return plateColor(result.kinematicDomainIds[sample]!);
  if (mode === 'crust-type') return crustColor(result.crustKind[sample]!);
  if (mode === 'structural-zones') return structuralColor(result.structuralZoneKind[sample]!);
  if (mode === 'provenance') return provenanceColor(result.nearestCoarseSource[sample]!);
  if (mode === 'inherited-mask') return result.inheritedSampleMask[sample] ? '#f4e27a' : '#5794c8';
  if (scalarDescriptor) {
    const span = Math.max(1e-12, scalarDescriptor.maximum - scalarDescriptor.minimum);
    const raw = Math.max(0, Math.min(1, (scalarDescriptor.values[sample]! - scalarDescriptor.minimum) / span));
    const quantized = Math.round(raw * (SCALAR_PALETTE_STEPS - 1)) / (SCALAR_PALETTE_STEPS - 1);
    return scalarColorFromT(quantized, scalarDescriptor.lowHue, scalarDescriptor.highHue);
  }
  return '#8297aa';
}

function buildStyleCache(result: WorldgenInheritanceResult, mode: string): StyleCache {
  const scalarDescriptor = scalar(result, mode);
  const sampleBuckets = bucketize(result.metrics.fineSampleCount, sample => sampleColor(result, mode, sample, scalarDescriptor));
  let boundaryBuckets: DrawBucket[] = [];
  if (mode === 'tectonic-boundaries') {
    boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => tectonicBoundaryColor(result.boundaryKinds[boundary]!));
  } else if (mode === 'geological-boundaries') {
    boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => geologicalBoundaryColor(result.geologicalBoundaryRegimes[boundary]!));
  } else if (mode === 'boundary-provenance') {
    boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => provenanceColor(result.boundaryCoarseSourceIndices[boundary]!));
  }
  return { result, mode, sampleBuckets, boundaryBuckets };
}

let projectionBuffers: ProjectionBuffers = { x: new Float32Array(0), y: new Float32Array(0), visible: new Uint8Array(0) };
let styleCache: StyleCache = { result: null, mode: '', sampleBuckets: [], boundaryBuckets: [] };

function ensureProjectionBuffers(sampleCount: number): ProjectionBuffers {
  if (projectionBuffers.x.length !== sampleCount) {
    projectionBuffers = {
      x: new Float32Array(sampleCount),
      y: new Float32Array(sampleCount),
      visible: new Uint8Array(sampleCount),
    };
  }
  return projectionBuffers;
}

function projectSamples(result: WorldgenInheritanceResult, projection: string, yaw: number, pitch: number, width: number, height: number): ProjectionBuffers {
  const buffers = ensureProjectionBuffers(result.metrics.fineSampleCount);
  const positions = result.positions;
  if (projection === 'map') {
    for (let sample = 0, offset = 0; sample < result.metrics.fineSampleCount; sample += 1, offset += 3) {
      const px = positions[offset]!;
      const py = positions[offset + 1]!;
      const pz = positions[offset + 2]!;
      const lon = Math.atan2(py, px);
      const lat = Math.asin(Math.max(-1, Math.min(1, pz)));
      buffers.x[sample] = (lon + Math.PI) / (2 * Math.PI) * width;
      buffers.y[sample] = (Math.PI / 2 - lat) / Math.PI * height;
      buffers.visible[sample] = 1;
    }
    return buffers;
  }

  const cosYaw = Math.cos(yaw);
  const sinYaw = Math.sin(yaw);
  const cosPitch = Math.cos(pitch);
  const sinPitch = Math.sin(pitch);
  const radius = Math.min(width, height) * 0.44;
  const centerX = width / 2;
  const centerY = height / 2;
  for (let sample = 0, offset = 0; sample < result.metrics.fineSampleCount; sample += 1, offset += 3) {
    const px = positions[offset]!;
    const py = positions[offset + 1]!;
    const pz = positions[offset + 2]!;
    const x1 = cosYaw * px - sinYaw * py;
    const y1 = sinYaw * px + cosYaw * py;
    const rotatedX = cosPitch * x1 + sinPitch * pz;
    const rotatedZ = -sinPitch * x1 + cosPitch * pz;
    buffers.x[sample] = centerX + y1 * radius;
    buffers.y[sample] = centerY - rotatedZ * radius;
    buffers.visible[sample] = rotatedX >= 0 ? 1 : 0;
  }
  return buffers;
}

function renderPlanet(canvas: HTMLCanvasElement, result: WorldgenInheritanceResult, projection: string, mode: string, yaw: number, pitch: number, interactive: boolean): void {
  const width = 1100;
  const height = projection === 'map' ? 550 : 760;
  if (canvas.width !== width) canvas.width = width;
  if (canvas.height !== height) canvas.height = height;
  const context = canvas.getContext('2d');
  if (!context) throw new Error('Worldgen Lab could not acquire a 2D canvas context.');
  context.fillStyle = '#08101a';
  context.fillRect(0, 0, width, height);

  const projected = projectSamples(result, projection, yaw, pitch, width, height);
  const x = projected.x;
  const y = projected.y;
  const visible = projected.visible;

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
      if (!visible[sample]) continue;
      const ax = x[sample]!;
      const ay = y[sample]!;
      for (let cursor = result.neighborOffsets[sample]!; cursor < result.neighborOffsets[sample + 1]!; cursor += 1) {
        const neighbor = result.neighbors[cursor]!;
        if (neighbor <= sample || !visible[neighbor]) continue;
        const bx = x[neighbor]!;
        if (projection === 'map' && Math.abs(ax - bx) > width / 2) continue;
        context.moveTo(ax, ay);
        context.lineTo(bx, y[neighbor]!);
      }
    }
    context.stroke();
    return;
  }

  if (styleCache.result !== result || styleCache.mode !== mode) styleCache = buildStyleCache(result, mode);
  const pointRadius = result.metrics.fineSampleCount > 100_000 ? 0.8 : result.metrics.fineSampleCount > 30_000 ? 1.15 : result.metrics.fineSampleCount > 5_000 ? 2.0 : 3.0;
  const boundaryMode = mode === 'tectonic-boundaries' || mode === 'geological-boundaries' || mode === 'boundary-provenance';
  context.globalAlpha = boundaryMode ? 0.28 : 0.94;

  const fastPoints = interactive && result.metrics.fineSampleCount > 20_000;
  if (fastPoints) {
    const size = Math.max(1.5, pointRadius * 1.7);
    const half = size / 2;
    for (const bucket of styleCache.sampleBuckets) {
      context.fillStyle = bucket.color;
      for (let cursor = 0; cursor < bucket.indices.length; cursor += 1) {
        const sample = bucket.indices[cursor]!;
        if (!visible[sample]) continue;
        context.fillRect(x[sample]! - half, y[sample]! - half, size, size);
      }
    }
  } else {
    for (const bucket of styleCache.sampleBuckets) {
      context.fillStyle = bucket.color;
      context.beginPath();
      for (let cursor = 0; cursor < bucket.indices.length; cursor += 1) {
        const sample = bucket.indices[cursor]!;
        if (!visible[sample]) continue;
        const sx = x[sample]!;
        const sy = y[sample]!;
        context.moveTo(sx + pointRadius, sy);
        context.arc(sx, sy, pointRadius, 0, Math.PI * 2);
      }
      context.fill();
    }
  }
  context.globalAlpha = 1;

  if (boundaryMode) {
    context.lineCap = 'round';
    context.lineWidth = mode === 'boundary-provenance' ? 1.4 : 2.0;
    for (const bucket of styleCache.boundaryBuckets) {
      context.strokeStyle = bucket.color;
      context.beginPath();
      for (let cursor = 0; cursor < bucket.indices.length; cursor += 1) {
        const boundary = bucket.indices[cursor]!;
        const sampleA = result.boundarySamples[boundary * 2]!;
        const sampleB = result.boundarySamples[boundary * 2 + 1]!;
        if (!visible[sampleA] || !visible[sampleB]) continue;
        const ax = x[sampleA]!;
        const bx = x[sampleB]!;
        if (projection === 'map' && Math.abs(ax - bx) > width / 2) continue;
        context.moveTo(ax, y[sampleA]!);
        context.lineTo(bx, y[sampleB]!);
      }
      context.stroke();
    }
  }
}

const seed = element<HTMLInputElement>('worldgen-seed');
const coarseLevel = element<HTMLInputElement>('worldgen-coarse-level');
const fineLevel = element<HTMLInputElement>('worldgen-level');
const plates = element<HTMLInputElement>('worldgen-plates');
const projection = element<HTMLSelectElement>('worldgen-projection');
const visualization = element<HTMLSelectElement>('worldgen-visualization');
const generate = element<HTMLButtonElement>('worldgen-generate');
const status = element<HTMLElement>('worldgen-status');
const metrics = element<HTMLElement>('worldgen-metrics');
const canvas = element<HTMLCanvasElement>('worldgen-field');
const client = createWorldgenClient();
let current: WorldgenInheritanceResult | null = null;
let yaw = -0.65;
let pitch = 0.25;
let drag: { x: number; y: number; yaw: number; pitch: number } | null = null;
let frameRequest: number | null = null;

function redraw(): void {
  if (!current || frameRequest !== null) return;
  frameRequest = requestAnimationFrame(() => {
    frameRequest = null;
    if (!current) return;
    renderPlanet(canvas, current, projection.value, visualization.value, yaw, pitch, drag !== null);
  });
}
function showMetrics(result: WorldgenInheritanceResult): void {
  metrics.replaceChildren();
  metric(metrics, 'Engine / stage', `v${result.engineVersion} · ${result.stage.id}@${result.stage.version}`);
  metric(metrics, 'Resolution', `L${result.coarseLevel} → L${result.fineLevel}`);
  metric(metrics, 'Samples', `${result.metrics.coarseSampleCount.toLocaleString()} → ${result.metrics.fineSampleCount.toLocaleString()} (+${result.metrics.addedSampleCount.toLocaleString()})`);
  metric(metrics, 'Fine boundaries', result.metrics.fineBoundaryEdgeCount.toLocaleString());
  metric(metrics, 'Inheritance hash', result.metrics.inheritanceHash);
  metric(metrics, 'Boundary hash', result.metrics.boundaryHash);
  metric(metrics, 'Provenance hash', result.metrics.provenanceHash);
  metric(metrics, 'Parameter hash', result.metrics.parameterHash);
  metric(metrics, 'Upstream hashes', `T ${result.metrics.tectonicHash} · G ${result.metrics.geologyHash} · L ${result.metrics.lithosphereHash}`);
  metric(metrics, 'Water inventory', `${(result.parameters.surfaceWaterMassKg / 1e21).toFixed(3)} ×10²¹ kg · ${result.parameters.equivalentGlobalWaterDepthM.toFixed(1)} m global equivalent`);
  metric(metrics, 'Interior forcing', `${result.parameters.internalHeatFluxWPerM2.toFixed(4)} W/m² · mantle ρ ${result.parameters.isostaticMantleDensityKgPerM3.toFixed(0)} kg/m³`);
  metric(metrics, 'Duration', `${result.stage.durationMs.toFixed(1)} ms`);
}
async function generatePlanet(): Promise<void> {
  generate.disabled = true;
  status.textContent = 'Generating accepted coarse physics, inheriting it to the fine topology, and reconstructing fine boundary interfaces in Rust/WASM…';
  try {
    const loaded = await client.generateInheritance({ seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) });
    current = loaded;
    styleCache = { result: null, mode: '', sampleBuckets: [], boundaryBuckets: [] };
    showMetrics(loaded);
    redraw();
    status.textContent = `WG-3.75 ready: L${loaded.coarseLevel} accepted physics inherited to L${loaded.fineLevel}; ${loaded.metrics.fineBoundaryEdgeCount.toLocaleString()} fine boundary interfaces.`;
  } catch (error) {
    status.textContent = error instanceof Error ? error.message : String(error);
  } finally {
    generate.disabled = false;
  }
}

generate.addEventListener('click', () => void generatePlanet());
projection.addEventListener('change', redraw);
visualization.addEventListener('change', redraw);
canvas.addEventListener('pointerdown', event => {
  if (projection.value !== 'globe') return;
  drag = { x: event.clientX, y: event.clientY, yaw, pitch };
  canvas.setPointerCapture(event.pointerId);
});
canvas.addEventListener('pointermove', event => {
  if (!drag || projection.value !== 'globe') return;
  yaw = drag.yaw + (event.clientX - drag.x) * 0.007;
  pitch = Math.max(-1.45, Math.min(1.45, drag.pitch + (event.clientY - drag.y) * 0.007));
  redraw();
});
canvas.addEventListener('pointerup', event => {
  drag = null;
  redraw();
  if (canvas.hasPointerCapture(event.pointerId)) canvas.releasePointerCapture(event.pointerId);
});
canvas.addEventListener('pointercancel', () => { drag = null; redraw(); });
window.addEventListener('beforeunload', () => {
  if (frameRequest !== null) cancelAnimationFrame(frameRequest);
  client.dispose();
});
void generatePlanet();
