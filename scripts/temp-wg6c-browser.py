from pathlib import Path


def replace_required(path: str, old: str, new: str, count: int = 1) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f'marker not found in {path}: {old[:220]!r}')
    target.write_text(text.replace(old, new, count))

# Protocol v13: cumulative result now includes WG-6C equilibrium lake state.
replace_required('src/worldgen/protocol.ts', 'export const WORLDGEN_PROTOCOL_VERSION = 12;', 'export const WORLDGEN_PROTOCOL_VERSION = 13;')
replace_required(
    'src/worldgen/protocol.ts',
    '  potentialDischargeM3S: Float32Array;\n}',
    '''  potentialDischargeM3S: Float32Array;
  lakeStage: WorldgenStageMetadata;
  lakeMetrics: WorldgenLakeMetrics;
  lakeId: Uint32Array;
  lakeKind: Uint8Array;
  lakeFraction: Float32Array;
  lakeDepthM: Float32Array;
  realizedDischargeM3S: Float32Array;
  lakeDepressionIds: Uint32Array;
  lakeKinds: Uint8Array;
  lakeSurfaceElevationsM: Float64Array;
  lakeAreasM2: Float64Array;
  lakeVolumesM3: Float64Array;
  lakeOutflowsM3S: Float64Array;
  lakeSpillSamples: Uint32Array;
}''',
)
replace_required(
    'src/worldgen/protocol.ts',
    'export interface WorldgenDrainageResult {',
    '''export interface WorldgenLakeMetrics {
  sampleCount: number;
  lakeCount: number;
  endorheicLakeCount: number;
  overflowingLakeCount: number;
  terminalStorageLakeCount: number;
  lakeSampleCount: number;
  totalLakeAreaM2: number;
  totalLakeVolumeM3: number;
  maximumLakeAreaM2: number;
  maximumLakeDepthM: number;
  totalLakePrecipitationM3S: number;
  totalLakeEvaporationM3S: number;
  terminalRealizedDischargeM3S: number;
  maximumRealizedDischargeM3S: number;
  unreleasedStorageM3S: number;
  waterBalanceRelativeError: number;
  lakeParameterHash: string;
  climateHash: string;
  drainageHash: string;
  runoffHash: string;
  lakeHash: string;
}

export interface WorldgenDrainageResult {''',
)

