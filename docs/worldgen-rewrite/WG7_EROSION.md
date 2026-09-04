# WG-7 — Fluvial erosion and sediment

WG-7 begins geomorphic evolution after the accepted WG-4 physical surface and WG-6 hydrology. WG-7A establishes deterministic erosive forcing and conservative sediment routing without changing terrain. WG-7B consumes that accepted forcing to create a distinct bounded evolved surface, apply conservative sediment deposition, rebuild drainage once, and reroute accepted annual runoff on the rebuilt DAG.

## WG-7A stage identity

The implemented stage is:

```text
geomorphology:fluvial-erosion-sediment@1
```

Its output is deterministic for the stage seed, parameter set, canonical fine topology, and accepted upstream physical identities.

WG-7A consumes the existing physical stack rather than rerunning it:

```text
WG-6D phase realized discharge
             +
WG-4 solid terrain + WG-6A receiver DAG
             +
WG-3.75 inherited strength / weakness / structural fabric / fragmentation
             +
WG-6C active lake-depression control volumes
             ↓
peak-sensitive effective erosive discharge
             ↓
receiver slope + hydraulic channel width + inherited erodibility
             ↓
bounded stream-power forcing and diagnostic incision potential
             ↓
local sediment production + transport capacity
             ↓
conservative routing on the accepted WG-6A DAG
             ↓
land deposition + lake trapping + terminal/ocean deposition
```

The stage retains explicit ancestry hashes for inheritance, topography, drainage, lakes, and seasonal hydrology. The cumulative browser Lab rejects a WG-7A result whose identities do not match the accepted upstream state being displayed.

## Effective erosive discharge

WG-7A does not use only annual mean discharge. For each land sample it reads all WG-6D realized-discharge phases and computes a power mean:

```text
Q_eff = (mean(Q_phase^p))^(1/p)
```

The default is `p = 2`. This makes erosive forcing sensitive to seasonal peaks while remaining bounded by the generated phase hydrograph. A steady hydrograph therefore produces its steady discharge, while a flashy hydrograph with the same arithmetic mean produces a larger erosive `Q_eff`.

## Channel slope and hydraulic width

Channel slope is evaluated only along the accepted WG-6A receiver edge:

```text
S = max(0, (z_i - z_receiver) / receiver_edge_length)
```

where `z` is immutable WG-4 `solid_elevation_m` and edge length is the canonical spherical neighbor distance in meters.

WG-7A derives a diagnostic hydraulic width from effective discharge:

```text
W = clamp(W_ref * (Q_eff / Q_ref)^b, W_min, W_max)
```

Default values are:

- `Q_ref = 100 m³/s`;
- `W_ref = 20 m`;
- width exponent `b = 0.5`;
- minimum width `1 m`;
- maximum width `2,000 m`.

The initial WG-7A implementation briefly estimated sediment-producing area as a constant fraction of each dual cell. That made production unnecessarily sensitive to mesh resolution. The accepted formulation instead uses receiver-segment length and discharge-derived channel width, so sediment production follows channel geometry rather than cell area.

## Inherited erodibility

WG-7A derives a bounded dimensionless erodibility index from already accepted lithospheric inheritance:

```text
K = clamp(
      0.05
    + 0.35 * weakness
    + 0.25 * fragmentation
    + 0.15 * structural_fabric
    + 0.20 * (1 - strength),
    0.05,
    1.0)
```

This is deliberately a first-order mechanical erodibility proxy. Detailed lithology, weathering, soil production, and chemistry are not inferred here.

## Bounded incision potential

WG-7A computes nondimensional stream-power forcing from effective discharge and channel slope:

```text
F = (Q_eff / Q_ref)^m * (S / S_ref)^n
B = F / (1 + F)
I = I_max * K * B
```

Default parameters are:

- discharge exponent `m = 0.5`;
- slope exponent `n = 1.0`;
- reference slope `S_ref = 0.01`;
- diagnostic maximum incision `I_max = 0.01 m/yr`.

