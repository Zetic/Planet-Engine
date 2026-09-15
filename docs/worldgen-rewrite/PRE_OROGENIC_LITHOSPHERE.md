# Pre-Orogenic Lithosphere

WG-3.5 now has an explicit **pre-orogenic** mechanical substrate for the tectonic-orogen rewrite. This stage exists to break the causal loop in the older lithosphere implementation, where present-day `orogenic_history`, rift/subduction response, and `crustal_strain` helped determine the mechanical fields that future mountain generation was expected to consume.

The new stage is intentionally upstream of present-day orogenic deformation:

```text
WG-1 topology
    ↓
WG-2 macro plates + rigid motion
    ↓
WG-2.5 connected boundary chronology
    ↓
WG-3 crustal assembly / crust age
    ↓
WG-3.5 PRE-OROGENIC LITHOSPHERE
    ↓
future tectonic orogen provinces
    ↓
post-deformation lithosphere / topography
```

## Causal cut

`generate_pre_orogenic_lithosphere` may use:

- macro-plate ownership and rigid plate motion;
- crust kind, crust-province identity, and crust age;
- accumulated divergent and transform boundary history;
- deterministic broad mechanical and mantle heterogeneity.

It deliberately does **not** use:

- `orogenic_history`;
- `subduction_history`;
- present `crustal_strain`;
- history-modified crust thickness, density, or buoyancy;
- cumulative convergent displacement as pre-existing damage.

Unit tests mutate those downstream/current-deformation fields and require the pre-orogenic identity to remain unchanged. Convergent chronology from WG-2.5 is reserved for the future orogen-province stage rather than feeding backward into substrate strength.

## Mechanical substrate

The stage generates dense fields for:

```text
intrinsicStrengthIndex
intrinsicWeaknessIndex
effectiveElasticThicknessKm
thermalStateIndex
provinceBoundaryIndex
ageDiscontinuityIndex
inheritedFabricStrength
inheritedStructureKind
inheritedRiftMemory
inheritedShearMemory
fragmentationPropensity
```

Strength is controlled by crust class, crust age, province-interior coherence, broad mechanical heterogeneity, thermal state, and inherited structural damage. The field is therefore allowed to vary substantially before any current collision is applied.

Thermal state uses broad mantle heterogeneity plus crust-age cooling and inherited extensional memory. It does not depend on current orogenic relief.

## Inherited structural fabric

The pre-orogenic model resolves several distinct structural causes:

- `PaleoSuture` — crust-province boundaries with strong age discontinuity;
- `InheritedRift` — accumulated divergent-system memory;
- `ShearZone` — accumulated transform-system memory;
- `ContinentalMargin` — crust-kind transitions;
- `CratonBoundary` — mechanically meaningful province boundaries inside continental crust.

These are not terrain masks. They are deformation controls for later province generation.

## Terranes and microplates move upstream

The stage constructs connected weak/fabric domains **before** present-day orogenic deformation. Accepted components become `Terrane` or `Microplate` fragments while retaining a single WG-2 parent macro plate. Microplates may receive a bounded deterministic perturbation to parent motion.

Unlike the previous downstream refinement, fragment eligibility no longer requires current `crustal_strain`, active subduction history, or current orogenic response.

The stage also emits a coarse fragment-contact graph. Contacts record:

- the two kinematic-domain IDs;
- number of topology edges in contact;
- physical angular contact length;
- mean intrinsic-strength contrast.

This graph is intended to let Tectonic Orogen Provinces react to accreted terranes, weak inherited contacts, and microplate boundaries instead of treating the macro boundary as the only meaningful structure.

## Radical-development policy

This stage is not calibrated to preserve the old WG-4/WG-5 morphology envelopes. During the tectonic rewrite, only correctness invariants are considered blocking: deterministic identity, finite/bounded state, sample alignment, valid parent-plate ownership, and valid fragment contacts.

The legacy `generate_lithosphere` path remains available temporarily so the existing full pipeline can continue to run while the new architecture is assembled. The future orogen-province PR should consume this pre-orogenic stage directly; once that cutover happens, downstream lithosphere/topography can be rebuilt around the new causal state rather than forcing this stage to mimic the old one.
