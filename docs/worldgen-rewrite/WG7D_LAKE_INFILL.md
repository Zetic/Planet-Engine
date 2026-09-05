# WG-7D — Lake sediment infill and basin-capacity evolution

WG-7D is the next geomorphic mutation after WG-7B/WG-7C. Its purpose is to convert sediment that is already conserved into lake sinks into bounded physical lake-bed fill, then reconcile drainage and hydrology to the resulting surface without introducing a long year-stepped landscape simulation.

## Intended causal contract

Accepted ancestry:

1. WG-4 fixed ocean mask and sea level.
2. WG-5 accepted climate forcing.
3. WG-7A sediment transport capacity / forcing identity.
4. WG-7B evolved terrain, applied sediment source ledger, selected geomorphic horizon, and pre-infill lake sediment sink accounting.
5. WG-7C reconciled post-erosion lakes and seasonal hydrology.

WG-7D will:

1. recover deterministic lake-directed sediment delivery from the accepted WG-7B applied sediment routing;
2. convert delivered sediment mass over the accepted WG-7B geomorphic horizon to deposited volume with the accepted sediment density;
3. fill lake/depression accommodation from the lowest basin floor upward with a bounded direct solve;
4. create a distinct post-infill evolved surface without rewriting WG-4 or WG-7B identities;
5. rebuild drainage once on the post-infill surface;
6. reconcile runoff, lake equilibrium, and seasonal hydrology to that final surface using the same physical rules as WG-7C;
7. retain explicit mass/volume conservation and deterministic ancestry hashes.

## Non-goals for this PR

- no delta or offshore sediment construction;
- no coastline or sea-level migration;
- no floodplain/channel migration;
- no hillslope diffusion/mass wasting;
- no glacier flow/erosion;
- no weathering/regolith/soil;
- no ecology or gameplay geography;
- no global year-by-year geomorphic loop.

## Visualization work included alongside WG-7D

The branch also adds Lab-only composite views that reuse the existing cumulative WG-7C result without changing the browser protocol:

- final physical-world base rendering from WG-7B evolved relief;
- final river overlay from reconciled drainage/realized discharge;
- final lake overlay;
- drainage-divide overlay;
- evolved topographic contours;
- snow/sea-ice overlay;
- physical-world, hydrologic-atlas, seasonal-world, and geomorphic-process presets.

These views are intended to make the cumulative procedural-generation result inspectable while WG-7D is developed.
