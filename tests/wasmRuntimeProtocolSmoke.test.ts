import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';
import initWasm, { worldgen_protocol_version } from '../src/wasm-worldgen/interlink_worldgen_wasm.js';

test('packaged WASM runtime reports the browser protocol version', async () => {
  const wasmBytes = readFileSync(new URL('../src/wasm-worldgen/interlink_worldgen_wasm_bg.wasm', import.meta.url));
  await initWasm(wasmBytes);
  assert.equal(worldgen_protocol_version(), WORLDGEN_PROTOCOL_VERSION);
});
