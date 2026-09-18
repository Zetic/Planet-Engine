import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_CLIMATE_COARSE_MAX_LEVEL, WORLDGEN_CLIMATE_FINE_MAX_LEVEL } from '../dist/worldgen/protocol.js';

test('Planet Engine Lab defaults generation controls to accepted maximum fidelity', () => {
  assert.equal(WORLDGEN_CLIMATE_COARSE_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_CLIMATE_FINE_MAX_LEVEL, 8);
  const html = fs.readFileSync('index.html', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(html, /id="worldgen-coarse-level"[^>]*max="6"[^>]*value="6"/);
  assert.match(html, /id="worldgen-level"[^>]*max="8"[^>]*value="8"/);
  assert.match(source, /coarseLevel\.value = String\(WORLDGEN_CLIMATE_COARSE_MAX_LEVEL\)/);
  assert.match(source, /fineLevel\.value = String\(WORLDGEN_CLIMATE_FINE_MAX_LEVEL\)/);
});

test('diagnostics are grouped by physical domain and carry an in-view legend', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  const css = fs.readFileSync('styles/worldgenLab.css', 'utf8');
  assert.match(html, /id="worldgen-diagnostic-category"/);
  assert.match(html, /id="worldgen-diagnostic-legend"/);
  for (const category of ['world','tectonics-history','crust-lithosphere','lithology-substrate','topography-forcing','climate','hydrology','geomorphology','technical']) {
    assert.match(html, new RegExp('data-diagnostic-category="' + category + '"'));
  }
  assert.match(source, /function refreshDiagnosticLegend/);
  assert.match(source, /CATEGORICAL_LEGENDS/);
  assert.match(source, /selectedDiagnosticSampleText/);
  assert.match(css, /\.worldgen-diagnostic-legend/);
  assert.match(css, /\.worldgen-legend-ramp/);
  assert.match(css, /\.worldgen-legend-swatches/);
});

test('every selectable diagnostic belongs to exactly one category', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  const a = html.indexOf('<select id="worldgen-visualization">');
  const b = html.indexOf('</select>', a);
  assert.ok(a >= 0 && b > a);
  const fragment = html.slice(a, b);
  const values = Array.from(fragment.matchAll(/<option value="([^"]+)"/g), match => match[1]);
  assert.ok(values.length > 70);
  assert.equal(new Set(values).size, values.length);
  const groups = Array.from(fragment.matchAll(/<optgroup[^>]*data-diagnostic-category="([^"]+)"[^>]*>([\s\S]*?)<\/optgroup>/g));
  const grouped = groups.flatMap(match => Array.from(match[2].matchAll(/<option value="([^"]+)"/g), option => option[1]));
  assert.deepEqual(grouped.sort(), values.sort());
});
