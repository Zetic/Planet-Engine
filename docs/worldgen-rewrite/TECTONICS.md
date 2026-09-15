# Spherical Plate Tectonics

WG-2 introduces the first causal physical partition on the canonical WG-1 sphere. It generates a deterministic present-day plate mosaic and rigid plate kinematics. WG-2.5 now derives connected boundary systems and analytical tectonic chronology from that accepted plate truth. Neither stage generates terrain.

## Stage boundary

```text
canonical PlanetTopology
        +
planet physical radius
        +
seed + requested macro plate count
        ↓
deterministic plate seeds
        ↓
connected spherical plate partition
        ↓
rigid Euler-pole motion per plate
        ↓
relative boundary kinematics
        ↓
WG-2.5 connected boundary systems
        ↓
analytical event age + accumulated displacement
```

The output is a kinematic and chronological tectonic substrate for later crust, lithosphere, orogen-province, and topography stages. It is deliberately not a time-stepped mantle or plate simulation.

## Plate partition

WG-2 chooses deterministic seed samples on the canonical topology with a seeded stochastic minimum-separation process. The first seed is stage-random. Later seeds are drawn from deterministic pseudo-random candidates and accepted once they satisfy a deliberately modest exclusion radius; a deterministic best-separated fallback exists for densely requested configurations.

The exclusion radius prevents pathological seed clusters without forcing a blue-noise or near-equal-area tessellation. Major and minor macro plates must be able to coexist. A fixed five-seed L5/18-plate regression therefore checks that the partition retains meaningful area variance rather than converging toward equal Voronoi territories.

Plate ownership is then solved as a multi-source shortest-path Voronoi partition over the `PlanetTopology` neighbor graph using canonical geodesic center distances. This gives every sample exactly one plate owner while preserving graph connectivity back to its seed.

Acceptance requires:

- every sample has a valid plate ID;
- every requested plate is non-empty;
- every plate owns its seed sample;
- every plate is one connected component on the topology graph;
- all plate control-area weights close to `4π` steradians;
- seed spacing remains macro scale rather than degenerating into pathological clusters;
- multi-seed plate-area statistics preserve both larger and smaller macro plates rather than a near-equal tessellation.

WG-2 supports 4–48 plates. Higher-resolution geological stages consume coarse physical truth through explicit refinement/interpolation rather than rerunning unrelated plate truth.

## Rigid plate motion

Each plate receives a deterministic Euler pole and angular speed. Angular velocity is stored as a 3-vector in radians per million years.

At unit surface direction `r`, rigid plate velocity is:

```text
v = ω × r × R
```

where `R` is physical planet radius. The velocity is therefore tangent to the spherical surface by construction.

WG-2 deliberately models plate-scale rigid kinematics rather than deforming plate interiors. Intracrustal strain, diffuse deformation, terranes, orogens, and geological inheritance belong to later stages.

## Boundary kinematics

A tectonic boundary edge is any canonical neighbor edge whose samples have different plate owners. For each such edge, WG-2 evaluates the two rigid plate velocities near the edge midpoint and decomposes their relative velocity into:

- boundary-normal rate;
- along-boundary shear rate.

The present diagnostic classification is:

```text
normal contribution < 35% of relative speed  → transform / shear-dominated
otherwise normal rate < 0                    → convergent
otherwise                                    → divergent
```

This classification is a kinematic descriptor, not a geological landform.

## WG-2.5 connected boundary systems

WG-2.5 replaces the assumption that every boundary edge is an independent causal source. Boundary edges are assembled into deterministic connected systems when they share the same plate pair, kinematic class, and local boundary neighborhood. Each system owns a stable list of source boundary edges.

For each boundary system WG-2.5 derives:

- connected system identity and plate pair;
- finite endpoints and branch/junction counts;
- an along-strike graph coordinate and physical distance;
- local boundary curvature;
- convergence obliquity from normal versus shear motion;
- deterministic tectonic event age;
- cumulative convergence, extension, and shear displacement.

The chronology is analytical rather than time-stepped. Event age is a deterministic causal state associated with the connected system; accumulated displacement is obtained from that age and accepted rigid-plate boundary velocity. This gives later stages a distinction between a young fast collision and a mature long-lived collision without advancing the entire planet through dozens of historical plate solutions.

This stage is intentionally allowed to invalidate old downstream calibration envelopes. It exists to provide stronger upstream causes for the forthcoming pre-orogenic lithosphere and Tectonic Orogen Province stages, not to preserve previous `orogenic_history` morphology.

## Determinism

WG-2 owns the isolated random namespace:

```text
worldgen:tectonics:plates:v1
```

WG-2.5 owns a separate chronology namespace:

```text
worldgen:tectonics:history-systems:v1
```

The WG-2 tectonic identity hash remains based on stage seed, plate seed samples, rigid angular-velocity vectors, ordered sample ownership, and ordered boundary kinematics. WG-2.5 has its own history hash, so adding chronology does not silently replace accepted macro plate identity.

## Diagnostics

WG-2 exposes:

- plate count and per-sample ownership;
- plate seed positions;
- Euler poles and angular velocities;
- plate area fractions;
- boundary edge count;
- convergent/divergent/transform counts;
- boundary normal and shear rates;
- minimum seed separation;
- mean reference plate speed;
- deterministic topology and tectonic hashes.

WG-2.5 additionally exposes model-level data for boundary-system identity, along-strike position, event age, cumulative convergence/extension/shear, obliquity, curvature, endpoint count, and junction count. Browser visualization is intentionally deferred until the later integration PR; the physical stage itself is available to Rust consumers immediately.

## Explicit non-goals

WG-2/WG-2.5 do **not** generate:

- continental versus oceanic crust;
- crust age or thickness;
- subduction polarity or slab physics;
- literal time-stepped plate reconstruction;
- uplift or subsidence;
- elevation, bathymetry, or relief;
- lithology;
- climate or hydrology;
- resources;
- gameplay Regions, Features, NAV, or selection state.

Those remain downstream. The next architectural stage should consume the accepted plate mosaic plus WG-2.5 chronology and connected systems to construct pre-orogenic lithospheric state without forcing new tectonic physics to match obsolete terrain measurements.