# Worker WASM contract and cumulative packaging.
replace_required(
    'src/worldgen/worldgenWorker.ts',
    '  actual_evapotranspiration_mm(): Float32Array; local_runoff_mm(): Float32Array; runoff_fraction(): Float32Array; local_runoff_m3_s(): Float32Array; potential_discharge_m3_s(): Float32Array;\n  free(): void;',
    '''  actual_evapotranspiration_mm(): Float32Array; local_runoff_mm(): Float32Array; runoff_fraction(): Float32Array; local_runoff_m3_s(): Float32Array; potential_discharge_m3_s(): Float32Array;
  lake_stage_id(): string; lake_stage_version(): number; lake_stage_seed_hex(): string; lake_hash_hex(): string; lake_parameter_hash_hex(): string; lake_climate_hash_hex(): string; lake_drainage_hash_hex(): string; lake_runoff_hash_hex(): string;
  lake_count(): number; endorheic_lake_count(): number; overflowing_lake_count(): number; terminal_storage_lake_count(): number; lake_sample_count(): number;
  total_lake_area_m2(): number; total_lake_volume_m3(): number; maximum_lake_area_m2(): number; maximum_lake_depth_m(): number; total_lake_precipitation_m3_s(): number; total_lake_evaporation_m3_s(): number; terminal_realized_discharge_m3_s(): number; maximum_realized_discharge_m3_s(): number; unreleased_storage_m3_s(): number; lake_water_balance_relative_error(): number;
  lake_id(): Uint32Array; lake_kind(): Uint8Array; lake_fraction(): Float32Array; lake_depth_m(): Float32Array; realized_discharge_m3_s(): Float32Array;
  lake_depression_ids(): Uint32Array; lake_kinds(): Uint8Array; lake_surface_elevations_m(): Float64Array; lake_areas_m2(): Float64Array; lake_volumes_m3(): Float64Array; lake_outflows_m3_s(): Float64Array; lake_spill_samples(): Uint32Array;
  free(): void;''',
)
replace_required('src/worldgen/worldgenWorker.ts', "progress('packaging', 11, 12, 0, 1);", "progress('packaging', 12, 13, 0, 1);")
replace_required('src/worldgen/worldgenWorker.ts', "progress('packaging', 11, 12, 1, 1);", "progress('packaging', 12, 13, 1, 1);")
replace_required(
    'src/worldgen/worldgenWorker.ts',
    '    const actualEvapotranspirationMm = output.actual_evapotranspiration_mm(); const localRunoffMm = output.local_runoff_mm(); const runoffFraction = output.runoff_fraction(); const localRunoffM3S = output.local_runoff_m3_s(); const potentialDischargeM3S = output.potential_discharge_m3_s();',
    '''    const actualEvapotranspirationMm = output.actual_evapotranspiration_mm(); const localRunoffMm = output.local_runoff_mm(); const runoffFraction = output.runoff_fraction(); const localRunoffM3S = output.local_runoff_m3_s(); const potentialDischargeM3S = output.potential_discharge_m3_s();
    const lakeId = output.lake_id(); const lakeKind = output.lake_kind(); const lakeFraction = output.lake_fraction(); const lakeDepthM = output.lake_depth_m(); const realizedDischargeM3S = output.realized_discharge_m3_s();
    const lakeDepressionIds = output.lake_depression_ids(); const lakeKinds = output.lake_kinds(); const lakeSurfaceElevationsM = output.lake_surface_elevations_m(); const lakeAreasM2 = output.lake_areas_m2(); const lakeVolumesM3 = output.lake_volumes_m3(); const lakeOutflowsM3S = output.lake_outflows_m3_s(); const lakeSpillSamples = output.lake_spill_samples();''',
)
replace_required(
    'src/worldgen/worldgenWorker.ts',
    '      actualEvapotranspirationMm, localRunoffMm, runoffFraction, localRunoffM3S, potentialDischargeM3S,\n    };',
    '''      actualEvapotranspirationMm, localRunoffMm, runoffFraction, localRunoffM3S, potentialDischargeM3S,
      lakeStage: { id: output.lake_stage_id(), version: output.lake_stage_version(), stageSeed: output.lake_stage_seed_hex(), durationMs: 0 },
      lakeMetrics: { sampleCount: output.fine_sample_count(), lakeCount: output.lake_count(), endorheicLakeCount: output.endorheic_lake_count(), overflowingLakeCount: output.overflowing_lake_count(), terminalStorageLakeCount: output.terminal_storage_lake_count(), lakeSampleCount: output.lake_sample_count(), totalLakeAreaM2: output.total_lake_area_m2(), totalLakeVolumeM3: output.total_lake_volume_m3(), maximumLakeAreaM2: output.maximum_lake_area_m2(), maximumLakeDepthM: output.maximum_lake_depth_m(), totalLakePrecipitationM3S: output.total_lake_precipitation_m3_s(), totalLakeEvaporationM3S: output.total_lake_evaporation_m3_s(), terminalRealizedDischargeM3S: output.terminal_realized_discharge_m3_s(), maximumRealizedDischargeM3S: output.maximum_realized_discharge_m3_s(), unreleasedStorageM3S: output.unreleased_storage_m3_s(), waterBalanceRelativeError: output.lake_water_balance_relative_error(), lakeParameterHash: output.lake_parameter_hash_hex(), climateHash: output.lake_climate_hash_hex(), drainageHash: output.lake_drainage_hash_hex(), runoffHash: output.lake_runoff_hash_hex(), lakeHash: output.lake_hash_hex() },
      lakeId, lakeKind, lakeFraction, lakeDepthM, realizedDischargeM3S, lakeDepressionIds, lakeKinds, lakeSurfaceElevationsM, lakeAreasM2, lakeVolumesM3, lakeOutflowsM3S, lakeSpillSamples,
    };''',
)
replace_required(
    'src/worldgen/worldgenWorker.ts',
    'result.potentialDischargeM3S.buffer]); return;',
    'result.potentialDischargeM3S.buffer, result.lakeId.buffer, result.lakeKind.buffer, result.lakeFraction.buffer, result.lakeDepthM.buffer, result.realizedDischargeM3S.buffer, result.lakeDepressionIds.buffer, result.lakeKinds.buffer, result.lakeSurfaceElevationsM.buffer, result.lakeAreasM2.buffer, result.lakeVolumesM3.buffer, result.lakeOutflowsM3S.buffer, result.lakeSpillSamples.buffer]); return;',
)

