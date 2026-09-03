# Canonical Planetary Topology

WG-1 implements the production spatial substrate selected by the rewrite blueprint: a hierarchical icosahedral geodesic sphere with a circumcentric dual control-volume geometry.

## Construction

- begin with a canonical regular icosahedron;
- subdivide each triangular face into four;
- normalize inserted edge midpoints onto the unit sphere;
- retain existing sample IDs at every finer level;
- record the birth level and parent edge for every inserted sample;
- treat mesh vertices as physical sample sites;
- use the vertex-neighbor graph for physical processes;
- use the spherical circumcentric Voronoi/Goldberg cell around each vertex as its surface-area domain.

The exact hierarchy obeys:

```text
V = 10 × 4^L + 2
E = 30 × 4^L
F = 20 × 4^L
V - E + F = 2
```

Exactly twelve sites have five neighbors. Every other site has six neighbors.

## Physical algorithm boundary

Physical stages must consume the `PlanetTopology` contract rather than depend on icosphere construction internals. The contract supplies:

```text
sample_count
unit_position(sample)
area_steradians(sample)
neighbors(sample)
neighbor_center_arc_length(sample, neighbor)
neighbor_dual_interface_arc_length(sample, neighbor)
```

The final two quantities deliberately distinguish the distance between adjacent physical sample centers from the length of their shared dual-cell boundary.

For a planet of radius `R`:

```text
cell area        = area_steradians × R²
center distance  = center_arc_radians × R
dual interface   = interface_arc_radians × R
```

This is sufficient geometric state for conservative finite-volume operators. A future diffusion, thermal transport, atmospheric transport, erosion, or other flux stage can weight exchange between adjacent cells using the shared interface length and center-to-center distance instead of pretending all tiles have equal geometry.

Production storage remains dense and specialized: flattened positions/faces plus CSR neighbor arrays and aligned center-distance/interface-length arrays. Topology independence is an algorithm boundary, not a requirement for heap-polymorphic cell objects.

## Dual cells and conservation

Every primal triangle receives a spherical circumcenter. Circumcenters around a sample are ordered in its local tangent plane to form the sample's spherical Voronoi control volume. The resulting dual areas are measured directly in steradians and must close to `4π` for the complete unit sphere.

Each primal edge belongs to exactly two triangles. The arc between those two triangle circumcenters is the shared dual interface of the two neighboring control volumes. The directed CSR representation stores that same interface length for both directions, and regression tests require those paired values to agree.

## Resolution hierarchy

The hierarchy is structural rather than a collection of unrelated grids. Coarse sample IDs remain valid prefixes of finer topologies, and new samples retain parent-edge provenance. Later world-generation stages can therefore select process-specific levels while retaining a deterministic relationship between coarse and fine planetary state.

WG-1 supports Rust topology construction through level 8. The browser diagnostic protocol is deliberately bounded to level 7 to keep transferable diagnostic arrays and canvas rendering within practical development limits. These are capability limits, not declarations that every future physical process should run at either level.

## Coordinate systems

The physical topology is spherical, while presentation and engineering coordinates remain separate concerns.

WG-1 provides:

- latitude/longitude ↔ unit-direction conversion;
- great-circle distance on a supplied physical planet radius;
- Earth-centered Cartesian surface anchors;
- local east/north/up tangent bases;
- local meter coordinates ↔ global Cartesian coordinates.

A machine or facility can therefore retain ordinary meter-scale Cartesian placement even when its global anchor lies on a spherical planet.

## Why the legacy equirectangular grid is not canonical

The current 4096 × 2048 coordinate space remains useful as a map projection and compatibility space, but it is not physical topology. The geodesic sphere removes polar singular rows and an artificial physical longitude seam while greatly reducing area and directional distortion.

`worldgen-lab.html` intentionally includes an equirectangular diagnostic projection to verify that projection is downstream of physical truth rather than the storage topology itself.

## Quality gates

WG-1 measures rather than assumes topology quality. Regression/CLI diagnostics include:

- exact V/E/F counts and Euler characteristic;
- five- versus six-neighbor counts;
- reciprocal adjacency;
- unit-vector normalization;
- total/minimum/maximum dual-cell area;
- dual-cell area coefficient of variation;
- center-edge length distribution;
- dual-interface length distribution;
- deterministic topology hash;
- refinement ID/provenance stability;
- geodetic and local-ENU round trips.

Tectonics does not begin until this substrate is accepted. Later physical models should treat area and interface variation explicitly rather than assume a perfectly uniform hexagonal lattice.
