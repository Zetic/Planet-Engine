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

## State contract

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

## Core invariants

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

This first drainage-topology stage does not decide whether a depression contains water. WG-6C will combine the depression geometry with water balance to distinguish dry playas, endorheic lakes, and overflowing lakes.

## Performance policy

WG-6A is a graph problem and must remain inexpensive. The intended operations are:

- neighbor scan: `O(E)`;
- Priority-Flood: `O(N log N)` with the initial binary-heap implementation;
- receiver construction: `O(E)`;
- topological ordering and contributing-area accumulation: `O(N)`;
- basin/depression labeling: `O(N + E)`.

On the canonical geodesic sphere `E` is approximately `3N`. WG-6A should remain comfortably below WG-5 Stage-6 climate cost; a sustained L6 runtime above 0.5 seconds is a profiling trigger rather than an accepted architectural baseline.

### Fixed release benchmark

Final WG-6A semantics were measured on GitHub Actions Ubuntu with Rust `1.98.1`, optimized release builds, fixed seed `ci-wg6-drainage`, and three drainage-only timed runs after generating the upstream WG-4 surface once.

| Fine level | Coarse level | Plates | Samples | Mean | Median | Basins | Depressions | Depressed samples | Area closure error | Drainage hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L4 | L3 | 12 | 2,562 | 0.454 ms | 0.437 ms | 256 | 3 | 6 | `1.172e-15` | `f3363c2ae888f19f` |
| L6 | L4 | 16 | 40,962 | 7.809 ms | 7.807 ms | 1,871 | 13 | 59 | `5.371e-15` | `fb05caac8bd88e1d` |
| L7 | L5 | 24 | 163,842 | 33.617 ms | 33.500 ms | 4,475 | 107 | 621 | `4.082e-15` | `349c2cd272766983` |

The L6 result is roughly 64 times below the 0.5-second profiling trigger. Runtime is therefore not a WG-6A architectural concern at the accepted resolution range.

### Fixed-ancestry L6/L7 diagnostic

A separate diagnostic held seed (`ci-wg6-l7`), coarse physical level (L5), and plate count (24) fixed while refining only the final topology from L6 to L7:

| Fine level | Samples | Land area | Basins | Depressions | Depressed samples | Largest contributing area | Area closure error | Drainage hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L6 | 40,962 | 141,134,347.810 km² | 1,921 | 45 | 274 | 9,629,879.474 km² | `6.421e-15` | `12f73cdeef2bd213` |
| L7 | 163,842 | 143,517,141.575 km² | 4,466 | 96 | 1,295 | 10,064,607.961 km² | `3.048e-15` | `5de9f83c2c202f0d` |

Basin/depression counts are resolution-dependent topology diagnostics, not cross-resolution equality targets: L7 resolves outlet and depression structure that does not exist as separate cells at L6. The broad largest-drainage-area signal changes by about 4.5%, while exact contributing-area conservation remains at floating-point noise at both levels.

## Permanent acceptance

`bash scripts/check-wg6a-drainage.sh` runs a fixed L4 Earthlike drainage case in CI and verifies the browser-independent drainage diagnostics contract, positive land/contributing area, nonempty basin topology, finite nonnegative metrics, the canonical L4 sample count, and contributing-area closure within `1e-10`.

Wall-clock runtime is intentionally measured rather than hard-gated in shared CI, where runner variance would make a timing assertion flaky.

## Deferred

WG-6A does not include precipitation runoff, evapotranspiration losses, snowmelt discharge, river width, groundwater, lake water balance, wetlands, erosion, sediment transport, deltas, biomes, resources, or gameplay geography.
