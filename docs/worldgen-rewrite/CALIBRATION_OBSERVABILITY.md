# Calibration observability

Planet Engine exposes a compact, versioned calibration packet for model-assisted analysis without changing physical generation. Schema `planet-engine-calibration@1` summarizes one accepted WG-0→WG-7D world instead of serializing the full L6/L7 typed-array state.

## Contents

The report records run identity and causal hashes, continental-component morphology, topographic and climate summaries, final WG-7D drainage/runoff/lake/seasonal metrics, ranked large basins/depressions/lakes, and WG-7 geomorphic/sediment summaries. Ranked collections are capped at eight records to keep the packet bounded and useful in an LLM context.

The report deliberately reports measurements rather than declaring visual or physical success. Existing stage acceptance tests remain authoritative. A later comparison layer may compare the same fixed-seed ensemble between two commits and classify statistically material deltas.

## Native export

```bash
cargo run --release -p interlink-worldgen-cli --bin calibration-report -- \
  --seed interlink-wg7c --coarse-level 5 --level 7 --plates 16 --format json
```

`--format markdown` emits a compact paste-ready summary. JSON is intended for automated comparisons and deeper analysis. The native Rust report uses canonical dual-cell areas and is the reference form for automated ensemble calibration.

## GitHub Pages

The static Pages Lab requires no server. After generation, **Copy LLM Summary** copies the Markdown form and **Download Calibration JSON** creates the packet entirely in the browser from the already-transferred protocol-v18 cumulative result.

The browser does not export raw per-cell arrays. It aggregates them locally first. Because protocol v18 does not transport dual-cell area or every internal per-lake water-budget term, the Pages packet estimates continental component area from equal sample area and leaves unavailable per-lake terms null. Native export should be used when exact ensemble comparisons are required.

## Scope

This observability layer does not alter physics, stage versions, engine version, browser/WASM protocol version, physical parameters, or acceptance thresholds. It is a measurement/export surface only.
