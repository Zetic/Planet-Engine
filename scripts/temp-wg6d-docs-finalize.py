from pathlib import Path

path = Path("docs/worldgen-rewrite/WG6_HYDROLOGY.md")
text = path.read_text()

old = "The primary browser Lab obtains WG-6A and WG-6B from the same cumulative generation result as WG-5, so inspecting hydrology does not issue a second climate or drainage generation. Browser/WASM protocol version `12` carries the cumulative contract."
new = "The primary browser Lab obtains WG-6A and WG-6B from the same cumulative generation result as WG-5, so inspecting hydrology does not issue a second climate or drainage generation. WG-6B originally entered the cumulative browser contract at protocol version `12`; the current cumulative Lab contract is protocol version `14` through WG-6D."
if text.count(old) != 1:
    raise SystemExit(f"expected one WG-6B browser-contract sentence, found {text.count(old)}")
text = text.replace(old, new, 1)

old = "This is an annual climatological balance. WG-6B currently treats annual snow storage change as zero; it does not attempt to time snow accumulation and melt within the year. WG-6D will add seasonal snowmelt timing, persistent-snow retention, intermittent flow, and seasonal discharge."
new = "This is an annual climatological balance. WG-6B treats annual snow storage change as zero and does not attempt to time snow accumulation and melt within the year. WG-6D adds seasonal snow accumulation/melt timing, phase discharge, dynamic lake storage, and intermittent/perennial realized-flow classification while preserving the accepted WG-6B annual runoff target. Persistent-snow retention that would change that annual target remains downstream work."
if text.count(old) != 1:
    raise SystemExit(f"expected one WG-6B seasonal-defer sentence, found {text.count(old)}")
text = text.replace(old, new, 1)

