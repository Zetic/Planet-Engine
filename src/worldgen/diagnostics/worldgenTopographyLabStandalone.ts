import { createWorldgenClient } from '../worldgenClient.js';
import { type WorldgenTopographyResult } from '../protocol.js';

type ScalarField = { values: Float32Array; minimum: number; maximum: number; lowHue: number; highHue: number };

type ProjectionBuffers = {
  x: Float32Array;
  y: Float32Array;
  visible: Uint8Array;
};

function element<T extends HTMLElement>(id: string): T {
  const target = document.getElementById(id);
  if (!target) throw new Error(`Planet Engine Lab is missing #${id}.`);
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

function scalarField(result: WorldgenTopographyResult, mode: string): ScalarField | null {
  switch (mode) {
    case 'solid-elevation': return { values: result.solidElevationM, minimum: -12_000, maximum: 8_000, lowHue: 225, highHue: 25 };
    case 'relative-elevation': return { values: result.elevationAboveSeaLevelM, minimum: -10_000, maximum: 6_000, lowHue: 220, highHue: 35 };
    case 'water-depth': return { values: result.waterDepthM, minimum: 0, maximum: 10_000, lowHue: 195, highHue: 245 };
    case 'isostatic': return { values: result.isostaticElevationM, minimum: 0, maximum: 10_000, lowHue: 210, highHue: 25 };
    case 'thermal': return { values: result.thermalElevationM, minimum: -5_000, maximum: 0, lowHue: 260, highHue: 185 };
    case 'orogenic': return { values: result.orogenicElevationM, minimum: 0, maximum: 6_000, lowHue: 55, highHue: 350 };
    case 'ridge-relief': return { values: result.ridgeElevationM, minimum: 0, maximum: 3_000, lowHue: 220, highHue: 165 };
    case 'rift-basin': return { values: result.riftBasinElevationM, minimum: -3_500, maximum: 0, lowHue: 250, highHue: 35 };
    case 'trench-relief': return { values: result.trenchElevationM, minimum: -7_000, maximum: 0, lowHue: 285, highHue: 210 };
    case 'arc-relief': return { values: result.arcElevationM, minimum: 0, maximum: 3_000, lowHue: 50, highHue: 5 };
    case 'mantle-relief': return { values: result.mantleDynamicElevationM, minimum: -1_200, maximum: 1_200, lowHue: 245, highHue: 25 };
    default: return null;
  }
}

const PALETTE_STEPS = 256;
function makePalette(field: ScalarField): string[] {
  const palette = new Array<string>(PALETTE_STEPS);
  for (let index = 0; index < PALETTE_STEPS; index += 1) {
    const t = index / (PALETTE_STEPS - 1);
    const hue = field.lowHue + (field.highHue - field.lowHue) * t;
    palette[index] = `hsl(${hue} 68% ${38 + t * 22}%)`;
  }
  return palette;
}

function paletteIndex(value: number, minimum: number, maximum: number): number {
  const t = Math.max(0, Math.min(1, (value - minimum) / Math.max(1e-12, maximum - minimum)));
  return Math.min(PALETTE_STEPS - 1, Math.floor(t * (PALETTE_STEPS - 1)));
}

function projectSamples(
  result: WorldgenTopographyResult,
  projection: string,
  yaw: number,
  pitch: number,
  width: number,
  height: number,
  buffers: ProjectionBuffers,
): void {
  const count = result.metrics.fineSampleCount;
  const positions = result.positions;
  if (projection === 'map') {
    for (let sample = 0; sample < count; sample += 1) {
      const offset = sample * 3;
      const px = positions[offset]!;
      const py = positions[offset + 1]!;
      const pz = positions[offset + 2]!;
      const lon = Math.atan2(py, px);
      const lat = Math.asin(Math.max(-1, Math.min(1, pz)));
      buffers.x[sample] = (lon + Math.PI) / (2 * Math.PI) * width;
      buffers.y[sample] = (Math.PI / 2 - lat) / Math.PI * height;
      buffers.visible[sample] = 1;
    }
    return;
  }
  const cy = Math.cos(yaw);
  const sy = Math.sin(yaw);
  const cp = Math.cos(pitch);
  const sp = Math.sin(pitch);
  const radius = Math.min(width, height) * 0.44;
  for (let sample = 0; sample < count; sample += 1) {
    const offset = sample * 3;
    const px = positions[offset]!;
    const py = positions[offset + 1]!;
    const pz = positions[offset + 2]!;
    const x1 = cy * px - sy * py;
    const y1 = sy * px + cy * py;
    const rotatedX = cp * x1 + sp * pz;
    const rotatedZ = -sp * x1 + cp * pz;
    buffers.x[sample] = width / 2 + y1 * radius;
    buffers.y[sample] = height / 2 - rotatedZ * radius;
    buffers.visible[sample] = rotatedX >= 0 ? 1 : 0;
  }
}

let styleKey = '';
let samplePaletteIndices = new Uint16Array(0);
let cachedPalette: string[] = [];
function prepareStyles(result: WorldgenTopographyResult, mode: string): void {
  const key = `${result.metrics.topographyHash}:${mode}`;
  if (key === styleKey) return;
  styleKey = key;
  samplePaletteIndices = new Uint16Array(result.metrics.fineSampleCount);
  if (mode === 'land-water') {
    cachedPalette = ['#214d7a', '#a99b72'];
    for (let sample = 0; sample < samplePaletteIndices.length; sample += 1) samplePaletteIndices[sample] = result.submergedMask[sample] ? 0 : 1;
    return;
  }
  if (mode === 'plates') {
    cachedPalette = new Array<string>(result.metrics.plateCount);
    for (let plate = 0; plate < cachedPalette.length; plate += 1) cachedPalette[plate] = `hsl(${(plate * 137.507764 + 18) % 360} 60% 55%)`;
    for (let sample = 0; sample < samplePaletteIndices.length; sample += 1) samplePaletteIndices[sample] = result.plateIds[sample]!;
    return;
  }
  const field = scalarField(result, mode) ?? scalarField(result, 'relative-elevation')!;
  cachedPalette = makePalette(field);
  for (let sample = 0; sample < samplePaletteIndices.length; sample += 1) samplePaletteIndices[sample] = paletteIndex(field.values[sample]!, field.minimum, field.maximum);
}

function renderPlanet(
  canvas: HTMLCanvasElement,
  result: WorldgenTopographyResult,
  projection: string,
  mode: string,
  yaw: number,
  pitch: number,
  buffers: ProjectionBuffers,
  interactive: boolean,
): void {
  const width = 1100;
  const height = projection === 'map' ? 550 : 760;
  if (canvas.width !== width) canvas.width = width;
  if (canvas.height !== height) canvas.height = height;
  const context = canvas.getContext('2d');
  if (!context) throw new Error('Planet Engine Lab could not acquire a 2D canvas context.');
  context.fillStyle = '#08101a';
  context.fillRect(0, 0, width, height);
  projectSamples(result, projection, yaw, pitch, width, height, buffers);
  prepareStyles(result, mode);

  if (projection === 'globe') {
    context.beginPath();
    context.arc(width / 2, height / 2, Math.min(width, height) * 0.44, 0, Math.PI * 2);
    context.strokeStyle = '#5d7890';
    context.lineWidth = 1;
    context.stroke();
  }

  const count = result.metrics.fineSampleCount;
  const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
  const fastPoints = interactive && count > 20_000;
  for (let paletteIndexValue = 0; paletteIndexValue < cachedPalette.length; paletteIndexValue += 1) {
    context.fillStyle = cachedPalette[paletteIndexValue]!;
    if (!fastPoints) context.beginPath();
    for (let sample = 0; sample < count; sample += 1) {
      if (!buffers.visible[sample] || samplePaletteIndices[sample] !== paletteIndexValue) continue;
      const x = buffers.x[sample]!;
      const y = buffers.y[sample]!;
      if (fastPoints) context.fillRect(x - 0.75, y - 0.75, 1.5, 1.5);
      else { context.moveTo(x + pointRadius, y); context.arc(x, y, pointRadius, 0, Math.PI * 2); }
    }
    if (!fastPoints) context.fill();
  }

  if (mode === 'geological-boundaries') {
    context.strokeStyle = '#f3d46b';
    context.lineWidth = 1.2;
    context.beginPath();
    for (let boundary = 0; boundary < result.metrics.fineBoundaryEdgeCount; boundary += 1) {
      const a = result.boundarySamples[boundary * 2]!;
      const b = result.boundarySamples[boundary * 2 + 1]!;
      if (!buffers.visible[a] || !buffers.visible[b]) continue;
      const ax = buffers.x[a]!;
      const bx = buffers.x[b]!;
      if (projection === 'map' && Math.abs(ax - bx) > width / 2) continue;
      context.moveTo(ax, buffers.y[a]!);
      context.lineTo(bx, buffers.y[b]!);
    }
    context.stroke();
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
let current: WorldgenTopographyResult | null = null;
let buffers: ProjectionBuffers | null = null;
let yaw = -0.65;
let pitch = 0.25;
let drag: { x: number; y: number; yaw: number; pitch: number } | null = null;
let frameRequest = 0;

function redraw(interactive = false): void {
  if (!current || !buffers) return;
  renderPlanet(canvas, current, projection.value, visualization.value, yaw, pitch, buffers, interactive);
}
function scheduleRedraw(interactive: boolean): void {
  if (frameRequest) return;
  frameRequest = requestAnimationFrame(() => { frameRequest = 0; redraw(interactive && drag !== null); });
}
function showMetrics(result: WorldgenTopographyResult): void {
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
  metric(metrics, 'Safety clamps', result.metrics.clampedSampleCount.toLocaleString());
  metric(metrics, 'Upstream', `I ${result.metrics.inheritanceHash} · B ${result.metrics.boundaryHash}`);
  metric(metrics, 'Duration', `${result.stage.durationMs.toFixed(1)} ms`);
}
async function generatePlanet(): Promise<void> {
  generate.disabled = true;
  status.textContent = 'Generating inherited physics, tectonic relief, mechanically filtered solid topography, and the global water-volume solution in Rust/WASM…';
  try {
    const loaded = await client.generateTopography({ seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) });
    current = loaded;
    buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };
    styleKey = '';
    showMetrics(loaded);
    redraw(false);
    status.textContent = `WG-4 ready: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${(loaded.metrics.landAreaFraction * 100).toFixed(1)}% land, ${(loaded.metrics.oceanAreaFraction * 100).toFixed(1)}% ocean.`;
  } catch (error) {
    status.textContent = error instanceof Error ? error.message : String(error);
  } finally {
    generate.disabled = false;
  }
}

generate.addEventListener('click', () => void generatePlanet());
projection.addEventListener('change', () => { styleKey = ''; redraw(false); });
visualization.addEventListener('change', () => { styleKey = ''; redraw(false); });
canvas.addEventListener('pointerdown', event => {
  if (projection.value !== 'globe') return;
  drag = { x: event.clientX, y: event.clientY, yaw, pitch };
  canvas.setPointerCapture(event.pointerId);
});
canvas.addEventListener('pointermove', event => {
  if (!drag || projection.value !== 'globe') return;
  yaw = drag.yaw + (event.clientX - drag.x) * 0.007;
  pitch = Math.max(-1.45, Math.min(1.45, drag.pitch + (event.clientY - drag.y) * 0.007));
  scheduleRedraw(true);
});
canvas.addEventListener('pointerup', event => {
  drag = null;
  if (canvas.hasPointerCapture(event.pointerId)) canvas.releasePointerCapture(event.pointerId);
  scheduleRedraw(false);
});
canvas.addEventListener('pointercancel', () => { drag = null; scheduleRedraw(false); });
window.addEventListener('beforeunload', () => { if (frameRequest) cancelAnimationFrame(frameRequest); client.dispose(); });
void generatePlanet();
