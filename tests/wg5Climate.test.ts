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

test('WG-5 browser protocol is versioned and bounded', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 8);
  assert.equal(WORLDGEN_CLIMATE_COARSE_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_CLIMATE_FINE_MAX_LEVEL, 7);
  const request = { seed: 'wg5-browser', coarseLevel: 3, fineLevel: 4, plateCount: 12 };
  assert.doesNotThrow(() => validateClimateRequest(request));
  assert.deepEqual(worldgenClimateCommand(91, request), {
    protocolVersion: 8,
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
    'Seasonal surface ocean currents', 'Annual precipitation', 'Aridity index',
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
