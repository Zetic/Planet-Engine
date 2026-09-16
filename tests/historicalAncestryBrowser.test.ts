import fs from 'node:fs';
import test from 'node:test';
import assert from 'node:assert/strict';

test('historical material ancestry browser diagnostic exposes all three identity levels', () => {
  const html = fs.readFileSync('inheritance.html', 'utf8');
  assert.match(html, /WasmWorldgenInheritance/);
  assert.match(html, /origin_plate_ids/);
  assert.match(html, /historical_fragment_ids/);
  assert.match(html, /current_plate_ids/);
  assert.match(html, /crust_birth_age_myr/);
  assert.match(html, /Ancestral origin plates/);
  assert.match(html, /Persistent crust fragments/);
  assert.match(html, /Current plate ownership/);
  assert.match(html, /Fossil discontinuities/);
  assert.match(html, /Oceanic birth-age span/);
});

test('WG-3.75 WASM bridge exposes ancestry explicitly without redefining compatibility fields', () => {
  const source = fs.readFileSync('rust/interlink-worldgen-wasm/src/inheritance_bridge.rs', 'utf8');
  assert.match(source, /pub fn historical_identity_hash_hex/);
  assert.match(source, /pub fn origin_plate_ids[\s\S]*self\.historical_identity\.origin_plate_ids/);
  assert.match(source, /pub fn historical_fragment_ids[\s\S]*self\.historical_identity\.fragment_ids/);
  assert.match(source, /pub fn current_plate_ids[\s\S]*self\.historical_identity\.current_plate_ids/);
  assert.match(source, /pub fn crust_birth_age_myr[\s\S]*self\.historical_identity\.crust_birth_age_myr/);
  assert.match(source, /pub fn plate_ids[\s\S]*self\.inner\.plate_ids/);
  assert.match(source, /pub fn crust_province_id[\s\S]*self\.inner\.crust_province_id/);
  assert.match(source, /pub fn crust_age_myr[\s\S]*self\.inner\.crust_age_myr/);
  assert.match(source, /pub fn fragment_ids[\s\S]*self\.inner\.fragment_ids/);
});
