# WG-5 Coupled Planetary Climate

WG-5 converts the accepted WG-4 physical surface into a deterministic climatology. It is a generation-time physical solve, not a perpetual post-generation weather simulation.

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

WG-5 intentionally includes a reduced B+ surface-ocean circulation model. Wind stress produces candidate currents, latitude- and rotation-rate-dependent Coriolis response deflects them, WG-4 ocean connectivity removes land-crossing flow, and bathymetry reduces shallow-water mobility. The candidate field is converted to antisymmetric ocean-interface transports and passed through a deterministic graph pressure projection so the retained transport has a small divergence residual. ENU current vectors are reconstructed from those projected interface transports for diagnostics, while SST heat advection uses the projected transports directly; ocean diffusion also remains on ocean-only neighbors. SST then feeds back into atmospheric temperature and circulation. WG-5 does not attempt a full 3-D salinity/thermohaline ocean.

Projected ocean-edge transports drive SST advection through a conservative donor-cell update. Aggregate donor outflow is CFL-limited per orbital phase, so the explicit heat step remains stable as mesh spacing shrinks through the L7 quality target without weakening circulation at coarser levels. Atmospheric moisture transport likewise scales aggregate outgoing graph transfers to the donor water mass before applying paired transfers, preserving moisture mass instead of relying on post-transport zero clamps.

The reduced ocean-advection coupling is calibrated separately from the CFL safety cap. The default coupling is `0.010`, which retains causal current-driven SST transport while meeting the WG-5 L7 annual convergence target; the donor CFL cap remains an independent resolution-safety bound.

Rotation is a physical input rather than an Earth-fixed display assumption. Slower rotation broadens the reduced overturning/Hadley regime and weakens zonal/Coriolis control, while faster rotation narrows the overturning regime and increases rotational control. Coriolis deflection varies continuously with latitude and is exactly zero at the equator.

## Physical parameter boundary

The accepted `PlanetPhysicalParameters` remains the WG-3.75/WG-4 profile contract. WG-5 introduces a separate `ClimatePhysicalParameters` identity for properties that should not retroactively change accepted terrain identity:

- orbital eccentricity;
- longitude of periapsis;
- atmospheric mean molar mass;
- atmospheric specific heat;
- reduced longwave optical depth.

`ClimateParameters` separately owns numerical/model choices such as albedo, thermal response, atmospheric heat transport, wind response, current coupling, ocean diffusion/advection, moisture transport, condensation, and orographic precipitation.

## Seasonal solve

The default solver evaluates 24 orbital phases. Those phases exist only while generating the climatology. The final state stores annual means, extrema, seasonality, and first annual harmonics that can reconstruct seasonal diagnostic views without rerunning WG-5.

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

The intended quality target remains L7 (~163,842 global samples / ~56 km characteristic Earth-like spacing), with lower levels used for CI and browser diagnostics. L8 remains a later profiling candidate rather than a requirement for WG-5 acceptance.
