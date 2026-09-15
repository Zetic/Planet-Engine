use crate::{
    random, CrustKind, CrustalModel, PlanetPhysicalParameters, PlanetTopology, PlateBoundaryKind,
    PreOrogenicLithosphereModel, StageIdentity, TectonicHistoryModel, TectonicModel, WorldgenError,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub const OROGEN_PROVINCE_STAGE_ID: &str = "geology:tectonic-orogen-provinces";
pub const OROGEN_PROVINCE_STAGE_VERSION: u32 = 1;
const OROGEN_PROVINCE_NAMESPACE: &str = "worldgen:geology:tectonic-orogen-provinces:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const NO_PLATE: u16 = u16::MAX;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrogenProvinceKind {
    ContinentalCollision = 1,
    CollisionalPlateau = 2,
    CordilleranArc = 3,
    IslandArc = 4,
    TerraneAccretion = 5,
    TranspressionalOrogen = 6,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrogenProvinceRequest {
    pub seed: String,
}
impl OrogenProvinceRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrogenProvince {
    pub id: u16,
    pub boundary_system_id: u32,
    pub boundary_indices: Vec<u32>,
    pub kind: OrogenProvinceKind,
    pub plate_a: u16,
    pub plate_b: u16,
    pub overriding_plate_id: u16,
    pub hinterland_plate_id: u16,
    pub foreland_or_subducting_plate_id: u16,
    pub along_strike_start: f32,
    pub along_strike_end: f32,
    pub length_km: f32,
    pub event_age_myr: f32,
    pub cumulative_convergence_km: f32,
    pub mean_obliquity_deg: f32,
    pub mean_curvature_deg: f32,
    pub maturity_index: f32,
    pub shortening_index: f32,
    pub mean_core_width_km: f32,
    pub minimum_core_width_km: f32,
    pub maximum_core_width_km: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrogenProvinceMetrics {
    pub sample_count: u32,
    pub province_count: u16,
    pub continental_collision_count: u16,
    pub collisional_plateau_count: u16,
    pub cordilleran_arc_count: u16,
    pub island_arc_count: u16,
    pub terrane_accretion_count: u16,
    pub transpressional_count: u16,
    pub orogenic_area_fraction: f64,
    pub mean_source_width_km: f64,
    pub minimum_source_width_km: f64,
    pub maximum_source_width_km: f64,
    pub maximum_shortening_index: f64,
    pub maximum_orogenic_intensity: f64,
    pub province_hash: u64,
}
impl OrogenProvinceMetrics {
    pub fn province_hash_hex(&self) -> String {
        format!("{:016x}", self.province_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrogenProvinceModel {
    pub stage: StageIdentity,
    pub province_ids: Vec<u16>,
    pub province_kind: Vec<u8>,
    pub orogenic_intensity: Vec<f32>,
    pub crustal_root_index: Vec<f32>,
    pub plateau_index: Vec<f32>,
    pub fold_thrust_index: Vec<f32>,
    pub foreland_basin_index: Vec<f32>,
    pub volcanic_arc_index: Vec<f32>,
    pub backarc_extension_index: Vec<f32>,
    pub suture_index: Vec<f32>,
    pub transpression_index: Vec<f32>,
    pub maturity_index: Vec<f32>,
    pub shortening_index: Vec<f32>,
    pub local_width_km: Vec<f32>,
    pub source_along_strike_fraction: Vec<f32>,
    pub provinces: Vec<OrogenProvince>,
    pub metrics: OrogenProvinceMetrics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BoundaryClass {
    Collision,
    Cordilleran,
    IslandArc,
}

#[derive(Clone, Copy, Debug)]
struct EdgeTraits {
    boundary_index: usize,
    system_id: u32,
    class: BoundaryClass,
    plate_a: u16,
    plate_b: u16,
    overriding_plate: u16,
    hinterland_plate: u16,
    foreland_or_subducting_plate: u16,
    along_fraction: f64,
    along_distance_km: f64,
    age_myr: f64,
    convergence_km: f64,
    obliquity_deg: f64,
    curvature_deg: f64,
    weakness: f64,
    fabric: f64,
    age_discontinuity: f64,
    province_boundary: f64,
    mean_te_km: f64,
    fragment_contact: f64,
    strength_a: f64,
    strength_b: f64,
}

#[derive(Clone, Copy, Debug)]
struct BoundarySource {
    boundary_index: usize,
    province_id: u16,
    kind: OrogenProvinceKind,
    plate_a: u16,
    plate_b: u16,
    overriding_plate: u16,
    hinterland_plate: u16,
    foreland_or_subducting_plate: u16,
    along_fraction: f64,
    maturity: f64,
    shortening: f64,
    obliquity: f64,
    curvature: f64,
    width_a_km: f64,
    width_b_km: f64,
    core_strength: f64,
}

#[derive(Clone, Copy, Debug)]
struct DistanceFrontier {
    distance_km: f64,
    source_index: usize,
    sample: u32,
}
impl PartialEq for DistanceFrontier {
    fn eq(&self, other: &Self) -> bool {
        self.distance_km.to_bits() == other.distance_km.to_bits()
            && self.source_index == other.source_index
            && self.sample == other.sample
    }
}
impl Eq for DistanceFrontier {}
impl Ord for DistanceFrontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance_km
            .total_cmp(&self.distance_km)
            .then_with(|| other.source_index.cmp(&self.source_index))
            .then_with(|| other.sample.cmp(&self.sample))
    }
}
impl PartialOrd for DistanceFrontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}
fn smoothstep(value: f64) -> f64 {
    let x = clamp01(value);
    x * x * (3.0 - 2.0 * x)
}
fn gaussian(value: f64, center: f64, sigma: f64) -> f64 {
    let z = (value - center) / sigma.max(1.0e-9);
    (-0.5 * z * z).exp()
}
fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
fn hash_f32(mut hash: u64, values: &[f32]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    for value in values {
        hash = fnv_update(hash, &value.to_bits().to_le_bytes());
    }
    hash
}
fn hash_u8(mut hash: u64, values: &[u8]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    fnv_update(hash, values)
}
fn hash_u16(mut hash: u64, values: &[u16]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    for value in values {
        hash = fnv_update(hash, &value.to_le_bytes());
    }
    hash
}

fn average_pair(values: &[f32], a: u32, b: u32) -> f64 {
    (f64::from(values[a as usize]) + f64::from(values[b as usize])) * 0.5
}

fn edge_class(
    geology: &CrustalModel,
    pre: &PreOrogenicLithosphereModel,
    boundary: &crate::PlateBoundaryEdge,
) -> (BoundaryClass, u16, u16, u16) {
    let kind_a = geology.crust_kind[boundary.sample_a as usize];
    let kind_b = geology.crust_kind[boundary.sample_b as usize];
    let oceanic_a = kind_a == CrustKind::Oceanic as u8;
    let oceanic_b = kind_b == CrustKind::Oceanic as u8;

    if oceanic_a && oceanic_b {
        let age_a = geology.crust_age_myr[boundary.sample_a as usize];
        let age_b = geology.crust_age_myr[boundary.sample_b as usize];
        let subducting = if age_a > age_b {
            boundary.plate_a
        } else if age_b > age_a {
            boundary.plate_b
        } else {
            boundary.plate_a.min(boundary.plate_b)
        };
        let overriding = if subducting == boundary.plate_a {
            boundary.plate_b
        } else {
            boundary.plate_a
        };
        return (BoundaryClass::IslandArc, overriding, NO_PLATE, subducting);
    }

    if oceanic_a != oceanic_b {
        let subducting = if oceanic_a { boundary.plate_a } else { boundary.plate_b };
        let overriding = if oceanic_a { boundary.plate_b } else { boundary.plate_a };
        return (BoundaryClass::Cordilleran, overriding, NO_PLATE, subducting);
    }

    let strength_a = f64::from(pre.intrinsic_strength_index[boundary.sample_a as usize]);
    let strength_b = f64::from(pre.intrinsic_strength_index[boundary.sample_b as usize]);
    let hinterland = if strength_a > strength_b {
        boundary.plate_a
    } else if strength_b > strength_a {
        boundary.plate_b
    } else {
        boundary.plate_a.min(boundary.plate_b)
    };
    let foreland = if hinterland == boundary.plate_a {
        boundary.plate_b
    } else {
        boundary.plate_a
    };
    (BoundaryClass::Collision, NO_PLATE, hinterland, foreland)
}

fn build_edge_traits(
    tectonics: &TectonicModel,
    history: &TectonicHistoryModel,
    geology: &CrustalModel,
    pre: &PreOrogenicLithosphereModel,
) -> Vec<Option<EdgeTraits>> {
    let mut traits = vec![None; tectonics.boundaries.len()];
    for (index, boundary) in tectonics.boundaries.iter().enumerate() {
        if boundary.kind != PlateBoundaryKind::Convergent {
            continue;
        }
        let state = &history.boundary_state[index];
        let (class, overriding, hinterland, foreland_or_subducting) =
            edge_class(geology, pre, boundary);
        let a = boundary.sample_a;
        let b = boundary.sample_b;
        traits[index] = Some(EdgeTraits {
            boundary_index: index,
            system_id: state.system_id,
            class,
            plate_a: boundary.plate_a,
            plate_b: boundary.plate_b,
            overriding_plate: overriding,
            hinterland_plate: hinterland,
            foreland_or_subducting_plate: foreland_or_subducting,
            along_fraction: f64::from(state.along_strike_fraction),
            along_distance_km: f64::from(state.along_strike_distance_km),
            age_myr: f64::from(state.event_age_myr),
            convergence_km: f64::from(state.cumulative_convergence_km),
            obliquity_deg: f64::from(state.obliquity_deg),
            curvature_deg: f64::from(state.curvature_deg),
            weakness: average_pair(&pre.intrinsic_weakness_index, a, b),
            fabric: average_pair(&pre.inherited_fabric_strength, a, b),
            age_discontinuity: average_pair(&pre.age_discontinuity_index, a, b),
            province_boundary: average_pair(&pre.province_boundary_index, a, b),
            mean_te_km: average_pair(&pre.effective_elastic_thickness_km, a, b),
            fragment_contact: if pre.kinematic_domain_ids[a as usize]
                != pre.kinematic_domain_ids[b as usize]
                || pre.fragment_ids[a as usize] > 0
                || pre.fragment_ids[b as usize] > 0
            {
                1.0
            } else {
                0.0
            },
            strength_a: f64::from(pre.intrinsic_strength_index[a as usize]),
            strength_b: f64::from(pre.intrinsic_strength_index[b as usize]),
        });
    }
    traits
}

fn discontinuity(a: EdgeTraits, b: EdgeTraits) -> f64 {
    if a.class != b.class {
        return 10.0;
    }
    (a.weakness - b.weakness).abs() * 1.4
        + (a.fabric - b.fabric).abs() * 0.9
        + ((a.obliquity_deg - b.obliquity_deg).abs() / 90.0) * 0.8
        + ((a.curvature_deg - b.curvature_deg).abs() / 90.0) * 0.45
        + (a.fragment_contact - b.fragment_contact).abs() * 0.55
        + ((a.mean_te_km - b.mean_te_km).abs() / 80.0) * 0.45
}

fn segmentation_length_km(class: BoundaryClass) -> f64 {
    match class {
        BoundaryClass::Collision => 2400.0,
        BoundaryClass::Cordilleran => 1800.0,
        BoundaryClass::IslandArc => 1450.0,
    }
}

fn segment_system(edges: &[usize], edge_traits: &[Option<EdgeTraits>]) -> Vec<Vec<usize>> {
    if edges.is_empty() {
        return Vec::new();
    }
    let mut ordered = edges.to_vec();
    ordered.sort_by(|left, right| {
        edge_traits[*left]
            .unwrap()
            .along_distance_km
            .total_cmp(&edge_traits[*right].unwrap().along_distance_km)
            .then_with(|| left.cmp(right))
    });

    let mut segments = Vec::new();
    let mut current = vec![ordered[0]];
    let mut start_distance = edge_traits[ordered[0]].unwrap().along_distance_km;
    let mut previous = edge_traits[ordered[0]].unwrap();

    for edge in ordered.into_iter().skip(1) {
        let traits = edge_traits[edge].unwrap();
        let span = (traits.along_distance_km - start_distance).abs();
        let target = segmentation_length_km(previous.class);
        let cut = traits.class != previous.class
            || (span > 650.0 && discontinuity(previous, traits) > 0.70)
            || span > target;
        if cut {
            segments.push(current);
            current = Vec::new();
            start_distance = traits.along_distance_km;
        }
        current.push(edge);
        previous = traits;
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

fn province_kind(segment: &[usize], edge_traits: &[Option<EdgeTraits>]) -> OrogenProvinceKind {
    let count = segment.len().max(1) as f64;
    let first = edge_traits[segment[0]].unwrap();
    let mean_obliquity = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().obliquity_deg)
        .sum::<f64>()
        / count;
    let mean_fragment = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().fragment_contact)
        .sum::<f64>()
        / count;
    let mean_age = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().age_myr)
        .sum::<f64>()
        / count;
    let mean_convergence = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().convergence_km)
        .sum::<f64>()
        / count;
    let maturity = 0.42 * clamp01(mean_age / 95.0) + 0.58 * clamp01(mean_convergence / 3200.0);
    let shortening = clamp01(mean_convergence / 3000.0);

    if mean_obliquity >= 62.0 {
        return OrogenProvinceKind::TranspressionalOrogen;
    }
    match first.class {
        BoundaryClass::Cordilleran => OrogenProvinceKind::CordilleranArc,
        BoundaryClass::IslandArc => OrogenProvinceKind::IslandArc,
        BoundaryClass::Collision => {
            if mean_fragment >= 0.34 {
                OrogenProvinceKind::TerraneAccretion
            } else if maturity >= 0.56 && shortening >= 0.48 {
                OrogenProvinceKind::CollisionalPlateau
            } else {
                OrogenProvinceKind::ContinentalCollision
            }
        }
    }
}

fn base_width_km(kind: OrogenProvinceKind, maturity: f64, weakness: f64) -> f64 {
    match kind {
        OrogenProvinceKind::ContinentalCollision => 360.0 + maturity * 620.0 + weakness * 430.0,
        OrogenProvinceKind::CollisionalPlateau => 760.0 + maturity * 820.0 + weakness * 520.0,
        OrogenProvinceKind::CordilleranArc => 250.0 + maturity * 390.0 + weakness * 260.0,
        OrogenProvinceKind::IslandArc => 180.0 + maturity * 300.0 + weakness * 180.0,
        OrogenProvinceKind::TerraneAccretion => 300.0 + maturity * 430.0 + weakness * 330.0,
        OrogenProvinceKind::TranspressionalOrogen => 170.0 + maturity * 320.0 + weakness * 180.0,
    }
}

fn source_widths(
    traits: EdgeTraits,
    kind: OrogenProvinceKind,
    system_has_endpoints: bool,
) -> (f64, f64, f64, f64, f64) {
    let maturity = 0.42 * clamp01(traits.age_myr / 95.0)
        + 0.58 * clamp01(traits.convergence_km / 3200.0);
    let shortening = clamp01(traits.convergence_km / 3000.0);
    let curvature = clamp01(traits.curvature_deg / 90.0);
    let local_control = (0.42
        + traits.weakness * 1.20
        + traits.fabric * 0.48
        + traits.age_discontinuity * 0.52
        + traits.province_boundary * 0.36
        + curvature * 0.28
        + traits.fragment_contact * 0.24)
        .clamp(0.38, 2.75);
    let mut base = base_width_km(kind, maturity, traits.weakness) * local_control;

    let taper = if system_has_endpoints {
        smoothstep(traits.along_fraction / 0.15)
            * smoothstep((1.0 - traits.along_fraction) / 0.15)
    } else {
        1.0
    };
    base *= 0.14 + taper * 0.86;

    let (minimum, maximum) = match kind {
        OrogenProvinceKind::CollisionalPlateau => (260.0, 2200.0),
        OrogenProvinceKind::ContinentalCollision => (150.0, 1750.0),
        OrogenProvinceKind::TerraneAccretion => (130.0, 1250.0),
        OrogenProvinceKind::CordilleranArc => (110.0, 1050.0),
        OrogenProvinceKind::IslandArc => (90.0, 780.0),
        OrogenProvinceKind::TranspressionalOrogen => (80.0, 700.0),
    };
    base = base.clamp(minimum, maximum);

    let asymmetry = ((traits.strength_b - traits.strength_a) * 0.85
        + (traits.fabric - 0.5) * 0.12)
        .clamp(-0.45, 0.45);
    let mut width_a = (base * (1.0 + asymmetry)).clamp(minimum * 0.70, maximum);
    let mut width_b = (base * (1.0 - asymmetry)).clamp(minimum * 0.70, maximum);

    if matches!(kind, OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc) {
        if traits.overriding_plate == traits.plate_a {
            width_a = (base * 1.15).clamp(minimum, maximum);
            width_b = (base * 0.28).clamp(55.0, maximum);
        } else {
            width_b = (base * 1.15).clamp(minimum, maximum);
            width_a = (base * 0.28).clamp(55.0, maximum);
        }
    }

    let orthogonality = 1.0 - clamp01(traits.obliquity_deg / 90.0);
    let core_strength = clamp01(
        (0.18 + maturity * 0.66 + shortening * 0.34)
            * (0.72 + orthogonality * 0.28)
            * (0.28 + taper * 0.72),
    );
    (width_a, width_b, maturity, shortening, core_strength)
}

fn build_provinces_and_sources(
    tectonics: &TectonicModel,
    history: &TectonicHistoryModel,
    geology: &CrustalModel,
    pre: &PreOrogenicLithosphereModel,
) -> (Vec<OrogenProvince>, Vec<BoundarySource>) {
    let edge_traits = build_edge_traits(tectonics, history, geology, pre);
    let mut provinces = Vec::new();
    let mut sources = Vec::new();

    for system in &history.systems {
        if system.kind != PlateBoundaryKind::Convergent {
            continue;
        }
        let eligible = system
            .boundary_indices
            .iter()
            .copied()
            .map(|value| value as usize)
            .filter(|edge| edge_traits[*edge].is_some())
            .collect::<Vec<_>>();
        for segment in segment_system(&eligible, &edge_traits) {
            if segment.is_empty() {
                continue;
            }
            let id = (provinces.len() + 1) as u16;
            let kind = province_kind(&segment, &edge_traits);
            let system_has_endpoints = system.endpoint_count > 0;
            let mut widths = Vec::with_capacity(segment.len() * 2);
            let mut age_sum = 0.0;
            let mut convergence_sum = 0.0;
            let mut obliquity_sum = 0.0;
            let mut curvature_sum = 0.0;
            let mut maturity_sum = 0.0;
            let mut shortening_sum = 0.0;
            let mut start_fraction = 1.0_f64;
            let mut end_fraction = 0.0_f64;
            let mut start_distance = f64::INFINITY;
            let mut end_distance = f64::NEG_INFINITY;
            let first_traits = edge_traits[segment[0]].unwrap();

            for edge in &segment {
                let traits = edge_traits[*edge].unwrap();
                let (width_a, width_b, maturity, shortening, core_strength) =
                    source_widths(traits, kind, system_has_endpoints);
                widths.push(width_a);
                widths.push(width_b);
                age_sum += traits.age_myr;
                convergence_sum += traits.convergence_km;
                obliquity_sum += traits.obliquity_deg;
                curvature_sum += traits.curvature_deg;
                maturity_sum += maturity;
                shortening_sum += shortening;
                start_fraction = start_fraction.min(traits.along_fraction);
                end_fraction = end_fraction.max(traits.along_fraction);
                start_distance = start_distance.min(traits.along_distance_km);
                end_distance = end_distance.max(traits.along_distance_km);
                sources.push(BoundarySource {
                    boundary_index: traits.boundary_index,
                    province_id: id,
                    kind,
                    plate_a: traits.plate_a,
                    plate_b: traits.plate_b,
                    overriding_plate: traits.overriding_plate,
                    hinterland_plate: traits.hinterland_plate,
                    foreland_or_subducting_plate: traits.foreland_or_subducting_plate,
                    along_fraction: traits.along_fraction,
                    maturity,
                    shortening,
                    obliquity: clamp01(traits.obliquity_deg / 90.0),
                    curvature: clamp01(traits.curvature_deg / 90.0),
                    width_a_km: width_a,
                    width_b_km: width_b,
                    core_strength,
                });
            }

            let count = segment.len() as f64;
            let mean_width = widths.iter().sum::<f64>() / widths.len().max(1) as f64;
            let min_width = widths.iter().copied().fold(f64::INFINITY, f64::min);
            let max_width = widths.iter().copied().fold(0.0_f64, f64::max);
            provinces.push(OrogenProvince {
                id,
                boundary_system_id: first_traits.system_id,
                boundary_indices: segment.iter().map(|edge| *edge as u32).collect(),
                kind,
                plate_a: first_traits.plate_a,
                plate_b: first_traits.plate_b,
                overriding_plate_id: first_traits.overriding_plate,
                hinterland_plate_id: first_traits.hinterland_plate,
                foreland_or_subducting_plate_id: first_traits.foreland_or_subducting_plate,
                along_strike_start: start_fraction as f32,
                along_strike_end: end_fraction as f32,
                length_km: (end_distance - start_distance).max(0.0) as f32,
                event_age_myr: (age_sum / count) as f32,
                cumulative_convergence_km: (convergence_sum / count) as f32,
                mean_obliquity_deg: (obliquity_sum / count) as f32,
                mean_curvature_deg: (curvature_sum / count) as f32,
                maturity_index: (maturity_sum / count) as f32,
                shortening_index: (shortening_sum / count) as f32,
                mean_core_width_km: mean_width as f32,
                minimum_core_width_km: min_width as f32,
                maximum_core_width_km: max_width as f32,
            });
        }
    }
    (provinces, sources)
}

fn nearest_sources<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    sources: &[BoundarySource],
    parameters: PlanetPhysicalParameters,
) -> (Vec<f64>, Vec<usize>) {
    let count = topology.sample_count() as usize;
    let mut distance = vec![f64::INFINITY; count];
    let mut source_id = vec![usize::MAX; count];
    let mut frontier = BinaryHeap::new();

    for (source_index, source) in sources.iter().enumerate() {
        let boundary = &tectonics.boundaries[source.boundary_index];
        for sample in [boundary.sample_a, boundary.sample_b] {
            let index = sample as usize;
            let sample_plate = tectonics.plate_ids[index];
            if sample_plate != source.plate_a && sample_plate != source.plate_b {
                continue;
            }
            let replace = distance[index] > 0.0
                || (distance[index] == 0.0 && source_index < source_id[index]);
            if replace {
                distance[index] = 0.0;
                source_id[index] = source_index;
                frontier.push(DistanceFrontier {
                    distance_km: 0.0,
                    source_index,
                    sample,
                });
            }
        }
    }

    let radius_km = parameters.radius_m / 1000.0;
    while let Some(current) = frontier.pop() {
        let index = current.sample as usize;
        if current.distance_km > distance[index] + 1.0e-9
            || current.source_index != source_id[index]
        {
            continue;
        }
        let plate = tectonics.plate_ids[index];
        let neighbors = topology.neighbors(current.sample);
        let lengths = topology.neighbor_arc_lengths_rad(current.sample);
        for neighbor_index in 0..neighbors.len() {
            let neighbor = neighbors[neighbor_index];
            let target = neighbor as usize;
            if tectonics.plate_ids[target] != plate {
                continue;
            }
            let candidate = current.distance_km + lengths[neighbor_index] * radius_km;
            if candidate + 1.0e-9 < distance[target]
                || ((candidate - distance[target]).abs() <= 1.0e-9
                    && current.source_index < source_id[target])
            {
                distance[target] = candidate;
                source_id[target] = current.source_index;
                frontier.push(DistanceFrontier {
                    distance_km: candidate,
                    source_index: current.source_index,
                    sample: neighbor,
                });
            }
        }
    }
    (distance, source_id)
}

