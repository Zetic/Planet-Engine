# Continental morphology observability

## Purpose

This document defines the morphology diagnostics used to evaluate continental assembly before and after changing WG-3 generation. The diagnostics themselves are observational only: they do not alter crust type, plate identity, terrain, sea level, climate, hydrology, erosion, or any later physical state.

The objective is to make known continental-shape failure modes measurable across fixed seeds so WG-3 changes can be judged against stable evidence rather than screenshots alone.

## Authoritative subject

The first morphology packet measures the **continental-crust mask** (`CrustKind::Continental`) on the canonical geodesic topology. This is intentionally distinct from emergent land. Continental crust may be submerged, and emergent land may be fragmented by topography and sea level. A later shoreline-quality packet may apply the generic mask analyzer to final land/ocean state without changing the meaning of the crust metrics defined here.

The default significant-component threshold remains 0.25% of planetary area, matching the existing WG-3 continental-assembly acceptance. A major multi-plate component remains a component covering at least 1.5% of planetary area and spanning at least two plate IDs.

## Metrics

`ContinentalMorphologySummary` reports the following deterministic quantities.

### Component hierarchy

- `all_component_count`: every connected continental-crust component, including tiny fragments.
- `significant_component_count`: components at or above the 0.25% area threshold.
- `component_area_coefficient_of_variation`: size dispersion across significant components.
- `largest_to_median_area_ratio`: hierarchy between the largest and median significant components.
- `largest_component_plate_count`: number of tectonic plates represented in the largest significant component.
- `has_major_multiplate_component`: whether any major component crosses plate domains.

These retain the existing WG-3 diversity concepts while moving their calculation into reusable library code.

### Shape

- `maximum_elongation`: maximum significant-component diameter divided by the diameter of an equal-area circular cap approximation.
- `maximum_compactness`: maximum `P² / (4πA)` across significant components using geodesic boundary length and dual-cell area. A perfect planar circle is 1; larger values indicate a less compact outline. On the sphere and discrete mesh this is a comparative diagnostic rather than a literal planar shape score.
- per-component `constricted_sample_fraction`: fraction of component samples with no more than two same-mask graph neighbors.
- global `constricted_sample_fraction`: the corresponding fraction over all significant-component samples.

The constriction metric is a graph-resolution proxy for thin one-cell necks, tips, and shredded fragments. It is not treated as a direct geological quantity.

### Satellite fragmentation

- `satellite_component_count`: components below the significant-area threshold.
- `satellite_area_fraction`: total planetary area occupied by those small components.

This separates a few small terranes/islands from a coastline that has fragmented into many tiny crust components.

### Complement fragmentation

- `secondary_complement_component_count`: connected non-mask regions after excluding the largest complement region.
- `secondary_complement_area_fraction`: their total planetary area fraction.

For continental crust this is an enclosed/partitioned-complement proxy. It may represent enclosed non-continental pockets or crust geometry that partitions the complement. It is diagnostic only; it must not be interpreted automatically as lakes or inland seas.

### Multi-scale coastline complexity

The analyzer reports area-weighted compactness for significant components at three observation scales:

- `coastline_complexity_fine`: the native mask.
- `coastline_complexity_medium`: after deterministic local majority smoothing.
- `coastline_complexity_coarse`: after a larger number of the same smoothing rounds.

The smoothing exists only inside diagnostics. It never feeds back into generation. The number of smoothing rounds scales with topology level and is emitted in the summary (`medium_smoothing_rounds`, `coarse_smoothing_rounds`) so reports remain interpretable.

The relationship between fine, medium, and coarse complexity is intended to distinguish large-scale continental form from graph-scale edge shredding. No assumption is made that complexity must decrease monotonically for every possible mask.

## Determinism and complexity

All analysis operates on canonical topology arrays and deterministic graph traversal. No random values are sampled. Connected-component anchors are the lowest sample encountered by deterministic index-order traversal, and ranked components are sorted by area.

Runtime is linear in samples and neighbor edges for each analysis pass. Multi-scale complexity performs a bounded number of whole-mask majority passes. At higher topology levels the smoothing count is capped through the level-scaling rule so diagnostics remain practical for calibration runs.

## Acceptance policy

The longstanding WG-3 acceptance thresholds remain authoritative:

- at least two significant continental components per accepted seed;
- material component-size hierarchy in the ensemble;
- material elongation in the ensemble;
- non-circular outlines in the ensemble;
- major multi-plate continental assembly in the ensemble;
- sufficient ensemble component-area coefficient of variation;
- tectonic layout must materially affect crust partition.

PR #47 established the fixed-seed v2 observability baseline without tuning the generator to the new measurements. WG-3 v3 then changes continental assembly and promotes two of those defect measurements to ensemble acceptance gates: mean satellite continental area must remain at or below 0.18% of planetary area, and mean graph-scale constricted-sample fraction must remain at or below 1.75%. These limits sit between the v2 baseline and the v3 implementation result so a regression toward the measured shredded-margin state fails CI while individual worlds can still retain legitimate terranes and narrow features.

Multi-scale complexity remains primarily comparative because minimizing it blindly would reward featureless ellipses. WG-3 v3 therefore uses only a coarse-complexity floor of 6.0 across the fixed-seed ensemble to prevent the macro field from collapsing toward overly simple shapes. Fine and medium complexity continue to be printed for before/after diagnosis rather than optimized as absolute targets.

## Synthetic regression coverage

Library tests use deterministic icosphere masks to verify that:

- identical masks produce identical summaries;
- a tiny remote component is classified as a satellite;
- an elongated band scores above a compact cap on elongation;
- topology/mask length mismatches fail explicitly.

WG-3 v3 additionally tests that the broad assembly-corridor influence is local and tapered and that short-scale margin detail is exactly zero away from the provisional macro thresholds. These directional tests protect the architecture behind the new ensemble gates rather than only protecting the current numeric output.

## Relationship to later work

WG-3 v3 uses this packet to verify reduced graph-scale fragmentation and constriction while retaining component-size diversity, multi-plate assembly, elongation, and nontrivial coarse structure. It does not attempt to generate final emergent-land coastline quality.

Final emergent-land coastline quality remains a separate concern because it also depends on topography, sea level, erosion, sedimentation, and later shoreline reconciliation.
