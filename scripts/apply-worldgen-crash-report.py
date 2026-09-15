from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if new in text:
        return
    if old not in text:
        raise SystemExit(f"missing patch anchor in {path}: {old[:100]!r}")
    p.write_text(text.replace(old, new, 1))


# Protocol: allow worker-side transport diagnostics to ride the existing progress channel.
replace_once(
    'src/worldgen/protocol.ts',
    "export interface WorldgenGenerationTiming { stageId: string; durationMs: number; }\nexport interface WorldgenGenerationProgress {\n  stageId: string;\n  stageIndex: number;\n  stageCount: number;\n  completed: number;\n  total: number;\n  elapsedMs: number;\n  stageElapsedMs: number;\n}\n",
    "export interface WorldgenGenerationTiming { stageId: string; durationMs: number; }\nexport interface WorldgenGenerationDiagnostics { transferBufferCount?: number; transferBytes?: number; note?: string; }\nexport interface WorldgenGenerationProgress {\n  stageId: string;\n  stageIndex: number;\n  stageCount: number;\n  completed: number;\n  total: number;\n  elapsedMs: number;\n  stageElapsedMs: number;\n  diagnostics?: WorldgenGenerationDiagnostics;\n}\n",
)

# Worker: emit a tiny checkpoint after packaging is fully materialized but immediately before the giant result transfer.
replace_once(
    'src/worldgen/worldgenWorker.ts',
    "    if (command.type === 'generate-climate') {\n      const result = await generateClimate(command);\n      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-climate', payload: result }, collectTransferables(result)); return;\n    }",
    "    if (command.type === 'generate-climate') {\n      const result = await generateClimate(command);\n      const transferables = collectTransferables(result);\n      const transferBytes = transferables.reduce((sum, item) => sum + (item instanceof ArrayBuffer ? item.byteLength : 0), 0);\n      workerScope.postMessage({\n        protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'progress',\n        payload: {\n          stageId: 'transport-ready', stageIndex: 17, stageCount: 18, completed: 1, total: 1,\n          elapsedMs: result.stage.durationMs, stageElapsedMs: 0,\n          diagnostics: { transferBufferCount: transferables.length, transferBytes, note: 'worker result materialized; next operation is generated-climate postMessage' },\n        },\n      });\n      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-climate', payload: result }, transferables); return;\n    }",
)

# Lab imports.
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "import { createWorldgenClient } from '../worldgenClient.js';\n",
    "import { createWorldgenClient } from '../worldgenClient.js';\nimport { createWorldgenCrashRecorder, installWorldgenGlobalFailureCapture } from '../worldgenCrashReport.js';\n",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "  WORLDGEN_INVALID_SAMPLE_ID,\n  type WorldgenClimateResult,",
    "  WORLDGEN_INVALID_SAMPLE_ID,\n  WORLDGEN_PROTOCOL_VERSION,\n  type WorldgenClimateResult,",
)

# Lab controls / recorder wiring.
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "const copyCalibration = element<HTMLButtonElement>('worldgen-copy-calibration');\nconst downloadCalibration = element<HTMLButtonElement>('worldgen-download-calibration');\nconst status = element<HTMLElement>('worldgen-status');",
    "const copyCalibration = element<HTMLButtonElement>('worldgen-copy-calibration');\nconst downloadCalibration = element<HTMLButtonElement>('worldgen-download-calibration');\nconst copyCrashReport = element<HTMLButtonElement>('worldgen-copy-crash-report');\nconst downloadCrashReport = element<HTMLButtonElement>('worldgen-download-crash-report');\nconst receiveOnlyDebug = element<HTMLInputElement>('worldgen-debug-receive-only');\nconst debugSummary = element<HTMLElement>('worldgen-debug-summary');\nconst status = element<HTMLElement>('worldgen-status');",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "const globeRenderer = new L8GlobeRenderer(surfaceCanvas);\nconst client = createWorldgenClient();",
    "const globeRenderer = new L8GlobeRenderer(surfaceCanvas);\nconst crashRecorder = createWorldgenCrashRecorder(WORLDGEN_PROTOCOL_VERSION);\nconst removeGlobalFailureCapture = installWorldgenGlobalFailureCapture(crashRecorder);\nconst client = createWorldgenClient({ onDiagnostic: diagnostic => {\n  crashRecorder.record('client', diagnostic.event, { requestId: diagnostic.requestId, commandType: diagnostic.commandType, ...diagnostic.details });\n  refreshCrashDebugSummary();\n} });",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "  packaging: 'Packaging / transfer',\n};",
    "  packaging: 'Packaging / transfer',\n  'transport-ready': 'Worker result ready for transfer',\n};",
)

