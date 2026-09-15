from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing replacement marker in {path}: {old[:160]!r}")
    if text.count(old) != 1:
        raise SystemExit(f"replacement marker not unique in {path}: {text.count(old)} matches")
    p.write_text(text.replace(old, new, 1))


def append_once(path: str, marker: str, addition: str) -> None:
    p = Path(path)
    text = p.read_text()
    if marker in text:
        return
    p.write_text(text.rstrip() + "\n\n" + addition.strip() + "\n")


# --- Pure equirectangular camera math, shared by interactions and tests.
math_path = "src/worldgen/diagnostics/worldgenClimateMath.ts"
append_once(
    math_path,
    "export type EquirectangularCamera",
    r'''
export type EquirectangularCamera = {
  centerLongitudeRad: number;
  centerLatitudeRad: number;
  zoom: number;
};

export function wrapLongitudeRad(value: number): number {
  const twoPi = Math.PI * 2;
  return ((value + Math.PI) % twoPi + twoPi) % twoPi - Math.PI;
}

export function clampEquirectangularCenterLatitude(value: number, zoom: number): number {
  const safeZoom = Math.max(1, zoom);
  const halfSpan = Math.PI / (2 * safeZoom);
  return Math.max(-Math.PI / 2 + halfSpan, Math.min(Math.PI / 2 - halfSpan, value));
}

export function equirectangularScreenToWorldDirection(
  canvasX: number,
  canvasY: number,
  width: number,
  height: number,
  camera: EquirectangularCamera,
): [number, number, number] {
  const safeZoom = Math.max(1, camera.zoom);
  const longitude = wrapLongitudeRad(
    camera.centerLongitudeRad + (canvasX / Math.max(1, width) - 0.5) * Math.PI * 2 / safeZoom,
  );
  const latitude = Math.max(-Math.PI / 2, Math.min(
    Math.PI / 2,
    camera.centerLatitudeRad + (0.5 - canvasY / Math.max(1, height)) * Math.PI / safeZoom,
  ));
  const cosLatitude = Math.cos(latitude);
  return [
    cosLatitude * Math.cos(longitude),
    cosLatitude * Math.sin(longitude),
    Math.sin(latitude),
  ];
}

export function equirectangularCameraForWorldDirectionAtScreen(
  direction: readonly [number, number, number],
  canvasX: number,
  canvasY: number,
  width: number,
  height: number,
  zoom: number,
): EquirectangularCamera {
  const safeZoom = Math.max(1, zoom);
  const longitude = Math.atan2(direction[1], direction[0]);
  const latitude = Math.asin(Math.max(-1, Math.min(1, direction[2])));
  const centerLongitudeRad = wrapLongitudeRad(
    longitude - (canvasX / Math.max(1, width) - 0.5) * Math.PI * 2 / safeZoom,
  );
  const centerLatitudeRad = clampEquirectangularCenterLatitude(
    latitude - (0.5 - canvasY / Math.max(1, height)) * Math.PI / safeZoom,
    safeZoom,
  );
  return { centerLongitudeRad, centerLatitudeRad, zoom: safeZoom };
}
''',
)

# --- Canonical climate lab: inverse-rasterized map plus full map camera interaction.
lab = "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts"
replace_once(
    lab,
    "import { mapVectorDelta, reconstructAnnualHarmonicFromBasis } from './worldgenClimateMath.js';",
    "import { clampEquirectangularCenterLatitude, equirectangularCameraForWorldDirectionAtScreen, equirectangularScreenToWorldDirection, mapVectorDelta, reconstructAnnualHarmonicFromBasis, wrapLongitudeRad } from './worldgenClimateMath.js';",
)

