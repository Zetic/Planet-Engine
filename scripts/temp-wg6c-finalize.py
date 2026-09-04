from pathlib import Path

path = Path('docs/worldgen-rewrite/WG6_HYDROLOGY.md')
text = path.read_text()

text = text.replace(
    'WG-6A does not decide whether a depression actually contains water. WG-6C will combine depression geometry with water balance to distinguish dry playas, endorheic lakes, and overflowing lakes.',
    'WG-6A does not decide whether a depression actually contains water. WG-6C combines depression geometry with water balance to distinguish dry depressions, endorheic lakes, overflowing lakes, and terminal storage.',
)
text = text.replace(
    'WG-6C will apply depression storage, lake equilibrium, endorheic retention, and spill activation and may therefore reduce or delay downstream realized discharge.',
    'WG-6C applies depression storage, lake equilibrium, endorheic retention, and spill activation and therefore produces a separate realized-discharge field downstream of active lakes.',
)

if '## WG-6C inputs and causality' not in text:
    marker = '\n## Permanent acceptance\n'
    if marker not in text:
        raise SystemExit('WG-6 permanent-acceptance marker not found')
    section = r'''
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

'''
    text = text.replace(marker, '\n' + section + marker, 1)

acceptance_anchor = '`bash scripts/check-wg6b-runoff.sh` runs a fixed L4 Earthlike WG-6B case and verifies the canonical sample count, finite nonnegative water-balance/discharge metrics, positive Earthlike precipitation/runoff/discharge, `AET <= precipitation`, runoff fraction bounds, annual `P = AET + runoff` closure at printed precision, physical maximum-discharge bounds, deterministic hash formatting, and terminal-discharge conservation within `1e-10`.\n'
if acceptance_anchor not in text:
    raise SystemExit('WG-6B acceptance paragraph not found')
if '`bash scripts/check-wg6c-lakes.sh`' not in text:
    text = text.replace(
        acceptance_anchor,
        acceptance_anchor + '\n`bash scripts/check-wg6c-lakes.sh` runs a fixed L4 Earthlike WG-6C case and verifies canonical sample count, nonempty equilibrium lake state, state-count consistency, positive finite lake geometry and discharge, deterministic hash formatting, and global annual lake/realized-flow water-balance closure within `1e-10`.\n',
        1,
    )

browser_start = text.index('\n## Browser diagnostic surface\n')
deferred_start = text.index('\n## Deferred\n', browser_start)
new_browser = r'''
## Browser diagnostic surface

`index.html` is the single public Planet Engine Lab entrypoint through WG-6C. The former `drainage.html` and `worldgen-lab.html` entrypoints and the standalone drainage-page controller remain removed. Browser/WASM protocol version `13` carries WG-6C in the same cumulative result as WG-5, WG-6A, and WG-6B, so the primary Lab does not issue a redundant second physical generation.

The cumulative Lab retains WG-6A/WG-6B diagnostics and adds:

- realized annual discharge;
- equilibrium lake depth;
- lake state;
- lake surface fraction.

WG-6B potential annual discharge remains available beside WG-6C realized discharge so lake retention/spill effects can be inspected directly.
'''
text = text[:browser_start] + '\n' + new_browser + text[deferred_start:]

deferred_start = text.index('\n## Deferred\n')
new_deferred = r'''
## Deferred

WG-6C is an annual equilibrium hydrology stage. It does not yet model seasonal runoff/discharge, snow accumulation and melt timing, intermittent/perennial classification, seasonal lake expansion, flood peaks, groundwater, river width/depth, wetlands, erosion, sediment transport, deltas, biomes, resources, or gameplay geography. Seasonal hydrology remains WG-6D; terrain response and sediment remain WG-7 or later stages.
'''
text = text[:deferred_start] + '\n' + new_deferred

path.write_text(text)
print('documented final WG-6C hydrology contract and evidence')
