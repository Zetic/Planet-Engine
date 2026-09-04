use crate::{
    derive_stage_seed, GeodesicTopology, PlanetPhysicalParameters, PlanetTopology, StageIdentity,
    TopographyState, WorldgenError, INVALID_SAMPLE_ID,
};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};

pub const DRAINAGE_STAGE_ID: &str = "hydrology:drainage-topology";
pub const DRAINAGE_STAGE_VERSION: u32 = 1;
const DRAINAGE_NAMESPACE: &str = "hydrology:drainage-topology:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const ELEVATION_EPSILON_M: f64 = 1.0e-6;
const DEPRESSION_EPSILON_M: f64 = 1.0e-4;

pub const DRAINAGE_OUTLET_NONE: u8 = 0;
pub const DRAINAGE_OUTLET_OCEAN: u8 = 1;
pub const DRAINAGE_OUTLET_INTERNAL: u8 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrainageRequest {
    pub seed: String,
}

impl DrainageRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrainageBasin {
    pub id: u32,
    pub outlet_sample: u32,
    pub outlet_kind: u8,
    pub sample_count: u32,
    pub area_m2: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrainageDepression {
    pub id: u32,
    pub sample_count: u32,
    pub area_m2: f64,
    pub floor_sample: u32,
    pub floor_elevation_m: f64,
    pub spill_elevation_m: f64,
    pub maximum_depth_m: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrainageMetrics {
    pub sample_count: u32,
    pub land_sample_count: u32,
    pub ocean_sample_count: u32,
    pub basin_count: u32,
    pub depression_count: u32,
    pub depression_sample_count: u32,
    pub land_area_m2: f64,
    pub terminal_contributing_area_m2: f64,
    pub area_conservation_relative_error: f64,
    pub maximum_contributing_area_m2: f64,
    pub maximum_depression_depth_m: f64,
    pub drainage_hash: u64,
}

impl DrainageMetrics {
    pub fn drainage_hash_hex(&self) -> String {
        format!("{:016x}", self.drainage_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrainageState {
    pub stage: StageIdentity,
    pub metrics: DrainageMetrics,
    pub receiver: Vec<u32>,
    pub outlet_sample: Vec<u32>,
    pub outlet_kind: Vec<u8>,
    pub basin_id: Vec<u32>,
    pub depression_id: Vec<u32>,
    pub hydrologic_escape_elevation_m: Vec<f32>,
    pub depression_depth_m: Vec<f32>,
    pub contributing_area_m2: Vec<f64>,
    pub drainage_order: Vec<u32>,
    pub basins: Vec<DrainageBasin>,
    pub depressions: Vec<DrainageDepression>,
}

#[derive(Clone, Copy, Debug)]
struct FloodEntry {
    elevation_m: f64,
    sample: u32,
}

impl PartialEq for FloodEntry {
    fn eq(&self, other: &Self) -> bool {
        self.elevation_m.to_bits() == other.elevation_m.to_bits() && self.sample == other.sample
    }
}
impl Eq for FloodEntry {}
impl PartialOrd for FloodEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FloodEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .elevation_m
            .total_cmp(&self.elevation_m)
            .then_with(|| other.sample.cmp(&self.sample))
    }
}

#[derive(Debug)]
struct DrainageCore {
    receiver: Vec<u32>,
    outlet_sample: Vec<u32>,
    outlet_kind: Vec<u8>,
    basin_id: Vec<u32>,
    depression_id: Vec<u32>,
    escape_elevation_m: Vec<f64>,
    depression_depth_m: Vec<f64>,
    contributing_area_m2: Vec<f64>,
    drainage_order: Vec<u32>,
    basins: Vec<DrainageBasin>,
    depressions: Vec<DrainageDepression>,
    land_area_m2: f64,
    terminal_contributing_area_m2: f64,
    area_conservation_relative_error: f64,
    maximum_contributing_area_m2: f64,
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn validate_inputs<T: PlanetTopology>(
    topology: &T,
    elevation_m: &[f32],
    submerged_mask: &[u8],
    radius_m: f64,
) -> Result<(), &'static str> {
    let count = topology.sample_count() as usize;
    if count == 0 {
        return Err("drainage topology requires at least one sample");
    }
    if elevation_m.len() != count || submerged_mask.len() != count {
        return Err("drainage inputs must align with topology sample count");
    }
    if !radius_m.is_finite() || radius_m <= 0.0 {
        return Err("drainage planet radius must be finite and positive");
    }
    if elevation_m.iter().any(|value| !value.is_finite()) {
        return Err("drainage elevation field must be finite");
    }
    if submerged_mask.iter().any(|value| *value > 1) {
        return Err("drainage submerged mask must contain only 0 or 1");
    }
    for sample in 0..topology.sample_count() {
        if topology.neighbors(sample).len() != topology.neighbor_arc_lengths_rad(sample).len() {
            return Err("drainage topology neighbor geometry is misaligned");
        }
    }
    Ok(())
}

fn priority_flood<T: PlanetTopology>(
    topology: &T,
    elevation_m: &[f32],
    submerged_mask: &[u8],
    sea_level_m: Option<f64>,
) -> (Vec<f64>, Vec<u32>, Vec<u32>, Vec<u8>) {
    let count = topology.sample_count() as usize;
    let mut escape = vec![f64::INFINITY; count];
    let mut parent = vec![INVALID_SAMPLE_ID; count];
    let mut flood_rank = vec![u32::MAX; count];
    let mut outlet_kind = vec![DRAINAGE_OUTLET_NONE; count];
    let mut seen = vec![false; count];
    let mut queue = BinaryHeap::new();

    let mut ocean_seed_count = 0usize;
    for i in 0..count {
        if submerged_mask[i] != 0 {
            seen[i] = true;
            escape[i] = sea_level_m.unwrap_or_else(|| f64::from(elevation_m[i]));
            outlet_kind[i] = DRAINAGE_OUTLET_OCEAN;
            queue.push(FloodEntry {
                elevation_m: escape[i],
                sample: i as u32,
            });
            ocean_seed_count += 1;
        }
    }

    if ocean_seed_count == 0 {
        let mut minimum = 0usize;
        for i in 1..count {
            let a = f64::from(elevation_m[i]);
            let b = f64::from(elevation_m[minimum]);
            if a < b || (a.to_bits() == b.to_bits() && i < minimum) {
                minimum = i;
            }
        }
        seen[minimum] = true;
        escape[minimum] = f64::from(elevation_m[minimum]);
        outlet_kind[minimum] = DRAINAGE_OUTLET_INTERNAL;
        queue.push(FloodEntry {
            elevation_m: escape[minimum],
            sample: minimum as u32,
        });
    }

    let mut next_rank = 0u32;
    while let Some(entry) = queue.pop() {
        let sample = entry.sample as usize;
        flood_rank[sample] = next_rank;
        next_rank += 1;
        for &neighbor in topology.neighbors(entry.sample) {
            let ni = neighbor as usize;
            if seen[ni] {
                continue;
            }
            seen[ni] = true;
            let physical = f64::from(elevation_m[ni]);
            escape[ni] = physical.max(entry.elevation_m);
            parent[ni] = entry.sample;
            queue.push(FloodEntry {
                elevation_m: escape[ni],
                sample: neighbor,
            });
        }
    }

    (escape, parent, flood_rank, outlet_kind)
}

fn build_receivers<T: PlanetTopology>(
    topology: &T,
    submerged_mask: &[u8],
    escape: &[f64],
    flood_parent: &[u32],
    flood_rank: &[u32],
    outlet_kind: &[u8],
    radius_m: f64,
) -> Result<Vec<u32>, &'static str> {
    let count = topology.sample_count() as usize;
    let mut receiver = vec![INVALID_SAMPLE_ID; count];

    for i in 0..count {
        if submerged_mask[i] != 0 || outlet_kind[i] == DRAINAGE_OUTLET_INTERNAL {
            continue;
        }
        let sample = i as u32;
        let neighbors = topology.neighbors(sample);
        let distances = topology.neighbor_arc_lengths_rad(sample);
        let mut best_neighbor = INVALID_SAMPLE_ID;
        let mut best_slope = f64::NEG_INFINITY;

        for (k, &neighbor) in neighbors.iter().enumerate() {
            let ni = neighbor as usize;
            let drop_m = escape[i] - escape[ni];
            if drop_m <= ELEVATION_EPSILON_M && submerged_mask[ni] == 0 {
                continue;
            }
            let distance_m = distances[k] * radius_m;
            if !distance_m.is_finite() || distance_m <= 0.0 {
                return Err("drainage edge distance must be finite and positive");
            }
            let slope = drop_m.max(0.0) / distance_m;
            if slope > best_slope
                || (slope.to_bits() == best_slope.to_bits() && neighbor < best_neighbor)
            {
                best_slope = slope;
                best_neighbor = neighbor;
            }
        }

        if best_neighbor != INVALID_SAMPLE_ID {
            receiver[i] = best_neighbor;
            continue;
        }

        let parent = flood_parent[i];
        if parent == INVALID_SAMPLE_ID {
            return Err("non-terminal land sample has no hydrologic escape parent");
        }
        if escape[parent as usize] > escape[i] + ELEVATION_EPSILON_M {
            return Err("drainage escape parent rises above routed surface");
        }
        if escape[parent as usize].to_bits() == escape[i].to_bits()
            && flood_rank[parent as usize] >= flood_rank[i]
        {
            return Err("drainage flat routing does not decrease flood rank");
        }
        receiver[i] = parent;
    }

    Ok(receiver)
}

fn build_drainage_order(
    submerged_mask: &[u8],
    receiver: &[u32],
    escape: &[f64],
    flood_rank: &[u32],
) -> Result<Vec<u32>, &'static str> {
    let mut order = (0..receiver.len())
        .filter(|&i| submerged_mask[i] == 0)
        .map(|i| i as u32)
        .collect::<Vec<_>>();
    order.sort_by(|a, b| {
        escape[*b as usize]
            .total_cmp(&escape[*a as usize])
            .then_with(|| flood_rank[*b as usize].cmp(&flood_rank[*a as usize]))
            .then_with(|| b.cmp(a))
    });

    let mut position = vec![u32::MAX; receiver.len()];
    for (rank, &sample) in order.iter().enumerate() {
        position[sample as usize] = rank as u32;
    }
    for &sample in &order {
        let r = receiver[sample as usize];
        if r == INVALID_SAMPLE_ID || submerged_mask[r as usize] != 0 {
            continue;
        }
        if position[sample as usize] >= position[r as usize] {
            return Err("drainage receiver graph is not acyclic in processing order");
        }
    }
    Ok(order)
}

fn build_depressions<T: PlanetTopology>(
    topology: &T,
    elevation_m: &[f32],
    submerged_mask: &[u8],
    escape: &[f64],
    radius_m: f64,
) -> (Vec<u32>, Vec<f64>, Vec<DrainageDepression>) {
    let count = topology.sample_count() as usize;
    let mut depression_id = vec![INVALID_SAMPLE_ID; count];
    let mut depth = vec![0.0; count];
    let mut depressions = Vec::new();

    for i in 0..count {
        if submerged_mask[i] == 0 {
            depth[i] = (escape[i] - f64::from(elevation_m[i])).max(0.0);
        }
    }

    for start in 0..count {
        if submerged_mask[start] != 0
            || depth[start] <= DEPRESSION_EPSILON_M
            || depression_id[start] != INVALID_SAMPLE_ID
        {
            continue;
        }
        let id = depressions.len() as u32;
        let mut queue = VecDeque::new();
        queue.push_back(start as u32);
        depression_id[start] = id;
        let mut sample_count = 0u32;
        let mut area_m2 = 0.0;
        let mut floor_sample = start as u32;
        let mut floor_elevation_m = f64::from(elevation_m[start]);
        let mut spill_elevation_m = escape[start];
        let mut maximum_depth_m = depth[start];

        while let Some(sample) = queue.pop_front() {
            let i = sample as usize;
            sample_count += 1;
            area_m2 += topology.area_steradians(sample) * radius_m * radius_m;
            let physical = f64::from(elevation_m[i]);
            if physical < floor_elevation_m
                || (physical.to_bits() == floor_elevation_m.to_bits() && sample < floor_sample)
            {
                floor_sample = sample;
                floor_elevation_m = physical;
            }
            spill_elevation_m = spill_elevation_m.max(escape[i]);
            maximum_depth_m = maximum_depth_m.max(depth[i]);
            for &neighbor in topology.neighbors(sample) {
                let ni = neighbor as usize;
                if submerged_mask[ni] == 0
                    && depth[ni] > DEPRESSION_EPSILON_M
                    && depression_id[ni] == INVALID_SAMPLE_ID
                {
                    depression_id[ni] = id;
                    queue.push_back(neighbor);
                }
            }
        }

        depressions.push(DrainageDepression {
            id,
            sample_count,
            area_m2,
            floor_sample,
            floor_elevation_m,
            spill_elevation_m,
            maximum_depth_m,
        });
    }

    (depression_id, depth, depressions)
}

fn solve_core<T: PlanetTopology>(
    topology: &T,
    elevation_m: &[f32],
    submerged_mask: &[u8],
    radius_m: f64,
    sea_level_m: Option<f64>,
) -> Result<DrainageCore, &'static str> {
    validate_inputs(topology, elevation_m, submerged_mask, radius_m)?;
    let count = topology.sample_count() as usize;
    if submerged_mask.iter().any(|value| *value != 0)
        && sea_level_m.is_none_or(|value| !value.is_finite())
    {
        return Err("drainage ocean routing requires a finite solved sea level");
    }
    let (escape, flood_parent, flood_rank, mut outlet_kind) =
        priority_flood(topology, elevation_m, submerged_mask, sea_level_m);
    let receiver = build_receivers(
        topology,
        submerged_mask,
        &escape,
        &flood_parent,
        &flood_rank,
        &outlet_kind,
        radius_m,
    )?;
    let drainage_order = build_drainage_order(submerged_mask, &receiver, &escape, &flood_rank)?;
    let (depression_id, depression_depth_m, depressions) =
        build_depressions(topology, elevation_m, submerged_mask, &escape, radius_m);

