use crate::{
    random, CrustKind, CrustalModel, PlanetPhysicalParameters, PlanetTopology, PlateBoundaryEdge,
    PlateBoundaryKind, PreOrogenicLithosphereModel, StageIdentity, TectonicHistoryModel,
    TectonicModel, WorldgenError,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub const OROGEN_PROVINCE_STAGE_ID: &str = "geology:tectonic-orogen-provinces";
pub const OROGEN_PROVINCE_STAGE_VERSION: u32 = 3;
const OROGEN_PROVINCE_NAMESPACE: &str = "worldgen:geology:tectonic-orogen-provinces:v3";
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
    /// Boundary-normal distance to the winning connected convergent source. Meaningful only where province_ids > 0.
    pub boundary_distance_km: Vec<f32>,
    /// Narrow high-relief orogenic/arc spine. This is intentionally much narrower than the full deformation province.
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
        let cut = traits.class != previous.class
            || (span > 650.0 && discontinuity(previous, traits) > 0.70)
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
                // Tibetan-style broad plateaus are deliberately exceptional. A collision must be
                // mature, strongly shortened, and mechanically capable of transmitting strain.
                OrogenProvinceKind::CollisionalPlateau
            } else {
                OrogenProvinceKind::ContinentalCollision
            }
        }
    }
}

