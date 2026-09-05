#!/usr/bin/env bash
set -euo pipefail

OUTPUT="$(mktemp)"
trap 'rm -f "${OUTPUT}"' EXIT

cargo run --release -p interlink-worldgen-cli --example lake_sediment_infill_performance -- \
  --seed ci-wg7b-evolution \
  --coarse-level 3 \
  --level 4 \
  --plates 12 \
  --runs 1 | tee "${OUTPUT}"

python - "${OUTPUT}" <<'PY'
import math
import re
import sys
from pathlib import Path

text = Path(sys.argv[1]).read_text()
header = re.search(
    r"seed=ci-wg7b-evolution coarse_level=3 level=4 plates=12 samples=(\d+) runs=1",
    text,
)
infill = re.search(
    r"infill horizon_years=([0-9.eE+-]+) historical_traps=(\d+) filled_depressions=(\d+) "
    r"filled_samples=(\d+) capacity_limited=(\d+) max_fill_m=([0-9.eE+-]+)",
    text,
)
sediment = re.search(
    r"sediment delivered_kg_s=([0-9.eE+-]+) applied_equivalent_kg_s=([0-9.eE+-]+) "
    r"unapplied_kg_s=([0-9.eE+-]+) volume_m3=([0-9.eE+-]+) closure=([0-9.eE+-]+)",
    text,
)
lakes = re.search(
    r"lakes pre=(\d+) post=(\d+) drainage_depressions=(\d+) -> (\d+)",
    text,
)
closure = re.search(
    r"closure runoff=([0-9.eE+-]+) lake=([0-9.eE+-]+) "
    r"seasonal_routing=([0-9.eE+-]+) seasonal_water=([0-9.eE+-]+)",
    text,
)
seasonal = re.search(
    r"seasonal spinup=(\d+) surface_drift_m=([0-9.eE+-]+) max_range_m=([0-9.eE+-]+)",
    text,
)
hashes = re.search(
    r"hash infill=([0-9a-f]{16}) surface=([0-9a-f]{16}) drainage=([0-9a-f]{16}) "
    r"runoff=([0-9a-f]{16}) lake=([0-9a-f]{16}) seasonal=([0-9a-f]{16}) parameters=([0-9a-f]{16})",
    text,
)
if not all((header, infill, sediment, lakes, closure, seasonal, hashes)):
    raise SystemExit("WG-7D smoke output did not match the expected diagnostics contract")

samples = int(header.group(1))
horizon = float(infill.group(1))
traps = int(infill.group(2))
filled_depressions = int(infill.group(3))
filled_samples = int(infill.group(4))
capacity_limited = int(infill.group(5))
max_fill = float(infill.group(6))
delivered = float(sediment.group(1))
applied = float(sediment.group(2))
unapplied = float(sediment.group(3))
volume = float(sediment.group(4))
sediment_closure = float(sediment.group(5))
pre_lakes = int(lakes.group(1))
post_lakes = int(lakes.group(2))
pre_depressions = int(lakes.group(3))
post_depressions = int(lakes.group(4))
runoff_closure = float(closure.group(1))
lake_closure = float(closure.group(2))
routing_closure = float(closure.group(3))
seasonal_closure = float(closure.group(4))
spinup = int(seasonal.group(1))
surface_drift = float(seasonal.group(2))
max_range = float(seasonal.group(3))

if samples != 2562:
    raise SystemExit(f"WG-7D L4 smoke expected 2562 samples, got {samples}")
if not math.isfinite(horizon) or horizon <= 0.0 or horizon > 250000.0001:
    raise SystemExit(f"WG-7D geomorphic horizon is invalid: {horizon}")
if traps <= 0 or filled_depressions <= 0 or filled_samples <= 0:
    raise SystemExit(
        f"WG-7D fixed smoke must exercise lake infill: traps={traps} filled_depressions={filled_depressions} filled_samples={filled_samples}"
    )
if capacity_limited < 0 or capacity_limited > traps:
    raise SystemExit("WG-7D capacity-limited depression count is outside trap bounds")
if not math.isfinite(max_fill) or max_fill <= 0.0 or max_fill > 120.001:
    raise SystemExit(f"WG-7D maximum fill depth is outside accepted bounds: {max_fill}")
if pre_lakes <= 0 or post_lakes < 0 or pre_depressions <= 0 or post_depressions < 0:
    raise SystemExit("WG-7D fixed smoke must retain meaningful lake/depression diagnostics")

for name, value in {
    "delivered sediment": delivered,
    "applied sediment": applied,
    "unapplied sediment": unapplied,
    "applied fill volume": volume,
    "sediment closure": sediment_closure,
    "runoff closure": runoff_closure,
    "lake closure": lake_closure,
    "seasonal routing closure": routing_closure,
    "seasonal water closure": seasonal_closure,
    "seasonal surface drift": surface_drift,
    "seasonal range": max_range,
}.items():
    if not math.isfinite(value) or value < 0.0:
        raise SystemExit(f"WG-7D {name} must be finite and nonnegative, got {value}")

if delivered <= 0.0 or applied <= 0.0 or volume <= 0.0:
    raise SystemExit("WG-7D fixed smoke must apply nonzero historical lake sediment")
if applied > delivered * (1.0 + 1.0e-8):
    raise SystemExit(f"WG-7D applied sediment exceeds accepted delivery: {applied} > {delivered}")
if abs((applied + unapplied) - delivered) > max(1.0e-6, delivered * 1.0e-6):
    raise SystemExit("WG-7D applied + unapplied sediment does not close to historical delivery")
if sediment_closure > 1.0e-8:
    raise SystemExit(f"WG-7D sediment conservation exceeded tolerance: {sediment_closure:.3e}")
for name, value in {
    "runoff conservation": runoff_closure,
    "lake water balance": lake_closure,
    "seasonal routing": routing_closure,
    "seasonal water balance": seasonal_closure,
}.items():
    if value > 1.0e-10:
        raise SystemExit(f"WG-7D {name} exceeded tolerance: {value:.3e}")
if not (1 <= spinup <= 24):
    raise SystemExit(f"WG-7D reconciled seasonal spinup is outside [1, 24]: {spinup}")
if surface_drift > 0.0200001:
    raise SystemExit(f"WG-7D post-infill seasonal lake cycle did not converge: {surface_drift:.9f} m")

print(
    f"WG-7D lake-infill smoke accepted: samples={samples} traps={traps} filled={filled_depressions}/{filled_samples} "
    f"max_fill_m={max_fill:.6f} delivered={delivered:.6f} applied={applied:.6f} unapplied={unapplied:.6f} "
    f"sediment_closure={sediment_closure:.3e} runoff={runoff_closure:.3e} lake={lake_closure:.3e} "
    f"seasonal_route={routing_closure:.3e} seasonal_water={seasonal_closure:.3e}"
)
PY
