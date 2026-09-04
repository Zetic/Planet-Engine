from pathlib import Path

path = Path('tests/wg4Topography.test.ts')
text = path.read_text()
old = r"  assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, and WG-6B annual runoff\/discharge/i);"
new = r"  assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff\/discharge, and WG-6C lake equilibrium/i);"
if old not in text:
    raise SystemExit('WG-4 cumulative WG-6C assertion marker missing')
path.write_text(text.replace(old, new, 1))
