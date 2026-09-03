#!/usr/bin/env bash
set -euo pipefail
MODE="${1:-smoke}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT
run_case() {
  local output="$1" seed="$2" coarse="$3" fine="$4" plates="$5"
  cargo run --release -q -p interlink-worldgen-cli --example hydroclimate_closure -- \
    --seed "${seed}" --coarse-level "${coarse}" --level "${fine}" --plates "${plates}" \
    --skip-orography-intervention | tee "${output}"
}
check_single() {
  python - "$1" <<'PY'
import re, sys
text = open(sys.argv[1], encoding='utf-8').read()
def grab(pattern):
    m = re.search(pattern, text)
    if not m: raise SystemExit(f'missing hydroclimate metric: {pattern}')
    return float(m.group(1))
mean_p = grab(r'land_precip_mm mean=([0-9.eE+-]+)')
p50 = grab(r'land_precip_mm mean=[^\n]* p50=([0-9.eE+-]+)')
p95 = grab(r'land_precip_mm mean=[^\n]* p95=([0-9.eE+-]+)')
mean_pet = grab(r'land_pet_mm mean=([0-9.eE+-]+)')
dry = grab(r'land_aridity[^\n]*below_0_2=([0-9.eE+-]+)')
humid = grab(r'land_aridity[^\n]*at_least_1=([0-9.eE+-]+)')
ratio = grab(r'tropical_to_subtropical_ratio=([0-9.eE+-]+)')
snow = grab(r'persistent_snow_land_fraction=([0-9.eE+-]+)')
ice = grab(r'sea_ice_ocean_fraction=([0-9.eE+-]+)')
checks = [(100 <= mean_p <= 3000, f'land precipitation mean {mean_p}'), (0 <= p50 <= p95, f'land precipitation percentiles {p50}, {p95}'), (100 <= mean_pet <= 2500, f'land PET mean {mean_pet}'), (0.05 <= dry <= 0.90, f'dry-land fraction {dry}'), (0.01 <= humid <= 0.60, f'humid-land fraction {humid}'), (1.20 <= ratio <= 10.0, f'tropical/subtropical precipitation ratio {ratio}'), (0 <= snow <= 0.60, f'persistent-snow land fraction {snow}'), (0 <= ice <= 0.60, f'sea-ice ocean fraction {ice}')]
failures = [msg for ok, msg in checks if not ok]
if failures: raise SystemExit('WG-5 hydroclimate smoke failed: ' + '; '.join(failures))
print('WG-5 hydroclimate smoke accepted')
PY
}
check_pair() {
  python - "$1" "$2" <<'PY'
import re, sys
def parse(path):
    text = open(path, encoding='utf-8').read()
    def grab(pattern):
        m = re.search(pattern, text)
        if not m: raise SystemExit(f'missing hydroclimate metric in {path}: {pattern}')
        return float(m.group(1))
    return {'precip': grab(r'land_precip_mm mean=([0-9.eE+-]+)'), 'pet': grab(r'land_pet_mm mean=([0-9.eE+-]+)'), 'dry': grab(r'land_aridity[^\n]*below_0_2=([0-9.eE+-]+)'), 'humid': grab(r'land_aridity[^\n]*at_least_1=([0-9.eE+-]+)'), 'ratio': grab(r'tropical_to_subtropical_ratio=([0-9.eE+-]+)'), 'snow': grab(r'persistent_snow_land_fraction=([0-9.eE+-]+)'), 'ice': grab(r'sea_ice_ocean_fraction=([0-9.eE+-]+)')}
low, high = parse(sys.argv[1]), parse(sys.argv[2])
rel = lambda a, b: abs(b-a) / max(abs(a), 1e-12)
failures = []
if rel(low['precip'], high['precip']) > 0.15: failures.append(f"mean land precipitation drift {rel(low['precip'], high['precip']):.3f}")
if rel(low['pet'], high['pet']) > 0.15: failures.append(f"mean land PET drift {rel(low['pet'], high['pet']):.3f}")
if abs(high['snow'] - low['snow']) > 0.08: failures.append(f"persistent snow drift {abs(high['snow']-low['snow']):.3f}")
if abs(high['ice'] - low['ice']) > 0.08: failures.append(f"sea ice drift {abs(high['ice']-low['ice']):.3f}")
for label, case in [('L6', low), ('L7', high)]:
    if not (0.05 <= case['dry'] <= 0.90): failures.append(f"{label} dry-land fraction {case['dry']:.3f}")
    if not (0.01 <= case['humid'] <= 0.60): failures.append(f"{label} humid-land fraction {case['humid']:.3f}")
    if not (1.20 <= case['ratio'] <= 10.0): failures.append(f"{label} tropical/subtropical ratio {case['ratio']:.3f}")
if failures: raise SystemExit('WG-5 hydroclimate quality failed: ' + '; '.join(failures))
print(f"WG-5 hydroclimate quality accepted: precip drift={rel(low['precip'], high['precip']):.3%}, PET drift={rel(low['pet'], high['pet']):.3%}")
PY
}
case "${MODE}" in
  smoke) run_case "${TMP_DIR}/smoke.txt" ci-wg5-hydroclimate 3 4 12; check_single "${TMP_DIR}/smoke.txt" ;;
  quality) run_case "${TMP_DIR}/l6.txt" ci-wg5-l7 5 6 24; run_case "${TMP_DIR}/l7.txt" ci-wg5-l7 5 7 24; check_single "${TMP_DIR}/l6.txt"; check_single "${TMP_DIR}/l7.txt"; check_pair "${TMP_DIR}/l6.txt" "${TMP_DIR}/l7.txt" ;;
  *) echo "usage: $0 [smoke|quality]" >&2; exit 2 ;;
esac
