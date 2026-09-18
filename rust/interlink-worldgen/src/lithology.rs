use crate::{
    derive_stage_seed, CrustKind, GeodesicTopology, InheritedHistoricalIdentity,
    InheritedPhysicalState, InheritedStructureKind, StageIdentity, WorldgenError,
};

pub const LITHOLOGY_STAGE_ID: &str = "geology:lithology-substrate";
pub const LITHOLOGY_STAGE_VERSION: u32 = 1;
const LITHOLOGY_NAMESPACE: &str = "worldgen:geology:lithology-substrate:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BedrockClass {
    OceanicBasalt = 1,
    OceanicSediment = 2,
    CrystallineBasement = 3,
    OrogenicMetamorphic = 4,
    ArcVolcanic = 5,
    RiftVolcanic = 6,
    ClasticSedimentary = 7,
    CarbonatePlatform = 8,
    AccretedTerrane = 9,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LithologyRequest {
    pub seed: String,
}
impl LithologyRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LithologyMetrics {
    pub sample_count: u32,
    pub class_sample_counts: [u32; 9],
    pub mean_rock_strength_index: f64,
    pub mean_erodibility_index: f64,
    pub mean_permeability_index: f64,
    pub mean_weathering_susceptibility: f64,
    pub mean_fines_fraction: f64,
    pub mean_carbonate_fraction: f64,
    pub lithology_hash: u64,
}
impl LithologyMetrics {
    pub fn lithology_hash_hex(&self) -> String {
        format!("{:016x}", self.lithology_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LithologyState {
    pub stage: StageIdentity,
    pub metrics: LithologyMetrics,
    pub bedrock_class: Vec<u8>,
    pub rock_strength_index: Vec<f32>,
    pub erodibility_index: Vec<f32>,
    pub permeability_index: Vec<f32>,
    pub weathering_susceptibility: Vec<f32>,
    pub fines_fraction: Vec<f32>,
    pub carbonate_fraction: Vec<f32>,
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn unit_random(value: u64) -> f64 {
    ((mix64(value) >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0)
}

fn provenance_variation(stage_seed: u64, origin_plate_id: u16, fragment_id: u16, lane: u64) -> f64 {
    let key = stage_seed
        ^ (u64::from(origin_plate_id) << 40)
        ^ (u64::from(fragment_id) << 16)
        ^ lane.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    unit_random(key)
}

fn validate_inputs(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    historical: &InheritedHistoricalIdentity,
) -> Result<(), WorldgenError> {
    let count = topology.metrics().sample_count as usize;
    let inherited_lengths = [
        inherited.crust_kind.len(),
        inherited.oceanic_age_myr.len(),
        inherited.continental_basement_age_myr.len(),
        inherited.last_tectonic_reworking_age_myr.len(),
        inherited.continental_stability_index.len(),
        inherited.crust_thickness_km.len(),
        inherited.rift_history.len(),
        inherited.ridge_history.len(),
        inherited.subduction_history.len(),
        inherited.volcanic_arc_history.len(),
        inherited.transform_history.len(),
        inherited.subsidence_history.len(),
        inherited.basin_potential.len(),
        inherited.strength_index.len(),
        inherited.weakness_index.len(),
        inherited.thermal_anomaly_index.len(),
        inherited.structural_fabric_strength.len(),
        inherited.structural_zone_kind.len(),
        inherited.fragmentation_propensity.len(),
        inherited.province_kind.len(),
        inherited.orogenic_history.len(),
        inherited.suture_index.len(),
    ];
    let historical_lengths = [
        historical.origin_plate_ids.len(),
        historical.fragment_ids.len(),
        historical.current_plate_ids.len(),
        historical.crust_kind.len(),
        historical.crust_birth_age_myr.len(),
    ];
    if inherited_lengths.iter().any(|length| *length != count)
        || historical_lengths.iter().any(|length| *length != count)
    {
        return Err(WorldgenError::InvalidLithology(
            "lithology inputs do not match the fine topology",
        ));
    }
    if historical.map.metrics.fine_sample_count as usize != count
        || inherited.map.metrics.fine_sample_count as usize != count
    {
        return Err(WorldgenError::InvalidLithology(
            "lithology inheritance maps do not match the fine topology",
        ));
    }
    Ok(())
}

fn classify_bedrock(
    inherited: &InheritedPhysicalState,
    historical: &InheritedHistoricalIdentity,
    sample: usize,
    stage_seed: u64,
) -> BedrockClass {
    let crust = inherited.crust_kind[sample];
    let oceanic_age = f64::from(inherited.oceanic_age_myr[sample]).max(0.0);
    let rift = clamp01(f64::from(inherited.rift_history[sample]));
    let ridge = clamp01(f64::from(inherited.ridge_history[sample]));
    let subduction = clamp01(f64::from(inherited.subduction_history[sample]));
    let arc = clamp01(f64::from(inherited.volcanic_arc_history[sample]));
    let subsidence = clamp01(f64::from(inherited.subsidence_history[sample]));
    let basin = clamp01(f64::from(inherited.basin_potential[sample]));
    let thermal = clamp01(f64::from(inherited.thermal_anomaly_index[sample]));
    let weakness = clamp01(f64::from(inherited.weakness_index[sample]));
    let fragmentation = clamp01(f64::from(inherited.fragmentation_propensity[sample]));
    let orogen = clamp01(f64::from(inherited.orogenic_history[sample]));
    let suture = clamp01(f64::from(inherited.suture_index[sample]));
    let structure = inherited.structural_zone_kind[sample];
    let fragment_bias = provenance_variation(
        stage_seed,
        historical.origin_plate_ids[sample],
        historical.fragment_ids[sample],
        1,
    );
    let carbonate_bias = provenance_variation(
        stage_seed,
        historical.origin_plate_ids[sample],
        historical.fragment_ids[sample],
        2,
    );

    if crust == CrustKind::Oceanic as u8 {
        let sediment_score =
            0.46 * clamp01(oceanic_age / 190.0) + 0.30 * subsidence + 0.25 * basin
            - 0.38 * ridge
            - 0.18 * thermal;
        return if sediment_score > 0.53 {
            BedrockClass::OceanicSediment
        } else {
            BedrockClass::OceanicBasalt
        };
    }

    let inherited_rift = structure == InheritedStructureKind::InheritedRift as u8;
    let passive_margin = structure == InheritedStructureKind::ContinentalMargin as u8;
    let paleo_suture = structure == InheritedStructureKind::PaleoSuture as u8;
    let shear_zone = structure == InheritedStructureKind::ShearZone as u8;
    let craton_boundary = structure == InheritedStructureKind::CratonBoundary as u8;

    if arc.max(subduction * 0.7) > 0.58 && thermal > 0.18 {
        return BedrockClass::ArcVolcanic;
    }
    if (inherited_rift || rift > 0.62) && thermal.max(ridge) > 0.28 {
        return BedrockClass::RiftVolcanic;
    }
    if inherited.province_kind[sample] != 0 && (orogen > 0.34 || suture > 0.34 || paleo_suture) {
        return BedrockClass::OrogenicMetamorphic;
    }
    if (paleo_suture || shear_zone || craton_boundary)
        && fragmentation.max(weakness) > 0.56
        && fragment_bias > 0.30
    {
        return BedrockClass::AccretedTerrane;
    }

    let sediment_score =
        0.42 * basin + 0.34 * subsidence + 0.22 * rift + if passive_margin { 0.28 } else { 0.0 };
    if sediment_score > 0.45 {
        let carbonate_score = 0.46 * carbonate_bias
            + 0.24 * (1.0 - arc)
            + 0.18 * (1.0 - thermal)
            + 0.18 * if passive_margin { 1.0 } else { 0.0 }
            - 0.16 * rift;
        if carbonate_score > 0.58 {
            return BedrockClass::CarbonatePlatform;
        }
        return BedrockClass::ClasticSedimentary;
    }

    if crust == CrustKind::Transitional as u8 {
        if carbonate_bias > 0.62 && rift < 0.48 {
            BedrockClass::CarbonatePlatform
        } else {
            BedrockClass::ClasticSedimentary
        }
    } else if orogen > 0.26 || suture > 0.28 {
        BedrockClass::OrogenicMetamorphic
    } else if fragmentation > 0.70 && fragment_bias > 0.60 {
        BedrockClass::AccretedTerrane
    } else {
        BedrockClass::CrystallineBasement
    }
}

fn base_properties(class: BedrockClass) -> [f64; 6] {
    match class {
        BedrockClass::OceanicBasalt => [0.72, 0.34, 0.12, 0.48, 0.12, 0.01],
        BedrockClass::OceanicSediment => [0.34, 0.74, 0.42, 0.66, 0.70, 0.06],
        BedrockClass::CrystallineBasement => [0.78, 0.24, 0.18, 0.38, 0.15, 0.04],
        BedrockClass::OrogenicMetamorphic => [0.82, 0.18, 0.12, 0.32, 0.10, 0.02],
        BedrockClass::ArcVolcanic => [0.68, 0.34, 0.22, 0.55, 0.25, 0.03],
        BedrockClass::RiftVolcanic => [0.60, 0.42, 0.28, 0.62, 0.30, 0.02],
        BedrockClass::ClasticSedimentary => [0.38, 0.70, 0.48, 0.72, 0.75, 0.08],
        BedrockClass::CarbonatePlatform => [0.62, 0.52, 0.70, 0.70, 0.18, 0.82],
        BedrockClass::AccretedTerrane => [0.58, 0.48, 0.30, 0.58, 0.38, 0.10],
    }
}

fn material_properties(
    class: BedrockClass,
    inherited: &InheritedPhysicalState,
    historical: &InheritedHistoricalIdentity,
    sample: usize,
    stage_seed: u64,
) -> [f32; 6] {
    let [base_strength, base_erodibility, base_permeability, base_weathering, base_fines, base_carbonate] =
        base_properties(class);
    let strength = clamp01(f64::from(inherited.strength_index[sample]));
    let weakness = clamp01(f64::from(inherited.weakness_index[sample]));
    let fabric = clamp01(f64::from(inherited.structural_fabric_strength[sample]));
    let basin = clamp01(f64::from(inherited.basin_potential[sample]));
    let subsidence = clamp01(f64::from(inherited.subsidence_history[sample]));
    let thermal = clamp01(f64::from(inherited.thermal_anomaly_index[sample]));
    let composition = provenance_variation(
        stage_seed,
        historical.origin_plate_ids[sample],
        historical.fragment_ids[sample],
        3,
    );
    let carbonate_bias = provenance_variation(
        stage_seed,
        historical.origin_plate_ids[sample],
        historical.fragment_ids[sample],
        2,
    );

    let rock_strength = clamp01(
        base_strength * (0.78 + 0.22 * strength) * (1.0 - 0.20 * weakness)
            + (composition - 0.5) * 0.05,
    );
    let erodibility = clamp01(
        base_erodibility * (1.08 - 0.24 * strength + 0.18 * weakness)
            + 0.08 * basin
            + 0.04 * fabric,
    );
    let permeability = clamp01(
        base_permeability + 0.14 * weakness + 0.08 * fabric + 0.06 * subsidence - 0.06 * strength,
    );
    let weathering =
        clamp01(base_weathering + 0.11 * thermal + 0.07 * weakness + (composition - 0.5) * 0.04);
    let fines = clamp01(base_fines + 0.17 * basin + 0.12 * subsidence - 0.08 * strength);
    let carbonate = clamp01(base_carbonate + (carbonate_bias - 0.5) * 0.12 - 0.10 * thermal);

    [
        rock_strength as f32,
        erodibility as f32,
        permeability as f32,
        weathering as f32,
        fines as f32,
        carbonate as f32,
    ]
}

pub fn generate_lithology_substrate(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    historical: &InheritedHistoricalIdentity,
    request: &LithologyRequest,
) -> Result<LithologyState, WorldgenError> {
    validate_inputs(topology, inherited, historical)?;
    let count = topology.metrics().sample_count as usize;
    let stage_seed = derive_stage_seed(&request.seed, LITHOLOGY_NAMESPACE);

    let mut bedrock_class = Vec::with_capacity(count);
    let mut rock_strength_index = Vec::with_capacity(count);
    let mut erodibility_index = Vec::with_capacity(count);
    let mut permeability_index = Vec::with_capacity(count);
    let mut weathering_susceptibility = Vec::with_capacity(count);
    let mut fines_fraction = Vec::with_capacity(count);
    let mut carbonate_fraction = Vec::with_capacity(count);
    let mut class_sample_counts = [0_u32; 9];
    let mut sums = [0.0_f64; 6];

    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, b"geology:lithology-substrate:v1\0");
    hash = fnv_update(hash, &stage_seed.to_le_bytes());
    hash = fnv_update(hash, &inherited.inheritance_hash().to_le_bytes());
    hash = fnv_update(hash, &historical.identity_hash.to_le_bytes());

    for sample in 0..count {
        let class = classify_bedrock(inherited, historical, sample, stage_seed);
        let properties = material_properties(class, inherited, historical, sample, stage_seed);
        bedrock_class.push(class as u8);
        class_sample_counts[class as usize - 1] += 1;
        hash = fnv_update(hash, &[class as u8]);
        for (index, value) in properties.iter().enumerate() {
            sums[index] += f64::from(*value);
            hash = fnv_update(hash, &value.to_bits().to_le_bytes());
        }
        rock_strength_index.push(properties[0]);
        erodibility_index.push(properties[1]);
        permeability_index.push(properties[2]);
        weathering_susceptibility.push(properties[3]);
        fines_fraction.push(properties[4]);
        carbonate_fraction.push(properties[5]);
    }

    let divisor = count.max(1) as f64;
    Ok(LithologyState {
        stage: StageIdentity {
            id: LITHOLOGY_STAGE_ID,
            version: LITHOLOGY_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics: LithologyMetrics {
            sample_count: count as u32,
            class_sample_counts,
            mean_rock_strength_index: sums[0] / divisor,
            mean_erodibility_index: sums[1] / divisor,
            mean_permeability_index: sums[2] / divisor,
            mean_weathering_susceptibility: sums[3] / divisor,
            mean_fines_fraction: sums[4] / divisor,
            mean_carbonate_fraction: sums[5] / divisor,
            lithology_hash: hash,
        },
        bedrock_class,
        rock_strength_index,
        erodibility_index,
        permeability_index,
        weathering_susceptibility,
        fines_fraction,
        carbonate_fraction,
    })
}