`I` is **incision potential**, not an applied terrain delta. The saturating response prevents extreme discharge/slope values from producing unbounded diagnostic incision before terrain-evolution semantics exist.

## Sediment production

Local sediment supply uses the accepted channel-geometry formulation:

```text
sediment_supply =
    incision_rate_m_per_s
  * receiver_segment_length_m
  * channel_width_m
  * sediment_bulk_density_kg_per_m3
```

The default bulk density is `1,800 kg/m³`. Samples without a downstream receiver have zero segment length and therefore generate no local fluvial sediment in this stage.

Global generated-mass accounting sums the exact public/routed `f32` source values. This is intentional: an early implementation summed the pre-cast `f64` values while routing the `f32` vector and created a small bookkeeping residual. Conservation now measures the exact mass that the router receives.

## Transport capacity and routing

For land samples with positive effective discharge and slope, transport capacity is:

```text
C = C_ref
  * (Q_eff / Q_ref)^a
  * (S / S_ref)^c
```

with defaults:

- `C_ref = 50 kg/s`;
- discharge exponent `a = 1.2`;
- slope exponent `c = 1.0`.

Routing follows the already accepted WG-6A upstream-to-downstream order. At each ordinary land sample:

```text
available = incoming_load + local_supply
carried   = min(available, capacity)
land_deposition = available - carried
```

The carried load is passed to the WG-6A receiver. If the receiver is ocean, the carried load becomes terminal/ocean deposition. If a land sample has no receiver, its available load is also accounted as terminal deposition rather than disappearing from the budget.

## Active lake depressions are sediment control volumes

WG-7A preserves the WG-6C control-volume rule. If a depression has an active WG-6C lake, **every land sample in that depression is a complete first-pass sediment trap**, including dry/high members of the same depression.

This prevents the same class of bypass that WG-6C had to eliminate for realized water routing: a route may geometrically cross a dry member of an active depression without crossing a currently wet lake-surface cell. Treating only wet cells as traps would allow sediment to escape the lake control volume incorrectly.

WG-7A therefore deposits all available sediment at the first active-lake-depression sample encountered and does not route that load onward. Sediment remobilization, delta growth, lake infill, and spill-threshold evolution belong to later terrain-evolution work.

## Conservation invariant

For every accepted WG-7A result:

```text
total sediment generated
  = land deposition
  + lake deposition
  + terminal/ocean deposition
```

The reported relative error is:

```text
abs(total_deposition - total_generated) / total_generated
```

when generated mass is positive. Permanent CI (`scripts/check-wg7a-erosion.sh`) requires sediment closure at or below `1e-10`. Current fixed benchmarks close at floating-point noise.

## State contract

`FluvialErosionState` exposes per-sample vectors for:

- effective erosive discharge;
- channel slope;
- hydraulic channel width;
- inherited erodibility;
- stream-power forcing index;
- incision potential;
- local sediment supply;
- sediment transport capacity;
- routed sediment load;
- sediment deposition.

Metrics include erosive-sample count, active lake traps receiving sediment, maxima for discharge/slope/width/incision/load, generated and deposited sediment totals, conservation error, parameter identity, all upstream ancestry hashes, and the final WG-7A hash.

Every public WG-7A vector participates in the deterministic stage hash.

## Browser and Lab contract

The cumulative browser/WASM contract is now version `16`. The single cumulative planet request still generates WG-7A after WG-6D and then WG-7B; there is intentionally no separate browser `generateErosion` or `generateEvolution` path that could create mismatched physical ancestry.

The primary Planet Engine Lab exposes WG-7A diagnostics for:

- effective erosive discharge;
- channel slope;
- hydraulic channel width;
- inherited erodibility;
- incision potential;
- local sediment supply;
- routed sediment load;
- sediment deposition.

The Lab also reports the WG-7A stage identity, erosive sample count, lake traps, maximum forcing geometry, generated sediment, deposition split, conservation closure, and erosion hash while retaining every earlier diagnostic surface.

## Resolution interpretation

The fixed release performance cases change coarse physical level and plate count together with fine resolution. Their absolute sediment totals are therefore **not** cross-resolution convergence targets.

