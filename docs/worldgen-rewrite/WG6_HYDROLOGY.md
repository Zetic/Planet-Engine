# WG-6 Hydrology

WG-6 converts accepted WG-4 terrain and WG-5 climatological forcing into deterministic surface hydrology. WG-6 is a generation-time hydrologic model, not a weather or storm simulation.

## Stage decomposition

WG-6 is intentionally split into reviewable causal stages:

1. **WG-6A — drainage topology**: receivers, hydrologic escape elevations, depressions, basin/outlet identity, drainage order, and contributing area.
2. **WG-6B — runoff and discharge**: annual runoff generation from WG-5 forcing, downstream accumulation, and river discharge potential.
3. **WG-6C — lakes and closed basins**: depression storage, lake equilibrium, spill activation, and endorheic versus overflowing basins.
4. **WG-6D — seasonal hydrology**: seasonal discharge, snowmelt contribution, intermittent channels, and seasonal lake behavior.

WG-6A deliberately consumes terrain only. Rainfall changes how much water flows through a drainage graph, not the underlying topographic escape graph.

## WG-6A inputs

The drainage topology stage consumes the canonical fine-resolution physical surface:

- `GeodesicTopology` adjacency, dual-cell area, and center-to-center edge distance;
- WG-4 solid elevation;
- WG-4 land/ocean mask and solved sea level;
- `PlanetPhysicalParameters::radius_m` for physical areas and distances.

It does **not** depend on WG-5 solver internals or climate implementation details. Later WG-6 stages may consume the documented WG-5 forcing contract: precipitation, temperature, PET, moisture balance, snowfall fraction, persistent snow, and hydrologically relevant sea-ice potential.

## Drainage topology

For ordinary sloping terrain, each land sample routes toward a neighboring sample with lower hydrologic escape elevation using physical geodesic edge distance. On flats and inside depressions, routing follows deterministic Priority-Flood escape ancestry so the receiver graph remains acyclic without modifying WG-4 solid elevation.

The physical surface and routing surface remain distinct:

```text
solid_elevation_m             = immutable WG-4 terrain
hydrologic_escape_elevation_m = minimum water-surface elevation required to escape downstream
```

A depression therefore remains physically present even when the routing topology knows how it would spill.

## Priority-Flood escape topology

Ocean cells are terminal outlets. They enter Priority-Flood at the solved sea-surface elevation, not at bathymetric seafloor elevation, so deep offshore relief cannot steer coastal drainage. The graph is flooded outward from terminal outlets in increasing escape elevation. Each visited land sample records:

- minimum hydrologic escape elevation;
- deterministic flood parent;
- flood rank used to route equal-elevation flats without cycles.

For a dry planet with no ocean, the deterministic global minimum becomes an internal terminal sink so the topology remains defined.

## WG-6A state contract

`DrainageState` exposes:

- `receiver`: one downstream neighbor per non-terminal land sample;
- `outlet_sample` and resolved `outlet_kind` for every land sample;
- canonical `basin_id`, assigned by ascending terminal outlet sample;
- `depression_id`;
- `hydrologic_escape_elevation_m`;
- `depression_depth_m`;
- `contributing_area_m2`;
- upstream-to-downstream `drainage_order`;
- explicit basin and depression summary records;
- deterministic stage/hash metrics.

`INVALID_SAMPLE_ID` marks absent receivers, basin IDs, depression IDs, or outlets where appropriate. A terminal is identified by an absent receiver; `outlet_kind` describes the resolved terminal kind for the whole contributing land path.

## WG-6A core invariants

WG-6A must satisfy all of the following:

1. Every receiver is a real canonical topology neighbor.
2. Every non-terminal land sample has exactly one receiver.
3. Receiver traversal cannot form a cycle.
4. Every land sample resolves to an ocean outlet or explicit internal sink.
5. Contributing area is accumulated in a single upstream-to-downstream traversal.
6. Total contributing area arriving at terminal outlets equals total land area within numerical tolerance.
7. WG-4 solid elevation is never changed by depression handling.
8. Same seed + same topology + same WG-4 surface produces the same drainage hash.

