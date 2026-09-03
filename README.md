# Planet Engine

Planet Engine is the standalone deterministic planetary physical-world generator for Project Interlink.

The engine owns canonical spherical topology, macro tectonics, crust and geological history, lithospheric mechanics and tectonic refinement, multiresolution physical inheritance, initial physical topography/bathymetry, planetary physical profiles, native diagnostics, browser/WASM transport, and the standalone Planet Engine Lab.

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
WG-5 climate (next)
```

Gameplay Regions, Features, resource nodes, selection, factories, and the industrial runtime are intentionally outside this repository.

## Repository layout

- `rust/interlink-worldgen` — deterministic Rust physical-generation core.
- `rust/interlink-worldgen-wasm` — browser WASM bridge.
- `rust/interlink-worldgen-cli` — native diagnostics and benchmarks.
- `src/worldgen` — browser protocol, Worker/client transport, and diagnostics.
- `src/wasm-worldgen` — committed packaged Planet Engine WASM assets.
- `worldgen-lab.html` — standalone browser diagnostic lab.
- `docs/worldgen-rewrite` — architecture, determinism, resolution, geology, lithosphere, parameters, and validation contracts.

## Development

```bash
npm install
npm run build
npm run test:ts
cargo test --workspace
npm run worldgen:inheritance
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