replace_once(
    lab,
    r'''function projectSamples(result: WorldgenClimateResult, projection: string, yaw: number, pitch: number, width: number, height: number, buffers: ProjectionBuffers, zoom = 1): void {
  const count = result.metrics.fineSampleCount;
  const positions = result.positions;
  if (projection === 'map') {
    for (let sample = 0; sample < count; sample += 1) {
      const offset = sample * 3;
      const px = positions[offset]!; const py = positions[offset + 1]!; const pz = positions[offset + 2]!;
      buffers.x[sample] = (Math.atan2(py, px) + Math.PI) / TWO_PI * width;
      buffers.y[sample] = (Math.PI / 2 - Math.asin(Math.max(-1, Math.min(1, pz)))) / Math.PI * height;
      buffers.visible[sample] = 1;
    }
    return;
  }''',
    r'''function projectSamples(result: WorldgenClimateResult, projection: string, yaw: number, pitch: number, width: number, height: number, buffers: ProjectionBuffers, zoom = 1): void {
  const count = result.metrics.fineSampleCount;
  const positions = result.positions;
  if (projection === 'map') {
    const safeZoom = Math.max(1, zoom);
    const longitudeSpan = TWO_PI / safeZoom;
    const latitudeSpan = Math.PI / safeZoom;
    for (let sample = 0; sample < count; sample += 1) {
      const offset = sample * 3;
      const px = positions[offset]!; const py = positions[offset + 1]!; const pz = positions[offset + 2]!;
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
  }''',
)

replace_once(
    lab,
    "if (projection === 'map') return mapVectorDelta(eastValue, northValue, lat, width, height);",
    "if (projection === 'map') return mapVectorDelta(eastValue, northValue, lat, width * Math.max(1, zoom), height * Math.max(1, zoom));",
)

