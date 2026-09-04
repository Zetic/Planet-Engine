#!/usr/bin/env bash
set -euo pipefail

OUTPUT="$(mktemp)"
trap 'rm -f "${OUTPUT}"' EXIT

cargo run --release -p interlink-worldgen-cli --example lake_performance -- \
  --seed ci-wg6c-lakes \
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
    r"seed=ci-wg6c-lakes coarse_level=3 level=4 plates=12 samples=(\d+) runs=1",
    text,
)
lakes = re.search(
    r"lakes total=(\d+) endorheic=(\d+) overflowing=(\d+) terminal_storage=(\d+) "
    r"lake_samples=(\d+) area_km2=([0-9.eE+-]+) volume_km3=([0-9.eE+-]+) max_depth_m=([0-9.eE+-]+)",
    text,
)
water = re.search(
    r"water_balance lake_precip_m3_s=([0-9.eE+-]+) lake_evap_m3_s=([0-9.eE+-]+) "
    r"terminal_realized_m3_s=([0-9.eE+-]+) storage_m3_s=([0-9.eE+-]+) relative_error=([0-9.eE+-]+)",
    text,
)
discharge = re.search(
    r"discharge potential_max_m3_s=([0-9.eE+-]+) realized_max_m3_s=([0-9.eE+-]+)",
    text,
)
hashes = re.search(
    r"hash lakes=([0-9a-f]{16}) runoff=([0-9a-f]{16}) drainage=([0-9a-f]{16}) "
    r"climate=([0-9a-f]{16}) parameters=([0-9a-f]{16})",
    text,
)
if not header or not lakes or not water or not discharge or not hashes:
    raise SystemExit("WG-6C smoke output did not match the expected diagnostics contract")

samples = int(header.group(1))
lake_count = int(lakes.group(1))
endorheic = int(lakes.group(2))
overflowing = int(lakes.group(3))
terminal_storage = int(lakes.group(4))
lake_samples = int(lakes.group(5))
area_km2 = float(lakes.group(6))
volume_km3 = float(lakes.group(7))
max_depth_m = float(lakes.group(8))
lake_precip = float(water.group(1))
lake_evap = float(water.group(2))
terminal_realized = float(water.group(3))
storage = float(water.group(4))
relative_error = float(water.group(5))
potential_max = float(discharge.group(1))
realized_max = float(discharge.group(2))

if samples != 2562:
    raise SystemExit(f"WG-6C L4 smoke expected 2562 samples, got {samples}")
if lake_count <= 0 or lake_samples <= 0:
    raise SystemExit("WG-6C Earthlike smoke must contain at least one equilibrium lake")
if endorheic + overflowing + terminal_storage != lake_count:
    raise SystemExit("WG-6C lake state counts do not sum to total lake count")
if lake_samples < lake_count:
    raise SystemExit("WG-6C each lake must occupy at least one sample")

for name, value in {
    "lake area": area_km2,
    "lake volume": volume_km3,
    "maximum lake depth": max_depth_m,
    "lake precipitation": lake_precip,
    "lake evaporation": lake_evap,
    "terminal realized discharge": terminal_realized,
    "unreleased storage": storage,
    "water balance error": relative_error,
    "maximum potential discharge": potential_max,
    "maximum realized discharge": realized_max,
}.items():
    if not math.isfinite(value) or value < 0:
        raise SystemExit(f"WG-6C {name} must be finite and nonnegative, got {value}")

if area_km2 <= 0 or volume_km3 <= 0 or max_depth_m <= 0:
    raise SystemExit("WG-6C Earthlike smoke must produce positive lake area, volume, and depth")
if terminal_realized <= 0 or potential_max <= 0 or realized_max <= 0:
    raise SystemExit("WG-6C Earthlike smoke must produce positive realized and potential discharge")
if relative_error > 1.0e-10:
    raise SystemExit(f"WG-6C global water-balance closure exceeded tolerance: {relative_error:.3e}")

print(
    f"WG-6C lake smoke accepted: samples={samples} lakes={lake_count} "
    f"lake_samples={lake_samples} realized_max_m3_s={realized_max:.3f} "
    f"water_balance_error={relative_error:.3e}"
)
PY
