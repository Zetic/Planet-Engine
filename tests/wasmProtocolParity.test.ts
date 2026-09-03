import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

test('browser and WASM protocol versions remain synchronized', () => {
  const browserProtocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const wasmProtocol = fs.readFileSync('rust/interlink-worldgen-wasm/src/lib.rs', 'utf8');
  const browserMatch = browserProtocol.match(/WORLDGEN_PROTOCOL_VERSION\s*=\s*(\d+)/);
  const wasmMatch = wasmProtocol.match(/WORLDGEN_WASM_PROTOCOL_VERSION:\s*u32\s*=\s*(\d+)/);
  assert.ok(browserMatch, 'browser protocol version constant must be present');
  assert.ok(wasmMatch, 'WASM protocol version constant must be present');
  assert.equal(
    Number(wasmMatch[1]),
    Number(browserMatch[1]),
    'Rust/WASM and browser protocol versions must match',
  );
});
