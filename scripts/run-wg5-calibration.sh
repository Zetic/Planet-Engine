#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-standard}"

run_case() {
  local seed="$1"
  local coarse="$2"
  local fine="$3"
  local plates="$4"
  echo ""
  echo "=== WG-5 calibration seed=${seed} L${coarse}->L${fine} plates=${plates} ==="
  cargo run --release -q -p interlink-worldgen-cli --example climate_calibration -- \
    --seed "${seed}" \
    --coarse-level "${coarse}" \
    --level "${fine}" \
    --plates "${plates}"
}

case "${MODE}" in
  smoke)
    run_case "interlink-wg5" 3 4 12
    ;;
  standard)
    run_case "wg5-cal-a" 3 4 12
    run_case "wg5-cal-b" 3 4 12
    run_case "wg5-cal-c" 3 4 12
    run_case "interlink-wg5" 4 6 16
    run_case "wg5-cal-a" 4 6 16
    run_case "wg5-cal-b" 4 6 16
    run_case "wg5-cal-c" 4 6 16
    ;;
  resolution)
    bash scripts/check-wg5-moisture-resolution.sh
    ;;
  quality)
    bash "$0" standard
    bash scripts/check-wg5-moisture-resolution.sh
    ;;
  *)
    echo "usage: $0 [smoke|standard|resolution|quality]" >&2
    exit 2
    ;;
esac
