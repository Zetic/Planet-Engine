from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"marker not found in {path}: {old[:140]!r}")
    target.write_text(text.replace(old, new, 1))


# Main GitHub Pages Lab now represents the physical stack through WG-6A.
replace_once(
    "index.html",
    "<title>Planet Engine · WG-5 Lab</title>",
    "<title>Planet Engine · Through WG-6A</title>",
)
replace_once(
    "index.html",
    '<p class="worldgen-lab-kicker">PLANET ENGINE · THROUGH WG-5</p>',
    '<p class="worldgen-lab-kicker">PLANET ENGINE · THROUGH WG-6A</p>',
)
replace_once(
    "index.html",
    "Generate one deterministic physical planet through WG-5, then inspect topology, tectonics, geology, lithosphere, topography, seasonal climate, winds, surface currents, and moisture from the same result.",
    "Generate one deterministic physical planet through WG-6A, then inspect topology, tectonics, geology, lithosphere, topography, climate, and terrain-driven drainage from one matched physical surface.",
)
replace_once(
    "index.html",
    'value="interlink-wg5"',
    'value="interlink-wg6a"',
)

hydrology_group = '''          <optgroup label="Hydrology · Drainage topology (WG-6A)">
            <option value="contributing-area">Contributing drainage area</option>
            <option value="basins">Drainage basins / outlets</option>
            <option value="flow-direction">Flow receivers</option>
            <option value="depression-depth">Depression depth</option>
            <option value="depressions">Depression regions</option>
            <option value="escape-elevation">Hydrologic escape elevation</option>
          </optgroup>
'''
replace_once(
    "index.html",
    '          <optgroup label="Climate · Radiation / temperature (WG-5)">',
    hydrology_group + '          <optgroup label="Climate · Radiation / temperature (WG-5)">',
)
replace_once(
    "index.html",
    '<strong>Current physical frontier: WG-5</strong>',
    '<strong>Current physical frontier: WG-6A</strong>',
)
replace_once(
    "index.html",
    "One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, and WG-5 coupled-climate pipeline. Every diagnostic mode inspects that same generated planet.",
    "One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, and WG-6A drainage topology. Climate and drainage are required to resolve to the same deterministic WG-4 topography identity before the Lab accepts the result.",
)
replace_once(
    "index.html",
    "Overlays are independent of the selected diagnostic, so climate fields can be compared directly against topographic contours, coastlines, boundaries, winds, and currents. Seasonal precipitation displays the retained final spin-up-year orbital phases; annual precipitation seasonality remains an annual summary statistic.",
    "Overlays remain independent of the selected diagnostic, so drainage basins and contributing area can be compared directly against topographic contours, coastlines, tectonic boundaries, winds, and currents. Seasonal precipitation displays the retained final spin-up-year orbital phases; WG-6A itself remains terrain-only and does not consume rainfall yet.",
)
replace_once(
    "index.html",
    "The physical surface remains pre-erosional. Drainage, river incision, sediment transport, glacier flow, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.",
    "The physical surface remains pre-erosional. Runoff and river discharge, lake water balance, river incision, sediment transport, glacier flow, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.",
)

# Blend the existing WG-6A transport into the primary Lab without changing the
# physical stage contracts. The climate and drainage requests share one seed /
# resolution configuration and are rejected if their WG-4 topography identities
# diverge.
replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    "  WORLDGEN_STRUCTURE_TRANSFORM,\n  type WorldgenClimateResult,\n  type WorldgenGenerationProgress,",
    "  WORLDGEN_STRUCTURE_TRANSFORM,\n  WORLDGEN_INVALID_SAMPLE_ID,\n  type WorldgenClimateResult,\n  type WorldgenDrainageResult,\n  type WorldgenGenerationProgress,",
)