# Debug summary and event persistence.
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "let generationStartedAt = 0;\nlet generationTimerHandle: ReturnType<typeof setInterval> | null = null;\n\nfunction selectedOverlays(): Set<string> {",
    "let generationStartedAt = 0;\nlet generationTimerHandle: ReturnType<typeof setInterval> | null = null;\n\nfunction refreshCrashDebugSummary(): void {\n  const snapshot = crashRecorder.snapshot();\n  copyCrashReport.disabled = !crashRecorder.hasAttempt();\n  downloadCrashReport.disabled = !crashRecorder.hasAttempt();\n  if (!crashRecorder.hasAttempt()) { debugSummary.textContent = 'No generation attempt recorded yet.'; return; }\n  const transport = snapshot.transport ? ` · packet ${snapshot.transport.totalMiB.toFixed(1)} MiB / ${snapshot.transport.arrayCount} arrays` : '';\n  const failure = snapshot.failure ? ` · ${snapshot.failure.name}: ${snapshot.failure.message}` : '';\n  debugSummary.textContent = `${snapshot.status} · last ${snapshot.lastCheckpoint ?? 'none'} · ${snapshot.events.length} events${transport}${failure}`;\n}\n\nfunction selectedOverlays(): Set<string> {",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "function handleGenerationProgress(progress: WorldgenGenerationProgress): void {\n  const stageFraction = progress.total > 0 ? Math.max(0, Math.min(1, progress.completed / progress.total)) : 0;",
    "function handleGenerationProgress(progress: WorldgenGenerationProgress): void {\n  crashRecorder.recordProgress(progress);\n  refreshCrashDebugSummary();\n  const stageFraction = progress.total > 0 ? Math.max(0, Math.min(1, progress.completed / progress.total)) : 0;",
)

# Copy/download crash report UI logic.
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "async function copyCalibrationReport(): Promise<void> {",
    "async function copyCrashReportToClipboard(): Promise<void> {\n  if (!crashRecorder.hasAttempt()) return;\n  try {\n    await navigator.clipboard.writeText(crashRecorder.toMarkdown());\n    status.textContent = 'Copied the full worldgen crash/debug report to the clipboard.';\n  } catch (error) {\n    status.textContent = `Could not copy crash report: ${error instanceof Error ? error.message : String(error)}`;\n  }\n}\nfunction downloadCrashReportJson(): void {\n  if (!crashRecorder.hasAttempt()) return;\n  const snapshot = crashRecorder.snapshot();\n  const blob = new Blob([crashRecorder.toJson()], { type: 'application/json;charset=utf-8' });\n  const url = URL.createObjectURL(blob);\n  const anchor = document.createElement('a');\n  anchor.href = url;\n  anchor.download = `planet-crash-report-${calibrationFileStem(snapshot.request?.seed ?? 'worldgen')}-${snapshot.runId.slice(0, 8)}.json`;\n  document.body.appendChild(anchor);\n  anchor.click();\n  anchor.remove();\n  URL.revokeObjectURL(url);\n  status.textContent = 'Downloaded structured worldgen crash/debug report.';\n}\n\nasync function copyCalibrationReport(): Promise<void> {",
)

