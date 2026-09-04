from pathlib import Path

for filename in ["tests/wg4Topography.test.ts", "tests/wg6Drainage.test.ts"]:
    path = Path(filename)
    text = path.read_text()
    text = text.replace("through WG-6D", "through WG-7A")
    text = text.replace("THROUGH WG-6D", "THROUGH WG-7A")
    path.write_text(text)

path = Path("tests/wg4Topography.test.ts")
text = path.read_text()
old = "  assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff\\/discharge, WG-6C lake equilibrium, and WG-6D seasonal hydrology/i);"
new = "  assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff\\/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, and WG-7A fluvial erosion\\/sediment diagnostics/i);"
if text.count(old) != 1:
    raise SystemExit(f"WG-4 cumulative pipeline assertion: expected one target, found {text.count(old)}")
path.write_text(text.replace(old, new, 1))
