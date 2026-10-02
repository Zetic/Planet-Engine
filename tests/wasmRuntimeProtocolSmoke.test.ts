import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';
import initWasm, { WasmWorldgenClimate, worldgen_protocol_version } from '../src/wasm-worldgen/interlink_worldgen_wasm.js';

test('packaged WASM runtime reports the browser protocol version', async () => {
  const wasmBytes = readFileSync(new URL('../src/wasm-worldgen/interlink_worldgen_wasm_bg.wasm', import.meta.url));
  await initWasm(wasmBytes);
  assert.equal(worldgen_protocol_version(), WORLDGEN_PROTOCOL_VERSION);

  const climate = new WasmWorldgenClimate('freeboard-observability-smoke', 2, 3, 8);
  try {
    const sampleCounts = climate.freeboard_causal_sample_counts();
    const stateCounts = climate.freeboard_continental_state_counts();
    const fields = [
      climate.freeboard_mean_crust_density_kg_per_m3(),
      climate.freeboard_mean_rift_history(),
      climate.freeboard_mean_subsidence_history(),
      climate.freeboard_mean_basin_potential(),
      climate.freeboard_mean_crustal_strain(),
      climate.freeboard_mean_compensated_buoyancy_index(),
      climate.freeboard_mean_effective_elastic_thickness_km(),
      climate.freeboard_mean_structural_fabric_strength(),
    ];
    assert.equal(sampleCounts.length, 5);
    assert.equal(stateCounts.length, 3);
    assert.ok(sampleCounts[0] > 0, 'smoke world should contain continental samples');
    for (const values of fields) {
      assert.equal(values.length, 5);
      for (let bucket = 0; bucket < values.length; bucket += 1) {
        if (sampleCounts[bucket] > 0) assert.ok(Number.isFinite(values[bucket]), `bucket ${bucket} causal mean must be finite`);
      }
    }
  } finally {
    climate.free();
  }
});