# Generation lifecycle checkpoints + receive-only diagnostic mode.
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "async function generatePlanet(): Promise<void> {\n  generate.disabled = true;\n  copyCalibration.disabled = true;\n  downloadCalibration.disabled = true;\n  startGenerationTelemetry();\n  status.textContent = 'Generating one physical planet through WG-7D lake sediment infill in Rust/WASM…';\n  try {\n    const request = { seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) };\n    const loaded = await client.generateClimate(request, handleGenerationProgress);",
    "async function generatePlanet(): Promise<void> {\n  const request = { seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) };\n  crashRecorder.startRun(request, receiveOnlyDebug.checked);\n  refreshCrashDebugSummary();\n  generate.disabled = true;\n  copyCalibration.disabled = true;\n  downloadCalibration.disabled = true;\n  startGenerationTelemetry();\n  status.textContent = receiveOnlyDebug.checked\n    ? 'Diagnostic receive-only run: generating through WG-7D and stopping immediately after transport / validation…'\n    : 'Generating one physical planet through WG-7D lake sediment infill in Rust/WASM…';\n  try {\n    crashRecorder.record('lab', 'generate-climate-await-begin', { receiveOnly: receiveOnlyDebug.checked });\n    const loaded = await client.generateClimate(request, handleGenerationProgress);\n    crashRecorder.record('lab', 'generate-climate-promise-resolved', { fineSampleCount: loaded.metrics.fineSampleCount });\n    crashRecorder.recordResult(loaded);\n    crashRecorder.record('lab', 'identity-validation-begin');\n    refreshCrashDebugSummary();",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "    if (loaded.infillMetrics.postInfillSeasonalHash !== loaded.seasonalMetrics.seasonalHydrologyHash) throw new Error('WG-7D final seasonal identity mismatch.');\n    current = loaded;",
    "    if (loaded.infillMetrics.postInfillSeasonalHash !== loaded.seasonalMetrics.seasonalHydrologyHash) throw new Error('WG-7D final seasonal identity mismatch.');\n    crashRecorder.record('lab', 'identity-validation-complete');\n    refreshCrashDebugSummary();\n    if (receiveOnlyDebug.checked) {\n      crashRecorder.record('lab', 'receive-only-viewer-skipped', { transportMiB: crashRecorder.snapshot().transport?.totalMiB });\n      finishGenerationTelemetry(loaded);\n      generationStage.textContent = 'Receive-only complete';\n      generationStep.textContent = 'Transport + identity validation succeeded; viewer allocation/render intentionally skipped';\n      generationTimer.textContent = formatDuration(performance.now() - generationStartedAt);\n      status.textContent = `Receive-only diagnostic succeeded: ${loaded.metrics.fineSampleCount.toLocaleString()} samples crossed the worker boundary. Viewer setup was skipped.`;\n      crashRecorder.complete('receive-only-complete');\n      refreshCrashDebugSummary();\n      return;\n    }\n    crashRecorder.record('lab', 'viewer-state-install-begin');\n    current = loaded;",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "    buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };",
    "    crashRecorder.record('lab', 'viewer-projection-buffer-allocation-begin', { fineSampleCount: loaded.metrics.fineSampleCount });\n    buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };\n    crashRecorder.record('lab', 'viewer-projection-buffer-allocation-complete', { bytes: buffers.x.byteLength + buffers.y.byteLength + buffers.visible.byteLength });",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "    showMetrics(loaded); redraw(false); updateAnimation(); finishGenerationTelemetry(loaded);",
    "    crashRecorder.record('lab', 'viewer-cache-reset-complete');\n    crashRecorder.record('lab', 'viewer-metrics-render-begin');\n    showMetrics(loaded);\n    crashRecorder.record('lab', 'viewer-metrics-render-complete');\n    crashRecorder.record('lab', 'viewer-first-render-begin');\n    redraw(false);\n    crashRecorder.record('lab', 'viewer-first-render-complete');\n    updateAnimation();\n    crashRecorder.record('lab', 'viewer-animation-armed');\n    finishGenerationTelemetry(loaded);\n    crashRecorder.record('lab', 'viewer-state-install-complete');",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "    status.textContent = `Planet ready through WG-7D: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.evolutionMetrics.erodedSampleCount.toLocaleString()} evolved erosion cells, ${loaded.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} drainage receivers changed, mean land |Δz| ${loaded.evolutionMetrics.meanLandAbsoluteTerrainChangeM.toFixed(3)} m, sediment closure ${loaded.evolutionMetrics.sedimentConservationRelativeError.toExponential(2)}.`;\n  } catch (error) {",
    "    status.textContent = `Planet ready through WG-7D: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.evolutionMetrics.erodedSampleCount.toLocaleString()} evolved erosion cells, ${loaded.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} drainage receivers changed, mean land |Δz| ${loaded.evolutionMetrics.meanLandAbsoluteTerrainChangeM.toFixed(3)} m, sediment closure ${loaded.evolutionMetrics.sedimentConservationRelativeError.toExponential(2)}.`;\n    crashRecorder.complete('completed');\n    refreshCrashDebugSummary();\n  } catch (error) {",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "  } catch (error) {\n    if (generationTimerHandle) { clearInterval(generationTimerHandle); generationTimerHandle = null; }\n    generationStage.textContent = 'Generation failed';\n    generationStep.textContent = '';\n    status.textContent = error instanceof Error ? error.message : String(error);\n  } finally {",
    "  } catch (error) {\n    if (generationTimerHandle) { clearInterval(generationTimerHandle); generationTimerHandle = null; }\n    crashRecorder.recordError(error, 'generate-planet');\n    const snapshot = crashRecorder.snapshot();\n    generationStage.textContent = 'Generation failed';\n    generationStep.textContent = `Last checkpoint: ${snapshot.lastCheckpoint ?? 'unknown'}`;\n    status.textContent = error instanceof Error ? error.message : String(error);\n    refreshCrashDebugSummary();\n  } finally {",
)

