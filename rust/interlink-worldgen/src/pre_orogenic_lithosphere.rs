use crate::{
    random, CrustKind, CrustalModel, PlanetTopology, PlateBoundaryKind, StageIdentity,
    TectonicHistoryModel, TectonicModel, WorldgenError,
};
use std::collections::{BTreeMap, VecDeque};
use std::f64::consts::PI;

pub const PRE_OROGENIC_LITHOSPHERE_STAGE_ID: &str = "geology:pre-orogenic-lithosphere";
pub const PRE_OROGENIC_LITHOSPHERE_STAGE_VERSION: u32 = 1;
pub const MAX_PRE_OROGENIC_FRAGMENTS: usize = 256;
const PRE_OROGENIC_NAMESPACE: &str = "worldgen:lithosphere:pre-orogenic:v1";
const PRE_OROGENIC_MECHANICAL_NAMESPACE: &str = "worldgen:lithosphere:pre-orogenic:mechanics:v1";
const PRE_OROGENIC_MANTLE_NAMESPACE: &str = "worldgen:lithosphere:pre-orogenic:mantle:v1";
const PRE_OROGENIC_FRAGMENT_NAMESPACE: &str = "worldgen:lithosphere:pre-orogenic:fragments:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InheritedStructureKind {
    None = 0,
    PaleoSuture = 1,
    InheritedRift = 2,
    ShearZone = 3,
    ContinentalMargin = 4,
    CratonBoundary = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreOrogenicFragmentKind {
    Terrane = 1,
    Microplate = 2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreOrogenicLithosphereRequest {
    pub seed: String,
}
impl PreOrogenicLithosphereRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreOrogenicFragment {
    pub id: u16,
    pub parent_plate_id: u16,
    pub dominant_crust_province_id: u16,
    pub seed_sample: u32,
    pub kind: PreOrogenicFragmentKind,
    pub sample_count: u32,
    pub area_steradians: f64,
    pub area_fraction_of_parent: f64,
    pub mean_intrinsic_weakness: f64,
    pub mean_inherited_fabric: f64,
    pub angular_velocity_rad_per_myr: [f64; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreOrogenicFragmentContact {
    pub domain_a: u16,
    pub domain_b: u16,
    pub edge_count: u32,
    pub boundary_length_rad: f64,
    pub mean_strength_contrast: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreOrogenicLithosphereMetrics {
    pub sample_count: u32,
    pub mean_intrinsic_strength_index: f64,
    pub mean_intrinsic_weakness_index: f64,
    pub mean_effective_elastic_thickness_km: f64,
    pub mean_thermal_state_index: f64,
    pub mean_inherited_fabric_strength: f64,
    pub paleo_suture_sample_count: u32,
    pub inherited_rift_sample_count: u32,
    pub shear_zone_sample_count: u32,
    pub continental_margin_sample_count: u32,
    pub craton_boundary_sample_count: u32,
    pub fragment_count: u16,
    pub microplate_count: u16,
    pub terrane_count: u16,
    pub contact_count: u32,
    pub fragmented_area_fraction: f64,
    pub pre_orogenic_hash: u64,
}
impl PreOrogenicLithosphereMetrics {
    pub fn pre_orogenic_hash_hex(&self) -> String {
        format!("{:016x}", self.pre_orogenic_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreOrogenicLithosphereModel {
    pub stage: StageIdentity,
    pub mechanical_seed: u64,
    pub mantle_seed: u64,
    pub fragment_seed: u64,
    pub intrinsic_strength_index: Vec<f32>,
    pub intrinsic_weakness_index: Vec<f32>,
    pub effective_elastic_thickness_km: Vec<f32>,
    pub thermal_state_index: Vec<f32>,
    pub inherited_fabric_strength: Vec<f32>,
    pub inherited_structure_kind: Vec<u8>,
    pub province_boundary_index: Vec<f32>,
    pub age_discontinuity_index: Vec<f32>,
    pub inherited_rift_memory: Vec<f32>,
    pub inherited_shear_memory: Vec<f32>,
    pub fragmentation_propensity: Vec<f32>,
    pub fragment_ids: Vec<u16>,
    pub kinematic_domain_ids: Vec<u16>,
    pub fragments: Vec<PreOrogenicFragment>,
    pub fragment_contacts: Vec<PreOrogenicFragmentContact>,
    pub metrics: PreOrogenicLithosphereMetrics,
}

#[derive(Clone, Debug)]
struct FragmentComponent {
    samples: Vec<u32>,
    parent_plate_id: u16,
    dominant_crust_province_id: u16,
    area_steradians: f64,
    mean_weakness: f64,
    mean_fabric: f64,
    mean_propensity: f64,
    seed_sample: u32,
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
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
fn hash_f32(mut hash: u64, values: &[f32]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    for value in values {
        hash = fnv_update(hash, &value.to_bits().to_le_bytes());
    }
    hash
}
fn unit_random(value: u64) -> f64 {
    ((random::mix64(value) >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0)
}
fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}
fn clamp_signed(value: f64) -> f64 {
    value.clamp(-1.0, 1.0)
}
fn norm(value: [f64; 3]) -> f64 {
    (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt()
}

fn random_unit_vector(seed: u64, stream: u64) -> [f64; 3] {
    let z = unit_random(seed ^ stream ^ 0xa076_1d64_78bd_642f) * 2.0 - 1.0;
    let angle = unit_random(seed ^ stream ^ 0xe703_7ed1_a0b4_28db) * 2.0 * PI;
    let radial = (1.0 - z * z).max(0.0).sqrt();
    [radial * angle.cos(), radial * angle.sin(), z]
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
            next[sample as usize] = values[sample as usize] * 0.36 + mean * 0.64;
        }
        std::mem::swap(&mut values, &mut next);
    }
    let maximum = values
        .iter()
        .map(|value| value.abs())
        .fold(0.0_f64, f64::max)
        .max(1.0e-12);
    for value in &mut values {
        *value /= maximum;
    }
    values
}

fn diffuse_memory<T: PlanetTopology>(
    topology: &T,
    seeds: &[f64],
    passes: usize,
    retention: f64,
) -> Vec<f64> {
    let mut current = seeds.to_vec();
    let mut next = vec![0.0_f64; current.len()];
    for _ in 0..passes {
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            let neighbors = topology.neighbors(sample);
            let neighbor_mean = neighbors
                .iter()
                .map(|neighbor| current[*neighbor as usize])
                .sum::<f64>()
                / neighbors.len() as f64;
            next[index] = current[index]
                .max(neighbor_mean * retention)
                .clamp(0.0, 1.0);
        }
        std::mem::swap(&mut current, &mut next);
    }
    current
}

fn boundary_memory<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    history: &TectonicHistoryModel,
) -> (Vec<f64>, Vec<f64>) {
    let count = topology.sample_count() as usize;
    let mut rift_seed = vec![0.0_f64; count];
    let mut shear_seed = vec![0.0_f64; count];

    for (boundary, state) in tectonics.boundaries.iter().zip(&history.boundary_state) {
        let age = clamp01(f64::from(state.event_age_myr) / 180.0);
        match boundary.kind {
            PlateBoundaryKind::Divergent => {
                let displacement = clamp01(f64::from(state.cumulative_extension_km) / 1800.0);
                let memory = clamp01(0.20 + age * 0.35 + displacement * 0.70);
                for sample in [boundary.sample_a, boundary.sample_b] {
                    rift_seed[sample as usize] = rift_seed[sample as usize].max(memory);
                }
            }
            PlateBoundaryKind::Transform => {
                let displacement = clamp01(f64::from(state.cumulative_shear_km) / 2200.0);
                let memory = clamp01(0.16 + age * 0.30 + displacement * 0.72);
                for sample in [boundary.sample_a, boundary.sample_b] {
                    shear_seed[sample as usize] = shear_seed[sample as usize].max(memory);
                }
            }
            PlateBoundaryKind::Convergent => {
                // Present convergence is intentionally excluded from the pre-orogenic substrate.
            }
        }
    }

    (
        diffuse_memory(topology, &rift_seed, 5, 0.72),
        diffuse_memory(topology, &shear_seed, 4, 0.70),
    )
}

fn static_crust_indices<T: PlanetTopology>(
    topology: &T,
    geology: &CrustalModel,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let count = topology.sample_count() as usize;
    let mut province_boundary = vec![0.0_f64; count];
    let mut age_discontinuity = vec![0.0_f64; count];
    let mut margin = vec![0.0_f64; count];

    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let kind = geology.crust_kind[index];
        let province = geology.crust_province_id[index];
        let age = f64::from(geology.crust_age_myr[index]);
        let neighbors = topology.neighbors(sample);
        let mut province_mismatch = 0.0;
        let mut kind_mismatch = 0.0;
        let mut max_age_delta = 0.0_f64;

        for neighbor in neighbors {
            let ni = *neighbor as usize;
            if geology.crust_province_id[ni] != province {
                province_mismatch += 1.0;
            }
            if geology.crust_kind[ni] != kind {
                kind_mismatch += 1.0;
            }
            let other_age = f64::from(geology.crust_age_myr[ni]);
            let scale = if kind == CrustKind::Oceanic as u8 {
                180.0
            } else {
                1800.0
            };
            max_age_delta = max_age_delta.max(((age - other_age).abs() / scale).clamp(0.0, 1.0));
        }
        let degree = neighbors.len() as f64;
        province_boundary[index] = (province_mismatch / degree).clamp(0.0, 1.0);
        margin[index] = (kind_mismatch / degree).clamp(0.0, 1.0);
        age_discontinuity[index] = max_age_delta;
    }

    (province_boundary, age_discontinuity, margin)
}

fn build_pre_orogenic_state<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    history: &TectonicHistoryModel,
    geology: &CrustalModel,
    mechanical_seed: u64,
    mantle_seed: u64,
) -> (
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<u8>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
) {
    let count = topology.sample_count() as usize;
    let mechanical_texture = smooth_random_field(topology, mechanical_seed, 7);
    let mantle_texture = smooth_random_field(topology, mantle_seed, 12);
    let (province_boundary, age_discontinuity, margin) = static_crust_indices(topology, geology);
    let (rift_memory, shear_memory) = boundary_memory(topology, tectonics, history);

    let mut strength = Vec::with_capacity(count);
    let mut weakness = Vec::with_capacity(count);
    let mut elastic_thickness = Vec::with_capacity(count);
    let mut thermal = Vec::with_capacity(count);
    let mut fabric = Vec::with_capacity(count);
    let mut structure = Vec::with_capacity(count);
    let mut propensity = Vec::with_capacity(count);

    for sample in 0..count {
        let crust = geology.crust_kind[sample];
        let age_myr = f64::from(geology.crust_age_myr[sample]);
        let inherited = mechanical_texture[sample];
        let mantle = mantle_texture[sample];

        let age_factor = match crust {
            value if value == CrustKind::Continental as u8 => clamp01(age_myr / 3200.0),
            value if value == CrustKind::Transitional as u8 => clamp01(age_myr / 1200.0),
            _ => clamp01(age_myr / 220.0),
        };
        let young_thermal = match crust {
            value if value == CrustKind::Oceanic as u8 => 1.0 - clamp01(age_myr / 220.0),
            value if value == CrustKind::Transitional as u8 => 0.22 * (1.0 - age_factor),
            _ => 0.08 * (1.0 - age_factor),
        };
        let thermal_state = clamp_signed(
            mantle * 0.62 + young_thermal * 0.42 + rift_memory[sample] * 0.18 - age_factor * 0.16,
        );

        let suture = if crust != CrustKind::Oceanic as u8 {
            clamp01(
                province_boundary[sample] * 0.72 + age_discontinuity[sample] * 0.46
                    - margin[sample] * 0.22,
            )
        } else {
            0.0
        };
        let craton_boundary = if crust == CrustKind::Continental as u8 {
            clamp01(province_boundary[sample] * 0.70 + age_discontinuity[sample] * 0.24)
        } else {
            0.0
        };
        let rift = rift_memory[sample]
            * if crust == CrustKind::Oceanic as u8 {
                0.45
            } else {
                1.0
            };
        let shear = shear_memory[sample];
        let continental_margin = margin[sample]
            * if crust == CrustKind::Oceanic as u8 {
                0.70
            } else {
                1.0
            };
        let inherited_fabric = suture
            .max(craton_boundary)
            .max(rift)
            .max(shear)
            .max(continental_margin)
            .clamp(0.0, 1.0);

        let structure_kind = if inherited_fabric < 0.20 {
            InheritedStructureKind::None
        } else if suture >= craton_boundary
            && suture >= rift
            && suture >= shear
            && suture >= continental_margin
        {
            InheritedStructureKind::PaleoSuture
        } else if rift >= shear && rift >= continental_margin && rift >= craton_boundary {
            InheritedStructureKind::InheritedRift
        } else if shear >= continental_margin && shear >= craton_boundary {
            InheritedStructureKind::ShearZone
        } else if continental_margin >= craton_boundary {
            InheritedStructureKind::ContinentalMargin
        } else {
            InheritedStructureKind::CratonBoundary
        };

        let base_strength = match crust {
            value if value == CrustKind::Continental as u8 => 0.61,
            value if value == CrustKind::Transitional as u8 => 0.43,
            _ => 0.48,
        };
        let interior_coherence =
            clamp01(1.0 - province_boundary[sample] * 0.85 - age_discontinuity[sample] * 0.35);
        let damage = clamp01(
            suture * 0.28
                + rift * 0.42
                + shear * 0.34
                + continental_margin * 0.18
                + craton_boundary * 0.12,
        );
        let intrinsic_strength = clamp01(
            base_strength + age_factor * 0.25 + interior_coherence * 0.12 + inherited * 0.065
                - thermal_state.max(0.0) * 0.24
                - damage * 0.38,
        );
        let intrinsic_weakness = clamp01(
            1.0 - intrinsic_strength + inherited_fabric * 0.22 + thermal_state.max(0.0) * 0.10,
        );
        let te_km = (4.0 + intrinsic_strength * 80.0 + age_factor * 8.0
            - thermal_state.max(0.0) * 12.0
            - inherited_fabric * 6.0)
            .clamp(4.0, 92.0);
        let fragmentation = clamp01(
            intrinsic_weakness * 0.44
                + inherited_fabric * 0.30
                + province_boundary[sample] * 0.13
                + age_discontinuity[sample] * 0.08
                + rift.max(shear) * 0.18,
        );

        strength.push(intrinsic_strength as f32);
        weakness.push(intrinsic_weakness as f32);
        elastic_thickness.push(te_km as f32);
        thermal.push(thermal_state as f32);
        fabric.push(inherited_fabric as f32);
        structure.push(structure_kind as u8);
        propensity.push(fragmentation as f32);
    }

    (
        strength,
        weakness,
        elastic_thickness,
        thermal,
        fabric,
        structure,
        province_boundary
            .into_iter()
            .map(|value| value as f32)
            .collect(),
        age_discontinuity
            .into_iter()
            .map(|value| value as f32)
            .collect(),
        rift_memory.into_iter().map(|value| value as f32).collect(),
        shear_memory.into_iter().map(|value| value as f32).collect(),
        propensity,
    )
}

fn dominant_province(samples: &[u32], geology: &CrustalModel) -> u16 {
    let mut counts = BTreeMap::<u16, u32>::new();
    for sample in samples {
        *counts
            .entry(geology.crust_province_id[*sample as usize])
            .or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|(left_id, left_count), (right_id, right_count)| {
            left_count
                .cmp(right_count)
                .then_with(|| right_id.cmp(left_id))
        })
        .map(|(province, _)| province)
        .unwrap_or(0)
}

fn collect_fragment_components<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    geology: &CrustalModel,
    weakness: &[f32],
    fabric: &[f32],
    propensity: &[f32],
    province_boundary: &[f32],
) -> Vec<FragmentComponent> {
    let count = topology.sample_count() as usize;
    let eligible = (0..count)
        .map(|sample| {
            let crust = geology.crust_kind[sample];
            let threshold = if crust == CrustKind::Oceanic as u8 {
                0.66
            } else {
                0.56
            };
            f64::from(propensity[sample]) >= threshold
                && f64::from(weakness[sample]) >= 0.42
                && (f64::from(fabric[sample]) >= 0.24
                    || f64::from(province_boundary[sample]) >= 0.34)
        })
        .collect::<Vec<_>>();
    let mut visited = vec![false; count];
    let mut components = Vec::new();

    for start in 0..count as u32 {
        let start_index = start as usize;
        if visited[start_index] || !eligible[start_index] {
            continue;
        }
        let parent_plate_id = tectonics.plate_ids[start_index];
        let mut queue = VecDeque::new();
        let mut samples = Vec::new();
        visited[start_index] = true;
        queue.push_back(start);

        while let Some(sample) = queue.pop_front() {
            samples.push(sample);
            for neighbor in topology.neighbors(sample) {
                let index = *neighbor as usize;
                if !visited[index]
                    && eligible[index]
                    && tectonics.plate_ids[index] == parent_plate_id
                {
                    visited[index] = true;
                    queue.push_back(*neighbor);
                }
            }
        }
        if samples.len() < 3 {
            continue;
        }

        let area_steradians = samples
            .iter()
            .map(|sample| topology.area_steradians(*sample))
            .sum::<f64>();
        let mean_weakness = samples
            .iter()
            .map(|sample| f64::from(weakness[*sample as usize]))
            .sum::<f64>()
            / samples.len() as f64;
        let mean_fabric = samples
            .iter()
            .map(|sample| f64::from(fabric[*sample as usize]))
            .sum::<f64>()
            / samples.len() as f64;
        let mean_propensity = samples
            .iter()
            .map(|sample| f64::from(propensity[*sample as usize]))
            .sum::<f64>()
            / samples.len() as f64;
        let seed_sample = *samples
            .iter()
            .max_by(|left, right| {
                propensity[**left as usize]
                    .total_cmp(&propensity[**right as usize])
                    .then_with(|| right.cmp(left))
            })
            .expect("fragment component is non-empty");

        components.push(FragmentComponent {
            dominant_crust_province_id: dominant_province(&samples, geology),
            samples,
            parent_plate_id,
            area_steradians,
            mean_weakness,
            mean_fabric,
            mean_propensity,
            seed_sample,
        });
    }

    components.sort_by(|left, right| {
        right
            .mean_propensity
            .total_cmp(&left.mean_propensity)
            .then_with(|| right.mean_fabric.total_cmp(&left.mean_fabric))
            .then_with(|| right.area_steradians.total_cmp(&left.area_steradians))
            .then_with(|| left.seed_sample.cmp(&right.seed_sample))
    });
    components
}

fn build_fragments<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    geology: &CrustalModel,
    weakness: &[f32],
    fabric: &[f32],
    propensity: &[f32],
    province_boundary: &[f32],
    fragment_seed: u64,
) -> (Vec<u16>, Vec<u16>, Vec<PreOrogenicFragment>) {
    let mut fragment_ids = vec![0_u16; topology.sample_count() as usize];
    let mut domain_ids = tectonics.plate_ids.clone();
    let mut fragments = Vec::new();
    let mut accepted_area = 0.0_f64;
    let maximum_fragmented_area = 4.0 * PI * 0.32;

    let components = collect_fragment_components(
        topology,
        tectonics,
        geology,
        weakness,
        fabric,
        propensity,
        province_boundary,
    );

    for component in components {
        if fragments.len() >= MAX_PRE_OROGENIC_FRAGMENTS {
            break;
        }
        let parent = &tectonics.plates[component.parent_plate_id as usize];
        let parent_fraction = component.area_steradians / parent.area_steradians.max(1.0e-12);
        if parent_fraction > 0.30
            || accepted_area + component.area_steradians > maximum_fragmented_area
        {
            continue;
        }

        let kind = if component.samples.len() >= 4
            && parent_fraction >= 0.004
            && parent_fraction <= 0.20
            && component.mean_propensity >= 0.62
            && component.mean_fabric >= 0.30
        {
            PreOrogenicFragmentKind::Microplate
        } else {
            PreOrogenicFragmentKind::Terrane
        };

        let fragment_id = (fragments.len() + 1) as u16;
        let domain_id = tectonics.plates.len() as u16 + fragment_id - 1;
        for sample in &component.samples {
            fragment_ids[*sample as usize] = fragment_id;
            domain_ids[*sample as usize] = domain_id;
        }

        let angular_velocity = if kind == PreOrogenicFragmentKind::Microplate {
            let parent_velocity = parent.angular_velocity_rad_per_myr;
            let parent_speed = norm(parent_velocity);
            let perturb_direction = random_unit_vector(
                fragment_seed,
                u64::from(component.seed_sample)
                    ^ u64::from(fragment_id).wrapping_mul(0x9e37_79b9_7f4a_7c15),
            );
            let perturb_scale = parent_speed
                * (0.04 + component.mean_weakness * 0.14 + component.mean_fabric * 0.05);
            [
                parent_velocity[0] + perturb_direction[0] * perturb_scale,
                parent_velocity[1] + perturb_direction[1] * perturb_scale,
                parent_velocity[2] + perturb_direction[2] * perturb_scale,
            ]
        } else {
            parent.angular_velocity_rad_per_myr
        };

        fragments.push(PreOrogenicFragment {
            id: fragment_id,
            parent_plate_id: component.parent_plate_id,
            dominant_crust_province_id: component.dominant_crust_province_id,
            seed_sample: component.seed_sample,
            kind,
            sample_count: component.samples.len() as u32,
            area_steradians: component.area_steradians,
            area_fraction_of_parent: parent_fraction,
            mean_intrinsic_weakness: component.mean_weakness,
            mean_inherited_fabric: component.mean_fabric,
            angular_velocity_rad_per_myr: angular_velocity,
        });
        accepted_area += component.area_steradians;
    }

    (fragment_ids, domain_ids, fragments)
}

#[derive(Clone, Copy, Debug, Default)]
struct ContactAccumulator {
    edge_count: u32,
    boundary_length_rad: f64,
    strength_contrast_sum: f64,
}

fn build_fragment_contacts<T: PlanetTopology>(
    topology: &T,
    domain_ids: &[u16],
    strength: &[f32],
    macro_domain_count: u16,
) -> Vec<PreOrogenicFragmentContact> {
    let mut contacts = BTreeMap::<(u16, u16), ContactAccumulator>::new();

    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let neighbors = topology.neighbors(sample);
        let lengths = topology.neighbor_arc_lengths_rad(sample);
        for neighbor_index in 0..neighbors.len() {
            let neighbor = neighbors[neighbor_index];
            if neighbor <= sample {
                continue;
            }
            let ni = neighbor as usize;
            let domain_a = domain_ids[index];
            let domain_b = domain_ids[ni];
            if domain_a == domain_b
                || (domain_a < macro_domain_count && domain_b < macro_domain_count)
            {
                continue;
            }
            let key = if domain_a < domain_b {
                (domain_a, domain_b)
            } else {
                (domain_b, domain_a)
            };
            let accumulator = contacts.entry(key).or_default();
            accumulator.edge_count += 1;
            accumulator.boundary_length_rad += lengths[neighbor_index];
            accumulator.strength_contrast_sum +=
                (f64::from(strength[index]) - f64::from(strength[ni])).abs();
        }
    }

    contacts
        .into_iter()
        .map(|((domain_a, domain_b), value)| PreOrogenicFragmentContact {
            domain_a,
            domain_b,
            edge_count: value.edge_count,
            boundary_length_rad: value.boundary_length_rad,
            mean_strength_contrast: if value.edge_count > 0 {
                (value.strength_contrast_sum / f64::from(value.edge_count)) as f32
            } else {
                0.0
            },
        })
        .collect()
}

