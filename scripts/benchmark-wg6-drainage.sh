#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-l6}"
RUNS="${RUNS:-3}"

case "${MODE}" in
  l4)
    COARSE_LEVEL=3
    LEVEL=4
    PLATES=12
    ;;
  l6)
    COARSE_LEVEL=4
    LEVEL=6
    PLATES=16
    ;;
  l7)
    COARSE_LEVEL=5
    LEVEL=7
    PLATES=24
    ;;
  *)
    echo "usage: $0 [l4|l6|l7]" >&2
    exit 2
    ;;
esac

cargo run --release -p interlink-worldgen-cli --example drainage_performance -- \
  --seed ci-wg6-drainage \
  --coarse-level "${COARSE_LEVEL}" \
  --level "${LEVEL}" \
  --plates "${PLATES}" \
  --runs "${RUNS}"
