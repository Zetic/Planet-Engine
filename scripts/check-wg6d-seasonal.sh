#!/usr/bin/env bash
set -euo pipefail

OUTPUT="$(mktemp)"
trap 'rm -f "${OUTPUT}"' EXIT

cargo run --release -p interlink-worldgen-cli --example seasonal_performance -- \
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
    r"seed=ci-wg6c-lakes coarse_level=3 level=4 plates=12 samples=(\d+) phases=(\d+) runs=1",
    text,
)
runoff = re.search(
    r"runoff annual_mean_local_m3_s=([0-9.eE+-]+) annual_target_error=([0-9.eE+-]+) snowmelt_fraction=([0-9.eE+-]+)",
    text,
)
potential = re.search(
    r"potential terminal_mean_m3_s=([0-9.eE+-]+) max_phase_m3_s=([0-9.eE+-]+) routing_error=([0-9.eE+-]+)",
    text,
)
realized = re.search(
    r"realized terminal_mean_m3_s=([0-9.eE+-]+) max_phase_m3_s=([0-9.eE+-]+) water_balance_error=([0-9.eE+-]+)",
    text,
)
lakes = re.search(
    r"lakes active=(\d+) spinup_years=(\d+) cycle_error=([0-9.eE+-]+) max_level_range_m=([0-9.eE+-]+) "
    r"precip_m3_s=([0-9.eE+-]+) evap_m3_s=([0-9.eE+-]+) terminal_storage_m3_s=([0-9.eE+-]+)",
    text,
)
flow = re.search(r"flow dry=(\d+) intermittent=(\d+) perennial=(\d+)", text)
lake_cycle = re.search(r"lake_cycle surface_change_m=([0-9.eE+-]+)", text)
hashes = re.search(
    r"hash seasonal=([0-9a-f]{16}) lakes=([0-9a-f]{16}) runoff=([0-9a-f]{16}) drainage=([0-9a-f]{16}) "
    r"climate=([0-9a-f]{16}) parameters=([0-9a-f]{16})",
    text,
)
if not header or not runoff or not potential or not realized or not lakes or not flow or not lake_cycle or not hashes:
    raise SystemExit("WG-6D smoke output did not match the expected diagnostics contract")

samples = int(header.group(1))
phases = int(header.group(2))
annual_local = float(runoff.group(1))
annual_target_error = float(runoff.group(2))
snowmelt_fraction = float(runoff.group(3))
potential_terminal = float(potential.group(1))
potential_max = float(potential.group(2))
routing_error = float(potential.group(3))
realized_terminal = float(realized.group(1))
realized_max = float(realized.group(2))
water_balance_error = float(realized.group(3))
active_lakes = int(lakes.group(1))
spinup_years = int(lakes.group(2))
cycle_error = float(lakes.group(3))
level_range = float(lakes.group(4))
lake_precip = float(lakes.group(5))
lake_evap = float(lakes.group(6))
terminal_storage = float(lakes.group(7))
dry_flow = int(flow.group(1))
intermittent_flow = int(flow.group(2))
perennial_flow = int(flow.group(3))
surface_cycle_change_m = float(lake_cycle.group(1))

if samples != 2562:
    raise SystemExit(f"WG-6D L4 smoke expected 2562 samples, got {samples}")
if phases != 24:
    raise SystemExit(f"WG-6D default Earthlike climate expected 24 orbital phases, got {phases}")
if active_lakes <= 0:
    raise SystemExit("WG-6D fixed smoke seed must exercise at least one WG-6C lake control volume")
if not (2 <= spinup_years <= 12):
    raise SystemExit(f"WG-6D lake cycle count is outside the supported bound: {spinup_years}")
if dry_flow + intermittent_flow + perennial_flow <= 0 or dry_flow + intermittent_flow + perennial_flow > samples:
    raise SystemExit("WG-6D flow-regime counts must describe a nonempty subset of canonical samples")
if intermittent_flow <= 0 or perennial_flow <= 0:
    raise SystemExit("WG-6D fixed smoke seed must exercise both intermittent and perennial realized flow")

for name, value in {
    "annual local runoff": annual_local,
    "annual target closure": annual_target_error,
    "snowmelt fraction": snowmelt_fraction,
    "potential terminal discharge": potential_terminal,
    "maximum phase potential discharge": potential_max,
    "routing closure": routing_error,
    "realized terminal discharge": realized_terminal,
    "maximum phase realized discharge": realized_max,
    "seasonal water balance": water_balance_error,
    "lake cycle change": cycle_error,
    "lake surface cycle change": surface_cycle_change_m,
    "lake level range": level_range,
    "lake precipitation": lake_precip,
    "lake evaporation": lake_evap,
    "terminal storage": terminal_storage,
}.items():
    if not math.isfinite(value) or value < 0:
        raise SystemExit(f"WG-6D {name} must be finite and nonnegative, got {value}")

if annual_local <= 0 or potential_terminal <= 0 or potential_max <= 0:
    raise SystemExit("WG-6D Earthlike smoke must produce positive seasonal runoff and potential discharge")
if realized_terminal <= 0 or realized_max <= 0:
    raise SystemExit("WG-6D Earthlike smoke must produce positive realized seasonal discharge")
if not (0.0 <= snowmelt_fraction <= 1.0):
    raise SystemExit(f"WG-6D snowmelt fraction must remain within [0, 1], got {snowmelt_fraction}")
if annual_target_error > 1.0e-6:
    raise SystemExit(f"WG-6D annual local-runoff target closure exceeded tolerance: {annual_target_error:.3e}")
if routing_error > 1.0e-10:
    raise SystemExit(f"WG-6D phase-routing conservation exceeded tolerance: {routing_error:.3e}")
if water_balance_error > 1.0e-10:
    raise SystemExit(f"WG-6D seasonal lake/global water-balance closure exceeded tolerance: {water_balance_error:.3e}")
if surface_cycle_change_m > 0.02 + 1.0e-12:
    raise SystemExit(f"WG-6D seasonal lake surface cycle did not converge within 2 cm: {surface_cycle_change_m:.9f} m")

print(
    f"WG-6D seasonal smoke accepted: samples={samples} phases={phases} active_lakes={active_lakes} "
    f"intermittent={intermittent_flow} perennial={perennial_flow} "
    f"surface_cycle_change_m={surface_cycle_change_m:.6f} water_balance_error={water_balance_error:.3e}"
)
PY
