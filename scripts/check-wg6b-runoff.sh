#!/usr/bin/env bash
set -euo pipefail

OUTPUT="$(mktemp)"
trap 'rm -f "${OUTPUT}"' EXIT

cargo run --release -p interlink-worldgen-cli --example runoff_performance -- \
  --seed ci-wg6b-runoff \
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
    r"seed=ci-wg6b-runoff coarse_level=3 level=4 plates=12 samples=(\d+) runs=1",
    text,
)
water = re.search(
    r"water_balance precip_mm=([0-9.eE+-]+) aet_mm=([0-9.eE+-]+) "
    r"runoff_mm=([0-9.eE+-]+) runoff_fraction=([0-9.eE+-]+)",
    text,
)
discharge = re.search(
    r"discharge total_local_m3_s=([0-9.eE+-]+) terminal_m3_s=([0-9.eE+-]+) "
    r"max_m3_s=([0-9.eE+-]+) relative_error=([0-9.eE+-]+)",
    text,
)
hashes = re.search(
    r"hash runoff=([0-9a-f]{16}) climate=([0-9a-f]{16}) drainage=([0-9a-f]{16}) parameters=([0-9a-f]{16})",
    text,
)
if not header or not water or not discharge or not hashes:
    raise SystemExit("WG-6B smoke output did not match the expected diagnostics contract")

samples = int(header.group(1))
precip = float(water.group(1))
aet = float(water.group(2))
runoff = float(water.group(3))
runoff_fraction = float(water.group(4))
total_local = float(discharge.group(1))
terminal = float(discharge.group(2))
maximum = float(discharge.group(3))
relative_error = float(discharge.group(4))

if samples != 2562:
    raise SystemExit(f"WG-6B L4 smoke expected 2562 samples, got {samples}")

for name, value in {
    "mean land precipitation": precip,
    "mean land AET": aet,
    "mean land runoff": runoff,
    "runoff fraction": runoff_fraction,
    "total local runoff discharge": total_local,
    "terminal discharge": terminal,
    "maximum potential discharge": maximum,
    "discharge conservation error": relative_error,
}.items():
    if not math.isfinite(value) or value < 0:
        raise SystemExit(f"WG-6B {name} must be finite and nonnegative, got {value}")

if precip <= 0 or runoff <= 0 or total_local <= 0 or terminal <= 0 or maximum <= 0:
    raise SystemExit("WG-6B Earthlike smoke must contain positive precipitation, runoff, and discharge")
if aet > precip + 1.0e-9:
    raise SystemExit(f"WG-6B AET exceeded precipitation: aet={aet} precip={precip}")
if runoff > precip + 1.0e-9:
    raise SystemExit(f"WG-6B runoff exceeded precipitation: runoff={runoff} precip={precip}")
if not 0.0 <= runoff_fraction <= 1.0:
    raise SystemExit(f"WG-6B runoff fraction escaped [0, 1]: {runoff_fraction}")
# Printed water-balance depths are rounded to 0.001 mm, so allow 0.01 mm.
if abs((aet + runoff) - precip) > 1.0e-2:
    raise SystemExit(
        f"WG-6B annual water balance failed: precip={precip} aet={aet} runoff={runoff}"
    )
if maximum > total_local * (1.0 + 1.0e-10):
    raise SystemExit(
        f"WG-6B maximum potential discharge exceeded total generated runoff: max={maximum} total={total_local}"
    )
if relative_error > 1.0e-10:
    raise SystemExit(f"WG-6B discharge closure exceeded tolerance: {relative_error:.3e}")
if abs(terminal - total_local) > max(1.0, total_local) * 1.0e-9:
    raise SystemExit(
        f"WG-6B terminal discharge did not conserve generated runoff: terminal={terminal} total={total_local}"
    )

print(
    f"WG-6B runoff smoke accepted: samples={samples} runoff_fraction={runoff_fraction:.6f} "
    f"max_discharge_m3_s={maximum:.3f} discharge_error={relative_error:.3e}"
)
PY
