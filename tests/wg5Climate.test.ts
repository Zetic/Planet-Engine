import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {
  WORLDGEN_CLIMATE_COARSE_MAX_LEVEL,
  WORLDGEN_CLIMATE_FINE_MAX_LEVEL,
  WORLDGEN_PROTOCOL_VERSION,
  validateClimateRequest,
  worldgenClimateCommand,
} from '../dist/worldgen/protocol.js';
import { mapVectorDelta, reconstructAnnualHarmonic } from '../dist/worldgen/diagnostics/worldgenClimateMath.js';

test('WG-5 browser protocol is versioned and bounded', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 14);
  assert.equal(WORLDGEN_CLIMATE_COARSE_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_CLIMATE_FINE_MAX_LEVEL, 7);
  const request = { seed: 'wg5-browser', coarseLevel: 3, fineLevel: 4, plateCount: 12 };
  assert.doesNotThrow(() => validateClimateRequest(request));
  assert.deepEqual(worldgenClimateCommand(91, request), {
    protocolVersion: 14,
    requestId: 91,
    type: 'generate-climate',
    payload: request,
  });
  assert.throws(() => validateClimateRequest({ ...request, seed: '' }), /seed/i);
  assert.throws(() => validateClimateRequest({ ...request, coarseLevel: 5, fineLevel: 4 }), /fine level/i);
});

test('cumulative WG-5 Lab exposes climate diagnostics and stored seasonal reconstruction', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(html, /id="worldgen-season"/);
  for (const label of [
    'Annual mean temperature', 'Seasonal temperature', 'Seasonal prevailing winds',
    'Annual precipitation', 'Seasonal precipitation', 'Annual precipitation seasonality', 'Aridity index',
    'Persistent snow potential', 'Sea-ice potential', 'Physical elevation / bathymetry',
  ]) assert.match(html, new RegExp(label, 'i'));
  assert.match(source, /generateClimate/);
  assert.doesNotMatch(source, /generateTopography\(|generateInheritance\(/);
  for (const field of [
    'temperatureAnnualCosK', 'temperatureAnnualSinK',
    'windEastAnnualCosMS', 'windEastAnnualSinMS',
    'seaSurfaceTemperatureAnnualCosK', 'seaSurfaceTemperatureAnnualSinK',
    'currentEastAnnualCosMS', 'currentEastAnnualSinMS',
  ]) assert.match(source, new RegExp(field));
  assert.match(source, /requestAnimationFrame/);
  assert.match(source, /VECTOR_ANIMATION_INTERVAL_MS\s*=\s*50/);
  assert.match(source, /redraw\(true\)/);
});

test('WG-5 Lab preserves viewport dimensions while splitting diagnostics, overlays, and details', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  const css = fs.readFileSync('styles/worldgenLab.css', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(html, /class="worldgen-lab-diagnostics"/);
  assert.match(html, /id="worldgen-overlays"/);
  assert.match(html, /Topographic contours/);
  assert.match(html, /id="worldgen-projection"/);
  assert.match(html, /class="worldgen-lab-details"/);
  assert.match(html, /id="worldgen-generation-progress"/);
  assert.match(css, /grid-template-columns:\s*minmax\(0, 1fr\) minmax\(260px, 340px\)/);
  assert.match(source, /const width = 1100;/);
  assert.match(source, /projection === 'map' \? 550 : 760/);
  assert.match(source, /precipitationPhaseRateMmYear/);
  assert.match(source, /drawDiagnosticOverlays/);
  assert.match(source, /handleGenerationProgress/);
});

test('WG-5 browser transport preserves seasonal SST and current harmonics', () => {
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');
  for (const field of [
    'seaSurfaceTemperatureAnnualCosK', 'seaSurfaceTemperatureAnnualSinK',
    'currentEastAnnualCosMS', 'currentEastAnnualSinMS',
    'currentNorthAnnualCosMS', 'currentNorthAnnualSinMS',
  ]) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  for (const field of [
    'sea_surface_temperature_annual_cos_k', 'sea_surface_temperature_annual_sin_k',
    'current_east_annual_cos_m_s', 'current_east_annual_sin_m_s',
    'current_north_annual_cos_m_s', 'current_north_annual_sin_m_s',
  ]) assert.match(bridge, new RegExp(field));
});


test('WG-5 seasonal reconstruction is numerically defined by stored annual harmonics', () => {
  assert.equal(reconstructAnnualHarmonic(10, 2, 3, 0), 12);
  assert.ok(Math.abs(reconstructAnnualHarmonic(10, 2, 3, 0.25) - 13) < 1e-12);
  assert.ok(Math.abs(reconstructAnnualHarmonic(10, 2, 3, 0.5) - 8) < 1e-12);
});

test('WG-5 flat-map vector projection uses local ENU components directly', () => {
  const [eastDx, eastDy] = mapVectorDelta(8, 0, 0, 1100, 550);
  assert.ok(eastDx > 0);
  assert.ok(Math.abs(eastDy) < 1e-12);
  const [northDx, northDy] = mapVectorDelta(0, 8, 0, 1100, 550);
  assert.ok(Math.abs(northDx) < 1e-12);
  assert.ok(northDy < 0);
  const [eastAtMidLatDx] = mapVectorDelta(8, 0, Math.PI / 4, 1100, 550);
  assert.ok(eastAtMidLatDx > eastDx, 'equirectangular longitude scale should expand by 1/cos(latitude)');
});

test('WG-5 climate hash source covers every public climate output vector', () => {
  const source = fs.readFileSync('rust/interlink-worldgen/src/climate.rs', 'utf8');
  for (const field of [
    'annual_mean_insolation_f32', 'seasonal_insolation_amplitude',
    'temperature_mean', 'temperature_annual_cos', 'temperature_annual_sin', 'temperature_min_f32', 'temperature_max_f32',
    'pressure_f32', 'wind_east_mean', 'wind_north_mean', 'wind_east_cos_out', 'wind_east_sin_out', 'wind_north_cos_out', 'wind_north_sin_out',
    'sst_mean', 'sst_cos_out', 'sst_sin_out', 'current_east_mean', 'current_north_mean',
    'current_east_cos_out', 'current_east_sin_out', 'current_north_cos_out', 'current_north_sin_out',
    'current_speed_mean', 'ocean_heat_transport', 'humidity_mean', 'annual_precipitation_mm',
    'precipitation_seasonality', 'potential_evaporation_mm', 'moisture_balance_mm', 'aridity_index',
    'snowfall_fraction', 'persistent_snow_potential', 'sea_ice_potential',
  ]) assert.match(source, new RegExp(`&${field}[,\\n]`));
});
