#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-l6}"
RUNS="${RUNS:-1}"
SEED="${SEED:-ci-wg5-performance}"

run_case() {
  local coarse="$1" fine="$2" plates="$3"
  cargo run --release -q -p interlink-worldgen-cli --example climate_performance -- \
    --seed "${SEED}" \
    --coarse-level "${coarse}" \
    --level "${fine}" \
    --plates "${plates}" \
    --runs "${RUNS}"
}

case "${MODE}" in
  l4)
    run_case 3 4 12
    ;;
  l6)
    run_case 4 6 16
    ;;
  l7)
    run_case 5 7 24
    ;;
  suite)
    echo "== WG-5 L4 =="
    run_case 3 4 12
    echo "== WG-5 L6 =="
    run_case 4 6 16
    echo "== WG-5 L7 =="
    run_case 5 7 24
    ;;
  *)
    echo "usage: $0 [l4|l6|l7|suite]" >&2
    exit 2
    ;;
esac
