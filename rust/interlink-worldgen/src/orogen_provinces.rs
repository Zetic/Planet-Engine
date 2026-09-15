use crate::{
    random, CrustKind, CrustalModel, InheritedStructureKind, PlanetPhysicalParameters,
    PlanetTopology, PlateBoundaryEdge, PlateBoundaryKind, PreOrogenicLithosphereModel,
    StageIdentity, TectonicHistoryModel, TectonicModel, WorldgenError,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub const OROGEN_PROVINCE_STAGE_ID: &str = "geology:tectonic-orogen-provinces";
pub const OROGEN_PROVINCE_STAGE_VERSION: u32 = 4;
const OROGEN_PROVINCE_NAMESPACE: &str = "worldgen:geology:tectonic-orogen-provinces:v4";
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
    /// Physical distance to the strongest connected convergent source. Structural belts may be
    /// displaced from this distance by inherited lithosphere and multi-source transfer geometry.
    pub boundary_distance_km: Vec<f32>,
    /// High-relief structural belts. In v4 this is no longer a direct boundary-normal ridge:
    /// primary and secondary range axes can migrate into inherited weak corridors.
    pub mountain_core_index: Vec<f32>,
    /// Pre-orogenic resistance to inland deformation; high values represent strong/cratonic substrate.
    pub interior_resistance_index: Vec<f32>,
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
    plateau_eligibility: f64,
}

#[derive(Clone, Copy, Debug)]
struct DistanceFrontier {
    cost_km: f64,
    physical_distance_km: f64,
    source_index: usize,
    sample: u32,
}
impl PartialEq for DistanceFrontier {
    fn eq(&self, other: &Self) -> bool {
        self.cost_km.to_bits() == other.cost_km.to_bits()
            && self.physical_distance_km.to_bits() == other.physical_distance_km.to_bits()
            && self.source_index == other.source_index
            && self.sample == other.sample
    }
}
impl Eq for DistanceFrontier {}
impl Ord for DistanceFrontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost_km
            .total_cmp(&self.cost_km)
            .then_with(|| {
                other
                    .physical_distance_km
                    .total_cmp(&self.physical_distance_km)
            })
            .then_with(|| other.source_index.cmp(&self.source_index))
            .then_with(|| other.sample.cmp(&self.sample))
    }
}
impl PartialOrd for DistanceFrontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct StructureSignals {
    weak_corridor: f64,
    inherited_belt: f64,
    craton: f64,
    rift: f64,
    transfer: f64,
    paleo_suture: f64,
}

#[derive(Clone, Copy, Debug, Default)]
struct StructuralProfile {
    province_id: u16,
    kind: u8,
    weight: f64,
    width_km: f64,
    boundary_distance_km: f64,
    along_fraction: f64,
    maturity: f64,
    shortening: f64,
    mountain: f64,
    root: f64,
    plateau: f64,
    fold: f64,
    foreland: f64,
    arc: f64,
    backarc: f64,
    suture: f64,
    transpression: f64,
    intensity: f64,
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
    boundary: &PlateBoundaryEdge,
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
        let subducting = if oceanic_a {
            boundary.plate_a
        } else {
            boundary.plate_b
        };
        let overriding = if oceanic_a {
            boundary.plate_b
        } else {
            boundary.plate_a
        };
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

fn fragment_contact_strength(pre: &PreOrogenicLithosphereModel, a: u32, b: u32) -> f64 {
    let fragment_a = pre.fragment_ids[a as usize];
    let fragment_b = pre.fragment_ids[b as usize];
    match (fragment_a > 0, fragment_b > 0) {
        (true, true) => 1.0,
        (true, false) | (false, true) => 0.55,
        (false, false) => 0.0,
    }
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
            fragment_contact: fragment_contact_strength(pre, a, b),
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
        BoundaryClass::Collision => 2600.0,
        BoundaryClass::Cordilleran => 2200.0,
        BoundaryClass::IslandArc => 1600.0,
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
        let cut = traits.class != previous.class
            || (span > 720.0 && discontinuity(previous, traits) > 0.82)
            || span > segmentation_length_km(previous.class);
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
    let mean_weakness = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().weakness)
        .sum::<f64>()
        / count;
    let mean_te_km = segment
        .iter()
        .map(|edge| edge_traits[*edge].unwrap().mean_te_km)
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
            if mean_fragment >= 0.55 {
                OrogenProvinceKind::TerraneAccretion
            } else if maturity >= 0.66
                && shortening >= 0.60
                && mean_weakness >= 0.32
                && mean_te_km <= 72.0
            {
                OrogenProvinceKind::CollisionalPlateau
            } else {
                OrogenProvinceKind::ContinentalCollision
            }
        }
    }
}

fn base_width_km(kind: OrogenProvinceKind, maturity: f64, weakness: f64) -> f64 {
    match kind {
        OrogenProvinceKind::ContinentalCollision => 245.0 + maturity * 300.0 + weakness * 125.0,
        OrogenProvinceKind::CollisionalPlateau => 420.0 + maturity * 390.0 + weakness * 165.0,
        OrogenProvinceKind::CordilleranArc => 145.0 + maturity * 145.0 + weakness * 70.0,
        OrogenProvinceKind::IslandArc => 105.0 + maturity * 115.0 + weakness * 55.0,
        OrogenProvinceKind::TerraneAccretion => 195.0 + maturity * 205.0 + weakness * 100.0,
        OrogenProvinceKind::TranspressionalOrogen => 130.0 + maturity * 145.0 + weakness * 65.0,
    }
}

