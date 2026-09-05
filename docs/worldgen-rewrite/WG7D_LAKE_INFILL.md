# WG-7D — Lake sediment infill and basin-capacity evolution

WG-7D is the geomorphic mutation after WG-7B/WG-7C. It converts sediment already conserved into lake sinks into bounded physical lake-bed fill, then reconciles drainage and hydrology to the resulting surface without introducing a long year-stepped landscape simulation.

## Causal contract

Accepted ancestry:

1. WG-4 fixed ocean mask and sea level.
2. WG-5 accepted climate forcing.
3. WG-7A sediment transport capacity / forcing identity.
4. WG-7B evolved terrain, applied sediment source ledger, selected geomorphic horizon, and pre-infill lake sediment sink accounting.
5. WG-7C reconciled post-erosion lakes and seasonal hydrology.

WG-7D:

1. recovers deterministic lake-directed sediment delivery from the accepted WG-7B applied sediment routing;
2. converts delivered sediment mass over the accepted WG-7B geomorphic horizon to deposited volume with the accepted sediment density;
3. fills lake/depression accommodation from the lowest basin floor upward with a bounded direct solve;
4. creates a distinct post-infill evolved surface without rewriting WG-4 or WG-7B identities;
5. rebuilds drainage once on the post-infill surface;
6. reconciles runoff, lake equilibrium, and seasonal hydrology to that final surface using the same physical rules as WG-7C;
7. retains explicit mass/volume conservation and deterministic ancestry hashes.

## Browser / WASM contract

Protocol v18 exposes WG-7D as the canonical final physical state. Final drainage, runoff, lakes, seasonal hydrology, and the post-infill solid-elevation surface are sourced from the WG-7D reconciliation. WG-7C remains available as compact pre-infill ancestry and change diagnostics rather than retaining a second complete final seasonal-hydrology state in WASM memory.

The Lab exposes dedicated lake-infill diagnostics and renders the final physical-world relief from the post-infill surface. The WG-4 ocean mask/coastline remains fixed in WG-7D v1.

## Acceptance gates

WG-7D integration is expected to pass all of the following together:

- the complete Rust workspace test suite;
- the committed WASM rebuild and Rust/browser protocol-parity checks;
- the browser build and full TypeScript regression suite;
- `scripts/check-wg7d-lake-infill.sh`, including deterministic generation, sediment conservation, post-infill runoff/lake/seasonal closure, and ancestry checks.

## Non-goals for this PR

- no delta or offshore sediment construction;
- no coastline or sea-level migration;
- no floodplain/channel migration;
- no hillslope diffusion/mass wasting;
- no glacier flow/erosion;
- no weathering/regolith/soil;
- no ecology or gameplay geography;
- no global year-by-year geomorphic loop.

## Visualization work included with WG-7D

The Lab keeps the cumulative historical diagnostic surfaces while adding WG-7D final-state inspection:

- final physical-world base rendering from WG-7D post-infill relief;
- final river overlay from WG-7D reconciled drainage/realized discharge;
- final lake overlay;
- drainage-divide overlay;
- final topographic contours;
- snow/sea-ice overlay;
- dedicated post-infill elevation and lake-fill-depth diagnostics;
- physical-world, hydrologic-atlas, seasonal-world, and geomorphic-process presets.
