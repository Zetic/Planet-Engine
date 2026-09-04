from pathlib import Path

path = Path('docs/worldgen-rewrite/WG7_EROSION.md')
text = path.read_text()
replacements = {
    '| fine level | coarse level | plates | samples | runtime mean / median (ms) | eroded / depositional | receiver changes | max erosion / deposition (m) | mean land |Δz| (m) | sediment closure | drainage area closure | runoff closure |':
    '| fine level | coarse level | plates | samples | runtime mean / median (ms) | eroded / depositional | receiver changes | max erosion / deposition (m) | mean land abs Δz (m) | sediment closure | drainage area closure | runoff closure |',
    '| fine level | samples | runtime mean / median (ms) | eroded / depositional | receiver changes | max erosion / deposition / |Δz| (m) | mean land |Δz| (m) | sediment closure | drainage area closure | runoff closure |':
    '| fine level | samples | runtime mean / median (ms) | eroded / depositional | receiver changes | max erosion / deposition / abs Δz (m) | mean land abs Δz (m) | sediment closure | drainage area closure | runoff closure |',
}
for old, new in replacements.items():
    if text.count(old) != 1:
        raise SystemExit(f'expected exactly one benchmark header: {old!r}')
    text = text.replace(old, new, 1)
path.write_text(text)
