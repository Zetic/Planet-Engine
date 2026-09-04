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

WG-6A does not decide whether a depression actually contains water. WG-6C will combine depression geometry with water balance to distinguish dry playas, endorheic lakes, and overflowing lakes.

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

WG-6B deliberately calls the result **potential discharge**. It routes water over WG-6A's hydrologic escape graph even through depressions so the downstream potential is defined, but it does not claim that a closed depression is already full or spilling. WG-6C will apply depression storage, lake equilibrium, endorheic retention, and spill activation and may therefore reduce or delay downstream realized discharge.

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

## Permanent acceptance

`bash scripts/check-wg6a-drainage.sh` runs a fixed L4 Earthlike drainage case in CI and verifies the browser-independent drainage diagnostics contract, positive land/contributing area, nonempty basin topology, finite nonnegative metrics, the canonical L4 sample count, and contributing-area closure within `1e-10`.

`bash scripts/check-wg6b-runoff.sh` runs a fixed L4 Earthlike WG-6B case and verifies the canonical sample count, finite nonnegative water-balance/discharge metrics, positive Earthlike precipitation/runoff/discharge, `AET <= precipitation`, runoff fraction bounds, annual `P = AET + runoff` closure at printed precision, physical maximum-discharge bounds, deterministic hash formatting, and terminal-discharge conservation within `1e-10`.

Wall-clock runtime is measured and documented but intentionally not hard-gated in shared CI, where runner variance would make a timing assertion flaky.

## Browser diagnostic surface

`index.html` is the single public Planet Engine Lab entrypoint through WG-6B. The former `drainage.html` and `worldgen-lab.html` entrypoints and the standalone drainage-page controller are removed. The cumulative Lab retains WG-6A diagnostics and adds:

- potential annual discharge;
- annual runoff depth;
- runoff fraction;
- annual actual evapotranspiration.

WG-6A remains available as a dedicated protocol command for non-primary diagnostic clients, but the primary Lab does not issue a redundant second drainage request.

## Deferred

WG-6B does not yet determine actual lake storage/spill state, realized downstream flow through closed basins, seasonal snowmelt timing, persistent-snow storage, intermittent channels, flood peaks, groundwater, river width/depth, wetlands, erosion, sediment transport, deltas, biomes, resources, or gameplay geography. Those remain assigned to WG-6C, WG-6D, WG-7, or later stages.