A separate fixed-ancestry diagnostic holds seed `ci-wg5-l7`, coarse physical level `L5`, and plate count `24` constant while refining only the final topology:

| Fine level | Samples | WG-7A mean runtime | Erosive samples | Generated sediment | Lake deposition | Terminal/ocean deposition | Closure | Erosion hash |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| L6 | 40,962 | 7.557 ms | 9,148 | 4,549.553 kg/s | 405.316 kg/s | 4,133.986 kg/s | `7.996e-16` | `0201ee1848a5ae59` |
| L7 | 163,842 | 31.071 ms | 36,997 | 5,089.738 kg/s | 642.577 kg/s | 4,411.300 kg/s | `1.072e-15` | `4dfc291ab7725c97` |

Generated sediment changes by about **11.9%** from L6 to L7 under fixed ancestry. Exact channel membership, slope, lake interception, and deposition location remain resolution-sensitive diagnostics because L7 resolves receiver geometry absent at L6. The fixed-ancestry result is accepted as an interpretation check, not an equality requirement.

## Fixed release benchmark

GitHub Actions Ubuntu, Rust `1.98.1`, optimized release build, seed `ci-wg7a-erosion`, three WG-7A-only timed runs after upstream generation:

| Case | Samples | Mean runtime | Erosive samples | Max effective Q | Max slope | Max width | Max incision | Generated sediment | Lake deposition | Terminal/ocean deposition | Closure |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| L4 / coarse L3 / 12 plates | 2,562 | 0.501 ms | 659 | 146,996 m³/s | 0.025730 | 766.802 m | 0.009406 m/yr | 7,068.830 kg/s | 1.622 kg/s | 7,062.359 kg/s | `0.000e0` |
| L6 / coarse L4 / 16 plates | 40,962 | 8.850 ms | 11,224 | 301,864 m³/s | 0.084199 | 1,098.843 m | 0.009721 m/yr | 6,017.336 kg/s | 1,160.134 kg/s | 4,842.843 kg/s | `6.046e-16` |
| L7 / coarse L5 / 24 plates | 163,842 | 35.563 ms | 43,910 | 486,760 m³/s | 0.154307 | 1,395.364 m | 0.009799 m/yr | 13,101.571 kg/s | 1,705.656 kg/s | 11,375.796 kg/s | `3.054e-15` |

WG-7A remains small relative to the full coupled climate/hydrology generation cost. A sustained WG-7A-only L7 runtime above `0.1 s` is a profiling trigger, not an accepted architectural baseline.

## WG-7A invariants

1. All input fields align on the canonical fine topology.
2. WG-6D drainage and lake ancestry must match the supplied WG-6A/WG-6C states.
3. The complete WG-6D phase realized-discharge field is required.
4. Submerged samples do not generate local fluvial sediment.
5. Channel slope and segment length come only from the accepted WG-6A receiver edge.
6. Incision potential is finite, non-negative, bounded, and does not mutate terrain.
7. Active WG-6C depressions are complete sediment traps across the whole depression membership.
8. Generated sediment is conserved into land, lake, and terminal/ocean deposition.
9. Public state and upstream identities are included in deterministic hashing.
10. WG-4 terrain, WG-6A receivers/depressions, and all accepted WG-6 hydrology remain immutable in WG-7A.

## WG-7A boundary carried into WG-7B

WG-7A intentionally does not perform:

- applied incision or elevation change;
- valley widening or channel-bed geometry evolution;
- sedimentary fill or delta construction;
- lake-capacity/spill-threshold change from infill;
- drainage or depression recomputation after terrain change;
- iterative hydro-geomorphic feedback;
- hillslope diffusion, mass wasting, glacial erosion, weathering, soil production, or detailed lithology.

WG-7B now consumes WG-7A under exactly those forcing, ancestry, resolution-interpretation, and sediment-conservation contracts.

## WG-7B stage identity

The implemented terrain-mutation stage is:

```text
geomorphology:bounded-terrain-evolution@1
```

