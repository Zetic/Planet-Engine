use crate::{
    generate_pre_orogenic_lithosphere, HistoricalMorphologyModel, InheritedStructureKind,
    PlanetTopology, PreOrogenicLithosphereModel, PreOrogenicLithosphereRequest,
    TectonicHistoryModel, TectonicModel, WorldgenError,
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

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

fn weighted_mean<T: PlanetTopology>(topology: &T, values: &[f32]) -> f64 {
    let mut weighted = 0.0_f64;
    let mut area = 0.0_f64;
    for sample in 0..topology.sample_count() {
        let weight = topology.area_steradians(sample);
        weighted += f64::from(values[sample as usize]) * weight;
        area += weight;
    }
    weighted / area.max(1.0e-12)
}

fn historical_pre_hash(
    base_hash: u64,
    morphology_hash: u64,
    model: &PreOrogenicLithosphereModel,
) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, b"geology:pre-orogenic-from-material-history:v1\0");
    hash = fnv_update(hash, &base_hash.to_le_bytes());
    hash = fnv_update(hash, &morphology_hash.to_le_bytes());
    hash = hash_f32(hash, &model.intrinsic_strength_index);
    hash = hash_f32(hash, &model.intrinsic_weakness_index);
    hash = hash_f32(hash, &model.effective_elastic_thickness_km);
    hash = hash_f32(hash, &model.inherited_fabric_strength);
    hash = hash_u8(hash, &model.inherited_structure_kind);
    hash = hash_f32(hash, &model.inherited_rift_memory);
    hash = hash_f32(hash, &model.inherited_shear_memory);
    hash = hash_f32(hash, &model.fragmentation_propensity);
    hash
}

