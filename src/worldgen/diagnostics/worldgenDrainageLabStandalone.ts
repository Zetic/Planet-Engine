import { createWorldgenClient } from '../worldgenClient.js';
import { type WorldgenDrainageResult, WORLDGEN_INVALID_SAMPLE_ID } from '../protocol.js';

type ProjectionBuffers = { x: Float32Array; y: Float32Array; visible: Uint8Array };

const TWO_PI = Math.PI * 2;
const INVALID = WORLDGEN_INVALID_SAMPLE_ID;

function element<T extends HTMLElement>(id: string): T {
  const target = document.getElementById(id);
  if (!target) throw new Error(`WG-6A Drainage Lab is missing #${id}.`);
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

function discreteColor(id: number, saturation = 62, lightness = 53): string {
  return `hsl(${(id * 137.507764 + 32) % 360} ${saturation}% ${lightness}%)`;
}

function scalarColor(t: number, lowHue: number, highHue: number): string {
  const clamped = Math.max(0, Math.min(1, t));
  const hue = lowHue + (highHue - lowHue) * clamped;
  return `hsl(${hue} 70% ${34 + 25 * clamped}%)`;
}

function hypsometricColor(result: WorldgenDrainageResult, sample: number): string {
  if (result.submergedMask[sample]) return '#174d73';
  const elevation = result.elevationAboveSeaLevelM[sample]!;
  if (elevation < 100) return '#507d45';
  if (elevation < 400) return '#6d9450';
  if (elevation < 1_000) return '#91a85d';
  if (elevation < 2_000) return '#aa9463';
  if (elevation < 3_500) return '#8f7157';
  if (elevation < 5_000) return '#9b9290';
  return '#e6ebed';
}

function sampleColor(result: WorldgenDrainageResult, mode: string, sample: number): string {
  if (mode === 'physical-elevation') return hypsometricColor(result, sample);
  if (mode === 'land-water') return result.submergedMask[sample] ? '#1f6795' : '#7aa05d';
  if (result.submergedMask[sample]) return '#102c43';

  if (mode === 'basins' || mode === 'flow-direction') {
    const basin = result.basinId[sample]!;
    return basin === INVALID ? '#4c5964' : discreteColor(basin);
  }
  if (mode === 'depressions') {
    const depression = result.depressionId[sample]!;
    return depression === INVALID ? '#31423c' : discreteColor(depression, 72, 58);
  }
  if (mode === 'depression-depth') {
    const depth = result.depressionDepthM[sample]!;
    if (depth <= 0) return '#283c34';
    const t = Math.log10(1 + depth) / Math.log10(1 + Math.max(50, result.metrics.maximumDepressionDepthM));
    return scalarColor(t, 55, 270);
  }
  if (mode === 'escape-elevation') {
    const value = result.hydrologicEscapeElevationM[sample]!;
    return scalarColor((value + 500) / 5_500, 220, 20);
  }
  const areaKm2 = Math.max(1e-9, result.contributingAreaM2[sample]! / 1e6);
  const logArea = Math.log10(areaKm2 + 1);
  const maxLog = Math.log10(Math.max(10, result.metrics.maximumContributingAreaM2 / 1e6) + 1);
  return scalarColor(logArea / maxLog, 225, 42);
}

function ensureBuffers(buffers: ProjectionBuffers, count: number): ProjectionBuffers {
  if (buffers.x.length === count) return buffers;
  return { x: new Float32Array(count), y: new Float32Array(count), visible: new Uint8Array(count) };
}

function projectSamples(result: WorldgenDrainageResult, projection: string, yaw: number, pitch: number, width: number, height: number, buffers: ProjectionBuffers): void {
  const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch);
  const radius = Math.min(width, height) * 0.44;
  for (let sample = 0; sample < result.metrics.sampleCount; sample += 1) {
    const offset = sample * 3;
    const x = result.positions[offset]!, y = result.positions[offset + 1]!, z = result.positions[offset + 2]!;
    if (projection === 'map') {
      const lon = Math.atan2(y, x);
      const lat = Math.asin(Math.max(-1, Math.min(1, z)));
      buffers.x[sample] = ((lon / TWO_PI) + 0.5) * width;
      buffers.y[sample] = (0.5 - lat / Math.PI) * height;
      buffers.visible[sample] = 1;
      continue;
    }
    const x1 = cy * x - sy * y;
    const y1 = sy * x + cy * y;
    const x2 = cp * x1 + sp * z;
    const z2 = -sp * x1 + cp * z;
    buffers.x[sample] = width / 2 + y1 * radius;
    buffers.y[sample] = height / 2 - z2 * radius;
    buffers.visible[sample] = x2 >= 0 ? 1 : 0;
  }
}

