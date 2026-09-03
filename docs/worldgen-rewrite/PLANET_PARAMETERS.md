# Planet Physical Parameters

`PlanetPhysicalParameters` remains Earth-like by default, preserving the existing accepted generation path, but now carries the physical values needed by later terrain/ocean stages and by future non-Earth rocky profiles.

The contract includes radius, gravity, rotation, axial tilt, orbital period, stellar flux, reference pressure, surface-water mass, ocean-water density, isostatic mantle density, internal heat flux, and mantle thermal expansivity.

Derived helpers provide planetary mass from surface gravity/radius, mean bulk density, surface area, water volume, equivalent global water depth, and a deterministic parameter hash.

The parameter hash is part of WG-3.75 inherited-state identity. A dry or otherwise altered rocky profile therefore cannot silently reuse an Earth-like downstream terrain identity, even when the currently inherited tectonic fields are unchanged.

WG-4 will consume the water inventory and isostatic parameters but owns the actual elevation/bathymetry and global sea-level solve.

WG-4 now consumes the explicit isostatic mantle density, surface-water mass, ocean-water density and planetary radius. The water inventory is converted to target volume and solved against generated basin geometry; zero-water profiles produce no fictitious sea-level state.