WG-7B does not overwrite WG-4. It owns a distinct evolved-terrain state whose identity includes the stage seed, WG-7B parameter hash, accepted WG-4/WG-6/WG-7A ancestry, evolved surface, rebuilt drainage identity, applied erosion/deposition state, changed-receiver mask, and rerouted post-erosion discharge.

The generation-time causality is:

```text
accepted WG-7A incision potential + channel width + transport capacity
             +
immutable WG-4 solid terrain / sea level / ocean mask
             +
accepted WG-6A receiver DAG + depression membership
             +
accepted WG-6B local runoff + WG-6C active lake depressions
             ↓
channel-to-valley cell-average erosion rate
             ↓
one adaptive bounded geomorphic horizon
             ↓
applied erosion + conservative applied-sediment routing
             ↓
ordinary land deposition + lake sink + terminal/ocean sink
             ↓
distinct evolved solid surface
             ↓
one WG-6A drainage rebuild on evolved terrain
             ↓
accepted WG-6B local runoff rerouted over rebuilt drainage
```

### Valley-footprint terrain response

WG-7A incision is a channel-bed diagnostic and must not lower an entire dual cell by the same depth. WG-7B expands the accepted hydraulic channel width into a bounded coarse valley width and converts incision to cell-average erosion:

```text
valley_width = clamp(channel_width * valley_width_multiplier,
                     minimum_valley_width,
                     maximum_valley_width)
valley_area = min(receiver_segment_length * valley_width, dual_cell_area)
resolved_erosion_rate = incision_potential * valley_area / dual_cell_area
```

Default valley parameters are multiplier `3`, minimum width `100 m`, and maximum width `20,000 m`.

### One adaptive direct geomorphic horizon

WG-7B is not a year-stepped landscape simulator. It first estimates the maximum resolved erosion or ordinary-land deposition rate, then chooses one direct duration:

```text
duration = min(maximum_geomorphic_years,
               maximum_resolved_elevation_change / maximum_resolved_rate)
```

The defaults are `250,000 years` and `120 m`. The stage therefore remains a small set of dense passes plus one drainage rebuild instead of repeatedly invoking climate/hydrology and erosion.

Applied erosion is additionally limited by available relief. Existing WG-4 land remains at least `1 m` above the fixed WG-4 sea level, and an eroding land cell remains at least `0.1 m` above its accepted downstream land receiver. These are v1 stability/base-level guards, not claims of detailed channel-bed geometry.

### Applied sediment ledger

After bounded erosion is known, WG-7B recomputes sediment production from the **actually applied erosion volume** rather than carrying forward WG-7A diagnostic production unchanged. The default source and deposited-sediment densities are both `1,800 kg/m³`.

Applied sediment is routed once over the accepted pre-erosion WG-6A DAG using the accepted WG-7A transport-capacity field. Ordinary land deposition is converted to elevation gain on the evolved surface. Every member of an active WG-6C depression remains a complete lake sink, and terminal/ocean export is retained as an explicit sink. Lake and terminal/ocean sinks do not construct lake fill or deltas in WG-7B v1.

The required mass invariant is:

```text
applied sediment generated
  = ordinary land deposition
  + active-lake sink
  + terminal/ocean sink
```

Permanent CI requires relative closure at or below `1e-10`.

### Evolved terrain and fixed coastline

`evolved_solid_elevation_m` is a new WG-7B field. WG-4 `solid_elevation_m`, its component decomposition, topography hash, solved sea level, and submerged mask remain immutable upstream truth.

WG-7B v1 deliberately preserves the WG-4 ocean mask. Existing land cannot erode through the fixed sea-level clearance and existing ocean cells are not raised by terminal sediment. This avoids claiming coastline migration before coastal construction and water-volume/sea-level feedback are modeled together.

### One drainage rebuild and cheap runoff reroute

After applying the terrain delta, WG-7B invokes the accepted WG-6A drainage solver exactly once on the evolved solid surface while retaining the WG-4 ocean mask. It records which land samples changed receiver and the complete rebuilt contributing-area state internally.

