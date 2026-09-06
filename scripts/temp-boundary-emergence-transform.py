#!/usr/bin/env python3
from pathlib import Path
import argparse

parser = argparse.ArgumentParser()
parser.add_argument("variant", choices=["strong", "balanced", "balanced2", "balanced3", "extreme"])
args = parser.parse_args()

# Keep the deliberately strong suppression of boundary-following oceanic relief in the balanced
# variants; only restore part of the offset volcanic-arc expression so real island arcs survive.
if args.variant == "strong":
    ridge_scale = "0.03"
    ridge_history = "25.0"
    transitional_scale = "0.45"
    collision_oceanic = "0.05"
    collision_transitional = "0.35"
    arc_oceanic = "0.35"
    arc_transitional = "0.65"
elif args.variant == "balanced":
    ridge_scale = "0.03"
    ridge_history = "25.0"
    transitional_scale = "0.45"
    collision_oceanic = "0.05"
    collision_transitional = "0.35"
    arc_oceanic = "0.60"
    arc_transitional = "0.80"
elif args.variant == "balanced2":
    ridge_scale = "0.03"
    ridge_history = "25.0"
    transitional_scale = "0.45"
    collision_oceanic = "0.05"
    collision_transitional = "0.35"
    arc_oceanic = "0.75"
    arc_transitional = "0.90"
elif args.variant == "balanced3":
    ridge_scale = "0.03"
    ridge_history = "25.0"
    transitional_scale = "0.45"
    collision_oceanic = "0.05"
    collision_transitional = "0.35"
    arc_oceanic = "0.90"
    arc_transitional = "1.00"
else:
    ridge_scale = "0.00"
    ridge_history = "0.0"
    transitional_scale = "0.25"
    collision_oceanic = "0.00"
    collision_transitional = "0.20"
    arc_oceanic = "0.20"
    arc_transitional = "0.45"

path = Path("rust/interlink-worldgen/src/topography.rs")
text = path.read_text()

replacements = [
    (
        "const OCEANIC_RIDGE_DIRECT_RESPONSE_SCALE: f64 = 0.10;",
        f"const OCEANIC_RIDGE_DIRECT_RESPONSE_SCALE: f64 = {ridge_scale};",
    ),
    (
        "            GeologicalBoundaryRegime::TransitionalDivergence => {\n                ridge[a] = ridge[a].max(0.35 + 0.65 * divergence);\n                ridge[b] = ridge[b].max(0.35 + 0.65 * divergence);\n            }",
        f"            GeologicalBoundaryRegime::TransitionalDivergence => {{\n                let strength = {transitional_scale} * (0.10 + 0.90 * divergence);\n                ridge[a] = ridge[a].max(strength);\n                ridge[b] = ridge[b].max(strength);\n            }}",
    ),
    (
        "        orogenic[i] = p.inherited_orogeny_scale_m * f64::from(inherited.orogenic_history[i])\n            + p.collision_uplift_scale_m * collision_kernel * collision_focus;",
        f"        let collision_crust_scale = match inherited.crust_kind[i] {{\n            CRUST_OCEANIC => {collision_oceanic},\n            CRUST_TRANSITIONAL => {collision_transitional},\n            _ => 1.0,\n        }};\n        orogenic[i] = collision_crust_scale\n            * (p.inherited_orogeny_scale_m * f64::from(inherited.orogenic_history[i])\n                + p.collision_uplift_scale_m * collision_kernel * collision_focus);",
    ),
    (
        "        ridge[i] =\n            p.ridge_uplift_scale_m * ridge_kernel + 75.0 * f64::from(inherited.ridge_history[i]);",
        f"        ridge[i] =\n            p.ridge_uplift_scale_m * ridge_kernel + {ridge_history} * f64::from(inherited.ridge_history[i]);",
    ),
    (
        "        arc[i] = p.arc_uplift_scale_m\n            * arc_kernel\n            * (0.65 + 0.35 * f64::from(inherited.volcanic_arc_history[i]));",
        f"        let arc_crust_scale = match inherited.crust_kind[i] {{\n            CRUST_OCEANIC => {arc_oceanic},\n            CRUST_TRANSITIONAL => {arc_transitional},\n            _ => 1.0,\n        }};\n        arc[i] = p.arc_uplift_scale_m\n            * arc_kernel\n            * (0.65 + 0.35 * f64::from(inherited.volcanic_arc_history[i]))\n            * arc_crust_scale;",
    ),
]

for old, new in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one target, found {count}: {old[:100]!r}")
    text = text.replace(old, new, 1)

path.write_text(text)
