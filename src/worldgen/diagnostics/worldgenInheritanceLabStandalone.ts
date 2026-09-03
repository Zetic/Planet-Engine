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

type Vec3 = [number, number, number];
type ScalarDescriptor = { values: Float32Array; minimum: number; maximum: number; lowHue: number; highHue: number };

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
function rotate(position: readonly number[], yaw: number, pitch: number): Vec3 {
  const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch);
  const x1 = cy * position[0]! - sy * position[1]!;
  const y1 = sy * position[0]! + cy * position[1]!;
  return [cp * x1 + sp * position[2]!, y1, -sp * x1 + cp * position[2]!];
}
function samplePosition(result: WorldgenInheritanceResult, sample: number): Vec3 {
  const offset = sample * 3;
  return [result.positions[offset]!, result.positions[offset + 1]!, result.positions[offset + 2]!];
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
function scalarColor(value: number, minimum: number, maximum: number, lowHue: number, highHue: number): string {
  const span = Math.max(1e-12, maximum - minimum);
  const t = Math.max(0, Math.min(1, (value - minimum) / span));
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

function renderPlanet(canvas: HTMLCanvasElement, result: WorldgenInheritanceResult, projection: string, mode: string, yaw: number, pitch: number): void {
  const width = 1100;
  const height = projection === 'map' ? 550 : 760;
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext('2d');
  if (!context) throw new Error('Worldgen Lab could not acquire a 2D canvas context.');
  context.fillStyle = '#08101a';
  context.fillRect(0, 0, width, height);

  function projectVector(raw: readonly number[]): [number, number, boolean] {
    if (projection === 'map') {
      const lon = Math.atan2(raw[1]!, raw[0]!);
      const lat = Math.asin(Math.max(-1, Math.min(1, raw[2]!)));
      return [(lon + Math.PI) / (2 * Math.PI) * width, (Math.PI / 2 - lat) / Math.PI * height, true];
    }
    const position = rotate(raw, yaw, pitch);
    const radius = Math.min(width, height) * 0.44;
    return [width / 2 + position[1] * radius, height / 2 - position[2] * radius, position[0] >= 0];
  }
  function projectSample(sample: number): [number, number, boolean] { return projectVector(samplePosition(result, sample)); }

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
      const [ax, ay, av] = projectSample(sample);
      if (!av) continue;
      for (let cursor = result.neighborOffsets[sample]!; cursor < result.neighborOffsets[sample + 1]!; cursor += 1) {
        const neighbor = result.neighbors[cursor]!;
        if (neighbor <= sample) continue;
        const [bx, by, bv] = projectSample(neighbor);
        if (!bv || (projection === 'map' && Math.abs(ax - bx) > width / 2)) continue;
        context.moveTo(ax, ay); context.lineTo(bx, by);
      }
    }
    context.stroke();
    return;
  }

  const pointRadius = result.metrics.fineSampleCount > 100_000 ? 0.8 : result.metrics.fineSampleCount > 30_000 ? 1.15 : result.metrics.fineSampleCount > 5_000 ? 2.0 : 3.0;
  const boundaryMode = mode === 'tectonic-boundaries' || mode === 'geological-boundaries' || mode === 'boundary-provenance';
  const scalarDescriptor = scalar(result, mode);
  context.globalAlpha = boundaryMode ? 0.28 : 0.94;
  for (let sample = 0; sample < result.metrics.fineSampleCount; sample += 1) {
    const [x, y, visible] = projectSample(sample);
    if (!visible) continue;
    if (mode === 'plates' || boundaryMode) context.fillStyle = plateColor(result.plateIds[sample]!);
    else if (mode === 'kinematic-domains') context.fillStyle = plateColor(result.kinematicDomainIds[sample]!);
    else if (mode === 'crust-type') context.fillStyle = crustColor(result.crustKind[sample]!);
    else if (mode === 'structural-zones') context.fillStyle = structuralColor(result.structuralZoneKind[sample]!);
    else if (mode === 'provenance') context.fillStyle = provenanceColor(result.nearestCoarseSource[sample]!);
    else if (mode === 'inherited-mask') context.fillStyle = result.inheritedSampleMask[sample] ? '#f4e27a' : '#5794c8';
    else if (scalarDescriptor) context.fillStyle = scalarColor(scalarDescriptor.values[sample]!, scalarDescriptor.minimum, scalarDescriptor.maximum, scalarDescriptor.lowHue, scalarDescriptor.highHue);
    else context.fillStyle = '#8297aa';
    context.beginPath(); context.arc(x, y, pointRadius, 0, Math.PI * 2); context.fill();
  }
  context.globalAlpha = 1;

  if (boundaryMode) {
    context.lineCap = 'round';
    for (let boundary = 0; boundary < result.metrics.fineBoundaryEdgeCount; boundary += 1) {
      const sampleA = result.boundarySamples[boundary * 2]!;
      const sampleB = result.boundarySamples[boundary * 2 + 1]!;
      const [ax, ay, av] = projectSample(sampleA);
      const [bx, by, bv] = projectSample(sampleB);
      if (!av || !bv || (projection === 'map' && Math.abs(ax - bx) > width / 2)) continue;
      if (mode === 'tectonic-boundaries') context.strokeStyle = tectonicBoundaryColor(result.boundaryKinds[boundary]!);
      else if (mode === 'geological-boundaries') context.strokeStyle = geologicalBoundaryColor(result.geologicalBoundaryRegimes[boundary]!);
      else context.strokeStyle = provenanceColor(result.boundaryCoarseSourceIndices[boundary]!);
      context.lineWidth = mode === 'boundary-provenance' ? 1.4 : 2.0;
      context.beginPath(); context.moveTo(ax, ay); context.lineTo(bx, by); context.stroke();
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

function redraw(): void {
  if (!current) return;
  renderPlanet(canvas, current, projection.value, visualization.value, yaw, pitch);
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
canvas.addEventListener('pointerdown', event => { if (projection.value !== 'globe') return; drag = { x: event.clientX, y: event.clientY, yaw, pitch }; canvas.setPointerCapture(event.pointerId); });
canvas.addEventListener('pointermove', event => { if (!drag || projection.value !== 'globe') return; yaw = drag.yaw + (event.clientX - drag.x) * 0.007; pitch = Math.max(-1.45, Math.min(1.45, drag.pitch + (event.clientY - drag.y) * 0.007)); redraw(); });
canvas.addEventListener('pointerup', event => { drag = null; if (canvas.hasPointerCapture(event.pointerId)) canvas.releasePointerCapture(event.pointerId); });
canvas.addEventListener('pointercancel', () => { drag = null; });
window.addEventListener('beforeunload', () => client.dispose());
void generatePlanet();
