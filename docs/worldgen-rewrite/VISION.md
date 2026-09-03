# Planet Engine Rewrite Vision

## Mission

The Planet Engine generates deterministic, physically causal, multiresolution planetary physical truth. Gameplay geography, deposits, Regions, Features, surveying state, and player-facing representations are downstream derivations.

The target dependency chain is:

```text
planetary parameters
  → spherical topology
  → tectonics
  → geological history
  → crustal state
  → initial topography
  → lithology / rock properties
  → climate circulation
  → precipitation
  → hydrology
  → erosion + sediment transport
  → glaciation / coastal evolution
  → mature terrain
  → resource geology / deposit genesis
  → derived geography
  → gameplay abstractions
```

## Legacy policy

Legacy Worldgen v7 remains the active gameplay path during the rewrite. It is kept deterministic, runnable, and available for comparison, but is not incrementally transformed into the new Planet Engine.

The new engine must not depend on legacy `Planet`, `Region`, GeographyPatch, resource Feature, NAV, Inspector, or SVG map contracts.

## Rewrite principle

World generation is treated as an independent physical-model subsystem. The Planet Engine must be testable and useful without a Region, Feature marker, border, NAV entry, or gameplay Inspector.

WG-0 deliberately produces no terrain. Its synthetic field is an architectural proof that the independent Rust → WASM → Worker → diagnostics route works before physical algorithms are added.