## Depressions

WG-6A records connected depressed regions where the hydrologic escape elevation exceeds physical terrain. Each record includes floor location/elevation, spill elevation, area, and maximum depth.

WG-6A does not decide whether a depression actually contains water. WG-6C combines depression geometry with water balance to distinguish dry depressions, endorheic lakes, overflowing lakes, and terminal storage.

## WG-6A performance policy

WG-6A is a graph problem and must remain inexpensive. The intended operations are:

- neighbor scan: `O(E)`;
- Priority-Flood: `O(N log N)` with the initial binary-heap implementation;
- receiver construction: `O(E)`;
- topological ordering and contributing-area accumulation: `O(N)`;
- basin/depression labeling: `O(N + E)`.

On the canonical geodesic sphere `E` is approximately `3N`. WG-6A should remain comfortably below WG-5 Stage-6 climate cost; a sustained L6 runtime above 0.5 seconds is a profiling trigger rather than an accepted architectural baseline.

### WG-6A fixed release benchmark

Final WG-6A semantics were measured on GitHub Actions Ubuntu with Rust `1.98.1`, optimized release builds, fixed seed `ci-wg6-drainage`, and three drainage-only timed runs after generating the upstream WG-4 surface once.

| Fine level | Coarse level | Plates | Samples | Mean | Median | Basins | Depressions | Depressed samples | Area closure error | Drainage hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L4 | L3 | 12 | 2,562 | 0.454 ms | 0.437 ms | 256 | 3 | 6 | `1.172e-15` | `f3363c2ae888f19f` |
| L6 | L4 | 16 | 40,962 | 7.809 ms | 7.807 ms | 1,871 | 13 | 59 | `5.371e-15` | `fb05caac8bd88e1d` |
| L7 | L5 | 24 | 163,842 | 33.617 ms | 33.500 ms | 4,475 | 107 | 621 | `4.082e-15` | `349c2cd272766983` |

The L6 result is roughly 64 times below the 0.5-second profiling trigger. Runtime is therefore not a WG-6A architectural concern at the accepted resolution range.

### WG-6A fixed-ancestry L6/L7 diagnostic

A separate diagnostic held seed (`ci-wg6-l7`), coarse physical level (L5), and plate count (24) fixed while refining only the final topology from L6 to L7:

| Fine level | Samples | Land area | Basins | Depressions | Depressed samples | Largest contributing area | Area closure error | Drainage hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L6 | 40,962 | 141,134,347.810 km² | 1,921 | 45 | 274 | 9,629,879.474 km² | `6.421e-15` | `12f73cdeef2bd213` |
| L7 | 163,842 | 143,517,141.575 km² | 4,466 | 96 | 1,295 | 10,064,607.961 km² | `3.048e-15` | `5de9f83c2c202f0d` |

Basin/depression counts are resolution-dependent topology diagnostics, not cross-resolution equality targets: L7 resolves outlet and depression structure that does not exist as separate cells at L6. The broad largest-drainage-area signal changes by about 4.5%, while exact contributing-area conservation remains at floating-point noise at both levels.

## WG-6B inputs and causality

WG-6B stage identity is `hydrology:runoff-discharge@1`. It consumes accepted outputs rather than solving climate or drainage again:

```text
WG-5 annual precipitation + potential evaporation
                    +
WG-6A receiver graph + upstream-to-downstream drainage order
                    +
canonical dual-cell area + orbital-year duration
                    ↓
annual actual evapotranspiration
                    ↓
local annual runoff depth and local runoff volume
                    ↓
single downstream accumulation traversal
                    ↓
potential annualized discharge
```

The primary browser Lab obtains WG-6A and WG-6B from the same cumulative generation result as WG-5, so inspecting hydrology does not issue a second climate or drainage generation. Browser/WASM protocol version `12` carries the cumulative contract.

## Annual water balance

WG-6B uses Fu's analytical Budyko relation with default shape parameter `omega = 2.6`. For land precipitation `P` and potential evaporation `PET`, define aridity `phi = PET / P` and compute:

```text
AET = P * [1 + phi - (1 + phi^omega)^(1/omega)]
```

The result is explicitly bounded to `0 <= AET <= min(P, PET)`. Annual local runoff depth is then:

```text
R = P - AET
```

and runoff fraction is `R / P` when `P > 0`, otherwise zero. Submerged samples do not generate terrestrial runoff.

This is an annual climatological balance. WG-6B currently treats annual snow storage change as zero; it does not attempt to time snow accumulation and melt within the year. WG-6D will add seasonal snowmelt timing, persistent-snow retention, intermittent flow, and seasonal discharge.

## Runoff volume and discharge accumulation

Local runoff depth is converted to physical mean flow rate using dual-cell area and orbital-year duration:

```text
local_runoff_m3_s = R_mm * 1e-3 * cell_area_m2 / orbital_period_s
```

The local values are then accumulated exactly once in WG-6A's upstream-to-downstream drainage order. This is an `O(N)` graph traversal after the climate and drainage states already exist.

WG-6B deliberately calls the result **potential discharge**. It routes water over WG-6A's hydrologic escape graph even through depressions so the downstream potential is defined, but it does not claim that a closed depression is already full or spilling. WG-6C applies depression storage, lake equilibrium, endorheic retention, and spill activation and therefore produces a separate realized-discharge field downstream of active lakes.

## WG-6B state contract

`RunoffState` exposes:

- `actual_evapotranspiration_mm`;
- `local_runoff_mm`;
- `runoff_fraction`;
- `local_runoff_m3_s`;
- `potential_discharge_m3_s`;
- area-weighted land precipitation, AET, and runoff diagnostics;
- total generated local runoff flow;
- terminal accumulated discharge;
- discharge-conservation relative error;
- maximum potential discharge;
- deterministic runoff-parameter, climate, drainage, and WG-6B hashes.

The WG-6B hash includes stage identity/version, derived stage seed, planet parameter hash, runoff parameter hash, accepted WG-5 climate hash, accepted WG-6A drainage hash, and every public WG-6B output vector.

## WG-6B core invariants

WG-6B must satisfy all of the following:

1. Every WG-5, WG-6A, and WG-6B field aligns on the same canonical fine topology.
2. `0 <= AET <= min(P, PET)` on every land sample.
3. `runoff = precipitation - AET` and runoff fraction remains within `[0, 1]`.
4. Submerged samples generate no terrestrial local runoff.
5. Local runoff is converted to physical volumetric flow using canonical cell area and orbital-year duration.
6. Potential discharge is accumulated in one pass over the accepted WG-6A DAG; WG-6B never constructs an independent routing graph.
7. Sum of terminal accumulated discharge equals total locally generated runoff within numerical tolerance.
8. Same planet, seed, parameters, climate state, and drainage state produce the same WG-6B hash.
9. WG-4 terrain and WG-6A topology remain immutable.

## WG-6B performance policy

WG-6B is intentionally a cheap residual-water-balance plus graph-accumulation stage. After WG-5 and WG-6A are available, its work is linear in sample count and contains no iterative global solve or geological/weather time stepping.

A sustained WG-6B L6 runtime above 0.1 seconds would be a profiling trigger. The accepted implementation is two orders of magnitude below that threshold.

### WG-6B fixed release benchmark

Final WG-6B semantics were measured on GitHub Actions Ubuntu with Rust `1.98.1`, optimized release builds, fixed seed `ci-wg6b-runoff`, and three WG-6B-only timed runs after generating the upstream WG-5 and WG-6A states once.

