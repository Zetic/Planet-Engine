use interlink_worldgen::{
    build_icosphere, generate_bounded_terrain_evolution, generate_coupled_climate_with_diagnostics,
    generate_crust_and_history, generate_drainage_topology, generate_fluvial_erosion_sediment,
    generate_initial_topography, generate_lake_sediment_infill, generate_lakes_closed_basins,
    generate_lithosphere, generate_post_erosion_hydrology, generate_runoff_discharge,
    generate_seasonal_hydrology, generate_tectonics, inherit_boundary_interfaces,
    inherit_physical_state, ClimateRequest, DrainageRequest, FluvialErosionRequest,
    GeologicalBoundaryRegime, GeologyRequest, LakeRequest, LakeSedimentInfillRequest,
    LithosphereRequest, PlanetPhysicalParameters, PostErosionHydrologyRequest, RunoffRequest,
    SeasonalHydrologyRequest, TectonicsRequest, TerrainEvolutionRequest, TopographyRequest,
};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

const CRUST_CONTINENTAL: u8 = 3;
const RIFT_CORRIDOR_RADIUS_M: f64 = 300_000.0;
const LARGE_RIFT_LAKE_AREA_M2: f64 = 100_000.0 * 1_000_000.0;

#[derive(Clone, Copy, Debug)]
struct QueueEntry {
    distance_m: f64,
    sample: u32,
}

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.distance_m.to_bits() == other.distance_m.to_bits() && self.sample == other.sample
    }
}
impl Eq for QueueEntry {}
impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance_m
            .total_cmp(&self.distance_m)
            .then_with(|| other.sample.cmp(&self.sample))
    }
}

#[derive(Clone, Debug, Default)]
struct RidgeIslandStats {
    component_count: u64,
    large_component_count: u64,
    total_area_m2: f64,
    maximum_area_m2: f64,
    ridge_land_endpoints: u64,
}

impl RidgeIslandStats {
    fn absorb(&mut self, other: &Self) {
        self.component_count += other.component_count;
        self.large_component_count += other.large_component_count;
        self.total_area_m2 += other.total_area_m2;
        self.maximum_area_m2 = self.maximum_area_m2.max(other.maximum_area_m2);
        self.ridge_land_endpoints += other.ridge_land_endpoints;
    }
}

#[derive(Clone, Debug, Default)]
struct RiftLakeStats {
    lake_count: u64,
    rift_lake_count: u64,
    large_rift_lake_count: u64,
    total_lake_area_m2: f64,
    rift_lake_area_m2: f64,
    maximum_rift_lake_area_m2: f64,
}

impl RiftLakeStats {
    fn absorb(&mut self, other: &Self) {
        self.lake_count += other.lake_count;
        self.rift_lake_count += other.rift_lake_count;
        self.large_rift_lake_count += other.large_rift_lake_count;
        self.total_lake_area_m2 += other.total_lake_area_m2;
        self.rift_lake_area_m2 += other.rift_lake_area_m2;
        self.maximum_rift_lake_area_m2 = self.maximum_rift_lake_area_m2.max(other.maximum_rift_lake_area_m2);
    }
}

fn rift_distances(
    topology: &interlink_worldgen::GeodesicTopology,
    boundaries: &interlink_worldgen::InheritedBoundarySet,
    radius_m: f64,
) -> Vec<f64> {
    let count = topology.metrics().sample_count as usize;
    let mut distances = vec![f64::INFINITY; count];
    let mut queue = BinaryHeap::new();
    for edge in &boundaries.boundaries {
        if edge.geological_regime != GeologicalBoundaryRegime::ContinentalRift {
            continue;
        }
        for sample in [edge.sample_a, edge.sample_b] {
            let index = sample as usize;
            if distances[index] > 0.0 {
                distances[index] = 0.0;
                queue.push(QueueEntry { distance_m: 0.0, sample });
            }
        }
    }
    while let Some(entry) = queue.pop() {
        let index = entry.sample as usize;
        if entry.distance_m > distances[index] + 1.0e-6 {
            continue;
        }
        if entry.distance_m > RIFT_CORRIDOR_RADIUS_M {
            continue;
        }
        for (neighbor, arc) in topology
            .neighbors_of(entry.sample)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(entry.sample).iter())
        {
            let candidate = entry.distance_m + *arc * radius_m;
            let target = *neighbor as usize;
            if candidate + 1.0e-6 < distances[target] && candidate <= RIFT_CORRIDOR_RADIUS_M {
                distances[target] = candidate;
                queue.push(QueueEntry { distance_m: candidate, sample: *neighbor });
            }
        }
    }
    distances
}

