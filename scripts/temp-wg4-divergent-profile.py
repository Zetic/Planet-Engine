#!/usr/bin/env python3
import argparse
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--ridge-direct", type=float, default=0.50)
parser.add_argument("--ridge-history-m", type=float, default=500.0)
parser.add_argument("--continental-rift-basin-reduction", type=float, default=0.0)
args = parser.parse_args()

path = Path("rust/interlink-worldgen/src/topography.rs")
text = path.read_text()

old = "const OCEANIC_RIDGE_DIRECT_RESPONSE_SCALE: f64 = 0.50;"
new = f"const OCEANIC_RIDGE_DIRECT_RESPONSE_SCALE: f64 = {args.ridge_direct:.2f};"
if old not in text:
    raise SystemExit("ridge direct-response constant drifted")
text = text.replace(old, new, 1)

old = "p.ridge_uplift_scale_m * ridge_kernel + 500.0 * f64::from(inherited.ridge_history[i]);"
new = f"p.ridge_uplift_scale_m * ridge_kernel + {args.ridge_history_m:.1f} * f64::from(inherited.ridge_history[i]);"
if old not in text:
    raise SystemExit("ridge history uplift expression drifted")
text = text.replace(old, new, 1)

if args.continental_rift_basin_reduction > 0.0:
    old = """        rift_basin[i] = -(p.rift_subsidence_scale_m
            * (rift_kernel * rift_focus
                + inherited_rift_relief_weight * f64::from(inherited.rift_history[i]))
            + p.basin_subsidence_scale_m
                * (0.55 * f64::from(inherited.basin_potential[i])
                    + 0.45 * f64::from(inherited.subsidence_history[i])));"""
    weight = args.continental_rift_basin_reduction
    new = f"""        let rift_basin_history_weight = match inherited.crust_kind[i] {{
            CRUST_OCEANIC | CRUST_TRANSITIONAL => 1.0,
            _ => 1.0 - {weight:.2f} * f64::from(inherited.rift_history[i]).clamp(0.0, 1.0),
        }};
        rift_basin[i] = -(p.rift_subsidence_scale_m
            * (rift_kernel * rift_focus
                + inherited_rift_relief_weight * f64::from(inherited.rift_history[i]))
            + p.basin_subsidence_scale_m
                * rift_basin_history_weight
                * (0.55 * f64::from(inherited.basin_potential[i])
                    + 0.45 * f64::from(inherited.subsidence_history[i])));"""
    if old not in text:
        raise SystemExit("rift basin expression drifted")
    text = text.replace(old, new, 1)

path.write_text(text)