| Fine level | Coarse level | Plates | Samples | Mean | Median | Mean land P | Mean land AET | Mean land runoff | Runoff fraction | Total local flow | Max potential discharge | Closure error | Runoff hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L4 | L3 | 12 | 2,562 | 0.084 ms | 0.086 ms | 489.383 mm/yr | 198.405 mm/yr | 290.977 mm/yr | 0.594581 | 852,982.691 m³/s | 82,895.227 m³/s | `4.094e-16` | `bf4f87e5829b54b8` |
| L6 | L4 | 16 | 40,962 | 1.378 ms | 1.384 ms | 717.269 mm/yr | 200.727 mm/yr | 516.542 mm/yr | 0.720151 | 2,464,567.617 m³/s | 748,724.827 m³/s | `1.323e-15` | `cc273575b8073aa5` |
| L7 | L5 | 24 | 163,842 | 5.439 ms | 5.416 ms | 1,532.612 mm/yr | 317.941 mm/yr | 1,214.671 mm/yr | 0.792550 | 4,529,256.282 m³/s | 471,619.895 m³/s | `6.786e-15` | `e59b60ed31a077a9` |

These three rows are performance cases with their normal coarse/fine settings; their hydrologic totals are not intended as cross-resolution convergence comparisons. The L6 stage itself is roughly 73 times below the 0.1-second profiling trigger.

### WG-6B accepted WG-5 fixed-ancestry diagnostic

A separate diagnostic uses the accepted WG-5 convergence seed `ci-wg5-l7`, holds the coarse physical level at L5 and plate count at 24, and refines only the final topology from L6 to L7:

| Fine level | Samples | Mean land P | Mean land AET | Mean land runoff | Runoff fraction | Total local flow | Max potential discharge | Closure error | Runoff hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L6 | 40,962 | 694.062 mm/yr | 264.622 mm/yr | 429.440 mm/yr | 0.618734 | 1,569,880.571 m³/s | 122,962.499 m³/s | `2.225e-15` | `0d329414adf433b9` |
| L7 | 163,842 | 744.195 mm/yr | 253.350 mm/yr | 490.845 mm/yr | 0.659565 | 1,808,947.664 m³/s | 173,264.611 m³/s | `9.525e-15` | `3ba80fbe4ff52ba0` |

Area-weighted land precipitation changes by about 7.2% across this accepted WG-5 ancestry. Because runoff is the residual after AET, the same forcing change is amplified to about 14.3% in mean runoff and about 15.2% in total generated flow. Maximum potential discharge is more resolution-sensitive because finer drainage topology can reorganize the largest resolved catchment. Exact river-by-river equality is therefore not an acceptance target; water-balance bounds, deterministic ancestry, broad drainage organization, and exact discharge conservation are.


## WG-6C inputs and causality

WG-6C stage identity is `hydrology:lakes-closed-basins@1`. It consumes accepted WG-6A depression/escape geometry, accepted WG-6B local runoff, and WG-5 precipitation/PET without changing WG-4 terrain or reconstructing the drainage graph:

```text
WG-6A depression membership + spill geometry
                    +
WG-6B local annual runoff
                    +
WG-5 precipitation + potential evaporation
                    ↓
depression inflow and hypsometric lake geometry
                    ↓
steady annual lake water-balance solve
                    ↓
dry / endorheic / overflowing / terminal-storage state
                    ↓
solved spill outflow injected into WG-6A downstream topology
                    ↓
realized annualized discharge
```

This is a generation-time equilibrium solve, not a year-by-year lake-filling simulation. The default open-water evaporation multiplier is `1.0`, applied to accepted WG-5 PET over the equilibrium lake surface.

## Lake equilibrium and depression control volumes

Each WG-6A depression is solved as one hydrologic control volume. WG-6C evaluates increasing candidate water surfaces using the depression members and physical dual-cell area, replacing submerged land runoff with direct lake-surface precipitation and evaporation as the lake expands. If evaporation balances catchment inflow below the spill elevation, the result is endorheic. If the balance remains positive at the spill elevation, the lake overflows and only the residual solved outflow is released downstream. A terminal depression that cannot spill records unreleased storage explicitly.

An active lake intercepts realized routing across the **entire owning depression**, not only samples covered by the equilibrium water surface. This is required because WG-6A escape ancestry can cross a higher, currently dry member of the same depression. Allowing such a member to route independently would count the same catchment inflow both in the lake balance and again in downstream realized flow. Solved overflow is therefore the only release from an active depression.

WG-4 `solid_elevation_m` remains immutable. Lake state is represented separately by lake identity/kind, equilibrium surface geometry, fractional water coverage, and lake depth.