WG-7B does **not** rerun WG-5 climate, WG-6C lake equilibrium, or WG-6D seasonal hydrology. For a first post-erosion hydrologic diagnostic, it reroutes the accepted WG-6B local annual runoff field over the rebuilt receiver DAG. This preserves annual runoff mass and exposes how changed terrain alters potential flow concentration without introducing an implicit iterative hydro-geomorphic loop.

Permanent CI requires rebuilt drainage area closure and post-erosion runoff closure at or below `1e-10`.

### WG-7B state contract

`TerrainEvolutionState` exposes or retains:

- evolved solid elevation;
- signed terrain delta;
- applied erosion depth;
- applied ordinary-land deposition depth;
- applied sediment source/load/deposition ledgers;
- a receiver-changed mask;
- the rebuilt drainage state;
- post-erosion potential annual discharge.

Metrics include the selected geomorphic duration, eroded/depositional/receiver-changed sample counts, maximum and mean terrain-change magnitudes, applied sediment source/sink totals and closure, maximum post-erosion potential discharge and runoff closure, parameter/upstream hashes, evolved-surface hash, rebuilt-drainage hash, and final terrain-evolution hash.

### Browser and Lab contract

Protocol `16` carries WG-7B in the same cumulative climate/planet result after WG-7A. The primary Lab exposes:

- evolved solid elevation;
- signed terrain elevation delta;
- applied erosion;
- applied ordinary-land deposition;
- changed receiver locations;
- post-erosion contributing area;
- post-erosion potential discharge.

The Lab validates that WG-7B references the exact displayed WG-4 topography, WG-6A drainage, WG-6B runoff, WG-6C lake, and WG-7A erosion identities. It also reports the selected horizon, terrain-change counts/magnitudes, sediment sink split and closure, post-erosion runoff closure, and WG-7B hashes.

### WG-7B invariants

1. Every input aligns on the canonical fine topology.
2. WG-7B requires exact accepted WG-4/WG-6/WG-7A ancestry.
3. WG-4 terrain identity and submerged mask remain unchanged.
4. Channel incision is converted to cell-average valley erosion before terrain mutation.
5. The direct geomorphic horizon is finite, nonnegative, and bounded by the parameterized duration/elevation-change limits.
6. Applied erosion cannot cross the fixed WG-4 land/sea or accepted downstream base-level guards.
7. Sediment production is recomputed from actually applied erosion.
8. Applied sediment closes into ordinary land deposition, active-lake sinks, and terminal/ocean sinks.
9. Only ordinary land deposition modifies elevation in WG-7B v1.
10. Drainage is rebuilt exactly once after terrain mutation.
11. Accepted WG-6B local runoff is rerouted on that rebuilt DAG without rerunning climate or seasonal hydrology.
12. Every public WG-7B diagnostic and ancestry identity participates in deterministic hashing.

### Final WG-7B benchmark matrix

The final benchmark matrix was run on GitHub Actions Ubuntu 24.04 with Rust `1.98.1`, optimized release builds, and three WG-7B-only timed runs after upstream state construction. Run `33923547403` benchmarked commit `2099497ede3272ed6ed71495c12d61f1bf60769c`. The WG-7B parameter hash is `79ebfd14fdef843c` in every case.

Fixed release seed `ci-wg7b-evolution`:

| fine level | coarse level | plates | samples | runtime mean / median (ms) | eroded / depositional | receiver changes | max erosion / deposition (m) | mean land abs Δz (m) | sediment closure | drainage area closure | runoff closure |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| L4 | L3 | 12 | 2,562 | 0.630 / 0.631 | 487 / 124 | 1 | 6.526032 / 1.146689 | 0.431530 | `3.657e-16` | `1.541e-16` | `0` |
| L6 | L4 | 16 | 40,962 | 8.547 / 8.611 | 8,938 / 3,141 | 30 | 38.269152 / 31.045940 | 0.800932 | `1.774e-16` | `3.264e-15` | `1.942e-16` |
| L7 | L5 | 24 | 163,842 | 43.938 / 44.813 | 30,026 / 14,497 | 70 | 35.117153 / 30.102793 | 1.128892 | `1.988e-15` | `4.269e-15` | `1.059e-14` |

