use crate::{random, PlanetPhysicalParameters, PlanetTopology, StageIdentity, WorldgenError};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64::consts::PI;

pub const TECTONICS_STAGE_ID: &str = "tectonics:plates";
pub const TECTONICS_STAGE_VERSION: u32 = 1;
pub const MIN_TECTONIC_PLATES: u16 = 4;
pub const MAX_TECTONIC_PLATES: u16 = 48;
const TECTONICS_NAMESPACE: &str = "worldgen:tectonics:plates:v1";
const MILLION_YEARS: f64 = 1_000_000.0;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlateBoundaryKind {
    Convergent = 1,
    Divergent = 2,
    Transform = 3,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TectonicsRequest {
    pub seed: String,
    pub plate_count: u16,
}

impl TectonicsRequest {
    pub fn new(seed: impl Into<String>, plate_count: u16) -> Self {
        Self {
            seed: seed.into(),
            plate_count,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TectonicPlate {
    pub id: u16,
    pub seed_sample: u32,
    pub euler_pole: [f64; 3],
    pub angular_velocity_rad_per_myr: [f64; 3],
    pub area_steradians: f64,
}

impl TectonicPlate {
    pub fn angular_speed_rad_per_myr(&self) -> f64 {
        norm(self.angular_velocity_rad_per_myr)
    }

    pub fn reference_speed_mm_per_year(&self, planet_radius_m: f64) -> f64 {
        self.angular_speed_rad_per_myr() * planet_radius_m / MILLION_YEARS * 1_000.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlateBoundaryEdge {
    pub sample_a: u32,
    pub sample_b: u32,
    pub plate_a: u16,
    pub plate_b: u16,
    pub kind: PlateBoundaryKind,
    pub normal_rate_m_per_year: f64,
    pub shear_rate_m_per_year: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TectonicMetrics {
    pub sample_count: u32,
    pub plate_count: u16,
    pub boundary_edge_count: u32,
    pub convergent_edge_count: u32,
    pub divergent_edge_count: u32,
    pub transform_edge_count: u32,
    pub minimum_plate_area_fraction: f64,
    pub maximum_plate_area_fraction: f64,
    pub mean_plate_area_fraction: f64,
    pub minimum_seed_separation_rad: f64,
    pub mean_reference_speed_mm_per_year: f64,
    pub tectonic_hash: u64,
}

impl TectonicMetrics {
    pub fn tectonic_hash_hex(&self) -> String {
        format!("{:016x}", self.tectonic_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TectonicModel {
    pub stage: StageIdentity,
    pub plates: Vec<TectonicPlate>,
    pub plate_ids: Vec<u16>,
    pub boundaries: Vec<PlateBoundaryEdge>,
    pub metrics: TectonicMetrics,
}

impl TectonicModel {
    pub fn plate_seed_samples(&self) -> Vec<u32> {
        self.plates.iter().map(|plate| plate.seed_sample).collect()
    }

    pub fn flattened_euler_poles(&self) -> Vec<f64> {
        let mut output = Vec::with_capacity(self.plates.len() * 3);
        for plate in &self.plates {
            output.extend_from_slice(&plate.euler_pole);
        }
        output
    }

    pub fn flattened_angular_velocities_rad_per_myr(&self) -> Vec<f64> {
        let mut output = Vec::with_capacity(self.plates.len() * 3);
        for plate in &self.plates {
            output.extend_from_slice(&plate.angular_velocity_rad_per_myr);
        }
        output
    }

    pub fn plate_area_steradians(&self) -> Vec<f64> {
        self.plates
            .iter()
            .map(|plate| plate.area_steradians)
            .collect()
    }

    pub fn flattened_boundary_samples(&self) -> Vec<u32> {
        let mut output = Vec::with_capacity(self.boundaries.len() * 2);
        for boundary in &self.boundaries {
            output.extend_from_slice(&[boundary.sample_a, boundary.sample_b]);
        }
        output
    }

    pub fn flattened_boundary_plate_ids(&self) -> Vec<u16> {
        let mut output = Vec::with_capacity(self.boundaries.len() * 2);
        for boundary in &self.boundaries {
            output.extend_from_slice(&[boundary.plate_a, boundary.plate_b]);
        }
        output
    }

    pub fn boundary_kinds(&self) -> Vec<u8> {
        self.boundaries
            .iter()
            .map(|boundary| boundary.kind as u8)
            .collect()
    }

    pub fn boundary_normal_rates_m_per_year(&self) -> Vec<f64> {
        self.boundaries
            .iter()
            .map(|boundary| boundary.normal_rate_m_per_year)
            .collect()
    }

    pub fn boundary_shear_rates_m_per_year(&self) -> Vec<f64> {
        self.boundaries
            .iter()
            .map(|boundary| boundary.shear_rate_m_per_year)
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct Frontier {
    distance_rad: f64,
    plate: u16,
    sample: u32,
}

impl PartialEq for Frontier {
    fn eq(&self, other: &Self) -> bool {
        self.distance_rad.to_bits() == other.distance_rad.to_bits()
            && self.plate == other.plate
            && self.sample == other.sample
    }
}
impl Eq for Frontier {}
impl Ord for Frontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance_rad
            .total_cmp(&self.distance_rad)
            .then_with(|| other.plate.cmp(&self.plate))
            .then_with(|| other.sample.cmp(&self.sample))
    }
}
impl PartialOrd for Frontier {
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
    scale(a, 1.0 / norm(a))
}
fn arc_radians(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(a, b).clamp(-1.0, 1.0).acos()
}

fn unit_random(value: u64) -> f64 {
    ((random::mix64(value) >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0)
}

fn random_unit_vector(seed: u64, stream: u64) -> [f64; 3] {
    let z = unit_random(seed ^ stream ^ 0x65b5_61f5_1b6b_42d1) * 2.0 - 1.0;
    let theta = unit_random(seed ^ stream ^ 0xa076_1d64_78bd_642f) * 2.0 * PI;
    let radial = (1.0 - z * z).max(0.0).sqrt();
    [radial * theta.cos(), radial * theta.sin(), z]
}

fn select_plate_seeds<T: PlanetTopology>(
    topology: &T,
    plate_count: u16,
    stage_seed: u64,
) -> Vec<u32> {
    let sample_count = topology.sample_count() as usize;
    let expected_spacing_rad = (4.0 * PI / f64::from(plate_count)).sqrt();
    let minimum_separation_rad = (expected_spacing_rad * 0.20).clamp(0.07, 0.32);
    let candidate_attempts = sample_count.clamp(256, 4_096);
    let first = (random::mix64(stage_seed ^ 0xd1b5_4a32_d192_ed03) % sample_count as u64) as u32;
    let mut seeds = Vec::with_capacity(plate_count as usize);
    seeds.push(first);

    for plate_index in 1..plate_count as usize {
        let mut accepted = None;
        let mut best_sample = None;
        let mut best_separation = f64::NEG_INFINITY;
        for attempt in 0..candidate_attempts {
            let stream = stage_seed
                ^ (plate_index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                ^ (attempt as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)
                ^ 0x94d0_49bb_1331_11eb;
            let candidate = (random::mix64(stream) % sample_count as u64) as u32;
            if seeds.contains(&candidate) {
                continue;
            }
            let position = topology.unit_position(candidate);
            let separation = seeds
                .iter()
                .map(|seed| arc_radians(position, topology.unit_position(*seed)))
                .fold(PI, f64::min);
            if separation > best_separation
                || (separation == best_separation && Some(candidate) < best_sample)
            {
                best_separation = separation;
                best_sample = Some(candidate);
            }
            if separation >= minimum_separation_rad {
                accepted = Some(candidate);
                break;
            }
        }

        if accepted.is_none() {
            for sample in 0..sample_count as u32 {
                if seeds.contains(&sample) {
                    continue;
                }
                let position = topology.unit_position(sample);
                let separation = seeds
                    .iter()
                    .map(|seed| arc_radians(position, topology.unit_position(*seed)))
                    .fold(PI, f64::min);
                if separation > best_separation
                    || (separation == best_separation && Some(sample) < best_sample)
                {
                    best_separation = separation;
                    best_sample = Some(sample);
                }
            }
        }
        seeds.push(
            accepted
                .or(best_sample)
                .expect("plate seed selection requires an unused topology sample"),
        );
    }
    seeds
}

fn build_plates(seeds: &[u32], stage_seed: u64) -> Vec<TectonicPlate> {
    seeds
        .iter()
        .enumerate()
        .map(|(index, seed_sample)| {
            let stream = index as u64;
            let euler_pole =
                random_unit_vector(stage_seed, stream.wrapping_mul(0x9e37_79b9_7f4a_7c15));
            let speed_fraction = unit_random(
                stage_seed ^ stream.wrapping_mul(0xbf58_476d_1ce4_e5b9) ^ 0x8ebc_6af0_9c88_c6e3,
            );
            let angular_speed_deg_per_myr = 0.12 + 1.08 * speed_fraction.powf(1.25);
            let angular_speed_rad_per_myr = angular_speed_deg_per_myr.to_radians();
            TectonicPlate {
                id: index as u16,
                seed_sample: *seed_sample,
                euler_pole,
                angular_velocity_rad_per_myr: scale(euler_pole, angular_speed_rad_per_myr),
                area_steradians: 0.0,
            }
        })
        .collect()
}

fn assign_plate_ids<T: PlanetTopology>(topology: &T, seeds: &[u32]) -> Vec<u16> {
    let sample_count = topology.sample_count() as usize;
    let mut distances = vec![f64::INFINITY; sample_count];
    let mut owners = vec![u16::MAX; sample_count];
    let mut frontier = BinaryHeap::new();
    for (plate, sample) in seeds.iter().enumerate() {
        distances[*sample as usize] = 0.0;
        owners[*sample as usize] = plate as u16;
        frontier.push(Frontier {
            distance_rad: 0.0,
            plate: plate as u16,
            sample: *sample,
        });
    }

    while let Some(current) = frontier.pop() {
        let index = current.sample as usize;
        if current.plate != owners[index] || current.distance_rad > distances[index] + 1.0e-14 {
            continue;
        }
        let neighbors = topology.neighbors(current.sample);
        let edge_lengths = topology.neighbor_arc_lengths_rad(current.sample);
        for neighbor_index in 0..neighbors.len() {
            let neighbor = neighbors[neighbor_index];
            let candidate = current.distance_rad + edge_lengths[neighbor_index];
            if candidate + 1.0e-14 < distances[neighbor as usize] {
                distances[neighbor as usize] = candidate;
                owners[neighbor as usize] = current.plate;
                frontier.push(Frontier {
                    distance_rad: candidate,
                    plate: current.plate,
                    sample: neighbor,
                });
            }
        }
    }
    owners
}

fn validate_connected_plates<T: PlanetTopology>(
    topology: &T,
    owners: &[u16],
    seeds: &[u32],
    plate_count: u16,
) -> Result<Vec<u32>, WorldgenError> {
    let mut expected = vec![0_u32; plate_count as usize];
    for owner in owners {
        if *owner == u16::MAX || usize::from(*owner) >= expected.len() {
            return Err(WorldgenError::InvalidTectonics(
                "tectonic partition left an unassigned sample",
            ));
        }
        expected[*owner as usize] += 1;
    }
    let mut seen = vec![false; owners.len()];
    for plate in 0..plate_count as usize {
        let start = seeds[plate];
        if owners[start as usize] != plate as u16 {
            return Err(WorldgenError::InvalidTectonics(
                "plate seed lost ownership of its source sample",
            ));
        }
        let mut reached = 0_u32;
        let mut stack = vec![start];
        seen[start as usize] = true;
        while let Some(sample) = stack.pop() {
            reached += 1;
            for neighbor in topology.neighbors(sample) {
                let neighbor_index = *neighbor as usize;
                if !seen[neighbor_index] && owners[neighbor_index] == plate as u16 {
                    seen[neighbor_index] = true;
                    stack.push(*neighbor);
                }
            }
        }
        if reached != expected[plate] {
            return Err(WorldgenError::InvalidTectonics(
                "tectonic plate partition contains a disconnected component",
            ));
        }
    }
    Ok(expected)
}

fn velocity_m_per_year(
    angular_velocity_rad_per_myr: [f64; 3],
    position: [f64; 3],
    planet_radius_m: f64,
) -> [f64; 3] {
    scale(
        cross(angular_velocity_rad_per_myr, position),
        planet_radius_m / MILLION_YEARS,
    )
}

fn classify_boundary(normal_rate: f64, shear_rate: f64) -> PlateBoundaryKind {
    let relative_speed = normal_rate.hypot(shear_rate);
    if relative_speed <= 1.0e-12 || normal_rate.abs() < relative_speed * 0.35 {
        PlateBoundaryKind::Transform
    } else if normal_rate < 0.0 {
        PlateBoundaryKind::Convergent
    } else {
        PlateBoundaryKind::Divergent
    }
}

fn build_boundaries<T: PlanetTopology>(
    topology: &T,
    owners: &[u16],
    plates: &[TectonicPlate],
    planet_radius_m: f64,
) -> Vec<PlateBoundaryEdge> {
    let mut boundaries = Vec::new();
    for sample_a in 0..topology.sample_count() {
        let position_a = topology.unit_position(sample_a);
        for sample_b in topology.neighbors(sample_a) {
            if *sample_b <= sample_a {
                continue;
            }
            let plate_a = owners[sample_a as usize];
            let plate_b = owners[*sample_b as usize];
            if plate_a == plate_b {
                continue;
            }
            let position_b = topology.unit_position(*sample_b);
            let midpoint = normalize(add(position_a, position_b));
            let normal = normalize(sub(position_b, position_a));
            let tangent = normalize(cross(midpoint, normal));
            let velocity_a = velocity_m_per_year(
                plates[plate_a as usize].angular_velocity_rad_per_myr,
                midpoint,
                planet_radius_m,
            );
            let velocity_b = velocity_m_per_year(
                plates[plate_b as usize].angular_velocity_rad_per_myr,
                midpoint,
                planet_radius_m,
            );
            let relative = sub(velocity_b, velocity_a);
            let normal_rate = dot(relative, normal);
            let shear_rate = dot(relative, tangent);
            boundaries.push(PlateBoundaryEdge {
                sample_a,
                sample_b: *sample_b,
                plate_a,
                plate_b,
                kind: classify_boundary(normal_rate, shear_rate),
                normal_rate_m_per_year: normal_rate,
                shear_rate_m_per_year: shear_rate,
            });
        }
    }
    boundaries
}

fn minimum_seed_separation<T: PlanetTopology>(topology: &T, seeds: &[u32]) -> f64 {
    let mut minimum = f64::INFINITY;
    for a in 0..seeds.len() {
        for b in (a + 1)..seeds.len() {
            minimum = minimum.min(arc_radians(
                topology.unit_position(seeds[a]),
                topology.unit_position(seeds[b]),
            ));
        }
    }
    minimum
}

fn fnv1a_update(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

fn tectonic_hash(
    stage_seed: u64,
    plates: &[TectonicPlate],
    owners: &[u16],
    boundaries: &[PlateBoundaryEdge],
) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    fnv1a_update(&mut hash, b"tectonics:plates:v1\0");
    fnv1a_update(&mut hash, &stage_seed.to_le_bytes());
    for plate in plates {
        fnv1a_update(&mut hash, &plate.seed_sample.to_le_bytes());
        for component in plate.angular_velocity_rad_per_myr {
            fnv1a_update(&mut hash, &component.to_bits().to_le_bytes());
        }
    }
    for owner in owners {
        fnv1a_update(&mut hash, &owner.to_le_bytes());
    }
    for boundary in boundaries {
        fnv1a_update(&mut hash, &boundary.sample_a.to_le_bytes());
        fnv1a_update(&mut hash, &boundary.sample_b.to_le_bytes());
        fnv1a_update(&mut hash, &[boundary.kind as u8]);
        fnv1a_update(
            &mut hash,
            &boundary.normal_rate_m_per_year.to_bits().to_le_bytes(),
        );
        fnv1a_update(
            &mut hash,
            &boundary.shear_rate_m_per_year.to_bits().to_le_bytes(),
        );
    }
    hash
}

pub fn generate_tectonics<T: PlanetTopology>(
    topology: &T,
    request: &TectonicsRequest,
    parameters: PlanetPhysicalParameters,
) -> Result<TectonicModel, WorldgenError> {
    if request.seed.trim().is_empty() {
        return Err(WorldgenError::InvalidTectonics(
            "tectonic seed must not be empty",
        ));
    }
    if request.plate_count < MIN_TECTONIC_PLATES || request.plate_count > MAX_TECTONIC_PLATES {
        return Err(WorldgenError::InvalidTectonics(
            "tectonic plate count is outside the supported WG-2 range",
        ));
    }
    if u32::from(request.plate_count) > topology.sample_count() {
        return Err(WorldgenError::InvalidTectonics(
            "tectonic plate count exceeds topology sample count",
        ));
    }
    parameters
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;

    let stage_seed = random::derive_stage_seed(&request.seed, TECTONICS_NAMESPACE);
    let seeds = select_plate_seeds(topology, request.plate_count, stage_seed);
    let mut plates = build_plates(&seeds, stage_seed);
    let plate_ids = assign_plate_ids(topology, &seeds);
    let sample_counts =
        validate_connected_plates(topology, &plate_ids, &seeds, request.plate_count)?;

    for sample in 0..topology.sample_count() {
        plates[plate_ids[sample as usize] as usize].area_steradians +=
            topology.area_steradians(sample);
    }
    if sample_counts.iter().any(|count| *count == 0) {
        return Err(WorldgenError::InvalidTectonics(
            "tectonic generator created an empty plate",
        ));
    }

    let boundaries = build_boundaries(topology, &plate_ids, &plates, parameters.radius_m);
    if boundaries.is_empty() {
        return Err(WorldgenError::InvalidTectonics(
            "tectonic generator produced no plate boundaries",
        ));
    }

    let mut convergent_edge_count = 0_u32;
    let mut divergent_edge_count = 0_u32;
    let mut transform_edge_count = 0_u32;
    for boundary in &boundaries {
        match boundary.kind {
            PlateBoundaryKind::Convergent => convergent_edge_count += 1,
            PlateBoundaryKind::Divergent => divergent_edge_count += 1,
            PlateBoundaryKind::Transform => transform_edge_count += 1,
        }
    }
    let total_area = plates
        .iter()
        .map(|plate| plate.area_steradians)
        .sum::<f64>();
    let area_fractions: Vec<f64> = plates
        .iter()
        .map(|plate| plate.area_steradians / total_area)
        .collect();
    let minimum_plate_area_fraction = area_fractions.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum_plate_area_fraction = area_fractions
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let mean_reference_speed_mm_per_year = plates
        .iter()
        .map(|plate| plate.reference_speed_mm_per_year(parameters.radius_m))
        .sum::<f64>()
        / plates.len() as f64;
    let tectonic_hash = tectonic_hash(stage_seed, &plates, &plate_ids, &boundaries);
    let metrics = TectonicMetrics {
        sample_count: topology.sample_count(),
        plate_count: request.plate_count,
        boundary_edge_count: boundaries.len() as u32,
        convergent_edge_count,
        divergent_edge_count,
        transform_edge_count,
        minimum_plate_area_fraction,
        maximum_plate_area_fraction,
        mean_plate_area_fraction: 1.0 / f64::from(request.plate_count),
        minimum_seed_separation_rad: minimum_seed_separation(topology, &seeds),
        mean_reference_speed_mm_per_year,
        tectonic_hash,
    };

    Ok(TectonicModel {
        stage: StageIdentity {
            id: TECTONICS_STAGE_ID,
            version: TECTONICS_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        plates,
        plate_ids,
        boundaries,
        metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_icosphere;

    #[test]
    fn tectonic_generation_is_deterministic_seed_sensitive_and_complete() {
        let topology = build_icosphere(4).unwrap();
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let request = TectonicsRequest::new("wg2-determinism", 16);
        let first = generate_tectonics(&topology, &request, parameters).unwrap();
        let second = generate_tectonics(&topology, &request, parameters).unwrap();
        let changed = generate_tectonics(
            &topology,
            &TectonicsRequest::new("wg2-different", 16),
            parameters,
        )
        .unwrap();
        assert_eq!(first.metrics.tectonic_hash, second.metrics.tectonic_hash);
        assert_eq!(first.plate_ids, second.plate_ids);
        assert_ne!(first.metrics.tectonic_hash, changed.metrics.tectonic_hash);
        assert_eq!(first.plate_ids.len(), topology.sample_count() as usize);
        assert!(first.plate_ids.iter().all(|plate| *plate < 16));
    }

    #[test]
    fn every_plate_is_nonempty_connected_and_owns_its_seed() {
        let topology = build_icosphere(4).unwrap();
        for seed in ["wg2-connect-a", "wg2-connect-b", "wg2-connect-c"] {
            let model = generate_tectonics(
                &topology,
                &TectonicsRequest::new(seed, 18),
                PlanetPhysicalParameters::earthlike_reference(),
            )
            .unwrap();
            let counts = validate_connected_plates(
                &topology,
                &model.plate_ids,
                &model.plate_seed_samples(),
                18,
            )
            .unwrap();
            assert!(counts.iter().all(|count| *count > 0));
            for plate in &model.plates {
                assert_eq!(model.plate_ids[plate.seed_sample as usize], plate.id);
            }
        }
    }

    #[test]
    fn rigid_plate_velocity_is_tangent_to_the_sphere() {
        let topology = build_icosphere(3).unwrap();
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let model = generate_tectonics(
            &topology,
            &TectonicsRequest::new("wg2-velocity", 12),
            parameters,
        )
        .unwrap();
        for sample in [0_u32, 7, 20, 100, 400] {
            let plate = &model.plates[model.plate_ids[sample as usize] as usize];
            let position = topology.unit_position(sample);
            let velocity = velocity_m_per_year(
                plate.angular_velocity_rad_per_myr,
                position,
                parameters.radius_m,
            );
            assert!(dot(position, velocity).abs() < 1.0e-12);
        }
    }

    #[test]
    fn boundaries_connect_different_plates_and_partition_all_boundary_edges() {
        let topology = build_icosphere(4).unwrap();
        let model = generate_tectonics(
            &topology,
            &TectonicsRequest::new("wg2-boundaries", 16),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
        assert!(model.metrics.boundary_edge_count > 0);
        assert_eq!(
            model.metrics.boundary_edge_count,
            model.metrics.convergent_edge_count
                + model.metrics.divergent_edge_count
                + model.metrics.transform_edge_count
        );
        for boundary in &model.boundaries {
            assert_ne!(boundary.plate_a, boundary.plate_b);
            assert_eq!(
                model.plate_ids[boundary.sample_a as usize],
                boundary.plate_a
            );
            assert_eq!(
                model.plate_ids[boundary.sample_b as usize],
                boundary.plate_b
            );
            assert!(boundary.normal_rate_m_per_year.is_finite());
            assert!(boundary.shear_rate_m_per_year.is_finite());
        }
    }

    #[test]
    fn plate_areas_close_to_the_sphere_and_remain_macro_scale() {
        let topology = build_icosphere(5).unwrap();
        let model = generate_tectonics(
            &topology,
            &TectonicsRequest::new("wg2-area", 20),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
        let total = model
            .plates
            .iter()
            .map(|plate| plate.area_steradians)
            .sum::<f64>();
        assert!((total - 4.0 * PI).abs() < 1.0e-10);
        assert!(model.metrics.minimum_plate_area_fraction > 0.005);
        assert!(model.metrics.maximum_plate_area_fraction < 0.20);
        assert!(model.metrics.minimum_seed_separation_rad > 0.15);
        assert!(model.metrics.mean_reference_speed_mm_per_year > 10.0);
        assert!(model.metrics.mean_reference_speed_mm_per_year < 150.0);
    }
}