    let mut contributing_area_m2 = vec![0.0; count];
    let mut land_area_m2 = 0.0;
    for i in 0..count {
        if submerged_mask[i] == 0 {
            let area = topology.area_steradians(i as u32) * radius_m * radius_m;
            contributing_area_m2[i] = area;
            land_area_m2 += area;
        }
    }
    for &sample in &drainage_order {
        let r = receiver[sample as usize];
        if r != INVALID_SAMPLE_ID {
            contributing_area_m2[r as usize] += contributing_area_m2[sample as usize];
        }
    }

    let mut outlet_sample = vec![INVALID_SAMPLE_ID; count];
    for i in 0..count {
        if submerged_mask[i] != 0 {
            outlet_sample[i] = i as u32;
            outlet_kind[i] = DRAINAGE_OUTLET_OCEAN;
        } else if receiver[i] == INVALID_SAMPLE_ID {
            outlet_sample[i] = i as u32;
            if outlet_kind[i] == DRAINAGE_OUTLET_NONE {
                outlet_kind[i] = DRAINAGE_OUTLET_INTERNAL;
            }
        }
    }
    for &sample in drainage_order.iter().rev() {
        let i = sample as usize;
        if outlet_sample[i] != INVALID_SAMPLE_ID {
            continue;
        }
        let r = receiver[i];
        if r == INVALID_SAMPLE_ID {
            return Err("drainage terminal outlet propagation failed");
        }
        let resolved = outlet_sample[r as usize];
        if resolved == INVALID_SAMPLE_ID {
            return Err("drainage downstream outlet was not resolved first");
        }
        outlet_sample[i] = resolved;
    }