map_raster_code = r'''
type MapRasterCache = {
  result: WorldgenClimateResult | null;
  lookupKey: string;
  styleKey: string;
  width: number;
  height: number;
  sampleIds: Uint32Array;
};
const mapRasterCanvas = document.createElement('canvas');
let mapRasterCache: MapRasterCache = {
  result: null,
  lookupKey: '',
  styleKey: '',
  width: 0,
  height: 0,
  sampleIds: new Uint32Array(0),
};

function nearestMapSampleFromSeed(
  result: WorldgenClimateResult,
  directionX: number,
  directionY: number,
  directionZ: number,
  seedSample: number,
): number {
  let bestSample = Math.max(0, Math.min(result.metrics.fineSampleCount - 1, seedSample));
  let offset = bestSample * 3;
  let bestDot = result.positions[offset]! * directionX
    + result.positions[offset + 1]! * directionY
    + result.positions[offset + 2]! * directionZ;
  while (true) {
    let improved = false;
    const start = result.neighborOffsets[bestSample]!;
    const end = result.neighborOffsets[bestSample + 1]!;
    for (let cursor = start; cursor < end; cursor += 1) {
      const neighbor = result.neighbors[cursor]!;
      offset = neighbor * 3;
      const dot = result.positions[offset]! * directionX
        + result.positions[offset + 1]! * directionY
        + result.positions[offset + 2]! * directionZ;
      if (dot > bestDot + 1e-12) {
        bestDot = dot;
        bestSample = neighbor;
        improved = true;
      }
    }
    if (!improved) return bestSample;
  }
}

function ensureMapSampleLookup(
  result: WorldgenClimateResult,
  centerLongitudeRad: number,
  centerLatitudeRad: number,
  zoom: number,
  width: number,
  height: number,
  interactive: boolean,
): MapRasterCache {
  const rasterScale = interactive && result.metrics.fineSampleCount > 100_000 ? 0.5 : 1;
  const rasterWidth = Math.max(1, Math.round(width * rasterScale));
  const rasterHeight = Math.max(1, Math.round(height * rasterScale));
  const lookupKey = `${centerLongitudeRad.toFixed(7)}:${centerLatitudeRad.toFixed(7)}:${zoom.toFixed(5)}:${rasterWidth}:${rasterHeight}`;
  if (mapRasterCache.result === result && mapRasterCache.lookupKey === lookupKey) return mapRasterCache;

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
      const direction: [number, number, number] = [
        cosLatitude * cosLongitude[0]!,
        cosLatitude * sinLongitude[0]!,
        directionZ,
      ];
      seedSample = pickNearestSample(result, direction);
    } else {
      seedSample = sampleIds[(y - 1) * rasterWidth]!;
    }
    for (let x = 0; x < rasterWidth; x += 1) {
      seedSample = nearestMapSampleFromSeed(
        result,
        cosLatitude * cosLongitude[x]!,
        cosLatitude * sinLongitude[x]!,
        directionZ,
        seedSample,
      );
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

function renderEquirectangularRaster(
  context: CanvasRenderingContext2D,
  result: WorldgenClimateResult,
  mode: string,
  phase: number,
  centerLongitudeRad: number,
  centerLatitudeRad: number,
  zoom: number,
  width: number,
  height: number,
  interactive: boolean,
  selectedSample: number | null,
): void {
  const lookup = ensureMapSampleLookup(
    result,
    centerLongitudeRad,
    centerLatitudeRad,
    zoom,
    width,
    height,
    interactive,
  );
  const gpu = ensureGpuColorCache(result, mode, phase);
  const styleKey = `${lookup.lookupKey}:${gpu.key}:${gpu.alpha.toFixed(3)}:${selectedSample ?? -1}`;
  if (mapRasterCanvas.width !== lookup.width) mapRasterCanvas.width = lookup.width;
  if (mapRasterCanvas.height !== lookup.height) mapRasterCanvas.height = lookup.height;
  const rasterContext = mapRasterCanvas.getContext('2d', { alpha: false });
  if (!rasterContext) throw new Error('Planet Engine Lab could not acquire the equirectangular raster context.');

  if (lookup.styleKey !== styleKey) {
    const image = rasterContext.createImageData(lookup.width, lookup.height);
    const pixels = image.data;
    const alpha = Math.max(0, Math.min(1, gpu.alpha));
    const inverseAlpha = 1 - alpha;
    for (let pixel = 0; pixel < lookup.sampleIds.length; pixel += 1) {
      const sample = lookup.sampleIds[pixel]!;
      const colorOffset = sample * 4;
      let red = gpu.colors[colorOffset]!;
      let green = gpu.colors[colorOffset + 1]!;
      let blue = gpu.colors[colorOffset + 2]!;
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
'''
replace_once(
    lab,
    "let gpuColorCache: GpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 };\nlet projectedResult: WorldgenClimateResult | null = null;",
    "let gpuColorCache: GpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 };\n" + map_raster_code + "\nlet projectedResult: WorldgenClimateResult | null = null;",
)

