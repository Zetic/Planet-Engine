# Spherical Plate Tectonics

Planet Engine separates **ancestral material geometry** from **present-day plate ownership**. The canonical historical frontend still uses the deterministic spherical WG-2 partition as a deep-time initializer, but modern plate faces are synthesized from a separate continuous spherical boundary field. Present boundaries can therefore migrate through ancestral material while persistent material ancestry survives underneath.

This is a deterministic tectonic synthesis model, not a mantle-convection solver or a literal geological reconstruction.

## Canonical causal chain

```text
canonical PlanetTopology
        +
planet radius + seed + requested modern plate count
        ↓
ancestral spherical plate partition
        ↓
plate-owned crust + coherent proto-continental nuclei
        ↓
persistent fragments / rift lineage
        ↓
provisional modern material grouping
        ↓
boundary-first spherical plate-field synthesis
        ↓
fragment capture / transfer provenance
        ↓
modern rigid Euler-pole kinematics
        ↓
modern boundary extraction + relative motion
        ↓
historical morphology / lithosphere / topography
```

Three identities remain distinct on every coarse physical sample:

```text
origin_plate_id   ancestral material provenance
fragment_id       persistent crust / terrane continuity
current_plate_id  present kinematic owner
```

A modern plate boundary may therefore cut through one ancestral plate. When modern ownership separates material that previously shared a fragment, the material is partitioned into child fragments and capture/transfer provenance is recorded rather than rewriting ancestral identity.

## Ancestral plate initializer

`tectonics.rs::generate_tectonics` remains the deterministic spherical plate initializer. It chooses plate seed samples with a seeded minimum-separation process and assigns ownership using multi-source shortest-path distance over the canonical topology graph. Each ancestral plate receives a rigid Euler pole and angular velocity.

This graph-Voronoi construction is useful for coherent deep-time domains, but it is not treated as final present-day geometry in the canonical historical pipeline.

Acceptance of the ancestral initializer still requires valid, connected, non-empty plates, exact area closure, deterministic seed spacing, and a non-degenerate distribution of plate areas.

## Coherent proto-continental material

Continental material is not initialized by independently selecting scattered ancestral carrier plates and attempting to weld them afterward. The historical lithosphere chooses a small set of separated proto-continental nuclei and grows each assembly contiguously across the ancestral adjacency graph until the global material target is reached.

Growth favors substantial shared contact and convergent relationships, tolerates some transform attachment, penalizes divergent attachment, and limits one assembly from consuming the whole target. Transitional margins are then derived from the edge of the resulting assembled continental material.

This makes isolated continental islands a later historical outcome—rift fragments, captured terranes, or microcontinents—rather than a default artifact of random carrier selection.

## Boundary-first modern plate synthesis

The present-day plate stage no longer grows ownership one sample at a time from local territorial rules. Instead it constructs a small set of continuous spherical plate fields and rasterizes their faces onto the canonical coarse topology.

The synthesis proceeds deterministically:

- choose globally separated modern plate cores on the sphere;
- inherit an area-weighted angular velocity for each field from the provisional material grouping under its core;
- evolve those cores through a bounded sequence of rigid Euler rotations while preventing core collapse;
- apply a coherent low-frequency spherical warp shared by all fields;
- assign each sample to the strongest spherical field, with bounded area bias and a far-span penalty;
- keep one final core per field and repair any disconnected raster remnants without changing material ancestry.

Because the field competition is evaluated independently of `origin_plate_id`, the resulting `current_plate_id` network can cross old plate interiors instead of tracing the ancestral graph-Voronoi tessellation. The provisional grouping still matters physically: it supplies inherited motion to the new plate fields and provides the material that is subsequently captured or split beneath the new boundaries.

This is a geometry-first cut. It deliberately avoids allowing a local growth heuristic, continental carrier layout, or ancestral cell adjacency to dictate the final outline of a modern plate.

## Modern rigid kinematics

After boundary-first ownership is established, modern angular velocity is reconstructed from the ancestral material currently carried by each modern plate, weighted by the physical area of that contribution. A representative interior seed is chosen from the final modern domain.

At unit surface direction `r`, rigid plate velocity remains:

```text
v = ω × r × R
```

where `R` is planet radius. Interior deformation is represented by persistent fragments, inherited structures, and later lithospheric fields rather than by violating the rigid modern-plate velocity contract.

## Boundary kinematics

A modern boundary edge is any canonical neighbor edge whose samples have different `current_plate_id` values. Relative rigid motion is decomposed into normal and shear components. The diagnostic classification remains:

```text
normal contribution < 35% of relative speed  → transform / shear-dominated
otherwise normal rate < 0                    → convergent
otherwise                                    → divergent
```

Geological interpretation—ridge, rift, subduction polarity, continental collision, arc, passive margin, fossil suture—is assigned by the historical geology and morphology stages using material type and event provenance.

## Historical event relationship

The bounded historical lineage pass and boundary-first modern geometry are intentionally sparse. They do not retain a complete dense raster for every geological epoch. Persistent outputs are material identities, fragment lineage, ownership transfers, ages, and event records; downstream morphology rasterizes those causes into sutures, rifts, passive margins, active orogens, and fossil structures.

The model therefore aims for the causal qualities observed in the project’s Gleba reference sequence—old plates, denser material fragments, fewer broad present plates, then crust verification—without claiming source-code equivalence or a full physical mantle simulation.

## Determinism

Important namespaces include:

```text
worldgen:tectonics:plates:v1
worldgen:geology:historical-lithosphere:ancestral:v1
worldgen:geology:historical-lithosphere:epochs:v1
worldgen:geology:dynamic-modern-plates:v2
worldgen:geology:boundary-first-plates:v1
worldgen:geology:historical-lithosphere:modern-tectonics:v1
```

The historical identity hash covers boundary-first current ownership and persistent fragment lineage. The modern tectonic hash covers modern plate motion, current ownership, extracted boundary kinematics, and the upstream history hash.

## Blocking geometry acceptance

Permanent validation checks structural properties that ancestry-locked or local-growth systems could pass while still producing implausible present-day geometry:

- a large fraction of present boundary edges must migrate away from ancestral plate boundaries;
- multiple ancestral plates must be split across modern owners with explicit material lineage;
- every modern plate must remain connected and non-empty;
- plate areas must stay bounded away from both collapse and planetary dominance;
- spherical plate span must stay bounded so no face wraps around the globe;
- perimeter/area compactness must remain bounded for substantial plate faces;
- no plate may place an excessive share of its boundary against a single neighbor;
- narrow-neck incidence must remain limited rather than producing long enclosure-prone tendrils;
- continental material must not regress to a large population of tiny satellite components;
- historical lithosphere, morphology, WG-4 topography, browser diagnostics, and packaged WASM remain compatible.

The WG-4 compatibility gate also requires active continental collision and accretion belts to retain sufficient crustal support after the boundary-network redistribution. That support is expressed in the tectonic relief model rather than by weakening the flooding acceptance threshold.

These gates supplement lineage and area closure. They do not substitute for same-seed visual review.

## Explicit non-goals

The tectonic synthesis still does **not** attempt to solve:

- mantle convection from first principles;
- continuous finite-element lithospheric deformation;
- exact Earth plate reconstruction;
- high-resolution tectonic history directly on L8;
- terrain, climate, hydrology, resources, or gameplay regions inside the plate stage.

Historical tectonics remains coarse and deterministic. Persistent identities and sparse event state are inherited to finer resolutions, where the existing lithosphere, topography, climate, and surface-process systems consume them.
