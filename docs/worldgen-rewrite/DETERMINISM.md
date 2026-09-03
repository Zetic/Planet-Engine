# Determinism Contract

Planet Engine identity consists of a user-facing generator version plus seed, while each stage owns isolated deterministic namespaces.

Current and planned examples:

```text
worldgen:foundation:synthetic:v1
worldgen:tectonics:plates:v1
worldgen:geology:crust-history:v1
worldgen:geology:crust-provinces:v1
worldgen:geology:crust-properties:v1
worldgen:geology:history:v1
worldgen:lithosphere:state:v1
worldgen:lithosphere:mechanics:v1
worldgen:lithosphere:mantle-support:v1
worldgen:lithosphere:tectonic-refinement:v1
terrain:structure:v1
lithology:v1
climate:v1
hydrology:v1
```

Changing random draw count in a later stage must not rearrange earlier world truth.

WG-0 establishes this contract with a stable FNV-1a namespace hash plus a documented integer mixer. It deliberately avoids `DefaultHasher` and hash-map iteration order for persistent identity.

WG-2 uses the isolated tectonic namespace `worldgen:tectonics:plates:v1` for deterministic plate seed placement and rigid Euler-pole kinematics. Its ordered tectonic hash covers the derived stage seed, plate seed samples, angular-velocity vectors, per-sample plate ownership, and boundary kinematics. Downstream crust/geology changes therefore cannot silently alter accepted WG-2 plate truth.

WG-3 separates geological random identity into four streams. `crust-provinces` controls coherent proto-continental/cratonic placement, `crust-properties` controls physical within-province variation, `history` controls inherited geological memory, and `crust-history` identifies the assembled WG-3 stage. A change to inherited basin or orogen calculations must therefore not move continental province seeds merely because the number or ordering of random draws changed.

WG-3's geology hash is ordered over the stage seed, dense crust type/province fields, all physical crust fields, all geological-history fields, boundary regime/polarity state, and derived plate scale classes. It is intentionally separate from the WG-2 tectonic hash: accepted plate geometry and motion can remain identical while geology algorithms evolve under a new geology stage version.

WG-3.5 adds four isolated streams. `mechanics` controls lithospheric mechanical texture, `mantle-support` controls the broad mantle anomaly/support field, `tectonic-refinement` controls deterministic fragment motion perturbations and related refinement identity, and `state` identifies the assembled WG-3.5 stage. Its ordered lithosphere hash includes dense mechanical, thermal/support, structural, fragmentation, fragment-ID and refined-domain fields plus compact fragment identities/kinematics.

A WG-3.5 algorithm change must not silently alter the WG-2 tectonic hash or WG-3 geology hash. Macro plate IDs remain upstream identity even when derived microplates/terranes are introduced as finer kinematic domains.

WG-3.75 adds deterministic resolution-change identity without introducing a new random world. The refinement provenance hash covers coarse/fine levels, stable inherited sample prefix, and the ordered nearest-coarse-source mapping. Existing coarse sample IDs remain exact at finer topology levels. Continuous midpoint interpolation is deterministic from accepted parent values; categorical provenance is resolved by deterministic graph-geodesic distance with stable source-ID tie breaking; domain-constrained interpolation uses accepted categorical domains to prevent averaging across physical discontinuities.

Fine tectonic interfaces have a separate ordered boundary-refinement hash. Every fine inter-plate edge resolves to a deterministic accepted coarse boundary source using plate-pair compatibility and stable spherical proximity rules. Tectonic kind, geological regime, subduction polarity, normal rate, shear rate, and coarse-boundary provenance therefore remain reproducible when WG-4 consumes a finer topology.

The WG-3.75 inherited physical-state hash combines the refinement stage identity, provenance identity, explicit planetary-parameter identity, and accepted WG-2/WG-3/WG-3.5 hashes. Changing the selected physical planet profile must change that inherited-state identity even when seed and coarse geology remain the same. The Earth-like reference profile remains a stable explicit default rather than an implicit collection of constants.

## Regression expectations

- same seed + engine/stage version → identical field or stage hash;
- changed seed → materially different identity;
- changed namespace → independent derived seed;
- persistent indexing must never depend on unordered iteration;
- field hashes include shape and ordered values;
- tectonic hashes include ordered ownership and boundary state;
- geology hashes include ordered dense crust/history state and interpreted geological boundaries;
- lithosphere hashes include ordered mechanics, mantle support, structural fabric, fragmentation, refined domains, and fragment identities;
- deterministic stochastic sampling must be derived from stable integer streams, not runtime RNG state;
- WG-3 namespace isolation prevents downstream geological-memory changes from perturbing upstream crust-province identity;
- WG-3.5 namespace isolation prevents later terrain changes from perturbing accepted mechanical/refinement identity;
- derived fragment IDs are deterministic and never replace their parent WG-2 plate IDs;
- WG-3.75 exact descendants retain accepted coarse values bit-for-bit at inherited sample IDs;
- direct coarse→fine scalar refinement is identical to equivalent staged refinement through intermediate hierarchy levels;
- categorical provenance uses deterministic distance and source-ID tie breaking;
- domain-constrained continuous refinement never averages across an incompatible accepted categorical boundary;
- fine boundary interfaces deterministically preserve an accepted coarse boundary source and its physical semantics;
- planetary parameter hashes are stable and form part of inherited physical-state identity;
- changing water inventory or another explicit physical profile parameter does not silently mutate WG-2/WG-3/WG-3.5 hashes.

Floating-point determinism requirements are documented per physical stage as those stages are introduced. WG-0's synthetic proof uses integer field generation specifically so infrastructure determinism can be tested independently from numerical-model choices; WG-1 topology, WG-2 kinematics, WG-3 geology, WG-3.5 lithosphere, and WG-3.75 inheritance regression-test deterministic floating-point output identities on the supported toolchain.
