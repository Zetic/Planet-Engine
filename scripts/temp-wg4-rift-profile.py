#!/usr/bin/env python3
import argparse
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument('--direct-scale', type=float, default=1.0)
p.add_argument('--base', type=float, default=0.35)
p.add_argument('--history-floor', type=float, default=0.65)
p.add_argument('--inherited-weight', type=float, default=0.55)
a = p.parse_args()
path = Path('rust/interlink-worldgen/src/topography.rs')
text = path.read_text()
old = '''            GeologicalBoundaryRegime::ContinentalRift => {
                rift[a] = rift[a].max(0.35 + 0.65 * divergence);
                rift[b] = rift[b].max(0.35 + 0.65 * divergence);
            }'''
strength = f"{a.direct_scale:.2f} * ({a.base:.2f} + {1-a.base:.2f} * divergence)"
new = f'''            GeologicalBoundaryRegime::ContinentalRift => {{
                let strength = {strength};
                rift[a] = rift[a].max(strength);
                rift[b] = rift[b].max(strength);
            }}'''
if old not in text:
    raise SystemExit('continental-rift source block drifted')
text = text.replace(old, new, 1)
old_hist = '        rift[i] *= 0.65 + 0.35 * f64::from(inherited.rift_history[i]);'
new_hist = f'        rift[i] *= {a.history_floor:.2f} + {1-a.history_floor:.2f} * f64::from(inherited.rift_history[i]);'
if old_hist not in text:
    raise SystemExit('rift history source multiplier drifted')
text = text.replace(old_hist, new_hist, 1)
old_inherited = 'rift_kernel * rift_focus + 0.55 * f64::from(inherited.rift_history[i])'
new_inherited = f'rift_kernel * rift_focus + {a.inherited_weight:.2f} * f64::from(inherited.rift_history[i])'
if old_inherited not in text:
    raise SystemExit('inherited rift term drifted')
text = text.replace(old_inherited, new_inherited, 1)
path.write_text(text)
