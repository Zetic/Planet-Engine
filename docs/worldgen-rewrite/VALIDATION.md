# Planet Engine Validation

Worldgen acceptance focuses on numerical and physical behavior rather than gameplay clickability.

## WG-0

- Rust core has no gameplay dependency;
- browser and WASM protocol versions are explicit and equal;
- dense field dimensions are bounded and validated;
- same seed produces identical values/hash;
- different seeds produce different values/hash;
- native CLI executes the same core generator;
- separate WASM crate compiles for `wasm32-unknown-unknown`;
- Worker transfer uses dense typed data rather than per-cell objects;
- legacy v7 gameplay path remains untouched.

## WG-1 topology gates

The canonical spherical substrate is not accepted from visual inspection alone.

- exact hierarchy counts `V = 10×4^L+2`, `E = 30×4^L`, `F = 20×4^L`;
- Euler characteristic `V-E+F = 2`;
- total dual-cell area closes to `4π` steradians within strict tolerance;
- exactly twelve degree-five sites;
- every regular site degree six;
- reciprocal neighbors;
- no non-manifold edges: every primal edge belongs to exactly two faces;
- normalized unit positions;
- positive center-to-center geodesic edge lengths;
- positive dual-interface lengths;
- directed center distances and dual-interface lengths are symmetric across each neighbor pair;
- dense neighbor, center-distance, and interface-distance arrays remain index aligned;
- dual-cell area coefficient of variation remains within the defined topology quality bound;
- dual-interface variation is measured and remains within the defined finite-volume quality bound;
- inherited sample IDs and refinement provenance remain stable between levels;
- deterministic topology hash;
- latitude/longitude conversion round-trips within strict tolerance;
- tangent bases remain orthonormal, including polar anchors;
- local east/north/up meter coordinates round-trip through global Cartesian coordinates within strict tolerance.

The topology CLI reports cell-area, center-edge, and dual-interface distributions so changes to geometric quality are visible before downstream physics is added.

## WG-2 tectonic gates

Plate tectonics is accepted as a spherical kinematic partition, not by whether a diagnostic screenshot resembles Earth.

- identical seed + plate count + topology produces identical plate ownership, kinematics, boundaries, and tectonic hash;
- changing the stage seed changes tectonic identity;
- every topology sample has exactly one valid plate owner;
- every requested plate is non-empty and owns its deterministic seed sample;
- every plate forms one connected component on the canonical neighbor graph;
- summed plate control areas close to `4π` steradians;
- representative worlds keep plate areas and plate-seed spacing at macro scale rather than producing degenerate fragments;
- a fixed five-seed L5/18-plate regression requires at least four worlds to contain both a plate larger than `1.5×` mean area and a plate smaller than `0.65×` mean area, preventing regression toward near-equal tessellation;
- every rigid plate velocity is tangent to the spherical surface (`r·v ≈ 0`);
- plate reference speeds remain finite and within the intended tectonic-scale diagnostic range;
- every boundary joins samples owned by different plates;
- each inter-plate canonical neighbor edge is represented once in the undirected boundary set;
- boundary normal/shear rates are finite;
- convergent + divergent + transform counts exactly equal total boundary count;
- boundary classification follows the documented relative-motion rule rather than geographic labels;
- WASM/Worker arrays preserve one-to-one sample, plate, and boundary indexing;
- WG-2 source remains independent from legacy Region/Feature/NAV/gameplay contracts.

The native `tectonics` CLI reports plate-area range, seed spacing, plate speed, boundary-type counts, and deterministic tectonic identity. Multi-seed regression tests are preferred over tuning a single visually attractive world.

## WG-3 crust and geological-history gates

WG-3 is accepted only if dense crustal state remains physically distinct and causally linked to accepted WG-2 kinematics.

- same seed + accepted topology/tectonics produces identical geology hash and ordered dense fields;
- all crust/history arrays have exactly one value per topology sample, are finite, and remain within documented physical/index bounds;
- continental + transitional + oceanic control areas close to total spherical area;
- continental crust is substantially older, thicker, and less dense/more buoyant than oceanic crust in ensemble statistics;
- oceanic ages remain within the supported young-oceanic range and the youngest oceanic samples preferentially occur near active spreading systems;
- oceanic crust age increases statistically with geodesic distance from spreading systems rather than being independent random texture;
- each WG-2 boundary edge is preserved exactly once by WG-3 geological interpretation;
- transform boundaries remain transform regimes;
- divergent boundaries resolve only to oceanic ridge, continental rift, or transitional divergence;
- convergent boundaries resolve only to oceanic subduction, ocean-continent subduction, or continental collision;
- subduction regimes carry a non-null polarity and preferentially select the lower-buoyancy/oceanic side according to the documented rule;
- trench and volcanic-arc histories appear on opposite sides of polarized subduction systems;
- continental collision and overriding-margin histories raise orogenic influence and can thicken continental/transitional crust;
- continental rifting raises extension history and can thin continental/transitional crust;
- all bounded history fields remain in `[0,1]`;
- plate crust fractions sum to one and are area-weighted rather than sample-count weighted;
- `Major` / `Intermediate` / `Minor` labels are derived from continuous plate area and never replace that physical quantity;
- representative worlds include mixed-crust plates, demonstrating that plate identity is not synonymous with crust type;
- multi-seed validation checks distributions across several planets instead of tuning one showcase seed;
- WASM/Worker sample, boundary, and plate-summary arrays preserve one-to-one indexing;
- WG-3 adds no elevation, bathymetry, terrain, lithology, climate, hydrology, resources, Regions, or gameplay state.

