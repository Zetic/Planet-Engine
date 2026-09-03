# Resolution Strategy

The Planet Engine has no universal `surfaceResolution`. Resolution belongs to a physical process.

For icosahedral subdivision level `L`, the sample count is approximately:

```text
N = 10 × 4^L + 2
```

Representative Earth-area scales from the rewrite blueprint:

| Level | Samples | Average area/sample | Characteristic spacing |
|---:|---:|---:|---:|
| 5 | 10,242 | ~49,800 km² | ~223 km |
| 6 | 40,962 | ~12,450 km² | ~112 km |
| 7 | 163,842 | ~3,113 km² | ~56 km |
| 8 | 655,362 | ~778 km² | ~28 km |
| 9 | 2,621,442 | ~195 km² | ~14 km |
| 10 | 10,485,762 | ~49 km² | ~7 km |

These are profiling candidates, not fixed stage assignments.

A likely early investigation is:

```text
plates / macro crust        L5–L6
geological history          L6–L7
lithosphere / structures    L6–L7
initial terrain             L7
climate                     L7–L8
hydrology / global erosion  L8 initially
local refinement            separate chunked system later
```

Fine detail must refine coarse physical truth rather than independently regenerate it. Meter-scale detail is never materialized globally.

## Current staged implementation

WG-1 makes the hierarchical topology and stable sample ancestry available before physical process resolutions are permanently assigned. WG-2, WG-3, and WG-3.5 currently accept an explicit topology level and evaluate their dense fields at that same requested level. This is an implementation checkpoint, not a declaration that tectonics, crust, lithosphere, terrain, and climate must share one permanent resolution.

The stage identities are already separated: WG-2 owns macro plate identity/kinematics, WG-3 owns crust/history, and WG-3.5 owns mechanical/structural state plus derived tectonic refinement. A later implementation may therefore evaluate accepted coarse tectonic truth onto a finer geology/lithosphere topology without changing macro plate identity or rerandomizing upstream stages. Resolution changes must preserve upstream physical structure through explicit interpolation/refinement/provenance.

## Tectonic refinement is not topology refinement

WG-3.5 introduces microplates/terranes as **semantic/kinematic refinement on the existing requested topology**. A fragment receives a new kinematic-domain identity, but the canonical sample mesh is not recursively subdivided merely because a fragment exists.

```text
canonical topology samples
        ↓
WG-2 macro plate ownership
        ↓
WG-3.5 selected sample subsets
        ↓
terrane / microplate domains
```

This distinction prevents the number of tectonic domains from being confused with physical sampling density. Future higher-resolution structural passes can refine fragment boundaries using WG-1 ancestry while preserving their parent macro-plate and fragment provenance.

Before increasing canonical process levels, profiling must report generation time, memory, transfer size, and numerical quality. A finer grid is not useful if it only samples the same broad fields more densely; later terrain/lithology stages must introduce process-appropriate structural detail constrained by accepted coarse truth.

## WG-3.75 physical inheritance contract

WG-3.75 implements the previously documented coarse-to-fine checkpoint. Accepted upstream truth can now be inherited onto a finer canonical topology without rerunning or rerandomizing the coarse stages. This permits WG-4 terrain to begin at a finer process level while macro tectonic/crust/lithosphere identity remains anchored to its accepted coarse samples.
