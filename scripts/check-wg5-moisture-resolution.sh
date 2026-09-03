#!/usr/bin/env bash
set -euo pipefail
l6="$(mktemp)"; l7="$(mktemp)"
trap 'rm -f "$l6" "$l7"' EXIT
for fine in 6 7; do
  out="$l6"; [[ "$fine" == 7 ]] && out="$l7"
  cargo run --release -q -p interlink-worldgen-cli --example climate_calibration -- \
    --seed ci-wg5-l7 --coarse-level 5 --level "$fine" --plates 24 \
    --skip-orography-intervention | tee "$out"
done
python3 - "$l6" "$l7" <<'PY'
import re, sys

def parse(path):
    text=open(path).read()
    precip=float(re.search(r"hydrology_mm_year precip_mean=([0-9.eE+-]+)", text).group(1))
    m=re.search(r"transport wind_cap=[0-9.eE+-]+ moisture_limiter=([0-9.eE+-]+) moisture_max_substeps=([0-9]+)", text)
    error=float(re.search(r"water_budget_kg evaporation=[0-9.eE+-]+ precipitation=[0-9.eE+-]+ relative_error=([0-9.eE+-]+)", text).group(1))
    return precip,float(m.group(1)),int(m.group(2)),error
l6,l7=parse(sys.argv[1]),parse(sys.argv[2])
for label,v in (("L6",l6),("L7",l7)):
    assert 500 <= v[0] <= 2000, f"{label} precip={v[0]}"
    assert v[1] < 0.01, f"{label} limiter={v[1]}"
    assert 4 <= v[2] <= 64, f"{label} substeps={v[2]}"
    assert v[3] < 1e-8, f"{label} moisture error={v[3]}"
drift=abs(l7[0]-l6[0])/max(0.5*(l6[0]+l7[0]),1.0)
assert drift < 0.10, f"precip resolution drift={drift:.3%}"
print(f"WG-5 moisture resolution acceptance: L6={l6[0]:.3f} L7={l7[0]:.3f} drift={drift:.3%}")
PY
