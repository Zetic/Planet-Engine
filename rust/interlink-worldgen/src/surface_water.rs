use crate::{GeodesicTopology, PlanetPhysicalParameters, PlanetTopology, WorldgenError};
use std::collections::VecDeque;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Debug, PartialEq)]
pub struct HydrostaticSurfaceWaterMetrics {
    pub sea_level_m: Option<f64>,
    pub target_water_volume_m3: f64,
    pub solved_water_volume_m3: f64,
    pub water_volume_relative_error: f64,
    pub submerged_sample_count: usize,
    pub surface_hash: u64,
    pub water_state_hash: u64,
}

impl HydrostaticSurfaceWaterMetrics {
    pub fn surface_hash_hex(&self) -> String {
        format!("{:016x}", self.surface_hash)
    }

    pub fn water_state_hash_hex(&self) -> String {
        format!("{:016x}", self.water_state_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HydrostaticSurfaceWaterState {
    pub metrics: HydrostaticSurfaceWaterMetrics,
    pub elevation_above_sea_level_m: Vec<f32>,
    pub water_depth_m: Vec<f32>,
    pub submerged_mask: Vec<u8>,
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn validate_inputs(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f64],
    planet: PlanetPhysicalParameters,
) -> Result<(), WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    let count = topology.metrics().sample_count as usize;
    if solid_elevation_m.len() != count {
        return Err(WorldgenError::InvalidTopography(
            "hydrostatic surface-water elevations are not aligned to topology",
        ));
    }
    if solid_elevation_m.iter().any(|value| !value.is_finite()) {
        return Err(WorldgenError::InvalidTopography(
            "hydrostatic surface-water elevations must be finite",
        ));
    }
    Ok(())
}

fn water_volume_at_level(
    elevation_m: &[f64],
    areas_sr: &[f64],
    radius_m: f64,
    sea_level_m: f64,
) -> f64 {
    elevation_m
        .iter()
        .zip(areas_sr.iter())
        .map(|(elevation, area_sr)| {
            (sea_level_m - *elevation).max(0.0) * *area_sr * radius_m * radius_m
        })
        .sum()
}

fn solve_sea_level(
    elevation_m: &[f64],
    areas_sr: &[f64],
    planet: PlanetPhysicalParameters,
) -> (Option<f64>, f64, f64) {
    let target = planet.surface_water_volume_m3();
    if target == 0.0 {
        return (None, 0.0, 0.0);
    }

    let minimum = elevation_m.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = elevation_m
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let mut low = minimum - 1.0;
    let mut high = maximum + planet.equivalent_global_water_depth_m() + 1.0;
    while water_volume_at_level(elevation_m, areas_sr, planet.radius_m, high) < target {
        high += (high - low).max(1_000.0);
    }
    for _ in 0..96 {
        let middle = (low + high) * 0.5;
        let volume = water_volume_at_level(elevation_m, areas_sr, planet.radius_m, middle);
        if volume < target {
            low = middle;
        } else {
            high = middle;
        }
    }
    let sea_level = (low + high) * 0.5;
    let solved = water_volume_at_level(elevation_m, areas_sr, planet.radius_m, sea_level);
    let error = ((solved - target) / target).abs();
    (Some(sea_level), solved, error)
}

fn water_volume_at_level_masked(
    elevation_m: &[f64],
    areas_sr: &[f64],
    radius_m: f64,
    sea_level_m: f64,
    active_mask: &[u8],
) -> f64 {
    elevation_m
        .iter()
        .zip(areas_sr.iter())
        .zip(active_mask.iter())
        .map(|((elevation, area_sr), active)| {
            if *active == 0 {
                0.0
            } else {
                (sea_level_m - *elevation).max(0.0) * *area_sr * radius_m * radius_m
            }
        })
        .sum()
}

fn solve_sea_level_masked(
    elevation_m: &[f64],
    areas_sr: &[f64],
    planet: PlanetPhysicalParameters,
    active_mask: &[u8],
) -> (Option<f64>, f64, f64) {
    let target = planet.surface_water_volume_m3();
    if target == 0.0 {
        return (None, 0.0, 0.0);
    }
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    for (index, elevation) in elevation_m.iter().enumerate() {
        if active_mask[index] != 0 {
            minimum = minimum.min(*elevation);
            maximum = maximum.max(*elevation);
        }
    }
    if !minimum.is_finite() {
        return solve_sea_level(elevation_m, areas_sr, planet);
    }
    let mut low = minimum - 1.0;
    let mut high = maximum + planet.equivalent_global_water_depth_m() + 1.0;
    while water_volume_at_level_masked(elevation_m, areas_sr, planet.radius_m, high, active_mask)
        < target
    {
        high += (high - low).max(1_000.0);
    }
    for _ in 0..96 {
        let middle = (low + high) * 0.5;
        let volume = water_volume_at_level_masked(
            elevation_m,
            areas_sr,
            planet.radius_m,
            middle,
            active_mask,
        );
        if volume < target {
            low = middle;
        } else {
            high = middle;
        }
    }
    let sea_level = (low + high) * 0.5;
    let solved = water_volume_at_level_masked(
        elevation_m,
        areas_sr,
        planet.radius_m,
        sea_level,
        active_mask,
    );
    let error = ((solved - target) / target).abs();
    (Some(sea_level), solved, error)
}

fn connected_ocean_mask_at_level(
    topology: &GeodesicTopology,
    elevation_m: &[f64],
    ocean_seed_mask: &[u8],
    ocean_access_mask: Option<&[u8]>,
    sea_level_m: f64,
) -> Vec<u8> {
    let count = elevation_m.len();
    let mut connected = vec![0_u8; count];
    let mut queue = VecDeque::new();
    for sample in 0..count {
        if ocean_seed_mask[sample] != 0
            && elevation_m[sample] < sea_level_m
            && ocean_access_mask.map_or(true, |mask| mask[sample] != 0)
        {
            connected[sample] = 1;
            queue.push_back(sample as u32);
        }
    }
    while let Some(sample) = queue.pop_front() {
        for neighbor in topology.neighbors(sample) {
            let index = *neighbor as usize;
            if connected[index] == 0
                && elevation_m[index] < sea_level_m
                && ocean_access_mask.map_or(true, |mask| mask[index] != 0)
            {
                connected[index] = 1;
                queue.push_back(*neighbor);
            }
        }
    }
    connected
}

fn solve_connected_sea_level(
    topology: &GeodesicTopology,
    elevation_m: &[f64],
    planet: PlanetPhysicalParameters,
    ocean_seed_mask: &[u8],
    ocean_access_mask: Option<&[u8]>,
) -> (Option<f64>, f64, f64, Vec<u8>) {
    let count = elevation_m.len();
    if planet.surface_water_volume_m3() == 0.0 {
        return (None, 0.0, 0.0, vec![0; count]);
    }
    if ocean_seed_mask.iter().all(|value| *value == 0) {
        let (level, solved, error) =
            solve_sea_level(elevation_m, topology.dual_area_steradians(), planet);
        let mut active = vec![0_u8; count];
        if let Some(level) = level {
            for sample in 0..count {
                active[sample] = u8::from(elevation_m[sample] < level);
            }
        }
        return (level, solved, error, active);
    }

    // Start from the geologically oceanic reservoir.  Each iteration solves the exact water
    // inventory over the currently reached domain, then admits any additional cells connected by
    // a below-water path.  Reached basins stay active if the redistributed ocean subsequently
    // lowers below their sill; this models a basin that was actually flooded rather than
    // teleporting water into every low continental depression on the planet.
    let mut active = ocean_seed_mask.to_vec();
    for _ in 0..32 {
        let (level, _, _) = solve_sea_level_masked(
            elevation_m,
            topology.dual_area_steradians(),
            planet,
            &active,
        );
        let Some(level) = level else {
            return (None, 0.0, 0.0, vec![0; count]);
        };
        let connected = connected_ocean_mask_at_level(
            topology,
            elevation_m,
            ocean_seed_mask,
            ocean_access_mask,
            level,
        );
        let mut grew = false;
        for sample in 0..count {
            if connected[sample] != 0 && active[sample] == 0 {
                active[sample] = 1;
                grew = true;
            }
        }
        if !grew {
            let (level, solved, error) = solve_sea_level_masked(
                elevation_m,
                topology.dual_area_steradians(),
                planet,
                &active,
            );
            return (level, solved, error, active);
        }
    }
    let (level, solved, error) = solve_sea_level_masked(
        elevation_m,
        topology.dual_area_steradians(),
        planet,
        &active,
    );
    (level, solved, error, active)
}

fn surface_hash(solid_elevation_m: &[f64]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, &(solid_elevation_m.len() as u64).to_le_bytes());
    for value in solid_elevation_m {
        hash = fnv_update(hash, &value.to_bits().to_le_bytes());
    }
    hash
}

fn water_state_hash(
    surface_hash: u64,
    planet: PlanetPhysicalParameters,
    sea_level_m: Option<f64>,
    target_water_volume_m3: f64,
    solved_water_volume_m3: f64,
    water_volume_relative_error: f64,
    elevation_above_sea_level_m: &[f32],
    water_depth_m: &[f32],
    submerged_mask: &[u8],
) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, &surface_hash.to_le_bytes());
    hash = fnv_update(hash, &planet.parameter_hash().to_le_bytes());
    match sea_level_m {
        Some(level) => {
            hash = fnv_update(hash, &[1]);
            hash = fnv_update(hash, &level.to_bits().to_le_bytes());
        }
        None => {
            hash = fnv_update(hash, &[0]);
        }
    }
    hash = fnv_update(hash, &target_water_volume_m3.to_bits().to_le_bytes());
    hash = fnv_update(hash, &solved_water_volume_m3.to_bits().to_le_bytes());
    hash = fnv_update(hash, &water_volume_relative_error.to_bits().to_le_bytes());
    for value in elevation_above_sea_level_m {
        hash = fnv_update(hash, &value.to_bits().to_le_bytes());
    }
    for value in water_depth_m {
        hash = fnv_update(hash, &value.to_bits().to_le_bytes());
    }
    fnv_update(hash, submerged_mask)
}