## Potential versus realized discharge

WG-6B `potential_discharge_m3_s` remains unchanged and answers how much annualized runoff would traverse the WG-6A escape topology without lake retention. WG-6C adds `realized_discharge_m3_s`, which removes water retained/evaporated by active closed basins and injects only solved overflow at the lake spill receiver.

The global WG-6C water-balance diagnostic is:

```text
dry-land runoff + lake-surface precipitation
    = terminal realized discharge
    + lake-surface evaporation
    + unreleased terminal storage
```

All physical accounting remains `f64` internally through this conservation calculation; public per-sample lake fractions/depth/discharge are converted at the output boundary.

## WG-6C state contract

`LakeState` exposes:

- per-sample `lake_id` and `lake_kind`;
- equilibrium `lake_fraction` and `lake_depth_m`;
- per-sample `realized_discharge_m3_s`;
- lake records containing depression identity, kind, surface elevation, area, volume, maximum depth, gross land inflow, lake precipitation, evaporation, outflow, unreleased storage, and spill location;
- aggregate counts/area/volume/depth and global water-balance diagnostics;
- deterministic lake-parameter, climate, drainage, runoff, and WG-6C hashes.

Lake kinds are `1 = endorheic`, `2 = overflowing`, and `3 = terminal storage`; `0` means no active lake on a sample.

The WG-6C hash includes stage identity/version, derived stage seed, planet and lake parameter hashes, accepted climate/drainage/runoff ancestry hashes, public lake fields, and lake records.

## WG-6C core invariants

WG-6C must satisfy all of the following:

1. WG-5, WG-6A, WG-6B, and WG-6C inputs align on the same canonical fine topology.
2. WG-4 solid terrain and WG-6A receiver/depression topology remain immutable.
3. Lake surface elevation never exceeds the owning depression spill elevation for spill-capable basins.
4. Endorheic lakes release no downstream spill flow; overflowing lakes release only the solved positive residual at the spill elevation.
5. Realized routing is blocked across every sample of an active depression, including dry members outside the equilibrium lake surface; only solved spill outflow is reinjected downstream.
6. Lake fraction stays within `[0, 1]`; lake area, volume, depth, precipitation, evaporation, outflow, and storage remain finite and nonnegative.
7. Global annual water balance closes to numerical tolerance using dry-land runoff, lake precipitation, terminal realized discharge, lake evaporation, and unreleased storage.
8. WG-6B potential discharge is preserved as a separate diagnostic and is not overwritten by WG-6C realized discharge.
9. Same planet, seed, parameters, climate, drainage, and runoff ancestry produce the same WG-6C hash.

## WG-6C performance policy

WG-6C is a direct equilibrium solve over explicit WG-6A depressions plus one realized-routing traversal. It contains no geological-year or weather time stepping. A sustained L6 WG-6C runtime above `0.1 s` is a profiling trigger; the accepted implementation is roughly fifty times below that threshold.

### WG-6C fixed release benchmark

Final WG-6C semantics were measured on GitHub Actions Ubuntu with Rust `1.98.1`, optimized release builds, fixed seed `ci-wg6c-lakes`, and three WG-6C-only timed runs after generating upstream WG-5/WG-6A/WG-6B state once.

| Fine level | Coarse level | Plates | Samples | Mean | Median | Lakes | Endorheic | Overflowing | Lake samples | Lake area | Lake volume | Max depth | Max realized discharge | Closure error | Lake hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L4 | L3 | 12 | 2,562 | 0.121 ms | 0.124 ms | 3 | 3 | 0 | 3 | 136,303.141 km² | 9,238.630 km³ | 74.235 m | 279,901.630 m³/s | `0.000e0` | `09b742e18a7bcb0a` |
| L6 | L4 | 16 | 40,962 | 1.839 ms | 1.827 ms | 19 | 14 | 5 | 47 | 493,483.393 km² | 78,239.426 km³ | 405.577 m | 418,828.323 m³/s | `1.470e-15` | `a71807555aaa625c` |
| L7 | L5 | 24 | 163,842 | 7.199 ms | 7.017 ms | 117 | 63 | 54 | 1,715 | 4,957,477.241 km² | 733,189.563 km³ | 776.147 m | 445,590.018 m³/s | `9.860e-16` | `06e244037f7ab093` |

