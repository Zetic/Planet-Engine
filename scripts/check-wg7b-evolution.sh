#!/usr/bin/env bash
set -euo pipefail

OUTPUT="$(mktemp)"
trap 'rm -f "${OUTPUT}"' EXIT

cargo run --release -p interlink-worldgen-cli --example terrain_evolution_performance -- \
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
terrain = re.search(
    r"terrain duration_years=([0-9.eE+-]+) eroded_samples=(\d+) depositional_samples=(\d+) "
    r"receiver_changed=(\d+) receiver_changed_fraction=([0-9.eE+-]+)",
    text,
)
change = re.search(
    r"change max_erosion_m=([0-9.eE+-]+) max_deposition_m=([0-9.eE+-]+) "
    r"max_abs_m=([0-9.eE+-]+) mean_land_abs_m=([0-9.eE+-]+)",
    text,
)
sediment = re.search(
    r"sediment generated_kg_s=([0-9.eE+-]+) land_deposition_kg_s=([0-9.eE+-]+) "
    r"lake_sink_kg_s=([0-9.eE+-]+) terminal_ocean_sink_kg_s=([0-9.eE+-]+) closure=([0-9.eE+-]+)",
    text,
)
drainage = re.search(
    r"drainage basins_before=(\d+) basins_after=(\d+) depressions_before=(\d+) "
    r"depressions_after=(\d+) area_closure=([0-9.eE+-]+)",
    text,
)
runoff = re.search(
    r"runoff max_post_q_m3_s=([0-9.eE+-]+) closure=([0-9.eE+-]+)",
    text,
)
hashes = re.search(
    r"hash evolution=([0-9a-f]{16}) surface=([0-9a-f]{16}) post_drainage=([0-9a-f]{16}) "
    r"erosion=([0-9a-f]{16}) parameters=([0-9a-f]{16})",
    text,
)
if not all((header, terrain, change, sediment, drainage, runoff, hashes)):
    raise SystemExit("WG-7B smoke output did not match the expected diagnostics contract")

samples = int(header.group(1))
duration = float(terrain.group(1))
eroded = int(terrain.group(2))
depositional = int(terrain.group(3))
receiver_changed = int(terrain.group(4))
receiver_fraction = float(terrain.group(5))
max_erosion = float(change.group(1))
max_deposition = float(change.group(2))
max_abs = float(change.group(3))
mean_abs = float(change.group(4))
generated = float(sediment.group(1))
land_dep = float(sediment.group(2))
lake_sink = float(sediment.group(3))
terminal_sink = float(sediment.group(4))
sediment_closure = float(sediment.group(5))
area_closure = float(drainage.group(5))
max_q = float(runoff.group(1))
runoff_closure = float(runoff.group(2))

if samples != 2562:
    raise SystemExit(f"WG-7B L4 smoke expected 2562 samples, got {samples}")
if not (0.0 < duration <= 250_000.000001):
    raise SystemExit(f"WG-7B direct geomorphic horizon is outside the accepted bound: {duration}")
if not (0 < eroded <= samples):
    raise SystemExit(f"WG-7B fixed smoke must erode land samples, got {eroded}")
if not (0 < depositional <= samples):
    raise SystemExit(f"WG-7B fixed smoke must deposit sediment on land, got {depositional}")
if not (0 <= receiver_changed <= samples) or not (0.0 <= receiver_fraction <= 1.0):
    raise SystemExit("WG-7B receiver-change diagnostics are outside physical bounds")

for name, value in {
    "maximum applied erosion": max_erosion,
    "maximum applied deposition": max_deposition,
    "maximum absolute terrain change": max_abs,
    "mean land absolute terrain change": mean_abs,
    "generated sediment": generated,
    "land deposition": land_dep,
    "lake sink": lake_sink,
    "terminal/ocean sink": terminal_sink,
    "sediment closure": sediment_closure,
    "drainage area closure": area_closure,
    "post-erosion maximum discharge": max_q,
    "post-erosion runoff closure": runoff_closure,
}.items():
    if not math.isfinite(value) or value < 0.0:
        raise SystemExit(f"WG-7B {name} must be finite and nonnegative, got {value}")

if max_erosion <= 0.0 or max_deposition <= 0.0 or mean_abs <= 0.0:
    raise SystemExit("WG-7B fixed smoke must produce nonzero erosion and ordinary land deposition")
if max_abs > 120.0001 or max_erosion > 120.0001 or max_deposition > 120.0001:
    raise SystemExit(
        f"WG-7B resolved terrain change exceeded the default 120 m cap: "
        f"erosion={max_erosion} deposition={max_deposition} absolute={max_abs}"
    )
if generated <= 0.0 or lake_sink <= 0.0 or terminal_sink <= 0.0:
    raise SystemExit("WG-7B fixed smoke must generate sediment and exercise lake and terminal/ocean sinks")
if sediment_closure > 1.0e-10:
    raise SystemExit(f"WG-7B sediment conservation exceeded tolerance: {sediment_closure:.3e}")
if area_closure > 1.0e-10:
    raise SystemExit(f"WG-7B rebuilt drainage area conservation exceeded tolerance: {area_closure:.3e}")
if max_q <= 0.0:
    raise SystemExit("WG-7B post-erosion runoff reroute must produce positive discharge")
if runoff_closure > 1.0e-10:
    raise SystemExit(f"WG-7B post-erosion runoff conservation exceeded tolerance: {runoff_closure:.3e}")

printed_sink_error = abs((land_dep + lake_sink + terminal_sink) - generated) / generated
if printed_sink_error > 1.0e-8:
    raise SystemExit(f"WG-7B printed applied sediment sinks do not close: {printed_sink_error:.3e}")

print(
    f"WG-7B terrain evolution smoke accepted: samples={samples} duration_years={duration:.1f} "
    f"eroded={eroded} depositional={depositional} max_abs_m={max_abs:.3f} "
    f"sediment_closure={sediment_closure:.3e} drainage_closure={area_closure:.3e} "
    f"runoff_closure={runoff_closure:.3e}"
)
PY