hydrology_helpers = r'''

const DRAINAGE_MODES = new Set([
  'contributing-area',
  'basins',
  'flow-direction',
  'depression-depth',
  'depressions',
  'escape-elevation',
]);

function isDrainageMode(mode: string): boolean { return DRAINAGE_MODES.has(mode); }
function discreteDrainageColor(id: number, saturation = 62, lightness = 53): string {
  return `hsl(${(id * 137.507764 + 32) % 360} ${saturation}% ${lightness}%)`;
}
function drainageScalarColor(t: number, lowHue: number, highHue: number): string {
  const clamped = Math.max(0, Math.min(1, t));
  const hue = lowHue + (highHue - lowHue) * clamped;
  return `hsl(${hue} 70% ${34 + 25 * clamped}%)`;
}
function drainageSampleColor(result: WorldgenDrainageResult, mode: string, sample: number): string {
  if (result.submergedMask[sample]) return '#102c43';
  if (mode === 'basins' || mode === 'flow-direction') {
    const basin = result.basinId[sample]!;
    return basin === WORLDGEN_INVALID_SAMPLE_ID ? '#4c5964' : discreteDrainageColor(basin);
  }
  if (mode === 'depressions') {
    const depression = result.depressionId[sample]!;
    return depression === WORLDGEN_INVALID_SAMPLE_ID ? '#31423c' : discreteDrainageColor(depression, 72, 58);
  }
  if (mode === 'depression-depth') {
    const depth = result.depressionDepthM[sample]!;
    if (depth <= 0) return '#283c34';
    const t = Math.log10(1 + depth) / Math.log10(1 + Math.max(50, result.metrics.maximumDepressionDepthM));
    return drainageScalarColor(t, 55, 270);
  }
  if (mode === 'escape-elevation') {
    return drainageScalarColor((result.hydrologicEscapeElevationM[sample]! + 500) / 5_500, 220, 20);
  }
  const areaKm2 = Math.max(1e-9, result.contributingAreaM2[sample]! / 1e6);
  const logArea = Math.log10(areaKm2 + 1);
  const maxLog = Math.log10(Math.max(10, result.metrics.maximumContributingAreaM2 / 1e6) + 1);
  return drainageScalarColor(logArea / maxLog, 225, 42);
}

function drawDrainageReceiverOverlay(
  context: CanvasRenderingContext2D,
  result: WorldgenDrainageResult,
  projection: string,
  width: number,
  buffers: ProjectionBuffers,
): void {
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
    if (receiver === WORLDGEN_INVALID_SAMPLE_ID || !buffers.visible[receiver]) continue;
    const ax = buffers.x[sample]!, bx = buffers.x[receiver]!;
    if (projection === 'map' && Math.abs(ax - bx) > width * 0.45) continue;
    context.moveTo(ax, buffers.y[sample]!);
    context.lineTo(bx, buffers.y[receiver]!);
  }
  context.stroke();
  context.restore();
}

function drawDrainageOutlets(context: CanvasRenderingContext2D, result: WorldgenDrainageResult, buffers: ProjectionBuffers): void {
  context.save();
  context.fillStyle = 'rgba(255,255,255,0.92)';
  for (const outlet of result.basinOutletSamples) {
    if (outlet === WORLDGEN_INVALID_SAMPLE_ID || !buffers.visible[outlet]) continue;
    context.beginPath();
    context.arc(buffers.x[outlet]!, buffers.y[outlet]!, 2.2, 0, TWO_PI);
    context.fill();
  }
  context.restore();
}

function renderDrainageDiagnostic(
  context: CanvasRenderingContext2D,
  result: WorldgenDrainageResult,
  projection: string,
  mode: string,
  width: number,
  buffers: ProjectionBuffers,
  interactive: boolean,
): void {
  const count = result.metrics.sampleCount;
  const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
  const fastPoints = interactive && count > 20_000;
  context.globalAlpha = 0.94;
  for (let sample = 0; sample < count; sample += 1) {
    if (!buffers.visible[sample]) continue;
    context.fillStyle = drainageSampleColor(result, mode, sample);
    const x = buffers.x[sample]!, y = buffers.y[sample]!;
    if (fastPoints) context.fillRect(x - 0.75, y - 0.75, 1.5, 1.5);
    else {
      context.beginPath();
      context.arc(x, y, pointRadius, 0, TWO_PI);
      context.fill();
    }
  }
  context.globalAlpha = 1;
  if (mode === 'flow-direction') drawDrainageReceiverOverlay(context, result, projection, width, buffers);
  if (mode === 'basins') drawDrainageOutlets(context, result, buffers);
}
'''
replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    "function scalarColor(value: number, field: ScalarField): string {",
    hydrology_helpers + "\nfunction scalarColor(value: number, field: ScalarField): string {",
)

replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    "function renderPlanet(canvas: HTMLCanvasElement, result: WorldgenClimateResult, projection: string, mode: string, overlays: ReadonlySet<string>, phase: number, yaw: number, pitch: number, buffers: ProjectionBuffers, interactive: boolean, animation: number): void {",
    "function renderPlanet(canvas: HTMLCanvasElement, result: WorldgenClimateResult, drainage: WorldgenDrainageResult | null, projection: string, mode: string, overlays: ReadonlySet<string>, phase: number, yaw: number, pitch: number, buffers: ProjectionBuffers, interactive: boolean, animation: number): void {",
)
replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    "  if (mode === 'mesh') {",
    "  if (isDrainageMode(mode)) {\n    if (!drainage) return;\n    renderDrainageDiagnostic(context, drainage, projection, mode, width, buffers, interactive);\n    drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation);\n    return;\n  }\n  if (mode === 'mesh') {",
)
replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    "let current: WorldgenClimateResult | null = null;\nlet buffers: ProjectionBuffers | null = null;",
    "let current: WorldgenClimateResult | null = null;\nlet currentDrainage: WorldgenDrainageResult | null = null;\nlet buffers: ProjectionBuffers | null = null;",
)
replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    "  renderPlanet(canvas, current, projection.value, visualization.value, selectedOverlays(), orbitalPhase(), yaw, pitch, buffers, interactive, animationPhase);",
    "  renderPlanet(canvas, current, currentDrainage, projection.value, visualization.value, selectedOverlays(), orbitalPhase(), yaw, pitch, buffers, interactive, animationPhase);",
)

replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    "function showMetrics(result: WorldgenClimateResult): void {",
    "function showMetrics(result: WorldgenClimateResult, drainage: WorldgenDrainageResult): void {",
)
replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    "  metric(metrics, 'Duration', `${result.stage.durationMs.toFixed(1)} ms`);\n}",
    "  metric(metrics, 'Climate duration', `${result.stage.durationMs.toFixed(1)} ms`);\n  metric(metrics, 'Hydrology / stage', `v${drainage.engineVersion} · ${drainage.stage.id}@${drainage.stage.version}`);\n  metric(metrics, 'Drainage topology', `${drainage.metrics.basinCount.toLocaleString()} basins · ${drainage.metrics.depressionCount.toLocaleString()} depressions`);\n  metric(metrics, 'Largest contributing area', `${(drainage.metrics.maximumContributingAreaM2 / 1e12).toFixed(3)} million km²`);\n  metric(metrics, 'Deepest depression', `${drainage.metrics.maximumDepressionDepthM.toFixed(1)} m`);\n  metric(metrics, 'Drainage area closure', drainage.metrics.areaConservationRelativeError.toExponential(2));\n  metric(metrics, 'Drainage hash', drainage.metrics.drainageHash);\n  metric(metrics, 'WG-4 surface identity', drainage.topographyHash === result.metrics.topographyHash ? 'Climate / drainage match' : 'MISMATCH');\n}\n",
)

old_generate = '''  status.textContent = 'Generating one physical planet through WG-5 coupled climate in Rust/WASM…';
  try {
    const loaded = await client.generateClimate(
      { seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) },
      handleGenerationProgress,
    );
    current = loaded;
    buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };
    styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
    edgeOverlayCache = { result: null, coastline: new Uint32Array(0), contours: [] };
    showMetrics(loaded); redraw(false); updateAnimation(); finishGenerationTelemetry(loaded);
    status.textContent = `Planet ready through WG-5: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.metrics.spinupYears} climate spin-up years, ${loaded.metrics.meanTemperatureK.toFixed(1)} K global mean.`;
'''
new_generate = '''  status.textContent = 'Generating one physical planet through WG-5 coupled climate in Rust/WASM…';
  currentDrainage = null;
  try {
    const request = { seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) };
    const loaded = await client.generateClimate(request, handleGenerationProgress);
    generationStage.textContent = 'WG-6A drainage topology';
    generationStep.textContent = 'running';
    status.textContent = 'Resolving WG-6A drainage topology on the same deterministic fine surface…';
    const drainage = await client.generateDrainage(request);
    if (drainage.metrics.sampleCount !== loaded.metrics.fineSampleCount) throw new Error('WG-6A drainage sample count does not match the WG-5 fine surface.');
    if (drainage.topographyHash !== loaded.metrics.topographyHash) throw new Error('WG-6A drainage topography identity does not match the WG-5 physical surface.');
    current = loaded;
    currentDrainage = drainage;
    buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };
    styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
    edgeOverlayCache = { result: null, coastline: new Uint32Array(0), contours: [] };
    showMetrics(loaded, drainage); redraw(false); updateAnimation(); finishGenerationTelemetry(loaded);
    generationStep.textContent = `${loaded.metrics.spinupYears} climate spin-up years · ${drainage.metrics.basinCount.toLocaleString()} drainage basins`;
    generationTimer.textContent = formatDuration(performance.now() - generationStartedAt);
    status.textContent = `Planet ready through WG-6A: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${drainage.metrics.basinCount.toLocaleString()} drainage basins, area closure ${drainage.metrics.areaConservationRelativeError.toExponential(2)}.`;
'''
replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    old_generate,
    new_generate,
)

# Regression coverage: the primary GitHub Pages surface must actually use the
# WG-6A transport and refuse mismatched deterministic surfaces.
target = Path("tests/wg6Drainage.test.ts")
text = target.read_text()
text = text.replace("import assert from 'node:assert/strict';\n", "import assert from 'node:assert/strict';\nimport fs from 'node:fs';\n", 1)
text += r'''

test('primary Planet Engine Lab blends WG-6A into the main physical diagnostic surface', () => {
  const page = fs.readFileSync('index.html', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(page, /THROUGH WG-6A/);
  for (const mode of ['contributing-area', 'basins', 'flow-direction', 'depression-depth', 'depressions', 'escape-elevation']) {
    assert.match(page, new RegExp(`value=["']${mode}["']`));
  }
  assert.match(source, /client\.generateDrainage\(request\)/);
  assert.match(source, /drainage\.topographyHash !== loaded\.metrics\.topographyHash/);
  assert.match(source, /currentDrainage/);
  assert.match(source, /renderDrainageDiagnostic/);
});
'''
target.write_text(text)

replace_once(
    "tests/worldgenRewrite.test.ts",
    "Planet Engine source stays independent from legacy gameplay world objects through WG-5",
    "Planet Engine source stays independent from legacy gameplay world objects through WG-6A",
)
replace_once(
    "tests/worldgenRewrite.test.ts",
    "    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',\n  ];",
    "    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',\n    'src/worldgen/diagnostics/worldgenDrainageLabStandalone.ts',\n  ];",
)