# Single canonical Lab entrypoint advances to WG-6C.
replace_required('index.html', '<title>Planet Engine · Through WG-6B</title>', '<title>Planet Engine · Through WG-6C</title>')
replace_required('index.html', 'PLANET ENGINE · THROUGH WG-6B', 'PLANET ENGINE · THROUGH WG-6C')
replace_required('index.html', 'Generate one deterministic physical planet through WG-6B,', 'Generate one deterministic physical planet through WG-6C,')
replace_required('index.html', 'value="interlink-wg6b"', 'value="interlink-wg6c"')
replace_required(
    'index.html',
    '            <select id="worldgen-visualization">\n          <optgroup label="Hydrology · Runoff / discharge (WG-6B)">',
    '''            <select id="worldgen-visualization">
          <optgroup label="Hydrology · Lakes / realized flow (WG-6C)">
            <option value="realized-discharge">Realized annual discharge</option>
            <option value="lake-depth">Equilibrium lake depth</option>
            <option value="lake-state">Lake state</option>
            <option value="lake-fraction">Lake surface fraction</option>
          </optgroup>
          <optgroup label="Hydrology · Runoff / discharge (WG-6B)">''',
)
replace_required('index.html', '<strong>Current physical frontier: WG-6B</strong>', '<strong>Current physical frontier: WG-6C</strong>')
replace_required(
    'index.html',
    'One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, and WG-6B annual runoff/discharge in one cumulative Rust/WASM result. WG-6B is required to reference the exact accepted WG-5 climate and WG-6A drainage identities.',
    'One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff/discharge, and WG-6C lake equilibrium in one cumulative Rust/WASM result. WG-6C is required to reference the exact accepted WG-5, WG-6A, and WG-6B identities.',
)
replace_required(
    'index.html',
    'Seasonal precipitation displays the retained final spin-up-year orbital phases; WG-6A remains terrain-only; WG-6B consumes the accepted annual precipitation and PET forcing to produce actual evapotranspiration, runoff, and potential routed discharge.',
    'Seasonal precipitation displays the retained final spin-up-year orbital phases; WG-6A remains terrain-only; WG-6B produces potential routed discharge; WG-6C closes depression water balance into dry basins, endorheic lakes, or overflowing lakes and exposes realized discharge.',
)
replace_required(
    'index.html',
    'The physical surface remains pre-erosional. Lake water balance and spill activation, seasonal discharge, river incision, sediment transport, glacier flow, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.',
    'The physical surface remains pre-erosional. Seasonal discharge and snowmelt timing, river incision, sediment transport, glacier flow, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.',
)

# Lab renderer and telemetry.
lab = Path('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts')
text = lab.read_text()
marker = "const RUNOFF_MODES = new Set(['annual-runoff', 'runoff-fraction', 'actual-et', 'potential-discharge']);"
if marker not in text:
    raise SystemExit('runoff modes marker missing')
