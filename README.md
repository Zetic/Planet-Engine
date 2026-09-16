# Planet Engine

Planet Engine is the standalone deterministic planetary physical-world generator for Project Interlink.

The engine owns canonical spherical topology, historical lithospheric material ancestry, modern macro tectonics, crust and geological history, lithospheric mechanics and tectonic refinement, multiresolution physical inheritance, initial physical topography/bathymetry, coupled planetary climate, hydrology through seasonal realized discharge and dynamic lake storage, diagnostic fluvial erosion, conservative sediment routing, bounded terrain evolution with post-erosion drainage reconciliation, bounded lake-sediment infill, planetary physical profiles, native diagnostics, browser/WASM transport, and the single cumulative Planet Engine Lab.

## Current physical pipeline

```text
seed + engine/stage versions
          ↓
WG-1 hierarchical geodesic sphere
          ↓
historical lithosphere: ancestral plates → persistent fragments → bounded epochs → modern ownership
          ↓
WG-2 modern macro tectonics and rigid kinematics
          ↓
WG-3 crust + geological history projected from persistent material ancestry
          ↓
WG-3.5 lithosphere + structural/kinematic refinement
          ↓
WG-3.75 multiresolution physical + material-identity inheritance
          ↓
WG-4 initial physical topography
          ↓
WG-5 coupled planetary climate
          ↓
WG-6A drainage topology and depressions
          ↓
WG-6B annual runoff and potential discharge
          ↓
WG-6C equilibrium lakes and realized discharge
          ↓
WG-6D seasonal hydrology
          ↓
WG-7A fluvial erosion and sediment diagnostics
          ↓
WG-7B bounded terrain evolution + rebuilt drainage
          ↓
WG-7C post-erosion hydrology reconciliation
          ↓
WG-7D bounded lake-sediment infill + final hydrology rebuild
```

The historical-material frontend is causally upstream of modern WG-2/WG-3 authority. It first creates an ancestral plate partition and persistent crust fragments, assigns plate/fragment-owned continental, transitional, and oceanic material, derives oceanic birth age from ancestral spreading geometry, and records sparse tectonic events. A bounded coarse epoch pass creates explicit parent/child fragment lineage without retaining per-epoch dense world state, after which ancestral domains are consolidated into the modern ownership partition. Public WG-2 is the modern kinematic projection of that history; public WG-3 consumes the same material ancestry rather than regenerating crust from a global affinity field. WG-3.75 carries the accepted origin → fragment → current-owner identity chain to the fine topology. `index.html` exposes those identities together with WG-3 crust provenance, fossil internal discontinuities, and oceanic birth-age span.

WG-5 derives deterministic seasonal insolation, temperature and pressure, rotation-sensitive prevailing winds, mass-projected wind-driven surface-ocean circulation, sea-surface temperature and conservative ocean heat transport, atmospheric moisture, precipitation, aridity, and snow/sea-ice potential from the accepted WG-4 physical planet. Orbital phases are generation-time climatology samples; the Lab season slider reconstructs stored seasonal harmonics and does not run a live climate simulation.

WG-6A derives deterministic terrain-following drainage receivers, contributing area, depression membership, and hydrologic escape geometry from WG-4 terrain. WG-6B combines WG-5 precipitation/PET with that topology to derive annual actual evapotranspiration, runoff, and potential discharge. WG-6C solves generation-time equilibrium lake states for active depressions, retaining water in endorheic basins and releasing only solved overflow into a separate realized-discharge field. WG-6D redistributes the accepted WG-6B annual runoff across retained WG-5 orbital phases, carries snow accumulation and degree-day melt timing, routes phase potential discharge over the accepted WG-6A DAG, advances WG-6C lake control volumes through the seasonal cycle, and classifies realized flow as dry, intermittent, or perennial. WG-6B potential and WG-6C annual realized discharge remain available as diagnostics beside the seasonal fields.

WG-7A consumes the accepted WG-6D phase realized-discharge field, immutable WG-4 terrain, WG-6A receiver topology, WG-6C lake control volumes, and inherited lithospheric mechanics to derive peak-sensitive effective discharge, channel slope and hydraulic width, erodibility, bounded incision potential, sediment production, transport capacity, routed load, and deposition. Active lake depressions are complete first-pass sediment traps and generated sediment is conserved into land, lake, and terminal/ocean deposition. WG-7A remains the immutable forcing/sediment foundation consumed by WG-7B.

WG-7B applies WG-7A incision over a bounded channel/valley footprint to a distinct evolved terrain state, chooses one adaptive direct geomorphic horizon capped by resolved elevation change, recomputes sediment from the actually applied erosion volume, and conserves that mass into ordinary land deposition plus lake and terminal/ocean sinks. It then rebuilds WG-6A drainage exactly once on the evolved surface and reroutes accepted WG-6B local runoff over that new DAG. WG-4 identity and its ocean mask remain unchanged; WG-5 climate, WG-6C lake equilibrium, and WG-6D seasonal hydrology are not iterated in WG-7B v1.

WG-7C closes that deliberate one-stage feedback gap without introducing a climate/terrain iteration loop. It keeps WG-4 sea level/ocean mask and WG-5 climate immutable, rebinds the exact accepted WG-6B local runoff field to the WG-7B drainage graph, recomputes WG-6C lake equilibrium on evolved solid elevation, and reruns WG-6D seasonal lake/realized-flow dynamics. WG-7D then applies bounded historical lake-sediment infill to a distinct final surface, rebuilds drainage once more, and reconciles runoff/lakes/seasonal hydrology again without rerunning WG-5 climate. Browser/WASM transport exposes the WG-7D drainage/runoff/lake/seasonal state as final hydrology while retaining compact WG-7C ancestry/change diagnostics rather than a second complete seasonal state.

Gameplay Regions, Features, resource nodes, selection, factories, and the industrial runtime are intentionally outside this repository.

## Repository layout

- `rust/interlink-worldgen` — deterministic Rust physical-generation core.
- `rust/interlink-worldgen-wasm` — browser WASM bridge.
- `rust/interlink-worldgen-cli` — native diagnostics and benchmarks.
- `src/worldgen` — browser protocol, Worker/client transport, and diagnostics.
- `src/wasm-worldgen` — committed packaged Planet Engine WASM assets.
- `index.html` — single cumulative Planet Engine Lab entrypoint.
- `docs/worldgen-rewrite` — architecture, determinism, resolution, geology, lithosphere, topography, climate, hydrology, erosion/sediment, parameters, and validation contracts.

## Development

```bash
npm install
npm run build
npm run test:ts
cargo test --workspace
bash scripts/check-wg7a-erosion.sh
bash scripts/check-wg7b-evolution.sh
bash scripts/check-wg7c-reconciliation.sh
npm run worldgen:inheritance
npm run worldgen:topography
npm run worldgen:climate
npm run worldgen:profile
```

Browser WASM packaging requires `wasm-bindgen-cli 0.2.127`:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.127 --locked
npm run build:worldgen-wasm
```

## Extraction provenance

Initial standalone foundation extracted from `Zetic/Project-Interlink` commit `4cd8d727b5d437396a0da0fe652b64685a3bc309`, the completed WG-3.75 PR #104 head. Future Planet Engine physics should be developed here and consumed by Project Interlink through an explicit versioned engine boundary.
