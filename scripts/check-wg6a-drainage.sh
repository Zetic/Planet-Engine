#!/usr/bin/env bash
set -euo pipefail

OUTPUT="$(mktemp)"
trap 'rm -f "${OUTPUT}"' EXIT

cargo run -p interlink-worldgen-cli --example drainage_performance -- \
  --seed ci-wg6-drainage \
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
header = re.search(r"seed=ci-wg6-drainage coarse_level=3 level=4 plates=12 samples=(\d+) runs=1", text)
drainage = re.search(
    r"drainage basins=(\d+) depressions=(\d+) depressed_samples=(\d+) "
    r"max_contributing_area_km2=([0-9.eE+-]+) max_depression_depth_m=([0-9.eE+-]+)",
    text,
)
area = re.search(
    r"area land_km2=([0-9.eE+-]+) terminal_km2=([0-9.eE+-]+) "
    r"relative_error=([0-9.eE+-]+) hash=([0-9a-f]{16})",
    text,
)
if not header or not drainage or not area:
    raise SystemExit("WG-6A smoke output did not match the expected diagnostics contract")

samples = int(header.group(1))
basins = int(drainage.group(1))
depressions = int(drainage.group(2))
depressed_samples = int(drainage.group(3))
max_area = float(drainage.group(4))
max_depth = float(drainage.group(5))
land_area = float(area.group(1))
terminal_area = float(area.group(2))
relative_error = float(area.group(3))

if samples != 2562:
    raise SystemExit(f"WG-6A L4 smoke expected 2562 samples, got {samples}")
if basins <= 0:
    raise SystemExit("WG-6A Earthlike smoke must resolve at least one drainage basin")
if depressions < 0 or depressed_samples < 0:
    raise SystemExit("WG-6A depression diagnostics must be nonnegative")
for name, value in {
    "maximum contributing area": max_area,
    "maximum depression depth": max_depth,
    "land area": land_area,
    "terminal area": terminal_area,
    "area conservation error": relative_error,
}.items():
    if not math.isfinite(value) or value < 0:
        raise SystemExit(f"WG-6A {name} must be finite and nonnegative, got {value}")
if land_area <= 0 or terminal_area <= 0 or max_area <= 0:
    raise SystemExit("WG-6A Earthlike smoke must contain positive land and contributing area")
if relative_error > 1.0e-10:
    raise SystemExit(f"WG-6A contributing-area closure exceeded tolerance: {relative_error:.3e}")

print(
    f"WG-6A drainage smoke accepted: samples={samples} basins={basins} "
    f"depressions={depressions} area_error={relative_error:.3e}"
)
PY
