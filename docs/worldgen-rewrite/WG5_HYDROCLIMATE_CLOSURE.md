# WG-5 hydroclimate closure

WG-5 Stage 5 closes the reduced Earth-like hydroclimate state that WG-6 hydrology will consume. It does not add rivers, lakes, soil moisture, groundwater, storms, cloud microphysics, or a 3-D atmosphere.

## Closure change

Stage `climate:coupled-surface@5` retains the accepted Stage-4 thermal, wind, ocean-current, moisture-transport, and precipitation equations. The physical precipitation field is intentionally unchanged by this closure pass.

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