fn solve_hydrostatic_surface_water_impl(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f64],
    planet: PlanetPhysicalParameters,
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    validate_inputs(topology, solid_elevation_m, planet)?;

    let count = topology.metrics().sample_count as usize;
    let target_water_volume_m3 = planet.surface_water_volume_m3();
    let (sea_level_m, solved_water_volume_m3, water_volume_relative_error) =
        solve_sea_level(solid_elevation_m, topology.dual_area_steradians(), planet);

    let mut elevation_above_sea_level_m = vec![0.0_f32; count];
    let mut water_depth_m = vec![0.0_f32; count];
    let mut submerged_mask = vec![0_u8; count];
    let mut submerged_sample_count = 0_usize;

    for sample in 0..count {
        if let Some(level) = sea_level_m {
            let relative = solid_elevation_m[sample] - level;
            elevation_above_sea_level_m[sample] = relative as f32;
            if relative < 0.0 {
                water_depth_m[sample] = (-relative) as f32;
                submerged_mask[sample] = 1;
                submerged_sample_count += 1;
            }
        } else {
            elevation_above_sea_level_m[sample] = solid_elevation_m[sample] as f32;
        }
    }

    let surface_hash = surface_hash(solid_elevation_m);
    let water_state_hash = water_state_hash(
        surface_hash,
        planet,
        sea_level_m,
        target_water_volume_m3,
        solved_water_volume_m3,
        water_volume_relative_error,
        &elevation_above_sea_level_m,
        &water_depth_m,
        &submerged_mask,
    );

    Ok(HydrostaticSurfaceWaterState {
        metrics: HydrostaticSurfaceWaterMetrics {
            sea_level_m,
            target_water_volume_m3,
            solved_water_volume_m3,
            water_volume_relative_error,
            submerged_sample_count,
            surface_hash,
            water_state_hash,
        },
        elevation_above_sea_level_m,
        water_depth_m,
        submerged_mask,
    })
}

