use crate::{
    random, PlanetPhysicalParameters, PlanetTopology, PlateBoundaryKind, StageIdentity,
    TectonicModel, WorldgenError,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::f64::consts::PI;

pub const GEOLOGY_STAGE_ID: &str = "geology:crust-history";
pub const GEOLOGY_STAGE_VERSION: u32 = 1;
const GEOLOGY_NAMESPACE: &str = "worldgen:geology:crust-history:v1";
const CRUST_PROVINCES_NAMESPACE: &str = "worldgen:geology:crust-provinces:v1";
const CRUST_PROPERTIES_NAMESPACE: &str = "worldgen:geology:crust-properties:v1";
const GEOLOGICAL_HISTORY_NAMESPACE: &str = "worldgen:geology:history:v1";
const OCEANIC_PROVINCE_BIT: u16 = 0x8000;
const MAX_CONTINENTAL_PROVINCES: usize = 24;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrustKind {
    Oceanic = 1,
    Transitional = 2,
    Continental = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlateScaleClass {
    Major = 1,
    Intermediate = 2,
    Minor = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeologicalBoundaryRegime {
    OceanicSubduction = 1,
    OceanContinentSubduction = 2,
    ContinentalCollision = 3,
    OceanicRidge = 4,
    ContinentalRift = 5,
    TransitionalDivergence = 6,
    Transform = 7,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubductionPolarity {
    None = 0,
    PlateA = 1,
    PlateB = 2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeologyRequest {
    pub seed: String,
}

impl GeologyRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeologicalBoundary {
    pub sample_a: u32,
    pub sample_b: u32,
    pub plate_a: u16,
    pub plate_b: u16,
    pub regime: GeologicalBoundaryRegime,
    pub subduction_polarity: SubductionPolarity,
    pub normal_rate_m_per_year: f64,
    pub shear_rate_m_per_year: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlateSummary {
    pub plate_id: u16,
    pub area_steradians: f64,
    pub area_fraction: f64,
    pub scale_class: PlateScaleClass,
    pub continental_fraction: f64,
    pub transitional_fraction: f64,
    pub oceanic_fraction: f64,
    pub mean_crust_age_myr: f64,
    pub mean_crust_thickness_km: f64,
    pub convergent_boundary_fraction: f64,
    pub divergent_boundary_fraction: f64,
    pub transform_boundary_fraction: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeologyMetrics {
    pub sample_count: u32,
    pub continental_area_fraction: f64,
    pub transitional_area_fraction: f64,
    pub oceanic_area_fraction: f64,
    pub mean_continental_age_myr: f64,
    pub mean_oceanic_age_myr: f64,
    pub mean_continental_thickness_km: f64,
    pub mean_oceanic_thickness_km: f64,
    pub oceanic_subduction_edges: u32,
    pub ocean_continent_subduction_edges: u32,
    pub continental_collision_edges: u32,
    pub oceanic_ridge_edges: u32,
    pub continental_rift_edges: u32,
    pub transitional_divergence_edges: u32,
    pub transform_edges: u32,
    pub geology_hash: u64,
}

impl GeologyMetrics {
    pub fn geology_hash_hex(&self) -> String {
        format!("{:016x}", self.geology_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CrustalModel {
    pub stage: StageIdentity,
    pub province_seed: u64,
    pub property_seed: u64,
    pub history_seed: u64,
    pub crust_kind: Vec<u8>,
    pub crust_province_id: Vec<u16>,
    pub crust_age_myr: Vec<f32>,
    pub crust_thickness_km: Vec<f32>,
    pub crust_density_kg_per_m3: Vec<f32>,
    pub buoyancy_index: Vec<f32>,
    pub orogenic_history: Vec<f32>,
    pub rift_history: Vec<f32>,
    pub ridge_history: Vec<f32>,
    pub subduction_history: Vec<f32>,
    pub trench_history: Vec<f32>,
    pub volcanic_arc_history: Vec<f32>,
    pub transform_history: Vec<f32>,
    pub subsidence_history: Vec<f32>,
    pub basin_potential: Vec<f32>,
    pub crustal_strain: Vec<f32>,
    pub boundaries: Vec<GeologicalBoundary>,
    pub plate_summaries: Vec<PlateSummary>,
    pub metrics: GeologyMetrics,
}

impl CrustalModel {
    pub fn boundary_regimes(&self) -> Vec<u8> {
        self.boundaries
            .iter()
            .map(|edge| edge.regime as u8)
            .collect()
    }
    pub fn subduction_polarities(&self) -> Vec<u8> {
        self.boundaries
            .iter()
            .map(|edge| edge.subduction_polarity as u8)
            .collect()
    }
    pub fn plate_scale_classes(&self) -> Vec<u8> {
        self.plate_summaries
            .iter()
            .map(|plate| plate.scale_class as u8)
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct DistanceFrontier {
    distance_rad: f64,
    source: u32,
    sample: u32,
}

impl PartialEq for DistanceFrontier {
    fn eq(&self, other: &Self) -> bool {
        self.distance_rad.to_bits() == other.distance_rad.to_bits()
            && self.source == other.source
            && self.sample == other.sample
    }
}
impl Eq for DistanceFrontier {}
impl Ord for DistanceFrontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance_rad
            .total_cmp(&self.distance_rad)
            .then_with(|| other.source.cmp(&self.source))
            .then_with(|| other.sample.cmp(&self.sample))
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
fn arc_radians(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(a, b).clamp(-1.0, 1.0).acos()
}
fn unit_random(value: u64) -> f64 {
    ((random::mix64(value) >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0)
}
fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn smooth_random_field<T: PlanetTopology>(topology: &T, seed: u64, passes: usize) -> Vec<f64> {
    let mut values = (0..topology.sample_count())
        .map(|sample| {
            unit_random(seed ^ u64::from(sample).wrapping_mul(0x9e37_79b9_7f4a_7c15)) * 2.0 - 1.0
        })
        .collect::<Vec<_>>();
    let mut next = vec![0.0; values.len()];
    for _ in 0..passes {
        for sample in 0..topology.sample_count() {
            let neighbors = topology.neighbors(sample);
            let mean = neighbors
                .iter()
                .map(|neighbor| values[*neighbor as usize])
                .sum::<f64>()
                / neighbors.len() as f64;
            next[sample as usize] = values[sample as usize] * 0.42 + mean * 0.58;
        }
        std::mem::swap(&mut values, &mut next);
    }
    let max_abs = values
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max)
        .max(1.0e-12);
    for value in &mut values {
        *value /= max_abs;
    }
    values
}

fn select_craton_seeds<T: PlanetTopology>(topology: &T, count: usize, seed: u64) -> Vec<u32> {
    let sample_count = topology.sample_count() as usize;
    let expected_spacing = (4.0 * PI / count as f64).sqrt();
    let minimum_separation = (expected_spacing * 0.32).clamp(0.16, 0.55);
    let mut selected = Vec::with_capacity(count);
    for index in 0..count {
        let mut accepted = None;
        let mut fallback = None;
        let mut fallback_distance = f64::NEG_INFINITY;
        for attempt in 0..sample_count.clamp(512, 4096) {
            let stream = seed
                ^ (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                ^ (attempt as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            let candidate = (random::mix64(stream) % sample_count as u64) as u32;
            if selected.contains(&candidate) {
                continue;
            }
            let position = topology.unit_position(candidate);
            let distance = if selected.is_empty() {
                PI
            } else {
                selected
                    .iter()
                    .map(|other| arc_radians(position, topology.unit_position(*other)))
                    .fold(PI, f64::min)
            };
            if distance > fallback_distance {
                fallback = Some(candidate);
                fallback_distance = distance;
            }
            if distance >= minimum_separation {
                accepted = Some(candidate);
                break;
            }
        }
        if accepted.is_none() {
            for candidate in 0..topology.sample_count() {
                if selected.contains(&candidate) {
                    continue;
                }
                let position = topology.unit_position(candidate);
                let distance = if selected.is_empty() {
                    PI
                } else {
                    selected
                        .iter()
                        .map(|other| arc_radians(position, topology.unit_position(*other)))
                        .fold(PI, f64::min)
                };
                if distance > fallback_distance {
                    fallback = Some(candidate);
                    fallback_distance = distance;
                }
            }
        }
        selected.push(
            accepted
                .or(fallback)
                .expect("craton seed selection requires a topology sample"),
        );
    }
    selected
}

fn weighted_descending_threshold<T: PlanetTopology>(
    topology: &T,
    values: &[f64],
    target_fraction: f64,
) -> f64 {
    let mut order = (0..values.len()).collect::<Vec<_>>();
    order.sort_by(|left, right| {
        values[*right]
            .total_cmp(&values[*left])
            .then_with(|| left.cmp(right))
    });
    let total = (0..topology.sample_count())
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>();
    let target = total * target_fraction;
    let mut accumulated = 0.0;
    for index in order {
        accumulated += topology.area_steradians(index as u32);
        if accumulated >= target {
            return values[index];
        }
    }
    values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
}

fn crust_kind(value: u8) -> CrustKind {
    match value {
        3 => CrustKind::Continental,
        2 => CrustKind::Transitional,
        _ => CrustKind::Oceanic,
    }
}

fn build_crust_partition<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    province_seed: u64,
) -> (Vec<u8>, Vec<u16>, Vec<u32>, Vec<f64>, Vec<f64>) {
    let craton_count = (6 + (random::mix64(province_seed ^ 0x3c6e_f372_fe94_f82b) % 9) as usize)
        .min(MAX_CONTINENTAL_PROVINCES)
        .min(topology.sample_count() as usize)
        .max(1);
    let cratons = select_craton_seeds(topology, craton_count, province_seed);
    let fabric = smooth_random_field(topology, province_seed ^ 0x97c2_9b3a_5f61_13d7, 7);
    let mut radii = Vec::with_capacity(cratons.len());
    let mut craton_ages = Vec::with_capacity(cratons.len());
    for index in 0..cratons.len() {
        let radius = 0.36
            + unit_random(province_seed ^ (index as u64).wrapping_mul(0xa076_1d64_78bd_642f))
                * 0.42;
        let age = 900.0
            + unit_random(
                province_seed
                    ^ (index as u64).wrapping_mul(0xe703_7ed1_a0b4_28db)
                    ^ 0x8ebc_6af0_9c88_c6e3,
            ) * 2400.0;
        radii.push(radius);
        craton_ages.push(age);
    }

    let mut affinity = vec![f64::NEG_INFINITY; topology.sample_count() as usize];
    let mut nearest = vec![0_u16; topology.sample_count() as usize];
    for sample in 0..topology.sample_count() {
        let position = topology.unit_position(sample);
        let mut best = f64::NEG_INFINITY;
        let mut best_index = 0_usize;
        for (index, seed_sample) in cratons.iter().enumerate() {
            let distance = arc_radians(position, topology.unit_position(*seed_sample));
            let score = (radii[index] - distance) / 0.18;
            if score > best {
                best = score;
                best_index = index;
            }
        }
        affinity[sample as usize] = best + fabric[sample as usize] * 0.62;
        nearest[sample as usize] = best_index as u16;
    }

    let continental_target = 0.30 + unit_random(province_seed ^ 0x243f_6a88_85a3_08d3) * 0.12;
    let transitional_target = 0.065 + unit_random(province_seed ^ 0x1319_8a2e_0370_7344) * 0.04;
    let continental_threshold =
        weighted_descending_threshold(topology, &affinity, continental_target);
    let transition_threshold = weighted_descending_threshold(
        topology,
        &affinity,
        (continental_target + transitional_target).min(0.50),
    );

    let mut kinds = vec![CrustKind::Oceanic as u8; affinity.len()];
    let mut provinces = vec![0_u16; affinity.len()];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        if affinity[index] >= continental_threshold {
            kinds[index] = CrustKind::Continental as u8;
            provinces[index] = nearest[index];
        } else if affinity[index] >= transition_threshold {
            kinds[index] = CrustKind::Transitional as u8;
            provinces[index] = nearest[index];
        } else {
            kinds[index] = CrustKind::Oceanic as u8;
            provinces[index] = OCEANIC_PROVINCE_BIT | tectonics.plate_ids[index];
        }
    }
    (kinds, provinces, cratons, craton_ages, affinity)
}

fn initial_properties<T: PlanetTopology>(
    topology: &T,
    kinds: &[u8],
    provinces: &[u16],
    craton_ages: &[f64],
    property_seed: u64,
) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let variation = smooth_random_field(topology, property_seed, 5);
    let mut ages = vec![0.0_f32; kinds.len()];
    let mut thicknesses = vec![0.0_f32; kinds.len()];
    let mut densities = vec![0.0_f32; kinds.len()];
    let mut buoyancies = vec![0.0_f32; kinds.len()];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let noise = variation[index];
        let kind = crust_kind(kinds[index]);
        let (age, thickness, density) = match kind {
            CrustKind::Continental => {
                let base = craton_ages[provinces[index] as usize];
                let age = (base + noise * 260.0).clamp(450.0, 3500.0);
                let oldness = ((age - 450.0) / 3050.0).clamp(0.0, 1.0);
                (
                    age,
                    (32.0 + oldness * 10.0 + noise * 2.0).clamp(28.0, 46.0),
                    (2810.0 - oldness * 95.0 + noise * 18.0).clamp(2670.0, 2840.0),
                )
            }
            CrustKind::Transitional => {
                let base = craton_ages[provinces[index] as usize];
                let age = (220.0 + base * 0.24 + noise * 120.0).clamp(120.0, 1050.0);
                (
                    age,
                    (22.0 + noise * 3.5).clamp(16.0, 29.0),
                    (2880.0 + noise * 25.0).clamp(2820.0, 2930.0),
                )
            }
            CrustKind::Oceanic => {
                let age = (105.0 + noise * 65.0).clamp(8.0, 210.0);
                let normalized = (age / 220.0_f64).sqrt();
                (age, 6.2 + normalized * 1.4, 2890.0 + normalized * 105.0)
            }
        };
        ages[index] = age as f32;
        thicknesses[index] = thickness as f32;
        densities[index] = density as f32;
        buoyancies[index] = buoyancy_index(thickness, density) as f32;
    }
    (ages, thicknesses, densities, buoyancies)
}

fn buoyancy_index(thickness_km: f64, density_kg_per_m3: f64) -> f64 {
    let density_component = (2950.0 - density_kg_per_m3) / 260.0;
    let thickness_component = (thickness_km - 20.0) / 32.0 * 0.48;
    (density_component + thickness_component).clamp(-1.0, 1.0)
}

fn classify_regime(
    kind_a: CrustKind,
    kind_b: CrustKind,
    buoyancy_a: f64,
    buoyancy_b: f64,
    tectonic_kind: PlateBoundaryKind,
) -> (GeologicalBoundaryRegime, SubductionPolarity) {
    match tectonic_kind {
        PlateBoundaryKind::Transform => (
            GeologicalBoundaryRegime::Transform,
            SubductionPolarity::None,
        ),
        PlateBoundaryKind::Divergent => {
            if matches!(kind_a, CrustKind::Oceanic) && matches!(kind_b, CrustKind::Oceanic) {
                (
                    GeologicalBoundaryRegime::OceanicRidge,
                    SubductionPolarity::None,
                )
            } else if matches!(kind_a, CrustKind::Continental)
                && matches!(kind_b, CrustKind::Continental)
            {
                (
                    GeologicalBoundaryRegime::ContinentalRift,
                    SubductionPolarity::None,
                )
            } else {
                (
                    GeologicalBoundaryRegime::TransitionalDivergence,
                    SubductionPolarity::None,
                )
            }
        }
        PlateBoundaryKind::Convergent => match (kind_a, kind_b) {
            (CrustKind::Continental, CrustKind::Continental) => (
                GeologicalBoundaryRegime::ContinentalCollision,
                SubductionPolarity::None,
            ),
            (CrustKind::Oceanic, CrustKind::Continental)
            | (CrustKind::Oceanic, CrustKind::Transitional) => (
                GeologicalBoundaryRegime::OceanContinentSubduction,
                SubductionPolarity::PlateA,
            ),
            (CrustKind::Continental, CrustKind::Oceanic)
            | (CrustKind::Transitional, CrustKind::Oceanic) => (
                GeologicalBoundaryRegime::OceanContinentSubduction,
                SubductionPolarity::PlateB,
            ),
            (CrustKind::Oceanic, CrustKind::Oceanic) => {
                let polarity = if buoyancy_a <= buoyancy_b {
                    SubductionPolarity::PlateA
                } else {
                    SubductionPolarity::PlateB
                };
                (GeologicalBoundaryRegime::OceanicSubduction, polarity)
            }
            _ => {
                if (buoyancy_a - buoyancy_b).abs() > 0.28 {
                    let polarity = if buoyancy_a < buoyancy_b {
                        SubductionPolarity::PlateA
                    } else {
                        SubductionPolarity::PlateB
                    };
                    (GeologicalBoundaryRegime::OceanContinentSubduction, polarity)
                } else {
                    (
                        GeologicalBoundaryRegime::ContinentalCollision,
                        SubductionPolarity::None,
                    )
                }
            }
        },
    }
}

fn build_geological_boundaries(
    kinds: &[u8],
    buoyancies: &[f32],
    tectonics: &TectonicModel,
) -> Vec<GeologicalBoundary> {
    tectonics
        .boundaries
        .iter()
        .map(|edge| {
            let kind_a = crust_kind(kinds[edge.sample_a as usize]);
            let kind_b = crust_kind(kinds[edge.sample_b as usize]);
            let (regime, polarity) = classify_regime(
                kind_a,
                kind_b,
                f64::from(buoyancies[edge.sample_a as usize]),
                f64::from(buoyancies[edge.sample_b as usize]),
                edge.kind,
            );
            GeologicalBoundary {
                sample_a: edge.sample_a,
                sample_b: edge.sample_b,
                plate_a: edge.plate_a,
                plate_b: edge.plate_b,
                regime,
                subduction_polarity: polarity,
                normal_rate_m_per_year: edge.normal_rate_m_per_year,
                shear_rate_m_per_year: edge.shear_rate_m_per_year,
            }
        })
        .collect()
}

fn multi_source_distance<T: PlanetTopology>(
    topology: &T,
    sources: &[u32],
    allowed: Option<&[bool]>,
) -> (Vec<f64>, Vec<u32>) {
    let mut distance = vec![f64::INFINITY; topology.sample_count() as usize];
    let mut source_id = vec![u32::MAX; topology.sample_count() as usize];
    let mut frontier = BinaryHeap::new();
    for (source, sample) in sources.iter().enumerate() {
        if let Some(mask) = allowed {
            if !mask[*sample as usize] {
                continue;
            }
        }
        if distance[*sample as usize] == 0.0 {
            continue;
        }
        distance[*sample as usize] = 0.0;
        source_id[*sample as usize] = source as u32;
        frontier.push(DistanceFrontier {
            distance_rad: 0.0,
            source: source as u32,
            sample: *sample,
        });
    }
    while let Some(current) = frontier.pop() {
        let index = current.sample as usize;
        if current.distance_rad > distance[index] + 1.0e-14 || current.source != source_id[index] {
            continue;
        }
        let neighbors = topology.neighbors(current.sample);
        let lengths = topology.neighbor_arc_lengths_rad(current.sample);
        for neighbor_index in 0..neighbors.len() {
            let neighbor = neighbors[neighbor_index];
            if let Some(mask) = allowed {
                if !mask[neighbor as usize] {
                    continue;
                }
            }
            let candidate = current.distance_rad + lengths[neighbor_index];
            let target = neighbor as usize;
            if candidate + 1.0e-14 < distance[target] {
                distance[target] = candidate;
                source_id[target] = current.source;
                frontier.push(DistanceFrontier {
                    distance_rad: candidate,
                    source: current.source,
                    sample: neighbor,
                });
            }
        }
    }
    (distance, source_id)
}

fn update_oceanic_ages<T: PlanetTopology>(
    topology: &T,
    parameters: PlanetPhysicalParameters,
    kinds: &[u8],
    boundaries: &[GeologicalBoundary],
    history_seed: u64,
    ages: &mut [f32],
    thicknesses: &mut [f32],
    densities: &mut [f32],
    buoyancies: &mut [f32],
) {
    let allowed = kinds
        .iter()
        .map(|kind| !matches!(crust_kind(*kind), CrustKind::Continental))
        .collect::<Vec<_>>();
    let mut sources = Vec::new();
    let mut source_rates = Vec::new();
    for boundary in boundaries {
        if !matches!(
            boundary.regime,
            GeologicalBoundaryRegime::OceanicRidge
                | GeologicalBoundaryRegime::TransitionalDivergence
        ) {
            continue;
        }
        let half_rate = (boundary.normal_rate_m_per_year.max(0.0) * 0.5).max(0.005);
        for sample in [boundary.sample_a, boundary.sample_b] {
            if allowed[sample as usize] {
                sources.push(sample);
                source_rates.push(half_rate);
            }
        }
    }
    let inherited = smooth_random_field(topology, history_seed ^ 0xdbe6_d5d5_fe4c_ce2f, 6);
    let (distance, source_ids) = multi_source_distance(topology, &sources, Some(&allowed));
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        if !matches!(crust_kind(kinds[index]), CrustKind::Oceanic) {
            continue;
        }
        let age = if distance[index].is_finite() && source_ids[index] != u32::MAX {
            let rate_m_per_myr = source_rates[source_ids[index] as usize] * 1_000_000.0;
            (distance[index] * parameters.radius_m / rate_m_per_myr).clamp(0.0, 220.0)
        } else {
            (130.0 + inherited[index] * 70.0).clamp(25.0, 220.0)
        };
        let normalized = (age / 220.0).sqrt();
        let thickness = 6.15 + normalized * 1.55;
        let density = 2885.0 + normalized * 115.0;
        ages[index] = age as f32;
        thicknesses[index] = thickness as f32;
        densities[index] = density as f32;
        buoyancies[index] = buoyancy_index(thickness, density) as f32;
    }
}

fn influence_field<T: PlanetTopology>(topology: &T, sources: &[u32], scale_rad: f64) -> Vec<f32> {
    if sources.is_empty() {
        return vec![0.0; topology.sample_count() as usize];
    }
    let (distance, _) = multi_source_distance(topology, sources, None);
    distance
        .into_iter()
        .map(|value| {
            if value.is_finite() {
                (-value / scale_rad).exp().clamp(0.0, 1.0) as f32
            } else {
                0.0
            }
        })
        .collect()
}

fn history_fields<T: PlanetTopology>(
    topology: &T,
    parameters: PlanetPhysicalParameters,
    kinds: &[u8],
    ages: &[f32],
    boundaries: &[GeologicalBoundary],
    history_seed: u64,
) -> (
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
) {
    let mut orogen_sources = Vec::new();
    let mut rift_sources = Vec::new();
    let mut ridge_sources = Vec::new();
    let mut subduction_sources = Vec::new();
    let mut trench_sources = Vec::new();
    let mut arc_sources = Vec::new();
    let mut transform_sources = Vec::new();
    for boundary in boundaries {
        match boundary.regime {
            GeologicalBoundaryRegime::ContinentalCollision => {
                orogen_sources.extend_from_slice(&[boundary.sample_a, boundary.sample_b]);
            }
            GeologicalBoundaryRegime::OceanicSubduction
            | GeologicalBoundaryRegime::OceanContinentSubduction => {
                subduction_sources.extend_from_slice(&[boundary.sample_a, boundary.sample_b]);
                match boundary.subduction_polarity {
                    SubductionPolarity::PlateA => {
                        trench_sources.push(boundary.sample_a);
                        arc_sources.push(boundary.sample_b);
                        orogen_sources.push(boundary.sample_b);
                    }
                    SubductionPolarity::PlateB => {
                        trench_sources.push(boundary.sample_b);
                        arc_sources.push(boundary.sample_a);
                        orogen_sources.push(boundary.sample_a);
                    }
                    SubductionPolarity::None => {}
                }
            }
            GeologicalBoundaryRegime::ContinentalRift
            | GeologicalBoundaryRegime::TransitionalDivergence => {
                rift_sources.extend_from_slice(&[boundary.sample_a, boundary.sample_b]);
            }
            GeologicalBoundaryRegime::OceanicRidge => {
                ridge_sources.extend_from_slice(&[boundary.sample_a, boundary.sample_b]);
            }
            GeologicalBoundaryRegime::Transform => {
                transform_sources.extend_from_slice(&[boundary.sample_a, boundary.sample_b]);
            }
        }
    }
    let km_per_rad = parameters.radius_m / 1000.0;
    let orogen_scale = 700.0 / km_per_rad;
    let rift_scale = 430.0 / km_per_rad;
    let ridge_scale = 330.0 / km_per_rad;
    let subduction_scale = 520.0 / km_per_rad;
    let trench_scale = 260.0 / km_per_rad;
    let arc_scale = 460.0 / km_per_rad;
    let transform_scale = 300.0 / km_per_rad;
    let mut orogen = influence_field(topology, &orogen_sources, orogen_scale);
    let mut rift = influence_field(topology, &rift_sources, rift_scale);
    let ridge = influence_field(topology, &ridge_sources, ridge_scale);
    let subduction = influence_field(topology, &subduction_sources, subduction_scale);
    let trench = influence_field(topology, &trench_sources, trench_scale);
    let arc = influence_field(topology, &arc_sources, arc_scale);
    let transform = influence_field(topology, &transform_sources, transform_scale);
    let inherited_orogen = smooth_random_field(topology, history_seed ^ 0x6a09_e667_f3bc_c909, 10);
    let inherited_basin = smooth_random_field(topology, history_seed ^ 0xbb67_ae85_84ca_a73b, 11);
    for sample in 0..topology.sample_count() as usize {
        let continental_weight = match crust_kind(kinds[sample]) {
            CrustKind::Continental => 1.0,
            CrustKind::Transitional => 0.55,
            CrustKind::Oceanic => 0.0,
        };
        let inherited =
            clamp01((inherited_orogen[sample] + 0.45) / 1.45) * continental_weight * 0.34;
        orogen[sample] = f64::max(f64::from(orogen[sample]), inherited) as f32;
        let inherited_extension =
            clamp01((-inherited_orogen[sample] + 0.25) / 1.25) * continental_weight * 0.13;
        rift[sample] = f64::max(f64::from(rift[sample]), inherited_extension) as f32;
    }
    let mut subsidence = vec![0.0_f32; kinds.len()];
    let mut basin = vec![0.0_f32; kinds.len()];
    let mut strain = vec![0.0_f32; kinds.len()];
    for sample in 0..kinds.len() {
        let ocean_age = if matches!(crust_kind(kinds[sample]), CrustKind::Oceanic) {
            f64::from(ages[sample]) / 220.0
        } else {
            0.0
        };
        let subsidence_value = clamp01(
            f64::from(rift[sample]) * 0.56 + f64::from(trench[sample]) * 0.38 + ocean_age * 0.34,
        );
        subsidence[sample] = subsidence_value as f32;
        let stable = 1.0
            - f64::from(orogen[sample])
                .max(f64::from(rift[sample]))
                .max(f64::from(subduction[sample]));
        let inherited = clamp01((inherited_basin[sample] + 0.55) / 1.55);
        let crust_weight = match crust_kind(kinds[sample]) {
            CrustKind::Continental => 1.0,
            CrustKind::Transitional => 0.85,
            CrustKind::Oceanic => 0.20,
        };
        basin[sample] = clamp01(
            (subsidence_value * 0.52 + stable.max(0.0) * 0.22 + inherited * 0.32) * crust_weight,
        ) as f32;
        strain[sample] = f64::from(orogen[sample])
            .max(f64::from(rift[sample]))
            .max(f64::from(subduction[sample]) * 0.88)
            .max(f64::from(transform[sample]) * 0.72)
            .max(f64::from(ridge[sample]) * 0.58) as f32;
    }
    (
        orogen, rift, ridge, subduction, trench, arc, transform, subsidence, basin, strain,
    )
}

fn apply_history_to_crust(
    kinds: &[u8],
    orogen: &[f32],
    rift: &[f32],
    thicknesses: &mut [f32],
    densities: &mut [f32],
    buoyancies: &mut [f32],
) {
    for index in 0..kinds.len() {
        let original = f64::from(thicknesses[index]);
        let adjusted = match crust_kind(kinds[index]) {
            CrustKind::Continental => (original + f64::from(orogen[index]) * 8.5
                - f64::from(rift[index]) * 5.5)
                .clamp(25.0, 56.0),
            CrustKind::Transitional => (original + f64::from(orogen[index]) * 4.5
                - f64::from(rift[index]) * 6.0)
                .clamp(12.0, 34.0),
            CrustKind::Oceanic => original,
        };
        thicknesses[index] = adjusted as f32;
        let density = f64::from(densities[index]);
        buoyancies[index] = buoyancy_index(adjusted, density) as f32;
    }
}

fn boundary_interface_length<T: PlanetTopology>(topology: &T, sample_a: u32, sample_b: u32) -> f64 {
    let neighbors = topology.neighbors(sample_a);
    let interfaces = topology.neighbor_interface_arc_lengths_rad(sample_a);
    neighbors
        .iter()
        .position(|neighbor| *neighbor == sample_b)
        .map(|index| interfaces[index])
        .unwrap_or(0.0)
}

fn build_plate_summaries<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    kinds: &[u8],
    ages: &[f32],
    thicknesses: &[f32],
) -> Vec<PlateSummary> {
    let plate_count = tectonics.plates.len();
    let mean_area = 4.0 * PI / plate_count as f64;
    let mut continental = vec![0.0; plate_count];
    let mut transitional = vec![0.0; plate_count];
    let mut oceanic = vec![0.0; plate_count];
    let mut weighted_age = vec![0.0; plate_count];
    let mut weighted_thickness = vec![0.0; plate_count];
    let mut convergent = vec![0.0; plate_count];
    let mut divergent = vec![0.0; plate_count];
    let mut transform = vec![0.0; plate_count];
    for sample in 0..topology.sample_count() {
        let plate = tectonics.plate_ids[sample as usize] as usize;
        let area = topology.area_steradians(sample);
        match crust_kind(kinds[sample as usize]) {
            CrustKind::Continental => continental[plate] += area,
            CrustKind::Transitional => transitional[plate] += area,
            CrustKind::Oceanic => oceanic[plate] += area,
        }
        weighted_age[plate] += area * f64::from(ages[sample as usize]);
        weighted_thickness[plate] += area * f64::from(thicknesses[sample as usize]);
    }
    for boundary in &tectonics.boundaries {
        let length = boundary_interface_length(topology, boundary.sample_a, boundary.sample_b);
        for plate in [boundary.plate_a as usize, boundary.plate_b as usize] {
            match boundary.kind {
                PlateBoundaryKind::Convergent => convergent[plate] += length,
                PlateBoundaryKind::Divergent => divergent[plate] += length,
                PlateBoundaryKind::Transform => transform[plate] += length,
            }
        }
    }
    tectonics
        .plates
        .iter()
        .map(|plate| {
            let index = plate.id as usize;
            let area = plate.area_steradians;
            let boundary_total = convergent[index] + divergent[index] + transform[index];
            let scale_class = if area >= mean_area * 1.5 {
                PlateScaleClass::Major
            } else if area <= mean_area * 0.65 {
                PlateScaleClass::Minor
            } else {
                PlateScaleClass::Intermediate
            };
            PlateSummary {
                plate_id: plate.id,
                area_steradians: area,
                area_fraction: area / (4.0 * PI),
                scale_class,
                continental_fraction: continental[index] / area,
                transitional_fraction: transitional[index] / area,
                oceanic_fraction: oceanic[index] / area,
                mean_crust_age_myr: weighted_age[index] / area,
                mean_crust_thickness_km: weighted_thickness[index] / area,
                convergent_boundary_fraction: if boundary_total > 0.0 {
                    convergent[index] / boundary_total
                } else {
                    0.0
                },
                divergent_boundary_fraction: if boundary_total > 0.0 {
                    divergent[index] / boundary_total
                } else {
                    0.0
                },
                transform_boundary_fraction: if boundary_total > 0.0 {
                    transform[index] / boundary_total
                } else {
                    0.0
                },
            }
        })
        .collect()
}

fn fnv1a_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn geology_hash(stage_seed: u64, model: &CrustalModel) -> u64 {
    let mut hash = fnv1a_update(0xcbf29ce484222325, b"interlink-geology:v1\0");
    hash = fnv1a_update(hash, &stage_seed.to_le_bytes());
    hash = fnv1a_update(hash, &model.crust_kind);
    for value in &model.crust_province_id {
        hash = fnv1a_update(hash, &value.to_le_bytes());
    }
    for field in [
        &model.crust_age_myr,
        &model.crust_thickness_km,
        &model.crust_density_kg_per_m3,
        &model.buoyancy_index,
        &model.orogenic_history,
        &model.rift_history,
        &model.ridge_history,
        &model.subduction_history,
        &model.trench_history,
        &model.volcanic_arc_history,
        &model.transform_history,
        &model.subsidence_history,
        &model.basin_potential,
        &model.crustal_strain,
    ] {
        for value in field {
            hash = fnv1a_update(hash, &value.to_bits().to_le_bytes());
        }
    }
    for boundary in &model.boundaries {
        hash = fnv1a_update(hash, &boundary.sample_a.to_le_bytes());
        hash = fnv1a_update(hash, &boundary.sample_b.to_le_bytes());
        hash = fnv1a_update(
            hash,
            &[boundary.regime as u8, boundary.subduction_polarity as u8],
        );
    }
    for plate in &model.plate_summaries {
        hash = fnv1a_update(hash, &[plate.scale_class as u8]);
    }
    hash
}

fn compute_metrics<T: PlanetTopology>(
    topology: &T,
    kinds: &[u8],
    ages: &[f32],
    thicknesses: &[f32],
    boundaries: &[GeologicalBoundary],
    hash: u64,
) -> GeologyMetrics {
    let total_area = (0..topology.sample_count())
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>();
    let mut areas = [0.0_f64; 3];
    let mut age_sum = [0.0_f64; 3];
    let mut thickness_sum = [0.0_f64; 3];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let bucket = match crust_kind(kinds[index]) {
            CrustKind::Oceanic => 0,
            CrustKind::Transitional => 1,
            CrustKind::Continental => 2,
        };
        let area = topology.area_steradians(sample);
        areas[bucket] += area;
        age_sum[bucket] += area * f64::from(ages[index]);
        thickness_sum[bucket] += area * f64::from(thicknesses[index]);
    }
    let mut regime_counts = [0_u32; 7];
    for boundary in boundaries {
        regime_counts[boundary.regime as usize - 1] += 1;
    }
    GeologyMetrics {
        sample_count: topology.sample_count(),
        continental_area_fraction: areas[2] / total_area,
        transitional_area_fraction: areas[1] / total_area,
        oceanic_area_fraction: areas[0] / total_area,
        mean_continental_age_myr: if areas[2] > 0.0 {
            age_sum[2] / areas[2]
        } else {
            0.0
        },
        mean_oceanic_age_myr: if areas[0] > 0.0 {
            age_sum[0] / areas[0]
        } else {
            0.0
        },
        mean_continental_thickness_km: if areas[2] > 0.0 {
            thickness_sum[2] / areas[2]
        } else {
            0.0
        },
        mean_oceanic_thickness_km: if areas[0] > 0.0 {
            thickness_sum[0] / areas[0]
        } else {
            0.0
        },
        oceanic_subduction_edges: regime_counts[0],
        ocean_continent_subduction_edges: regime_counts[1],
        continental_collision_edges: regime_counts[2],
        oceanic_ridge_edges: regime_counts[3],
        continental_rift_edges: regime_counts[4],
        transitional_divergence_edges: regime_counts[5],
        transform_edges: regime_counts[6],
        geology_hash: hash,
    }
}

fn validate_model<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    model: &CrustalModel,
) -> Result<(), WorldgenError> {
    let count = topology.sample_count() as usize;
    let fields: [&[f32]; 14] = [
        &model.crust_age_myr,
        &model.crust_thickness_km,
        &model.crust_density_kg_per_m3,
        &model.buoyancy_index,
        &model.orogenic_history,
        &model.rift_history,
        &model.ridge_history,
        &model.subduction_history,
        &model.trench_history,
        &model.volcanic_arc_history,
        &model.transform_history,
        &model.subsidence_history,
        &model.basin_potential,
        &model.crustal_strain,
    ];
    if model.crust_kind.len() != count
        || model.crust_province_id.len() != count
        || fields.iter().any(|field| field.len() != count)
    {
        return Err(WorldgenError::InvalidGeology(
            "crust fields do not match topology sample count",
        ));
    }
    if model.boundaries.len() != tectonics.boundaries.len() {
        return Err(WorldgenError::InvalidGeology(
            "geological boundary interpretation lost tectonic edges",
        ));
    }
    if model.plate_summaries.len() != tectonics.plates.len() {
        return Err(WorldgenError::InvalidGeology(
            "plate summaries do not match tectonic plate count",
        ));
    }
    if fields
        .iter()
        .flat_map(|field| field.iter())
        .any(|value| !value.is_finite())
    {
        return Err(WorldgenError::InvalidGeology(
            "geological fields contain non-finite values",
        ));
    }
    if model
        .crust_age_myr
        .iter()
        .any(|value| *value < 0.0 || *value > 4000.0)
    {
        return Err(WorldgenError::InvalidGeology(
            "crust ages leave the supported physical range",
        ));
    }
    if model
        .crust_thickness_km
        .iter()
        .any(|value| *value < 4.0 || *value > 60.0)
    {
        return Err(WorldgenError::InvalidGeology(
            "crust thickness leaves the supported physical range",
        ));
    }
    if model
        .crust_density_kg_per_m3
        .iter()
        .any(|value| *value < 2500.0 || *value > 3200.0)
    {
        return Err(WorldgenError::InvalidGeology(
            "crust density leaves the supported physical range",
        ));
    }
    let fraction_sum = model.metrics.continental_area_fraction
        + model.metrics.transitional_area_fraction
        + model.metrics.oceanic_area_fraction;
    if (fraction_sum - 1.0).abs() > 1.0e-10 {
        return Err(WorldgenError::InvalidGeology(
            "crust area fractions do not close to the sphere",
        ));
    }
    Ok(())
}

pub fn generate_crust_and_history<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    request: &GeologyRequest,
    parameters: PlanetPhysicalParameters,
) -> Result<CrustalModel, WorldgenError> {
    parameters
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    if tectonics.plate_ids.len() != topology.sample_count() as usize {
        return Err(WorldgenError::InvalidGeology(
            "tectonic ownership does not match geology topology",
        ));
    }
    let stage_seed = random::derive_stage_seed(&request.seed, GEOLOGY_NAMESPACE);
    let province_seed = random::derive_stage_seed(&request.seed, CRUST_PROVINCES_NAMESPACE);
    let property_seed = random::derive_stage_seed(&request.seed, CRUST_PROPERTIES_NAMESPACE);
    let history_seed = random::derive_stage_seed(&request.seed, GEOLOGICAL_HISTORY_NAMESPACE);
    let (kinds, provinces, _cratons, craton_ages, _affinity) =
        build_crust_partition(topology, tectonics, province_seed);
    let (mut ages, mut thicknesses, mut densities, mut buoyancies) =
        initial_properties(topology, &kinds, &provinces, &craton_ages, property_seed);
    let initial_boundaries = build_geological_boundaries(&kinds, &buoyancies, tectonics);
    update_oceanic_ages(
        topology,
        parameters,
        &kinds,
        &initial_boundaries,
        history_seed,
        &mut ages,
        &mut thicknesses,
        &mut densities,
        &mut buoyancies,
    );
    let mut boundaries = build_geological_boundaries(&kinds, &buoyancies, tectonics);
    let (orogen, rift, ridge, subduction, trench, arc, transform, subsidence, basin, strain) =
        history_fields(
            topology,
            parameters,
            &kinds,
            &ages,
            &boundaries,
            history_seed,
        );
    apply_history_to_crust(
        &kinds,
        &orogen,
        &rift,
        &mut thicknesses,
        &mut densities,
        &mut buoyancies,
    );
    boundaries = build_geological_boundaries(&kinds, &buoyancies, tectonics);
    let plate_summaries = build_plate_summaries(topology, tectonics, &kinds, &ages, &thicknesses);
    let stage = StageIdentity {
        id: GEOLOGY_STAGE_ID,
        version: GEOLOGY_STAGE_VERSION,
        derived_seed: stage_seed,
    };
    let placeholder_metrics = GeologyMetrics {
        sample_count: topology.sample_count(),
        continental_area_fraction: 0.0,
        transitional_area_fraction: 0.0,
        oceanic_area_fraction: 0.0,
        mean_continental_age_myr: 0.0,
        mean_oceanic_age_myr: 0.0,
        mean_continental_thickness_km: 0.0,
        mean_oceanic_thickness_km: 0.0,
        oceanic_subduction_edges: 0,
        ocean_continent_subduction_edges: 0,
        continental_collision_edges: 0,
        oceanic_ridge_edges: 0,
        continental_rift_edges: 0,
        transitional_divergence_edges: 0,
        transform_edges: 0,
        geology_hash: 0,
    };
    let mut model = CrustalModel {
        stage,
        province_seed,
        property_seed,
        history_seed,
        crust_kind: kinds,
        crust_province_id: provinces,
        crust_age_myr: ages,
        crust_thickness_km: thicknesses,
        crust_density_kg_per_m3: densities,
        buoyancy_index: buoyancies,
        orogenic_history: orogen,
        rift_history: rift,
        ridge_history: ridge,
        subduction_history: subduction,
        trench_history: trench,
        volcanic_arc_history: arc,
        transform_history: transform,
        subsidence_history: subsidence,
        basin_potential: basin,
        crustal_strain: strain,
        boundaries,
        plate_summaries,
        metrics: placeholder_metrics,
    };
    let hash = geology_hash(stage_seed, &model);
    model.metrics = compute_metrics(
        topology,
        &model.crust_kind,
        &model.crust_age_myr,
        &model.crust_thickness_km,
        &model.boundaries,
        hash,
    );
    validate_model(topology, tectonics, &model)?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_icosphere, generate_tectonics, TectonicsRequest};

    fn world(seed: &str) -> (crate::GeodesicTopology, TectonicModel, CrustalModel) {
        let topology = build_icosphere(4).unwrap();
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let tectonics =
            generate_tectonics(&topology, &TectonicsRequest::new(seed, 14), parameters).unwrap();
        let geology = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new(seed),
            parameters,
        )
        .unwrap();
        (topology, tectonics, geology)
    }

    #[test]
    fn crust_generation_is_deterministic_complete_and_mixed_within_plates() {
        let (topology, tectonics, geology) = world("wg3-determinism");
        let repeat = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new("wg3-determinism"),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
        assert_eq!(geology.metrics.geology_hash, repeat.metrics.geology_hash);
        assert_eq!(geology.crust_kind.len(), topology.sample_count() as usize);
        assert!((geology.metrics.continental_area_fraction - 0.36).abs() < 0.12);
        assert!(geology.metrics.oceanic_area_fraction > 0.45);
        assert!(geology
            .plate_summaries
            .iter()
            .any(|plate| plate.continental_fraction > 0.05 && plate.oceanic_fraction > 0.05));
    }

    #[test]
    fn continental_and_oceanic_crust_have_physically_distinct_age_and_thickness_distributions() {
        let (_, _, geology) = world("wg3-crust-physics");
        assert!(
            geology.metrics.mean_continental_age_myr > geology.metrics.mean_oceanic_age_myr * 3.0
        );
        assert!(
            geology.metrics.mean_continental_thickness_km
                > geology.metrics.mean_oceanic_thickness_km * 3.0
        );
        assert!(geology.metrics.mean_oceanic_age_myr <= 220.0);
    }

    #[test]
    fn geological_boundaries_preserve_every_tectonic_edge_and_assign_causal_regimes() {
        let (_, tectonics, geology) = world("wg3-boundaries");
        assert_eq!(geology.boundaries.len(), tectonics.boundaries.len());
        for (tectonic, geological) in tectonics.boundaries.iter().zip(&geology.boundaries) {
            assert_eq!(
                (tectonic.sample_a, tectonic.sample_b),
                (geological.sample_a, geological.sample_b)
            );
            match tectonic.kind {
                PlateBoundaryKind::Transform => {
                    assert_eq!(geological.regime, GeologicalBoundaryRegime::Transform)
                }
                PlateBoundaryKind::Divergent => assert!(matches!(
                    geological.regime,
                    GeologicalBoundaryRegime::OceanicRidge
                        | GeologicalBoundaryRegime::ContinentalRift
                        | GeologicalBoundaryRegime::TransitionalDivergence
                )),
                PlateBoundaryKind::Convergent => assert!(matches!(
                    geological.regime,
                    GeologicalBoundaryRegime::OceanicSubduction
                        | GeologicalBoundaryRegime::OceanContinentSubduction
                        | GeologicalBoundaryRegime::ContinentalCollision
                )),
            }
        }
    }

    #[test]
    fn derived_plate_scale_classes_are_area_based_and_do_not_replace_physical_area() {
        let (_, tectonics, geology) = world("wg3-plate-classes");
        let mean = 4.0 * PI / tectonics.plates.len() as f64;
        for summary in &geology.plate_summaries {
            let expected = if summary.area_steradians >= mean * 1.5 {
                PlateScaleClass::Major
            } else if summary.area_steradians <= mean * 0.65 {
                PlateScaleClass::Minor
            } else {
                PlateScaleClass::Intermediate
            };
            assert_eq!(summary.scale_class, expected);
            assert!(
                (summary.continental_fraction
                    + summary.transitional_fraction
                    + summary.oceanic_fraction
                    - 1.0)
                    .abs()
                    < 1.0e-8
            );
        }
    }

    #[test]
    fn history_fields_are_bounded_and_crust_responds_to_orogeny_and_rifting() {
        let (_, _, geology) = world("wg3-history");
        for field in [
            &geology.orogenic_history,
            &geology.rift_history,
            &geology.ridge_history,
            &geology.subduction_history,
            &geology.trench_history,
            &geology.volcanic_arc_history,
            &geology.transform_history,
            &geology.subsidence_history,
            &geology.basin_potential,
            &geology.crustal_strain,
        ] {
            assert!(field
                .iter()
                .all(|value| value.is_finite() && *value >= 0.0 && *value <= 1.0));
        }
        let mut orogenic_continental = Vec::new();
        let mut quiet_continental = Vec::new();
        for index in 0..geology.crust_kind.len() {
            if crust_kind(geology.crust_kind[index]) != CrustKind::Continental {
                continue;
            }
            if geology.orogenic_history[index] > 0.65 {
                orogenic_continental.push(geology.crust_thickness_km[index]);
            }
            if geology.orogenic_history[index] < 0.18 && geology.rift_history[index] < 0.18 {
                quiet_continental.push(geology.crust_thickness_km[index]);
            }
        }
        if !orogenic_continental.is_empty() && !quiet_continental.is_empty() {
            let orogenic_mean = orogenic_continental
                .iter()
                .map(|value| f64::from(*value))
                .sum::<f64>()
                / orogenic_continental.len() as f64;
            let quiet_mean = quiet_continental
                .iter()
                .map(|value| f64::from(*value))
                .sum::<f64>()
                / quiet_continental.len() as f64;
            assert!(orogenic_mean > quiet_mean);
        }
    }
}
