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

The static Pages Lab requires no server. After generation, **Copy Calibration Data** copies the full canonical calibration JSON packet directly to the clipboard from the already-transferred cumulative result. The browser no longer exposes separate LLM-summary and Calibration JSON download controls; one calibration action now provides the complete diagnostic packet for paste into ChatGPT or another analysis tool.

The browser does not export the raw per-cell arrays. Canonical WG-4 topography scalars and water-closure metrics are carried directly from Rust. Causal freeboard fields that are intentionally released after WG-4 for memory control—crust density, rift/subsidence/basin history, strain, compensated buoyancy, elastic thickness, and structural fabric—are reduced to five continental bucket means in Rust before scratch release (all, emergent, submerged, restored-margin, and other continental). The browser receives only those compact means plus the corresponding bucket/state counts, preventing missing/empty arrays from silently becoming JSON `null` values. Continental-component and crust/freeboard area fractions remain equal-sample estimates because the cumulative Pages result does not transport fine dual-cell area. Per-lake gross inflow and evaporation also remain unavailable and are emitted as null. These limitations are recorded under `fidelity`, so copied JSON packets remain self-describing. Native export should still be used when exact area-weighted ensemble comparisons are required.

## Scope

This observability layer does not alter physics, stage versions, engine version, physical parameters, or acceptance thresholds. The browser/WASM protocol is versioned when the cumulative transport contract changes; the compact pre-release freeboard snapshot uses protocol v24.
