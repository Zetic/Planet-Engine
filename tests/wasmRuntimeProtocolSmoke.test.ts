import assert from 'node:assert/strict';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';
import initWasm, { worldgen_protocol_version } from '../src/wasm-worldgen/interlink_worldgen_wasm.js';

test('packaged WASM runtime reports the browser protocol version', async () => {
  await initWasm();
  assert.equal(worldgen_protocol_version(), WORLDGEN_PROTOCOL_VERSION);
});
