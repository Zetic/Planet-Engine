use crate::{
    random, PlanetPhysicalParameters, PlanetTopology, PlateBoundaryEdge, PlateBoundaryKind,
    StageIdentity, TectonicModel, WorldgenError,
};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};

pub const TECTONIC_HISTORY_STAGE_ID: &str = "tectonics:history-systems";
pub const TECTONIC_HISTORY_STAGE_VERSION: u32 = 1;
const TECTONIC_HISTORY_NAMESPACE: &str = "worldgen:tectonics:history-systems:v1";
const INVALID_SYSTEM_ID: u32 = u32::MAX;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TectonicHistoryRequest {
    pub seed: String,
}

impl TectonicHistoryRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoundaryHistoryState {
    pub boundary_index: u32,
    pub system_id: u32,
    pub along_strike_distance_km: f32,
    pub along_strike_fraction: f32,
    pub obliquity_deg: f32,
    pub curvature_deg: f32,
    pub event_age_myr: f32,
    pub cumulative_convergence_km: f32,
    pub cumulative_extension_km: f32,
    pub cumulative_shear_km: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TectonicBoundarySystem {
    pub id: u32,
    pub plate_low: u16,
    pub plate_high: u16,
    pub kind: PlateBoundaryKind,
    pub boundary_indices: Vec<u32>,
    pub event_age_myr: f32,
    pub length_km: f32,
    pub mean_normal_rate_m_per_year: f32,
    pub mean_shear_rate_m_per_year: f32,
    pub mean_obliquity_deg: f32,
    pub mean_curvature_deg: f32,
    pub cumulative_convergence_km: f32,
    pub cumulative_extension_km: f32,
    pub cumulative_shear_km: f32,
    pub endpoint_count: u16,
    pub junction_count: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TectonicHistoryMetrics {
    pub boundary_system_count: u32,
    pub convergent_system_count: u32,
    pub divergent_system_count: u32,
    pub transform_system_count: u32,
    pub maximum_event_age_myr: f64,
    pub maximum_cumulative_convergence_km: f64,
    pub maximum_cumulative_extension_km: f64,
    pub mean_obliquity_deg: f64,
    pub history_hash: u64,
}

impl TectonicHistoryMetrics {
    pub fn history_hash_hex(&self) -> String {
        format!("{:016x}", self.history_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TectonicHistoryModel {
    pub stage: StageIdentity,
    pub systems: Vec<TectonicBoundarySystem>,
    pub boundary_state: Vec<BoundaryHistoryState>,
    pub metrics: TectonicHistoryMetrics,
}

impl TectonicHistoryModel {
    pub fn boundary_system_ids(&self) -> Vec<u32> {
        self.boundary_state
            .iter()
            .map(|state| state.system_id)
            .collect()
    }

    pub fn boundary_event_ages_myr(&self) -> Vec<f32> {
        self.boundary_state
            .iter()
            .map(|state| state.event_age_myr)
            .collect()
    }

    pub fn boundary_cumulative_convergence_km(&self) -> Vec<f32> {
        self.boundary_state
            .iter()
            .map(|state| state.cumulative_convergence_km)
            .collect()
    }

    pub fn boundary_along_strike_fraction(&self) -> Vec<f32> {
        self.boundary_state
            .iter()
            .map(|state| state.along_strike_fraction)
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct DistanceFrontier {
    distance_m: f64,
    edge: usize,
}

impl PartialEq for DistanceFrontier {
    fn eq(&self, other: &Self) -> bool {
        self.distance_m.to_bits() == other.distance_m.to_bits() && self.edge == other.edge
    }
}
impl Eq for DistanceFrontier {}
impl Ord for DistanceFrontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance_m
            .total_cmp(&self.distance_m)
            .then_with(|| other.edge.cmp(&self.edge))
    }
}
impl PartialOrd for DistanceFrontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: [f64; 3], factor: f64) -> [f64; 3] {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}
fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}
fn normalize(a: [f64; 3]) -> [f64; 3] {
    let magnitude = norm(a).max(1.0e-15);
    scale(a, 1.0 / magnitude)
}
fn arc_radians(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(a, b).clamp(-1.0, 1.0).acos()
}
fn plate_pair(edge: &PlateBoundaryEdge) -> (u16, u16) {
    if edge.plate_a <= edge.plate_b {
        (edge.plate_a, edge.plate_b)
    } else {
        (edge.plate_b, edge.plate_a)
    }
}
fn compatible(a: &PlateBoundaryEdge, b: &PlateBoundaryEdge) -> bool {
    a.kind == b.kind && plate_pair(a) == plate_pair(b)
}
fn boundary_midpoint<T: PlanetTopology>(topology: &T, edge: &PlateBoundaryEdge) -> [f64; 3] {
    normalize(add(
        topology.unit_position(edge.sample_a),
        topology.unit_position(edge.sample_b),
    ))
}
fn boundary_tangent<T: PlanetTopology>(topology: &T, edge: &PlateBoundaryEdge) -> [f64; 3] {
    let a = topology.unit_position(edge.sample_a);
    let b = topology.unit_position(edge.sample_b);
    let midpoint = normalize(add(a, b));
    let normal = normalize(sub(b, a));
    normalize(cross(midpoint, normal))
}

fn build_boundary_adjacency<T: PlanetTopology>(
    topology: &T,
    boundaries: &[PlateBoundaryEdge],
) -> Vec<Vec<usize>> {
    let mut incident = BTreeMap::<u32, Vec<usize>>::new();
    for (index, edge) in boundaries.iter().enumerate() {
        incident.entry(edge.sample_a).or_default().push(index);
        incident.entry(edge.sample_b).or_default().push(index);
    }
    let mut adjacency = vec![Vec::<usize>::new(); boundaries.len()];
    for (index, edge) in boundaries.iter().enumerate() {
        let mut candidate_samples = vec![edge.sample_a, edge.sample_b];
        candidate_samples.extend(topology.neighbors(edge.sample_a).iter().copied());
        candidate_samples.extend(topology.neighbors(edge.sample_b).iter().copied());
        candidate_samples.sort_unstable();
        candidate_samples.dedup();
        for sample in candidate_samples {
            let Some(candidates) = incident.get(&sample) else {
                continue;
            };
            for &other in candidates {
                if other != index && compatible(edge, &boundaries[other]) {
                    adjacency[index].push(other);
                }
            }
        }
        adjacency[index].sort_unstable();
        adjacency[index].dedup();
    }
    adjacency
}

fn connected_systems(
    boundaries: &[PlateBoundaryEdge],
    adjacency: &[Vec<usize>],
) -> Vec<Vec<usize>> {
    let mut visited = vec![false; boundaries.len()];
    let mut systems = Vec::new();
    for start in 0..boundaries.len() {
        if visited[start] {
            continue;
        }
        let mut queue = VecDeque::new();
        let mut component = Vec::new();
        queue.push_back(start);
        visited[start] = true;
        while let Some(edge) = queue.pop_front() {
            component.push(edge);
            for &neighbor in &adjacency[edge] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        component.sort_unstable();
        systems.push(component);
    }
    systems.sort_by_key(|component| component[0]);
    systems
}

fn event_age_myr(stage_seed: u64, system_id: u32, edge: &PlateBoundaryEdge) -> f64 {
    let (low, high) = plate_pair(edge);
    let kind = edge.kind as u64;
    let stream = stage_seed
        ^ u64::from(low).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ u64::from(high).wrapping_mul(0xbf58_476d_1ce4_e5b9)
        ^ u64::from(system_id).wrapping_mul(0x94d0_49bb_1331_11eb)
        ^ kind.wrapping_mul(0xd6e8_feb8_6659_fd93);
    let unit = ((random::mix64(stream) >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0);
    match edge.kind {
        PlateBoundaryKind::Convergent => 8.0 + 52.0 * unit.powf(1.35),
        PlateBoundaryKind::Divergent => 4.0 + 96.0 * unit.powf(1.45),
        PlateBoundaryKind::Transform => 3.0 + 65.0 * unit.powf(1.25),
    }
}

fn root_edge(component: &[usize], adjacency: &[Vec<usize>]) -> usize {
    component
        .iter()
        .copied()
        .min_by_key(|edge| {
            let degree = adjacency[*edge]
                .iter()
                .filter(|candidate| component.binary_search(candidate).is_ok())
                .count();
            (degree, *edge)
        })
        .unwrap_or(component[0])
}

fn along_strike_distances<T: PlanetTopology>(
    topology: &T,
    boundaries: &[PlateBoundaryEdge],
    adjacency: &[Vec<usize>],
    component: &[usize],
    planet_radius_m: f64,
) -> Vec<f64> {
    let mut in_component = vec![false; boundaries.len()];
    for &edge in component {
        in_component[edge] = true;
    }
    let root = root_edge(component, adjacency);
    let mut distances = vec![f64::INFINITY; boundaries.len()];
    let mut frontier = BinaryHeap::new();
    distances[root] = 0.0;
    frontier.push(DistanceFrontier {
        distance_m: 0.0,
        edge: root,
    });
    while let Some(current) = frontier.pop() {
        if current.distance_m > distances[current.edge] + 1.0e-9 {
            continue;
        }
        let current_midpoint = boundary_midpoint(topology, &boundaries[current.edge]);
        for &neighbor in &adjacency[current.edge] {
            if !in_component[neighbor] {
                continue;
            }
            let neighbor_midpoint = boundary_midpoint(topology, &boundaries[neighbor]);
            let step = arc_radians(current_midpoint, neighbor_midpoint) * planet_radius_m;
            let candidate = current.distance_m + step;
            if candidate + 1.0e-9 < distances[neighbor] {
                distances[neighbor] = candidate;
                frontier.push(DistanceFrontier {
                    distance_m: candidate,
                    edge: neighbor,
                });
            }
        }
    }
    distances
}

fn local_curvature_deg<T: PlanetTopology>(
    topology: &T,
    boundaries: &[PlateBoundaryEdge],
    adjacency: &[Vec<usize>],
    edge_index: usize,
    system_id: u32,
    system_ids: &[u32],
) -> f64 {
    let tangent = boundary_tangent(topology, &boundaries[edge_index]);
    let mut total = 0.0;
    let mut count = 0_u32;
    for &neighbor in &adjacency[edge_index] {
        if system_ids[neighbor] != system_id {
            continue;
        }
        let neighbor_tangent = boundary_tangent(topology, &boundaries[neighbor]);
        let alignment = dot(tangent, neighbor_tangent).abs().clamp(0.0, 1.0);
        total += alignment.acos().to_degrees();
        count += 1;
    }
    if count > 0 {
        total / f64::from(count)
    } else {
        0.0
    }
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn history_hash(
    stage_seed: u64,
    systems: &[TectonicBoundarySystem],
    states: &[BoundaryHistoryState],
) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    hash = fnv_update(hash, b"tectonics:history-systems:v1\0");
    hash = fnv_update(hash, &stage_seed.to_le_bytes());
    for system in systems {
        hash = fnv_update(hash, &system.id.to_le_bytes());
        hash = fnv_update(hash, &system.plate_low.to_le_bytes());
        hash = fnv_update(hash, &system.plate_high.to_le_bytes());
        hash = fnv_update(hash, &[system.kind as u8]);
        hash = fnv_update(hash, &system.event_age_myr.to_bits().to_le_bytes());
        hash = fnv_update(hash, &system.length_km.to_bits().to_le_bytes());
        for boundary in &system.boundary_indices {
            hash = fnv_update(hash, &boundary.to_le_bytes());
        }
    }
    for state in states {
        hash = fnv_update(hash, &state.boundary_index.to_le_bytes());
        hash = fnv_update(hash, &state.system_id.to_le_bytes());
        hash = fnv_update(hash, &state.along_strike_fraction.to_bits().to_le_bytes());
        hash = fnv_update(hash, &state.event_age_myr.to_bits().to_le_bytes());
        hash = fnv_update(
            hash,
            &state.cumulative_convergence_km.to_bits().to_le_bytes(),
        );
        hash = fnv_update(hash, &state.cumulative_extension_km.to_bits().to_le_bytes());
        hash = fnv_update(hash, &state.cumulative_shear_km.to_bits().to_le_bytes());
    }
    hash
}

pub fn generate_tectonic_history<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    request: &TectonicHistoryRequest,
    parameters: PlanetPhysicalParameters,
) -> Result<TectonicHistoryModel, WorldgenError> {
    if request.seed.trim().is_empty() {
        return Err(WorldgenError::InvalidTectonics(
            "tectonic history seed must not be empty",
        ));
    }
    parameters
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    if tectonics.boundaries.is_empty() {
        return Err(WorldgenError::InvalidTectonics(
            "tectonic history requires plate boundaries",
        ));
    }

    let stage_seed = random::derive_stage_seed(&request.seed, TECTONIC_HISTORY_NAMESPACE);
    let adjacency = build_boundary_adjacency(topology, &tectonics.boundaries);
    let components = connected_systems(&tectonics.boundaries, &adjacency);
    let mut system_ids = vec![INVALID_SYSTEM_ID; tectonics.boundaries.len()];
    for (system_id, component) in components.iter().enumerate() {
        for &edge in component {
            system_ids[edge] = system_id as u32;
        }
    }

    let mut systems = Vec::with_capacity(components.len());
    let mut boundary_state = vec![
        BoundaryHistoryState {
            boundary_index: 0,
            system_id: INVALID_SYSTEM_ID,
            along_strike_distance_km: 0.0,
            along_strike_fraction: 0.0,
            obliquity_deg: 0.0,
            curvature_deg: 0.0,
            event_age_myr: 0.0,
            cumulative_convergence_km: 0.0,
            cumulative_extension_km: 0.0,
            cumulative_shear_km: 0.0,
        };
        tectonics.boundaries.len()
    ];

    for (system_id, component) in components.iter().enumerate() {
        let first = &tectonics.boundaries[component[0]];
        let (plate_low, plate_high) = plate_pair(first);
        let age = event_age_myr(stage_seed, system_id as u32, first);
        let distances = along_strike_distances(
            topology,
            &tectonics.boundaries,
            &adjacency,
            component,
            parameters.radius_m,
        );
        let maximum_distance = component
            .iter()
            .map(|edge| distances[*edge])
            .filter(|value| value.is_finite())
            .fold(0.0_f64, f64::max);
        let mut normal_sum = 0.0;
        let mut shear_sum = 0.0;
        let mut obliquity_sum = 0.0;
        let mut curvature_sum = 0.0;
        let mut convergence_sum = 0.0;
        let mut extension_sum = 0.0;
        let mut shear_displacement_sum = 0.0;
        let mut endpoints = 0_u16;
        let mut junctions = 0_u16;

        for &edge_index in component {
            let edge = &tectonics.boundaries[edge_index];
            let normal = edge.normal_rate_m_per_year;
            let shear = edge.shear_rate_m_per_year;
            let obliquity = shear.abs().atan2(normal.abs().max(1.0e-12)).to_degrees();
            let curvature = local_curvature_deg(
                topology,
                &tectonics.boundaries,
                &adjacency,
                edge_index,
                system_id as u32,
                &system_ids,
            );
            let degree = adjacency[edge_index]
                .iter()
                .filter(|neighbor| system_ids[**neighbor] == system_id as u32)
                .count();
            if degree <= 1 {
                endpoints = endpoints.saturating_add(1);
            }
            if degree >= 3 {
                junctions = junctions.saturating_add(1);
            }
            let convergence = (-normal).max(0.0) * age * 1_000.0;
            let extension = normal.max(0.0) * age * 1_000.0;
            let shear_displacement = shear.abs() * age * 1_000.0;
            let distance = distances[edge_index];
            let fraction = if maximum_distance > 0.0 && distance.is_finite() {
                (distance / maximum_distance).clamp(0.0, 1.0)
            } else {
                0.0
            };
            boundary_state[edge_index] = BoundaryHistoryState {
                boundary_index: edge_index as u32,
                system_id: system_id as u32,
                along_strike_distance_km: (distance / 1_000.0) as f32,
                along_strike_fraction: fraction as f32,
                obliquity_deg: obliquity as f32,
                curvature_deg: curvature as f32,
                event_age_myr: age as f32,
                cumulative_convergence_km: convergence as f32,
                cumulative_extension_km: extension as f32,
                cumulative_shear_km: shear_displacement as f32,
            };
            normal_sum += normal;
            shear_sum += shear;
            obliquity_sum += obliquity;
            curvature_sum += curvature;
            convergence_sum += convergence;
            extension_sum += extension;
            shear_displacement_sum += shear_displacement;
        }
        let count = component.len() as f64;
        systems.push(TectonicBoundarySystem {
            id: system_id as u32,
            plate_low,
            plate_high,
            kind: first.kind,
            boundary_indices: component.iter().map(|edge| *edge as u32).collect(),
            event_age_myr: age as f32,
            length_km: (maximum_distance / 1_000.0) as f32,
            mean_normal_rate_m_per_year: (normal_sum / count) as f32,
            mean_shear_rate_m_per_year: (shear_sum / count) as f32,
            mean_obliquity_deg: (obliquity_sum / count) as f32,
            mean_curvature_deg: (curvature_sum / count) as f32,
            cumulative_convergence_km: (convergence_sum / count) as f32,
            cumulative_extension_km: (extension_sum / count) as f32,
            cumulative_shear_km: (shear_displacement_sum / count) as f32,
            endpoint_count: endpoints,
            junction_count: junctions,
        });
    }

    if boundary_state
        .iter()
        .any(|state| state.system_id == INVALID_SYSTEM_ID)
    {
        return Err(WorldgenError::InvalidTectonics(
            "tectonic history left a boundary outside every system",
        ));
    }

    let mut convergent_system_count = 0_u32;
    let mut divergent_system_count = 0_u32;
    let mut transform_system_count = 0_u32;
    let mut maximum_event_age_myr = 0.0_f64;
    let mut maximum_cumulative_convergence_km = 0.0_f64;
    let mut maximum_cumulative_extension_km = 0.0_f64;
    let mut obliquity_sum = 0.0_f64;
    for system in &systems {
        match system.kind {
            PlateBoundaryKind::Convergent => convergent_system_count += 1,
            PlateBoundaryKind::Divergent => divergent_system_count += 1,
            PlateBoundaryKind::Transform => transform_system_count += 1,
        }
        maximum_event_age_myr = maximum_event_age_myr.max(f64::from(system.event_age_myr));
        maximum_cumulative_convergence_km =
            maximum_cumulative_convergence_km.max(f64::from(system.cumulative_convergence_km));
        maximum_cumulative_extension_km =
            maximum_cumulative_extension_km.max(f64::from(system.cumulative_extension_km));
        obliquity_sum += f64::from(system.mean_obliquity_deg);
    }
    let mean_obliquity_deg = if systems.is_empty() {
        0.0
    } else {
        obliquity_sum / systems.len() as f64
    };
    let hash = history_hash(stage_seed, &systems, &boundary_state);

    Ok(TectonicHistoryModel {
        stage: StageIdentity {
            id: TECTONIC_HISTORY_STAGE_ID,
            version: TECTONIC_HISTORY_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        systems,
        boundary_state,
        metrics: TectonicHistoryMetrics {
            boundary_system_count: components.len() as u32,
            convergent_system_count,
            divergent_system_count,
            transform_system_count,
            maximum_event_age_myr,
            maximum_cumulative_convergence_km,
            maximum_cumulative_extension_km,
            mean_obliquity_deg,
            history_hash: hash,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_icosphere, generate_tectonics, TectonicsRequest};

    #[test]
    fn history_systems_cover_every_boundary_and_are_deterministic() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let tectonics = generate_tectonics(
            &topology,
            &TectonicsRequest::new("wg25-history", 16),
            planet,
        )
        .unwrap();
        let request = TectonicHistoryRequest::new("wg25-history");
        let first = generate_tectonic_history(&topology, &tectonics, &request, planet).unwrap();
        let second = generate_tectonic_history(&topology, &tectonics, &request, planet).unwrap();
        assert_eq!(first.metrics.history_hash, second.metrics.history_hash);
        assert_eq!(first.boundary_state.len(), tectonics.boundaries.len());
        assert_eq!(
            first
                .systems
                .iter()
                .map(|system| system.boundary_indices.len())
                .sum::<usize>(),
            tectonics.boundaries.len()
        );
        assert!(first
            .boundary_state
            .iter()
            .all(|state| state.system_id < first.systems.len() as u32));
    }

    #[test]
    fn chronology_changes_without_repartitioning_boundary_systems() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let tectonics = generate_tectonics(
            &topology,
            &TectonicsRequest::new("wg25-chronology", 18),
            planet,
        )
        .unwrap();
        let a = generate_tectonic_history(
            &topology,
            &tectonics,
            &TectonicHistoryRequest::new("wg25-chronology-a"),
            planet,
        )
        .unwrap();
        let b = generate_tectonic_history(
            &topology,
            &tectonics,
            &TectonicHistoryRequest::new("wg25-chronology-b"),
            planet,
        )
        .unwrap();
        assert_eq!(a.boundary_system_ids(), b.boundary_system_ids());
        assert_ne!(a.metrics.history_hash, b.metrics.history_hash);
        assert_ne!(a.boundary_event_ages_myr(), b.boundary_event_ages_myr());
    }

    #[test]
    fn kinematic_history_fields_are_finite_and_causally_signed() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let tectonics = generate_tectonics(
            &topology,
            &TectonicsRequest::new("wg25-kinematics", 20),
            planet,
        )
        .unwrap();
        let history = generate_tectonic_history(
            &topology,
            &tectonics,
            &TectonicHistoryRequest::new("wg25-kinematics"),
            planet,
        )
        .unwrap();
        for state in &history.boundary_state {
            assert!(state.along_strike_distance_km.is_finite());
            assert!((0.0..=1.0).contains(&state.along_strike_fraction));
            assert!((0.0..=90.0).contains(&state.obliquity_deg));
            assert!(state.curvature_deg.is_finite() && state.curvature_deg >= 0.0);
            assert!(state.event_age_myr.is_finite() && state.event_age_myr > 0.0);
            assert!(state.cumulative_convergence_km >= 0.0);
            assert!(state.cumulative_extension_km >= 0.0);
            assert!(state.cumulative_shear_km >= 0.0);
            let edge = &tectonics.boundaries[state.boundary_index as usize];
            match edge.kind {
                PlateBoundaryKind::Convergent => assert!(state.cumulative_extension_km <= 1.0e-5),
                PlateBoundaryKind::Divergent => assert!(state.cumulative_convergence_km <= 1.0e-5),
                PlateBoundaryKind::Transform => {}
            }
        }
    }
}
