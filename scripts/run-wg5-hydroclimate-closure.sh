#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-standard}"

run_case() {
  local seed="$1"
  local coarse="$2"
  local fine="$3"
  local plates="$4"
  shift 4
  echo ""
  echo "=== WG-5 hydroclimate seed=${seed} L${coarse}->L${fine} plates=${plates} ==="
  cargo run --release -q -p interlink-worldgen-cli --example hydroclimate_closure -- \
    --seed "${seed}" \
    --coarse-level "${coarse}" \
    --level "${fine}" \
    --plates "${plates}" \
    "$@"
}

case "${MODE}" in
  smoke)
    run_case "ci-wg5-hydroclimate" 3 4 12 --skip-orography-intervention
    ;;
  standard)
    run_case "interlink-wg5" 4 6 16
    run_case "wg5-cal-a" 4 6 16 --skip-orography-intervention
    run_case "wg5-cal-b" 4 6 16 --skip-orography-intervention
    run_case "wg5-cal-c" 4 6 16 --skip-orography-intervention
    ;;
  quality)
    bash "$0" standard
    run_case "ci-wg5-l7" 5 6 24 --skip-orography-intervention
    run_case "ci-wg5-l7" 5 7 24 --skip-orography-intervention
    ;;
  *)
    echo "usage: $0 [smoke|standard|quality]" >&2
    exit 2
    ;;
esac