    for i in 0..count {
        if submerged_mask[i] != 0 {
            continue;
        }
        let outlet = outlet_sample[i];
        let resolved_kind = outlet_kind[outlet as usize];
        if resolved_kind == DRAINAGE_OUTLET_NONE {
            return Err("drainage outlet kind was not resolved at terminal sample");
        }
        outlet_kind[i] = resolved_kind;
    }

    let mut sorted_outlets = outlet_sample
        .iter()
        .enumerate()
        .filter(|(sample, _)| submerged_mask[*sample] == 0)
        .map(|(_, outlet)| *outlet)
        .collect::<Vec<_>>();
    sorted_outlets.sort_unstable();
    sorted_outlets.dedup();
    let unique_outlets = sorted_outlets
        .into_iter()
        .enumerate()
        .map(|(id, outlet)| (outlet, id as u32))
        .collect::<BTreeMap<_, _>>();
    let mut basin_id = vec![INVALID_SAMPLE_ID; count];
    let mut basins = unique_outlets
        .iter()
        .map(|(&outlet, &id)| DrainageBasin {
            id,
            outlet_sample: outlet,
            outlet_kind: outlet_kind[outlet as usize],
            sample_count: 0,
            area_m2: 0.0,
        })
        .collect::<Vec<_>>();
    basins.sort_by_key(|basin| basin.id);
    for i in 0..count {
        if submerged_mask[i] != 0 {
            continue;
        }
        let id = unique_outlets[&outlet_sample[i]];
        basin_id[i] = id;
        basins[id as usize].sample_count += 1;
        basins[id as usize].area_m2 += topology.area_steradians(i as u32) * radius_m * radius_m;
    }

