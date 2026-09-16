#!/usr/bin/env python3
from pathlib import Path

path = Path('rust/interlink-worldgen/src/causal_pipeline.rs')
text = path.read_text()
needle = '''    let mountain_load = (0.58 * mountain_core + 0.27 * root + 0.15 * fold).clamp(0.0, 1.0);\n    let foreland_deflection = 320.0 * foreland * mountain_load.powf(1.20);\n    let collision_relief = crust_scale\n'''
replacement = '''    let mountain_load = (0.58 * mountain_core + 0.27 * root + 0.15 * fold).clamp(0.0, 1.0);\n    let foreland_deflection = 320.0 * foreland * mountain_load.powf(1.20);\n    // Boundary-first modern plates distribute collision systems more evenly and expose broad\n    // low-core portions of accretion belts that the old ownership-growth geometry often buried\n    // inside a larger domain. Preserve those mechanically active continental belts with a\n    // moderate crustal-thickening pedestal rather than forcing every accepted orogen to depend\n    // on a narrow mountain-core raster. Terrane accretion receives the stronger support because\n    // its added crust is mechanically real even where the topographic core remains coastal.\n    let continental_collision_pedestal = if kind == OrogenProvinceKind::ContinentalCollision as u8 {\n        520.0 * intensity * (0.55 + 0.45 * shortening)\n    } else {\n        0.0\n    };\n    let terrane_accretion_pedestal = if kind == OrogenProvinceKind::TerraneAccretion as u8 {\n        1_050.0 * intensity * (0.55 + 0.45 * maturity)\n    } else {\n        0.0\n    };\n    let collision_relief = crust_scale\n'''
if needle in text:
    text = text.replace(needle, replacement, 1)
elif 'terrane_accretion_pedestal' not in text:
    raise SystemExit('collision relief insertion point missing')

old = '''            + 360.0 * intensity\n            - foreland_deflection\n            - 70.0 * suture);\n'''
new = '''            + 360.0 * intensity\n            + continental_collision_pedestal\n            + terrane_accretion_pedestal\n            - foreland_deflection\n            - 70.0 * suture);\n'''
if old in text:
    text = text.replace(old, new, 1)
elif new not in text:
    raise SystemExit('collision relief term insertion point missing')
path.write_text(text)
print('boundary-first orogen support tuning applied')