fn rasterize<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    sources: &[BoundarySource],
    parameters: PlanetPhysicalParameters,
) -> (
    Vec<u16>,
    Vec<u8>,
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
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
) {
    let count = topology.sample_count() as usize;
    let (distance, source_id) = nearest_sources(topology, tectonics, sources, parameters);
    let mut province_ids = vec![0_u16; count];
    let mut province_kind = vec![0_u8; count];
    let mut orogenic = vec![0.0_f32; count];
    let mut root = vec![0.0_f32; count];
    let mut plateau = vec![0.0_f32; count];
    let mut fold_thrust = vec![0.0_f32; count];
    let mut foreland = vec![0.0_f32; count];
    let mut arc = vec![0.0_f32; count];
    let mut backarc = vec![0.0_f32; count];
    let mut suture = vec![0.0_f32; count];
    let mut transpression = vec![0.0_f32; count];
    let mut maturity = vec![0.0_f32; count];
    let mut shortening = vec![0.0_f32; count];
    let mut local_width = vec![0.0_f32; count];
    let mut along_fraction = vec![0.0_f32; count];

    for sample in 0..count {
        let source_index = source_id[sample];
        if source_index == usize::MAX || !distance[sample].is_finite() {
            continue;
        }
        let source = sources[source_index];
        let plate = tectonics.plate_ids[sample];
        let width = if plate == source.plate_a {
            source.width_a_km
        } else if plate == source.plate_b {
            source.width_b_km
        } else {
            continue;
        };
        let x = distance[sample] / width.max(1.0);
        if x > 3.2 {
            continue;
        }
        let radial = (-x.powf(1.35)).exp();
        let overriding = plate == source.overriding_plate;
        let hinterland = plate == source.hinterland_plate;
        let foreland_side = plate == source.foreland_or_subducting_plate;
        let subduction_kind = matches!(
            source.kind,
            OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc
        );
        let side_amplitude = if subduction_kind {
            if overriding { 1.0 } else { 0.24 }
        } else if hinterland {
            1.0
        } else if foreland_side {
            0.88
        } else {
            0.72
        };
        let intensity = clamp01(source.core_strength * radial * side_amplitude);
        if intensity < 0.018 {
            continue;
        }

        let collision_kind = matches!(
            source.kind,
            OrogenProvinceKind::ContinentalCollision
                | OrogenProvinceKind::CollisionalPlateau
                | OrogenProvinceKind::TerraneAccretion
                | OrogenProvinceKind::TranspressionalOrogen
        );
        let root_value = if collision_kind {
            clamp01(source.core_strength * gaussian(x, 0.16, 0.44) * if hinterland { 1.0 } else { 0.78 })
        } else if overriding {
            clamp01(source.core_strength * gaussian(x, 0.34, 0.42) * 0.58)
        } else {
            clamp01(source.core_strength * gaussian(x, 0.05, 0.25) * 0.18)
        };
        let plateau_value = if source.kind == OrogenProvinceKind::CollisionalPlateau {
            clamp01(source.maturity * source.shortening * gaussian(x, 0.46, 0.52) * if hinterland { 1.0 } else { 0.62 })
        } else {
            0.0
        };
        let fold_value = if collision_kind {
            clamp01(source.core_strength * gaussian(x, 0.88, 0.36) * if foreland_side { 1.0 } else { 0.70 })
        } else if overriding {
            clamp01(source.core_strength * gaussian(x, 0.62, 0.30) * 0.52)
        } else {
            0.0
        };
        let foreland_value = if collision_kind && foreland_side {
            clamp01(source.maturity * gaussian(x, 1.48, 0.34))
        } else {
            0.0
        };
        let arc_value = if subduction_kind && overriding {
            clamp01(source.maturity * gaussian(x, 0.48, 0.18))
        } else {
            0.0
        };
        let backarc_value = if subduction_kind && overriding {
            clamp01(source.maturity * gaussian(x, 1.02, 0.28) * 0.82)
        } else {
            0.0
        };
        let suture_value = if collision_kind {
            clamp01(source.core_strength * gaussian(x, 0.0, 0.16))
        } else {
            clamp01(source.core_strength * gaussian(x, 0.0, 0.12) * 0.32)
        };
        let transpression_value = clamp01(
            source.core_strength
                * source.obliquity
                * (0.72 + source.curvature * 0.28)
                * gaussian(x, 0.16, 0.32),
        );

        province_ids[sample] = source.province_id;
        province_kind[sample] = source.kind as u8;
        orogenic[sample] = intensity as f32;
        root[sample] = root_value as f32;
        plateau[sample] = plateau_value as f32;
        fold_thrust[sample] = fold_value as f32;
        foreland[sample] = foreland_value as f32;
        arc[sample] = arc_value as f32;
        backarc[sample] = backarc_value as f32;
        suture[sample] = suture_value as f32;
        transpression[sample] = transpression_value as f32;
        maturity[sample] = source.maturity as f32;
        shortening[sample] = source.shortening as f32;
        local_width[sample] = width as f32;
        along_fraction[sample] = source.along_fraction as f32;
    }

    (
        province_ids,
        province_kind,
        orogenic,
        root,
        plateau,
        fold_thrust,
        foreland,
        arc,
        backarc,
        suture,
        transpression,
        maturity,
        shortening,
        local_width,
        along_fraction,
    )
}

