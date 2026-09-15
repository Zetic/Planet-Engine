# Tectonic Orogen Provinces

WG-3.6 replaces the old assumption that mountain history is a single isotropic distance field around convergent boundary samples. It creates finite, connected tectonic provinces from WG-2.5 boundary systems and the WG-3.5 pre-orogenic substrate.

## Causal chain

```text
WG-2 macro plates / rigid motion
        +
WG-2.5 boundary systems / chronology / accumulated convergence
        +
WG-3 crust kind / crust province / crust age
        +
WG-3.5 pre-orogenic strength / Te / inherited fabric / fragments
        ↓
WG-3.6 Tectonic Orogen Provinces
        ↓
future WG-4 province-driven topography
```

The stage deliberately does not consume the legacy `orogenic_history`, present `crustal_strain`, history-modified crust thickness/density/buoyancy, or any existing WG-4 mountain response. Those fields remain compatibility state until the topography cutover.

## Province construction

Every convergent WG-2.5 boundary system is ordered by its along-strike coordinate and segmented into finite provinces. Segments split when tectonic style changes, when the pre-orogenic substrate changes strongly, or when a long connected system exceeds a physically useful province length.

Segmentation uses causal variables including:

- accumulated convergence and event age;
- convergence obliquity and local curvature;
- intrinsic lithospheric weakness and effective elastic thickness;
- inherited structural fabric;
- crust-province and crust-age discontinuities;
- terrane/microplate contacts.

The result is not a union of fixed-radius buffers. Each boundary source belongs to one province and carries a local width derived from the local causal substrate.

## Province classes

WG-3.6 distinguishes:

```text
ContinentalCollision
CollisionalPlateau
CordilleranArc
IslandArc
TerraneAccretion
TranspressionalOrogen
```

Continental collision provinces can mature into broad collisional plateaus as age and accumulated shortening increase. Ocean-continent convergence produces strongly asymmetric cordilleran provinces on the overriding plate. Ocean-ocean convergence produces island-arc provinces. Strong inherited fragmentation can produce terrane-accretion provinces. Highly oblique convergence is represented as transpressional orogeny rather than being forced through an orthogonal collision template.

## Width, asymmetry, and terminations

Local province width is derived from maturity, cumulative shortening, weakness, inherited fabric, age/province discontinuity, curvature, and fragment contacts. It is intentionally allowed to vary by large factors along one connected boundary system.

The stage also applies explicit longitudinal taper at open boundary-system terminations. Width and core intensity collapse toward real system endpoints instead of ending as a constant-width tube.

Deformation is asymmetric by construction:

- continental collisions distinguish hinterland and foreland sides from pre-orogenic strength contrast;
- subduction systems identify subducting and overriding plates from crust type and oceanic age;
- overriding-plate cordilleras can carry volcanic-arc and back-arc zones while the subducting side remains narrow;
- strong obliquity produces a narrow transpressional core.

Nearest-source assignment is restricted to the source plate pair, so a broad neighboring segment cannot simply refill a narrow segment by taking the maximum of multiple radial fields. This is the architectural break from the old inherited-orogen model.

## Dense province fields

WG-3.6 outputs sample-aligned fields intended for the next topography rewrite:

```text
provinceIds
provinceKind
orogenicIntensity
crustalRootIndex
plateauIndex
foldThrustIndex
forelandBasinIndex
volcanicArcIndex
backarcExtensionIndex
sutureIndex
transpressionIndex
maturityIndex
shorteningIndex
localWidthKm
sourceAlongStrikeFraction
```

These are causal tectonic/topographic inputs, not elevations. WG-4 will decide how each structural zone becomes relief.

## Radical-development policy

WG-3.6 is not tuned against the legacy inherited-orogen width, active-collision response, climate, hydrology, or elevation calibration envelopes. Those measurements are observational telemetry until the new tectonic/topographic chain is complete.

Blocking validation is limited to correctness:

- deterministic province identity;
- finite and bounded dense fields;
- valid province membership;
- convergent boundary coverage;
- valid plate-side ownership;
- causal independence from legacy present-day orogenic output;
- successful workspace/browser compilation.

No gate requires the replacement model to resemble the old mountain geometry.

## Deliberate compatibility boundary

The old `CrustalModel::orogenic_history` still exists after this PR because the current WG-4 implementation consumes it. WG-3.6 does not use it. The next PR will rewire WG-4 to consume these province fields and can then retire the old radial orogen path from visual authority.