function drawReceiverOverlay(context: CanvasRenderingContext2D, result: WorldgenDrainageResult, projection: string, width: number, buffers: ProjectionBuffers): void {
  const targetSegments = 3_500;
  const stride = Math.max(1, Math.floor(result.metrics.landSampleCount / targetSegments));
  context.save();
  context.strokeStyle = 'rgba(235,247,255,0.68)';
  context.lineWidth = 0.75;
  context.beginPath();
  let accepted = 0;
  for (let sample = 0; sample < result.metrics.sampleCount; sample += 1) {
    if (result.submergedMask[sample] || !buffers.visible[sample]) continue;
    if ((accepted++ % stride) !== 0) continue;
    const receiver = result.receiver[sample]!;
    if (receiver === INVALID || !buffers.visible[receiver]) continue;
    const ax = buffers.x[sample]!, bx = buffers.x[receiver]!;
    if (projection === 'map' && Math.abs(ax - bx) > width * 0.45) continue;
    context.moveTo(ax, buffers.y[sample]!);
    context.lineTo(bx, buffers.y[receiver]!);
  }
  context.stroke();
  context.restore();
}

function drawOutlets(context: CanvasRenderingContext2D, result: WorldgenDrainageResult, buffers: ProjectionBuffers): void {
  context.save();
  context.fillStyle = 'rgba(255,255,255,0.92)';
  for (const outlet of result.basinOutletSamples) {
    if (outlet === INVALID || !buffers.visible[outlet]) continue;
    context.beginPath();
    context.arc(buffers.x[outlet]!, buffers.y[outlet]!, 2.2, 0, TWO_PI);
    context.fill();
  }
  context.restore();
}

function render(canvas: HTMLCanvasElement, result: WorldgenDrainageResult, projection: string, mode: string, yaw: number, pitch: number, buffers: ProjectionBuffers): void {
  const width = 1100;
  const height = projection === 'map' ? 550 : 760;
  if (canvas.width !== width) canvas.width = width;
  if (canvas.height !== height) canvas.height = height;
  const context = canvas.getContext('2d');
  if (!context) throw new Error('WG-6A Drainage Lab could not acquire a 2D canvas context.');
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

  const count = result.metrics.sampleCount;
  const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
  for (let sample = 0; sample < count; sample += 1) {
    if (!buffers.visible[sample]) continue;
    context.fillStyle = sampleColor(result, mode, sample);
    context.beginPath();
    context.arc(buffers.x[sample]!, buffers.y[sample]!, pointRadius, 0, TWO_PI);
    context.fill();
  }
  if (mode === 'flow-direction') drawReceiverOverlay(context, result, projection, width, buffers);
  if (mode === 'basins') drawOutlets(context, result, buffers);
}

