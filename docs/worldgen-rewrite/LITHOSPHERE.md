# Lithosphere and Tectonic Refinement

WG-3.5 inserts a mechanical/structural lithosphere stage between accepted WG-3 crust/geological history and future WG-4 topography. The stage answers how inherited crust and tectonic history are mechanically expressed before elevation exists.

## Stage boundary

```text
WG-1 PlanetTopology
        +
WG-2 macro-plate ownership / rigid kinematics
        +
WG-3 crust / geological history
        ↓
lithospheric mechanical state
        ↓
inherited structural fabric
        ↓
lightweight mantle thermal/support state
        ↓
selective tectonic refinement
        ↓
WG-4 topographic inputs
```

WG-3.5 does **not** modify WG-2 macro-plate identity or WG-3 crust/history fields. It produces downstream state derived from them.

## Mechanical state

Every canonical sample receives dense physical/engineering-response fields:

```text
strengthIndex
weaknessIndex
effectiveElasticThicknessKm
thermalAnomalyIndex
mantleUpwellingIndex
mantleDynamicSupportIndex
compensatedBuoyancyIndex
structuralFabricStrength
structuralZoneKind
fragmentationPropensity
```

The first model is deliberately compact. Older, thicker, stable crust tends to be mechanically stronger. High strain, rifting, transform deformation, subduction influence, and positive thermal anomaly reduce effective strength. Effective elastic thickness is bounded and intended as a terrain-response control rather than a full viscoelastic lithosphere solver.

`compensatedBuoyancyIndex` combines WG-3 crustal buoyancy with WG-3.5 mantle dynamic support. WG-4 can therefore distinguish crustal-column support from deeper thermal/dynamic support without inventing a second tectonic system.

## Structural fabric

WG-3.5 converts strong inherited WG-3 histories into persistent structural categories:

```text
None
Suture
Rift
Transform
ContinentalMargin
```

These categories summarize the dominant inherited weak/structural zone at a sample. Continuous `structuralFabricStrength` remains the authoritative magnitude. WG-4 should use both to guide the width, orientation, localization, and mechanical response of later relief.

## Mantle support

WG-3.5 includes a smooth deterministic mantle thermal/support field modulated by accepted ridge, rift, and subduction context. It provides:

- broad thermal anomaly;
- mantle-upwelling tendency;
- signed dynamic support.

This is **not** a mantle-convection simulation and does not time-step mantle flow. It is a lightweight causal input for hotspot-like swells, anomalous uplift/support, rift/ridge thermal state, and future volcanism/topography.

## Selective tectonic refinement

WG-2 macro plates remain authoritative upstream identities. WG-3.5 may derive additional mechanically distinct domains only inside sufficiently weak, strained, structurally organized parts of a macro plate.

Two first-order fragment types are represented:

```text
Terrane
Microplate
```

A fragment:

- belongs to exactly one WG-2 parent macro plate;
- is a connected sample component;
- has a stable fragment ID and seed sample;
- stores physical area and parent-area fraction;
- stores mean weakness and fragmentation propensity;
- receives a refined kinematic-domain ID;
- either inherits parent motion (terrane) or receives a small deterministic perturbation to parent rigid motion (microplate).

The refinement is intentionally selective. It is not equivalent to increasing the global macro plate count or repartitioning the whole sphere into hundreds of Voronoi cells. Current validation caps total fragmented area at roughly 22% of the sphere and limits the number of derived fragments.

## Identity hierarchy

```text
WG-2 macro plate ID          upstream, unchanged
        ↓
WG-3.5 fragment ID           0 means no fragment
        ↓
WG-3.5 kinematic domain ID   macro interior or derived fragment domain
```

This hierarchy lets future terrain and boundary-local processes respond to microplates/terranes without changing the deterministic identity of accepted WG-2 tectonics.

## Determinism

WG-3.5 uses isolated namespaces:

```text
worldgen:lithosphere:state:v1
worldgen:lithosphere:mechanics:v1
worldgen:lithosphere:mantle-support:v1
worldgen:lithosphere:tectonic-refinement:v1
```

Changing later topographic algorithms must not move WG-3.5 structural/refinement state. Likewise, changing WG-3.5 must not alter WG-1 topology, WG-2 tectonic hashes, or WG-3 geology hashes for the same accepted upstream version.

## Validation

Acceptance includes:

- every dense field is sample-aligned and finite;
- normalized/signed fields remain within documented bounds;
- effective elastic thickness remains finite and bounded;
- stable old continental crust is stronger on average than highly strained crust in a multi-seed ensemble;
- every fragment remains inside exactly one parent macro plate;
- refined kinematic domains are deterministic;
- fragmentation remains selective rather than becoming a global tessellation;
- a multi-seed ensemble exercises actual microplate generation;
- same seed/version produces the same lithosphere hash;
- changed seed materially changes lithospheric identity.

## Explicit non-goals

WG-3.5 does **not** generate:

- elevation or bathymetry;
- mountains, plateaus, trenches, rift valleys, shelves, or basins as geometric terrain;
- full mantle convection;
- time-stepped plate fracture dynamics;
- detailed rock/lithology classification;
- climate or hydrology;
- erosion or sediment transport;
- glaciation;
- resource deposits;
- gameplay geography or gameplay cutover.

The next physical stage should be **WG-4: Initial Physical Topography**, using WG-3 crust/history plus WG-3.5 compensated buoyancy, elastic thickness, structural fabric, mantle support, and refined kinematic domains to derive the first elevation/bathymetry field.
