# WG-7C — Post-erosion hydrology reconciliation

WG-7C closes the first terrain/hydrology feedback gap created by WG-7B.

WG-7B owns the evolved solid surface and one rebuilt drainage graph, but WG-6C lake equilibrium and WG-6D seasonal realized flow still describe the pre-erosion WG-4 surface. WG-7C reconciles those hydrologic states to the evolved surface without rerunning WG-5 climate or applying another terrain mutation.

## Causal contract

Accepted ancestry:

1. WG-4 fixed ocean mask / sea level.
2. WG-5 coupled climate and retained phase forcing.
3. WG-6B runoff parameters and accepted local water production.
4. WG-7B evolved solid elevation and rebuilt post-erosion drainage.

WG-7C then performs:

1. regenerate WG-6B runoff/discharge identity on the WG-7B drainage graph while retaining the same fixed climate forcing and fixed land/ocean mask;
2. solve WG-6C lake equilibrium against `WG-7B evolved_solid_elevation_m` and the rebuilt drainage depressions/spills;
3. solve WG-6D seasonal hydrology against the reconciled runoff/lake state;
4. emit explicit before/after diagnostics and a deterministic WG-7C identity.

WG-7C does **not** rerun climate, mutate terrain, rebuild drainage a second time, move the coastline, or apply sediment again.

## Performance policy

The stage must reuse the existing dense O(N) runoff/lake/seasonal kernels. WG-5 is not rerun. WG-7B drainage is reused directly. New persistent state should be limited to the final reconciled runoff/lake/seasonal states plus compact comparison fields; phase-major arrays must not be duplicated beyond what is required for the public final seasonal state.

## Acceptance

Permanent acceptance will require:

- exact WG-4/WG-5/WG-7B ancestry;
- reconciled runoff bound to the WG-7B rebuilt drainage hash;
- lake equilibrium bound to evolved elevation + rebuilt drainage;
- seasonal hydrology bound to reconciled runoff/lakes;
- runoff, lake, phase-routing, and seasonal water closure at numerical noise;
- deterministic hashes;
- nontrivial before/after hydrologic changes on a fixed acceptance seed;
- L4/L6/L7 runtime and fixed-ancestry resolution diagnostics.

## Deferred

Lake infill, sediment-driven lake-capacity evolution, deltas, alluvial fans, coastal construction, coastline/sea-level migration, hillslope transport, glaciers, weathering/regolith/soil, ecology, Regions, Features, resources, and gameplay geography remain downstream.
