# WG-3.75 Multiresolution Physical Inheritance

WG-3.75 makes resolution changes an explicit deterministic operation instead of rerunning accepted physics at a finer level.

```text
coarse WG-2/WG-3/WG-3.5 truth
        ↓
WG-1 stable sample ancestry
        ↓
WG-3.75 refinement operators
        ├─ fine sample provenance
        └─ fine boundary-interface provenance
        ↓
finer physical substrate for WG-4
```

## Sample-field contracts

- Existing sample IDs are preserved exactly.
- Continuous scalar/vector fields use hierarchical midpoint interpolation.
- Categorical fields use a deterministic geodesic nearest-coarse-source map on the fine graph.
- Domain-sensitive continuous fields use constrained interpolation so a midpoint does not average values across a categorical discontinuity.
- The provenance map has its own deterministic hash.
- The inherited physical state identity combines the refinement stage version, provenance hash, physical-parameter hash, and accepted WG-2/WG-3/WG-3.5 hashes.

The high-level `inherit_physical_state` bundle carries plate ownership, crust properties/history, lithospheric mechanics, structural state, and refined kinematic domains onto a finer topology. It does not add new geology or terrain. Fine-scale physics may later add detail, but inherited coarse samples remain exact.

## Boundary-interface contracts

WG-4 cannot reconstruct ridges, trenches, collisions, or transform systems from smoothed scalar fields alone. WG-3.75 therefore separately refines accepted coarse boundary interfaces.

For every fine inter-plate edge, `inherit_boundary_interfaces` resolves a deterministic coarse source boundary and preserves:

- WG-2 tectonic boundary kind;
- WG-3 geological boundary regime;
- WG-3 subduction polarity;
- accepted normal convergence/divergence rate;
- accepted shear rate;
- the source coarse-boundary index.

Exact descendants of a coarse boundary retain that source directly. New fine interfaces select a compatible source from the accepted coarse boundary network by deterministic spherical proximity and plate-pair compatibility. The resulting boundary set has a separate deterministic hash. This keeps narrow causal interface semantics sharp even when broad history fields are interpolated smoothly.

## Operator choice

Crust properties are constrained by crust province. Mechanical strength/weakness/elastic thickness and fragmentation-related fields are constrained by WG-3.5 kinematic domain. Broad geological influence and mantle-support fields are interpolated continuously because those physical influences are allowed to cross present-day domain boundaries.

## Browser/WASM transport

Planet Engine protocol v6 exposes a dedicated `generate-inheritance` request with independent coarse and fine levels. The Worker transfers the fine topology, provenance arrays, complete inherited physical sample bundle, reconstructed fine boundary interfaces, deterministic hashes, and the active physical planet profile as typed arrays/scalars.

The standalone `worldgen-lab.html` uses this path directly. Its WG-3.75 diagnostics include exact inherited-sample masks, nearest coarse provenance, fine boundary provenance, inherited tectonic/crust/lithosphere fields, and the active Earth-like water/interior parameters.

## Non-goals

WG-3.75 does not calculate elevation, bathymetry, sea level, hydrology, erosion, climate, detailed lithology, resources, or gameplay geography.