function showMetrics(result: WorldgenDrainageResult): void {
  const metrics = element<HTMLDivElement>('worldgen-metrics');
  metrics.replaceChildren();
  metric(metrics, 'Engine / stage', `v${result.engineVersion} · ${result.stage.id}@${result.stage.version}`);
  metric(metrics, 'Resolution', `L${result.coarseLevel} → L${result.fineLevel}`);
  metric(metrics, 'Samples', result.metrics.sampleCount.toLocaleString());
  metric(metrics, 'Land / ocean samples', `${result.metrics.landSampleCount.toLocaleString()} / ${result.metrics.oceanSampleCount.toLocaleString()}`);
  metric(metrics, 'Basins', result.metrics.basinCount.toLocaleString());
  metric(metrics, 'Depressions', `${result.metrics.depressionCount.toLocaleString()} · ${result.metrics.depressionSampleCount.toLocaleString()} cells`);
  metric(metrics, 'Largest contributing area', `${(result.metrics.maximumContributingAreaM2 / 1e12).toFixed(3)} million km²`);
  metric(metrics, 'Deepest depression', `${result.metrics.maximumDepressionDepthM.toFixed(1)} m`);
  metric(metrics, 'Area closure', result.metrics.areaConservationRelativeError.toExponential(2));
  metric(metrics, 'Drainage hash', result.metrics.drainageHash);
  metric(metrics, 'Topography hash', result.topographyHash);
  metric(metrics, 'Topology hash', result.topologyHash);
}

const client = createWorldgenClient();
const canvas = element<HTMLCanvasElement>('worldgen-field');
const seedInput = element<HTMLInputElement>('worldgen-seed');
const coarseInput = element<HTMLInputElement>('worldgen-coarse-level');
const levelInput = element<HTMLInputElement>('worldgen-level');
const platesInput = element<HTMLInputElement>('worldgen-plates');
const generateButton = element<HTMLButtonElement>('worldgen-generate');
const status = element<HTMLParagraphElement>('worldgen-status');
const modeSelect = element<HTMLSelectElement>('worldgen-visualization');
const projectionSelect = element<HTMLSelectElement>('worldgen-projection');
const timer = element<HTMLSpanElement>('worldgen-generation-timer');

let current: WorldgenDrainageResult | null = null;
let buffers: ProjectionBuffers = { x: new Float32Array(0), y: new Float32Array(0), visible: new Uint8Array(0) };
let yaw = -0.55;
let pitch = 0.2;
let dragging = false;
let lastX = 0;
let lastY = 0;

function rerender(): void {
  if (!current) return;
  buffers = ensureBuffers(buffers, current.metrics.sampleCount);
  render(canvas, current, projectionSelect.value, modeSelect.value, yaw, pitch, buffers);
}

async function generate(): Promise<void> {
  generateButton.disabled = true;
  status.textContent = 'Generating WG-6A drainage topology…';
  const started = performance.now();
  try {
    current = await client.generateDrainage({
      seed: seedInput.value,
      coarseLevel: Number(coarseInput.value),
      fineLevel: Number(levelInput.value),
      plateCount: Number(platesInput.value),
    });
    timer.textContent = `${(performance.now() - started).toFixed(1)} ms total · ${current.stage.durationMs.toFixed(1)} ms through WG-6A`;
    status.textContent = `Generated ${current.metrics.sampleCount.toLocaleString()} drainage cells · ${current.metrics.basinCount.toLocaleString()} basins · area closure ${current.metrics.areaConservationRelativeError.toExponential(2)}`;
    showMetrics(current);
    rerender();
  } catch (error) {
    status.textContent = error instanceof Error ? error.message : String(error);
  } finally {
    generateButton.disabled = false;
  }
}

generateButton.addEventListener('click', () => void generate());
modeSelect.addEventListener('change', rerender);
projectionSelect.addEventListener('change', rerender);
canvas.addEventListener('pointerdown', event => {
  if (projectionSelect.value !== 'globe') return;
  dragging = true;
  lastX = event.clientX;
  lastY = event.clientY;
  canvas.setPointerCapture(event.pointerId);
});
canvas.addEventListener('pointermove', event => {
  if (!dragging || projectionSelect.value !== 'globe') return;
  yaw += (event.clientX - lastX) * 0.007;
  pitch = Math.max(-1.35, Math.min(1.35, pitch + (event.clientY - lastY) * 0.007));
  lastX = event.clientX;
  lastY = event.clientY;
  rerender();
});
canvas.addEventListener('pointerup', event => {
  dragging = false;
  if (canvas.hasPointerCapture(event.pointerId)) canvas.releasePointerCapture(event.pointerId);
});

status.textContent = 'Planet Engine Worker ready. Generate WG-6A drainage topology.';
void generate();
