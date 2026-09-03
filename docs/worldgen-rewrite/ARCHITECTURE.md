# Planet Engine Architecture

## Ownership

```text
Legacy game path                         Planet Engine rewrite
────────────────                         ─────────────────────
TypeScript legacy worldgen               TypeScript diagnostic client
        ↓                                         ↓
Regions / Features                       dedicated Worldgen Worker
        ↓                                         ↓
existing game                            separate worldgen WASM package
                                                  ↓
                                         interlink-worldgen (Rust)
```

The realtime industrial runtime remains separate. World generation and realtime process simulation have different load timing, memory profiles, APIs, and lifecycle.

## Rust workspace

WG-0 introduces:

```text
rust/interlink-worldgen/       physical generation core
rust/interlink-worldgen-wasm/  browser/Worker boundary only
rust/interlink-worldgen-cli/   native diagnostics and profiling
```

`interlink-worldgen` owns deterministic generator contracts and dense physical fields/topology. It has no browser or gameplay dependency.

## Browser boundary

```text
src/worldgen/protocol.ts
src/worldgen/worldgenClient.ts
src/worldgen/worldgenWorker.ts
src/worldgen/diagnostics/
```

The Worker lazy-loads the separate generated worldgen WASM package and keeps numerical generation off the main thread. The client adds the current protocol version to the Worker URL, so static-host/browser caches cannot silently pair an old Worker with a new protocol or WASM package.

The protocol grows cumulatively by physical stage:

```text
WG-0     synthetic transport proof
WG-1     packed topology arrays
WG-2     packed macro-plate ownership / kinematics / boundaries
WG-3     dense crust fields / geological-history fields / boundary regimes / plate summaries
WG-3.5   lithospheric mechanics / structural fabric / mantle support / derived tectonic refinement
WG-3.75  coarse→fine provenance / inherited physical arrays / fine boundary-interface provenance / planet profile
```

Protocol v6 adds a dedicated inheritance request with independent coarse and fine topology levels. WG-2, WG-3, and WG-3.5 execute on the requested coarse topology only. Rust then produces the finer sample-aligned substrate and fine interface-aligned boundary arrays without rerunning accepted tectonic/geological stages at the finer level.

Topology uses packed positions, faces, CSR adjacency, finite-volume geometry, and refinement provenance. Later physical stages extend that model with sample-aligned typed arrays and boundary/plate/fragment-aligned arrays rather than serializing one JavaScript object per cell. WG-3.75 keeps this same SoA boundary: inherited fields, nearest-coarse provenance, exact-inherited masks, and fine boundary source indices are transferred as packed typed arrays.

The diagnostic lab is a consumer of those arrays. It does not own physical generation logic and changing globe/map projection does not change world truth.

## Upstream identity and derived refinement

The physical-stage dependency is directional:

```text
PlanetTopology
      ↓
WG-2 macro tectonics @ coarse physical resolution
      ↓
WG-3 crust / geological history
      ↓
WG-3.5 lithosphere / derived refinement
      ↓
WG-3.75 deterministic physical inheritance
      ├─ finer sample substrate
      └─ finer causal boundary interfaces
      ↓
WG-4 initial physical topography
```

WG-3.5 does not rewrite WG-2 macro plate IDs or WG-3 crust/history. A microplate or terrane is a downstream mechanically distinct domain with a parent macro plate. `kinematicDomainId` may therefore be finer than `plateId`, while `plateId` remains the stable upstream tectonic identity.

WG-3.75 likewise does not rerandomize or reinterpret accepted WG-2/WG-3/WG-3.5 truth. Stable WG-1 sample ancestry preserves the coarse sample prefix exactly; newly introduced fine samples receive deterministic provenance/interpolation. Fine inter-plate edges receive a deterministic source from the accepted coarse boundary network and carry its tectonic kind, geological regime, subduction polarity, and normal/shear rates. This gives WG-4 sharp causal interfaces without making terrain resolution authoritative over tectonic identity.

## Planet physical profile boundary

`PlanetPhysicalParameters` is a physical input contract rather than a terrain preset. Earth-like values remain the default accepted path. The contract now explicitly includes water inventory, ocean density, isostatic mantle density, internal heat flux, and mantle thermal expansivity alongside radius/gravity/rotation/orbital inputs.

The parameter identity participates in WG-3.75 inherited-state identity. A future dry, water-rich, lower-gravity, or otherwise altered rocky planet therefore cannot silently share downstream terrain identity with the Earth-like reference profile. WG-4 owns the actual isostatic elevation/bathymetry and sea-level solution.

## Generated WASM asset contract

WG-1 promotes the Planet Engine browser package from a transient development artifact to a committed static-hosting asset so `worldgen-lab.html` works from normal GitHub Pages after merge.

`npm run build:worldgen-wasm` writes the browser package to:

```text
src/wasm-worldgen/
```

The committed package is generated code, not hand-edited source. CI rebuilds it with the pinned `wasm-bindgen` version and byte-compares every committed output against the fresh package. TypeScript browser modules under `dist/` are likewise regenerated and checked for a clean diff. This keeps normal static hosting reproducible without moving physical generation authority out of Rust.

The production industrial runtime remains a separate WASM package under `src/wasm/`; the two packages have independent protocols and lifecycle boundaries.

## Topology algorithm boundary

Physical world-generation stages consume the narrow Rust `PlanetTopology` contract rather than icosphere construction details. The contract exposes unit positions, physical control-area weights, neighbors, center-to-center geodesic distances, and shared dual-interface lengths. This supports conservative finite-volume algorithms while allowing storage and refinement strategy to evolve internally.

WG-2, WG-3, and WG-3.5 consume this topology contract plus accepted upstream physical-stage outputs. Crustal state is not embedded in `PlanetTopology`; geological history does not alter WG-2 plate ownership or motion; lithospheric refinement does not alter either upstream identity. WG-3.75 consumes the concrete hierarchical topology provenance required to map accepted state between resolutions, but it still does not make topology construction depend on geology or gameplay objects.

## Rewrite isolation rule

No source under `src/worldgen/` or `rust/interlink-worldgen*` may depend on:

- legacy `Planet`;
- Region;
- GeographyPatch;
- resource Feature/node placement;
- MapSelection;
- NAV;
- gameplay Inspector;
- SVG geography.

This rule is regression-tested. Legacy Worldgen v7 remains the active gameplay path until a later explicit cutover stage.