fn weighted_mean<T: PlanetTopology>(topology: &T, values: &[f32]) -> f64 {
    let mut sum = 0.0;
    let mut area = 0.0;
    for sample in 0..topology.sample_count() {
        let weight = topology.area_steradians(sample);
        sum += f64::from(values[sample as usize]) * weight;
        area += weight;
    }
    sum / area.max(1.0e-12)
}

fn model_hash(model: &PreOrogenicLithosphereModel) -> u64 {
    let mut hash = fnv_update(FNV_OFFSET_BASIS, b"interlink-pre-orogenic-lithosphere:v1\0");
    hash = fnv_update(hash, &model.stage.derived_seed.to_le_bytes());
    hash = hash_f32(hash, &model.intrinsic_strength_index);
    hash = hash_f32(hash, &model.intrinsic_weakness_index);
    hash = hash_f32(hash, &model.effective_elastic_thickness_km);
    hash = hash_f32(hash, &model.thermal_state_index);
    hash = hash_f32(hash, &model.inherited_fabric_strength);
    hash = hash_u8(hash, &model.inherited_structure_kind);
    hash = hash_f32(hash, &model.province_boundary_index);
    hash = hash_f32(hash, &model.age_discontinuity_index);
    hash = hash_f32(hash, &model.inherited_rift_memory);
    hash = hash_f32(hash, &model.inherited_shear_memory);
    hash = hash_f32(hash, &model.fragmentation_propensity);
    hash = hash_u16(hash, &model.fragment_ids);
    hash = hash_u16(hash, &model.kinematic_domain_ids);

    for fragment in &model.fragments {
        hash = fnv_update(hash, &fragment.id.to_le_bytes());
        hash = fnv_update(hash, &fragment.parent_plate_id.to_le_bytes());
        hash = fnv_update(hash, &fragment.dominant_crust_province_id.to_le_bytes());
        hash = fnv_update(hash, &fragment.seed_sample.to_le_bytes());
        hash = fnv_update(hash, &[fragment.kind as u8]);
        hash = fnv_update(hash, &fragment.area_steradians.to_bits().to_le_bytes());
        for value in fragment.angular_velocity_rad_per_myr {
            hash = fnv_update(hash, &value.to_bits().to_le_bytes());
        }
    }
    for contact in &model.fragment_contacts {
        hash = fnv_update(hash, &contact.domain_a.to_le_bytes());
        hash = fnv_update(hash, &contact.domain_b.to_le_bytes());
        hash = fnv_update(hash, &contact.edge_count.to_le_bytes());
        hash = fnv_update(hash, &contact.boundary_length_rad.to_bits().to_le_bytes());
        hash = fnv_update(
            hash,
            &contact.mean_strength_contrast.to_bits().to_le_bytes(),
        );
    }
    hash
}