map_branch = r'''
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
    if (!needsProjectedDecoration) return;
    ensureProjectedSamples(result, projection, yaw, pitch, width, height, buffers, zoom);

    if (mode === 'mesh') {
      context.beginPath(); context.strokeStyle = '#5d7890'; context.lineWidth = 0.55;
      for (let sample = 0; sample < result.metrics.fineSampleCount; sample += 1) {
        if (!buffers.visible[sample]) continue;
        const ax = buffers.x[sample]!, ay = buffers.y[sample]!;
        for (let cursor = result.neighborOffsets[sample]!; cursor < result.neighborOffsets[sample + 1]!; cursor += 1) {
          const neighbor = result.neighbors[cursor]!;
          if (neighbor <= sample || !buffers.visible[neighbor]) continue;
          const bx = buffers.x[neighbor]!;
          if (Math.abs(ax - bx) > width * 0.45) continue;
          context.moveTo(ax, ay); context.lineTo(bx, buffers.y[neighbor]!);
        }
      }
      context.stroke();
    } else {
      if (isDrainageMode(mode)) {
        if (mode === 'flow-direction') drawDrainageReceiverOverlay(context, result, projection, width, buffers);
        if (mode === 'basins') drawDrainageOutlets(context, result, buffers);
      }
      const cacheKey = `${mode}:${GPU_SEASONAL_MODES.has(mode) ? phase.toFixed(3) : 'mean'}`;
      if (styleCache.result !== result || styleCache.key !== cacheKey) styleCache = buildStyleCache(result, mode, phase);
      if (styleCache.boundaryBuckets.length > 0) {
        context.lineCap = 'round'; context.lineWidth = mode === 'boundary-provenance' ? 1.4 : 2.0;
        for (const bucket of styleCache.boundaryBuckets) {
          context.strokeStyle = bucket.color; context.beginPath();
          for (let cursor = 0; cursor < bucket.indices.length; cursor += 1) {
            const boundary = bucket.indices[cursor]!;
            const a = result.boundarySamples[boundary * 2]!, b = result.boundarySamples[boundary * 2 + 1]!;
            if (!buffers.visible[a] || !buffers.visible[b]) continue;
            const ax = buffers.x[a]!, bx = buffers.x[b]!;
            if (Math.abs(ax - bx) > width * 0.45) continue;
            context.moveTo(ax, buffers.y[a]!); context.lineTo(bx, buffers.y[b]!);
          }
          context.stroke();
        }
      }
      if (mode === 'winds' || mode === 'currents') drawVectors(context, result, mode, phase, projection, yaw, pitch, width, height, buffers, animation, zoom);
    }
    drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation, zoom);
    return;
  }
'''
replace_once(
    lab,
    "\n  surfaceCanvas.hidden = true;\n  context.fillStyle = '#08101a'; context.fillRect(0, 0, width, height);",
    "\n" + map_branch + "\n  surfaceCanvas.hidden = true;\n  surfaceCanvas.style.display = 'none';\n  context.fillStyle = '#08101a'; context.fillRect(0, 0, width, height);",
)
replace_once(
    lab,
    "    surfaceCanvas.hidden = false;\n    const gpuDrawn = globeRenderer.draw(",
    "    surfaceCanvas.hidden = false;\n    surfaceCanvas.style.display = '';\n    const gpuDrawn = globeRenderer.draw(",
)

replace_once(
    lab,
    "let yaw = -0.65;\nlet pitch = 0.25;\nlet zoom = 1;",
    "let yaw = -0.65;\nlet pitch = 0.25;\nlet mapCenterLongitude = 0;\nlet mapCenterLatitude = 0;\nlet zoom = 1;",
)
replace_once(
    lab,
    "let drag: { x: number; y: number; yaw: number; pitch: number } | null = null;",
    "let drag: { x: number; y: number; yaw: number; pitch: number; mapLongitude: number; mapLatitude: number; projection: string } | null = null;",
)
replace_once(
    lab,
    "function redraw(interactive = false): void {\n  if (!current || !buffers) return;\n  renderPlanet(surfaceCanvas, canvas, current, projection.value, visualization.value, selectedOverlays(), orbitalPhase(), yaw, pitch, zoom, buffers, interactive, animationPhase, selectedTile);\n}",
    "function redraw(interactive = false): void {\n  if (!current || !buffers) return;\n  const map = projection.value === 'map';\n  renderPlanet(surfaceCanvas, canvas, current, projection.value, visualization.value, selectedOverlays(), orbitalPhase(), map ? mapCenterLongitude : yaw, map ? mapCenterLatitude : pitch, zoom, buffers, interactive, animationPhase, selectedTile);\n}",
)
replace_once(
    lab,
    "function setZoom(next: number, interactive = true): void {\n  zoom = Math.max(1, Math.min(24, next));\n  zoomControl.value = zoom.toFixed(1);",
    "function setZoom(next: number, interactive = true): void {\n  zoom = Math.max(1, Math.min(24, next));\n  if (projection.value === 'map') mapCenterLatitude = clampEquirectangularCenterLatitude(mapCenterLatitude, zoom);\n  zoomControl.value = zoom.toFixed(1);",
)
replace_once(
    lab,
    "function updateCameraControls(): void {\n  const globe = projection.value === 'globe';\n  zoomControl.disabled = !globe;\n  resetCamera.disabled = !globe;\n}",
    "function updateCameraControls(): void {\n  zoomControl.disabled = false;\n  resetCamera.disabled = false;\n}",
)
replace_once(
    lab,
    "function inspectTile(sample: number | null): void {\n  if (!current || sample === null) { cellInspector.textContent = 'Click the globe to inspect an L8 physical cell.'; return; }",
    "function inspectTile(sample: number | null): void {\n  if (!current || sample === null) { cellInspector.textContent = 'Click the globe or map to inspect an L8 physical cell.'; return; }",
)
replace_once(
    lab,
    r'''function pickTileAtPointer(event: PointerEvent | WheelEvent): number | null {
  if (!current || projection.value !== 'globe') return null;
  const rect = canvas.getBoundingClientRect();
  const x = (event.clientX - rect.left) * canvas.width / Math.max(1, rect.width);
  const y = (event.clientY - rect.top) * canvas.height / Math.max(1, rect.height);
  const direction = screenToWorldDirection(x, y, canvas.width, canvas.height, { yaw, pitch, zoom });
  return direction ? pickNearestSample(current, direction) : null;
}''',
    r'''function pickTileAtPointer(event: PointerEvent | WheelEvent): number | null {
  if (!current) return null;
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
}''',
)

