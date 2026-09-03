use crate::{
    random, CrustKind, CrustalModel, PlanetTopology, StageIdentity, TectonicModel, WorldgenError,
};
use std::collections::VecDeque;
use std::f64::consts::PI;

pub const LITHOSPHERE_STAGE_ID: &str = "geology:lithosphere-refinement";
pub const LITHOSPHERE_STAGE_VERSION: u32 = 1;
pub const MAX_TECTONIC_FRAGMENTS: usize = 192;
const LITHOSPHERE_NAMESPACE: &str = "worldgen:lithosphere:state:v1";
const MECHANICAL_NAMESPACE: &str = "worldgen:lithosphere:mechanics:v1";
const MANTLE_NAMESPACE: &str = "worldgen:lithosphere:mantle-support:v1";
const REFINEMENT_NAMESPACE: &str = "worldgen:lithosphere:tectonic-refinement:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructuralZoneKind {
    None = 0,
    Suture = 1,
    Rift = 2,
    Transform = 3,
    ContinentalMargin = 4,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TectonicFragmentKind {
    Terrane = 1,
    Microplate = 2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LithosphereRequest {
    pub seed: String,
}
impl LithosphereRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TectonicFragment {
    pub id: u16,
    pub parent_plate_id: u16,
    pub seed_sample: u32,
    pub kind: TectonicFragmentKind,
    pub sample_count: u32,
    pub area_steradians: f64,
    pub area_fraction_of_parent: f64,
    pub mean_weakness: f64,
    pub mean_fragmentation_propensity: f64,
    pub angular_velocity_rad_per_myr: [f64; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct LithosphereMetrics {
    pub sample_count: u32,
    pub mean_strength_index: f64,
    pub mean_weakness_index: f64,
    pub mean_effective_elastic_thickness_km: f64,
    pub mean_mantle_upwelling_index: f64,
    pub mean_dynamic_support_index: f64,
    pub suture_sample_count: u32,
    pub rift_zone_sample_count: u32,
    pub transform_zone_sample_count: u32,
    pub continental_margin_sample_count: u32,
    pub tectonic_fragment_count: u16,
    pub microplate_count: u16,
    pub terrane_count: u16,
    pub fragmented_area_fraction: f64,
    pub lithosphere_hash: u64,
}
impl LithosphereMetrics {
    pub fn lithosphere_hash_hex(&self) -> String {
        format!("{:016x}", self.lithosphere_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LithosphericModel {
    pub stage: StageIdentity,
    pub mechanical_seed: u64,
    pub mantle_seed: u64,
    pub refinement_seed: u64,
    pub strength_index: Vec<f32>,
    pub weakness_index: Vec<f32>,
    pub effective_elastic_thickness_km: Vec<f32>,
    pub thermal_anomaly_index: Vec<f32>,
    pub mantle_upwelling_index: Vec<f32>,
    pub mantle_dynamic_support_index: Vec<f32>,
    pub compensated_buoyancy_index: Vec<f32>,
    pub structural_fabric_strength: Vec<f32>,
    pub structural_zone_kind: Vec<u8>,
    pub fragmentation_propensity: Vec<f32>,
    pub fragment_ids: Vec<u16>,
    pub kinematic_domain_ids: Vec<u16>,
    pub fragments: Vec<TectonicFragment>,
    pub metrics: LithosphereMetrics,
}

#[derive(Clone, Debug)]
struct FragmentComponent {
    samples: Vec<u32>,
    parent_plate_id: u16,
    area_steradians: f64,
    mean_weakness: f64,
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
fn normalize(value: [f64; 3]) -> [f64; 3] {
    let magnitude = norm(value).max(1.0e-15);
    [
        value[0] / magnitude,
        value[1] / magnitude,
        value[2] / magnitude,
    ]
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
            next[sample as usize] = values[sample as usize] * 0.40 + mean * 0.60;
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

fn build_mechanical_state<T: PlanetTopology>(
    topology: &T,
    geology: &CrustalModel,
    mechanical_seed: u64,
    mantle_seed: u64,
) -> (
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<u8>,
    Vec<f32>,
) {
    let mechanical_texture = smooth_random_field(topology, mechanical_seed, 6);
    let mantle_texture = smooth_random_field(topology, mantle_seed, 11);
    let count = topology.sample_count() as usize;
    let mut strength = Vec::with_capacity(count);
    let mut weakness = Vec::with_capacity(count);
    let mut elastic_thickness = Vec::with_capacity(count);
    let mut thermal = Vec::with_capacity(count);
    let mut upwelling = Vec::with_capacity(count);
    let mut dynamic_support = Vec::with_capacity(count);
    let mut compensated_buoyancy = Vec::with_capacity(count);
    let mut fabric_strength = Vec::with_capacity(count);
    let mut structural_kind = Vec::with_capacity(count);
    let mut fragmentation_propensity = Vec::with_capacity(count);

    for sample in 0..count {
        let crust = geology.crust_kind[sample];
        let age_myr = f64::from(geology.crust_age_myr[sample]);
        let thickness_km = f64::from(geology.crust_thickness_km[sample]);
        let orogeny = f64::from(geology.orogenic_history[sample]);
        let rift = f64::from(geology.rift_history[sample]);
        let ridge = f64::from(geology.ridge_history[sample]);
        let subduction = f64::from(geology.subduction_history[sample]);
        let arc = f64::from(geology.volcanic_arc_history[sample]);
        let transform = f64::from(geology.transform_history[sample]);
        let strain = f64::from(geology.crustal_strain[sample]);
        let inherited = mechanical_texture[sample];
        let mantle = mantle_texture[sample];

        let age_factor = match crust {
            value if value == CrustKind::Continental as u8 => clamp01(age_myr / 3000.0),
            value if value == CrustKind::Transitional as u8 => clamp01(age_myr / 1800.0),
            _ => clamp01(age_myr / 180.0),
        };
        let base_strength = match crust {
            value if value == CrustKind::Continental as u8 => 0.58,
            value if value == CrustKind::Transitional as u8 => 0.44,
            _ => 0.50,
        };
        let thermal_index = clamp_signed(
            mantle * 0.58 + ridge * 0.72 + rift * 0.38 + arc * 0.18 - age_factor * 0.18,
        );
        let deformation_penalty =
            strain * 0.34 + rift * 0.25 + transform * 0.20 + subduction * 0.12;
        let thickness_factor = clamp01((thickness_km - 5.0) / 45.0);
        let strength_index = clamp01(
            base_strength + age_factor * 0.24 + thickness_factor * 0.10 + inherited * 0.055
                - thermal_index.max(0.0) * 0.24
                - deformation_penalty,
        );
        let weakness_index =
            clamp01(1.0 - strength_index + strain * 0.16 + rift * 0.12 + transform * 0.08);
        let te_km = (5.0 + strength_index * 73.0 + age_factor * 8.0 - thermal_index.max(0.0) * 9.0)
            .clamp(4.0, 86.0);
        let upwelling_index =
            clamp01(0.16 + mantle.max(0.0) * 0.58 + ridge * 0.52 + rift * 0.22 - subduction * 0.12);
        let support_index =
            clamp_signed(mantle * 0.52 + upwelling_index * 0.34 + ridge * 0.22 - subduction * 0.18);
        let column_buoyancy =
            clamp_signed(f64::from(geology.buoyancy_index[sample]) * 0.82 + support_index * 0.18);

        let margin = if crust == CrustKind::Transitional as u8 {
            0.58
        } else {
            0.0
        };
        let suture = if crust != CrustKind::Oceanic as u8 {
            orogeny * (0.55 + strain * 0.45)
        } else {
            0.0
        };
        let rift_zone = rift * (0.62 + strain * 0.38);
        let transform_zone = transform * (0.62 + strain * 0.38);
        let maximum_fabric = suture.max(rift_zone).max(transform_zone).max(margin);
        let zone = if maximum_fabric < 0.24 {
            StructuralZoneKind::None
        } else if suture >= rift_zone && suture >= transform_zone && suture >= margin {
            StructuralZoneKind::Suture
        } else if rift_zone >= transform_zone && rift_zone >= margin {
            StructuralZoneKind::Rift
        } else if transform_zone >= margin {
            StructuralZoneKind::Transform
        } else {
            StructuralZoneKind::ContinentalMargin
        };
        let propensity = clamp01(
            weakness_index * 0.42
                + strain * 0.26
                + maximum_fabric * 0.20
                + subduction.max(rift).max(transform) * 0.12,
        );

        strength.push(strength_index as f32);
        weakness.push(weakness_index as f32);
        elastic_thickness.push(te_km as f32);
        thermal.push(thermal_index as f32);
        upwelling.push(upwelling_index as f32);
        dynamic_support.push(support_index as f32);
        compensated_buoyancy.push(column_buoyancy as f32);
        fabric_strength.push(maximum_fabric as f32);
        structural_kind.push(zone as u8);
        fragmentation_propensity.push(propensity as f32);
    }

    (
        strength,
        weakness,
        elastic_thickness,
        thermal,
        upwelling,
        dynamic_support,
        compensated_buoyancy,
        fabric_strength,
        structural_kind,
        fragmentation_propensity,
    )
}

fn collect_fragment_components<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    strength: &[f32],
    weakness: &[f32],
    fabric: &[f32],
    propensity: &[f32],
    geology: &CrustalModel,
) -> Vec<FragmentComponent> {
    let count = topology.sample_count() as usize;
    let eligible = (0..count)
        .map(|sample| {
            let active_history = f64::from(geology.rift_history[sample])
                .max(f64::from(geology.transform_history[sample]))
                .max(f64::from(geology.subduction_history[sample]))
                .max(f64::from(geology.crustal_strain[sample]));
            f64::from(propensity[sample]) >= 0.57
                && f64::from(strength[sample]) <= 0.68
                && f64::from(fabric[sample]) >= 0.26
                && active_history >= 0.18
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
        let area = samples
            .iter()
            .map(|sample| topology.area_steradians(*sample))
            .sum::<f64>();
        let mean_weakness = samples
            .iter()
            .map(|sample| f64::from(weakness[*sample as usize]))
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
            samples,
            parent_plate_id,
            area_steradians: area,
            mean_weakness,
            mean_propensity,
            seed_sample,
        });
    }
    components.sort_by(|left, right| {
        right
            .mean_propensity
            .total_cmp(&left.mean_propensity)
            .then_with(|| right.area_steradians.total_cmp(&left.area_steradians))
            .then_with(|| left.seed_sample.cmp(&right.seed_sample))
    });
    components
}

fn build_refinement<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    geology: &CrustalModel,
    strength: &[f32],
    weakness: &[f32],
    fabric: &[f32],
    propensity: &[f32],
    refinement_seed: u64,
) -> (Vec<u16>, Vec<u16>, Vec<TectonicFragment>) {
    let mut fragment_ids = vec![0_u16; topology.sample_count() as usize];
    let mut domain_ids = tectonics.plate_ids.clone();
    let mut fragments = Vec::new();
    let mut accepted_area = 0.0_f64;
    let maximum_fragmented_area = 4.0 * PI * 0.22;
    let components = collect_fragment_components(
        topology, tectonics, strength, weakness, fabric, propensity, geology,
    );

    for component in components {
        if fragments.len() >= MAX_TECTONIC_FRAGMENTS {
            break;
        }
        let parent = &tectonics.plates[component.parent_plate_id as usize];
        let parent_fraction = component.area_steradians / parent.area_steradians.max(1.0e-12);
        if parent_fraction > 0.24
            || accepted_area + component.area_steradians > maximum_fragmented_area
        {
            continue;
        }
        let kind = if component.samples.len() >= 4
            && parent_fraction >= 0.006
            && parent_fraction <= 0.16
            && component.mean_propensity >= 0.62
        {
            TectonicFragmentKind::Microplate
        } else {
            TectonicFragmentKind::Terrane
        };
        let fragment_id = (fragments.len() + 1) as u16;
        let domain_id = tectonics.plates.len() as u16 + fragment_id - 1;
        for sample in &component.samples {
            fragment_ids[*sample as usize] = fragment_id;
            domain_ids[*sample as usize] = domain_id;
        }
        let angular_velocity = if kind == TectonicFragmentKind::Microplate {
            let parent_velocity = parent.angular_velocity_rad_per_myr;
            let parent_speed = norm(parent_velocity);
            let perturb_direction = random_unit_vector(
                refinement_seed,
                u64::from(component.seed_sample) ^ u64::from(fragment_id),
            );
            let perturb_scale = parent_speed * (0.035 + component.mean_weakness * 0.115);
            [
                parent_velocity[0] + perturb_direction[0] * perturb_scale,
                parent_velocity[1] + perturb_direction[1] * perturb_scale,
                parent_velocity[2] + perturb_direction[2] * perturb_scale,
            ]
        } else {
            parent.angular_velocity_rad_per_myr
        };
        fragments.push(TectonicFragment {
            id: fragment_id,
            parent_plate_id: component.parent_plate_id,
            seed_sample: component.seed_sample,
            kind,
            sample_count: component.samples.len() as u32,
            area_steradians: component.area_steradians,
            area_fraction_of_parent: parent_fraction,
            mean_weakness: component.mean_weakness,
            mean_fragmentation_propensity: component.mean_propensity,
            angular_velocity_rad_per_myr: angular_velocity,
        });
        accepted_area += component.area_steradians;
    }
    (fragment_ids, domain_ids, fragments)
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

fn model_hash(model: &LithosphericModel) -> u64 {
    let mut hash = fnv_update(FNV_OFFSET_BASIS, b"interlink-lithosphere:v1\0");
    hash = fnv_update(hash, &model.stage.derived_seed.to_le_bytes());
    hash = hash_f32(hash, &model.strength_index);
    hash = hash_f32(hash, &model.weakness_index);
    hash = hash_f32(hash, &model.effective_elastic_thickness_km);
    hash = hash_f32(hash, &model.thermal_anomaly_index);
    hash = hash_f32(hash, &model.mantle_upwelling_index);
    hash = hash_f32(hash, &model.mantle_dynamic_support_index);
    hash = hash_f32(hash, &model.compensated_buoyancy_index);
    hash = hash_f32(hash, &model.structural_fabric_strength);
    hash = hash_u8(hash, &model.structural_zone_kind);
    hash = hash_f32(hash, &model.fragmentation_propensity);
    hash = hash_u16(hash, &model.fragment_ids);
    hash = hash_u16(hash, &model.kinematic_domain_ids);
    for fragment in &model.fragments {
        hash = fnv_update(hash, &fragment.id.to_le_bytes());
        hash = fnv_update(hash, &fragment.parent_plate_id.to_le_bytes());
        hash = fnv_update(hash, &fragment.seed_sample.to_le_bytes());
        hash = fnv_update(hash, &[fragment.kind as u8]);
        hash = fnv_update(hash, &fragment.area_steradians.to_bits().to_le_bytes());
        for value in fragment.angular_velocity_rad_per_myr {
            hash = fnv_update(hash, &value.to_bits().to_le_bytes());
        }
    }
    hash
}

fn validate_model<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    model: &LithosphericModel,
) -> Result<(), WorldgenError> {
    let count = topology.sample_count() as usize;
    let lengths = [
        model.strength_index.len(),
        model.weakness_index.len(),
        model.effective_elastic_thickness_km.len(),
        model.thermal_anomaly_index.len(),
        model.mantle_upwelling_index.len(),
        model.mantle_dynamic_support_index.len(),
        model.compensated_buoyancy_index.len(),
        model.structural_fabric_strength.len(),
        model.structural_zone_kind.len(),
        model.fragmentation_propensity.len(),
        model.fragment_ids.len(),
        model.kinematic_domain_ids.len(),
    ];
    if lengths.iter().any(|length| *length != count) {
        return Err(WorldgenError::InvalidLithosphere(
            "lithosphere fields do not match topology sample count",
        ));
    }
    for sample in 0..count {
        let unit_fields = [
            model.strength_index[sample],
            model.weakness_index[sample],
            model.mantle_upwelling_index[sample],
            model.structural_fabric_strength[sample],
            model.fragmentation_propensity[sample],
        ];
        if unit_fields
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0 || *value > 1.0)
        {
            return Err(WorldgenError::InvalidLithosphere(
                "lithosphere normalized field is outside [0,1]",
            ));
        }
        let signed_fields = [
            model.thermal_anomaly_index[sample],
            model.mantle_dynamic_support_index[sample],
            model.compensated_buoyancy_index[sample],
        ];
        if signed_fields
            .iter()
            .any(|value| !value.is_finite() || *value < -1.0 || *value > 1.0)
        {
            return Err(WorldgenError::InvalidLithosphere(
                "lithosphere signed field is outside [-1,1]",
            ));
        }
        if !model.effective_elastic_thickness_km[sample].is_finite()
            || model.effective_elastic_thickness_km[sample] < 4.0
            || model.effective_elastic_thickness_km[sample] > 86.0
        {
            return Err(WorldgenError::InvalidLithosphere(
                "effective elastic thickness is outside supported bounds",
            ));
        }
        let fragment_id = model.fragment_ids[sample];
        if fragment_id > 0 {
            let fragment = model.fragments.get(fragment_id as usize - 1).ok_or(
                WorldgenError::InvalidLithosphere("fragment field references missing fragment"),
            )?;
            if tectonics.plate_ids[sample] != fragment.parent_plate_id {
                return Err(WorldgenError::InvalidLithosphere(
                    "tectonic fragment crosses a macro-plate boundary",
                ));
            }
        }
    }
    if model.fragments.len() > MAX_TECTONIC_FRAGMENTS {
        return Err(WorldgenError::InvalidLithosphere(
            "too many tectonic fragments",
        ));
    }
    if model.metrics.fragmented_area_fraction > 0.220_001 {
        return Err(WorldgenError::InvalidLithosphere(
            "tectonic refinement is not selective",
        ));
    }
    Ok(())
}

pub fn generate_lithosphere<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    geology: &CrustalModel,
    request: &LithosphereRequest,
) -> Result<LithosphericModel, WorldgenError> {
    if tectonics.plate_ids.len() != topology.sample_count() as usize
        || geology.crust_kind.len() != topology.sample_count() as usize
    {
        return Err(WorldgenError::InvalidLithosphere(
            "upstream tectonic/geology state does not match lithosphere topology",
        ));
    }
    let stage_seed = random::derive_stage_seed(&request.seed, LITHOSPHERE_NAMESPACE);
    let mechanical_seed = random::derive_stage_seed(&request.seed, MECHANICAL_NAMESPACE);
    let mantle_seed = random::derive_stage_seed(&request.seed, MANTLE_NAMESPACE);
    let refinement_seed = random::derive_stage_seed(&request.seed, REFINEMENT_NAMESPACE);
    let (
        strength_index,
        weakness_index,
        effective_elastic_thickness_km,
        thermal_anomaly_index,
        mantle_upwelling_index,
        mantle_dynamic_support_index,
        compensated_buoyancy_index,
        structural_fabric_strength,
        structural_zone_kind,
        fragmentation_propensity,
    ) = build_mechanical_state(topology, geology, mechanical_seed, mantle_seed);
    let (fragment_ids, kinematic_domain_ids, fragments) = build_refinement(
        topology,
        tectonics,
        geology,
        &strength_index,
        &weakness_index,
        &structural_fabric_strength,
        &fragmentation_propensity,
        refinement_seed,
    );
    let total_area = (0..topology.sample_count())
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>();
    let fragmented_area = (0..topology.sample_count())
        .filter(|sample| fragment_ids[*sample as usize] > 0)
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>();
    let microplates = fragments
        .iter()
        .filter(|fragment| fragment.kind == TectonicFragmentKind::Microplate)
        .count() as u16;
    let terranes = fragments.len() as u16 - microplates;
    let mut metrics = LithosphereMetrics {
        sample_count: topology.sample_count(),
        mean_strength_index: weighted_mean(topology, &strength_index),
        mean_weakness_index: weighted_mean(topology, &weakness_index),
        mean_effective_elastic_thickness_km: weighted_mean(
            topology,
            &effective_elastic_thickness_km,
        ),
        mean_mantle_upwelling_index: weighted_mean(topology, &mantle_upwelling_index),
        mean_dynamic_support_index: weighted_mean(topology, &mantle_dynamic_support_index),
        suture_sample_count: structural_zone_kind
            .iter()
            .filter(|kind| **kind == StructuralZoneKind::Suture as u8)
            .count() as u32,
        rift_zone_sample_count: structural_zone_kind
            .iter()
            .filter(|kind| **kind == StructuralZoneKind::Rift as u8)
            .count() as u32,
        transform_zone_sample_count: structural_zone_kind
            .iter()
            .filter(|kind| **kind == StructuralZoneKind::Transform as u8)
            .count() as u32,
        continental_margin_sample_count: structural_zone_kind
            .iter()
            .filter(|kind| **kind == StructuralZoneKind::ContinentalMargin as u8)
            .count() as u32,
        tectonic_fragment_count: fragments.len() as u16,
        microplate_count: microplates,
        terrane_count: terranes,
        fragmented_area_fraction: fragmented_area / total_area.max(1.0e-12),
        lithosphere_hash: 0,
    };
    let mut model = LithosphericModel {
        stage: StageIdentity {
            id: LITHOSPHERE_STAGE_ID,
            version: LITHOSPHERE_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        mechanical_seed,
        mantle_seed,
        refinement_seed,
        strength_index,
        weakness_index,
        effective_elastic_thickness_km,
        thermal_anomaly_index,
        mantle_upwelling_index,
        mantle_dynamic_support_index,
        compensated_buoyancy_index,
        structural_fabric_strength,
        structural_zone_kind,
        fragmentation_propensity,
        fragment_ids,
        kinematic_domain_ids,
        fragments,
        metrics: metrics.clone(),
    };
    metrics.lithosphere_hash = model_hash(&model);
    model.metrics = metrics;
    validate_model(topology, tectonics, &model)?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_icosphere, generate_crust_and_history, generate_tectonics, GeologyRequest,
        PlanetPhysicalParameters, TectonicsRequest,
    };

    fn generate(
        seed: &str,
        level: u8,
        plate_count: u16,
    ) -> (
        crate::GeodesicTopology,
        TectonicModel,
        CrustalModel,
        LithosphericModel,
    ) {
        let topology = build_icosphere(level).unwrap();
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let tectonics = generate_tectonics(
            &topology,
            &TectonicsRequest::new(seed, plate_count),
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
        let lithosphere = generate_lithosphere(
            &topology,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed),
        )
        .unwrap();
        (topology, tectonics, geology, lithosphere)
    }

    #[test]
    fn lithosphere_is_deterministic_and_seed_sensitive() {
        let (_, _, _, first) = generate("wg3-5-determinism", 4, 16);
        let (_, _, _, second) = generate("wg3-5-determinism", 4, 16);
        let (_, _, _, changed) = generate("wg3-5-determinism-b", 4, 16);
        assert_eq!(
            first.metrics.lithosphere_hash,
            second.metrics.lithosphere_hash
        );
        assert_eq!(first.strength_index, second.strength_index);
        assert_ne!(
            first.metrics.lithosphere_hash,
            changed.metrics.lithosphere_hash
        );
    }

    #[test]
    fn fragments_are_selective_and_never_cross_parent_macro_plates() {
        let (topology, tectonics, _, lithosphere) = generate("wg3-5-fragments", 5, 18);
        assert!(lithosphere.metrics.fragmented_area_fraction <= 0.220_001);
        for sample in 0..topology.sample_count() as usize {
            let fragment_id = lithosphere.fragment_ids[sample];
            if fragment_id == 0 {
                continue;
            }
            let fragment = &lithosphere.fragments[fragment_id as usize - 1];
            assert_eq!(tectonics.plate_ids[sample], fragment.parent_plate_id);
            assert!(lithosphere.kinematic_domain_ids[sample] >= tectonics.plates.len() as u16);
        }
    }

    #[test]
    fn stable_old_crust_is_stronger_than_high_strain_crust_in_ensemble() {
        let mut stable_sum = 0.0;
        let mut stable_count = 0_u32;
        let mut active_sum = 0.0;
        let mut active_count = 0_u32;
        let mut microplate_total = 0_u32;
        for seed in ["wg3-5-a", "wg3-5-b", "wg3-5-c", "wg3-5-d", "wg3-5-e"] {
            let (_, _, geology, lithosphere) = generate(seed, 4, 16);
            microplate_total += u32::from(lithosphere.metrics.microplate_count);
            for sample in 0..geology.crust_kind.len() {
                if geology.crust_kind[sample] == CrustKind::Continental as u8
                    && geology.crust_age_myr[sample] > 1500.0
                    && geology.crustal_strain[sample] < 0.18
                {
                    stable_sum += f64::from(lithosphere.strength_index[sample]);
                    stable_count += 1;
                }
                if geology.crustal_strain[sample] > 0.52 {
                    active_sum += f64::from(lithosphere.strength_index[sample]);
                    active_count += 1;
                }
            }
        }
        assert!(stable_count > 0 && active_count > 0);
        assert!(stable_sum / f64::from(stable_count) > active_sum / f64::from(active_count));
        assert!(
            microplate_total > 0,
            "ensemble should exercise selective microplate refinement"
        );
    }
}