fn source_widths(
    traits: EdgeTraits,
    kind: OrogenProvinceKind,
    system_has_endpoints: bool,
) -> (f64, f64, f64, f64, f64, f64) {
    let maturity =
        0.42 * clamp01(traits.age_myr / 95.0) + 0.58 * clamp01(traits.convergence_km / 3200.0);
    let shortening = clamp01(traits.convergence_km / 3000.0);
    let local_control = (0.72
        + traits.weakness * 0.34
        + traits.fabric * 0.18
        + traits.age_discontinuity * 0.14
        + traits.province_boundary * 0.10
        + clamp01(traits.curvature_deg / 90.0) * 0.10
        + traits.fragment_contact * 0.08)
        .clamp(0.68, 1.48);
    let mut base = base_width_km(kind, maturity, traits.weakness) * local_control;

    let taper = if system_has_endpoints {
        smoothstep(traits.along_fraction / 0.15) * smoothstep((1.0 - traits.along_fraction) / 0.15)
    } else {
        1.0
    };
    base *= 0.28 + taper * 0.72;

    let (minimum, maximum) = match kind {
        OrogenProvinceKind::CollisionalPlateau => (280.0, 1000.0),
        OrogenProvinceKind::ContinentalCollision => (165.0, 780.0),
        OrogenProvinceKind::TerraneAccretion => (135.0, 680.0),
        OrogenProvinceKind::CordilleranArc => (95.0, 460.0),
        OrogenProvinceKind::IslandArc => (75.0, 360.0),
        OrogenProvinceKind::TranspressionalOrogen => (75.0, 380.0),
    };
    base = base.clamp(minimum, maximum);

    let asymmetry = ((traits.strength_b - traits.strength_a) * 0.42 + (traits.fabric - 0.5) * 0.10)
        .clamp(-0.30, 0.30);
    let mut width_a = (base * (1.0 + asymmetry)).clamp(minimum * 0.78, maximum);
    let mut width_b = (base * (1.0 - asymmetry)).clamp(minimum * 0.78, maximum);
    if matches!(
        kind,
        OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc
    ) {
        if traits.overriding_plate == traits.plate_a {
            width_a = base.clamp(minimum, maximum);
            width_b = (base * 0.18).clamp(45.0, maximum);
        } else {
            width_b = base.clamp(minimum, maximum);
            width_a = (base * 0.18).clamp(45.0, maximum);
        }
    }

    let orthogonality = 1.0 - clamp01(traits.obliquity_deg / 90.0);
    let shortening_gate = smoothstep(shortening / 0.55);
    let tectonic_focus = clamp01(
        0.26
            + traits.weakness * 0.18
            + traits.age_discontinuity * 0.22
            + traits.province_boundary * 0.12
            + clamp01(traits.curvature_deg / 90.0) * 0.14
            + traits.fragment_contact * 0.10,
    );
    let core_strength = clamp01(
        shortening_gate
            * (0.32 + maturity * 0.68)
            * (0.76 + orthogonality * 0.24)
            * (0.42 + taper * 0.58)
            * (0.80 + tectonic_focus * 0.20),
    );
    let te_norm = clamp01((traits.mean_te_km - 18.0) / 62.0);
    let plateau_eligibility = if kind == OrogenProvinceKind::CollisionalPlateau {
        clamp01(
            smoothstep((maturity - 0.58) / 0.32)
                * smoothstep((shortening - 0.52) / 0.38)
                * (0.55 + traits.weakness * 0.45)
                * (1.0 - 0.42 * te_norm),
        )
    } else {
        0.0
    };
    (
        width_a,
        width_b,
        maturity,
        shortening,
        core_strength,
        plateau_eligibility,
    )
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
            .map(|value| *value as usize)
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
            let first = edge_traits[segment[0]].unwrap();

            for edge in &segment {
                let traits = edge_traits[*edge].unwrap();
                let (width_a, width_b, maturity, shortening, core_strength, plateau_eligibility) =
                    source_widths(traits, kind, system_has_endpoints);
                widths.extend_from_slice(&[width_a, width_b]);
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
                    plateau_eligibility,
                });
            }

            let count = segment.len() as f64;
            let mean_width = widths.iter().sum::<f64>() / widths.len().max(1) as f64;
            let min_width = widths.iter().copied().fold(f64::INFINITY, f64::min);
            let max_width = widths.iter().copied().fold(0.0_f64, f64::max);
            provinces.push(OrogenProvince {
                id,
                boundary_system_id: first.system_id,
                boundary_indices: segment.iter().map(|edge| *edge as u32).collect(),
                kind,
                plate_a: first.plate_a,
                plate_b: first.plate_b,
                overriding_plate_id: first.overriding_plate,
                hinterland_plate_id: first.hinterland_plate,
                foreland_or_subducting_plate_id: first.foreland_or_subducting_plate,
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

fn sample_interior_resistance(pre: &PreOrogenicLithosphereModel, sample: usize) -> f64 {
    let strength = f64::from(pre.intrinsic_strength_index[sample]).clamp(0.0, 1.0);
    let weakness = f64::from(pre.intrinsic_weakness_index[sample]).clamp(0.0, 1.0);
    let te = clamp01((f64::from(pre.effective_elastic_thickness_km[sample]) - 8.0) / 78.0);
    let fabric = f64::from(pre.inherited_fabric_strength[sample]).clamp(0.0, 1.0);
    let rift = f64::from(pre.inherited_rift_memory[sample]).clamp(0.0, 1.0);
    let shear = f64::from(pre.inherited_shear_memory[sample]).clamp(0.0, 1.0);
    let age_break = f64::from(pre.age_discontinuity_index[sample]).clamp(0.0, 1.0);
    let province_break = f64::from(pre.province_boundary_index[sample]).clamp(0.0, 1.0);
    let weak_corridor = clamp01(0.42 * fabric + 0.30 * rift + 0.18 * shear + 0.10 * age_break);
    clamp01(
        0.12 + 0.50 * strength + 0.20 * te + 0.18 * (1.0 - weakness)
            - 0.48 * weak_corridor
            - 0.10 * province_break,
    )
}

fn structure_signals(pre: &PreOrogenicLithosphereModel, sample: usize) -> StructureSignals {
    let strength = f64::from(pre.intrinsic_strength_index[sample]).clamp(0.0, 1.0);
    let weakness = f64::from(pre.intrinsic_weakness_index[sample]).clamp(0.0, 1.0);
    let te = clamp01((f64::from(pre.effective_elastic_thickness_km[sample]) - 8.0) / 78.0);
    let fabric = f64::from(pre.inherited_fabric_strength[sample]).clamp(0.0, 1.0);
    let rift = f64::from(pre.inherited_rift_memory[sample]).clamp(0.0, 1.0);
    let shear = f64::from(pre.inherited_shear_memory[sample]).clamp(0.0, 1.0);
    let age_break = f64::from(pre.age_discontinuity_index[sample]).clamp(0.0, 1.0);
    let province_break = f64::from(pre.province_boundary_index[sample]).clamp(0.0, 1.0);
    let structure = pre.inherited_structure_kind[sample];
    let paleo_suture = if structure == InheritedStructureKind::PaleoSuture as u8 {
        1.0
    } else {
        0.0
    };
    let inherited_rift = if structure == InheritedStructureKind::InheritedRift as u8 {
        1.0
    } else {
        0.0
    };
    let shear_zone = if structure == InheritedStructureKind::ShearZone as u8 {
        1.0
    } else {
        0.0
    };
    let craton_boundary = if structure == InheritedStructureKind::CratonBoundary as u8 {
        1.0
    } else {
        0.0
    };
    let weak_corridor = clamp01(
        0.34 * fabric
            + 0.24 * shear
            + 0.17 * rift
            + 0.11 * age_break
            + 0.08 * province_break
            + 0.06 * paleo_suture,
    );
    let inherited_belt = clamp01(
        0.34 * fabric
            + 0.20 * shear
            + 0.16 * age_break
            + 0.13 * province_break
            + 0.10 * paleo_suture
            + 0.07 * craton_boundary,
    );
    let craton = clamp01(
        smoothstep((strength - 0.48) / 0.42)
            * smoothstep((te - 0.36) / 0.48)
            * (1.0 - 0.52 * weakness)
            * (1.0 - 0.36 * rift),
    );
    let transfer = clamp01(0.34 * province_break + 0.28 * shear + 0.20 * age_break + 0.18 * fabric);
    StructureSignals {
        weak_corridor,
        inherited_belt,
        craton,
        rift: clamp01(0.72 * rift + 0.28 * inherited_rift),
        transfer,
        paleo_suture,
    }
}

fn propagation_penalty(kind: OrogenProvinceKind, resistance: f64) -> f64 {
    let r = clamp01(resistance);
    let gain = match kind {
        OrogenProvinceKind::CollisionalPlateau => 1.45,
        OrogenProvinceKind::ContinentalCollision => 2.35,
        OrogenProvinceKind::TerraneAccretion => 2.00,
        OrogenProvinceKind::TranspressionalOrogen => 2.70,
        OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc => 0.72,
    };
    1.0 + gain * r.powf(1.30)
}

fn insert_source_candidate(
    index: usize,
    source_index: usize,
    cost_km: f64,
    physical_distance_km: f64,
    best_cost: &mut [f64],
    best_physical: &mut [f64],
    best_source: &mut [usize],
    second_cost: &mut [f64],
    second_physical: &mut [f64],
    second_source: &mut [usize],
) -> bool {
    const EPS: f64 = 1.0e-9;
    if best_source[index] == source_index {
        if cost_km + EPS < best_cost[index] {
            best_cost[index] = cost_km;
            best_physical[index] = physical_distance_km;
            return true;
        }
        return false;
    }
    if second_source[index] == source_index {
        if cost_km + EPS < second_cost[index] {
            second_cost[index] = cost_km;
            second_physical[index] = physical_distance_km;
            if second_cost[index] + EPS < best_cost[index] {
                std::mem::swap(&mut best_cost[index], &mut second_cost[index]);
                std::mem::swap(&mut best_physical[index], &mut second_physical[index]);
                std::mem::swap(&mut best_source[index], &mut second_source[index]);
            }
            return true;
        }
        return false;
    }
    if cost_km + EPS < best_cost[index] {
        second_cost[index] = best_cost[index];
        second_physical[index] = best_physical[index];
        second_source[index] = best_source[index];
        best_cost[index] = cost_km;
        best_physical[index] = physical_distance_km;
        best_source[index] = source_index;
        return true;
    }
    if cost_km + EPS < second_cost[index] {
        second_cost[index] = cost_km;
        second_physical[index] = physical_distance_km;
        second_source[index] = source_index;
        return true;
    }
    false
}

#[allow(clippy::type_complexity)]
fn nearest_source_pairs<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    sources: &[BoundarySource],
    resistance: &[f64],
    parameters: PlanetPhysicalParameters,
) -> (
    Vec<f64>,
    Vec<f64>,
    Vec<usize>,
    Vec<f64>,
    Vec<f64>,
    Vec<usize>,
) {
    let count = topology.sample_count() as usize;
    let mut best_cost = vec![f64::INFINITY; count];
    let mut best_physical = vec![f64::INFINITY; count];
    let mut best_source = vec![usize::MAX; count];
    let mut second_cost = vec![f64::INFINITY; count];
    let mut second_physical = vec![f64::INFINITY; count];
    let mut second_source = vec![usize::MAX; count];
    let mut frontier = BinaryHeap::new();

    for (source_index, source) in sources.iter().enumerate() {
        let boundary = &tectonics.boundaries[source.boundary_index];
        for sample in [boundary.sample_a, boundary.sample_b] {
            let index = sample as usize;
            let plate = tectonics.plate_ids[index];
            if plate != source.plate_a && plate != source.plate_b {
                continue;
            }
            if insert_source_candidate(
                index,
                source_index,
                0.0,
                0.0,
                &mut best_cost,
                &mut best_physical,
                &mut best_source,
                &mut second_cost,
                &mut second_physical,
                &mut second_source,
            ) {
                frontier.push(DistanceFrontier {
                    cost_km: 0.0,
                    physical_distance_km: 0.0,
                    source_index,
                    sample,
                });
            }
        }
    }

    let radius_km = parameters.radius_m / 1000.0;
    while let Some(current) = frontier.pop() {
        let index = current.sample as usize;
        let retained = (best_source[index] == current.source_index
            && current.cost_km <= best_cost[index] + 1.0e-9)
            || (second_source[index] == current.source_index
                && current.cost_km <= second_cost[index] + 1.0e-9);
        if !retained {
            continue;
        }
        let plate = tectonics.plate_ids[index];
        let source = sources[current.source_index];
        let neighbors = topology.neighbors(current.sample);
        let lengths = topology.neighbor_arc_lengths_rad(current.sample);
        for neighbor_index in 0..neighbors.len() {
            let neighbor = neighbors[neighbor_index];
            let target = neighbor as usize;
            if tectonics.plate_ids[target] != plate {
                continue;
            }
            let step_km = lengths[neighbor_index] * radius_km;
            let edge_resistance = (resistance[index] + resistance[target]) * 0.5;
            let candidate_cost =
                current.cost_km + step_km * propagation_penalty(source.kind, edge_resistance);
            let candidate_physical = current.physical_distance_km + step_km;
            if insert_source_candidate(
                target,
                current.source_index,
                candidate_cost,
                candidate_physical,
                &mut best_cost,
                &mut best_physical,
                &mut best_source,
                &mut second_cost,
                &mut second_physical,
                &mut second_source,
            ) {
                frontier.push(DistanceFrontier {
                    cost_km: candidate_cost,
                    physical_distance_km: candidate_physical,
                    source_index: current.source_index,
                    sample: neighbor,
                });
            }
        }
    }

    (
        best_cost,
        best_physical,
        best_source,
        second_cost,
        second_physical,
        second_source,
    )
}

fn collision_axis_center(signals: StructureSignals, source: BoundarySource) -> f64 {
    let inherited_shift = 0.17 * signals.weak_corridor
        + 0.08 * signals.transfer
        + 0.06 * signals.paleo_suture
        + 0.04 * signals.rift;
    let maturity_shift = 0.05 * source.maturity * source.shortening;
    (0.16 + inherited_shift + maturity_shift).clamp(0.14, 0.52)
}

fn source_profile(
    source: BoundarySource,
    plate: u16,
    cost_distance_km: f64,
    physical_distance_km: f64,
    resistance: f64,
    signals: StructureSignals,
) -> StructuralProfile {
    let width = if plate == source.plate_a {
        source.width_a_km
    } else if plate == source.plate_b {
        source.width_b_km
    } else {
        return StructuralProfile::default();
    };
    let x = cost_distance_km / width.max(1.0);
    let reach_limit = match source.kind {
        OrogenProvinceKind::CollisionalPlateau => 1.95,
        OrogenProvinceKind::ContinentalCollision => 1.82,
        OrogenProvinceKind::TerraneAccretion => 1.78,
        OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc => 1.70,
        OrogenProvinceKind::TranspressionalOrogen => 1.60,
    };
    if x > reach_limit {
        return StructuralProfile::default();
    }
    let reach_envelope = smoothstep((reach_limit - x) / (reach_limit * 0.40));
    let overriding = plate == source.overriding_plate;
    let hinterland = plate == source.hinterland_plate;
    let foreland_side = plate == source.foreland_or_subducting_plate;
    let subduction = matches!(
        source.kind,
        OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc
    );
    let transmission = (1.0 - 0.40 * resistance).clamp(0.54, 1.0);

    let mut profile = StructuralProfile {
        province_id: source.province_id,
        kind: source.kind as u8,
        weight: reach_envelope * (0.38 + 0.62 * source.core_strength.max(0.15)),
        width_km: width,
        boundary_distance_km: physical_distance_km,
        along_fraction: source.along_fraction,
        maturity: source.maturity,
        shortening: source.shortening,
        ..StructuralProfile::default()
    };

    if subduction {
        if overriding {
            let arc_center_km = 145.0
                + 115.0 * source.maturity
                + 36.0 * source.curvature
                + 55.0 * signals.weak_corridor;
            let arc_sigma_km = 78.0 + 48.0 * (1.0 - resistance) + 18.0 * signals.transfer;
            let arc_profile = gaussian(physical_distance_km, arc_center_km, arc_sigma_km);
            let secondary_arc = gaussian(
                physical_distance_km,
                arc_center_km + 150.0 + 55.0 * signals.transfer,
                arc_sigma_km * 1.15,
            ) * signals.inherited_belt
                * source.maturity
                * 0.32;
            profile.mountain = clamp01(
                source.core_strength
                    * (0.58 + 0.42 * source.maturity)
                    * (arc_profile + secondary_arc)
                    * reach_envelope,
            );
            profile.root = clamp01(
                source.core_strength
                    * (gaussian(physical_distance_km, arc_center_km, arc_sigma_km * 1.55)
                        + secondary_arc * 0.55)
                    * 0.60
                    * transmission,
            );
            profile.fold = clamp01(
                source.core_strength
                    * gaussian(
                        physical_distance_km,
                        arc_center_km + 80.0,
                        arc_sigma_km * 1.45,
                    )
                    * 0.40,
            );
            profile.arc = clamp01(
                (0.52 + 0.48 * source.maturity)
                    * (gaussian(physical_distance_km, arc_center_km, arc_sigma_km * 0.78)
                        + secondary_arc * 0.22),
            );
            profile.backarc = clamp01(
                source.maturity
                    * gaussian(physical_distance_km, arc_center_km + 275.0, 190.0)
                    * (0.34 + 0.14 * signals.rift),
            );
            profile.suture =
                clamp01(source.core_strength * gaussian(physical_distance_km, 0.0, 75.0) * 0.15);
            profile.intensity = profile
                .mountain
                .max(profile.arc * 0.82)
                .max(profile.suture * 0.20);
        } else {
            profile.suture =
                clamp01(source.core_strength * gaussian(physical_distance_km, 0.0, 70.0) * 0.20);
            profile.root = clamp01(profile.suture * 0.24);
            profile.intensity = profile.suture * 0.18;
        }
        return profile;
    }

    let side_amplitude = if hinterland {
        1.0
    } else if foreland_side {
        0.90
    } else {
        0.76
    };
    let axis_center = collision_axis_center(signals, source);
    let primary_sigma = 0.24 + 0.07 * signals.transfer + 0.04 * (1.0 - resistance);
    let primary = gaussian(x, axis_center, primary_sigma);
    let secondary_eligibility = clamp01(
        smoothstep((source.maturity - 0.25) / 0.60)
            * smoothstep((source.shortening - 0.20) / 0.65)
            * (0.30 + 0.70 * signals.inherited_belt),
    );
    let secondary_center = (axis_center
        + 0.42
        + 0.16 * (1.0 - resistance)
        + 0.10 * signals.transfer)
        .clamp(0.48, 1.16);
    let secondary = gaussian(x, secondary_center, 0.24 + 0.08 * signals.transfer)
        * secondary_eligibility;
    let tertiary = gaussian(x, secondary_center + 0.32, 0.20)
        * secondary_eligibility
        * signals.transfer
        * 0.42;
    let boundary_core_suppression = 0.58 + 0.42 * smoothstep(x / 0.10);
    profile.mountain = clamp01(
        source.core_strength.powf(1.05)
            * (primary + 0.62 * secondary + tertiary)
            * side_amplitude
            * (0.82 + 0.18 * transmission)
            * reach_envelope
            * boundary_core_suppression,
    );
    profile.root = clamp01(
        source.core_strength
            * (0.82 * gaussian(x, axis_center + 0.08, primary_sigma * 1.55)
                + 0.46 * secondary
                + 0.22 * tertiary)
            * if hinterland { 1.0 } else { 0.80 }
            * transmission,
    );
    profile.plateau = if source.kind == OrogenProvinceKind::CollisionalPlateau {
        clamp01(
            source.plateau_eligibility
                * (0.72 * gaussian(x, 0.68, 0.45) + 0.42 * secondary)
                * if hinterland { 1.0 } else { 0.24 }
                * transmission,
        )
    } else {
        clamp01(
            source.maturity
                * source.shortening
                * signals.craton
                * gaussian(x, 0.68, 0.52)
                * 0.10,
        )
    };
    profile.fold = clamp01(
        source.core_strength
            * (gaussian(x, 0.92, 0.32) + 0.24 * tertiary)
            * if foreland_side { 1.0 } else { 0.46 }
            * (0.72 + 0.28 * transmission),
    );
    profile.foreland = if foreland_side {
        clamp01(
            source.maturity
                * source.shortening
                * gaussian(x, 1.30 + 0.08 * signals.transfer, 0.38)
                * reach_envelope
                * 0.60,
        )
    } else {
        0.0
    };
    profile.suture = clamp01(
        source.core_strength * gaussian(x, 0.0, 0.09) * (0.80 + 0.20 * signals.paleo_suture),
    );
    profile.transpression = clamp01(
        source.core_strength
            * source.obliquity
            * (0.70 + source.curvature * 0.20 + signals.transfer * 0.10)
            * (0.78 * primary + 0.50 * secondary),
    );
    profile.intensity = profile
        .mountain
        .max(profile.fold * 0.75)
        .max(profile.plateau * 0.55)
        .max(profile.transpression * 0.85)
        * reach_envelope;
    profile
}

fn apply_interior_morphology(
    geology: &CrustalModel,
    pre: &PreOrogenicLithosphereModel,
    sample: usize,
    signals: StructureSignals,
    mountain: &mut [f32],
    root: &mut [f32],
    plateau: &mut [f32],
    fold: &mut [f32],
    suture: &mut [f32],
    transpression: &mut [f32],
    orogenic: &mut [f32],
) {
    let crust = geology.crust_kind[sample];
    let continental_factor = if crust == CrustKind::Continental as u8 {
        1.0
    } else if crust == CrustKind::Transitional as u8 {
        0.42
    } else {
        return;
    };
    let fragment = if pre.fragment_ids[sample] > 0 { 1.0 } else { 0.0 };
    let shield = signals.craton * (1.0 - 0.56 * signals.rift) * continental_factor;
    let old_belt = signals.inherited_belt
        * (0.52 + 0.48 * (1.0 - signals.craton))
        * (1.0 - 0.34 * signals.rift)
        * continental_factor;
    let mobile = clamp01(old_belt * (0.74 + 0.16 * fragment + 0.10 * signals.transfer));

    root[sample] = root[sample].max((0.14 * shield + 0.15 * mobile) as f32);
    plateau[sample] = plateau[sample].max((0.085 * shield + 0.045 * mobile) as f32);
    mountain[sample] = mountain[sample].max((0.075 * mobile) as f32);
    fold[sample] = fold[sample].max((0.080 * mobile * (0.55 + 0.45 * signals.transfer)) as f32);
    suture[sample] = suture[sample]
        .max((0.15 * signals.paleo_suture + 0.055 * signals.transfer) as f32);
    transpression[sample] = transpression[sample].max((0.060 * mobile * signals.transfer) as f32);
    orogenic[sample] = orogenic[sample].max((0.10 * mobile + 0.035 * shield) as f32);
}

fn blend_profiles(primary: StructuralProfile, secondary: StructuralProfile) -> StructuralProfile {
    if primary.weight <= 0.0 {
        return secondary;
    }
    if secondary.weight <= 0.0 {
        return primary;
    }
    let scale = primary.width_km.max(secondary.width_km).max(80.0) * 0.42;
    let distance_delta = (secondary.boundary_distance_km - primary.boundary_distance_km).abs();
    let proximity = (-distance_delta / scale).exp();
    let relative = clamp01((secondary.weight / primary.weight.max(1.0e-9)) * proximity);
    if relative < 0.08 {
        return primary;
    }

    let different_province = primary.province_id != secondary.province_id;
    let junction = if different_province {
        relative * primary.intensity.min(secondary.intensity)
    } else {
        0.0
    };
    let choose_secondary = secondary.intensity * relative > primary.intensity;
    StructuralProfile {
        province_id: if choose_secondary {
            secondary.province_id
        } else {
            primary.province_id
        },
        kind: if choose_secondary { secondary.kind } else { primary.kind },
        weight: primary.weight.max(secondary.weight * relative),
        width_km: primary.width_km * (1.0 - 0.34 * relative)
            + secondary.width_km * (0.34 * relative),
        boundary_distance_km: primary.boundary_distance_km.min(secondary.boundary_distance_km),
        along_fraction: primary.along_fraction * (1.0 - 0.38 * relative)
            + secondary.along_fraction * (0.38 * relative),
        maturity: primary.maturity.max(secondary.maturity * relative),
        shortening: primary.shortening.max(secondary.shortening * relative),
        mountain: clamp01(primary.mountain.max(secondary.mountain * (0.58 + 0.34 * relative)) + 0.10 * junction),
        root: clamp01(primary.root.max(secondary.root * 0.72) + 0.22 * junction),
        plateau: clamp01(primary.plateau.max(secondary.plateau * 0.68) + 0.10 * junction),
        fold: clamp01(primary.fold.max(secondary.fold * 0.76) + 0.18 * junction),
        foreland: primary.foreland.max(secondary.foreland * 0.70),
        arc: primary.arc.max(secondary.arc * 0.82),
        backarc: primary.backarc.max(secondary.backarc * 0.72),
        suture: clamp01(primary.suture.max(secondary.suture * 0.80) + 0.10 * junction),
        transpression: clamp01(
            primary
                .transpression
                .max(secondary.transpression * 0.82)
                + 0.12 * junction,
        ),
        intensity: clamp01(primary.intensity.max(secondary.intensity * 0.82) + 0.08 * junction),
    }
}

fn smooth_same_plate<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    province_ids: &[u16],
    values: &[f32],
    passes: usize,
) -> Vec<f32> {
    let mut current = values.to_vec();
    let mut next = current.clone();
    for _ in 0..passes {
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            let plate = tectonics.plate_ids[index];
            let province = province_ids[index];
            let mut sum = f64::from(current[index]) * 5.0;
            let mut weight = 5.0;
            for neighbor in topology.neighbors(sample) {
                let neighbor = *neighbor as usize;
                if tectonics.plate_ids[neighbor] != plate {
                    continue;
                }
                let neighbor_province = province_ids[neighbor];
                let coupling = if neighbor_province == province {
                    1.0
                } else if province == 0 || neighbor_province == 0 {
                    0.42
                } else {
                    0.28
                };
                sum += f64::from(current[neighbor]) * coupling;
                weight += coupling;
            }
            next[index] = (sum / weight) as f32;
        }
        std::mem::swap(&mut current, &mut next);
    }
    current
}