wg6d = r'''## WG-6D inputs and causality

WG-6D stage identity is `hydrology:seasonal-hydrology@1`. It consumes accepted upstream states rather than solving climate, drainage, runoff, or annual lake equilibrium again:

```text
WG-5 retained phase precipitation + annual temperature harmonics
                    +
WG-6B accepted annual local runoff target
                    +
WG-6A receiver DAG and depression/spill topology
                    +
WG-6C active lake identities + annual equilibrium surfaces
                    ↓
rain/snow partition + bounded degree-day snowmelt timing
                    ↓
phase local runoff preserving the WG-6B annual target
                    ↓
phase potential discharge over the accepted WG-6A DAG
                    ↓
dynamic WG-6C lake control volumes + same-phase spill cascades
                    ↓
phase realized discharge + seasonal lake geometry
                    ↓
dry / intermittent / perennial realized-flow classification
```

WG-6D requires exact WG-5, WG-6A, WG-6B, and WG-6C ancestry hashes. It never mutates WG-4 terrain, rebuilds the drainage graph, or overwrites the accepted annual WG-6B/WG-6C diagnostics.

## Seasonal runoff and snow timing

The default seasonal parameters are:

- snow/rain transition centered at `273.15 K` with a `2 K` transition width;
- melt threshold `273.15 K`;
- degree-day melt factor `3 mm K^-1 day^-1`.

For each land sample, retained WG-5 phase precipitation is partitioned into rain and snow using reconstructed WG-5 temperature harmonics. Snow is carried forward as water-equivalent storage and released when phase temperature exceeds the melt threshold, bounded by available storage and the degree-day melt capacity.

WG-6D then redistributes the accepted WG-6B annual local runoff across orbital phases in proportion to liquid water availability. Phase runoff is normalized so its annual mean returns the WG-6B local-runoff target. Where the current simplified snow timing would otherwise leave no liquid phase despite positive WG-6B annual runoff, the implementation falls back to precipitation-phase shares. WG-6D therefore changes runoff **timing**, not the accepted annual runoff mass. Long-term persistent-snow retention that would reduce the WG-6B annual runoff target is not claimed by this stage.

`phase_snowmelt_runoff_m3_s` records the portion of phase runoff timing attributable to snowmelt, and `phase_snow_storage_mm` records phase-end water-equivalent snow storage.

## Phase routing and potential discharge

Each orbital phase is routed independently over the accepted WG-6A receiver DAG. WG-6D performs no seasonal rerouting: topography controls the drainage graph, while climate controls the amount and timing of water on that graph.

The phase potential field is conserved before lake interception:

```text
sum(phase local runoff) = sum(phase terminal potential discharge)
```

The annual mean of phase local runoff is separately checked against the accepted WG-6B annual local-runoff total.

## Dynamic seasonal lake control volumes

WG-6D initializes each active WG-6C depression at its accepted annual equilibrium lake surface, converts that surface to physical volume, and advances the lake through full orbital cycles. During each phase it:

1. derives fractional lake area from current lake volume and WG-6A/WG-4 hypsometry;
2. intercepts dry-land routed inflow across the entire active depression control volume;
3. adds direct phase precipitation on the current lake surface;
4. applies seasonally weighted evaporation while preserving the WG-6C annual evaporation scale;
5. retains water up to the depression spill volume;
6. routes excess spill water downstream in the same phase, including deterministic cascades into downstream active lakes;
7. records phase lake surface elevation, area, volume, and realized discharge.

Terminal depressions without a real spill receiver retain excess as explicit terminal storage. As in WG-6C, active-depression routing is blocked across all depression members; solved spill outflow is the only downstream release from an active lake control volume.

Seasonal lake spin-up is bounded and deterministic. A lake run performs at least two complete years and at most twelve. It stops once either the maximum lake-volume cycle change is at most `1e-6` of spill-volume scale or the maximum start/end lake-surface drift is at most `0.02 m`. The surface criterion prevents tiny or shallow resolved basins from forcing long transients because of a large fractional volume change that is physically centimetre-scale.

The final-year global water balance is:

```text
start lake storage
    + dry-land local runoff
    + lake-surface precipitation
= end lake storage
    + terminal realized discharge
    + lake-surface evaporation
    + unreleased terminal storage
```

All conservation accounting remains `f64` internally.

## Realized-flow presence and channel regime

WG-6D classifies realized flow from the phase-major post-lake discharge field. A land sample counts as flowing in a phase when realized discharge exceeds `1e-6 m3/s`. Its presence fraction is the fraction of retained orbital phases above that numerical threshold:

- `0 = dry`: no phase has present realized flow;
- `1 = intermittent`: at least one but not every phase has present realized flow;
- `2 = perennial`: every phase has present realized flow.

Submerged samples are excluded from the dry/intermittent/perennial land counts. This is a hydrologic persistence classification, not yet a river-width, bankfull, floodplain, or navigability model.

## WG-6D state contract

`SeasonalHydrologyState` exposes:

- phase-major `phase_local_runoff_m3_s`;
- phase-major `phase_snowmelt_runoff_m3_s`;
- phase-major `phase_snow_storage_mm`;
- phase-major `phase_potential_discharge_m3_s`;
- phase-major `phase_realized_discharge_m3_s`;
- per-sample `flow_presence_fraction` and `flow_regime`;
- phase-major lake surface elevation, area, and volume in WG-6C lake-record order;
- annual local-runoff target closure and phase-routing conservation diagnostics;
- annual-mean terminal realized discharge, lake precipitation, lake evaporation, and unreleased storage diagnostics;
- bounded seasonal lake-cycle convergence diagnostics;
- deterministic seasonal-parameter, WG-5, WG-6A, WG-6B, WG-6C, and WG-6D hashes.

The WG-6D hash includes stage identity/version, derived stage seed, seasonal parameter hash, all accepted upstream ancestry hashes, and every public WG-6D phase/per-sample output vector.

## WG-6D core invariants

WG-6D must satisfy all of the following:

1. WG-5, WG-6A, WG-6B, WG-6C, and WG-6D fields align on one canonical fine topology and exact accepted ancestry.
2. WG-4 terrain, WG-6A receivers/depression membership, WG-6B annual runoff, and WG-6C annual lake state remain immutable.
3. Phase local runoff, snowmelt runoff, snow storage, potential discharge, realized discharge, and seasonal lake geometry remain finite and nonnegative.
4. Annual mean phase local runoff closes to the accepted WG-6B local-runoff target within numerical tolerance.
5. Each phase potential-routing traversal conserves locally generated runoff to terminal potential discharge within numerical tolerance.
6. Active lake routing blocks the entire depression control volume and reinjects only solved same-phase spill outflow.
7. Seasonal lake surfaces never exceed owning spill elevation; phase lake volume never exceeds spill volume for spill-capable basins.
8. Final-year realized-flow/lake storage accounting closes globally within numerical tolerance.
9. Seasonal lake spin-up is bounded to at most twelve years and accepted only after the deterministic volume-or-surface cycle criterion is reached in the fixed smoke contract.
10. Realized-flow presence/regime is derived only from post-lake phase discharge using the documented numerical threshold.
11. Same planet, seed, parameters, and accepted upstream ancestry produce the same WG-6D hash.

## WG-6D performance policy

WG-6D is intentionally more expensive than WG-6A/B/C because it routes all retained orbital phases and advances dynamic lake storage for multiple bounded cycles. A sustained L6 WG-6D runtime above `0.25 s` on the accepted runner class is a profiling trigger. The final fixed release cases remain well below that threshold; L7 remains around `0.3 s` for the stage itself.

### WG-6D fixed release benchmark

Final WG-6D semantics were measured on GitHub Actions Ubuntu with Rust `1.98.1`, optimized release builds, fixed seed `ci-wg6c-lakes`, and three WG-6D-only timed runs after generating upstream WG-5/WG-6A/WG-6B/WG-6C state once.

| Fine level | Coarse | Plates | Samples | Mean | Median | Lakes | Spin-up years | Snowmelt share | Surface-cycle drift | Max seasonal lake range | Dry | Intermittent | Perennial | Water closure | Seasonal hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L4 | L3 | 12 | 2,562 | 3.351 ms | 3.280 ms | 3 | 2 | 1.842% | 0.005723 m | 0.254 m | 16 | 338 | 304 | `1.300e-16` | `9ceb1b73f2398fdf` |
| L6 | L4 | 16 | 40,962 | 61.387 ms | 62.522 ms | 19 | 3 | 1.433% | 0.017124 m | 0.392 m | 238 | 4,463 | 6,562 | `3.002e-16` | `6fab9e3d47d6f344` |
| L7 | L5 | 24 | 163,842 | 278.074 ms | 279.388 ms | 117 | 4 | 1.941% | 0.014866 m | 0.608 m | 2,579 | 18,048 | 15,748 | `1.045e-15` | `27552e85e22131e0` |

Potential-routing conservation is `1.926e-16`, `1.509e-16`, and `1.888e-16` respectively. The lake surface cycle converges below the accepted `0.02 m` threshold in 2–4 years for these fixed release cases. Exact flow-regime counts are resolution-sensitive diagnostics rather than cross-resolution equality targets.

### WG-6D accepted WG-5 fixed-ancestry diagnostic

The accepted WG-5 convergence seed `ci-wg5-l7` holds coarse physical level L5 and plate count 24 fixed while refining only the final topology from L6 to L7:

| Fine level | Samples | Runtime | Lakes | Spin-up years | Snowmelt share | Surface-cycle drift | Intermittent | Perennial | Terminal realized discharge | Water closure | Seasonal hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L6 | 40,962 | 78.111 ms | 50 | 5 | 3.942% | 0.014765 m | 5,375 | 3,716 | 1,561,311.068 m3/s | `0.000e0` | `84a7187464d8f128` |
| L7 | 163,842 | 300.901 ms | 110 | 5 | 3.353% | 0.014786 m | 22,264 | 14,170 | 1,797,460.411 m3/s | `8.765e-16` | `e948241c030be5dc` |

The fixed ancestry confirms that the bounded lake-cycle policy reaches the same centimetre-scale surface criterion at both resolutions while global conservation remains at floating-point noise. Flow-regime counts change substantially with final topology resolution because finer drainage resolves many additional low-order channels; equality of per-cell river classification is not a convergence target.

'''
marker = "## Permanent acceptance\n"
if text.count(marker) != 1:
    raise SystemExit(f"expected one permanent-acceptance marker, found {text.count(marker)}")