fn solve_hydrostatic_surface_water_connected_impl(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f64],
    planet: PlanetPhysicalParameters,
    ocean_seed_mask: &[u8],
    ocean_access_mask: Option<&[u8]>,
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    validate_inputs(topology, solid_elevation_m, planet)?;
    let count = topology.metrics().sample_count as usize;
    if ocean_seed_mask.len() != count {
        return Err(WorldgenError::InvalidTopography(
            "connected-ocean seed mask is not aligned to topology",
        ));
    }
    if let Some(access) = ocean_access_mask {
        if access.len() != count {
            return Err(WorldgenError::InvalidTopography(
                "connected-ocean access mask is not aligned to topology",
            ));
        }
        if ocean_seed_mask
            .iter()
            .zip(access.iter())
            .any(|(seed, allowed)| *seed != 0 && *allowed == 0)
        {
            return Err(WorldgenError::InvalidTopography(
                "connected-ocean seed mask contains a cell excluded by the access mask",
            ));
        }
    }

    let target_water_volume_m3 = planet.surface_water_volume_m3();
    let (sea_level_m, solved_water_volume_m3, water_volume_relative_error, active_mask) =
        solve_connected_sea_level(
            topology,
            solid_elevation_m,
            planet,
            ocean_seed_mask,
            ocean_access_mask,
        );

    let mut elevation_above_sea_level_m = vec![0.0_f32; count];
    let mut water_depth_m = vec![0.0_f32; count];
    let mut submerged_mask = vec![0_u8; count];
    let mut submerged_sample_count = 0_usize;
    for sample in 0..count {
        if let Some(level) = sea_level_m {
            let relative = solid_elevation_m[sample] - level;
            elevation_above_sea_level_m[sample] = relative as f32;
            if active_mask[sample] != 0 && relative < 0.0 {
                water_depth_m[sample] = (-relative) as f32;
                submerged_mask[sample] = 1;
                submerged_sample_count += 1;
            }
        } else {
            elevation_above_sea_level_m[sample] = solid_elevation_m[sample] as f32;
        }
    }

    let surface_hash = surface_hash(solid_elevation_m);
    let water_state_hash = water_state_hash(
        surface_hash,
        planet,
        sea_level_m,
        target_water_volume_m3,
        solved_water_volume_m3,
        water_volume_relative_error,
        &elevation_above_sea_level_m,
        &water_depth_m,
        &submerged_mask,
    );
    Ok(HydrostaticSurfaceWaterState {
        metrics: HydrostaticSurfaceWaterMetrics {
            sea_level_m,
            target_water_volume_m3,
            solved_water_volume_m3,
            water_volume_relative_error,
            submerged_sample_count,
            surface_hash,
            water_state_hash,
        },
        elevation_above_sea_level_m,
        water_depth_m,
        submerged_mask,
    })
}

