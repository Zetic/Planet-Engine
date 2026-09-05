# Worldgen Rewrite

This directory defines the greenfield Planet Engine rewrite that runs in parallel with Legacy Worldgen v7.

- [`VISION.md`](VISION.md) — mission and legacy-worldgen policy.
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — Rust/WASM/Worker ownership and isolation.
- [`TOPOLOGY.md`](TOPOLOGY.md) — canonical hierarchical geodesic sphere and finite-volume geometry.
- [`TECTONICS.md`](TECTONICS.md) — deterministic spherical plate partitioning and rigid plate kinematics.
- [`GEOLOGY.md`](GEOLOGY.md) — dense crustal state, geological boundary regimes, and inferred geological memory.
- [`LITHOSPHERE.md`](LITHOSPHERE.md) — mechanical lithosphere, structural fabric, mantle support, terranes, and microplates.
- [`MULTIRESOLUTION.md`](MULTIRESOLUTION.md) — deterministic coarse-to-fine physical inheritance and boundary provenance.
- [`PLANET_PARAMETERS.md`](PLANET_PARAMETERS.md) — Earth-like default physical profile and future rocky-planet parameter contract.
- [`RESOLUTION.md`](RESOLUTION.md) — process-specific multiresolution policy.
- [`DETERMINISM.md`](DETERMINISM.md) — generator/stage identity and random-stream isolation.
- [`VALIDATION.md`](VALIDATION.md) — numerical, topology, and physical acceptance gates.
- [`WG7_EROSION.md`](WG7_EROSION.md) — WG-7A fluvial forcing/sediment conservation plus WG-7B bounded terrain evolution, applied sediment mass, and post-erosion drainage rebuild.

The complete rewrite blueprint from planning is reflected across these reviewable architecture documents.

## WG-0 — isolated engine foundation

WG-0 establishes the separate Rust worldgen core, independent WASM package, native CLI, dedicated browser Worker/protocol, and diagnostic lab. Its deterministic synthetic field is retained only as an end-to-end transport/determinism regression. Legacy Worldgen v7 remains the active gameplay path.

## WG-1 — canonical spherical topology

WG-1 introduces the first production planetary state:

- deterministic hierarchical icosahedral samples;
- stable refinement identity/provenance;
- dense 5/6-neighbor topology;
- spherical circumcentric dual-cell areas;
- center-to-center geodesic distances;
- shared dual-interface lengths for conservative finite-volume fluxes;
- latitude/longitude, great-circle, Cartesian-anchor, and local ENU coordinate contracts;
- native/WASM/browser topology diagnostics;
- committed worldgen WASM assets so the standalone lab is compatible with static GitHub Pages after merge.

WG-1 generates no tectonics or downstream geography. Its topology is the fixed physical substrate consumed by later stages.

## WG-2 — spherical plate tectonics

WG-2 adds deterministic plate-scale physical truth without yet creating crust or terrain:

- seeded stochastic minimum-separation plate origins on the canonical sphere, permitting large and small macro plates rather than forcing near-equal territories;
- connected graph-Voronoi plate ownership using canonical geodesic edge distances;
- one rigid Euler pole/angular velocity per plate;
- plate area accounting from canonical dual-cell areas;
- boundary extraction from inter-plate neighbor edges;
- relative normal/shear kinematics and convergent/divergent/transform classification;
- deterministic tectonic identity independent from later geology;
- native, WASM, Worker, and browser plate diagnostics.

WG-2 intentionally stops before crustal state or terrain. Those derive from accepted plate truth rather than being embedded into the plate partition algorithm.

## WG-3 — crustal state and geological history

WG-3 introduces dense geological state on the same canonical topology:

- coherent continental, transitional, and oceanic crust independent from current plate borders;
- physical crust age, thickness, density, and relative buoyancy;
- oceanic crust age inferred from spreading-system distance and rate;
- convergent boundaries resolved into oceanic subduction, ocean-continent subduction, or continental collision;
- divergent boundaries resolved into oceanic ridges, continental rifts, or transitional divergence;
- deterministic subduction polarity from local crustal state and buoyancy;
- propagated orogenic, rift, ridge, subduction, trench, volcanic-arc, transform, subsidence, basin, and strain history fields;
- derived plate summaries, including area-based major/intermediate/minor classes and mixed crust fractions;
- native, WASM, Worker, and browser crust/history diagnostics.

WG-3 still generates no elevation or bathymetry. Its physical crust/history output is intended to constrain WG-4 initial terrain through isostasy and tectonic structure rather than direct terrain noise.

## WG-3.5 — lithospheric mechanics and selective tectonic refinement

WG-3.5 derives the mechanical substrate that WG-4 will use for terrain response:

- strength, weakness, and effective elastic thickness;
- thermal anomaly, mantle upwelling, and mantle dynamic support;
- compensated buoyancy and inherited structural fabric;
- structural-zone classification from accepted geological history;
- selective mechanically plausible terranes and microplates;
- refined kinematic-domain identities while WG-2 macro plates remain authoritative upstream truth.

WG-3.5 does not globally increase macro-plate count and still does not generate terrain.

## WG-3.75 — multiresolution physical inheritance

WG-3.75 breaks the temporary same-resolution coupling before terrain generation:

- WG-2/WG-3/WG-3.5 run on an accepted coarse topology;
- stable WG-1 sample ancestry preserves every inherited coarse sample exactly;
- continuous fields interpolate hierarchically, with categorical-domain constraints where discontinuities matter;
- categorical fields use deterministic geodesic nearest-source provenance;
- fine plate-boundary interfaces retain tectonic kind, geological regime, subduction polarity, normal/shear rates, and coarse-source identity;
- provenance, physical parameters, upstream stage hashes, inherited fields, and reconstructed boundaries have explicit deterministic identities;
- WASM protocol v7 transports the finer physical substrate to the standalone Planet Engine Lab;
- an Earth-like physical profile remains the default while water inventory and interior/isostatic parameters are explicit for future rocky planets.

WG-3.75 is still foundation work. It generates no elevation, bathymetry, sea-level solution, climate, hydrology, erosion, resources, Regions, Features, or gameplay cutover. WG-4 consumes this accepted finer substrate instead of rerunning tectonics/geology at terrain resolution.

## WG-4

WG-4 derives initial solid elevation, tectonic bathymetry, a deterministic vertical datum, and water-volume-conserving sea level from accepted WG-3.75 truth. See `TOPOGRAPHY.md`.

## WG-7A — fluvial erosion and sediment foundation

WG-7A is the first geomorphology stage after accepted hydrology. It derives peak-sensitive effective discharge from WG-6D seasonal realized flow, receiver slope and hydraulic channel width from accepted terrain/drainage geometry, lithosphere-derived erodibility, bounded diagnostic incision potential, and conservative sediment production/transport/deposition.

WG-7A deliberately leaves WG-4 terrain and WG-6 drainage/hydrology unchanged. Active WG-6C lake depressions are complete first-pass sediment traps, and global generated sediment must close into land, lake, and terminal/ocean deposition.

## WG-7B — bounded terrain evolution and drainage rebuild

WG-7B consumes the accepted WG-7A forcing as immutable input, converts channel incision into bounded cell-average valley erosion, chooses one adaptive direct geomorphic horizon, applies ordinary land deposition from a conservative sediment ledger, creates a distinct evolved solid surface, rebuilds drainage exactly once, and reroutes accepted WG-6B local runoff over the rebuilt DAG. The WG-4 ocean mask remains fixed and WG-5/WG-6 seasonal physics are not rerun inside the stage. See [`WG7_EROSION.md`](WG7_EROSION.md) for the complete causality, invariants, browser contract, benchmarks, and deferred feedback processes.