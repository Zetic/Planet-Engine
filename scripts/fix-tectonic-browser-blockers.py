from pathlib import Path


# The consolidated main lab uses the WG-7D renderer's actual scheduling/buffer contract.
# Retire assertions that were copied from the removed WG-3.75 standalone inheritance lab.
Path('tests/rendererPerformance.test.ts').write_text(r'''import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');

test('main cumulative renderer coalesces pointer motion to animation frames', () => {
  assert.match(source, /requestAnimationFrame\(/);
  assert.match(source, /if \(frameRequest\) return/);
  assert.match(source, /frameRequest = requestAnimationFrame/);
  assert.match(source, /scheduleRedraw\(true\)/);
});

test('main cumulative renderer reuses projection storage and canvas backing dimensions', () => {
  assert.match(source, /type ProjectionBuffers/);
  assert.match(source, /new Float32Array\(loaded\.metrics\.fineSampleCount\)/);
  assert.match(source, /new Uint8Array\(loaded\.metrics\.fineSampleCount\)/);
  assert.match(source, /if \(canvas\.width !== width\) canvas\.width = width/);
  assert.match(source, /if \(canvas\.height !== height\) canvas\.height = height/);
});

test('main cumulative equirectangular diagnostics avoid oversized static Canvas paths', () => {
  assert.match(source, /const fastPoints = \(projection === 'map' \|\| interactive\) && count > 20_000/);
  assert.match(source, /if \(fastPoints\) context\.fillRect\(x - 0\.75, y - 0\.75, 1\.5, 1\.5\)/);
});
''')


# Protocol 21 is intentional because the cumulative result now transports historical identity
# and morphology channels. Update stale fixed-version browser expectations without weakening any
# shape, range, ancestry, or transport assertions.
for path in Path('tests').glob('*.test.ts'):
    text = path.read_text()
    text = text.replace('WORLDGEN_PROTOCOL_VERSION, 20', 'WORLDGEN_PROTOCOL_VERSION, 21')
    text = text.replace('command.protocolVersion, 20', 'command.protocolVersion, 21')
    text = text.replace('/WORLDGEN_PROTOCOL_VERSION = 20/', '/WORLDGEN_PROTOCOL_VERSION = 21/')
    # Keep test names synchronized with the contract they assert.
    text = text.replace('protocol v17', 'protocol v21')
    text = text.replace('protocol v18', 'protocol v21')
    path.write_text(text)