The native `geology` CLI reports crust fractions, continental/oceanic age and thickness separation, derived plate scale-class counts, geological boundary regimes, deterministic tectonic/geology identities, and elapsed generation time.

## WG-3.5 lithosphere and tectonic-refinement gates

WG-3.5 is accepted only if it adds mechanically useful downstream state without replacing accepted topology, macro tectonics, or crust/history identity.

- same seed + accepted WG-1/WG-2/WG-3 inputs produces identical lithosphere hash and ordered dense fields;
- changing the seed materially changes WG-3.5 identity;
- every lithosphere field has exactly one finite value per topology sample;
- normalized mechanical/structural fields remain in `[0,1]` and signed thermal/support/buoyancy fields remain in `[-1,1]`;
- effective elastic thickness remains finite and within the documented 4–86 km first-model bounds;
- old, low-strain continental crust is stronger on average than highly strained crust across a multi-seed ensemble;
- inherited rift/transform/suture/margin categories arise from continuous structural fabric rather than arbitrary global tessellation;
- mantle upwelling and dynamic support remain broad support fields rather than high-frequency terrain noise;
- every derived terrane/microplate is a connected subset of exactly one WG-2 parent macro plate;
- fragment IDs never replace or mutate upstream macro plate IDs;
- refined kinematic-domain IDs are deterministic and distinguish derived fragments from unfragmented macro interiors;
- terranes inherit parent macro motion while microplates receive only bounded deterministic motion refinement;
- total fragmented area remains selective and is currently capped at approximately 22% of the sphere;
- the global fragment count remains bounded;
- a multi-seed regression exercises actual microplate generation rather than accepting a permanently empty refinement system;
- WG-2 tectonic and WG-3 geology hashes remain unchanged when WG-3.5 is generated;
- WASM/Worker sample and fragment-summary arrays preserve one-to-one indexing;
- WG-3.5 adds no elevation, bathymetry, terrain geometry, climate, hydrology, detailed lithology, resources, or gameplay state.

The native `lithosphere` CLI reports mean strength/weakness, effective elastic thickness, mantle upwelling/support, structural-zone populations, terrane/microplate counts, fragmented area, representative fragment properties, all upstream hashes, lithosphere hash, and elapsed generation time.

## WG-3.75 multiresolution inheritance and physical-profile gates

WG-3.75 is accepted only if finer global substrates refine accepted physical truth instead of silently regenerating a different planet.

- coarse level must not exceed fine level;
- WG-1's stable inherited sample prefix is required and validated before refinement;
- every fine sample has exactly one deterministic nearest coarse provenance source;
- every coarse sample remains present at the same sample ID and retains its accepted categorical and physical values exactly;
- direct scalar refinement from a coarse level to a fine level is identical to staged refinement through intermediate hierarchy levels;
- categorical refinement is deterministic, complete, and seed-independent once accepted coarse truth/topology are fixed;
- geodesic provenance tie breaking is stable by source ID rather than runtime iteration order;
- continuous interpolation remains finite and within the physical bounds of the source stage where those bounds apply;
- domain-constrained interpolation does not average crust/mechanical state across incompatible accepted categorical domains;
- the assembled inherited WG-2/WG-3/WG-3.5 bundle preserves accepted tectonic, geology, and lithosphere hashes;
- the inherited physical-state hash explicitly includes refinement provenance, planetary-parameter identity, and accepted upstream stage hashes;
- every fine inter-plate edge receives a compatible accepted coarse-boundary provenance source;
- inherited fine boundary interfaces preserve tectonic kind, geological regime, subduction polarity, normal rate, and shear rate from their accepted coarse source;
- fine boundary reconstruction is deterministic and carries a separate ordered boundary hash;
- protocol/WASM version 6 exposes exactly one value per fine sample for transported inherited fields and index-aligned arrays for fine interfaces/provenance;
- alternate rocky physical profiles can vary water inventory and other explicit physical parameters without changing the Earth-like default or upstream tectonic/geological identities;
- the Earth-like reference profile remains physically valid and reports finite derived mass, bulk density, surface area, water volume, and equivalent global-water depth;
- the native inheritance smoke path supports a representative L4→L6 refinement and reports upstream/provenance/parameter/inheritance hashes;
- WG-3.75 adds no elevation, bathymetry, sea-level solution, climate, hydrology, erosion, detailed lithology, resources, Regions, or gameplay state.

