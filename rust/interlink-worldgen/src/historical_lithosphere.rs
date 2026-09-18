use crate::{
    generate_tectonics, random, CrustKind, PlanetPhysicalParameters, PlanetTopology,
    PlateBoundaryKind, StageIdentity, TectonicModel, TectonicsRequest, WorldgenError,
    MAX_TECTONIC_PLATES, MIN_TECTONIC_PLATES,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const HISTORICAL_LITHOSPHERE_STAGE_ID: &str = "geology:historical-lithosphere";
pub const HISTORICAL_LITHOSPHERE_STAGE_VERSION: u32 = 3;
const HISTORICAL_LITHOSPHERE_NAMESPACE: &str = "worldgen:geology:historical-lithosphere:v2";
const ANCESTRAL_TECTONICS_NAMESPACE: &str = "worldgen:geology:historical-lithosphere:ancestral:v1";
const FRAGMENT_NAMESPACE: &str = "worldgen:geology:historical-lithosphere:fragments:v1";
const CRUST_NAMESPACE: &str = "worldgen:geology:historical-lithosphere:crust:v2";
const OCEANIC_SPREADING_KM_PER_MYR: f64 = 25.0;
const MODERN_GROUPING_NAMESPACE: &str = "worldgen:geology:historical-lithosphere:modern:v2";
const EVENT_NAMESPACE: &str = "worldgen:geology:historical-lithosphere:events:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoricalEventKind {
    Rift = 1,
    Spreading = 2,
    Subduction = 3,
    Collision = 4,
    Transform = 5,
    Accretion = 6,
    Capture = 7,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoricalLithosphereRequest {
    pub seed: String,
    pub modern_plate_count: u16,
}

impl HistoricalLithosphereRequest {
    pub fn new(seed: impl Into<String>, modern_plate_count: u16) -> Self {
        Self {
            seed: seed.into(),
            modern_plate_count,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CrustFragment {
    pub id: u16,
    pub parent_fragment_id: Option<u16>,
    pub origin_plate_id: u16,
    pub current_plate_id: u16,
    pub seed_sample: u32,
    pub birth_age_myr: f32,
    pub capture_age_myr: Option<f32>,
    pub accretion_age_myr: Option<f32>,
    pub dominant_crust_kind: u8,
    pub sample_count: u32,
    pub area_steradians: f64,
    pub mean_thickness_km: f32,
    pub mean_density_kg_per_m3: f32,
    pub inherited_fabric: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoricalTectonicEvent {
    pub id: u32,
    pub kind: HistoricalEventKind,
    pub epoch: u8,
    pub age_myr: f32,
    pub plate_a: u16,
    pub plate_b: u16,
    pub fragment_a: u16,
    pub fragment_b: u16,
    pub displacement_km: f32,
    pub strength: f32,
    pub geometry_sample_a: u32,
    pub geometry_sample_b: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoricalLithosphereMetrics {
    pub sample_count: u32,
    pub ancestral_plate_count: u16,
    pub fragment_count: u16,
    pub modern_plate_count: u16,
    pub event_count: u32,
    pub continental_area_fraction: f64,
    pub transitional_area_fraction: f64,
    pub oceanic_area_fraction: f64,
    pub mean_oceanic_age_myr: f64,
    pub history_hash: u64,
}

impl HistoricalLithosphereMetrics {
    pub fn history_hash_hex(&self) -> String {
        format!("{:016x}", self.history_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoricalLithosphereModel {
    pub stage: StageIdentity,
    pub ancestral_tectonics: TectonicModel,
    pub origin_plate_ids: Vec<u16>,
    pub fragment_ids: Vec<u16>,
    pub current_plate_ids: Vec<u16>,
    pub crust_kind: Vec<u8>,
    pub crust_birth_age_myr: Vec<f32>,
    pub fragments: Vec<CrustFragment>,
    pub events: Vec<HistoricalTectonicEvent>,
    pub metrics: HistoricalLithosphereMetrics,
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn unit_random(value: u64) -> f64 {
    ((random::mix64(value) >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn arc_radians(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(a, b).clamp(-1.0, 1.0).acos()
}

fn vector_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let x = a[0] - b[0];
    let y = a[1] - b[1];
    let z = a[2] - b[2];
    (x * x + y * y + z * z).sqrt()
}

fn ancestral_plate_count(modern_plate_count: u16, sample_count: u32) -> Result<u16, WorldgenError> {
    if !(MIN_TECTONIC_PLATES..=MAX_TECTONIC_PLATES).contains(&modern_plate_count) {
        return Err(WorldgenError::InvalidLithosphere(
            "historical lithosphere modern plate count is outside the supported tectonic range",
        ));
    }
    if sample_count < u32::from(modern_plate_count) {
        return Err(WorldgenError::InvalidLithosphere(
            "historical lithosphere requires at least one topology sample per modern plate",
        ));
    }
    let target = modern_plate_count.saturating_mul(3).max(modern_plate_count);
    let sample_limit = u16::try_from(sample_count.min(u32::from(u16::MAX))).unwrap_or(u16::MAX);
    Ok(target
        .min(MAX_TECTONIC_PLATES)
        .min(sample_limit)
        .max(modern_plate_count))
}

fn choose_fragment_seeds<T: PlanetTopology>(
    topology: &T,
    plate_samples: &[u32],
    plate_seed: u32,
    desired_count: usize,
    seed: u64,
) -> Vec<u32> {
    let mut seeds = vec![plate_seed];
    while seeds.len() < desired_count && seeds.len() < plate_samples.len() {
        let mut best = None;
        let mut best_score = f64::NEG_INFINITY;
        for &sample in plate_samples {
            if seeds.contains(&sample) {
                continue;
            }
            let position = topology.unit_position(sample);
            let separation = seeds
                .iter()
                .map(|existing| arc_radians(position, topology.unit_position(*existing)))
                .fold(std::f64::consts::PI, f64::min);
            let jitter =
                unit_random(seed ^ u64::from(sample).wrapping_mul(0x9e37_79b9_7f4a_7c15)) * 0.025;
            let score = separation + jitter;
            if score > best_score || (score == best_score && Some(sample) < best) {
                best_score = score;
                best = Some(sample);
            }
        }
        let Some(sample) = best else { break };
        seeds.push(sample);
    }
    seeds
}

fn build_fragments<T: PlanetTopology>(
    topology: &T,
    ancestral: &TectonicModel,
    seed: u64,
) -> (Vec<u16>, Vec<CrustFragment>) {
    let plate_count = ancestral.plates.len();
    let mut samples_by_plate = vec![Vec::<u32>::new(); plate_count];
    for (sample, plate) in ancestral.plate_ids.iter().copied().enumerate() {
        samples_by_plate[plate as usize].push(sample as u32);
    }

    let mut fragment_ids = vec![u16::MAX; topology.sample_count() as usize];
    let mut fragments = Vec::new();

    for plate in 0..plate_count {
        let samples = &samples_by_plate[plate];
        if samples.is_empty() {
            continue;
        }
        let area_fraction = ancestral.plates[plate].area_steradians / (4.0 * std::f64::consts::PI);
        let size_bonus = if area_fraction > 0.12 {
            2
        } else if area_fraction > 0.055 {
            1
        } else {
            0
        };
        let random_bonus = usize::from(
            unit_random(seed ^ (plate as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)) > 0.58,
        );
        let desired = (1 + size_bonus + random_bonus).min(4).min(samples.len());
        let seeds = choose_fragment_seeds(
            topology,
            samples,
            ancestral.plates[plate].seed_sample,
            desired,
            seed ^ (plate as u64).wrapping_mul(0x94d0_49bb_1331_11eb),
        );
        let first_fragment_id = fragments.len() as u16;
        for (local, &fragment_seed) in seeds.iter().enumerate() {
            fragments.push(CrustFragment {
                id: first_fragment_id + local as u16,
                parent_fragment_id: None,
                origin_plate_id: plate as u16,
                current_plate_id: u16::MAX,
                seed_sample: fragment_seed,
                birth_age_myr: 0.0,
                capture_age_myr: None,
                accretion_age_myr: None,
                dominant_crust_kind: CrustKind::Oceanic as u8,
                sample_count: 0,
                area_steradians: 0.0,
                mean_thickness_km: 0.0,
                mean_density_kg_per_m3: 0.0,
                inherited_fabric: unit_random(
                    seed ^ u64::from(first_fragment_id + local as u16)
                        .wrapping_mul(0xd6e8_feb8_6659_fd93),
                ) as f32,
            });
        }
        for &sample in samples {
            let position = topology.unit_position(sample);
            let local = seeds
                .iter()
                .enumerate()
                .min_by(|(left_index, left), (right_index, right)| {
                    let left_distance = arc_radians(position, topology.unit_position(**left));
                    let right_distance = arc_radians(position, topology.unit_position(**right));
                    left_distance
                        .total_cmp(&right_distance)
                        .then_with(|| left_index.cmp(right_index))
                })
                .map(|(index, _)| index)
                .unwrap_or(0);
            fragment_ids[sample as usize] = first_fragment_id + local as u16;
        }
    }

    for sample in 0..topology.sample_count() {
        let fragment = fragment_ids[sample as usize] as usize;
        fragments[fragment].sample_count += 1;
        fragments[fragment].area_steradians += topology.area_steradians(sample);
    }

    (fragment_ids, fragments)
}

fn build_continental_assemblies<T: PlanetTopology>(
    topology: &T,
    ancestral: &TectonicModel,
    seed: u64,
) -> (Vec<bool>, Vec<u16>) {
    let count = ancestral.plates.len();
    let total_area = ancestral
        .plates
        .iter()
        .map(|plate| plate.area_steradians)
        .sum::<f64>()
        .max(1.0e-12);
    let target_fraction = 0.50 + (unit_random(seed ^ 0x3c79_ac49_2ba7_b653) - 0.5) * 0.08;
    let target_area = total_area * target_fraction.clamp(0.46, 0.54);

    let mut pair_kinds = BTreeMap::<(u16, u16), [u32; 3]>::new();
    let mut plate_perimeter = vec![0_u32; count];
    let mut adjacency = vec![BTreeSet::<usize>::new(); count];
    for boundary in &ancestral.boundaries {
        let pair = if boundary.plate_a < boundary.plate_b {
            (boundary.plate_a, boundary.plate_b)
        } else {
            (boundary.plate_b, boundary.plate_a)
        };
        let counts = pair_kinds.entry(pair).or_insert([0; 3]);
        match boundary.kind {
            PlateBoundaryKind::Convergent => counts[0] += 1,
            PlateBoundaryKind::Divergent => counts[1] += 1,
            PlateBoundaryKind::Transform => counts[2] += 1,
        }
        plate_perimeter[boundary.plate_a as usize] += 1;
        plate_perimeter[boundary.plate_b as usize] += 1;
        adjacency[boundary.plate_a as usize].insert(boundary.plate_b as usize);
        adjacency[boundary.plate_b as usize].insert(boundary.plate_a as usize);
    }

    // Continental material starts from a small number of coherent proto-continental nuclei.
    // Growth is contiguous across the ancestral plate graph; isolated random carrier plates are
    // not permitted. Later rifting may split these masses, but archipelagos are no longer an
    // initial-condition artifact.
    let nucleus_target = (3 + (unit_random(seed ^ 0xa076_1d64_78bd_642f) * 4.0).floor() as usize)
        .clamp(2, 6)
        .min(count.max(1));
    let mut nuclei = Vec::<usize>::new();
    if count > 0 {
        let first = (0..count)
            .max_by(|left, right| {
                let left_score = (ancestral.plates[*left].area_steradians / total_area).sqrt()
                    * 0.72
                    + unit_random(seed ^ (*left as u64).wrapping_mul(0xe703_7ed1_a0b4_28db)) * 0.28;
                let right_score = (ancestral.plates[*right].area_steradians / total_area).sqrt()
                    * 0.72
                    + unit_random(seed ^ (*right as u64).wrapping_mul(0xe703_7ed1_a0b4_28db))
                        * 0.28;
                left_score
                    .total_cmp(&right_score)
                    .then_with(|| right.cmp(left))
            })
            .unwrap_or(0);
        nuclei.push(first);
    }
    while nuclei.len() < nucleus_target {
        let mut best = None;
        let mut best_score = f64::NEG_INFINITY;
        for plate in 0..count {
            if nuclei.contains(&plate) {
                continue;
            }
            let position = topology.unit_position(ancestral.plates[plate].seed_sample);
            let separation = nuclei
                .iter()
                .map(|nucleus| {
                    arc_radians(
                        position,
                        topology.unit_position(ancestral.plates[*nucleus].seed_sample),
                    )
                })
                .fold(std::f64::consts::PI, f64::min);
            let area_bonus = (ancestral.plates[plate].area_steradians / total_area).sqrt() * 0.22;
            let jitter =
                unit_random(seed ^ (plate as u64).wrapping_mul(0x8ebc_6af0_9c88_c6e3)) * 0.04;
            let score = separation + area_bonus + jitter;
            if score > best_score || (score == best_score && Some(plate) < best) {
                best_score = score;
                best = Some(plate);
            }
        }
        let Some(plate) = best else { break };
        nuclei.push(plate);
    }

    let mut carriers = vec![false; count];
    let mut assemblies = vec![u16::MAX; count];
    let mut carrier_area = 0.0_f64;
    let mut assembly_area = vec![0.0_f64; nuclei.len()];
    for (assembly, plate) in nuclei.iter().copied().enumerate() {
        if !carriers[plate] {
            carriers[plate] = true;
            assemblies[plate] = assembly as u16;
            carrier_area += ancestral.plates[plate].area_steradians;
            assembly_area[assembly] += ancestral.plates[plate].area_steradians;
        }
    }

    while carrier_area < target_area {
        let mut best: Option<(f64, usize, u16)> = None;
        for plate in 0..count {
            if carriers[plate] {
                continue;
            }
            let mut by_assembly = BTreeMap::<u16, [u32; 4]>::new();
            for neighbor in &adjacency[plate] {
                if !carriers[*neighbor] {
                    continue;
                }
                let assembly = assemblies[*neighbor];
                let pair = if plate < *neighbor {
                    (plate as u16, *neighbor as u16)
                } else {
                    (*neighbor as u16, plate as u16)
                };
                let kinds = pair_kinds.get(&pair).copied().unwrap_or([0; 3]);
                let totals = by_assembly.entry(assembly).or_insert([0; 4]);
                totals[0] += kinds[0];
                totals[1] += kinds[1];
                totals[2] += kinds[2];
                totals[3] += kinds.iter().sum::<u32>();
            }
            for (assembly, totals) in by_assembly {
                let shared = f64::from(totals[3].max(1));
                let convergence = f64::from(totals[0]) / shared;
                let divergence = f64::from(totals[1]) / shared;
                let transform = f64::from(totals[2]) / shared;
                let contact = shared / f64::from(plate_perimeter[plate].max(1));
                let assembly_fraction = assembly_area[assembly as usize] / target_area.max(1.0e-12);
                let dominance_penalty = ((assembly_fraction - 0.52).max(0.0) / 0.48).powi(2);
                let jitter = unit_random(
                    seed ^ (plate as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                        ^ u64::from(assembly).wrapping_mul(0xbf58_476d_1ce4_e5b9),
                ) * 0.025;
                let score = contact * 1.65 + convergence * 0.34 + transform * 0.10
                    - divergence * 0.58
                    - dominance_penalty * 0.25
                    + jitter;
                let candidate = (score, plate, assembly);
                if best
                    .map(|current| {
                        candidate.0 > current.0
                            || (candidate.0 == current.0
                                && (candidate.1, candidate.2) < (current.1, current.2))
                    })
                    .unwrap_or(true)
                {
                    best = Some(candidate);
                }
            }
        }
        let Some((_, plate, assembly)) = best else {
            break;
        };
        carriers[plate] = true;
        assemblies[plate] = assembly;
        carrier_area += ancestral.plates[plate].area_steradians;
        assembly_area[assembly as usize] += ancestral.plates[plate].area_steradians;
    }

    // Non-carriers never participate in continental depth propagation, but give every ancestral
    // plate a dense assembly id for deterministic diagnostics and downstream indexing.
    let mut next_assembly = nuclei.len() as u16;
    for plate in 0..count {
        if assemblies[plate] == u16::MAX {
            assemblies[plate] = next_assembly;
            next_assembly = next_assembly.saturating_add(1);
        }
    }
    (carriers, assemblies)
}

fn oceanic_spreading_distance_km<T: PlanetTopology>(
    topology: &T,
    sample_origin: &[u16],
    sample_carrier: &[bool],
    ancestral: &TectonicModel,
    planet: PlanetPhysicalParameters,
) -> Vec<f64> {
    let count = topology.sample_count() as usize;
    let mut distance_km = vec![f64::INFINITY; count];
    let mut queued = vec![false; count];
    let mut queue = VecDeque::<u32>::new();

    let register_seed = |sample: u32, plate: u16, distance_km: &mut [f64], queued: &mut [bool], queue: &mut VecDeque<u32>| {
        let index = sample as usize;
        if !sample_carrier[index]
            && sample_origin[index] == plate
            && distance_km[index] > 0.0
        {
            distance_km[index] = 0.0;
            if !queued[index] {
                queued[index] = true;
                queue.push_back(sample);
            }
        }
    };

    for boundary in ancestral
        .boundaries
        .iter()
        .filter(|boundary| boundary.kind == PlateBoundaryKind::Divergent)
    {
        for (sample, plate) in [
            (boundary.sample_a, boundary.plate_a),
            (boundary.sample_b, boundary.plate_b),
        ] {
            register_seed(sample, plate, &mut distance_km, &mut queued, &mut queue);
            for neighbor in topology.neighbors(sample) {
                register_seed(*neighbor, plate, &mut distance_km, &mut queued, &mut queue);
            }
        }
    }

    while let Some(sample) = queue.pop_front() {
        let index = sample as usize;
        queued[index] = false;
        let origin = sample_origin[index];
        let neighbors = topology.neighbors(sample);
        let lengths = topology.neighbor_arc_lengths_rad(sample);
        for neighbor_index in 0..neighbors.len() {
            let neighbor = neighbors[neighbor_index];
            let ni = neighbor as usize;
            if sample_carrier[ni] || sample_origin[ni] != origin {
                continue;
            }
            let candidate =
                distance_km[index] + lengths[neighbor_index] * planet.radius_m / 1000.0;
            if candidate + 1.0e-9 < distance_km[ni] {
                distance_km[ni] = candidate;
                if !queued[ni] {
                    queued[ni] = true;
                    queue.push_back(neighbor);
                }
            }
        }
    }

    distance_km
}

fn build_plate_owned_crust<T: PlanetTopology>(
    topology: &T,
    fragment_ids: &[u16],
    fragments: &mut [CrustFragment],
    seed: u64,
    planet: PlanetPhysicalParameters,
    ancestral: &TectonicModel,
) -> (Vec<u8>, Vec<f32>) {
    let (plate_carriers, plate_assemblies) =
        build_continental_assemblies(topology, ancestral, seed);
    let count = topology.sample_count() as usize;
    let sample_origin = fragment_ids
        .iter()
        .map(|fragment| fragments[*fragment as usize].origin_plate_id)
        .collect::<Vec<_>>();
    let sample_assembly = sample_origin
        .iter()
        .map(|origin| plate_assemblies[*origin as usize])
        .collect::<Vec<_>>();
    let sample_carrier = sample_origin
        .iter()
        .map(|origin| plate_carriers[*origin as usize])
        .collect::<Vec<_>>();

    // Distance inward from the edge of an assembled continental material domain. Internal
    // fragment and welded ancestral-plate contacts are deliberately ignored here: they remain
    // provenance/suture structure, not automatic seaways.
    let mut depth = vec![u16::MAX; count];
    let mut queue = VecDeque::<u32>::new();
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        if !sample_carrier[index] {
            continue;
        }
        let assembly = sample_assembly[index];
        let edge = topology.neighbors(sample).iter().any(|neighbor| {
            let ni = *neighbor as usize;
            !sample_carrier[ni] || sample_assembly[ni] != assembly
        });
        if edge {
            depth[index] = 0;
            queue.push_back(sample);
        }
    }
    while let Some(sample) = queue.pop_front() {
        let index = sample as usize;
        let assembly = sample_assembly[index];
        let next_depth = depth[index].saturating_add(1);
        for neighbor in topology.neighbors(sample) {
            let ni = *neighbor as usize;
            if !sample_carrier[ni] || sample_assembly[ni] != assembly || depth[ni] != u16::MAX {
                continue;
            }
            depth[ni] = next_depth;
            queue.push_back(*neighbor);
        }
    }

    // Convert the coarse mesh scale into a bounded physical continental-margin width. The
    // outer carrier ring is transitional crust; oceanic crust begins in the neighboring
    // non-carrier material domain rather than being painted as an automatic moat inside every
    // continental assembly.
    let mut edge_length_sum_km = 0.0_f64;
    let mut edge_length_count = 0_u64;
    for sample in 0..topology.sample_count() {
        let position = topology.unit_position(sample);
        for neighbor in topology.neighbors(sample) {
            if *neighbor <= sample {
                continue;
            }
            edge_length_sum_km +=
                arc_radians(position, topology.unit_position(*neighbor)) * planet.radius_m / 1000.0;
            edge_length_count += 1;
        }
    }
    let mean_edge_km = if edge_length_count > 0 {
        edge_length_sum_km / edge_length_count as f64
    } else {
        350.0
    };
    let transition_steps = (360.0 / mean_edge_km.max(1.0)).ceil().clamp(1.0, 3.0) as u16;

    // Oceanic chronology propagates through the material domain that owns the crust instead of
    // using unrestricted great-circle distance to any ridge. This prevents unrelated spreading
    // systems from stamping concentric age/bathymetry kernels through intervening plates.
    let spreading_distance_km = oceanic_spreading_distance_km(
        topology,
        &sample_origin,
        &sample_carrier,
        ancestral,
        planet,
    );

    let mut crust_kind = vec![CrustKind::Oceanic as u8; count];
    let mut birth_age = vec![0.0_f32; count];
    let mut kind_counts = vec![[0_u32; 3]; fragments.len()];
    let mut thickness_sums = vec![0.0_f64; fragments.len()];
    let mut density_sums = vec![0.0_f64; fragments.len()];

    for sample in 0..topology.sample_count() {
        let sample_index = sample as usize;
        let fragment = fragment_ids[sample_index] as usize;
        let edge_warp = unit_random(
            seed ^ u64::from(sample).wrapping_mul(0xe703_7ed1_a0b4_28db) ^ (fragment as u64),
        );
        let kind = if sample_carrier[sample_index] {
            let local_depth = depth[sample_index];
            if local_depth != u16::MAX
                && (local_depth.saturating_add(1) < transition_steps
                    || (local_depth < transition_steps && edge_warp >= 0.35))
            {
                CrustKind::Transitional
            } else {
                CrustKind::Continental
            }
        } else {
            CrustKind::Oceanic
        };
        crust_kind[sample_index] = kind as u8;

        let local_random = unit_random(
            seed ^ u64::from(sample).wrapping_mul(0x8ebc_6af0_9c88_c6e3) ^ 0x243f_6a88_85a3_08d3,
        );
        let (age, thickness, density, kind_index) = match kind {
            CrustKind::Continental => (
                650.0
                    + 2800.0
                        * unit_random(seed ^ (fragment as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)),
                31.0 + 13.0 * local_random,
                2720.0 + 90.0 * (1.0 - local_random),
                2,
            ),
            CrustKind::Transitional => (
                120.0
                    + 900.0
                        * unit_random(seed ^ (fragment as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)),
                15.0 + 14.0 * local_random,
                2820.0 + 110.0 * (1.0 - local_random),
                1,
            ),
            CrustKind::Oceanic => {
                let age = if spreading_distance_km[sample_index].is_finite() {
                    (spreading_distance_km[sample_index] / OCEANIC_SPREADING_KM_PER_MYR).clamp(0.0, 220.0)
                } else {
                    110.0 + 90.0 * local_random
                };
                (
                    age,
                    5.8 + 1.8 * local_random,
                    2870.0 + 120.0 * (1.0 - local_random),
                    0,
                )
            }
        };
        birth_age[sample_index] = age as f32;
        kind_counts[fragment][kind_index] += 1;
        thickness_sums[fragment] += thickness;
        density_sums[fragment] += density;
    }

    for (index, fragment) in fragments.iter_mut().enumerate() {
        let dominant = kind_counts[index]
            .iter()
            .enumerate()
            .max_by_key(|(kind, count)| (**count, *kind))
            .map(|(kind, _)| kind)
            .unwrap_or(0);
        fragment.dominant_crust_kind = match dominant {
            2 => CrustKind::Continental as u8,
            1 => CrustKind::Transitional as u8,
            _ => CrustKind::Oceanic as u8,
        };
        let count = f64::from(fragment.sample_count.max(1));
        fragment.mean_thickness_km = (thickness_sums[index] / count) as f32;
        fragment.mean_density_kg_per_m3 = (density_sums[index] / count) as f32;
        fragment.birth_age_myr = if fragment.dominant_crust_kind == CrustKind::Continental as u8 {
            (650.0
                + 2800.0 * unit_random(seed ^ (index as u64).wrapping_mul(0x517c_c1b7_2722_0a95)))
                as f32
        } else {
            (80.0 + 140.0 * unit_random(seed ^ (index as u64).wrapping_mul(0x6a09_e667_f3bc_c909)))
                as f32
        };
    }

    (crust_kind, birth_age)
}

fn group_ancestral_plates(
    ancestral: &TectonicModel,
    modern_plate_count: u16,
    seed: u64,
) -> Vec<u16> {
    let count = ancestral.plates.len();
    let mut groups = (0..count as u16).collect::<Vec<_>>();
    let mut active = count;
    let total_area = ancestral
        .plates
        .iter()
        .map(|plate| plate.area_steradians)
        .sum::<f64>()
        .max(1.0e-12);

    while active > modern_plate_count as usize {
        let mut area = BTreeMap::<u16, f64>::new();
        let mut velocity_sum = BTreeMap::<u16, [f64; 3]>::new();
        let mut members = BTreeMap::<u16, u32>::new();
        for (plate_index, plate) in ancestral.plates.iter().enumerate() {
            let group = groups[plate_index];
            *area.entry(group).or_insert(0.0) += plate.area_steradians;
            *members.entry(group).or_insert(0) += 1;
            let sum = velocity_sum.entry(group).or_insert([0.0; 3]);
            for axis in 0..3 {
                sum[axis] += plate.angular_velocity_rad_per_myr[axis] * plate.area_steradians;
            }
        }

        let mut contacts = BTreeMap::<(u16, u16), u32>::new();
        let mut perimeter = BTreeMap::<u16, u32>::new();
        for boundary in &ancestral.boundaries {
            let group_a = groups[boundary.plate_a as usize];
            let group_b = groups[boundary.plate_b as usize];
            if group_a == group_b {
                continue;
            }
            let pair = if group_a < group_b {
                (group_a, group_b)
            } else {
                (group_b, group_a)
            };
            *contacts.entry(pair).or_insert(0) += 1;
            *perimeter.entry(group_a).or_insert(0) += 1;
            *perimeter.entry(group_b).or_insert(0) += 1;
        }

        let enclosed_after_merge = |merge_a: u16, merge_b: u16| -> usize {
            let keep = merge_a.min(merge_b);
            let remove = merge_a.max(merge_b);
            let mut neighbors = BTreeMap::<u16, BTreeSet<u16>>::new();
            for &(left, right) in contacts.keys() {
                let mapped_left = if left == remove { keep } else { left };
                let mapped_right = if right == remove { keep } else { right };
                if mapped_left == mapped_right {
                    continue;
                }
                neighbors
                    .entry(mapped_left)
                    .or_default()
                    .insert(mapped_right);
                neighbors
                    .entry(mapped_right)
                    .or_default()
                    .insert(mapped_left);
            }
            neighbors
                .iter()
                .filter(|(group, adjacent)| {
                    **group != keep && adjacent.len() == 1 && adjacent.contains(&keep)
                })
                .count()
        };

        let mut best: Option<(f64, u16, u16)> = None;
        for (&(group_a, group_b), &shared_contact) in &contacts {
            let area_a = area[&group_a];
            let area_b = area[&group_b];
            let velocity_a_sum = velocity_sum[&group_a];
            let velocity_b_sum = velocity_sum[&group_b];
            let velocity_a = [
                velocity_a_sum[0] / area_a.max(1.0e-12),
                velocity_a_sum[1] / area_a.max(1.0e-12),
                velocity_a_sum[2] / area_a.max(1.0e-12),
            ];
            let velocity_b = [
                velocity_b_sum[0] / area_b.max(1.0e-12),
                velocity_b_sum[1] / area_b.max(1.0e-12),
                velocity_b_sum[2] / area_b.max(1.0e-12),
            ];
            let velocity_cost = vector_distance(velocity_a, velocity_b);
            let merged_area = area_a + area_b;
            let merged_fraction = merged_area / total_area;
            let balance_cost = (area_a - area_b).abs() / merged_area.max(1.0e-12);
            let contact_fraction = f64::from(shared_contact)
                / f64::from(perimeter[&group_a].min(perimeter[&group_b]).max(1));
            let compactness_cost = 1.0 - contact_fraction.clamp(0.0, 1.0);
            let dominance_cost = ((merged_fraction - 0.30).max(0.0) / 0.20).powi(2);
            let enclosure_cost = enclosed_after_merge(group_a, group_b) as f64;
            let member_total = f64::from(members[&group_a] + members[&group_b]);
            let member_imbalance =
                f64::from(members[&group_a].max(members[&group_b])) / member_total.max(1.0);
            let tie = unit_random(
                seed ^ u64::from(group_a).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                    ^ u64::from(group_b).wrapping_mul(0xbf58_476d_1ce4_e5b9),
            ) * 1.0e-6;
            let score = velocity_cost * 32.0
                + compactness_cost * 0.30
                + balance_cost * 0.05
                + dominance_cost * 1.80
                + enclosure_cost * 4.0
                + member_imbalance * 0.04
                + tie;
            let candidate = (score, group_a.min(group_b), group_a.max(group_b));
            if best.map(|current| candidate < current).unwrap_or(true) {
                best = Some(candidate);
            }
        }
        let Some((_, keep, remove)) = best else {
            break;
        };
        for group in &mut groups {
            if *group == remove {
                *group = keep;
            }
        }
        active -= 1;
    }

    let mut compact = BTreeMap::<u16, u16>::new();
    for group in &groups {
        if !compact.contains_key(group) {
            let next = compact.len() as u16;
            compact.insert(*group, next);
        }
    }
    groups.into_iter().map(|group| compact[&group]).collect()
}

fn event_kind_for_boundary(
    kind: PlateBoundaryKind,
    crust_a: u8,
    crust_b: u8,
) -> HistoricalEventKind {
    match kind {
        PlateBoundaryKind::Divergent => {
            if crust_a == CrustKind::Oceanic as u8 && crust_b == CrustKind::Oceanic as u8 {
                HistoricalEventKind::Spreading
            } else {
                HistoricalEventKind::Rift
            }
        }
        PlateBoundaryKind::Transform => HistoricalEventKind::Transform,
        PlateBoundaryKind::Convergent => {
            if crust_a != CrustKind::Oceanic as u8 && crust_b != CrustKind::Oceanic as u8 {
                HistoricalEventKind::Collision
            } else if crust_a == CrustKind::Oceanic as u8 && crust_b == CrustKind::Oceanic as u8 {
                HistoricalEventKind::Subduction
            } else {
                HistoricalEventKind::Accretion
            }
        }
    }
}

fn build_events(
    ancestral: &TectonicModel,
    fragment_ids: &[u16],
    crust_kind: &[u8],
    groups: &[u16],
    seed: u64,
) -> Vec<HistoricalTectonicEvent> {
    let mut pair_representatives = BTreeMap::<(u16, u16), usize>::new();
    for (index, boundary) in ancestral.boundaries.iter().enumerate() {
        let pair = if boundary.plate_a < boundary.plate_b {
            (boundary.plate_a, boundary.plate_b)
        } else {
            (boundary.plate_b, boundary.plate_a)
        };
        pair_representatives.entry(pair).or_insert(index);
    }

    let mut events = Vec::new();
    for ((plate_a, plate_b), boundary_index) in pair_representatives {
        let boundary = &ancestral.boundaries[boundary_index];
        let stream = seed
            ^ u64::from(plate_a).wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ u64::from(plate_b).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        let epoch = (random::mix64(stream) % 8) as u8;
        let age_myr = 18.0
            + f32::from(epoch) * 42.0
            + (unit_random(stream ^ 0x94d0_49bb_1331_11eb) * 28.0) as f32;
        let rate = boundary
            .normal_rate_m_per_year
            .abs()
            .max(boundary.shear_rate_m_per_year.abs());
        let displacement_km = (rate * f64::from(age_myr) * 1000.0).clamp(0.0, 5000.0) as f32;
        let kind = event_kind_for_boundary(
            boundary.kind,
            crust_kind[boundary.sample_a as usize],
            crust_kind[boundary.sample_b as usize],
        );
        events.push(HistoricalTectonicEvent {
            id: events.len() as u32,
            kind,
            epoch,
            age_myr,
            plate_a,
            plate_b,
            fragment_a: fragment_ids[boundary.sample_a as usize],
            fragment_b: fragment_ids[boundary.sample_b as usize],
            displacement_km,
            strength: (0.25 + (rate / 0.08).clamp(0.0, 1.0) * 0.75) as f32,
            geometry_sample_a: boundary.sample_a,
            geometry_sample_b: boundary.sample_b,
        });
    }

    let mut members_by_group = BTreeMap::<u16, Vec<u16>>::new();
    for (origin, group) in groups.iter().copied().enumerate() {
        members_by_group
            .entry(group)
            .or_default()
            .push(origin as u16);
    }
    for members in members_by_group.values() {
        if members.len() <= 1 {
            continue;
        }
        let anchor = members[0];
        for &captured in &members[1..] {
            let stream = seed
                ^ u64::from(anchor).wrapping_mul(0xd6e8_feb8_6659_fd93)
                ^ u64::from(captured).wrapping_mul(0xa5a3_56d9_2d85_85ad);
            let age_myr = (12.0 + unit_random(stream) * 120.0) as f32;
            let fragment_a = fragment_ids
                .iter()
                .position(|fragment| {
                    ancestral
                        .plate_ids
                        .get(
                            fragment_ids
                                .iter()
                                .position(|candidate| candidate == fragment)
                                .unwrap_or(0),
                        )
                        .copied()
                        == Some(anchor)
                })
                .map(|sample| fragment_ids[sample])
                .unwrap_or(0);
            let fragment_b = ancestral
                .plate_ids
                .iter()
                .position(|plate| *plate == captured)
                .map(|sample| fragment_ids[sample])
                .unwrap_or(fragment_a);
            events.push(HistoricalTectonicEvent {
                id: events.len() as u32,
                kind: HistoricalEventKind::Capture,
                epoch: 0,
                age_myr,
                plate_a: anchor,
                plate_b: captured,
                fragment_a,
                fragment_b,
                displacement_km: 0.0,
                strength: 0.5,
                geometry_sample_a: ancestral.plates[anchor as usize].seed_sample,
                geometry_sample_b: ancestral.plates[captured as usize].seed_sample,
            });
        }
    }

    events
}

fn history_hash(model: &HistoricalLithosphereModel) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, HISTORICAL_LITHOSPHERE_STAGE_ID.as_bytes());
    hash = fnv_update(hash, &model.stage.derived_seed.to_le_bytes());
    for values in [
        &model.origin_plate_ids,
        &model.fragment_ids,
        &model.current_plate_ids,
    ] {
        for value in values {
            hash = fnv_update(hash, &value.to_le_bytes());
        }
    }
    hash = fnv_update(hash, &model.crust_kind);
    for age in &model.crust_birth_age_myr {
        hash = fnv_update(hash, &age.to_bits().to_le_bytes());
    }
    for fragment in &model.fragments {
        hash = fnv_update(hash, &fragment.id.to_le_bytes());
        hash = fnv_update(hash, &fragment.origin_plate_id.to_le_bytes());
        hash = fnv_update(hash, &fragment.current_plate_id.to_le_bytes());
        hash = fnv_update(hash, &fragment.seed_sample.to_le_bytes());
    }
    for event in &model.events {
        hash = fnv_update(hash, &event.id.to_le_bytes());
        hash = fnv_update(hash, &[event.kind as u8, event.epoch]);
        hash = fnv_update(hash, &event.age_myr.to_bits().to_le_bytes());
        hash = fnv_update(hash, &event.plate_a.to_le_bytes());
        hash = fnv_update(hash, &event.plate_b.to_le_bytes());
    }
    hash
}

pub fn generate_historical_lithosphere<T: PlanetTopology>(
    topology: &T,
    request: &HistoricalLithosphereRequest,
    planet: PlanetPhysicalParameters,
) -> Result<HistoricalLithosphereModel, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    let ancestral_count =
        ancestral_plate_count(request.modern_plate_count, topology.sample_count())?;
    let stage_seed = random::derive_stage_seed(&request.seed, HISTORICAL_LITHOSPHERE_NAMESPACE);
    let ancestral_seed = random::derive_stage_seed(&request.seed, ANCESTRAL_TECTONICS_NAMESPACE);
    let fragment_seed = random::derive_stage_seed(&request.seed, FRAGMENT_NAMESPACE);
    let crust_seed = random::derive_stage_seed(&request.seed, CRUST_NAMESPACE);
    let grouping_seed = random::derive_stage_seed(&request.seed, MODERN_GROUPING_NAMESPACE);
    let event_seed = random::derive_stage_seed(&request.seed, EVENT_NAMESPACE);

    let ancestral = generate_tectonics(
        topology,
        &TectonicsRequest::new(
            format!("{}:{ancestral_seed:016x}", request.seed),
            ancestral_count,
        ),
        planet,
    )?;
    let origin_plate_ids = ancestral.plate_ids.clone();
    let (fragment_ids, mut fragments) = build_fragments(topology, &ancestral, fragment_seed);
    let (crust_kind, crust_birth_age_myr) = build_plate_owned_crust(
        topology,
        &fragment_ids,
        &mut fragments,
        crust_seed,
        planet,
        &ancestral,
    );
    let groups = group_ancestral_plates(&ancestral, request.modern_plate_count, grouping_seed);
    let current_plate_ids = origin_plate_ids
        .iter()
        .map(|origin| groups[*origin as usize])
        .collect::<Vec<_>>();
    for fragment in &mut fragments {
        fragment.current_plate_id = groups[fragment.origin_plate_id as usize];
        if fragment.current_plate_id != fragment.origin_plate_id {
            fragment.capture_age_myr = Some(
                (12.0
                    + unit_random(
                        event_seed ^ u64::from(fragment.id).wrapping_mul(0x243f_6a88_85a3_08d3),
                    ) * 120.0) as f32,
            );
        }
    }
    let events = build_events(&ancestral, &fragment_ids, &crust_kind, &groups, event_seed);

    let total_area = (0..topology.sample_count())
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>()
        .max(1.0e-12);
    let mut continental_area = 0.0;
    let mut transitional_area = 0.0;
    let mut oceanic_area = 0.0;
    let mut oceanic_age_area = 0.0;
    for sample in 0..topology.sample_count() {
        let area = topology.area_steradians(sample);
        match crust_kind[sample as usize] {
            value if value == CrustKind::Continental as u8 => continental_area += area,
            value if value == CrustKind::Transitional as u8 => transitional_area += area,
            _ => {
                oceanic_area += area;
                oceanic_age_area += area * f64::from(crust_birth_age_myr[sample as usize]);
            }
        }
    }

    let mut model = HistoricalLithosphereModel {
        stage: StageIdentity {
            id: HISTORICAL_LITHOSPHERE_STAGE_ID,
            version: HISTORICAL_LITHOSPHERE_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        ancestral_tectonics: ancestral,
        origin_plate_ids,
        fragment_ids,
        current_plate_ids,
        crust_kind,
        crust_birth_age_myr,
        fragments,
        events,
        metrics: HistoricalLithosphereMetrics {
            sample_count: topology.sample_count(),
            ancestral_plate_count: ancestral_count,
            fragment_count: 0,
            modern_plate_count: request.modern_plate_count,
            event_count: 0,
            continental_area_fraction: continental_area / total_area,
            transitional_area_fraction: transitional_area / total_area,
            oceanic_area_fraction: oceanic_area / total_area,
            mean_oceanic_age_myr: if oceanic_area > 0.0 {
                oceanic_age_area / oceanic_area
            } else {
                0.0
            },
            history_hash: 0,
        },
    };
    model.metrics.fragment_count = model.fragments.len() as u16;
    model.metrics.event_count = model.events.len() as u32;
    model.metrics.history_hash = history_hash(&model);

    if model
        .origin_plate_ids
        .iter()
        .any(|plate| usize::from(*plate) >= model.ancestral_tectonics.plates.len())
        || model
            .fragment_ids
            .iter()
            .any(|fragment| usize::from(*fragment) >= model.fragments.len())
        || model
            .current_plate_ids
            .iter()
            .any(|plate| *plate >= request.modern_plate_count)
    {
        return Err(WorldgenError::InvalidLithosphere(
            "historical lithosphere produced invalid material ownership",
        ));
    }

    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_icosphere;

    #[test]
    fn historical_lithosphere_is_deterministic_and_seed_sensitive() {
        let topology = build_icosphere(3).unwrap();
        let request = HistoricalLithosphereRequest::new("historical-regression", 12);
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let a = generate_historical_lithosphere(&topology, &request, planet).unwrap();
        let b = generate_historical_lithosphere(&topology, &request, planet).unwrap();
        assert_eq!(a.metrics.history_hash, b.metrics.history_hash);
        assert_eq!(a.origin_plate_ids, b.origin_plate_ids);
        assert_eq!(a.fragment_ids, b.fragment_ids);
        assert_eq!(a.current_plate_ids, b.current_plate_ids);

        let changed = generate_historical_lithosphere(
            &topology,
            &HistoricalLithosphereRequest::new("historical-regression-b", 12),
            planet,
        )
        .unwrap();
        assert_ne!(a.metrics.history_hash, changed.metrics.history_hash);
    }

    fn single_neighbor_plate_count<T: PlanetTopology>(
        topology: &T,
        plate_ids: &[u16],
        plate_count: u16,
    ) -> usize {
        let mut neighbors = vec![BTreeSet::<u16>::new(); plate_count as usize];
        for sample in 0..topology.sample_count() {
            let plate = plate_ids[sample as usize];
            for neighbor in topology.neighbors(sample) {
                let other = plate_ids[*neighbor as usize];
                if other != plate {
                    neighbors[plate as usize].insert(other);
                }
            }
        }
        neighbors
            .iter()
            .filter(|adjacent| adjacent.len() == 1)
            .count()
    }

    fn continental_components<T: PlanetTopology>(topology: &T, crust_kind: &[u8]) -> (usize, f64) {
        let mut visited = vec![false; topology.sample_count() as usize];
        let mut components = 0usize;
        let mut largest_area = 0.0f64;
        let total_area = (0..topology.sample_count())
            .map(|sample| topology.area_steradians(sample))
            .sum::<f64>()
            .max(1.0e-12);
        for start in 0..topology.sample_count() {
            let si = start as usize;
            if visited[si] || crust_kind[si] == CrustKind::Oceanic as u8 {
                continue;
            }
            components += 1;
            visited[si] = true;
            let mut queue = VecDeque::from([start]);
            let mut area = 0.0;
            while let Some(sample) = queue.pop_front() {
                area += topology.area_steradians(sample);
                for neighbor in topology.neighbors(sample) {
                    let ni = *neighbor as usize;
                    if !visited[ni] && crust_kind[ni] != CrustKind::Oceanic as u8 {
                        visited[ni] = true;
                        queue.push_back(*neighbor);
                    }
                }
            }
            largest_area = largest_area.max(area);
        }
        (components, largest_area / total_area)
    }

    #[test]
    fn modern_plate_synthesis_avoids_enclosed_single_neighbor_plates() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        for seed in [
            "plate-topology-a",
            "plate-topology-b",
            "interlink-wg7c",
            "plate-topology-d",
        ] {
            let model = generate_historical_lithosphere(
                &topology,
                &HistoricalLithosphereRequest::new(seed, 16),
                planet,
            )
            .unwrap();
            assert_eq!(
                single_neighbor_plate_count(&topology, &model.current_plate_ids, 16),
                0,
                "seed {seed} produced a modern plate enclosed by one neighbor"
            );
        }
    }

    #[test]
    fn continental_assemblies_are_broader_than_fragment_islands() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        for seed in [
            "continent-assembly-a",
            "interlink-wg7c",
            "continent-assembly-c",
        ] {
            let model = generate_historical_lithosphere(
                &topology,
                &HistoricalLithosphereRequest::new(seed, 16),
                planet,
            )
            .unwrap();
            let (components, largest_fraction) =
                continental_components(&topology, &model.crust_kind);
            assert!(
                components < model.metrics.fragment_count as usize / 2,
                "seed {seed} remained fragment-island dominated"
            );
            assert!(
                largest_fraction >= 0.06,
                "seed {seed} lacked a substantial assembled continent"
            );
            assert!(
                (0.20..=0.50).contains(&model.metrics.continental_area_fraction),
                "seed {seed} continental coverage escaped the synthesis envelope: {}",
                model.metrics.continental_area_fraction
            );
            assert!(
                (0.02..=0.18).contains(&model.metrics.transitional_area_fraction),
                "seed {seed} transitional margins collapsed or dominated: {}",
                model.metrics.transitional_area_fraction
            );
            assert!(
                (0.38..=0.75).contains(&model.metrics.oceanic_area_fraction),
                "seed {seed} ocean coverage escaped the synthesis envelope: {}",
                model.metrics.oceanic_area_fraction
            );
        }
    }

    #[test]
    fn historical_lithosphere_preserves_three_identity_levels() {
        let topology = build_icosphere(3).unwrap();
        let model = generate_historical_lithosphere(
            &topology,
            &HistoricalLithosphereRequest::new("identity-levels", 10),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
        assert!(model.metrics.ancestral_plate_count >= model.metrics.modern_plate_count);
        assert!(model.metrics.fragment_count >= model.metrics.ancestral_plate_count);
        assert_eq!(
            model.origin_plate_ids.len(),
            topology.sample_count() as usize
        );
        assert_eq!(model.fragment_ids.len(), topology.sample_count() as usize);
        assert_eq!(
            model.current_plate_ids.len(),
            topology.sample_count() as usize
        );
        assert!(model.metrics.event_count > 0);
        assert!(model.metrics.oceanic_area_fraction > 0.0);
        assert!(model.metrics.continental_area_fraction > 0.0);
        assert!(model.metrics.mean_oceanic_age_myr.is_finite());
    }
}
