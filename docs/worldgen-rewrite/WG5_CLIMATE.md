# WG-5 Coupled Planetary Climate

WG-5 converts the accepted WG-4 physical surface into a deterministic climatology. It is a generation-time physical solve, not a perpetual post-generation weather simulation. The current algorithm is stage version `6`. Stage `5` closed potential evaporation and aridity; Stage `6` preserves that accepted physical model while solving broad climate on a coarse global mesh and applying deterministic fine-grid corrections.

## Causal pipeline

```text
WG-4 elevation / bathymetry / land-ocean state
        +
planet rotation / orbit / stellar forcing / atmosphere
        ↓
seasonal insolation
        ↓
land-ocean thermal response
        ↓
atmospheric heat redistribution
        ↓
prevailing winds
        ↓
wind-driven surface ocean circulation
        ↓
sea-surface-temperature heat transport
        ↺ atmospheric temperature / winds
        ↓
ocean evaporation + conservative atmospheric moisture transport
        ↓
orographic lifting / condensation / precipitation
        ↓
moisture balance / aridity / snow and sea-ice potential
```

Stage `3` separates unresolved atmospheric/background shortwave reflection from the fraction of local surface albedo that reaches the reduced top-of-atmosphere budget. The Earth-like reference uses atmospheric shortwave reflectivity `0.25` and surface-albedo coupling `0.25`; land, ocean, snow, and ice therefore remain causally distinct without exposing their raw surface-albedo contrast directly to the planetary budget. This is a reduced cloud/atmosphere masking term, not an explicit cloud field. When reference atmospheric pressure is zero, that masking vanishes and the exposed surface albedo directly controls absorbed shortwave; ocean SST likewise collapses to the exposed radiative surface temperature because no distinct atmospheric air-sea reservoir exists.

Atmospheric heat redistribution is now a conservative geometry-aware diffusion solve. Mesh interfaces use physical interface length and center distance, atmospheric thermal capacity uses local pressure, cell area, and specific heat, and a deterministic diagonally-preconditioned conjugate-gradient solve advances the implicit diffusion step. The Earth-like reference diffusivity is `2.0e6 m^2/s`. Air-sea exchange is likewise heat-capacity-aware: an `8 W/m^2/K` exchange coefficient couples the atmospheric column to a `14 m` effective mixed layer while conserving their combined column heat absent diagnostic clamps.

WG-5 intentionally includes a reduced B+ surface-ocean circulation model. Wind stress produces candidate currents, latitude- and rotation-rate-dependent Coriolis response deflects them, WG-4 ocean connectivity removes land-crossing flow, and bathymetry reduces shallow-water mobility. The candidate field is converted to antisymmetric ocean-interface transports and passed through a deterministic graph pressure projection so the retained transport has a small divergence residual. ENU current vectors are reconstructed from those projected interface transports for diagnostics, while SST heat advection uses the projected transports directly; ocean diffusion also remains on ocean-only neighbors. SST then feeds back into atmospheric temperature and circulation. WG-5 does not attempt a full 3-D salinity/thermohaline ocean.

Projected ocean-edge transports drive SST advection through a conservative donor-cell update. Aggregate donor outflow is CFL-limited per orbital phase, so the explicit heat step remains stable as mesh spacing shrinks through the L7 quality target without weakening circulation at coarser levels. Stage `4` gives atmospheric moisture its own conservative finite-volume graph transport. Seasonal-mean flow is integrated with adaptive Courant substeps, paired edge transfers conserve water mass, and a `1.0 m/s` climatological moisture-speed ceiling bounds unresolved travel distance without altering the physical wind field used by circulation, evaporation demand, or orography. The default minimum is four substeps, the adaptive ceiling is 64, and the donor CFL limit is `0.90`. Runtime limiter occupancy and maximum substeps are exported directly by the solver.

Bulk ocean evaporation is now an aerodynamic mass-flux demand rather than a per-phase humidity relaxation. WG-5 remains a reduced climatology rather than a full surface-energy/GCM solve, so stage `4` applies a latent-energy availability ceiling: at most `0.45` of reduced absorbed ocean shortwave energy in a phase may support evaporation, using `2.45 MJ/kg` latent heat. This is a climatological availability bound, not a second thermal-energy sink; the independently calibrated stage-3 thermal EBM remains unchanged. Moisture convergence after each transport substep may precipitate sufficiently humid inflow before the existing condensation and orographic sinks.

