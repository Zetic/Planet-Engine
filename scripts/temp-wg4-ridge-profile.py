from __future__ import annotations

import argparse
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--base", type=float, required=True)
parser.add_argument("--multiplier", type=float, default=1.0)
parser.add_argument("--width-km", type=float, default=450.0)
args = parser.parse_args()

path = Path("rust/interlink-worldgen/src/topography.rs")
text = path.read_text()
old = '''            GeologicalBoundaryRegime::OceanicRidge
            | GeologicalBoundaryRegime::TransitionalDivergence => {
                ridge[a] = ridge[a].max(0.35 + 0.65 * divergence);
                ridge[b] = ridge[b].max(0.35 + 0.65 * divergence);
            }
'''
base = args.base
slope = 1.0 - base
multiplier = args.multiplier
new = f'''            GeologicalBoundaryRegime::OceanicRidge => {{
                ridge[a] = ridge[a].max({multiplier:.6f} * ({base:.6f} + {slope:.6f} * divergence));
                ridge[b] = ridge[b].max({multiplier:.6f} * ({base:.6f} + {slope:.6f} * divergence));
            }}
            GeologicalBoundaryRegime::TransitionalDivergence => {{
                ridge[a] = ridge[a].max(0.35 + 0.65 * divergence);
                ridge[b] = ridge[b].max(0.35 + 0.65 * divergence);
            }}
'''
if text.count(old) != 1:
    raise RuntimeError("expected one combined oceanic/transitional ridge source arm")
text = text.replace(old, new, 1)
old_width = "            ridge_width_m: 450_000.0,"
new_width = f"            ridge_width_m: {args.width_km * 1000.0:.1f},"
if text.count(old_width) != 1:
    raise RuntimeError("expected one calibrated 450 km ridge width default")
text = text.replace(old_width, new_width, 1)
path.write_text(text)