fn base_width_km(kind: OrogenProvinceKind, maturity: f64, weakness: f64) -> f64 {
    // Width is now a deformation-reach scale, not a direct mountain footprint. Values are
    // intentionally narrow enough that strong continental interiors can halt strain within
    // hundreds of kilometres while exceptional mature plateaus can still exceed 1000 km total.
    match kind {
        // Collision systems need room for a range + hinterland + fold/thrust transition.
        // The v2 widths were narrow enough that the range itself became a coloured boundary line.
        OrogenProvinceKind::ContinentalCollision => 220.0 + maturity * 260.0 + weakness * 110.0,
        OrogenProvinceKind::CollisionalPlateau => 380.0 + maturity * 360.0 + weakness * 150.0,
        OrogenProvinceKind::CordilleranArc => 135.0 + maturity * 125.0 + weakness * 60.0,
        OrogenProvinceKind::IslandArc => 100.0 + maturity * 105.0 + weakness * 50.0,
        OrogenProvinceKind::TerraneAccretion => 175.0 + maturity * 180.0 + weakness * 90.0,
        OrogenProvinceKind::TranspressionalOrogen => 115.0 + maturity * 125.0 + weakness * 55.0,
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
        + traits.weakness * 0.36
        + traits.fabric * 0.14
        + traits.age_discontinuity * 0.12
        + traits.province_boundary * 0.08
        + clamp01(traits.curvature_deg / 90.0) * 0.10
        + traits.fragment_contact * 0.08)
        .clamp(0.68, 1.42);
    let mut base = base_width_km(kind, maturity, traits.weakness) * local_control;

    let taper = if system_has_endpoints {
        smoothstep(traits.along_fraction / 0.16) * smoothstep((1.0 - traits.along_fraction) / 0.16)
    } else {
        1.0
    };
    base *= 0.24 + taper * 0.76;

    let (minimum, maximum) = match kind {
        OrogenProvinceKind::CollisionalPlateau => (260.0, 980.0),
        OrogenProvinceKind::ContinentalCollision => (150.0, 720.0),
        OrogenProvinceKind::TerraneAccretion => (125.0, 620.0),
        OrogenProvinceKind::CordilleranArc => (90.0, 420.0),
        OrogenProvinceKind::IslandArc => (70.0, 340.0),
        OrogenProvinceKind::TranspressionalOrogen => (70.0, 340.0),
    };
    base = base.clamp(minimum, maximum);

    let asymmetry = ((traits.strength_b - traits.strength_a) * 0.42 + (traits.fabric - 0.5) * 0.08)
        .clamp(-0.28, 0.28);
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
    // High mountains are not guaranteed merely because a boundary is convergent.  Require
    // accumulated shortening, then let inherited weakness/discontinuities and curvature focus
    // the load along strike.  This keeps convergent margins mountain-prone without turning every
    // edge of the plate graph into an equally tall ribbon.
    let shortening_gate = smoothstep(shortening / 0.55);
    let tectonic_focus = clamp01(
        0.28
            + traits.weakness * 0.18
            + traits.age_discontinuity * 0.20
            + traits.province_boundary * 0.10
            + clamp01(traits.curvature_deg / 90.0) * 0.14
            + traits.fragment_contact * 0.10,
    );
    let core_strength = clamp01(
        shortening_gate
            * (0.32 + maturity * 0.68)
            * (0.76 + orthogonality * 0.24)
            * (0.42 + taper * 0.58)
            * (0.82 + tectonic_focus * 0.18),
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

fn propagation_penalty(kind: OrogenProvinceKind, resistance: f64) -> f64 {
    let r = clamp01(resistance);
    let gain = match kind {
        OrogenProvinceKind::CollisionalPlateau => 1.60,
        OrogenProvinceKind::ContinentalCollision => 2.60,
        OrogenProvinceKind::TerraneAccretion => 2.20,
        OrogenProvinceKind::TranspressionalOrogen => 3.00,
        OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc => 0.75,
    };
    1.0 + gain * r.powf(1.35)
}

fn nearest_sources<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    sources: &[BoundarySource],
    resistance: &[f64],
    parameters: PlanetPhysicalParameters,
) -> (Vec<f64>, Vec<f64>, Vec<usize>) {
    let count = topology.sample_count() as usize;
    let mut cost = vec![f64::INFINITY; count];
    let mut physical_distance = vec![f64::INFINITY; count];
    let mut source_id = vec![usize::MAX; count];
    let mut frontier = BinaryHeap::new();

    for (source_index, source) in sources.iter().enumerate() {
        let boundary = &tectonics.boundaries[source.boundary_index];
        for sample in [boundary.sample_a, boundary.sample_b] {
            let index = sample as usize;
            let plate = tectonics.plate_ids[index];
            if plate != source.plate_a && plate != source.plate_b {
                continue;
            }
            if cost[index] > 0.0 || (cost[index] == 0.0 && source_index < source_id[index]) {
                cost[index] = 0.0;
                physical_distance[index] = 0.0;
                source_id[index] = source_index;
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
        if current.cost_km > cost[index] + 1.0e-9 || current.source_index != source_id[index] {
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
            if candidate_cost + 1.0e-9 < cost[target]
                || ((candidate_cost - cost[target]).abs() <= 1.0e-9
                    && current.source_index < source_id[target])
            {
                cost[target] = candidate_cost;
                physical_distance[target] = candidate_physical;
                source_id[target] = current.source_index;
                frontier.push(DistanceFrontier {
                    cost_km: candidate_cost,
                    physical_distance_km: candidate_physical,
                    source_index: current.source_index,
                    sample: neighbor,
                });
            }
        }
    }
    (cost, physical_distance, source_id)
}

fn smooth_within_province<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    province_ids: &[u16],
    values: &[f32],
) -> Vec<f32> {
    let mut smoothed = values.to_vec();
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let province = province_ids[index];
        if province == 0 {
            continue;
        }
        let plate = tectonics.plate_ids[index];
        let mut sum = f64::from(values[index]) * 4.0;
        let mut weight = 4.0;
        for neighbor in topology.neighbors(sample) {
            let neighbor = *neighbor as usize;
            if province_ids[neighbor] == province && tectonics.plate_ids[neighbor] == plate {
                sum += f64::from(values[neighbor]);
                weight += 1.0;
            }
        }
        smoothed[index] = (sum / weight) as f32;
    }
    smoothed
}

fn rasterize<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
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
    let (cost_distance, physical_distance, source_id) =
        nearest_sources(topology, tectonics, sources, &resistance, parameters);
    let mut province_ids = vec![0_u16; count];
    let mut province_kind = vec![0_u8; count];
    let mut orogenic = vec![0.0_f32; count];
    let mut boundary_distance = vec![0.0_f32; count];
    let mut mountain_core = vec![0.0_f32; count];
    let interior_resistance = resistance
        .iter()
        .map(|value| *value as f32)
        .collect::<Vec<_>>();
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
        if source_index == usize::MAX
            || !cost_distance[sample].is_finite()
            || !physical_distance[sample].is_finite()
        {
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
        let x = cost_distance[sample] / width.max(1.0);
        let reach_limit = match source.kind {
            OrogenProvinceKind::CollisionalPlateau => 1.85,
            OrogenProvinceKind::ContinentalCollision => 1.70,
            OrogenProvinceKind::TerraneAccretion => 1.65,
            OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc => 1.60,
            OrogenProvinceKind::TranspressionalOrogen => 1.45,
        };
        if x > reach_limit {
            continue;
        }
        let reach_envelope = smoothstep((reach_limit - x) / (reach_limit * 0.42));
        let overriding = plate == source.overriding_plate;
        let hinterland = plate == source.hinterland_plate;
        let foreland_side = plate == source.foreland_or_subducting_plate;
        let subduction = matches!(
            source.kind,
            OrogenProvinceKind::CordilleranArc | OrogenProvinceKind::IslandArc
        );
        let resistance_value = resistance[sample];
        let transmission = (1.0 - 0.42 * resistance_value).clamp(0.52, 1.0);
        let distance_km = physical_distance[sample];

        let (
            mountain_value,
            root_value,
            plateau_value,
            fold_value,
            foreland_value,
            arc_value,
            backarc_value,
            suture_value,
            transpression_value,
            intensity,
        ) = if subduction {
            if overriding {
                // The mountain/volcanic arc is explicitly offset inland from the trench. The
                // offset is in physical kilometres so a very broad province cannot push the arc
                // thousands of kilometres into the overriding plate.
                let arc_center_km = 145.0 + 120.0 * source.maturity + 35.0 * source.curvature;
                let arc_sigma_km = 80.0 + 45.0 * (1.0 - resistance_value);
                let arc_profile = gaussian(distance_km, arc_center_km, arc_sigma_km);
                let mountain = clamp01(
                    source.core_strength
                        * (0.58 + 0.42 * source.maturity)
                        * arc_profile
                        * reach_envelope,
                );
                let root_value = clamp01(
                    source.core_strength
                        * gaussian(distance_km, arc_center_km, arc_sigma_km * 1.45)
                        * 0.58
                        * transmission,
                );
                let fold_value = clamp01(
                    source.core_strength
                        * gaussian(distance_km, arc_center_km + 70.0, arc_sigma_km * 1.35)
                        * 0.38,
                );
                let arc_value = clamp01(
                    (0.52 + 0.48 * source.maturity)
                        * gaussian(distance_km, arc_center_km, arc_sigma_km * 0.78),
                );
                let backarc_value = clamp01(
                    source.maturity * gaussian(distance_km, arc_center_km + 260.0, 180.0) * 0.42,
                );
                let suture_value =
                    clamp01(source.core_strength * gaussian(distance_km, 0.0, 70.0) * 0.16);
                let intensity = mountain.max(arc_value * 0.82).max(suture_value * 0.20);
                (
                    mountain,
                    root_value,
                    0.0,
                    fold_value,
                    0.0,
                    arc_value,
                    backarc_value,
                    suture_value,
                    0.0,
                    intensity,
                )
            } else {
                let suture_value =
                    clamp01(source.core_strength * gaussian(distance_km, 0.0, 65.0) * 0.20);
                (
                    0.0,
                    clamp01(suture_value * 0.25),
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    suture_value,
                    0.0,
                    suture_value * 0.18,
                )
            }
        } else {
            let side_amplitude = if hinterland {
                1.0
            } else if foreland_side {
                0.92
            } else {
                0.76
            };
            let mountain = clamp01(
                source.core_strength.powf(1.08)
                    * gaussian(x, 0.18, 0.32)
                    * side_amplitude
                    * (0.82 + 0.18 * transmission)
                    * reach_envelope,
            );
            let root_value = clamp01(
                source.core_strength
                    * gaussian(x, 0.30, 0.44)
                    * if hinterland { 1.0 } else { 0.78 }
                    * transmission,
            );
            let plateau_value = if source.kind == OrogenProvinceKind::CollisionalPlateau {
                clamp01(
                    source.plateau_eligibility
                        * gaussian(x, 0.62, 0.40)
                        * if hinterland { 1.0 } else { 0.22 }
                        * transmission,
                )
            } else {
                0.0
            };
            let fold_value = clamp01(
                source.core_strength
                    * gaussian(x, 0.86, 0.30)
                    * if foreland_side { 1.0 } else { 0.45 }
                    * (0.72 + 0.28 * transmission),
            );
            let foreland_value = if foreland_side {
                clamp01(
                    source.maturity * source.shortening * gaussian(x, 1.22, 0.34) * reach_envelope * 0.62,
                )
            } else {
                0.0
            };
            let suture_value = clamp01(source.core_strength * gaussian(x, 0.0, 0.08));
            let transpression_value = clamp01(
                source.core_strength
                    * source.obliquity
                    * (0.72 + source.curvature * 0.28)
                    * gaussian(x, 0.22, 0.28),
            );
            let intensity = mountain
                .max(fold_value * 0.75)
                .max(plateau_value * 0.55)
                .max(transpression_value * 0.85)
                * reach_envelope;
            (
                mountain,
                root_value,
                plateau_value,
                fold_value,
                foreland_value,
                0.0,
                0.0,
                suture_value,
                transpression_value,
                intensity,
            )
        };

        let active_signal = intensity
            .max(root_value * 0.45)
            .max(fold_value * 0.50)
            .max(foreland_value * 0.35)
            .max(backarc_value * 0.30)
            .max(suture_value * 0.20);
        if active_signal < 0.025 {
            continue;
        }

        province_ids[sample] = source.province_id;
        province_kind[sample] = source.kind as u8;
        orogenic[sample] = intensity as f32;
        boundary_distance[sample] = distance_km as f32;
        mountain_core[sample] = mountain_value as f32;
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

    // Remove source-cell/Voronoi seams without blurring across plates or between distinct
    // tectonic provinces. The tectonic geometry stays causal; only adjacent samples owned by the
    // same connected province share a small amount of structural state.
    orogenic = smooth_within_province(topology, tectonics, &province_ids, &orogenic);
    mountain_core = smooth_within_province(topology, tectonics, &province_ids, &mountain_core);
    root = smooth_within_province(topology, tectonics, &province_ids, &root);
    plateau = smooth_within_province(topology, tectonics, &province_ids, &plateau);
    fold_thrust = smooth_within_province(topology, tectonics, &province_ids, &fold_thrust);
    foreland = smooth_within_province(topology, tectonics, &province_ids, &foreland);
    arc = smooth_within_province(topology, tectonics, &province_ids, &arc);
    backarc = smooth_within_province(topology, tectonics, &province_ids, &backarc);
    suture = smooth_within_province(topology, tectonics, &province_ids, &suture);
    transpression = smooth_within_province(topology, tectonics, &province_ids, &transpression);

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
    let mut hash = fnv_update(FNV_OFFSET_BASIS, b"interlink-orogen-provinces:v1\0");
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
        minimum_source_width_km: if min_width.is_finite() {
            min_width
        } else {
            0.0
        },
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
        .any(|value| !value.is_finite() || *value < 0.0 || *value > 2500.0)
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
    ) = rasterize(topology, tectonics, pre, &sources, parameters);

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
        let seed = "wg36-orogen-determinism";
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
        assert!(first
            .orogenic_intensity
            .iter()
            .all(|value| value.is_finite()));
    }

    #[test]
    fn orogen_provinces_ignore_legacy_present_day_orogenic_outputs() {
        let seed = "wg36-causal-cut";
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
        assert_eq!(
            baseline.metrics.province_hash,
            changed.metrics.province_hash
        );
    }

    #[test]
    fn inland_resistance_increases_collision_propagation_cost() {
        let low = propagation_penalty(OrogenProvinceKind::ContinentalCollision, 0.10);
        let high = propagation_penalty(OrogenProvinceKind::ContinentalCollision, 0.90);
        let plateau_high = propagation_penalty(OrogenProvinceKind::CollisionalPlateau, 0.90);
        assert!(high > low * 2.0);
        assert!(plateau_high < high);
    }

    #[test]
    fn mountain_cores_remain_boundary_localized() {
        let seed = "wg36-boundary-localized-core";
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
        for sample in 0..model.mountain_core_index.len() {
            if model.mountain_core_index[sample] > 0.20 {
                core_count += 1;
                assert!(
                    model.boundary_distance_km[sample] < 800.0,
                    "mountain core escaped too far inland: {:.1} km",
                    model.boundary_distance_km[sample]
                );
            }
        }
        assert!(core_count > 0);
    }

    #[test]
    fn orogen_provinces_reference_real_connected_segments() {
        let seed = "wg36-province-identity";
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
