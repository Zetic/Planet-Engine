from pathlib import Path

path = Path('tests/wg4Topography.test.ts')
text = path.read_text()
text = text.replace("test('WG-4 browser protocol v7 exposes bounded coarse-to-fine topography generation'", "test('WG-4 browser contract remains available under protocol v8'", 1)
text = text.replace('assert.equal(WORLDGEN_PROTOCOL_VERSION, 7);', 'assert.equal(WORLDGEN_PROTOCOL_VERSION, 8);', 1)
text = text.replace('protocolVersion: 7, requestId: 77', 'protocolVersion: 8, requestId: 77', 1)
path.write_text(text)

path = Path('tests/worldgenRewrite.test.ts')
text = path.read_text()
text = text.replace('const PROTOCOL = 7;', 'const PROTOCOL = 8;', 1)
text = text.replace("test('Planet Engine browser protocol v7 preserves WG-0 through WG-3.75 contracts'", "test('Planet Engine browser protocol v8 preserves WG-0 through WG-3.75 contracts'", 1)
path.write_text(text)
