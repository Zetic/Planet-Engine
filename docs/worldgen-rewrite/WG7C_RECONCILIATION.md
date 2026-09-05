# WG-7C — Post-erosion hydrology reconciliation

WG-7C closes the first terrain/hydrology feedback gap created by WG-7B.

WG-7B owns the evolved solid surface and exactly one rebuilt post-erosion drainage graph. WG-7C keeps WG-4 sea level/ocean identity and WG-5 climate forcing immutable, then reconciles runoff routing, lake equilibrium, and seasonal realized flow to the evolved terrain. It does not create a climate/terrain iteration loop.

Stage identity: `hydrology:post-erosion-reconciliation@1`.

## Causal contract

Accepted ancestry:

1. WG-4 fixed ocean mask / sea level and original topography identity.
2. WG-5 coupled climate and retained orbital-phase forcing.
3. WG-6B accepted local runoff production plus its parameters.
4. WG-6C pre-erosion lake state and WG-6D pre-erosion seasonal hydrology for before/after diagnostics.
5. WG-7B evolved solid elevation and the single rebuilt post-erosion drainage graph.

WG-7C then performs:

1. rebind the **exact accepted public WG-6B local runoff field** to the WG-7B drainage DAG and recompute accumulated potential discharge;
2. solve WG-6C lake equilibrium against `WG-7B evolved_solid_elevation_m` and the rebuilt drainage depressions/spills;
3. solve WG-6D seasonal hydrology against the reconciled runoff/lake state and WG-7B drainage;
4. emit compact before/after hydrologic diagnostics plus deterministic reconciliation identity.

The runoff step deliberately does not rerun Budyko runoff generation. WG-7B already reroutes the accepted public `f32` local-runoff field after terrain evolution, so WG-7C preserves that same accepted local water production exactly and only rebinds its routing ancestry. This makes WG-7C post-erosion potential discharge identical to WG-7B's accepted post-erosion routing result.

WG-7C does **not** rerun WG-5 climate, mutate terrain, rebuild drainage a second time, move the coastline, change sea level, or apply sediment again.

## Final state and comparison fields

`PostErosionHydrologyState` owns the final reconciled:

- runoff/discharge state;
- lake equilibrium state;
- seasonal hydrology state.

It also stores compact per-sample comparison fields rather than a second full seasonal phase state:

- `lake_kind_changed_mask`;
- `lake_depth_delta_m`;
- `annual_realized_discharge_delta_m3_s`;
- `flow_regime_changed_mask`;
- `flow_presence_delta`.

Metrics retain the exact pre-erosion WG-6 ancestry hashes, WG-7B terrain/drainage ancestry hashes, reconciled runoff/lake/seasonal hashes, conservation errors, before/after lake counts, and maximum hydrologic deltas.

## Seasonal lake convergence

WG-6D's ordinary generation contract remains unchanged: its maximum seasonal lake spinup is 12 orbital years.

Evolved WG-7B lake geometry can require more cycles to settle. WG-7C therefore owns a separate `maximum_lake_spinup_years` parameter:

- default: 24 years;
- accepted validation range: 2–48 years;
- included in the WG-7C reconciliation parameter hash.

The physical convergence criterion remains the WG-6D rule: after the minimum spinup, convergence is accepted when either the relative lake-volume cycle criterion is met or the maximum start/end lake-surface drift is at most **0.02 m**.

This distinction matters on the fixed accepted-ancestry L7 case. A 12-year cap ended with about 0.0245 m surface drift. The WG-7C-specific 24-year bound reaches 0.019562795 m after 22 years without changing the ordinary WG-6D contract.

## Conservation and determinism

Permanent acceptance requires:

- exact WG-4/WG-5/WG-7B ancestry;
- exact accepted WG-6B local-runoff production retained through the post-erosion rebind;
- reconciled runoff bound to the WG-7B rebuilt drainage hash;
- lake equilibrium bound to evolved elevation and rebuilt drainage;
- seasonal hydrology bound to reconciled runoff and lakes;
- runoff, lake, phase-routing, and seasonal water closure at numerical noise;
- reconciled seasonal lake surface-cycle drift at or below 0.02 m;
- deterministic 16-hex-character stage/state hashes;
- nontrivial before/after hydrologic changes on fixed acceptance cases.

`scripts/check-wg7c-reconciliation.sh` is part of the permanent Rust CI smoke path.

## Browser / WASM contract

WG-7C advances the cumulative browser protocol to **17** and adds `post-erosion-hydrology` after WG-7B bounded terrain evolution. Packaging is stage 16 of 17.