    let mut terminal_contributing_area_m2 = 0.0;
    for i in 0..count {
        if submerged_mask[i] != 0 || receiver[i] == INVALID_SAMPLE_ID {
            terminal_contributing_area_m2 += contributing_area_m2[i];
        }
    }
    let area_conservation_relative_error = if land_area_m2 > 0.0 {
        (terminal_contributing_area_m2 - land_area_m2).abs() / land_area_m2
    } else {
        0.0
    };
    let maximum_contributing_area_m2 = contributing_area_m2.iter().copied().fold(0.0_f64, f64::max);

    Ok(DrainageCore {
        receiver,
        outlet_sample,
        outlet_kind,
        basin_id,
        depression_id,
        escape_elevation_m: escape,
        depression_depth_m,
        contributing_area_m2,
        drainage_order,
        basins,
        depressions,
        land_area_m2,
        terminal_contributing_area_m2,
        area_conservation_relative_error,
        maximum_contributing_area_m2,
    })
}

pub fn generate_drainage_topology(
    topology: &GeodesicTopology,
    topography: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &DrainageRequest,
) -> Result<DrainageState, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    if topography.metrics.sample_count != topology.sample_count() {
        return Err(WorldgenError::InvalidHydrology(
            "drainage topography must align with canonical topology",
        ));
    }
    let core = solve_core(
        topology,
        &topography.solid_elevation_m,
        &topography.submerged_mask,
        planet.radius_m,
        topography.metrics.sea_level_m,
    )
    .map_err(WorldgenError::InvalidHydrology)?;

    let stage_seed = derive_stage_seed(&request.seed, DRAINAGE_NAMESPACE);
    let mut drainage_hash = FNV_OFFSET_BASIS;
    drainage_hash = fnv_update(drainage_hash, DRAINAGE_STAGE_ID.as_bytes());
    drainage_hash = fnv_update(drainage_hash, &DRAINAGE_STAGE_VERSION.to_le_bytes());
    drainage_hash = fnv_update(drainage_hash, &stage_seed.to_le_bytes());
    drainage_hash = fnv_update(drainage_hash, &planet.parameter_hash().to_le_bytes());
    drainage_hash = fnv_update(
        drainage_hash,
        &topography.metrics.topography_hash.to_le_bytes(),
    );
    for &value in &core.receiver {
        drainage_hash = fnv_update(drainage_hash, &value.to_le_bytes());
    }
    for &value in &core.outlet_sample {
        drainage_hash = fnv_update(drainage_hash, &value.to_le_bytes());
    }
    for &value in &core.basin_id {
        drainage_hash = fnv_update(drainage_hash, &value.to_le_bytes());
    }
    for &value in &core.depression_id {
        drainage_hash = fnv_update(drainage_hash, &value.to_le_bytes());
    }
    for &value in &core.escape_elevation_m {
        drainage_hash = fnv_update(drainage_hash, &value.to_bits().to_le_bytes());
    }
    for &value in &core.contributing_area_m2 {
        drainage_hash = fnv_update(drainage_hash, &value.to_bits().to_le_bytes());
    }

    let depression_sample_count = core
        .depression_id
        .iter()
        .filter(|value| **value != INVALID_SAMPLE_ID)
        .count() as u32;
    let maximum_depression_depth_m = core
        .depression_depth_m
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let land_sample_count = topography
        .submerged_mask
        .iter()
        .filter(|value| **value == 0)
        .count() as u32;
    let ocean_sample_count = topology.sample_count() - land_sample_count;

    Ok(DrainageState {
        stage: StageIdentity {
            id: DRAINAGE_STAGE_ID,
            version: DRAINAGE_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics: DrainageMetrics {
            sample_count: topology.sample_count(),
            land_sample_count,
            ocean_sample_count,
            basin_count: core.basins.len() as u32,
            depression_count: core.depressions.len() as u32,
            depression_sample_count,
            land_area_m2: core.land_area_m2,
            terminal_contributing_area_m2: core.terminal_contributing_area_m2,
            area_conservation_relative_error: core.area_conservation_relative_error,
            maximum_contributing_area_m2: core.maximum_contributing_area_m2,
            maximum_depression_depth_m,
            drainage_hash,
        },
        receiver: core.receiver,
        outlet_sample: core.outlet_sample,
        outlet_kind: core.outlet_kind,
        basin_id: core.basin_id,
        depression_id: core.depression_id,
        hydrologic_escape_elevation_m: core
            .escape_elevation_m
            .into_iter()
            .map(|value| value as f32)
            .collect(),
        depression_depth_m: core
            .depression_depth_m
            .into_iter()
            .map(|value| value as f32)
            .collect(),
        contributing_area_m2: core.contributing_area_m2,
        drainage_order: core.drainage_order,
        basins: core.basins,
        depressions: core.depressions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTopology {
        neighbors: Vec<Vec<u32>>,
        distances: Vec<Vec<f64>>,
        areas: Vec<f64>,
    }

    impl TestTopology {
        fn chain(count: usize) -> Self {
            let mut neighbors = vec![Vec::new(); count];
            let mut distances = vec![Vec::new(); count];
            for i in 0..count - 1 {
                neighbors[i].push((i + 1) as u32);
                distances[i].push(1.0e-3);
                neighbors[i + 1].push(i as u32);
                distances[i + 1].push(1.0e-3);
            }
            Self {
                neighbors,
                distances,
                areas: vec![1.0; count],
            }
        }
    }

    impl PlanetTopology for TestTopology {
        fn sample_count(&self) -> u32 {
            self.neighbors.len() as u32
        }
        fn unit_position(&self, _sample: u32) -> [f64; 3] {
            [1.0, 0.0, 0.0]
        }
        fn area_steradians(&self, sample: u32) -> f64 {
            self.areas[sample as usize]
        }
        fn neighbors(&self, sample: u32) -> &[u32] {
            &self.neighbors[sample as usize]
        }
        fn neighbor_arc_lengths_rad(&self, sample: u32) -> &[f64] {
            &self.distances[sample as usize]
        }
        fn neighbor_interface_arc_lengths_rad(&self, sample: u32) -> &[f64] {
            &self.distances[sample as usize]
        }
    }

    #[test]
    fn simple_slope_routes_monotonically_to_ocean() {
        let topology = TestTopology::chain(4);
        let result = solve_core(
            &topology,
            &[3.0, 2.0, 1.0, -1.0],
            &[0, 0, 0, 1],
            1.0,
            Some(0.0),
        )
        .unwrap();
        assert_eq!(result.receiver, vec![1, 2, 3, INVALID_SAMPLE_ID]);
        assert_eq!(result.outlet_sample[0], 3);
        assert_eq!(result.outlet_kind[0], DRAINAGE_OUTLET_OCEAN);
        assert_eq!(result.outlet_kind[1], DRAINAGE_OUTLET_OCEAN);
        assert_eq!(result.outlet_kind[2], DRAINAGE_OUTLET_OCEAN);
        assert_eq!(result.basins.len(), 1);
        assert!(result.area_conservation_relative_error < 1.0e-12);
    }

    #[test]
    fn enclosed_bowl_records_escape_height_without_editing_physical_elevation() {
        let topology = TestTopology::chain(4);
        let elevation = [1.0, 5.0, 4.0, -1.0];
        let result = solve_core(&topology, &elevation, &[0, 0, 0, 1], 1.0, Some(0.0)).unwrap();
        assert!((result.escape_elevation_m[0] - 5.0).abs() < 1.0e-12);
        assert!((result.depression_depth_m[0] - 4.0).abs() < 1.0e-12);
        assert_eq!(result.receiver[0], 1);
        assert_eq!(result.depressions.len(), 1);
        assert!((result.depressions[0].spill_elevation_m - 5.0).abs() < 1.0e-12);
        assert!((f64::from(elevation[0]) - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn flat_plateau_uses_flood_rank_to_remain_acyclic() {
        let topology = TestTopology::chain(4);
        let result = solve_core(
            &topology,
            &[2.0, 2.0, 2.0, -1.0],
            &[0, 0, 0, 1],
            1.0,
            Some(0.0),
        )
        .unwrap();
        assert_eq!(result.receiver, vec![1, 2, 3, INVALID_SAMPLE_ID]);
        assert_eq!(result.drainage_order, vec![0, 1, 2]);
    }

    #[test]
    fn ocean_bathymetry_does_not_steer_coastal_land_routing() {
        let topology = TestTopology {
            neighbors: vec![vec![1, 2], vec![0], vec![0]],
            distances: vec![vec![1.0e-3, 1.0e-3], vec![1.0e-3], vec![1.0e-3]],
            areas: vec![1.0; 3],
        };
        let result = solve_core(
            &topology,
            &[10.0, -100.0, -5_000.0],
            &[0, 1, 1],
            1.0,
            Some(0.0),
        )
        .unwrap();
        assert_eq!(result.escape_elevation_m[1], 0.0);
        assert_eq!(result.escape_elevation_m[2], 0.0);
        assert_eq!(result.receiver[0], 1);
        assert_eq!(result.outlet_sample[0], 1);
        assert_eq!(result.outlet_kind[0], DRAINAGE_OUTLET_OCEAN);
    }

    #[test]
    fn dry_planet_uses_global_minimum_as_internal_terminal() {
        let topology = TestTopology::chain(4);
        let result =
            solve_core(&topology, &[4.0, 3.0, 1.0, 2.0], &[0, 0, 0, 0], 1.0, None).unwrap();
        assert_eq!(result.receiver[2], INVALID_SAMPLE_ID);
        assert_eq!(result.outlet_kind[2], DRAINAGE_OUTLET_INTERNAL);
        assert!(result
            .outlet_kind
            .iter()
            .all(|value| *value == DRAINAGE_OUTLET_INTERNAL));
        assert!(result.outlet_sample.iter().all(|value| *value == 2));
        assert!(result.area_conservation_relative_error < 1.0e-12);
    }
}