replace_once(
    lab,
    r'''resetCamera.addEventListener('click', () => {
  yaw = -0.65; pitch = 0.25; selectedTile = null; inspectTile(null); setZoom(1, false);
});''',
    r'''resetCamera.addEventListener('click', () => {
  if (projection.value === 'map') {
    mapCenterLongitude = 0;
    mapCenterLatitude = 0;
  } else {
    yaw = -0.65;
    pitch = 0.25;
  }
  selectedTile = null; inspectTile(null); setZoom(1, false);
});''',
)
replace_once(
    lab,
    r'''canvas.addEventListener('wheel', event => {
  if (projection.value !== 'globe' || !current) return;
  event.preventDefault();
  const rect = canvas.getBoundingClientRect();
  const canvasX = (event.clientX - rect.left) * canvas.width / Math.max(1, rect.width);
  const canvasY = (event.clientY - rect.top) * canvas.height / Math.max(1, rect.height);
  const anchorDirection = screenToWorldDirection(canvasX, canvasY, canvas.width, canvas.height, { yaw, pitch, zoom });
  const nextZoom = Math.max(1, Math.min(24, zoom * Math.exp(-event.deltaY * 0.0015)));
  if (anchorDirection) {
    const camera = cameraForWorldDirectionAtScreen(anchorDirection, canvasX, canvasY, canvas.width, canvas.height, nextZoom, { yaw, pitch, zoom });
    yaw = camera.yaw;
    pitch = camera.pitch;
  }
  setZoom(nextZoom, true);
}, { passive: false });''',
    r'''canvas.addEventListener('wheel', event => {
  if (!current) return;
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
  } else {
    const anchorDirection = screenToWorldDirection(canvasX, canvasY, canvas.width, canvas.height, { yaw, pitch, zoom });
    if (anchorDirection) {
      const camera = cameraForWorldDirectionAtScreen(anchorDirection, canvasX, canvasY, canvas.width, canvas.height, nextZoom, { yaw, pitch, zoom });
      yaw = camera.yaw;
      pitch = camera.pitch;
    }
  }
  setZoom(nextZoom, true);
}, { passive: false });''',
)
replace_once(
    lab,
    r'''canvas.addEventListener('pointerdown', event => {
  if (projection.value !== 'globe') return;
  drag = { x: event.clientX, y: event.clientY, yaw, pitch }; canvas.setPointerCapture(event.pointerId);
});
canvas.addEventListener('pointermove', event => {
  if (!drag || projection.value !== 'globe') return;
  const sensitivity = 0.007 / Math.sqrt(zoom);
  yaw = drag.yaw + (event.clientX - drag.x) * sensitivity;
  pitch = Math.max(-1.45, Math.min(1.45, drag.pitch + (event.clientY - drag.y) * sensitivity));
  scheduleRedraw(true);
});''',
    r'''canvas.addEventListener('pointerdown', event => {
  if (!current) return;
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
  if (!drag || projection.value !== drag.projection) return;
  if (projection.value === 'map') {
    const rect = canvas.getBoundingClientRect();
    const longitudeSpan = TWO_PI / Math.max(1, zoom);
    const latitudeSpan = Math.PI / Math.max(1, zoom);
    mapCenterLongitude = wrapLongitudeRad(drag.mapLongitude - (event.clientX - drag.x) / Math.max(1, rect.width) * longitudeSpan);
    mapCenterLatitude = clampEquirectangularCenterLatitude(
      drag.mapLatitude + (event.clientY - drag.y) / Math.max(1, rect.height) * latitudeSpan,
      zoom,
    );
  } else {
    const sensitivity = 0.007 / Math.sqrt(zoom);
    yaw = drag.yaw + (event.clientX - drag.x) * sensitivity;
    pitch = Math.max(-1.45, Math.min(1.45, drag.pitch + (event.clientY - drag.y) * sensitivity));
  }
  scheduleRedraw(true);
  scheduleSettledCameraRedraw();
});''',
)