text = text.replace(marker, wg6d + marker, 1)

old = "`bash scripts/check-wg6c-lakes.sh` runs a fixed L4 Earthlike WG-6C case and verifies canonical sample count, nonempty equilibrium lake state, state-count consistency, positive finite lake geometry and discharge, deterministic hash formatting, and global annual lake/realized-flow water-balance closure within `1e-10`.\n"
new = old + "\n`bash scripts/check-wg6d-seasonal.sh` runs a fixed L4 Earthlike WG-6D case and verifies the 24-phase seasonal contract, positive runoff and potential/realized discharge, exact annual runoff-target closure within `1e-6`, phase-routing and seasonal global water-balance closure within `1e-10`, bounded 2–12 year lake spin-up, final lake-surface cycle drift within `0.02 m`, nonempty active-lake state, and both intermittent and perennial realized-flow classifications.\n"
if text.count(old) != 1:
    raise SystemExit(f"expected one WG-6C acceptance paragraph, found {text.count(old)}")
text = text.replace(old, new, 1)

browser_start = text.index("## Browser diagnostic surface\n")
deferred_start = text.index("## Deferred\n", browser_start)
new_browser = r'''## Browser diagnostic surface

`index.html` is the single public Planet Engine Lab entrypoint through WG-6D. The former `drainage.html` and `worldgen-lab.html` entrypoints and the standalone drainage-page controller remain removed. Browser/WASM protocol version `14` carries WG-6D in the same cumulative result as WG-5, WG-6A, WG-6B, and WG-6C, so the primary Lab still performs one matched physical generation rather than issuing a redundant hydrology request.

The cumulative Lab retains all prior WG-6 diagnostics and adds:

- seasonal realized discharge at the selected orbital phase;
- realized-flow presence fraction;
- dry/intermittent/perennial realized-flow regime;
- seasonal snow storage at the selected orbital phase.

The existing season/orbital-phase control drives the phase-indexed WG-6D fields. WG-6B potential annual discharge and WG-6C annual realized discharge/lake diagnostics remain available beside the new seasonal fields so annual and seasonal hydrologic behavior can be compared directly.

'''
text = text[:browser_start] + new_browser + text[deferred_start:]

deferred_start = text.index("## Deferred\n")
new_deferred = r'''## Deferred

WG-6D completes the planned WG-6 generation-time surface-hydrology stack through phase runoff/discharge timing, snow accumulation/melt timing, dynamic seasonal lake storage, and dry/intermittent/perennial realized-flow classification. It does not model individual storms or flood peaks, groundwater, permanent snow/glacier mass balance, river width/depth or hydraulic geometry, floodplains/wetlands, channel migration, erosion, sediment transport, deltas, biomes, resources, or gameplay geography. Terrain response, incision, and sediment transport remain WG-7 or later stages.
'''
text = text[:deferred_start] + new_deferred

path.write_text(text)
