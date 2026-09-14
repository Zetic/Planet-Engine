# Final Worldgen Roadmap

This document defines the remaining Planet Engine work required to move from the current deterministic WG-7D physical-world pipeline to a mature, reviewable final planetary physical state suitable for later Project Interlink derivations.

## Current baseline

The production path is the deterministic L5→L8 pipeline, with L8 as the authoritative final physical topology. The engine already owns spherical topology, tectonics, geological/crustal state, lithospheric mechanics, multiresolution inheritance, physical topography and bathymetry, coupled climate, drainage, runoff, lake equilibrium and seasonal storage, fluvial erosion and sediment routing, bounded terrain evolution, post-erosion hydrology reconciliation, and lake sediment infill through WG-7D.

Recent viewer work is presentation-only and must remain separate from generator calibration. Palette, relief shading, and related visualization changes do not alter physical world state.

## Locked constraints

Remaining work must preserve deterministic generation, stable coarse-to-fine ancestry, conservative water/sediment accounting, explicit stage identities, the current Rust/WASM/Worker ownership boundary, and L8 as the authoritative scale for final drainage topology, depressions, lake-size distributions, coastlines, and final geomorphology.

Do not add gameplay Regions, resource nodes, Features, political geography, settlements, or other downstream Project Interlink systems until the physical generator is accepted.

## Recommended completion sequence

### WG-8A — continental macro-morphology

Fix the remaining large-scale continental-shape problem before adding more coastline detail. The current continental assembly can produce valid multi-plate continents, but visually it still tends toward rounded/blob-like unions with locally shredded borders.

The implementation should separate continental macro-shape from margin detail. Large continental bodies should be assembled from tectonically coherent large-scale structures first; margin roughness should then be applied at a subordinate scale rather than being allowed to dominate the silhouette.

Acceptance should add explicit morphology metrics that are currently missing: significant-component size hierarchy, coastline complexity by spatial scale, small-island area fraction, narrow-neck frequency, bay/peninsula hierarchy, hole/enclave frequency, perimeter-area behavior, and sensitivity to the underlying geodesic lattice. Existing multi-plate-continent and crust-fraction requirements remain in force.

### WG-8B — ocean-basin and ridge morphology

Remove residual geometric/chevron bathymetric structure that is not justified by tectonic state. Broad ridge elevation, transform segmentation, trench systems, thermal subsidence with oceanic age, and basin-scale mantle support are physically appropriate; large regular angular patterns inherited from graph distance, coarse boundary geometry, or the icosphere are not.

Add component diagnostics that expose oceanic crust age, thermal-subsidence contribution, ridge/rift relief, trench/arc contribution, mantle dynamic support, and final bathymetry side by side. Calibration should target plausible ridge-to-abyssal gradients and transform offsets without allowing the substrate geometry to become visible at planetary scale.

### WG-8C — final sea-level, shoreline, and hydrographic reconciliation

WG-7B/WG-7D mutate the solid surface after the original WG-4 water-volume/sea-level solve. Add a bounded final reconciliation that derives the final shoreline from the final solid terrain while preserving the engine's water-accounting semantics.

The reconciliation must distinguish globally connected ocean from disconnected below-sea-level basins, avoid turning every negative-elevation depression into ocean, and explicitly define how final ocean connectivity interacts with lakes and drainage. It must not introduce an open-ended terrain/climate iteration loop.

Acceptance should verify final ocean connectivity, shoreline consistency, global water-volume closure, stable lake/ocean classification, no invalid drainage receivers after shoreline changes, and deterministic identity.

### WG-8D — L8 depression, lake, and drainage calibration

Use the existing calibration packet and a fixed multi-seed L8 ensemble to tune the WG-6A/WG-6C/WG-7C/WG-7D basin chain after the final shoreline contract is settled.

Focus on depression count and area distributions, spill-depth distributions, endorheic fraction, lake area/depth/storage distributions, drainage-basin hierarchy, river-network continuity, and sensitivity to local L8 relief. Calibration should change causal parameters only when the ranked basin/depression diagnostics identify the responsible stage.

Do not tune by screenshots alone and do not mask topology problems with viewer changes.

### WG-8E — final climate and cryosphere calibration

After terrain, shoreline, and hydrology stabilize, run the L8 climate/cryosphere ensemble against the final surface. Verify latitudinal temperature structure, continentality, orographic precipitation, rain shadows, aridity patterns, seasonal snow/ice behavior, sea ice, and climate-water closure.

Any changes should remain physically parameterized and deterministic. Avoid climate changes that compensate for unresolved topography or hydrology defects.

### WG-9 — final physical-world acceptance

Freeze a canonical final-physical-world contract after the preceding stages pass. This stage should define the authoritative downstream fields, hashes/stage identities, required calibration summaries, performance envelope, and deterministic regression ensemble.

The acceptance suite should combine numerical invariants with morphology/distribution gates so a world cannot pass solely because it conserves mass while still exposing obvious generator artifacts.

Only after WG-9 should Project Interlink begin deriving lower-level gameplay geography such as Regions, Features, resources, traversal/clickability structures, and later human systems from the accepted physical planet.

## PR order

Keep the work narrow and reviewable:

1. Add WG-8A morphology observability and acceptance metrics before changing continental generation.
2. Tune continental macro-morphology against those metrics.
3. Add WG-8B ocean-component diagnostics and then remove nonphysical bathymetric geometry.
4. Implement WG-8C final shoreline/ocean reconciliation with conservation tests.
5. Run WG-8D L8 depression/lake/drainage calibration on the reconciled terrain.
6. Run WG-8E final climate/cryosphere calibration.
7. Add WG-9 final-world contract and freeze the deterministic regression ensemble.

Each implementation PR should preserve exact-head CI, report the causal fields changed, provide fixed-seed before/after diagnostics, and wait for explicit approval before merge.

## Immediate next PR

Start with WG-8A observability rather than directly injecting more margin noise. The first PR should add the missing continental/coastline morphology measurements and expose them in the calibration/acceptance path. That creates objective targets for the subsequent continental-shape tuning PR and prevents another visually motivated adjustment from simply trading blobs for shredded coastlines.