# Reset the map raster cache when a newly generated world is installed.
replace_once(
    lab,
    "    gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 };\n    projectedResult = null; projectedKey = '';",
    "    gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 };\n    mapRasterCache = { result: null, lookupKey: '', styleKey: '', width: 0, height: 0, sampleIds: new Uint32Array(0) };\n    projectedResult = null; projectedKey = '';",
)

# --- CSS: author canvas display must never override the semantic hidden attribute.
css = "styles/worldgenLab.css"
append_once(
    css,
    ".worldgen-lab-viewport canvas[hidden]",
    ".worldgen-lab-viewport canvas[hidden] { display: none !important; }",
)

# --- UI labels now describe both projections.
html = "index.html"
replace_once(html, "<label>Globe zoom", "<label>View zoom")
replace_once(html, ">Reset globe camera</button>", ">Reset view</button>")
replace_once(html, ">Click the globe to inspect an L8 physical cell.</div>", ">Click the globe or map to inspect an L8 physical cell.</div>")

# --- Documentation reflects inverse projection and map interaction parity.
docs = "docs/worldgen-rewrite/L8_VIEWER_USABILITY.md"
replace_once(
    docs,
    "Enabled diagnostic overlays stay on the Canvas overlay layer during interaction. CPU sample projection is skipped entirely for the common no-overlay globe path; it is refreshed only when an active overlay or diagnostic line layer needs projected sample coordinates. The equirectangular map retains the CPU/Canvas fallback path.",
    "Enabled diagnostic overlays stay on the Canvas overlay layer during interaction. CPU sample projection is skipped entirely for the common no-overlay globe path; it is refreshed only when an active overlay or diagnostic line layer needs projected sample coordinates. The equirectangular map is inverse-rasterized: each output pixel is mapped back to a spherical direction and resolved to the nearest L8 physical sample by neighbor-graph hill climbing. The map therefore renders a continuous field instead of flattening equal-area sample dots into polar fans.",
)
replace_once(
    docs,
    "Globe zoom is continuous from 1x to 24x and does not regenerate the planet or change renderers. The WebGL globe is contiguous at every zoom because it renders triangulated dual cells rather than point sprites. The optional cell-boundary overlay fades in as cells become screen-resolved instead of appearing through a hard 4.5x renderer transition. Clicking the globe resolves the nearest L8 sample by searching the inherited L5 seed set and then hill-climbing the L8 neighbor graph. The selected physical cell is always filled and outlined on the GPU, independent of the active diagnostic or whether the cell-boundary overlay is enabled, and the inspector reports the stable cell/sample id plus physical context.",
    "View zoom is continuous from 1x to 24x and does not regenerate the planet or change renderers. The WebGL globe is contiguous at every zoom because it renders triangulated dual cells rather than point sprites. The equirectangular map uses the same zoom control, cursor-anchored wheel zoom, drag-to-pan interaction, and click-to-inspect workflow; its inverse raster is rebuilt at reduced resolution during active dragging and at full viewport resolution once interaction settles. The optional globe cell-boundary overlay fades in as cells become screen-resolved instead of appearing through a hard 4.5x renderer transition. Clicking either projection resolves the nearest L8 sample and the inspector reports the stable cell/sample id plus physical context.",
)
append_once(
    docs,
    "Only one projection surface",
    "## Projection switching\n\nOnly one projection surface is visible at a time. The WebGL surface canvas is explicitly hidden when the equirectangular map is active, including an author-level `[hidden]` CSS rule so the viewport's generic canvas display rule cannot resurrect the globe underneath the map. Switching back to the globe restores the WebGL surface and leaves the Canvas layer available for overlays.",
)