# Event handlers / cleanup.
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "downloadCalibration.addEventListener('click', downloadCalibrationReport);",
    "downloadCalibration.addEventListener('click', downloadCalibrationReport);\ncopyCrashReport.addEventListener('click', () => void copyCrashReportToClipboard());\ndownloadCrashReport.addEventListener('click', downloadCrashReportJson);",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "  globeRenderer.dispose();\n  client.dispose();",
    "  crashRecorder.record('browser', 'beforeunload');\n  removeGlobalFailureCapture();\n  globeRenderer.dispose();\n  client.dispose();",
)
replace_once(
    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',
    "updateOverlaySummary();\napplyViewPreset(preset.value);",
    "updateOverlaySummary();\nrefreshCrashDebugSummary();\napplyViewPreset(preset.value);",
)

# HTML controls and report status panel.
replace_once(
    'index.html',
    "      <button id=\"worldgen-copy-calibration\" type=\"button\" disabled>Copy LLM Summary</button>\n      <button id=\"worldgen-download-calibration\" type=\"button\" disabled>Download Calibration JSON</button>",
    "      <button id=\"worldgen-copy-calibration\" type=\"button\" disabled>Copy LLM Summary</button>\n      <button id=\"worldgen-download-calibration\" type=\"button\" disabled>Download Calibration JSON</button>\n      <button id=\"worldgen-copy-crash-report\" type=\"button\" disabled>Copy Crash Report</button>\n      <button id=\"worldgen-download-crash-report\" type=\"button\" disabled>Download Debug JSON</button>\n      <label class=\"worldgen-debug-toggle\"><input id=\"worldgen-debug-receive-only\" type=\"checkbox\"> Debug: receive only (skip viewer)</label>",
)
replace_once(
    'index.html',
    "        <div id=\"worldgen-generation-profile\" class=\"worldgen-generation-profile\"></div>",
    "        <div id=\"worldgen-generation-profile\" class=\"worldgen-generation-profile\"></div>\n        <div class=\"worldgen-debug-report\">\n          <strong>Crash debugger</strong>\n          <span id=\"worldgen-debug-summary\">No generation attempt recorded yet.</span>\n        </div>",
)