#[allow(clippy::type_complexity)]
fn rasterize<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    geology: &CrustalModel,
    pre: &PreOrogenicLithosphereModel,
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
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
) {
    let count = topology.sample_count() as usize;
    let resistance = (0..count)
        .map(|sample| sample_interior_resistance(pre, sample))
        .collect::<Vec<_>>();
    let (
        best_cost,
        best_physical,
        best_source,
        second_cost,
        second_physical,
        second_source,
    ) = nearest_source_pairs(topology, tectonics, sources, &resistance, parameters);

    let mut province_ids = vec![0_u16; count];
    let mut province_kind = vec![0_u8; count];
    let mut orogenic = vec![0.0_f32; count];
    let mut boundary_distance = vec![0.0_f32; count];
    let mut mountain_core = vec![0.0_f32; count];
    let interior_resistance = resistance.iter().map(|value| *value as f32).collect::<Vec<_>>();
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
        let signals = structure_signals(pre, sample);
        apply_interior_morphology(
            geology,
            pre,
            sample,
            signals,
            &mut mountain_core,
            &mut root,
            &mut plateau,
            &mut fold_thrust,
            &mut suture,
            &mut transpression,
            &mut orogenic,
        );

        let primary_index = best_source[sample];
        if primary_index == usize::MAX
            || !best_cost[sample].is_finite()
            || !best_physical[sample].is_finite()
        {
            continue;
        }
        let plate = tectonics.plate_ids[sample];
        let primary = source_profile(
            sources[primary_index],
            plate,
            best_cost[sample],
            best_physical[sample],
            resistance[sample],
            signals,
        );
        if primary.weight <= 0.0 {
            continue;
        }
        let secondary = if second_source[sample] != usize::MAX
            && second_cost[sample].is_finite()
            && second_physical[sample].is_finite()
        {
            source_profile(
                sources[second_source[sample]],
                plate,
                second_cost[sample],
                second_physical[sample],
                resistance[sample],
                signals,
            )
        } else {
            StructuralProfile::default()
        };
        let profile = blend_profiles(primary, secondary);
        let active_signal = profile
            .intensity
            .max(profile.root * 0.45)
            .max(profile.fold * 0.50)
            .max(profile.foreland * 0.35)
            .max(profile.backarc * 0.30)
            .max(profile.suture * 0.20);
        if active_signal < 0.022 {
            continue;
        }

        province_ids[sample] = profile.province_id;
        province_kind[sample] = profile.kind;
        orogenic[sample] = orogenic[sample].max(profile.intensity as f32);
        boundary_distance[sample] = profile.boundary_distance_km as f32;
        mountain_core[sample] = mountain_core[sample].max(profile.mountain as f32);
        root[sample] = root[sample].max(profile.root as f32);
        plateau[sample] = plateau[sample].max(profile.plateau as f32);
        fold_thrust[sample] = fold_thrust[sample].max(profile.fold as f32);
        foreland[sample] = profile.foreland as f32;
        arc[sample] = profile.arc as f32;
        backarc[sample] = profile.backarc as f32;
        suture[sample] = suture[sample].max(profile.suture as f32);
        transpression[sample] = transpression[sample].max(profile.transpression as f32);
        maturity[sample] = profile.maturity as f32;
        shortening[sample] = profile.shortening as f32;
        local_width[sample] = profile.width_km as f32;
        along_fraction[sample] = profile.along_fraction as f32;
    }

    orogenic = smooth_same_plate(topology, tectonics, &province_ids, &orogenic, 2);
    mountain_core = smooth_same_plate(topology, tectonics, &province_ids, &mountain_core, 2);
    root = smooth_same_plate(topology, tectonics, &province_ids, &root, 2);
    plateau = smooth_same_plate(topology, tectonics, &province_ids, &plateau, 2);
    fold_thrust = smooth_same_plate(topology, tectonics, &province_ids, &fold_thrust, 2);
    foreland = smooth_same_plate(topology, tectonics, &province_ids, &foreland, 1);
    arc = smooth_same_plate(topology, tectonics, &province_ids, &arc, 1);
    backarc = smooth_same_plate(topology, tectonics, &province_ids, &backarc, 1);
    suture = smooth_same_plate(topology, tectonics, &province_ids, &suture, 1);
    transpression = smooth_same_plate(topology, tectonics, &province_ids, &transpression, 2);

    (
        province_ids,
        province_kind,
        orogenic,
        boundary_distance,
        mountain_core,
        interior_resistance,
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
    let mut hash = fnv_update(FNV_OFFSET_BASIS, b"interlink-orogen-provinces:v4\0");
    hash = fnv_update(hash, &model.stage.derived_seed.to_le_bytes());
    hash = hash_u16(hash, &model.province_ids);
    hash = hash_u8(hash, &model.province_kind);
    hash = hash_f32(hash, &model.orogenic_intensity);
    hash = hash_f32(hash, &model.boundary_distance_km);
    hash = hash_f32(hash, &model.mountain_core_index);
    hash = hash_f32(hash, &model.interior_resistance_index);
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
        widths.extend_from_slice(&[source.width_a_km, source.width_b_km]);
    }
    let mean_width = if widths.is_empty() {
        0.0
    } else {
        widths.iter().sum::<f64>() / widths.len() as f64
    };
    let min_width = widths.iter().copied().fold(f64::INFINITY, f64::min);
    let max_width = widths.iter().copied().fold(0.0_f64, f64::max);
    let mut total_area = 0.0;
    let mut active_area = 0.0;
    for sample in 0..topology.sample_count() {
        let area = topology.area_steradians(sample);
        total_area += area;
        if model.province_ids[sample as usize] > 0 {
            active_area += area;
        }
    }
    OrogenProvinceMetrics {
        sample_count: topology.sample_count(),
        province_count: model.provinces.len() as u16,
        continental_collision_count: counts[0],
        collisional_plateau_count: counts[1],
        cordilleran_arc_count: counts[2],
        island_arc_count: counts[3],
        terrane_accretion_count: counts[4],
        transpressional_count: counts[5],
        orogenic_area_fraction: active_area / total_area.max(1.0e-12),
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
        model.boundary_distance_km.len(),
        model.mountain_core_index.len(),
        model.interior_resistance_index.len(),
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
    let normalized: [&[f32]; 13] = [
        &model.orogenic_intensity,
        &model.mountain_core_index,
        &model.interior_resistance_index,
        &model.crustal_root_index,
        &model.plateau_index,
        &model.fold_thrust_index,
        &model.foreland_basin_index,
        &model.volcanic_arc_index,
        &model.backarc_extension_index,
        &model.suture_index,
        &model.transpression_index,
        &model.maturity_index,
        &model.shortening_index,
    ];
    if normalized
        .iter()
        .flat_map(|field| field.iter())
        .any(|value| !value.is_finite() || *value < 0.0 || *value > 1.0)
    {
        return Err(WorldgenError::InvalidLithosphere(
            "orogen province normalized field is outside [0,1]",
        ));
    }
    if model
        .local_width_km
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0 || *value > 1000.001)
    {
        return Err(WorldgenError::InvalidLithosphere(
            "orogen deformation reach is outside supported bounds",
        ));
    }
    if model
        .boundary_distance_km
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0 || *value > 3000.0)
    {
        return Err(WorldgenError::InvalidLithosphere(
            "orogen boundary distance is outside supported bounds",
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
        || pre.inherited_structure_kind.len() != count
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
        boundary_distance_km,
        mountain_core_index,
        interior_resistance_index,
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
    ) = rasterize(topology, tectonics, geology, pre, &sources, parameters);

    let mut model = OrogenProvinceModel {
        stage: StageIdentity {
            id: OROGEN_PROVINCE_STAGE_ID,
            version: OROGEN_PROVINCE_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        province_ids,
        province_kind,
        orogenic_intensity,
        boundary_distance_km,
        mountain_core_index,
        interior_resistance_index,
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
        metrics: OrogenProvinceMetrics {
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
        },
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
        generate_tectonic_history, generate_tectonics, GeologyRequest,
        PreOrogenicLithosphereRequest, TectonicHistoryRequest, TectonicsRequest,
    };

    fn world(
        seed: &str,
    ) -> (
        crate::GeodesicTopology,
        TectonicModel,
        TectonicHistoryModel,
        CrustalModel,
        PreOrogenicLithosphereModel,
    ) {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let tectonics =
            generate_tectonics(&topology, &TectonicsRequest::new(seed, 16), planet).unwrap();
        let history = generate_tectonic_history(
            &topology,
            &tectonics,
            &TectonicHistoryRequest::new(seed),
            planet,
        )
        .unwrap();
        let geology =
            generate_crust_and_history(&topology, &tectonics, &GeologyRequest::new(seed), planet)
                .unwrap();
        let pre = generate_pre_orogenic_lithosphere(
            &topology,
            &tectonics,
            &history,
            &geology,
            &PreOrogenicLithosphereRequest::new(seed),
        )
        .unwrap();
        (topology, tectonics, history, geology, pre)
    }

    #[test]
    fn orogen_provinces_are_deterministic_complete_and_finite() {
        let seed = "wg36-v4-orogen-determinism";
        let (topology, tectonics, history, geology, pre) = world(seed);
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let first = generate_tectonic_orogen_provinces(
            &topology,
            &tectonics,
            &history,
            &geology,
            &pre,
            &OrogenProvinceRequest::new(seed),
            planet,
        )
        .unwrap();
        let second = generate_tectonic_orogen_provinces(
            &topology,
            &tectonics,
            &history,
            &geology,
            &pre,
            &OrogenProvinceRequest::new(seed),
            planet,
        )
        .unwrap();
        assert_eq!(first.metrics.province_hash, second.metrics.province_hash);
        assert_eq!(first.province_ids, second.province_ids);
        assert!(!first.provinces.is_empty());
        assert!(first.orogenic_intensity.iter().all(|value| value.is_finite()));
    }

    #[test]
    fn orogen_provinces_ignore_legacy_present_day_orogenic_outputs() {
        let seed = "wg36-v4-causal-cut";
        let (topology, tectonics, history, geology, pre) = world(seed);
        let planet = PlanetPhysicalParameters::earthlike_reference();
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
    fn inland_resistance_increases_collision_propagation_cost() {
        let low = propagation_penalty(OrogenProvinceKind::ContinentalCollision, 0.10);
        let high = propagation_penalty(OrogenProvinceKind::ContinentalCollision, 0.90);
        let plateau_high = propagation_penalty(OrogenProvinceKind::CollisionalPlateau, 0.90);
        assert!(high > low * 1.8);
        assert!(plateau_high < high);
    }

    #[test]
    fn inherited_corridors_move_collision_axes_inland() {
        let source = BoundarySource {
            boundary_index: 0,
            province_id: 1,
            kind: OrogenProvinceKind::ContinentalCollision,
            plate_a: 1,
            plate_b: 2,
            overriding_plate: NO_PLATE,
            hinterland_plate: 1,
            foreland_or_subducting_plate: 2,
            along_fraction: 0.5,
            maturity: 0.72,
            shortening: 0.78,
            obliquity: 0.15,
            curvature: 0.3,
            width_a_km: 500.0,
            width_b_km: 420.0,
            core_strength: 0.85,
            plateau_eligibility: 0.0,
        };
        let plain = collision_axis_center(StructureSignals::default(), source);
        let inherited = collision_axis_center(
            StructureSignals {
                weak_corridor: 0.9,
                inherited_belt: 0.8,
                craton: 0.1,
                rift: 0.4,
                transfer: 0.8,
                paleo_suture: 1.0,
            },
            source,
        );
        assert!(plain >= 0.14);
        assert!(inherited > plain + 0.15);
        assert!(inherited <= 0.52);
    }

    #[test]
    fn structural_belts_are_not_forced_to_the_literal_boundary() {
        let seed = "wg36-v4-distributed-belts";
        let (topology, tectonics, history, geology, pre) = world(seed);
        let model = generate_tectonic_orogen_provinces(
            &topology,
            &tectonics,
            &history,
            &geology,
            &pre,
            &OrogenProvinceRequest::new(seed),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
        let mut core_count = 0usize;
        let mut inland_core_count = 0usize;
        for sample in 0..model.mountain_core_index.len() {
            if model.mountain_core_index[sample] > 0.20 {
                core_count += 1;
                if model.boundary_distance_km[sample] > 90.0 {
                    inland_core_count += 1;
                }
                assert!(
                    model.boundary_distance_km[sample] < 1300.0,
                    "mountain core escaped too far inland: {:.1} km",
                    model.boundary_distance_km[sample]
                );
            }
        }
        assert!(core_count > 0);
        assert!(inland_core_count > 0);
    }

    #[test]
    fn continental_interiors_receive_persistent_low_amplitude_structure() {
        let seed = "wg36-v4-interior-structure";
        let (topology, tectonics, history, geology, pre) = world(seed);
        let model = generate_tectonic_orogen_provinces(
            &topology,
            &tectonics,
            &history,
            &geology,
            &pre,
            &OrogenProvinceRequest::new(seed),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
        let structured_interior = (0..model.province_ids.len()).filter(|sample| {
            model.province_ids[*sample] == 0
                && geology.crust_kind[*sample] == CrustKind::Continental as u8
                && (model.crustal_root_index[*sample] > 0.025
                    || model.plateau_index[*sample] > 0.02
                    || model.suture_index[*sample] > 0.05)
        }).count();
        assert!(structured_interior > 0);
    }

    #[test]
    fn orogen_provinces_reference_real_connected_segments() {
        let seed = "wg36-v4-province-identity";
        let (topology, tectonics, history, geology, pre) = world(seed);
        let model = generate_tectonic_orogen_provinces(
            &topology,
            &tectonics,
            &history,
            &geology,
            &pre,
            &OrogenProvinceRequest::new(seed),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
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
