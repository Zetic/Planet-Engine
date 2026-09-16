# Spherical Plate Tectonics

Planet Engine now separates **ancestral plate geometry** from **present-day plate ownership**. The canonical historical frontend still uses the deterministic spherical WG-2 partition as a deep-time initializer, but modern plates are no longer restricted to unions of those original graph-Voronoi cells. A bounded dynamic ownership pass allows present boundaries to migrate through ancestral material while persistent material ancestry survives underneath.

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
dynamic modern ownership evolution
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

This graph-Voronoi construction is useful for coherent deep-time domains, but it is no longer treated as final present-day geometry in the canonical historical pipeline.

Acceptance of the ancestral initializer still requires valid, connected, non-empty plates, exact area closure, deterministic seed spacing, and a non-degenerate distribution of plate areas.

## Coherent proto-continental material

Continental material is no longer initialized by independently selecting scattered ancestral carrier plates and attempting to weld them afterward. The historical lithosphere chooses a small set of separated proto-continental nuclei and grows each assembly contiguously across the ancestral adjacency graph until the global material target is reached.

Growth favors substantial shared contact and convergent relationships, tolerates some transform attachment, penalizes divergent attachment, and limits one assembly from consuming the whole target. Transitional margins are then derived from the edge of the resulting assembled continental material.

This makes isolated continental islands a later historical outcome—rift fragments, captured terranes, or microcontinents—rather than a default artifact of random carrier selection.

## Dynamic modern plate evolution

The modern ownership stage operates at the coarse sample level for a bounded number of deterministic synthetic epochs. It starts from the ancestry-based provisional modern grouping, then allows boundary samples to change present owner according to a combination of:

- local and second-ring plate cohesion;
- inherited rigid Euler-motion direction;
- deterministic low-frequency spherical shape forcing;
- bounded ownership inertia and ancestral affinity;
- plate-size constraints and connectivity repair.

Each plate retains an interior anchor so evolution cannot erase it. Ownership transfers are bounded per epoch, disconnected remnants are reassigned, and an explicit dominance-balancing pass prevents a single modern plate from swallowing an implausibly large fraction of the sphere.

The key architectural change is that `current_plate_id` is now free to evolve independently of `origin_plate_id`. Modern boundaries are therefore not required to coincide with ancestral cell edges.

## Modern rigid kinematics

After dynamic ownership stabilizes, modern angular velocity is reconstructed from the ancestral material currently carried by each modern plate, weighted by the physical area of that contribution. A representative interior seed is chosen from the evolved modern domain.

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

The bounded historical lineage pass and the dynamic modern-geometry pass are intentionally sparse. They do not retain a complete dense raster for every geological epoch. Persistent outputs are material identities, fragment lineage, ownership transfers, ages, and event records; downstream morphology rasterizes those causes into sutures, rifts, passive margins, active orogens, and fossil structures.

The model therefore aims for the causal qualities observed in the project’s Gleba reference sequence—old plates, denser material fragments, fewer broad present plates, then crust verification—without claiming source-code equivalence or a full physical mantle simulation.

## Determinism

Important namespaces include:

```text
worldgen:tectonics:plates:v1
worldgen:geology:historical-lithosphere:ancestral:v1
worldgen:geology:historical-lithosphere:epochs:v1
worldgen:geology:dynamic-modern-plates:v1
worldgen:geology:historical-lithosphere:modern-tectonics:v1
```

The historical identity hash covers evolved current ownership and persistent fragment lineage. The modern tectonic hash covers modern plate motion, evolved ownership, extracted boundary kinematics, and the upstream history hash.

## Blocking geometry acceptance

Permanent validation now checks properties that the older merge-only system could pass while still producing visibly polygonal plates or continental archipelagos:

- present boundaries must cut through ancestral material rather than remaining locked to ancestral cell edges;
- multiple ancestral plates must be split across modern owners with explicit material lineage;
- every modern plate must remain connected and non-empty;
- no modern plate may dominate an excessive fraction of the planet;
- continental material must not regress to a large population of tiny satellite components;
- historical lithosphere, morphology, WG-4 topography, browser diagnostics, and packaged WASM remain compatible.

These gates supplement lineage and area closure. They do not substitute for same-seed visual review.

## Explicit non-goals

The tectonic synthesis still does **not** attempt to solve:

- mantle convection from first principles;
- continuous finite-element lithospheric deformation;
- exact Earth plate reconstruction;
- high-resolution tectonic history directly on L8;
- terrain, climate, hydrology, resources, or gameplay regions inside the plate stage.

Historical tectonics remains coarse and deterministic. Persistent identities and sparse event state are inherited to finer resolutions, where the existing lithosphere, topography, climate, and surface-process systems consume them.