The native `inheritance` CLI reports coarse/fine levels and sample counts, deterministic provenance/parameter/inheritance hashes, accepted WG-2/WG-3/WG-3.5 hashes, Earth-like equivalent global-water depth, and elapsed time. The `profile` CLI reports the active physical planet profile and its derived physical quantities. Browser regressions require protocol v6 and the WG-3.75 lab to expose inherited sample masks, coarse-source provenance, fine boundary provenance, inherited physical fields, and the explicit no-terrain contract.

## Later physical gates

Later stages add validation for hypsometry and isostatic response, lithologic distributions, climate zonation, drainage acyclicity, water routing, erosion/sediment conservation, and glacial/coastal morphology.

Earth datasets are validation targets for distributions and morphology, not required generated outputs.

## WG-4 initial-topography gates

WG-4 acceptance requires finite sample-aligned component and final elevation fields; deterministic topography identity; unchanged upstream identities; nontrivial crustal/isostatic and tectonic relief; bounded safety-clamp use; area-weighted hypsometry; exact dry-profile behavior; and an Earth-like water-volume solve whose integrated standing-water volume matches the explicit target within numerical tolerance. Ensemble validation checks that oceanic cooling tends toward deeper old seafloor and that collision, ridge, rift, trench/arc polarity, basin tendency and mantle support produce the expected signed statistical responses without requiring any generated seed to reproduce Earth exactly. The Earth-like reference also carries a broad multi-seed pre-erosion hypsometry guard: representative L4 worlds must retain mixed land/ocean coverage, kilometer-scale rather than multi-kilometer mean continental elevation, plausible standing-ocean depth, bounded p95 relief, exact water-volume closure, and no safety clamps. The guard is intentionally wide enough to preserve seed diversity and is not an Earth heightmap target.


## WG-5 coupled-climate gates

WG-5 requires deterministic identity, annual thermal convergence, atmospheric moisture conservation, finite sample-aligned climate fields, small projected ocean divergence, and permanent L7 execution. Stage `4` additionally requires runtime moisture donor-limiter occupancy below `1%` and same-coarse L6→L7 global precipitation drift below `10%` on seed `ci-wg5-l7`, coarse L5, 24 plates. This directly guards against recurrence of the former resolution-cap-dominated single-sweep moisture routing.

## WG-7A fluvial-erosion and sediment gates

WG-7A is accepted as a deterministic, conservative forcing/routing stage before terrain mutation:

- all WG-7A fields align on the canonical fine topology;
- the supplied WG-6D state retains the exact accepted WG-6A drainage and WG-6C lake ancestry;
- the complete phase-major WG-6D realized-discharge field is present and finite;
- effective erosive discharge is peak-sensitive and deterministic across the retained seasonal hydrograph;
- receiver slope and channel length come only from the accepted WG-6A receiver edge and immutable WG-4 solid terrain;
- hydraulic channel width remains finite, positive for positive effective discharge, and inside the parameterized bounds;
- inherited erodibility remains in `[0.05, 1]` and responds monotonically to the documented strength/weakness/fragmentation/fabric terms;
- diagnostic incision is finite, nonnegative, bounded by the parameterized ceiling, and does not mutate WG-4 terrain;
- sediment production uses receiver-segment length and discharge-derived channel width rather than dual-cell area;
- sediment transport follows the accepted WG-6A upstream-to-downstream order;
- every member of an active WG-6C depression acts as a complete first-pass sediment trap, preventing dry depression members from bypassing the lake control volume;
- generated sediment closes into land, lake, and terminal/ocean deposition within `1e-10` in permanent CI;
- the fixed-ancestry L6/L7 diagnostic is interpreted as a resolution-sensitivity check rather than per-cell equality, with accepted generated-sediment drift around 11.9% for seed `ci-wg5-l7`;
- every public WG-7A vector and upstream ancestry identity participates in deterministic hashing;
- browser/WASM protocol v15 carries WG-7A in the same cumulative planet result, and the primary Lab exposes the accepted WG-7A diagnostic fields without a second generation request;
- WG-4 terrain, WG-6A receiver/depression topology, and all accepted WG-6 hydrology remain unchanged by WG-7A.

`bash scripts/check-wg7a-erosion.sh` is the permanent fixed L4 acceptance path. It checks canonical dimensions, nonempty erosive behavior, finite positive forcing, deterministic hash formatting, and global sediment closure. Native unit regressions additionally cover peak-sensitive effective discharge, inherited erodibility, bounded incision, hydraulic-width scaling, complete lake-depression trapping, ocean export, and mass conservation.