fn model_hash(model: &OrogenProvinceModel) -> u64 {
    let mut hash = fnv_update(FNV_OFFSET_BASIS, b"interlink-orogen-provinces:v1\0");
    hash = fnv_update(hash, &model.stage.derived_seed.to_le_bytes());
    hash = hash_u16(hash, &model.province_ids);
    hash = hash_u8(hash, &model.province_kind);
    hash = hash_f32(hash, &model.orogenic_intensity);
    hash = hash_f32(hash, &model.crustal_root_index);
    hash = hash_f32(hash, &model.plateau_index);
    hash = hash_f32(hash, &model.fold_thrust_index);
    hash = hash_f32(hash, &model.foreland_basin_index);
    hash = hash_f32(hash, &model.volcanic_arc_index);
    hash = hash_f32(hash, &model.backarc_extension_index);
    hash = hash_f32(hash, &model.suture_index);
    hash = hash_f32(hash, &model.transpression_index);
    hash = hash_f32(hash, &model.maturity_index);
    hash = hash_f32(hash, &model.shortening_index);
    hash = hash_f32(hash, &model.local_width_km);
    hash = hash_f32(hash, &model.source_along_strike_fraction);
    for province in &model.provinces {
        hash = fnv_update(hash, &province.id.to_le_bytes());
        hash = fnv_update(hash, &province.boundary_system_id.to_le_bytes());
        hash = fnv_update(hash, &[province.kind as u8]);
        hash = fnv_update(hash, &province.plate_a.to_le_bytes());
        hash = fnv_update(hash, &province.plate_b.to_le_bytes());
        hash = fnv_update(hash, &province.maturity_index.to_bits().to_le_bytes());
        hash = fnv_update(hash, &province.shortening_index.to_bits().to_le_bytes());
        hash = fnv_update(hash, &province.mean_core_width_km.to_bits().to_le_bytes());
        for edge in &province.boundary_indices {
            hash = fnv_update(hash, &edge.to_le_bytes());
        }
    }
    hash
}