# --- Regression tests: pure map camera math plus viewer wiring.
test_path = "tests/l8Viewer.test.ts"
replace_once(
    test_path,
    "import { buildDualCellGpuMesh, buildGpuPositions, cameraForWorldDirectionAtScreen, collectViewportSampleIndices, pickNearestSample, screenToWorldDirection } from '../dist/worldgen/diagnostics/worldgenL8GlobeRenderer.js';",
    "import { buildDualCellGpuMesh, buildGpuPositions, cameraForWorldDirectionAtScreen, collectViewportSampleIndices, pickNearestSample, screenToWorldDirection } from '../dist/worldgen/diagnostics/worldgenL8GlobeRenderer.js';\nimport { clampEquirectangularCenterLatitude, equirectangularCameraForWorldDirectionAtScreen, equirectangularScreenToWorldDirection } from '../dist/worldgen/diagnostics/worldgenClimateMath.js';",
)
insert_test = r'''
test('equirectangular zoom preserves the world direction under the cursor', () => {
  const initial = { centerLongitudeRad: 0.4, centerLatitudeRad: 0, zoom: 1 };
  const x = 730, y = 180, width = 1100, height = 550;
  const anchor = equirectangularScreenToWorldDirection(x, y, width, height, initial);
  const zoomed = equirectangularCameraForWorldDirectionAtScreen(anchor, x, y, width, height, 6);
  const recovered = equirectangularScreenToWorldDirection(x, y, width, height, zoomed);
  const dot = anchor[0] * recovered[0] + anchor[1] * recovered[1] + anchor[2] * recovered[2];
  assert.ok(dot > 0.999999999, `map zoom anchor drifted: dot=${dot}`);
  assert.equal(clampEquirectangularCenterLatitude(0.6, 1), 0);
});

'''
replace_once(
    test_path,
    "test('L8 picking refines from a coarse seed set through topology neighbors', () => {",
    insert_test + "test('L8 picking refines from a coarse seed set through topology neighbors', () => {",
)
replace_once(
    test_path,
    "  assert.match(source, /addEventListener\\('wheel'/);",
    "  assert.match(source, /addEventListener\\('wheel'/);\n  assert.match(source, /renderEquirectangularRaster/);\n  assert.match(source, /nearestMapSampleFromSeed/);\n  assert.match(source, /equirectangularCameraForWorldDirectionAtScreen/);\n  assert.match(source, /mapCenterLongitude/);\n  assert.match(css, /canvas\\[hidden\\] \\{ display: none !important; \\}/);\n  assert.match(html, /View zoom/);\n  assert.match(html, /Reset view/);\n  assert.match(html, /Click the globe or map to inspect/);",
)

print('interactive equirectangular viewer rewrite applied')