fn ridge_island_stats(
    topology: &interlink_worldgen::GeodesicTopology,
    inherited: &interlink_worldgen::InheritedPhysicalState,
    boundaries: &interlink_worldgen::InheritedBoundarySet,
    terrain: &interlink_worldgen::TopographyState,
    radius_m: f64,
) -> RidgeIslandStats {
    let count = topology.metrics().sample_count as usize;
    let mut ridge_endpoint = vec![false; count];
    for edge in &boundaries.boundaries {
        if edge.geological_regime == GeologicalBoundaryRegime::OceanicRidge {
            ridge_endpoint[edge.sample_a as usize] = true;
            ridge_endpoint[edge.sample_b as usize] = true;
        }
    }

    let mut visited = vec![false; count];
    let mut stats = RidgeIslandStats::default();
    for start in 0..count {
        if visited[start] || terrain.submerged_mask[start] != 0 {
            continue;
        }
        let mut stack = vec![start as u32];
        visited[start] = true;
        let mut area_m2 = 0.0_f64;
        let mut has_continental = false;
        let mut touches_ridge = false;
        let mut ridge_endpoints = 0_u64;
        while let Some(sample) = stack.pop() {
            let i = sample as usize;
            area_m2 += topology.dual_area_steradians()[i] * radius_m * radius_m;
            has_continental |= inherited.crust_kind[i] == CRUST_CONTINENTAL;
            if ridge_endpoint[i] {
                touches_ridge = true;
                ridge_endpoints += 1;
            }
            for neighbor in topology.neighbors_of(sample) {
                let n = *neighbor as usize;
                if !visited[n] && terrain.submerged_mask[n] == 0 {
                    visited[n] = true;
                    stack.push(*neighbor);
                }
            }
        }
        if touches_ridge && !has_continental {
            stats.component_count += 1;
            stats.total_area_m2 += area_m2;
            stats.maximum_area_m2 = stats.maximum_area_m2.max(area_m2);
            stats.ridge_land_endpoints += ridge_endpoints;
            if area_m2 >= 25_000.0 * 1_000_000.0 {
                stats.large_component_count += 1;
            }
        }
    }
    stats
}

fn rift_lake_stats(
    topology: &interlink_worldgen::GeodesicTopology,
    inherited: &interlink_worldgen::InheritedPhysicalState,
    rift_distance_m: &[f64],
    lakes: &interlink_worldgen::LakeState,
    radius_m: f64,
) -> RiftLakeStats {
    let mut total_by_lake: HashMap<u32, f64> = HashMap::new();
    let mut rift_by_lake: HashMap<u32, f64> = HashMap::new();
    for i in 0..lakes.lake_fraction.len() {
        let fraction = f64::from(lakes.lake_fraction[i]);
        if fraction <= 0.0 {
            continue;
        }
        let id = lakes.lake_id[i];
        let area_m2 = topology.dual_area_steradians()[i] * radius_m * radius_m * fraction;
        *total_by_lake.entry(id).or_insert(0.0) += area_m2;
        if inherited.crust_kind[i] == CRUST_CONTINENTAL && rift_distance_m[i] <= RIFT_CORRIDOR_RADIUS_M {
            *rift_by_lake.entry(id).or_insert(0.0) += area_m2;
        }
    }

    let mut stats = RiftLakeStats {
        lake_count: total_by_lake.len() as u64,
        total_lake_area_m2: total_by_lake.values().sum(),
        ..RiftLakeStats::default()
    };
    for (id, total_area) in total_by_lake {
        let rift_area = rift_by_lake.get(&id).copied().unwrap_or(0.0);
        if total_area <= 0.0 || rift_area / total_area < 0.50 {
            continue;
        }
        stats.rift_lake_count += 1;
        stats.rift_lake_area_m2 += total_area;
        stats.maximum_rift_lake_area_m2 = stats.maximum_rift_lake_area_m2.max(total_area);
        if total_area >= LARGE_RIFT_LAKE_AREA_M2 {
            stats.large_rift_lake_count += 1;
        }
    }
    stats
}

