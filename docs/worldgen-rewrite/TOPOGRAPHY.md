# WG-4 Initial Physical Topography

WG-4 is the first Planet Engine stage that owns authoritative solid-surface elevation and initial bathymetry. It consumes accepted WG-3.75 physical inheritance and fine boundary provenance; it does not regenerate tectonics or use a dominant arbitrary terrain-noise field.

## Stage boundary

```text
WG-3.75 inherited crust / history / lithosphere
        +
fine geological boundary provenance
        +
PlanetPhysicalParameters
        ↓
crustal isostatic support
        +
oceanic age / thermal subsidence
        +
collision, ridge, rift, trench and arc responses
        +
basin/subsidence history
        +
broad mantle dynamic support
        ↓
lithospheric mechanical filtering
        ↓
area-weighted solid-surface datum
        ↓
global surface-water volume solve
        ↓
solid elevation + sea level + water depth + land/ocean mask
```

## Physical components

The v2 terrain state keeps forcing components separately inspectable: isostatic, oceanic thermal, orogenic/collision, ridge, rift/basin, trench, arc, and mantle-dynamic elevation. The final solid surface is their mechanically filtered sum. This accounting is diagnostic and prevents tectonic relief from becoming an opaque final noise function.

Crustal support uses WG-3 thickness and density against the explicit isostatic mantle density. Oceanic and transitional crust subsides with a bounded square-root age relation. Fine inherited boundary interfaces seed geodesic distance fields for collision, spreading, rifting and polarized subduction morphology. Subduction polarity keeps trenches on the subducting plate and arc uplift on the overriding plate, with the arc peak displaced inland from the interface.

WG-3.5 effective elastic thickness, weakness, and structural fabric control a bounded finite-volume neighbor filter using WG-1 center-distance and dual-interface geometry. This is a first mechanical-response approximation, not a full elastic thin-shell solver.

## Datum and water solve

The mechanically expressed solid surface is shifted to zero area-weighted global mean. This datum is arbitrary but deterministic; physical land/ocean classification comes only after the water solve.

For a candidate sea level `S`, standing-water volume is integrated as:

```text
V(S) = Σ area_sr[i] × radius² × max(0, S - elevation[i])
```

WG-4 solves this monotonic equation against `surface_water_mass_kg / ocean_water_density_kg_per_m3`. Wet profiles therefore derive sea level from basin volume rather than a fixed land percentile. Zero-water profiles expose no fictitious sea level or submerged samples.

WG-4's water mask is an initial hydrostatic standing-water surface. Closed-basin routing, lakes, rivers, overflow and freshwater belong to later hydrology.

## Earth-like hypsometry calibration

The `@2` default retunes broad relief amplitudes after the WG-5 calibration baseline showed that the original Earth-like reference placed roughly two thirds to four fifths of land above 2 km across representative L6 worlds. The correction remains upstream in WG-4 rather than compensating with an artificial climate warming term.

The calibrated defaults reduce broad crustal/isostatic support and old/broad uplift while preserving signed tectonic morphology: `isostatic_scale = 0.55`, inherited orogeny `1200 m`, collision uplift `2400 m` over a `600 km` kernel, and mantle-dynamic relief `650 m`. Ridge, rift, trench, arc, thermal-subsidence, water-inventory and mechanical-filter parameters are unchanged.

A five-seed L4 Earth-like ensemble now occupies a deliberately broad **pre-erosion** envelope: land fraction `23–30%`, mean land elevation `1.28–1.82 km`, mean standing-ocean depth `3.49–3.81 km`, and p95 solid elevation `4.49–5.91 km`, with exact water-volume closure and no safety clamps. These are calibration guards, not a requirement to reproduce Earth exactly; later erosion and glaciation are still expected to reshape the distribution.

## Resolution

WG-4 consumes WG-3.75 coarse-to-fine inheritance. The intended global production investigation is accepted L6 physical truth inherited onto an L7 terrain substrate; lower levels remain supported for tests and fast diagnostics. WG-4 never reruns WG-2/WG-3/WG-3.5 independently at the terrain level.

## Determinism

Stage identity is `terrain:initial-topography@2` with namespace `terrain:structure:v1`. The topography hash includes stage/version/seed, WG-4 model parameters, planetary parameters, WG-3.75 inheritance identity, fine boundary identity, ordered solid elevation, sea-level state, and ordered water depth. Upstream tectonic/geology/lithosphere/inheritance hashes are not mutated.

## Explicit non-goals

WG-4 does not generate climate, drainage, river incision, erosion, sediment transport, glaciation, mature coastlines, detailed lithology, resource deposits, gameplay Regions/Features, factories, or meter-scale global terrain. Those remain downstream stages.
