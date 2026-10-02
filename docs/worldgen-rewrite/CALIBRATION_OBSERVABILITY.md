# Calibration observability

Planet Engine exposes a compact, versioned calibration packet for model-assisted analysis without changing physical generation. Schema `planet-engine-calibration@1` summarizes one accepted WG-0→WG-7D world instead of serializing the full L6/L7 typed-array state.

## Contents

The report records run identity and causal hashes, planetary physical parameters, continental-component morphology, canonical WG-4 topography summaries, climate summaries, final WG-7D drainage/runoff/lake/seasonal metrics, ranked large basins/depressions/lakes, and WG-7 geomorphic/sediment summaries. It also includes a compact crust/freeboard causal budget: crust-class surface and submerged fractions, continental structural state, the explicit restored continental-margin material marker introduced by forward margin reconciliation, crust-thickness bands, submerged shelf depth, and emergent-vs-submerged continental means for crust thickness/density, rift/subsidence/basin history, passive-margin state, strain/buoyancy/mechanical fields, and every WG-4 elevation component. Ranked collections are capped at eight records to keep the packet bounded and useful in an LLM context.

The report deliberately reports measurements rather than declaring visual or physical success. Existing stage acceptance tests remain authoritative. A later comparison layer may compare the same fixed-seed ensemble between two commits and classify statistically material deltas.

## Native export

```bash
cargo run --release -p interlink-worldgen-cli --bin calibration-report -- \
  --seed interlink-wg7c --coarse-level 5 --level 7 --plates 16 --format json
```

`--format markdown` emits a compact paste-ready summary. JSON is intended for automated comparisons and deeper analysis. The native Rust report uses canonical dual-cell areas and is the reference form for automated ensemble calibration.

## GitHub Pages

The static Pages Lab requires no server. After generation, **Copy LLM Summary** copies the Markdown form and **Download Calibration JSON** serializes the full compact calibration packet in the browser from the already-transferred cumulative result. The Markdown summary is a selected rendering of that same packet; the JSON is the canonical browser artifact and contains every field available to the summary plus additional diagnostic detail.

The browser does not export the raw per-cell arrays. It aggregates them locally first. Canonical WG-4 topography scalars and water-closure metrics are now carried directly from Rust, while continental-component and crust/freeboard area fractions remain equal-sample estimates because the cumulative Pages result does not transport fine dual-cell area. Per-lake gross inflow and evaporation also remain unavailable and are emitted as null. These limitations are recorded under `fidelity`, so detached JSON/Markdown exports remain self-describing. Native export should still be used when exact area-weighted ensemble comparisons are required.

## Scope

This observability layer does not alter physics, stage versions, engine version, browser/WASM protocol version, physical parameters, or acceptance thresholds. It is a measurement/export surface only.