The cumulative WASM bridge is intentionally memory-conscious. It does not retain independent pre-erosion runoff, lake, drainage, and seasonal states after reconciliation. The final browser-facing hydrology is:

- WG-7B post-erosion drainage;
- WG-7C reconciled runoff;
- WG-7C reconciled lakes;
- WG-7C reconciled seasonal hydrology.

Pre-erosion WG-6 state survives as compact hashes, metrics, and the five comparison fields above. This avoids retaining a second complete phase-major seasonal hydrology state at L7.

The Lab exposes WG-7C diagnostic modes for:

- lake depth change;
- lake state change;
- annual realized-discharge change;
- flow-presence change;
- flow-regime change.

The Lab also verifies that WG-7A/WG-7B inputs match WG-7C's recorded pre-erosion ancestry and that the final top-level drainage/runoff/lake/seasonal identities match WG-7C's reconciled hashes.

Protocol-v17 browser regressions, the cumulative browser build, Rust/WASM compilation, and committed WASM package parity are permanent validation paths.

## Final benchmark matrix

Final release benchmarks were measured on GitHub Actions Ubuntu 24.04 with Rust 1.98.1, optimized release builds, and three timed WG-7C runs after upstream state generation.

| Case | Samples | Runtime mean / median | Lakes pre→post | Lake-change samples | Flow-regime changes | Spinup | Surface drift | Max seasonal range | Closures runoff / lake / seasonal route / seasonal water | Reconciliation hash |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| `ci-wg7b-evolution` L4, coarse L3, 12 plates | 2,562 | 3.584 / 3.676 ms | 2→2 | 0 | 0 | 2 y | 0.002318982 m | 0.171951 m | 0 / 1.177e-16 / 0 / 3.982e-16 | `cdeb1c6ce2cefb9c` |
| `ci-wg7b-evolution` L6, coarse L4, 16 plates | 40,962 | 69.962 / 71.096 ms | 24→24 | 8 | 8 | 4 y | 0.018544788 m | 0.403747 m | 1.942e-16 / 1.938e-16 / 2.590e-16 / 0 | `1135ba5f26493018` |
| `ci-wg7b-evolution` L7, coarse L5, 24 plates | 163,842 | 265.025 / 265.595 ms | 121→121 | 2 | 10 | 5 y | 0.017462505 m | 0.897664 m | 1.059e-14 / 1.075e-14 / 3.529e-16 / 4.887e-16 | `5415b80c77312c0e` |
| `ci-wg5-l7` L6, coarse L5, 24 plates | 40,962 | 84.814 / 85.931 ms | 50→49 | 1 | 1 | 5 y | 0.014787557 m | 0.530042 m | 1.335e-15 / 1.476e-15 / 3.955e-16 / 1.523e-16 | `84d790356212013b` |
| `ci-wg5-l7` L7, coarse L5, 24 plates | 163,842 | 863.940 / 864.949 ms | 110→107 | 10 | 19 | 22 y | 0.019562795 m | 0.640745 m | 2.574e-15 / 2.688e-15 / 1.030e-15 / 8.903e-16 | `8e3961228ef52a2f` |

Additional fixed-case deltas demonstrate that reconciliation is materially changing hydrology where the evolved surface requires it:

- `ci-wg7b-evolution` L6: maximum lake-depth delta 38.269043 m and maximum annual realized-discharge delta 480,642.343750 m³/s;
- `ci-wg7b-evolution` L7: maximum lake-depth delta 17.050537 m and maximum annual realized-discharge delta 37,506.488281 m³/s;
- accepted-ancestry `ci-wg5-l7` L7: lakes 110→107, one added and nine removed lake samples/states, maximum lake-depth delta 36.369629 m, and maximum annual realized-discharge delta 13,427.160156 m³/s.

The 22-year accepted-ancestry L7 case is the deliberate worst-case convergence path in the matrix and is the reason WG-7C owns a larger spinup ceiling. Typical L7 reconciliation remains around 0.27 s on the fixed release seed.

## Deferred

WG-7C completes the immediate terrain→hydrology reconciliation pass. It does not model:

- lake infill or sediment-driven lake-capacity evolution;
- deltas, alluvial fans, or coastal construction;
- coastline or sea-level migration;
- hillslope transport;
- glaciers or permanent snow mass balance;
- weathering, regolith, soils, or explicit lithology evolution;
- groundwater;
- floodplains or wetlands;
- ecology/biomes;
- Regions, Features, resources, or gameplay geography.

Those remain downstream stages rather than hidden iterations inside WG-7C.
