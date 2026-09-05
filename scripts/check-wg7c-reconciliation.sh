#!/usr/bin/env bash
set -euo pipefail

OUTPUT="$(mktemp)"
trap 'rm -f "${OUTPUT}"' EXIT

cargo run --release -p interlink-worldgen-cli --example post_erosion_hydrology_performance -- \
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
lakes = re.search(
    r"lakes pre=(\d+) post=(\d+) kind_changed=(\d+) added=(\d+) removed=(\d+) "
    r"max_depth_delta_m=([0-9.eE+-]+)",
    text,
)
flow = re.search(
    r"flow regime_changed=(\d+) max_presence_delta=([0-9.eE+-]+) "
    r"max_annual_realized_delta_m3_s=([0-9.eE+-]+)",
    text,
)
closure = re.search(
    r"closure runoff=([0-9.eE+-]+) lake=([0-9.eE+-]+) "
    r"seasonal_routing=([0-9.eE+-]+) seasonal_water=([0-9.eE+-]+)",
    text,
)
seasonal = re.search(
    r"seasonal lakes=(\d+) spinup=(\d+) surface_drift_m=([0-9.eE+-]+) "
    r"relative_cycle=([0-9.eE+-]+) max_range_m=([0-9.eE+-]+) "
    r"flow_dry=(\d+) intermittent=(\d+) perennial=(\d+)",
    text,
)
hashes = re.search(
    r"hash reconciliation=([0-9a-f]{16}) runoff=([0-9a-f]{16}) lake=([0-9a-f]{16}) "
    r"seasonal=([0-9a-f]{16}) evolution=([0-9a-f]{16}) surface=([0-9a-f]{16}) "
    r"post_drainage=([0-9a-f]{16}) parameters=([0-9a-f]{16})",
    text,
)
if not all((header, lakes, flow, closure, seasonal, hashes)):
    raise SystemExit("WG-7C smoke output did not match the expected diagnostics contract")

samples = int(header.group(1))
pre_lakes = int(lakes.group(1))
post_lakes = int(lakes.group(2))
kind_changed = int(lakes.group(3))
added = int(lakes.group(4))
removed = int(lakes.group(5))
max_depth_delta = float(lakes.group(6))
regime_changed = int(flow.group(1))
max_presence_delta = float(flow.group(2))
max_realized_delta = float(flow.group(3))
runoff_closure = float(closure.group(1))
lake_closure = float(closure.group(2))
routing_closure = float(closure.group(3))
seasonal_closure = float(closure.group(4))
seasonal_lakes = int(seasonal.group(1))
spinup = int(seasonal.group(2))
surface_drift = float(seasonal.group(3))
relative_cycle = float(seasonal.group(4))
max_range = float(seasonal.group(5))
dry = int(seasonal.group(6))
intermittent = int(seasonal.group(7))
perennial = int(seasonal.group(8))

if samples != 2562:
    raise SystemExit(f"WG-7C L4 smoke expected 2562 samples, got {samples}")
if pre_lakes <= 0 or post_lakes <= 0 or seasonal_lakes <= 0:
    raise SystemExit(
        f"WG-7C fixed smoke must exercise active lakes: pre={pre_lakes} post={post_lakes} seasonal={seasonal_lakes}"
    )
if not (0 <= kind_changed <= samples and 0 <= added <= samples and 0 <= removed <= samples):
    raise SystemExit("WG-7C lake-change counts are outside sample bounds")
if not (0 <= regime_changed <= samples):
    raise SystemExit("WG-7C flow-regime change count is outside sample bounds")
if not (1 <= spinup <= 24):
    raise SystemExit(f"WG-7C reconciled seasonal spinup is outside [1, 24]: {spinup}")
if not math.isfinite(surface_drift) or surface_drift < 0.0 or surface_drift > 0.0200001:
    raise SystemExit(f"WG-7C reconciled seasonal lake surface cycle did not converge: {surface_drift:.9f} m")
if not math.isfinite(relative_cycle) or relative_cycle < 0.0 or not math.isfinite(max_range) or max_range < 0.0:
    raise SystemExit("WG-7C reconciled lake cycle diagnostics must be finite and nonnegative")
if dry + intermittent + perennial > samples:
    raise SystemExit("WG-7C reconciled flow counts exceed sample count")
if intermittent <= 0 or perennial <= 0:
    raise SystemExit("WG-7C fixed smoke must retain both intermittent and perennial flow")

for name, value in {
    "maximum lake depth delta": max_depth_delta,
    "maximum flow-presence delta": max_presence_delta,
    "maximum realized-discharge delta": max_realized_delta,
    "runoff closure": runoff_closure,
    "lake closure": lake_closure,
    "seasonal routing closure": routing_closure,
    "seasonal water closure": seasonal_closure,
}.items():
    if not math.isfinite(value) or value < 0.0:
        raise SystemExit(f"WG-7C {name} must be finite and nonnegative, got {value}")

if max_presence_delta > 1.000001:
    raise SystemExit(f"WG-7C flow-presence delta exceeds physical bound: {max_presence_delta}")
if max_presence_delta <= 0.0 and max_realized_delta <= 0.0 and max_depth_delta <= 0.0:
    raise SystemExit("WG-7C fixed smoke must produce a nontrivial post-erosion hydrology change")

for name, value in {
    "runoff conservation": runoff_closure,
    "lake water balance": lake_closure,
    "seasonal routing": routing_closure,
    "seasonal water balance": seasonal_closure,
}.items():
    if value > 1.0e-10:
        raise SystemExit(f"WG-7C {name} exceeded tolerance: {value:.3e}")

print(
    f"WG-7C reconciliation smoke accepted: samples={samples} lakes={pre_lakes}->{post_lakes} "
    f"max_depth_delta_m={max_depth_delta:.6f} max_presence_delta={max_presence_delta:.6f} "
    f"max_realized_delta_m3_s={max_realized_delta:.3f} runoff_closure={runoff_closure:.3e} "
    f"lake_closure={lake_closure:.3e} seasonal_routing={routing_closure:.3e} "
    f"seasonal_water={seasonal_closure:.3e}"
)
PY
