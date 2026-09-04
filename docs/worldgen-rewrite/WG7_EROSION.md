# WG-7 — Fluvial erosion and sediment

WG-7 begins geomorphic evolution after the accepted WG-4 physical surface and WG-6 hydrology. WG-7A establishes deterministic erosive forcing and conservative sediment routing without changing terrain. WG-7B is reserved for applying that forcing to terrain and rebuilding drainage after the surface changes.

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

WG-7A advances the cumulative browser/WASM protocol to version `15`. The single cumulative planet request generates WG-7A after WG-6D; there is intentionally no separate browser `generateErosion` path that could create a mismatched physical ancestry.

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

## Deferred to WG-7B and later

WG-7A intentionally does not perform:

- applied incision or elevation change;
- valley widening or channel-bed geometry evolution;
- sedimentary fill or delta construction;
- lake-capacity/spill-threshold change from infill;
- drainage or depression recomputation after terrain change;
- iterative hydro-geomorphic feedback;
- hillslope diffusion, mass wasting, glacial erosion, weathering, soil production, or detailed lithology.

WG-7B should begin only after the WG-7A forcing, ancestry, resolution interpretation, and sediment-conservation contracts are treated as stable inputs.