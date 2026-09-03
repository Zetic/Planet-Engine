# Spherical Plate Tectonics

WG-2 introduces the first causal physical partition on the canonical WG-1 sphere. It generates a deterministic present-day plate mosaic and rigid plate kinematics. It does not yet generate crustal composition, geological history, or terrain.

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
```

The output is a kinematic tectonic substrate for later geological-history and crustal-state stages, not a time-stepped mantle or plate simulation.

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

WG-2 supports 4–48 plates. Browser diagnostics are bounded to topology level 6; higher-resolution geological stages may later consume a coarse tectonic field through explicit refinement/interpolation rather than rerunning unrelated plate truth.

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

This classification is a kinematic descriptor, not yet a geological landform. A later geological-history stage will distinguish consequences such as ocean-ocean subduction, continent-continent collision, continental rifting, ridge spreading, trench formation, or transform fault systems using crustal state and inherited history.

## Determinism

WG-2 owns the isolated random namespace:

```text
worldgen:tectonics:plates:v1
```

The tectonic identity hash includes stage seed, plate seed samples, rigid angular-velocity vectors, ordered sample ownership, and ordered boundary kinematics. Changes to downstream geology must not alter WG-2 plate truth.

## Diagnostics

The native CLI and browser lab expose:

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

The browser can render plate ownership, boundary type, rigid motion direction, and the underlying topology on either an orthographic globe or an equirectangular projection. Projection never changes physical truth.

## Explicit non-goals

WG-2 does **not** generate:

- continental versus oceanic crust;
- crust age or thickness;
- subduction polarity;
- geological time evolution;
- uplift or subsidence;
- elevation, bathymetry, or relief;
- lithology;
- climate or hydrology;
- resources;
- gameplay Regions, Features, NAV, or selection state.

Those remain downstream. The immediate next physical stage after WG-2 should consume this plate mosaic and boundary kinematics to construct geological history/crustal state without replacing the accepted topology or plate identities.