/// Solves a hydrostatic surface-water state for a stored solid-elevation field.
///
/// The solve uses canonical geodesic dual-cell area, planet radius, and the
/// planet's total surface-water inventory. Samples exactly on the solved sea
/// level remain unsubmerged, matching the existing WG-4 tie behavior.
pub fn solve_hydrostatic_surface_water(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f32],
    planet: PlanetPhysicalParameters,
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    let solid_elevation_m = solid_elevation_m
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    solve_hydrostatic_surface_water_impl(topology, &solid_elevation_m, planet)
}

/// Solves the global ocean from geologic seed cells and expands it only through terrain that
/// is actually reachable below the solved water surface.  Closed below-datum continental basins
/// remain dry until a marine connection is physically overtopped.
pub fn solve_hydrostatic_surface_water_connected(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f32],
    planet: PlanetPhysicalParameters,
    ocean_seed_mask: &[u8],
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    let solid_elevation_m = solid_elevation_m
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    solve_hydrostatic_surface_water_connected_impl(
        topology,
        &solid_elevation_m,
        planet,
        ocean_seed_mask,
        None,
    )
}

pub(crate) fn solve_hydrostatic_surface_water_connected_f64(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f64],
    planet: PlanetPhysicalParameters,
    ocean_seed_mask: &[u8],
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    solve_hydrostatic_surface_water_connected_impl(
        topology,
        solid_elevation_m,
        planet,
        ocean_seed_mask,
        None,
    )
}