All three fixed-release cases selected the full `250,000 year` bounded horizon. Their applied-sediment and post-erosion identities are:

| fine level | generated / land / lake / terminal-ocean sediment (kg/s) | max post-erosion Q (m³/s) | evolution hash | evolved-surface hash | rebuilt-drainage hash | WG-7A erosion hash |
|---:|---:|---:|---|---|---|---|
| L4 | 9,948.766885 / 161.149188 / 235.152654 / 9,552.465043 | 82,860.638672 | `71838b2cc6b700f7` | `910e5346a04431f7` | `017602cbf617a7b5` | `60c2ee7716d00fd2` |
| L6 | 20,504.592989 / 1,205.766005 / 2,793.767165 / 16,505.059818 | 493,990.306288 | `a29a936533c787c4` | `a9da933d095ecf17` | `afa34e4e3e4e05c7` | `975860e59fbbaf1f` |
| L7 | 25,620.296763 / 1,150.176735 / 1,027.475446 / 23,442.644583 | 144,990.635417 | `606bd14884243c3d` | `0a974d1f538c4f8c` | `f370f70d1392efeb` | `d290e5f38b0114a7` |

Fixed accepted WG-5 ancestry seed `ci-wg5-l7`, coarse L5, 24 plates:

| fine level | samples | runtime mean / median (ms) | eroded / depositional | receiver changes | max erosion / deposition / abs Δz (m) | mean land abs Δz (m) | sediment closure | drainage area closure | runoff closure |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| L6 | 40,962 | 8.730 / 8.815 | 9,139 / 3,179 | 13 | 27.247991 / 36.617950 / 36.420056 | 0.631664 | `0` | `3.115e-15` | `1.335e-15` |
| L7 | 163,842 | 46.904 / 46.339 | 36,961 / 16,520 | 134 | 57.346347 / 102.291346 / 101.656348 | 0.784727 | `5.295e-16` | `1.438e-14` | `2.574e-15` |

Both accepted-ancestry cases also selected `250,000 years`. L6 generated `16,507.432963 kg/s` of applied sediment, closing into `728.049710` land, `1,152.293564` lake, and `14,627.089689 kg/s` terminal/ocean deposition; its evolution/surface/drainage hashes are `6f6e08c69167ea2a`, `40ae5f0105a3e36c`, and `5e2f369357847826`. L7 generated `20,610.611447 kg/s`, closing into `1,629.925343` land, `2,229.895484` lake, and `16,750.790621 kg/s` terminal/ocean deposition; its evolution/surface/drainage hashes are `a60c68094eda5373`, `12c720550eb3fa63`, and `e4aa3b27b0d61992`.

The incremental stage is comfortably below its performance policy: fixed-release L7 is `43.938 ms` mean and accepted-ancestry L7 is `46.904 ms` mean, versus the `150 ms` preferred target and `250 ms` profiling trigger. The benchmark also demonstrates that terrain mutation can change basin/depression counts on accepted ancestry (L6: 1,916→1,915 basins and 50→49 depressions; L7: 4,395→4,394 basins and 110→107 depressions) while preserving drainage-area and rerouted-runoff conservation.

### Deferred beyond WG-7B

WG-7B intentionally does not yet model:

- coastline migration or a new water-volume/sea-level solve;
- evolving lake capacity, bathymetry, spill thresholds, or lake infill;
- delta, alluvial-fan, estuary, or other coastal construction;
- iterative climate/runoff/lake/erosion feedback after each terrain update;
- explicit channel cross-sections, migration, avulsion, or floodplains;
- hillslope diffusion, mass wasting, regolith, weathering, or soil production;
- glacier flow and glacial erosion/deposition;
- detailed lithology, chemistry, resources, Regions, Features, or gameplay geography.

WG-7B is therefore the first bounded terrain-response pass, not a terminal landscape-evolution model.