lake_code = '''const LAKE_MODES = new Set(['realized-discharge', 'lake-depth', 'lake-state', 'lake-fraction']);
function isLakeMode(mode: string): boolean { return LAKE_MODES.has(mode); }
function lakeSampleColor(result: WorldgenClimateResult, mode: string, sample: number): string {
  if (result.submergedMask[sample]) return '#102c43';
  if (mode === 'lake-state') {
    const kind = result.lakeKind[sample]!;
    if (kind === 1) return '#3aa7c9';
    if (kind === 2) return '#63d0a5';
    if (kind === 3) return '#9b78d0';
    return '#31423c';
  }
  if (mode === 'lake-fraction') return drainageScalarColor(result.lakeFraction[sample]!, 210, 175);
  if (mode === 'lake-depth') {
    const depth = Math.max(0, result.lakeDepthM[sample]!);
    if (depth <= 0) return '#31423c';
    const maxDepth = Math.max(1, result.lakeMetrics.maximumLakeDepthM);
    return drainageScalarColor(Math.log1p(depth) / Math.log1p(maxDepth), 220, 175);
  }
  const maxValue = Math.max(1e-6, result.lakeMetrics.maximumRealizedDischargeM3S);
  return drainageScalarColor(Math.log1p(Math.max(0, result.realizedDischargeM3S[sample]!)) / Math.log1p(maxValue), 205, 35);
}

'''
text = text.replace(marker, lake_code + marker, 1)
render_marker = "  if (isRunoffMode(mode)) {"
if render_marker not in text:
    raise SystemExit('render runoff marker missing')
lake_render = '''  if (isLakeMode(mode)) {
    const count = result.metrics.fineSampleCount;
    const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
    const fastPoints = interactive && count > 20_000;
    context.globalAlpha = 0.94;
    for (let sample = 0; sample < count; sample += 1) {
      if (!buffers.visible[sample]) continue;
      context.fillStyle = lakeSampleColor(result, mode, sample);
      const x = buffers.x[sample]!, y = buffers.y[sample]!;
      if (fastPoints) context.fillRect(x - 0.75, y - 0.75, 1.5, 1.5);
      else { context.beginPath(); context.arc(x, y, pointRadius, 0, TWO_PI); context.fill(); }
    }
    context.globalAlpha = 1;
    drawDiagnosticOverlays(context, result, overlays, phase, projection, yaw, pitch, width, height, buffers, animation);
    return;
  }
'''
text = text.replace(render_marker, lake_render + render_marker, 1)
text = text.replace("  'climate-spinup': 'Climate spin-up',\n  packaging: 'Packaging / transfer',", "  'climate-spinup': 'Climate spin-up',\n  'drainage-topology': 'Drainage topology',\n  'runoff-discharge': 'Annual runoff / discharge',\n  'lake-equilibrium': 'Lake equilibrium',\n  packaging: 'Packaging / transfer',", 1)
metrics_marker = "  metric(metrics, 'WG-6B runoff hash', result.runoffMetrics.runoffHash);\n}"
if metrics_marker not in text:
    raise SystemExit('metrics marker missing')
