#!/usr/bin/env bash
set -euo pipefail

OUTPUT="$(mktemp)"
trap 'rm -f "${OUTPUT}"' EXIT

cargo run --release -p interlink-worldgen-cli --example erosion_performance -- \
  --seed ci-wg7a-erosion \
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
    r"seed=ci-wg7a-erosion coarse_level=3 level=4 plates=12 samples=(\d+) phases=(\d+) runs=1",
    text,
)
erosion = re.search(
    r"erosion erosive_samples=(\d+) max_effective_q_m3_s=([0-9.eE+-]+) max_slope=([0-9.eE+-]+) max_incision_m_year=([0-9.eE+-]+)",
    text,
)
sediment = re.search(
    r"sediment generated_kg_s=([0-9.eE+-]+) land_deposition_kg_s=([0-9.eE+-]+) "
    r"lake_deposition_kg_s=([0-9.eE+-]+) terminal_ocean_deposition_kg_s=([0-9.eE+-]+) "
    r"max_load_kg_s=([0-9.eE+-]+) closure=([0-9.eE+-]+)",
    text,
)
hashes = re.search(
    r"lakes active_traps=(\d+) hash erosion=([0-9a-f]{16}) seasonal=([0-9a-f]{16}) lake=([0-9a-f]{16}) "
    r"drainage=([0-9a-f]{16}) topography=([0-9a-f]{16}) inheritance=([0-9a-f]{16}) parameters=([0-9a-f]{16})",
    text,
)
if not header or not erosion or not sediment or not hashes:
    raise SystemExit("WG-7A smoke output did not match the expected diagnostics contract")

samples = int(header.group(1))
phases = int(header.group(2))
erosive_samples = int(erosion.group(1))
max_q = float(erosion.group(2))
max_slope = float(erosion.group(3))
max_incision = float(erosion.group(4))
generated = float(sediment.group(1))
land_dep = float(sediment.group(2))
lake_dep = float(sediment.group(3))
terminal_dep = float(sediment.group(4))
max_load = float(sediment.group(5))
closure = float(sediment.group(6))
active_traps = int(hashes.group(1))

if samples != 2562:
    raise SystemExit(f"WG-7A L4 smoke expected 2562 samples, got {samples}")
if phases != 24:
    raise SystemExit(f"WG-7A Earthlike smoke expected 24 orbital phases, got {phases}")
if not (0 < erosive_samples <= samples):
    raise SystemExit(f"WG-7A fixed smoke must contain erosive land samples, got {erosive_samples}")
if active_traps <= 0:
    raise SystemExit("WG-7A fixed smoke must exercise sediment trapping in an active WG-6C lake depression")

for name, value in {
    "maximum effective discharge": max_q,
    "maximum channel slope": max_slope,
    "maximum incision potential": max_incision,
    "generated sediment": generated,
    "land deposition": land_dep,
    "lake deposition": lake_dep,
    "terminal/ocean deposition": terminal_dep,
    "maximum sediment load": max_load,
    "sediment conservation": closure,
}.items():
    if not math.isfinite(value) or value < 0:
        raise SystemExit(f"WG-7A {name} must be finite and nonnegative, got {value}")

if max_q <= 0 or max_slope <= 0 or max_incision <= 0:
    raise SystemExit("WG-7A Earthlike smoke must produce positive erosive discharge, slope, and incision potential")
if max_incision > 0.010000001:
    raise SystemExit(f"WG-7A diagnostic incision exceeded the default 0.01 m/yr ceiling: {max_incision}")
if generated <= 0 or max_load <= 0:
    raise SystemExit("WG-7A Earthlike smoke must generate and transport positive sediment")
if lake_dep <= 0 or terminal_dep <= 0:
    raise SystemExit("WG-7A fixed smoke must exercise both lake trapping and terminal/ocean deposition")
if closure > 1.0e-10:
    raise SystemExit(f"WG-7A sediment conservation exceeded tolerance: {closure:.3e}")

component_sum = land_dep + lake_dep + terminal_dep
printed_error = abs(component_sum - generated) / generated
# Printed component totals are rounded to 1e-6 kg/s, so keep this diagnostic check looser than
# the internal closure value while still catching a missing sink term.
if printed_error > 1.0e-8:
    raise SystemExit(f"WG-7A printed sediment sink totals do not close: {printed_error:.3e}")

print(
    f"WG-7A erosion smoke accepted: samples={samples} phases={phases} erosive={erosive_samples} "
    f"lake_traps={active_traps} max_incision_m_year={max_incision:.6f} closure={closure:.3e}"
)
PY
