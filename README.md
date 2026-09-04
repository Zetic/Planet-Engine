# Planet Engine

Planet Engine is the standalone deterministic planetary physical-world generator for Project Interlink.

The engine owns canonical spherical topology, macro tectonics, crust and geological history, lithospheric mechanics and tectonic refinement, multiresolution physical inheritance, initial physical topography/bathymetry, coupled planetary climate, hydrology through equilibrium lakes and realized annual discharge, planetary physical profiles, native diagnostics, browser/WASM transport, and the single cumulative Planet Engine Lab.

## Current physical pipeline

```text
seed + engine/stage versions
          ↓
WG-1 hierarchical geodesic sphere
          ↓
WG-2 macro tectonics and rigid kinematics
          ↓
WG-3 crust + inferred geological history
          ↓
WG-3.5 lithosphere + structural/kinematic refinement
          ↓
WG-3.75 multiresolution physical inheritance
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
```

WG-5 derives deterministic seasonal insolation, temperature and pressure, rotation-sensitive prevailing winds, mass-projected wind-driven surface-ocean circulation, sea-surface temperature and conservative ocean heat transport, atmospheric moisture, precipitation, aridity, and snow/sea-ice potential from the accepted WG-4 physical planet. Orbital phases are generation-time climatology samples; the Lab season slider reconstructs stored seasonal harmonics and does not run a live climate simulation.

WG-6A derives deterministic terrain-following drainage receivers, contributing area, depression membership, and hydrologic escape geometry from WG-4 terrain. WG-6B combines WG-5 precipitation/PET with that topology to derive annual actual evapotranspiration, runoff, and potential discharge. WG-6C then solves generation-time equilibrium lake states for active depressions, retaining water in endorheic basins and releasing only solved overflow into a separate realized-discharge field. WG-6B potential discharge remains available as a diagnostic.

Gameplay Regions, Features, resource nodes, selection, factories, and the industrial runtime are intentionally outside this repository.

## Repository layout

- `rust/interlink-worldgen` — deterministic Rust physical-generation core.
- `rust/interlink-worldgen-wasm` — browser WASM bridge.
- `rust/interlink-worldgen-cli` — native diagnostics and benchmarks.
- `src/worldgen` — browser protocol, Worker/client transport, and diagnostics.
- `src/wasm-worldgen` — committed packaged Planet Engine WASM assets.
- `index.html` — single cumulative Planet Engine Lab entrypoint.
- `docs/worldgen-rewrite` — architecture, determinism, resolution, geology, lithosphere, topography, climate, hydrology, parameters, and validation contracts.

## Development

```bash
npm install
npm run build
npm run test:ts
cargo test --workspace
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