fn weighted_area_fraction<T: PlanetTopology>(topology: &T, province_ids: &[u16]) -> f64 {
    let mut total = 0.0;
    let mut active = 0.0;
    for sample in 0..topology.sample_count() {
        let area = topology.area_steradians(sample);
        total += area;
        if province_ids[sample as usize] > 0 {
            active += area;
        }
    }
    active / total.max(1.0e-12)
}

fn build_metrics<T: PlanetTopology>(
    topology: &T,
    sources: &[BoundarySource],
    model: &OrogenProvinceModel,
    hash: u64,
) -> OrogenProvinceMetrics {
    let mut counts = [0_u16; 6];
    for province in &model.provinces {
        counts[province.kind as usize - 1] += 1;
    }
    let mut widths = Vec::with_capacity(sources.len() * 2);
    for source in sources {
        widths.push(source.width_a_km);
        widths.push(source.width_b_km);
    }
    let mean_width = if widths.is_empty() {
        0.0
    } else {
        widths.iter().sum::<f64>() / widths.len() as f64
    };
    let min_width = widths.iter().copied().fold(f64::INFINITY, f64::min);
    let max_width = widths.iter().copied().fold(0.0_f64, f64::max);
    OrogenProvinceMetrics {
        sample_count: topology.sample_count(),
        province_count: model.provinces.len() as u16,
        continental_collision_count: counts[0],
        collisional_plateau_count: counts[1],
        cordilleran_arc_count: counts[2],
        island_arc_count: counts[3],
        terrane_accretion_count: counts[4],
        transpressional_count: counts[5],
        orogenic_area_fraction: weighted_area_fraction(topology, &model.province_ids),
        mean_source_width_km: mean_width,
        minimum_source_width_km: if min_width.is_finite() { min_width } else { 0.0 },
        maximum_source_width_km: max_width,
        maximum_shortening_index: model
            .shortening_index
            .iter()
            .copied()
            .map(f64::from)
            .fold(0.0, f64::max),
        maximum_orogenic_intensity: model
            .orogenic_intensity
            .iter()
            .copied()
            .map(f64::from)
            .fold(0.0, f64::max),
        province_hash: hash,
    }
}

