# WG-5 hydroclimate closure

WG-5 Stage 5 closes the reduced Earth-like hydroclimate state that WG-6 hydrology will consume. It does not add rivers, lakes, soil moisture, groundwater, storms, cloud microphysics, or a 3-D atmosphere.

## Stage 7 hydroclimate partition recalibration

Stage `climate:coupled-surface@7` retains the Stage-6 multiresolution climate architecture and the accepted thermal, wind, ocean-current, moisture-transport, precipitation, snow, and sea-ice equations. It changes only the exported land potential-evaporation forcing consumed by WG-6.

Stage 5 made land PET resolution-stable by taking the minimum of aerodynamic saturation-deficit demand and the same `evaporation_energy_fraction` used to limit actual ocean evaporation. Downstream profiling after WG-4 `@7` showed that this conflated potential land demand with humidity-limited realized evaporation: wet cells drove the aerodynamic term toward zero, WG-6B Budyko AET stayed too small, and Earth-like seeds could exceed 80% runoff. A genuine full-L7 climate solve reproduced the same imbalance, ruling out multiresolution reconstruction as the root cause.

Stage 7 separates those roles. Actual ocean evaporation keeps `evaporation_energy_fraction = 0.45`, so the conserved atmospheric moisture source and precipitation physics are unchanged. Land PET uses a distinct `land_potential_evaporation_energy_fraction = 0.70` and the accepted absorbed-surface-energy field without taking a minimum against local aerodynamic humidity deficit. The value is an explicit reduced-model calibration parameter for potential surface-water demand, not a claim that 70% of surface energy is realized evapotranspiration. WG-6B retains the Fu/Budyko partition with `omega = 2.6`; WG-6C lake physics is unchanged.

Across the six-seed L5-to-L7 Earth-like acceptance ensemble, area-weighted runoff falls from roughly 72.8% before this recalibration to 46.9%, with individual seeds spanning about 34.6% to 56.2%. For `interlink-wg7c`, runoff falls from about 80.9% to 56.2% (`P=1049.3`, `AET=459.9`, `R=589.4 mm/yr`). As a downstream diagnostic, unchanged WG-6C geometry then reduces total lake area from about 8.50 million to 5.36 million km2. The remaining lake excess, if any, can therefore be investigated separately from the land-water partition.

The fixed-ancestry L6/L7 control measured mean land PET of about 2059 and 2070 mm/yr under the split formulation, roughly 0.5% drift. Precipitation-resolution behavior remains a separate Stage-6 reconstruction concern and is not altered by this closure.

## Closure change

Historical Stage `climate:coupled-surface@5` retained the accepted Stage-4 thermal, wind, ocean-current, moisture-transport, and precipitation equations. Stage 6 keeps this closure model and changes its execution architecture through multiresolution climate solving.

The exported land potential-evaporation diagnostic is now bounded by the same reduced latent-energy availability fraction used by ocean evaporation. Previously PET integrated unrestricted aerodynamic saturation-deficit demand on land while ocean evaporation was energy-limited. That made PET and the derived aridity index strongly resolution-sensitive even when the conserved water cycle itself had converged.

The `evaporation_energy_fraction` is also now included in `ClimateParameters::parameter_hash()`. Changing this physical closure parameter therefore changes the climate identity as required.

## Fixed-ancestry resolution evidence

Reference seed `ci-wg5-l7`, coarse L5, 24 plates:

| Metric | L6 | L7 |
| --- | ---: | ---: |
| samples | 40,962 | 163,842 |
| mean land precipitation | 764.353 mm/yr | 712.658 mm/yr |
| mean land PET, Stage 4 | 1,933.743 mm/yr | 2,824.906 mm/yr |
| mean land PET, Stage 5 | 949.582 mm/yr | 1,004.102 mm/yr |
| Stage-5 PET L6→L7 drift | — | 5.7% |
| persistent-snow land fraction | 0.1317 | 0.1642 |
| sea-ice ocean fraction | 0.1835 | 0.1987 |

Stage 5 reduces PET resolution drift from roughly 46% to roughly 5.7% without changing precipitation, thermal state, winds, currents, or moisture conservation. Native-cell precipitation percentiles and spatial coefficient of variation remain diagnostic rather than cross-resolution acceptance metrics because L7 intentionally resolves finer spatial structure.

## Permanent acceptance

`scripts/check-wg5-hydroclimate-closure.sh` provides `smoke` low-resolution structural bounds and `quality` fixed-ancestry L6↔L7 acceptance for precipitation, PET, dry/humid land fractions, cryosphere area, and tropical-to-subtropical rainfall structure. The existing WG-5 L7 gate remains authoritative for thermal convergence and global moisture conservation.

## WG-5 → WG-6 forcing contract

WG-6 may consume WG-4 surface elevation and land/ocean mask plus WG-5 annual/seasonal precipitation, temperature, potential evaporation, moisture balance, snowfall fraction, persistent-snow potential, and hydrologically relevant sea-ice potential. WG-6 must not depend on WG-5 implementation details such as moisture CFL substeps, convergence-precipitation efficiency, solver iteration counts, or limiter occupancy.

## Deferred

Performance optimization is deliberately separate from hydroclimate closure. WG-5 can now be optimized against the Stage-5 accepted outputs without moving the physical target. Hydrology, runoff routing, drainage basins, rivers, lakes, groundwater, erosion, and sediment remain downstream stages.