fn validate_model<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    history: &TectonicHistoryModel,
    geology: &CrustalModel,
    model: &PreOrogenicLithosphereModel,
) -> Result<(), WorldgenError> {
    let count = topology.sample_count() as usize;
    if tectonics.plate_ids.len() != count
        || geology.crust_kind.len() != count
        || geology.crust_province_id.len() != count
        || geology.crust_age_myr.len() != count
        || history.boundary_state.len() != tectonics.boundaries.len()
    {
        return Err(WorldgenError::InvalidLithosphere(
            "pre-orogenic upstream state does not match topology/boundary dimensions",
        ));
    }

    let lengths = [
        model.intrinsic_strength_index.len(),
        model.intrinsic_weakness_index.len(),
        model.effective_elastic_thickness_km.len(),
        model.thermal_state_index.len(),
        model.inherited_fabric_strength.len(),
        model.inherited_structure_kind.len(),
        model.province_boundary_index.len(),
        model.age_discontinuity_index.len(),
        model.inherited_rift_memory.len(),
        model.inherited_shear_memory.len(),
        model.fragmentation_propensity.len(),
        model.fragment_ids.len(),
        model.kinematic_domain_ids.len(),
    ];
    if lengths.iter().any(|length| *length != count) {
        return Err(WorldgenError::InvalidLithosphere(
            "pre-orogenic lithosphere fields do not match topology sample count",
        ));
    }

    for sample in 0..count {
        let unit_fields = [
            model.intrinsic_strength_index[sample],
            model.intrinsic_weakness_index[sample],
            model.inherited_fabric_strength[sample],
            model.province_boundary_index[sample],
            model.age_discontinuity_index[sample],
            model.inherited_rift_memory[sample],
            model.inherited_shear_memory[sample],
            model.fragmentation_propensity[sample],
        ];
        if unit_fields
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0 || *value > 1.0)
        {
            return Err(WorldgenError::InvalidLithosphere(
                "pre-orogenic normalized field is outside [0,1]",
            ));
        }
        let thermal = model.thermal_state_index[sample];
        if !thermal.is_finite() || !(-1.0..=1.0).contains(&thermal) {
            return Err(WorldgenError::InvalidLithosphere(
                "pre-orogenic thermal state is outside [-1,1]",
            ));
        }
        let te = model.effective_elastic_thickness_km[sample];
        if !te.is_finite() || !(4.0..=92.0).contains(&te) {
            return Err(WorldgenError::InvalidLithosphere(
                "pre-orogenic elastic thickness is outside supported bounds",
            ));
        }
        let fragment_id = model.fragment_ids[sample];
        if fragment_id > 0 {
            let fragment = model.fragments.get(fragment_id as usize - 1).ok_or(
                WorldgenError::InvalidLithosphere(
                    "pre-orogenic fragment field references missing fragment",
                ),
            )?;
            if tectonics.plate_ids[sample] != fragment.parent_plate_id {
                return Err(WorldgenError::InvalidLithosphere(
                    "pre-orogenic fragment crosses a macro-plate boundary",
                ));
            }
        }
    }

    if model.fragments.len() > MAX_PRE_OROGENIC_FRAGMENTS {
        return Err(WorldgenError::InvalidLithosphere(
            "too many pre-orogenic tectonic fragments",
        ));
    }
    for contact in &model.fragment_contacts {
        if contact.domain_a >= contact.domain_b
            || contact.edge_count == 0
            || !contact.boundary_length_rad.is_finite()
            || contact.boundary_length_rad <= 0.0
            || !contact.mean_strength_contrast.is_finite()
            || !(0.0..=1.0).contains(&contact.mean_strength_contrast)
        {
            return Err(WorldgenError::InvalidLithosphere(
                "invalid pre-orogenic fragment contact",
            ));
        }
    }
    Ok(())
}