fn validate_model<T: PlanetTopology>(
    topology: &T,
    model: &OrogenProvinceModel,
) -> Result<(), WorldgenError> {
    let count = topology.sample_count() as usize;
    let lengths = [
        model.province_ids.len(),
        model.province_kind.len(),
        model.orogenic_intensity.len(),
        model.crustal_root_index.len(),
        model.plateau_index.len(),
        model.fold_thrust_index.len(),
        model.foreland_basin_index.len(),
        model.volcanic_arc_index.len(),
        model.backarc_extension_index.len(),
        model.suture_index.len(),
        model.transpression_index.len(),
        model.maturity_index.len(),
        model.shortening_index.len(),
        model.local_width_km.len(),
        model.source_along_strike_fraction.len(),
    ];
    if lengths.iter().any(|length| *length != count) {
        return Err(WorldgenError::InvalidLithosphere(
            "orogen province fields do not match topology sample count",
        ));
    }
    let normalized: [&[f32]; 10] = [
        &model.orogenic_intensity,
        &model.crustal_root_index,
        &model.plateau_index,
        &model.fold_thrust_index,
        &model.foreland_basin_index,
        &model.volcanic_arc_index,
        &model.backarc_extension_index,
        &model.suture_index,
        &model.transpression_index,
        &model.maturity_index,
    ];
    if normalized.iter().flat_map(|field| field.iter()).any(|value| {
        !value.is_finite() || *value < 0.0 || *value > 1.0
    }) || model
        .shortening_index
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0 || *value > 1.0)
    {
        return Err(WorldgenError::InvalidLithosphere(
            "orogen province normalized field is outside [0,1]",
        ));
    }
    if model.local_width_km.iter().any(|value| {
        !value.is_finite() || *value < 0.0 || *value > 2200.001
    }) {
        return Err(WorldgenError::InvalidLithosphere(
            "orogen province width is outside supported bounds",
        ));
    }
    for sample in 0..count {
        let id = model.province_ids[sample];
        if id > model.provinces.len() as u16 {
            return Err(WorldgenError::InvalidLithosphere(
                "orogen field references a missing province",
            ));
        }
        if id == 0 && model.province_kind[sample] != 0 {
            return Err(WorldgenError::InvalidLithosphere(
                "orogen kind is populated outside a province",
            ));
        }
    }
    for (index, province) in model.provinces.iter().enumerate() {
        if province.id as usize != index + 1 || province.boundary_indices.is_empty() {
            return Err(WorldgenError::InvalidLithosphere(
                "orogen province identity or membership is invalid",
            ));
        }
    }
    Ok(())
}

