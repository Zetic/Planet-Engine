# Continental morphology observability

## Purpose

This document defines the morphology diagnostics used to evaluate continental assembly before changing WG-3 generation. The diagnostics are observational only: they do not alter crust type, plate identity, terrain, sea level, climate, hydrology, erosion, or any later physical state.

The objective is to make known continental-shape failure modes measurable across fixed seeds so later WG-3 changes can be judged against stable evidence rather than screenshots alone.

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

The constriction metric is a graph-resolution proxy for thin one-cell necks, tips, and shredded fragments. It is not treated as a direct geological quantity and is not currently a hard quality gate.

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

The relationship between fine, medium, and coarse complexity is intended to distinguish large-scale continental form from graph-scale edge shredding. No assumption is made that complexity must decrease monotonically for every possible mask, so the initial observability PR records these values without imposing subjective thresholds.

## Determinism and complexity

All analysis operates on canonical topology arrays and deterministic graph traversal. No random values are sampled. Connected-component anchors are the lowest sample encountered by deterministic index-order traversal, and ranked components are sorted by area.

Runtime is linear in samples and neighbor edges for each analysis pass. Multi-scale complexity performs a bounded number of whole-mask majority passes. At higher topology levels the smoothing count is capped through the level-scaling rule so diagnostics remain practical for calibration runs.

## Acceptance policy

The existing WG-3 acceptance thresholds remain authoritative in this packet:

- at least two significant continental components per accepted seed;
- material component-size hierarchy in the ensemble;
- material elongation in the ensemble;
- non-circular outlines in the ensemble;
- major multi-plate continental assembly in the ensemble;
- sufficient ensemble component-area coefficient of variation;
- tectonic layout must materially affect crust partition.

The new satellite, constriction, complement-fragmentation, and multi-scale complexity metrics are **observability metrics first**. The acceptance example validates that they are finite and bounded where appropriate, prints a fixed-seed baseline, and does not introduce tuned morphology targets until a subsequent generation-changing PR has evidence for useful ranges.

This avoids locking the current defects into acceptance merely because they are the present baseline.

## Synthetic regression coverage

Library tests use deterministic icosphere masks to verify that:

- identical masks produce identical summaries;
- a tiny remote component is classified as a satellite;
- an elongated band scores above a compact cap on elongation;
- topology/mask length mismatches fail explicitly.

Additional fixtures should be added when later PRs introduce a metric that becomes a hard gate. A metric should not become an optimization target without a regression proving the intended directional response.

## Relationship to later work

This packet deliberately does not alter WG-3. The intended use is to establish a stable baseline before continental macro-shape generation is reworked. Later generation changes should compare fixed-seed reports for:

- reduced graph-scale fragmentation without collapsing legitimate islands/terranes;
- stronger large-scale shape hierarchy;
- lower dependence on one-cell constrictions;
- retained component-size diversity and multi-plate assembly;
- coastline complexity that is expressed at coarse and medium scales rather than almost entirely at the native mesh scale.

Final emergent-land coastline quality is a separate concern because it also depends on topography, sea level, erosion, sedimentation, and later shoreline reconciliation.
