# Continental tectonic morphology (WG-3.6 v4)

WG-3.6 v4 turns the existing tectonic-orogen province stage from a single-source boundary-distance raster into a distributed structural morphology model.

## Problem

WG-3.6 v3 already carried mature tectonic state: connected convergent systems, segmented provinces, lithospheric resistance, shortening, maturity, roots, plateaus, fold/thrust belts, arcs, foreland response, sutures, and transpression. The remaining geometric failure was downstream of that state. Every cell was effectively assigned to one winning convergent boundary source and most active structure was then evaluated as a one-dimensional profile away from that source.

That made the plate graph remain too legible in relief, encouraged source/Voronoi seams, and left stable continental interiors visually under-structured.

## v4 causal model

### Two-source deformation transfer

The distance solve now retains the two strongest reachable convergent sources on each plate instead of one hard owner. The secondary source participates when its deformation reach is comparable to the primary source. Overlap between different provinces creates a transfer/junction complex rather than an abrupt ownership seam.

The plate boundary remains the causal organizer, but it no longer uniquely determines the local structural axis.

### Structural range axes

Continental collision profiles now contain:

- a primary range axis displaced inland from the literal suture;
- a maturity- and inheritance-controlled secondary belt;
- an optional transfer-controlled tertiary belt;
- broad crustal-root support spanning the structural system;
- a fold/thrust front and foreland response outside the main range system.

The primary-axis offset is controlled by inherited weak corridors, paleo-sutures, rift memory, province boundaries, shear zones, and collision maturity. Strong inherited structure can therefore capture deformation and move high relief hundreds of kilometres away from the present plate edge.

Long connected systems are still preserved. An Andes-scale or Himalayan-scale system remains one coherent tectonic system even when local range axes step, branch, or migrate.

### Junction and transfer complexes

Where two convergent sources have comparable influence, their fields blend instead of switching discretely. Cross-province overlap broadens roots and fold/thrust deformation and adds limited junction uplift. This is intended to replace clean Y-shaped plate-graph mountain junctions with distributed transfer geometry.

### Continental interiors

Pre-orogenic lithosphere now contributes low-amplitude persistent relief outside active orogenic provinces:

- strong, thick lithosphere produces broad shield/cratonic support;
- inherited mobile belts retain modest root, fold, and upland expression;
- paleo-sutures remain weak structural lows/lineaments;
- shear/province-boundary corridors can preserve old tectonic grain;
- inherited rifts suppress shield uplift rather than being overwritten by it.

These effects use the same existing WG-3.6 fields consumed by WG-4, so they survive the normal causal inheritance path without a second topography system.

## What intentionally remains unchanged

WG-4 connected-ocean semantics from v14 remain unchanged. Closed continental depressions are not automatically flooded by the global ocean.

This PR also does not implement fluvial incision, drainage capture, sediment routing, or lake spill evolution. Those processes should act on the new tectonic surface in the subsequent surface-evolution work.

## Expected visual change

For the same seed, the expected differences are substantial:

- collision mountains should no longer trace every convergent edge as a narrow ribbon;
- mature collisions should commonly show multiple structural belts;
- range axes should step and migrate into inherited weak corridors;
- triple-junction geometry should be less visibly Y-shaped;
- continental interiors should contain broad shields and fossil structural grain instead of mostly procedural mottling;
- long cordilleras should remain possible and coherent rather than being broken into unrelated short ranges.

## Acceptance checks

The implementation adds unit coverage for deterministic stage output, preservation of the causal cut from legacy present-day orogenic fields, resistance-dependent deformation propagation, inherited-corridor axis migration, off-boundary mountain cores, persistent low-amplitude continental-interior structure, and valid province membership.

Visual acceptance should compare identical seeds before/after with plate-boundary overlays both on and off. The important criterion is not zero correlation with plate boundaries; tectonics must remain causal. The criterion is that high-relief skeletons are no longer near-direct copies of the boundary graph, especially for continental collisions and junctions.
