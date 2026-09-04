from pathlib import Path

path = Path("src/worldgen/diagnostics/worldgenClimateLabStandalone.ts")
text = path.read_text()
old = "  if (EROSION_MODES.has(mode) && result.submergedMask[sample]) return '#102c43';"
new = "  if (EROSION_MODES.has(mode) && mode !== 'erosion-sediment-deposition' && result.submergedMask[sample]) return '#102c43';"
if text.count(old) != 1:
    raise SystemExit(f"WG-7A ocean diagnostic mask: expected one target, found {text.count(old)}")
path.write_text(text.replace(old, new, 1))

path = Path("tests/wg7Erosion.test.ts")
text = path.read_text()
anchor = "  assert.match(lab, /erosionMetrics\\.topographyHash/);\n"
addition = anchor + "  assert.match(lab, /mode !== 'erosion-sediment-deposition'/);\n"
if text.count(anchor) != 1:
    raise SystemExit(f"WG-7A deposition visualization regression: expected one target, found {text.count(anchor)}")
path.write_text(text.replace(anchor, addition, 1))
