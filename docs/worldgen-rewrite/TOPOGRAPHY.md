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

The terrain state keeps forcing components separately inspectable: isostatic, oceanic thermal, orogenic/collision, ridge, rift/basin, trench, arc, and mantle-dynamic elevation. The final solid surface is their mechanically filtered sum. This accounting is diagnostic and prevents tectonic relief from becoming an opaque final noise function.

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

The calibrated defaults reduce broad crustal/isostatic support and old/broad uplift while preserving signed tectonic morphology: `isostatic_scale = 0.55`, inherited orogeny `1200 m`, collision uplift `2400 m` over a `600 km` kernel, and mantle-dynamic relief `650 m`. WG-4 `@3` narrowed and lowered the shared ridge kernel from `2000 m / 600 km` to `1200 m / 450 km`. WG-4 `@4` separates mature `OceanicRidge` source response from `TransitionalDivergence`: pure oceanic spreading now uses `0.50 × (0.10 + 0.90 × normalized_divergence)` before the existing inherited ridge-history factor, while transitional divergence keeps the prior `0.35 + 0.65 × normalized_divergence` response. This avoids double-counting young-ocean thermal relief as a second near-full direct uplift without flattening rifted/transitional margins. Across a six-seed L5→L7 / 16-plate ensemble, aggregate oceanic-ridge endpoint emergence falls from about `20%` to `12%`, both-land ridge edges from about `11%` to `5%`, and submerged ridge endpoints shallower than `500 m` from about `30%` to `10%`; median submerged ridge depth rises from ~`0.86 km` to ~`1.23 km`. Mean land fraction remains ~`27%`, mean land elevation ~`1.41 km`, mean ocean depth ~`3.68 km`, and continental-collision endpoint emergence remains >`92%`. Trench, arc, thermal-subsidence, water-inventory and mechanical-filter parameters remain unchanged in `@4`.

WG-4 `@5` separates an active **continental rift valley** from a **transitional breakup seaway**. WG-3 already expresses extension through continental crust thinning, rift-derived subsidence, and basin potential; the older WG-4 response then stacked a strong guaranteed `ContinentalRift` trough plus another full inherited-rift depression on top of those upstream signals. At an active rift source, `rift_history` is intentionally near one, so that combination made mature-looking marine corridors nearly automatic. `@5` reduces only the extra direct continental-rift response to `0.10 × (0.05 + 0.95 × normalized_divergence)` and limits duplicated inherited-rift relief on continental crust to `0.05`. Transitional crust retains the prior `0.55` inherited-rift relief weight so advanced breakup can still form Red-Sea-style seaways, while basin/subsidence history and crustal-isostatic thinning remain authoritative. In the six-seed L5→L7 / 16-plate acceptance ensemble, aggregate `ContinentalRift` endpoint flooding falls from about `65%` to `43%`, endpoints deeper than `500 m` from about `61%` to `28%`, and endpoints deeper than `1 km` from about `55%` to `17%`. For the visible `interlink-wg7c` calibration seed, continental-rift flooding falls from about `70%` to `32%` and no continental-rift endpoint remains deeper than `500 m`. `TransitionalDivergence` remains strongly marine at about `82%` flooded, preserving genuine breakup/young-sea morphology.

WG-4 `@6` tightens the **pure oceanic ridge** morphology after connected-component profiling showed that endpoint-only gates still allowed long emergent ridge chains to join into oversized noncontinental islands. Oceanic-ridge direct response is reduced from `0.50` to `0.10` of the WG-4 ridge source function and inherited ridge-history relief from `500 m` to `75 m`; `TransitionalDivergence`, continental-rift response, thermal subsidence, collision relief, and the global water inventory are unchanged. Across the same six-seed L5→L7 / 16-plate ensemble, noncontinental land components touching pure oceanic ridges fall from about `15.42 million km²` total to `4.94 million km²`, large components (≥`25,000 km²`) from `143` to `65`, the largest component from about `431,000 km²` to `195,000 km²`, and emergent ridge endpoints from `1190` to `485`. The existing ridge acceptance remains comfortably physical: mean submerged ridge depth is ~`1.89 km`, median ~`1.84 km`, overall land fraction ~`28.6%`, continental-collision endpoint emergence ~`92.9%`, and transitional-divergence flooding remains ~`81%`. A dedicated connected-component regression now guards this morphology rather than relying only on individual boundary endpoints.

A five-seed L4 Earth-like ensemble now occupies a deliberately broad **pre-erosion** envelope: land fraction `23–30%`, mean land elevation `1.28–1.82 km`, mean standing-ocean depth `3.49–3.81 km`, and p95 solid elevation `4.49–5.91 km`, with exact water-volume closure and no safety clamps. These are calibration guards, not a requirement to reproduce Earth exactly; later erosion and glaciation are still expected to reshape the distribution.

## Resolution

WG-4 consumes WG-3.75 coarse-to-fine inheritance. The intended global production investigation is accepted L6 physical truth inherited onto an L7 terrain substrate; lower levels remain supported for tests and fast diagnostics. WG-4 never reruns WG-2/WG-3/WG-3.5 independently at the terrain level.

## Determinism

Stage identity is `terrain:initial-topography@6` with namespace `terrain:structure:v1`. The topography hash includes stage/version/seed, WG-4 model parameters, planetary parameters, WG-3.75 inheritance identity, fine boundary identity, ordered solid elevation, sea-level state, and ordered water depth. Upstream tectonic/geology/lithosphere/inheritance hashes are not mutated.

## Explicit non-goals

WG-4 does not generate climate, drainage, river incision, erosion, sediment transport, glaciation, mature coastlines, detailed lithology, resource deposits, gameplay Regions/Features, factories, or meter-scale global terrain. Those remain downstream stages.