pub fn generate_pre_orogenic_lithosphere<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    history: &TectonicHistoryModel,
    geology: &CrustalModel,
    request: &PreOrogenicLithosphereRequest,
) -> Result<PreOrogenicLithosphereModel, WorldgenError> {
    if request.seed.trim().is_empty() {
        return Err(WorldgenError::InvalidLithosphere(
            "pre-orogenic lithosphere seed must not be empty",
        ));
    }
    if tectonics.plate_ids.len() != topology.sample_count() as usize
        || geology.crust_kind.len() != topology.sample_count() as usize
        || history.boundary_state.len() != tectonics.boundaries.len()
    {
        return Err(WorldgenError::InvalidLithosphere(
            "pre-orogenic inputs do not align with topology",
        ));
    }

    let stage_seed = random::derive_stage_seed(&request.seed, PRE_OROGENIC_NAMESPACE);
    let mechanical_seed =
        random::derive_stage_seed(&request.seed, PRE_OROGENIC_MECHANICAL_NAMESPACE);
    let mantle_seed = random::derive_stage_seed(&request.seed, PRE_OROGENIC_MANTLE_NAMESPACE);
    let fragment_seed = random::derive_stage_seed(&request.seed, PRE_OROGENIC_FRAGMENT_NAMESPACE);

    let (
        intrinsic_strength_index,
        intrinsic_weakness_index,
        effective_elastic_thickness_km,
        thermal_state_index,
        inherited_fabric_strength,
        inherited_structure_kind,
        province_boundary_index,
        age_discontinuity_index,
        inherited_rift_memory,
        inherited_shear_memory,
        fragmentation_propensity,
    ) = build_pre_orogenic_state(
        topology,
        tectonics,
        history,
        geology,
        mechanical_seed,
        mantle_seed,
    );

    let (fragment_ids, kinematic_domain_ids, fragments) = build_fragments(
        topology,
        tectonics,
        geology,
        &intrinsic_weakness_index,
        &inherited_fabric_strength,
        &fragmentation_propensity,
        &province_boundary_index,
        fragment_seed,
    );
    let fragment_contacts = build_fragment_contacts(
        topology,
        &kinematic_domain_ids,
        &intrinsic_strength_index,
        tectonics.plates.len() as u16,
    );

    let total_area = (0..topology.sample_count())
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>();
    let fragmented_area = (0..topology.sample_count())
        .filter(|sample| fragment_ids[*sample as usize] > 0)
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>();
    let microplate_count = fragments
        .iter()
        .filter(|fragment| fragment.kind == PreOrogenicFragmentKind::Microplate)
        .count() as u16;
    let terrane_count = fragments.len() as u16 - microplate_count;

    let mut metrics = PreOrogenicLithosphereMetrics {
        sample_count: topology.sample_count(),
        mean_intrinsic_strength_index: weighted_mean(topology, &intrinsic_strength_index),
        mean_intrinsic_weakness_index: weighted_mean(topology, &intrinsic_weakness_index),
        mean_effective_elastic_thickness_km: weighted_mean(
            topology,
            &effective_elastic_thickness_km,
        ),
        mean_thermal_state_index: weighted_mean(topology, &thermal_state_index),
        mean_inherited_fabric_strength: weighted_mean(topology, &inherited_fabric_strength),
        paleo_suture_sample_count: inherited_structure_kind
            .iter()
            .filter(|kind| **kind == InheritedStructureKind::PaleoSuture as u8)
            .count() as u32,
        inherited_rift_sample_count: inherited_structure_kind
            .iter()
            .filter(|kind| **kind == InheritedStructureKind::InheritedRift as u8)
            .count() as u32,
        shear_zone_sample_count: inherited_structure_kind
            .iter()
            .filter(|kind| **kind == InheritedStructureKind::ShearZone as u8)
            .count() as u32,
        continental_margin_sample_count: inherited_structure_kind
            .iter()
            .filter(|kind| **kind == InheritedStructureKind::ContinentalMargin as u8)
            .count() as u32,
        craton_boundary_sample_count: inherited_structure_kind
            .iter()
            .filter(|kind| **kind == InheritedStructureKind::CratonBoundary as u8)
            .count() as u32,
        fragment_count: fragments.len() as u16,
        microplate_count,
        terrane_count,
        contact_count: fragment_contacts.len() as u32,
        fragmented_area_fraction: fragmented_area / total_area.max(1.0e-12),
        pre_orogenic_hash: 0,
    };

    let mut model = PreOrogenicLithosphereModel {
        stage: StageIdentity {
            id: PRE_OROGENIC_LITHOSPHERE_STAGE_ID,
            version: PRE_OROGENIC_LITHOSPHERE_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        mechanical_seed,
        mantle_seed,
        fragment_seed,
        intrinsic_strength_index,
        intrinsic_weakness_index,
        effective_elastic_thickness_km,
        thermal_state_index,
        inherited_fabric_strength,
        inherited_structure_kind,
        province_boundary_index,
        age_discontinuity_index,
        inherited_rift_memory,
        inherited_shear_memory,
        fragmentation_propensity,
        fragment_ids,
        kinematic_domain_ids,
        fragments,
        fragment_contacts,
        metrics: metrics.clone(),
    };
    metrics.pre_orogenic_hash = model_hash(&model);
    model.metrics = metrics;
    validate_model(topology, tectonics, history, geology, &model)?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_icosphere, generate_crust_and_history, generate_tectonic_history, generate_tectonics,
        GeologyRequest, PlanetPhysicalParameters, TectonicHistoryRequest, TectonicsRequest,
    };

    fn generate(
        seed: &str,
    ) -> (
        crate::GeodesicTopology,
        TectonicModel,
        TectonicHistoryModel,
        CrustalModel,
        PreOrogenicLithosphereModel,
    ) {
        let topology = build_icosphere(4).unwrap();
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let tectonics =
            generate_tectonics(&topology, &TectonicsRequest::new(seed, 16), parameters).unwrap();
        let history = generate_tectonic_history(
            &topology,
            &tectonics,
            &TectonicHistoryRequest::new(seed),
            parameters,
        )
        .unwrap();
        let geology = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new(seed),
            parameters,
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
        (topology, tectonics, history, geology, pre)
    }

    #[test]
    fn pre_orogenic_state_is_deterministic_and_complete() {
        let (topology, _, _, _, first) = generate("wg35-pre-determinism");
        let (_, _, _, _, second) = generate("wg35-pre-determinism");
        let (_, _, _, _, changed) = generate("wg35-pre-determinism-b");
        assert_eq!(
            first.metrics.pre_orogenic_hash,
            second.metrics.pre_orogenic_hash
        );
        assert_ne!(
            first.metrics.pre_orogenic_hash,
            changed.metrics.pre_orogenic_hash
        );
        assert_eq!(
            first.intrinsic_strength_index.len(),
            topology.sample_count() as usize
        );
    }

    #[test]
    fn present_orogenic_response_cannot_feed_back_into_pre_orogenic_state() {
        let (topology, tectonics, history, geology, first) = generate("wg35-pre-causal-cut");
        let mut mutated = geology.clone();
        for value in &mut mutated.orogenic_history {
            *value = 1.0 - *value;
        }
        for value in &mut mutated.rift_history {
            *value = 1.0 - *value;
        }
        for value in &mut mutated.subduction_history {
            *value = 1.0 - *value;
        }
        for value in &mut mutated.transform_history {
            *value = 1.0 - *value;
        }
        for value in &mut mutated.crustal_strain {
            *value = 1.0 - *value;
        }
        for value in &mut mutated.crust_thickness_km {
            *value = (*value + 11.0).clamp(4.0, 60.0);
        }
        for value in &mut mutated.crust_density_kg_per_m3 {
            *value = (*value + 90.0).clamp(2500.0, 3200.0);
        }
        for value in &mut mutated.buoyancy_index {
            *value = -*value;
        }

        let regenerated = generate_pre_orogenic_lithosphere(
            &topology,
            &tectonics,
            &history,
            &mutated,
            &PreOrogenicLithosphereRequest::new("wg35-pre-causal-cut"),
        )
        .unwrap();

        assert_eq!(
            first.metrics.pre_orogenic_hash,
            regenerated.metrics.pre_orogenic_hash
        );
        assert_eq!(
            first.intrinsic_strength_index,
            regenerated.intrinsic_strength_index
        );
    }

    #[test]
    fn convergent_history_is_not_used_as_pre_orogenic_damage() {
        let (topology, tectonics, history, geology, first) = generate("wg35-pre-no-convergence");
        let mut mutated_history = history.clone();
        for (boundary, state) in tectonics
            .boundaries
            .iter()
            .zip(&mut mutated_history.boundary_state)
        {
            if boundary.kind == PlateBoundaryKind::Convergent {
                state.event_age_myr = 250.0;
                state.cumulative_convergence_km = 20_000.0;
            }
        }

        let regenerated = generate_pre_orogenic_lithosphere(
            &topology,
            &tectonics,
            &mutated_history,
            &geology,
            &PreOrogenicLithosphereRequest::new("wg35-pre-no-convergence"),
        )
        .unwrap();

        assert_eq!(
            first.metrics.pre_orogenic_hash,
            regenerated.metrics.pre_orogenic_hash
        );
    }

    #[test]
    fn fragments_are_connected_to_parent_plate_domains_and_contacts_are_valid() {
        let (topology, tectonics, _, _, model) = generate("wg35-pre-fragments");
        for sample in 0..topology.sample_count() as usize {
            let fragment_id = model.fragment_ids[sample];
            if fragment_id == 0 {
                continue;
            }
            let fragment = &model.fragments[fragment_id as usize - 1];
            assert_eq!(tectonics.plate_ids[sample], fragment.parent_plate_id);
            assert!(model.kinematic_domain_ids[sample] >= tectonics.plates.len() as u16);
        }
        for contact in &model.fragment_contacts {
            assert!(contact.domain_a < contact.domain_b);
            assert!(contact.edge_count > 0);
            assert!(contact.boundary_length_rad > 0.0);
        }
    }
}