The reduced ocean-advection coupling is calibrated separately from the CFL safety cap. The default coupling is `0.010`, which retains causal current-driven SST transport while meeting the WG-5 L7 annual convergence target; the donor CFL cap remains an independent resolution-safety bound.

Rotation is a physical input rather than an Earth-fixed display assumption. Slower rotation broadens the reduced overturning/Hadley regime and weakens zonal/Coriolis control, while faster rotation narrows the overturning regime and increases rotational control. Coriolis deflection varies continuously with latitude and is exactly zero at the equator.

## Physical parameter boundary

The accepted `PlanetPhysicalParameters` remains the WG-3.75/WG-4 profile contract. WG-5 introduces a separate `ClimatePhysicalParameters` identity for properties that should not retroactively change accepted terrain identity:

- orbital eccentricity;
- longitude of periapsis;
- atmospheric mean molar mass;
- atmospheric specific heat;
- reduced atmospheric shortwave reflectivity;
- reduced longwave optical depth.

`ClimateParameters` separately owns numerical/model choices such as the maximum global climate-solver level, surface-albedo shortwave coupling, land/ocean thermal response, atmospheric heat diffusivity and solver iterations, air-sea exchange and mixed-layer depth, wind response, current coupling, ocean diffusion/advection, moisture transport, condensation, and orographic precipitation.

## Seasonal solve

The default solver evaluates 24 equal-time orbital phases. For eccentric orbits each phase is interpreted as mean longitude, Kepler's equation is solved for eccentric anomaly, and the resulting true solar longitude and orbital distance drive declination and stellar-flux scaling. Those phases exist only while generating the climatology. The final state stores annual means, extrema, seasonality, and first annual harmonics that can reconstruct seasonal diagnostic views without rerunning WG-5.

## Current scope

WG-5 owns:

- seasonal insolation;
- surface temperature and local atmospheric pressure;
- prevailing surface winds;
- wind-driven surface currents;
- SST and surface-ocean heat transport;
- atmospheric humidity and conservative graph moisture advection;
- evaporation and precipitation;
- orographic precipitation and resulting rain-shadow depletion;
- potential evaporation, moisture balance, and aridity;
- snowfall fraction, persistent-snow potential, and sea-ice potential.

WG-5 does not own weather systems, storms, cloud microphysics, 3-D atmosphere, deep-ocean overturning, salinity circulation, rivers, lakes, glacier flow, erosion, biomes, resources, Regions/Features, or gameplay integration.

## Resolution

The canonical quality target remains L7 (~163,842 output samples / ~56 km characteristic Earth-like spacing). Stage 6 solves broad coupled climate at no more than L5 by default, then reconstructs L6/L7 fields and applies local topographic, radiative, pressure, wind-drag, orographic, PET, coastline, and ocean-mask corrections. `ClimateMetrics` exposes the global solver level and cell count separately from the canonical output count. L8 remains a later profiling candidate rather than a requirement for WG-5 acceptance.

## Acceptance and diagnostics hardening

The public WG-5 generator now rejects a climate state if the configured annual temperature convergence tolerance is not reached or if the final atmospheric moisture budget exceeds the conservation tolerance. Native CLI, Rust callers, WASM, and browser generation therefore share the same acceptance contract.

`atmospheric_specific_heat_j_per_kg_k` now causally scales reduced atmospheric heat redistribution rather than acting only as hash metadata. Airless planets disable that atmospheric redistribution path entirely. Reported wind mean/max statistics are time-aware speed statistics over the retained final climatology year rather than magnitudes of annual-mean vector components, and the reported ocean divergence residual is the worst orbital-phase residual from the retained year.

The climate hash covers the full public climate output-vector state as well as stage identity, upstream topography identity, planet parameters, and climate parameter hashes. Permanent CI includes an optimized L7 WG-5 convergence/conservation run in addition to the lower-resolution smoke test.