metrics_add = """  metric(metrics, 'WG-6B runoff hash', result.runoffMetrics.runoffHash);
  metric(metrics, 'WG-6C / stage', `v${result.engineVersion} · ${result.lakeStage.id}@${result.lakeStage.version}`);
  metric(metrics, 'WG-6C lakes', `${result.lakeMetrics.lakeCount.toLocaleString()} total · ${result.lakeMetrics.endorheicLakeCount.toLocaleString()} endorheic · ${result.lakeMetrics.overflowingLakeCount.toLocaleString()} overflowing · ${result.lakeMetrics.terminalStorageLakeCount.toLocaleString()} terminal storage`);
  metric(metrics, 'WG-6C lake area / volume', `${(result.lakeMetrics.totalLakeAreaM2 / 1e12).toFixed(3)} million km² · ${(result.lakeMetrics.totalLakeVolumeM3 / 1e12).toFixed(3)} thousand km³`);
  metric(metrics, 'WG-6C deepest lake', `${result.lakeMetrics.maximumLakeDepthM.toFixed(1)} m`);
  metric(metrics, 'WG-6C lake evaporation', `${result.lakeMetrics.totalLakeEvaporationM3S.toFixed(1)} m³/s`);
  metric(metrics, 'WG-6C terminal realized flow', `${result.lakeMetrics.terminalRealizedDischargeM3S.toFixed(1)} m³/s`);
  metric(metrics, 'WG-6C water balance', result.lakeMetrics.waterBalanceRelativeError.toExponential(2));
  metric(metrics, 'WG-6C lake hash', result.lakeMetrics.lakeHash);
}"
text = text.replace(metrics_marker, metrics_add, 1)
identity_marker = "    if (loaded.runoffMetrics.drainageHash !== loaded.drainageMetrics.drainageHash) throw new Error('WG-6B drainage identity does not match accepted WG-6A topology.');"
if identity_marker not in text:
    raise SystemExit('identity marker missing')
identity_add = identity_marker + "\n    if (loaded.lakeMetrics.climateHash !== loaded.metrics.climateHash) throw new Error('WG-6C climate identity does not match accepted WG-5 forcing.');\n    if (loaded.lakeMetrics.drainageHash !== loaded.drainageMetrics.drainageHash) throw new Error('WG-6C drainage identity does not match accepted WG-6A topology.');\n    if (loaded.lakeMetrics.runoffHash !== loaded.runoffMetrics.runoffHash) throw new Error('WG-6C runoff identity does not match accepted WG-6B runoff.');"
text = text.replace(identity_marker, identity_add, 1)
text = text.replace("generationStep.textContent = `${loaded.metrics.spinupYears} climate spin-up years · ${loaded.drainageMetrics.basinCount.toLocaleString()} basins · ${loaded.runoffMetrics.meanLandRunoffMm.toFixed(1)} mm/yr land runoff`;", "generationStep.textContent = `${loaded.metrics.spinupYears} climate spin-up years · ${loaded.drainageMetrics.basinCount.toLocaleString()} basins · ${loaded.lakeMetrics.lakeCount.toLocaleString()} equilibrium lakes`;", 1)
text = text.replace("status.textContent = `Planet ready through WG-6B: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.drainageMetrics.basinCount.toLocaleString()} drainage basins, max potential discharge ${loaded.runoffMetrics.maximumPotentialDischargeM3S.toFixed(1)} m³/s.`;", "status.textContent = `Planet ready through WG-6C: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.lakeMetrics.lakeCount.toLocaleString()} equilibrium lakes, max realized discharge ${loaded.lakeMetrics.maximumRealizedDischargeM3S.toFixed(1)} m³/s.`;", 1)
lab.write_text(text)

# Update current protocol/frontier expectations in existing tests.
for path in Path('tests').glob('*.test.ts'):
    text = path.read_text()
    text = text.replace('protocolVersion: 12', 'protocolVersion: 13')
    text = text.replace('protocolVersion, 12', 'protocolVersion, 13')
    text = text.replace('WORLDGEN_PROTOCOL_VERSION, 12', 'WORLDGEN_PROTOCOL_VERSION, 13')
    text = text.replace('const PROTOCOL = 12;', 'const PROTOCOL = 13;')
    text = text.replace('protocol v12', 'protocol v13')
    text = text.replace('through WG-6B', 'through WG-6C')
    text = text.replace('THROUGH WG-6B', 'THROUGH WG-6C')
    path.write_text(text)

Path('tests/wg6Lakes.test.ts').write_text('''import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-6C cumulative browser contract is protocol v13 and single-request', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 13);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  const html = fs.readFileSync('index.html', 'utf8');
  for (const field of ['lakeMetrics', 'lakeId', 'lakeKind', 'lakeFraction', 'lakeDepthM', 'realizedDischargeM3S']) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  for (const mode of ['realized-discharge', 'lake-depth', 'lake-state', 'lake-fraction']) assert.match(html, new RegExp(mode));
  assert.match(lab, /loaded\\.lakeMetrics\\.climateHash !== loaded\\.metrics\\.climateHash/);
  assert.match(lab, /loaded\\.lakeMetrics\\.drainageHash !== loaded\\.drainageMetrics\\.drainageHash/);
  assert.match(lab, /loaded\\.lakeMetrics\\.runoffHash !== loaded\\.runoffMetrics\\.runoffHash/);
  assert.doesNotMatch(lab, /client\\.generateDrainage\\(/);
  assert.match(lab, /client\\.generateClimate\\(request, handleGenerationProgress\\)/);
  assert.ok(!fs.existsSync('drainage.html'));
  assert.ok(!fs.existsSync('worldgen-lab.html'));
});
''')