pub(crate) fn solve_hydrostatic_surface_water_connected_with_access(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f32],
    planet: PlanetPhysicalParameters,
    ocean_seed_mask: &[u8],
    ocean_access_mask: &[u8],
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    let solid_elevation_m = solid_elevation_m
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    solve_hydrostatic_surface_water_connected_impl(
        topology,
        &solid_elevation_m,
        planet,
        ocean_seed_mask,
        Some(ocean_access_mask),
    )
}

pub(crate) fn solve_hydrostatic_surface_water_connected_with_access_f64(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f64],
    planet: PlanetPhysicalParameters,
    ocean_seed_mask: &[u8],
    ocean_access_mask: &[u8],
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    solve_hydrostatic_surface_water_connected_impl(
        topology,
        solid_elevation_m,
        planet,
        ocean_seed_mask,
        Some(ocean_access_mask),
    )
}

/// Internal f64 entry point used by WG-4 while its solid surface is still in
/// calculation precision. Keeping that precision boundary intact preserves the
/// accepted WG-4 sea level, water-depth field, and topography hash during this
/// extraction-only refactor.
pub(crate) fn solve_hydrostatic_surface_water_f64(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f64],
    planet: PlanetPhysicalParameters,
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    solve_hydrostatic_surface_water_impl(topology, solid_elevation_m, planet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_icosphere;

    fn synthetic_surface(topology: &GeodesicTopology) -> Vec<f32> {
        (0..topology.metrics().sample_count as usize)
            .map(|sample| ((sample % 11) as f32 - 5.0) * 240.0)
            .collect()
    }

    #[test]
    fn hydrostatic_solver_is_deterministic() {
        let topology = build_icosphere(1).unwrap();
        let surface = synthetic_surface(&topology);
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let a = solve_hydrostatic_surface_water(&topology, &surface, planet).unwrap();
        let b = solve_hydrostatic_surface_water(&topology, &surface, planet).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.metrics.water_state_hash, b.metrics.water_state_hash);
    }

    #[test]
    fn zero_water_inventory_produces_no_submerged_samples() {
        let topology = build_icosphere(1).unwrap();
        let surface = synthetic_surface(&topology);
        let mut planet = PlanetPhysicalParameters::earthlike_reference();
        planet.surface_water_mass_kg = 0.0;
        let state = solve_hydrostatic_surface_water(&topology, &surface, planet).unwrap();
        assert_eq!(state.metrics.sea_level_m, None);
        assert_eq!(state.metrics.target_water_volume_m3, 0.0);
        assert_eq!(state.metrics.solved_water_volume_m3, 0.0);
        assert_eq!(state.metrics.water_volume_relative_error, 0.0);
        assert_eq!(state.metrics.submerged_sample_count, 0);
        assert!(state.water_depth_m.iter().all(|value| *value == 0.0));
        assert!(state.submerged_mask.iter().all(|value| *value == 0));
        assert_eq!(state.elevation_above_sea_level_m, surface);
    }

    #[test]
    fn increasing_water_inventory_does_not_lower_sea_level() {
        let topology = build_icosphere(1).unwrap();
        let surface = synthetic_surface(&topology);
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let low = solve_hydrostatic_surface_water(&topology, &surface, planet).unwrap();
        let mut wetter_planet = planet;
        wetter_planet.surface_water_mass_kg *= 1.5;
        let high = solve_hydrostatic_surface_water(&topology, &surface, wetter_planet).unwrap();
        assert!(high.metrics.sea_level_m.unwrap() >= low.metrics.sea_level_m.unwrap());
    }

    #[test]
    fn solved_water_volume_closes_to_target() {
        let topology = build_icosphere(1).unwrap();
        let surface = synthetic_surface(&topology);
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let state = solve_hydrostatic_surface_water(&topology, &surface, planet).unwrap();
        assert!(state.metrics.water_volume_relative_error < 1.0e-10);
        assert!(state.metrics.solved_water_volume_m3 > 0.0);
    }

    #[test]
    fn constant_vertical_surface_offset_translates_sea_level_equally() {
        let topology = build_icosphere(1).unwrap();
        let surface = synthetic_surface(&topology);
        let offset_m = 512.0_f32;
        let shifted = surface
            .iter()
            .map(|value| *value + offset_m)
            .collect::<Vec<_>>();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let base = solve_hydrostatic_surface_water(&topology, &surface, planet).unwrap();
        let translated = solve_hydrostatic_surface_water(&topology, &shifted, planet).unwrap();
        let sea_level_delta =
            translated.metrics.sea_level_m.unwrap() - base.metrics.sea_level_m.unwrap();
        assert!((sea_level_delta - f64::from(offset_m)).abs() < 1.0e-9);
        assert_eq!(base.submerged_mask, translated.submerged_mask);
        assert_eq!(base.water_depth_m, translated.water_depth_m);
    }

    #[test]
    fn analytical_single_cell_basin_solves_known_level() {
        let topology = build_icosphere(1).unwrap();
        let count = topology.metrics().sample_count as usize;
        let mut surface = vec![1_000.0_f32; count];
        surface[0] = -1_000.0;
        let mut planet = PlanetPhysicalParameters::earthlike_reference();
        let physical_cell_area_m2 =
            topology.dual_area_steradians()[0] * planet.radius_m * planet.radius_m;
        let target_volume_m3 = 100.0 * physical_cell_area_m2;
        planet.surface_water_mass_kg = target_volume_m3 * planet.ocean_water_density_kg_per_m3;
        let state = solve_hydrostatic_surface_water(&topology, &surface, planet).unwrap();
        assert!((state.metrics.sea_level_m.unwrap() + 900.0).abs() < 1.0e-9);
        assert_eq!(state.metrics.submerged_sample_count, 1);
        assert!((f64::from(state.water_depth_m[0]) - 100.0).abs() < 1.0e-5);
        assert!(state.water_depth_m[1..].iter().all(|value| *value == 0.0));
    }

    #[test]
    fn connected_ocean_does_not_teleport_into_closed_lowland() {
        let topology = build_icosphere(1).unwrap();
        let count = topology.metrics().sample_count as usize;
        let seed = 0usize;
        let seed_neighbors = topology.neighbors(seed as u32);
        let closed = (1..count)
            .find(|sample| !seed_neighbors.contains(&(*sample as u32)))
            .expect("level-1 sphere must have a sample outside the seed neighborhood");
        let mut surface = vec![2_000.0_f32; count];
        surface[seed] = -1_000.0;
        surface[closed] = -1_200.0;
        let mut ocean_seed_mask = vec![0_u8; count];
        ocean_seed_mask[seed] = 1;
        let mut planet = PlanetPhysicalParameters::earthlike_reference();
        let physical_cell_area_m2 =
            topology.dual_area_steradians()[seed] * planet.radius_m * planet.radius_m;
        let target_volume_m3 = 100.0 * physical_cell_area_m2;
        planet.surface_water_mass_kg = target_volume_m3 * planet.ocean_water_density_kg_per_m3;

        let state = solve_hydrostatic_surface_water_connected(
            &topology,
            &surface,
            planet,
            &ocean_seed_mask,
        )
        .unwrap();
        assert_eq!(state.submerged_mask[seed], 1);
        assert_eq!(state.submerged_mask[closed], 0);
        assert_eq!(state.water_depth_m[closed], 0.0);
        assert!(state.elevation_above_sea_level_m[closed] < 0.0);
        assert!(state.metrics.water_volume_relative_error < 1.0e-10);
    }
}
