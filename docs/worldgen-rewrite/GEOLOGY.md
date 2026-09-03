# Crustal State and Geological History

WG-3 converts the accepted spherical plate mosaic and present-day WG-2 kinematics into dense crustal state plus inferred geological memory. It still does not generate elevation or lithology.

## Stage boundary

```text
WG-1 PlanetTopology
        +
WG-2 TectonicModel
        +
planet physical parameters
        +
seeded geology namespaces
        ↓
coherent crustal provinces
        ↓
crust type / age / thickness / density / buoyancy
        ↓
geological interpretation of tectonic boundaries
        ↓
oceanic spreading-age inference + subduction polarity
        ↓
propagated geological-history fields
        ↓
derived plate summaries
```

The output is the causal substrate for WG-4 initial topography. WG-4 should use crustal thickness, buoyancy, orogenic/rift/subduction/ridge history, and related fields to create physical relief rather than re-inventing tectonics with terrain noise.

## Plate identity is not crust type

WG-3 deliberately rejects the legacy simplification that an entire plate is intrinsically continental or oceanic.

Crust is stored per canonical surface sample. One rigid WG-2 plate may contain continental, transitional, and oceanic crust simultaneously. Coherent proto-continental/cratonic provinces are generated independently of current plate borders, allowing current plate boundaries to cut through or combine older crustal domains.

The initial crust taxonomy is intentionally broad:

```text
Oceanic
Transitional
Continental
```

Rock types, igneous differentiation, sedimentary packages, metamorphic grade, and other lithologic state belong to a later lithology stage.

## Dense physical fields

WG-3 owns the following sample-aligned physical/state fields:

```text
crustKind                 u8
crustProvinceId           u16
crustAgeMyr               f32
crustThicknessKm          f32
crustDensityKgPerM3       f32
buoyancyIndex             f32

orogenicHistory           f32
riftHistory               f32
ridgeHistory              f32
subductionHistory         f32
trenchHistory             f32
volcanicArcHistory        f32
transformHistory          f32
subsidenceHistory         f32
basinPotential            f32
crustalStrain             f32
```

`buoyancyIndex` is a compact relative index used for geological decisions such as which oceanic side is more subductable. The underlying age, density, and thickness remain authoritative physical quantities.

History fields are dimensionless bounded memories/influences. They are not elevation and do not directly imply a specific landform.

## Continental provinces

WG-3 creates several deterministic ancient continental nuclei and a smooth spherical structural fabric. Their combined affinity is converted into physically area-weighted continental and transitional fractions rather than classifying each plate wholesale.

The target planet remains broadly Earth-like but seed-variable. Continental interiors receive older/thicker/lower-density crust than oceanic domains. Transitional crust forms margins between the two states.

Current WG-2 plate borders do not clip these provinces. This permits, for example, a plate containing both a continental block and adjoining oceanic lithosphere.

## Oceanic crust age

Oceanic age is causal with respect to spreading systems.

WG-3 first identifies divergent boundaries whose local crustal state makes them oceanic ridges or transitional divergence. It then propagates geodesic distance through non-continental crust and converts distance to an inferred age using the associated half-spreading rate:

```text
age ≈ distance_from_spreading_system / half_spreading_speed
```

Oceanic age is bounded to the supported young-oceanic range. Domains that cannot be connected to a current spreading system receive deterministic inherited oceanic age rather than becoming undefined.

This is an inferred-history model, not an exact reconstruction of plate positions through hundreds of millions of years.

## Geological boundary regimes

WG-2 supplies only present-day kinematics:

```text
convergent
divergent
transform
```

WG-3 combines that with local crustal state to derive geological regimes:

```text
OceanicSubduction
OceanContinentSubduction
ContinentalCollision
OceanicRidge
ContinentalRift
TransitionalDivergence
Transform
```

At convergent ocean-continent boundaries, the oceanic side subducts. At ocean-ocean boundaries, the lower-buoyancy side subducts. Continental collisions do not receive a forced subduction polarity in the WG-3 approximation.

Subduction polarity is stored separately from boundary regime so downstream stages can distinguish the trench/subducting side from the overriding/arc side.

## Geological history propagation

Present boundary regimes seed finite-distance influence fields over the canonical neighbor graph. Different processes use different physical length scales. The first model includes:

- collision/overriding-margin orogenic memory;
- continental extension/rift memory;
- spreading-ridge memory;
- subduction-zone memory;
- trench-side memory;
- overriding volcanic-arc memory;
- transform/fault-zone memory;
- subsidence tendency;
- sedimentary-basin potential;
- aggregate crustal strain.

A low-amplitude deterministic inherited component allows ancient continental structure and basin tendency away from currently active boundaries. It is subordinate to the causal boundary/crust system and is not terrain noise.

Orogenic history thickens continental/transitional crust; rift history thins it. Oceanic thickness and density vary with inferred age. These altered physical properties are then available to WG-4 isostatic/topographic calculations.

## Derived plate summaries

Plate labels are summaries of physical truth, not generator inputs.

Each WG-2 plate receives a derived summary containing:

- physical area and area fraction;
- area-based `Major`, `Intermediate`, or `Minor` scale class;
- continental/transitional/oceanic area fractions;
- area-weighted mean crust age and thickness;
- length-weighted convergent/divergent/transform boundary fractions.

Current thresholds are relative to the mean plate area on that planet:

```text
Major         area >= 1.50 × mean plate area
Minor         area <= 0.65 × mean plate area
Intermediate  otherwise
```

Physics should consume the continuous quantities. The categorical scale class is diagnostic/derived metadata. True microplates remain a later geological-fragmentation problem rather than being forced into the primary WG-2 partition.

## Determinism

WG-3 splits deterministic identity into isolated namespaces:

```text
worldgen:geology:crust-history:v1
worldgen:geology:crust-provinces:v1
worldgen:geology:crust-properties:v1
worldgen:geology:history:v1
```

Changing a later history calculation must not silently move the upstream continental province seeds because random draw order changed.

## Explicit non-goals

WG-3 does **not** generate:

- elevation or bathymetry;
- mountains, trenches, rift valleys, shelves, or basins as geometric terrain;
- lithology or rock classification;
- climate;
- hydrology;
- erosion or sediment transport;
- glaciation;
- resource deposits;
- gameplay Regions, Features, NAV, selection, or machine placement.

Those remain downstream consumers of accepted physical truth.