pub fn generate_pre_orogenic_lithosphere_from_history<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    active_history: &TectonicHistoryModel,
    geology: &crate::CrustalModel,
    morphology: &HistoricalMorphologyModel,
    request: &PreOrogenicLithosphereRequest,
) -> Result<PreOrogenicLithosphereModel, WorldgenError> {
    let mut model =
        generate_pre_orogenic_lithosphere(topology, tectonics, active_history, geology, request)?;
    let count = topology.sample_count() as usize;
    let fields = [
        morphology.rift_intensity.len(),
        morphology.shear_intensity.len(),
        morphology.suture_intensity.len(),
        morphology.passive_margin_index.len(),
        morphology.fossil_orogen_intensity.len(),
        morphology.cumulative_shortening_km.len(),
    ];
    if fields.iter().any(|length| *length != count) {
        return Err(WorldgenError::InvalidLithosphere(
            "historical morphology does not align with pre-orogenic topology",
        ));
    }

    let base_hash = model.metrics.pre_orogenic_hash;
    for sample in 0..count {
        let historical_rift = f64::from(morphology.rift_intensity[sample]);
        let historical_shear = f64::from(morphology.shear_intensity[sample]);
        let historical_suture = f64::from(morphology.suture_intensity[sample]);
        let passive_margin = f64::from(morphology.passive_margin_index[sample]);
        let fossil_orogen = f64::from(morphology.fossil_orogen_intensity[sample]);
        let shortening = clamp01(f64::from(morphology.cumulative_shortening_km[sample]) / 1200.0);

        // Current-boundary-derived memory becomes a weak compatibility floor. Persistent event
        // history is the authority for inherited rifts and shear zones.
        let rift = historical_rift.max(f64::from(model.inherited_rift_memory[sample]) * 0.22);
        let shear = historical_shear.max(f64::from(model.inherited_shear_memory[sample]) * 0.22);
        model.inherited_rift_memory[sample] = clamp01(rift) as f32;
        model.inherited_shear_memory[sample] = clamp01(shear) as f32;

        let static_crust_boundary = f64::from(model.province_boundary_index[sample]);
        let craton_boundary = static_crust_boundary
            * if historical_suture.max(historical_rift).max(historical_shear) < 0.22 {
                0.72
            } else {
                0.38
            };
        let historical_fabric = historical_suture
            .max(rift)
            .max(shear)
            .max(passive_margin)
            .max(fossil_orogen * 0.82)
            .max(craton_boundary)
            .clamp(0.0, 1.0);
        model.inherited_fabric_strength[sample] = historical_fabric as f32;

        model.inherited_structure_kind[sample] = if historical_fabric < 0.20 {
            InheritedStructureKind::None as u8
        } else if historical_suture >= rift
            && historical_suture >= shear
            && historical_suture >= passive_margin
            && historical_suture >= craton_boundary
        {
            InheritedStructureKind::PaleoSuture as u8
        } else if rift >= shear && rift >= passive_margin && rift >= craton_boundary {
            InheritedStructureKind::InheritedRift as u8
        } else if shear >= passive_margin && shear >= craton_boundary {
            InheritedStructureKind::ShearZone as u8
        } else if passive_margin >= craton_boundary {
            InheritedStructureKind::ContinentalMargin as u8
        } else {
            InheritedStructureKind::CratonBoundary as u8
        };

        let damage = clamp01(
            historical_suture * 0.22
                + rift * 0.34
                + shear * 0.28
                + passive_margin * 0.16
                + fossil_orogen * 0.15
                + shortening * 0.10,
        );
        let old_strength = f64::from(model.intrinsic_strength_index[sample]);
        let old_weakness = f64::from(model.intrinsic_weakness_index[sample]);
        let strength = clamp01(old_strength * 0.82 + (1.0 - damage) * 0.18 - damage * 0.12);
        let weakness =
            clamp01(old_weakness * 0.68 + (1.0 - strength) * 0.18 + historical_fabric * 0.25);
        model.intrinsic_strength_index[sample] = strength as f32;
        model.intrinsic_weakness_index[sample] = weakness as f32;

        let te = f64::from(model.effective_elastic_thickness_km[sample]);
        model.effective_elastic_thickness_km[sample] =
            (te - 9.0 * damage - 4.0 * passive_margin).clamp(4.0, 92.0) as f32;
        model.fragmentation_propensity[sample] = clamp01(
            f64::from(model.fragmentation_propensity[sample]) * 0.58
                + weakness * 0.22
                + historical_fabric * 0.30
                + rift.max(shear) * 0.18,
        ) as f32;
    }

    model.metrics.mean_intrinsic_strength_index =
        weighted_mean(topology, &model.intrinsic_strength_index);
    model.metrics.mean_intrinsic_weakness_index =
        weighted_mean(topology, &model.intrinsic_weakness_index);
    model.metrics.mean_effective_elastic_thickness_km =
        weighted_mean(topology, &model.effective_elastic_thickness_km);
    model.metrics.mean_inherited_fabric_strength =
        weighted_mean(topology, &model.inherited_fabric_strength);
    model.metrics.paleo_suture_sample_count = model
        .inherited_structure_kind
        .iter()
        .filter(|kind| **kind == InheritedStructureKind::PaleoSuture as u8)
        .count() as u32;
    model.metrics.inherited_rift_sample_count = model
        .inherited_structure_kind
        .iter()
        .filter(|kind| **kind == InheritedStructureKind::InheritedRift as u8)
        .count() as u32;
    model.metrics.shear_zone_sample_count = model
        .inherited_structure_kind
        .iter()
        .filter(|kind| **kind == InheritedStructureKind::ShearZone as u8)
        .count() as u32;
    model.metrics.continental_margin_sample_count = model
        .inherited_structure_kind
        .iter()
        .filter(|kind| **kind == InheritedStructureKind::ContinentalMargin as u8)
        .count() as u32;
    model.metrics.craton_boundary_sample_count = model
        .inherited_structure_kind
        .iter()
        .filter(|kind| **kind == InheritedStructureKind::CratonBoundary as u8)
        .count() as u32;
    model.metrics.pre_orogenic_hash =
        historical_pre_hash(base_hash, morphology.metrics.morphology_hash, &model);

    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_historical_tectonic_morphology, build_icosphere, generate_historical_frontend,
        generate_tectonic_history, HistoricalLithosphereRequest, PlanetPhysicalParameters,
        TectonicHistoryRequest,
    };

    #[test]
    fn historical_structure_overrides_present_boundary_memory() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let seed = "historical-pre-orogenic-test";
        let frontend = generate_historical_frontend(
            &topology,
            &HistoricalLithosphereRequest::new(seed, 16),
            planet,
        )
        .unwrap();
        let active_history = generate_tectonic_history(
            &topology,
            &frontend.tectonics,
            &TectonicHistoryRequest::new(seed),
            planet,
        )
        .unwrap();
        let morphology = build_historical_tectonic_morphology(
            &topology,
            &frontend.historical,
            &frontend.tectonics,
            seed,
        )
        .unwrap();
        let model = generate_pre_orogenic_lithosphere_from_history(
            &topology,
            &frontend.tectonics,
            &active_history,
            &frontend.geology,
            &morphology,
            &PreOrogenicLithosphereRequest::new(seed),
        )
        .unwrap();
        assert!(model.metrics.paleo_suture_sample_count > 0);
        assert!(model.metrics.inherited_rift_sample_count > 0);
        assert!(model.metrics.mean_inherited_fabric_strength > 0.0);
        assert_ne!(model.metrics.pre_orogenic_hash, 0);
    }
}