fn main() -> Result<(), String> {
    let seeds = [
        "3",
        "interlink-wg7c",
        "wg4-boundary-a",
        "wg4-boundary-b",
        "wg4-boundary-c",
        "wg4-boundary-d",
    ];
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;

    let mut ridge_all = RidgeIslandStats::default();
    let mut rift_lakes_all = RiftLakeStats::default();

    for seed in seeds {
        let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, plates), planet)
            .map_err(|error| error.to_string())?;
        let geology = generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet)
            .map_err(|error| error.to_string())?;
        let lithosphere = generate_lithosphere(
            &coarse,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;
        let inherited = inherit_physical_state(
            &fine,
            coarse_level,
            &tectonics,
            &geology,
            &lithosphere,
            planet,
        )
        .map_err(|error| error.to_string())?;
        let boundaries = inherit_boundary_interfaces(
            &coarse,
            &fine,
            &tectonics,
            &geology,
            &inherited.plate_ids,
        )
        .map_err(|error| error.to_string())?;
        let terrain = generate_initial_topography(
            &fine,
            &inherited,
            &boundaries,
            planet,
            &TopographyRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;

        let ridge_stats = ridge_island_stats(&fine, &inherited, &boundaries, &terrain, planet.radius_m);
        let rift_distance_m = rift_distances(&fine, &boundaries, planet.radius_m);

        let climate_request = ClimateRequest::new(seed);
        let mut progress = |_completed_years: u8, _maximum_years: u8| {};
        let (climate, climate_diagnostics) = generate_coupled_climate_with_diagnostics(
            &fine,
            &terrain,
            planet,
            &climate_request,
            &mut progress,
        )
        .map_err(|error| error.to_string())?;
        let drainage = generate_drainage_topology(
            &fine,
            &terrain,
            planet,
            &DrainageRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;
        let runoff = generate_runoff_discharge(
            &fine,
            &terrain,
            &climate,
            &drainage,
            planet,
            &RunoffRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;
        let lakes = generate_lakes_closed_basins(
            &fine,
            &terrain,
            &climate,
            &drainage,
            &runoff,
            planet,
            &LakeRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;
        let seasonal = generate_seasonal_hydrology(
            &fine,
            &terrain,
            &climate,
            &climate_diagnostics,
            &drainage,
            &runoff,
            &lakes,
            planet,
            &SeasonalHydrologyRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;
        let erosion = generate_fluvial_erosion_sediment(
            &fine,
            &inherited,
            &terrain,
            &drainage,
            &lakes,
            &seasonal,
            planet,
            &FluvialErosionRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;
        let evolution = generate_bounded_terrain_evolution(
            &fine,
            &terrain,
            &drainage,
            &runoff,
            &lakes,
            &erosion,
            planet,
            &TerrainEvolutionRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;
        let reconciliation = generate_post_erosion_hydrology(
            &fine,
            &terrain,
            &climate,
            &climate_diagnostics,
            &drainage,
            &runoff,
            &lakes,
            &seasonal,
            &evolution,
            planet,
            &PostErosionHydrologyRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;
        let infill = generate_lake_sediment_infill(
            &fine,
            &terrain,
            &climate,
            &climate_diagnostics,
            &drainage,
            &lakes,
            &erosion,
            &evolution,
            &reconciliation,
            planet,
            &LakeSedimentInfillRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;

        let lake_stats = rift_lake_stats(
            &fine,
            &inherited,
            &rift_distance_m,
            &infill.reconciled_lakes,
            planet.radius_m,
        );

        println!(
            "{seed}: ridge_islands={} ridge_large={} ridge_area_km2={:.0} ridge_max_km2={:.0} ridge_land_endpoints={} final_lakes={} rift_lakes={} rift_large={} rift_lake_area_km2={:.0} rift_max_km2={:.0} total_lake_area_km2={:.0}",
            ridge_stats.component_count,
            ridge_stats.large_component_count,
            ridge_stats.total_area_m2 / 1.0e6,
            ridge_stats.maximum_area_m2 / 1.0e6,
            ridge_stats.ridge_land_endpoints,
            lake_stats.lake_count,
            lake_stats.rift_lake_count,
            lake_stats.large_rift_lake_count,
            lake_stats.rift_lake_area_m2 / 1.0e6,
            lake_stats.maximum_rift_lake_area_m2 / 1.0e6,
            lake_stats.total_lake_area_m2 / 1.0e6,
        );
        ridge_all.absorb(&ridge_stats);
        rift_lakes_all.absorb(&lake_stats);
    }

    println!(
        "AGGREGATE: ridge_islands={} ridge_large={} ridge_area_km2={:.0} ridge_max_km2={:.0} ridge_land_endpoints={} final_lakes={} rift_lakes={} rift_large={} rift_lake_area_km2={:.0} rift_max_km2={:.0} total_lake_area_km2={:.0} rift_share={:.2}%",
        ridge_all.component_count,
        ridge_all.large_component_count,
        ridge_all.total_area_m2 / 1.0e6,
        ridge_all.maximum_area_m2 / 1.0e6,
        ridge_all.ridge_land_endpoints,
        rift_lakes_all.lake_count,
        rift_lakes_all.rift_lake_count,
        rift_lakes_all.large_rift_lake_count,
        rift_lakes_all.rift_lake_area_m2 / 1.0e6,
        rift_lakes_all.maximum_rift_lake_area_m2 / 1.0e6,
        rift_lakes_all.total_lake_area_m2 / 1.0e6,
        100.0 * rift_lakes_all.rift_lake_area_m2 / rift_lakes_all.total_lake_area_m2.max(1.0),
    );

    Ok(())
}
