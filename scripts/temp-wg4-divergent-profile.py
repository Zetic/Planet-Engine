#!/usr/bin/env python3
import argparse
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--ridge-direct", type=float, default=0.50)
parser.add_argument("--ridge-history-m", type=float, default=500.0)
parser.add_argument("--continental-rift-direct", type=float, default=0.10)
parser.add_argument("--continental-rift-width-m", type=float, default=450_000.0)
parser.add_argument("--continental-rift-basin-reduction", type=float, default=0.0)
parser.add_argument("--continental-rift-offaxis-basin-reduction", type=float, default=0.0)
args = parser.parse_args()

if args.continental_rift_basin_reduction > 0.0 and args.continental_rift_offaxis_basin_reduction > 0.0:
    raise SystemExit("choose either broad or off-axis continental-rift basin reduction, not both")

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

old = "const CONTINENTAL_RIFT_DIRECT_RESPONSE_SCALE: f64 = 0.10;"
new = f"const CONTINENTAL_RIFT_DIRECT_RESPONSE_SCALE: f64 = {args.continental_rift_direct:.2f};"
if old not in text:
    raise SystemExit("continental-rift direct-response constant drifted")
text = text.replace(old, new, 1)

old = "            rift_width_m: 450_000.0,"
width = f"{int(args.continental_rift_width_m):_}.0"
new = f"            rift_width_m: {width},"
if old not in text:
    raise SystemExit("continental-rift width default drifted")
text = text.replace(old, new, 1)

original_rift_basin = """        rift_basin[i] = -(p.rift_subsidence_scale_m
            * (rift_kernel * rift_focus
                + inherited_rift_relief_weight * f64::from(inherited.rift_history[i]))
            + p.basin_subsidence_scale_m
                * (0.55 * f64::from(inherited.basin_potential[i])
                    + 0.45 * f64::from(inherited.subsidence_history[i])));"""

if args.continental_rift_offaxis_basin_reduction > 0.0:
    weight = args.continental_rift_offaxis_basin_reduction
    focused = f"""        let rift_axis_focus = gaussian(rift_distance[i], p.rift_width_m);
        let rift_basin_history_weight = match inherited.crust_kind[i] {{
            CRUST_OCEANIC | CRUST_TRANSITIONAL => 1.0,
            _ => 1.0
                - {weight:.2f}
                    * f64::from(inherited.rift_history[i]).clamp(0.0, 1.0)
                    * (1.0 - rift_axis_focus),
        }};
        rift_basin[i] = -(p.rift_subsidence_scale_m
            * (rift_kernel * rift_focus
                + inherited_rift_relief_weight * f64::from(inherited.rift_history[i]))
            + p.basin_subsidence_scale_m
                * rift_basin_history_weight
                * (0.55 * f64::from(inherited.basin_potential[i])
                    + 0.45 * f64::from(inherited.subsidence_history[i])));"""
    if original_rift_basin not in text:
        raise SystemExit("rift basin expression drifted")
    text = text.replace(original_rift_basin, focused, 1)
elif args.continental_rift_basin_reduction > 0.0:
    weight = args.continental_rift_basin_reduction
    broad = f"""        let rift_basin_history_weight = match inherited.crust_kind[i] {{
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
    if original_rift_basin not in text:
        raise SystemExit("rift basin expression drifted")
    text = text.replace(original_rift_basin, broad, 1)

path.write_text(text)