# Styling for debug controls / report status.
css = Path('styles/worldgenLab.css')
css_text = css.read_text()
marker = '/* worldgen crash debugger */'
if marker not in css_text:
    css.write_text(css_text + "\n\n/* worldgen crash debugger */\n.worldgen-debug-toggle { align-self: center; grid-auto-flow: column; align-items: center; gap: 6px !important; padding: 7px 9px; border: 1px solid #2b3b50; border-radius: 4px; background: #0c1622; }\n.worldgen-debug-toggle input { width: auto !important; margin: 0; }\n.worldgen-debug-report { display: grid; gap: 4px; margin-top: 10px; padding: 8px 10px; border: 1px solid #2b3b50; border-radius: 4px; background: #0a131e; color: #8fa6bd; font-size: 11px; overflow-wrap: anywhere; }\n.worldgen-debug-report strong { color: #b8cee4; font-weight: 600; }\n")

# Regression coverage for the diagnostic plumbing.
wg5 = Path('tests/wg5Climate.test.ts')
wg5_text = wg5.read_text()
needle = "  assert.match(source, /handleGenerationProgress/);\n});"
replacement = "  assert.match(source, /handleGenerationProgress/);\n  assert.match(html, /id=\\\"worldgen-copy-crash-report\\\"/);\n  assert.match(html, /id=\\\"worldgen-debug-receive-only\\\"/);\n  assert.match(source, /createWorldgenCrashRecorder/);\n  assert.match(source, /receive-only-viewer-skipped/);\n  const clientSource = fs.readFileSync('src/worldgen/worldgenClient.ts', 'utf8');\n  const workerSource = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');\n  assert.match(clientSource, /messageerror/);\n  assert.match(clientSource, /result-message-resolving/);\n  assert.match(workerSource, /transport-ready/);\n  assert.match(workerSource, /transferBytes/);\n});"
if replacement not in wg5_text:
    if needle not in wg5_text:
        raise SystemExit('missing wg5 test anchor')
    wg5.write_text(wg5_text.replace(needle, replacement, 1))

# Focused pure recorder tests.
crash_test = Path('tests/worldgenCrashReport.test.ts')
if not crash_test.exists():
    crash_test.write_text("""import assert from 'node:assert/strict';
import test from 'node:test';
import { createWorldgenCrashRecorder, formatWorldgenCrashReport, summarizeTypedArrayPayload } from '../src/worldgen/worldgenCrashReport.ts';

test('worldgen crash report measures nested typed-array transport without retaining payload data', () => {
  const payload = { a: new Float32Array(4), nested: { b: new Uint8Array(3), c: new Uint32Array(2) } };
  const summary = summarizeTypedArrayPayload(payload);
  assert.equal(summary.arrayCount, 3);
  assert.equal(summary.totalBytes, 27);
  assert.equal(summary.largest[0]?.byteLength, 16);
  assert.equal(summary.largest[0]?.path, 'result.a');
});

test('worldgen crash recorder keeps request, progress checkpoint, and failure details', () => {
  const recorder = createWorldgenCrashRecorder(19);
  recorder.clear();
  recorder.startRun({ seed: 'debug-seed', coarseLevel: 5, fineLevel: 8, plateCount: 16 }, false);
  recorder.recordProgress({ stageId: 'transport-ready', stageIndex: 17, stageCount: 18, completed: 1, total: 1, elapsedMs: 1234, stageElapsedMs: 0, diagnostics: { transferBufferCount: 12, transferBytes: 4096 } });
  recorder.recordError(new Error('synthetic failure'), 'test');
  const report = recorder.snapshot();
  assert.equal(report.status, 'failed');
  assert.equal(report.request?.seed, 'debug-seed');
  assert.equal(report.lastCheckpoint, 'test:error');
  assert.match(report.failure?.message ?? '', /synthetic failure/);
  assert.ok(report.events.some(event => event.event === 'transport-ready'));
  assert.match(formatWorldgenCrashReport(report), /Planet Engine Worldgen Crash Report/);
  assert.match(formatWorldgenCrashReport(report), /synthetic failure/);
});
""")

print('worldgen crash report integration applied')