pub fn generate_tectonic_orogen_provinces<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    history: &TectonicHistoryModel,
    geology: &CrustalModel,
    pre: &PreOrogenicLithosphereModel,
    request: &OrogenProvinceRequest,
    parameters: PlanetPhysicalParameters,
) -> Result<OrogenProvinceModel, WorldgenError> {
    parameters
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    let count = topology.sample_count() as usize;
    if tectonics.plate_ids.len() != count
        || history.boundary_state.len() != tectonics.boundaries.len()
        || geology.crust_kind.len() != count
        || pre.intrinsic_strength_index.len() != count
    {
        return Err(WorldgenError::InvalidLithosphere(
            "orogen province upstream state does not match topology",
        ));
    }

    let stage_seed = random::derive_stage_seed(&request.seed, OROGEN_PROVINCE_NAMESPACE);
    let (provinces, sources) = build_provinces_and_sources(tectonics, history, geology, pre);
    let (
        province_ids,
        province_kind,
        orogenic_intensity,
        crustal_root_index,
        plateau_index,
        fold_thrust_index,
        foreland_basin_index,
        volcanic_arc_index,
        backarc_extension_index,
        suture_index,
        transpression_index,
        maturity_index,
        shortening_index,
        local_width_km,
        source_along_strike_fraction,
    ) = rasterize(topology, tectonics, &sources, parameters);

    let placeholder = OrogenProvinceMetrics {
        sample_count: topology.sample_count(),
        province_count: 0,
        continental_collision_count: 0,
        collisional_plateau_count: 0,
        cordilleran_arc_count: 0,
        island_arc_count: 0,
        terrane_accretion_count: 0,
        transpressional_count: 0,
        orogenic_area_fraction: 0.0,
        mean_source_width_km: 0.0,
        minimum_source_width_km: 0.0,
        maximum_source_width_km: 0.0,
        maximum_shortening_index: 0.0,
        maximum_orogenic_intensity: 0.0,
        province_hash: 0,
    };
    let mut model = OrogenProvinceModel {
        stage: StageIdentity {
            id: OROGEN_PROVINCE_STAGE_ID,
            version: OROGEN_PROVINCE_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        province_ids,
        province_kind,
        orogenic_intensity,
        crustal_root_index,
        plateau_index,
        fold_thrust_index,
        foreland_basin_index,
        volcanic_arc_index,
        backarc_extension_index,
        suture_index,
        transpression_index,
        maturity_index,
        shortening_index,
        local_width_km,
        source_along_strike_fraction,
        provinces,
        metrics: placeholder,
    };
    let hash = model_hash(&model);
    model.metrics = build_metrics(topology, &sources, &model, hash);
    validate_model(topology, &model)?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_icosphere, generate_crust_and_history, generate_pre_orogenic_lithosphere,
        generate_tectonic_history, generate_tectonics, GeologyRequest, PreOrogenicLithosphereRequest,
        TectonicHistoryRequest, TectonicsRequest,
    };

    fn generate(seed: &str) -> OrogenProvinceModel {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let tectonics = generate_tectonics(&topology, &TectonicsRequest::new(seed, 16), planet).unwrap();
        let history = generate_tectonic_history(
            &topology,
            &tectonics,
            &TectonicHistoryRequest::new(seed),
            planet,
        )
        .unwrap();
        let geology = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new(seed),
            planet,
        )
        .unwrap();
        let pre = generate_pre_orogenic_lithosphere(
            &topology,
            &tectonics,
            &history,
            &geology,
            &PreOrogenicLithosphereRequest::new(seed),
        )
        .unwrap();
        generate_tectonic_orogen_provinces(
            &topology,
            &tectonics,
            &history,
            &geology,
            &pre,
            &OrogenProvinceRequest::new(seed),
            planet,
        )
        .unwrap()
    }

    #[test]
    fn tectonic_orogen_provinces_are_deterministic_complete_and_finite() {
        let first = generate("wg36-orogen-determinism");
        let second = generate("wg36-orogen-determinism");
        assert_eq!(first.metrics.province_hash, second.metrics.province_hash);
        assert_eq!(first.province_ids, second.province_ids);
        assert!(!first.provinces.is_empty());
        assert!(first
            .orogenic_intensity
            .iter()
            .all(|value| value.is_finite()));
    }

    #[test]
    fn present_legacy_orogenic_history_does_not_define_new_provinces() {
        let seed = "wg36-causal-cut";
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let tectonics = generate_tectonics(&topology, &TectonicsRequest::new(seed, 16), planet).unwrap();
        let history = generate_tectonic_history(
            &topology,
            &tectonics,
            &TectonicHistoryRequest::new(seed),
            planet,
        )
        .unwrap();
        let geology = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new(seed),
            planet,
        )
        .unwrap();
        let pre = generate_pre_orogenic_lithosphere(
            &topology,
            &tectonics,
            &history,
            &geology,
            &PreOrogenicLithosphereRequest::new(seed),
        )
        .unwrap();
        let baseline = generate_tectonic_orogen_provinces(
            &topology,
            &tectonics,
            &history,
            &geology,
            &pre,
            &OrogenProvinceRequest::new(seed),
            planet,
        )
        .unwrap();
        let mut mutated = geology.clone();
        mutated.orogenic_history.fill(1.0);
        mutated.crustal_strain.fill(1.0);
        mutated.subduction_history.fill(1.0);
        mutated.crust_thickness_km.fill(59.0);
        mutated.crust_density_kg_per_m3.fill(3190.0);
        mutated.buoyancy_index.fill(-1.0);
        let changed = generate_tectonic_orogen_provinces(
            &topology,
            &tectonics,
            &history,
            &mutated,
            &pre,
            &OrogenProvinceRequest::new(seed),
            planet,
        )
        .unwrap();
        assert_eq!(baseline.metrics.province_hash, changed.metrics.province_hash);
    }

    #[test]
    fn populated_samples_reference_real_connected_provinces() {
        let model = generate("wg36-province-identity");
        for id in &model.province_ids {
            if *id > 0 {
                assert!((*id as usize) <= model.provinces.len());
            }
        }
        for (index, province) in model.provinces.iter().enumerate() {
            assert_eq!(province.id as usize, index + 1);
            assert!(!province.boundary_indices.is_empty());
        }
    }
}
