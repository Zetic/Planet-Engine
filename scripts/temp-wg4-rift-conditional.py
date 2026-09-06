#!/usr/bin/env python3
from pathlib import Path
path = Path('rust/interlink-worldgen/src/topography.rs')
text = path.read_text()
old = '''            GeologicalBoundaryRegime::ContinentalRift => {
                rift[a] = rift[a].max(0.35 + 0.65 * divergence);
                rift[b] = rift[b].max(0.35 + 0.65 * divergence);
            }'''
new = '''            GeologicalBoundaryRegime::ContinentalRift => {
                let strength = 0.10 * (0.05 + 0.95 * divergence);
                rift[a] = rift[a].max(strength);
                rift[b] = rift[b].max(strength);
            }'''
if old not in text:
    raise SystemExit('continental-rift source block drifted')
text = text.replace(old, new, 1)
old_hist = '        rift[i] *= 0.65 + 0.35 * f64::from(inherited.rift_history[i]);'
new_hist = '        rift[i] *= 0.05 + 0.95 * f64::from(inherited.rift_history[i]);'
if old_hist not in text:
    raise SystemExit('rift source history multiplier drifted')
text = text.replace(old_hist, new_hist, 1)
old_relief = '''        rift_basin[i] = -(p.rift_subsidence_scale_m
            * (rift_kernel * rift_focus + 0.55 * f64::from(inherited.rift_history[i]))
            + p.basin_subsidence_scale_m'''
new_relief = '''        let inherited_rift_relief_weight = match inherited.crust_kind[i] {
            CRUST_TRANSITIONAL => 0.55,
            CRUST_OCEANIC => 0.0,
            _ => 0.05,
        };
        rift_basin[i] = -(p.rift_subsidence_scale_m
            * (rift_kernel * rift_focus
                + inherited_rift_relief_weight * f64::from(inherited.rift_history[i]))
            + p.basin_subsidence_scale_m'''
if old_relief not in text:
    raise SystemExit('rift relief block drifted')
text = text.replace(old_relief, new_relief, 1)
path.write_text(text)