The rows above are performance cases with their normal coarse/fine settings, so lake counts and totals are not cross-resolution convergence targets. Even at L7 the WG-6C solve itself remains below 10 ms on the accepted runner class.

### WG-6C accepted WG-5 fixed-ancestry diagnostic

The accepted WG-5 convergence seed `ci-wg5-l7` holds coarse physical level L5 and plate count 24 fixed while refining only the final topology from L6 to L7:

| Fine level | Samples | Runtime | Lakes | Endorheic | Overflowing | Lake samples | Lake area | Lake volume | Max depth | Terminal realized discharge | Max realized discharge | Closure error | Lake hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L6 | 40,962 | 1.811 ms | 50 | 35 | 15 | 92 | 694,672.651 km² | 54,207.938 km³ | 656.330 m | 1,561,175.467 m³/s | 122,908.432 m³/s | `1.328e-15` | `5fdc23e5769f1a72` |
| L7 | 163,842 | 8.546 ms | 110 | 71 | 39 | 324 | 783,343.965 km² | 49,653.688 km³ | 679.648 m | 1,797,421.034 m³/s | 172,026.802 m³/s | `2.432e-15` | `aaed5ec3f29d8651` |

Exact lake count and cell membership are resolution-sensitive because finer WG-6A terrain resolves additional depression geometry. On this fixed ancestry, total lake area changes by about 12.8% and lake volume by about 8.4%, while terminal realized flow changes consistently with the accepted WG-5/WG-6B forcing shift. Conservation remains at floating-point noise at both resolutions.


## Permanent acceptance

`bash scripts/check-wg6a-drainage.sh` runs a fixed L4 Earthlike drainage case in CI and verifies the browser-independent drainage diagnostics contract, positive land/contributing area, nonempty basin topology, finite nonnegative metrics, the canonical L4 sample count, and contributing-area closure within `1e-10`.

`bash scripts/check-wg6b-runoff.sh` runs a fixed L4 Earthlike WG-6B case and verifies the canonical sample count, finite nonnegative water-balance/discharge metrics, positive Earthlike precipitation/runoff/discharge, `AET <= precipitation`, runoff fraction bounds, annual `P = AET + runoff` closure at printed precision, physical maximum-discharge bounds, deterministic hash formatting, and terminal-discharge conservation within `1e-10`.

`bash scripts/check-wg6c-lakes.sh` runs a fixed L4 Earthlike WG-6C case and verifies canonical sample count, nonempty equilibrium lake state, state-count consistency, positive finite lake geometry and discharge, deterministic hash formatting, and global annual lake/realized-flow water-balance closure within `1e-10`.

Wall-clock runtime is measured and documented but intentionally not hard-gated in shared CI, where runner variance would make a timing assertion flaky.


## Browser diagnostic surface

`index.html` is the single public Planet Engine Lab entrypoint through WG-6C. The former `drainage.html` and `worldgen-lab.html` entrypoints and the standalone drainage-page controller remain removed. Browser/WASM protocol version `13` carries WG-6C in the same cumulative result as WG-5, WG-6A, and WG-6B, so the primary Lab does not issue a redundant second physical generation.

The cumulative Lab retains WG-6A/WG-6B diagnostics and adds:

- realized annual discharge;
- equilibrium lake depth;
- lake state;
- lake surface fraction.

WG-6B potential annual discharge remains available beside WG-6C realized discharge so lake retention/spill effects can be inspected directly.


## Deferred

WG-6C is an annual equilibrium hydrology stage. It does not yet model seasonal runoff/discharge, snow accumulation and melt timing, intermittent/perennial classification, seasonal lake expansion, flood peaks, groundwater, river width/depth, wetlands, erosion, sediment transport, deltas, biomes, resources, or gameplay geography. Seasonal hydrology remains WG-6D; terrain response and sediment remain WG-7 or later stages.
