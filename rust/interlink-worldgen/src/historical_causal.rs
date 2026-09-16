use crate::{
    build_historical_tectonic_morphology, generate_event_driven_orogen_provinces,
    generate_pre_orogenic_lithosphere_from_history, HistoricalLithosphereModel,
    InheritedStructureKind, LithosphereRequest, LithosphericModel, OrogenProvinceRequest,
    PlanetPhysicalParameters, PlanetTopology, PreOrogenicLithosphereRequest, StructuralZoneKind,
    TectonicModel, WorldgenError,
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

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

fn area_weighted_mean<T: PlanetTopology>(topology: &T, values: &[f32]) -> f64 {
    let mut weighted = 0.0_f64;
    let mut area = 0.0_f64;
    for sample in 0..topology.sample_count() {
        let weight = topology.area_steradians(sample);
        weighted += f64::from(values[sample as usize]) * weight;
        area += weight;
    }
    weighted / area.max(1.0e-12)
}

fn project_historical_substrate<T: PlanetTopology>(
    topology: &T,
    model: &mut LithosphericModel,
) {
    let base_hash = model.metrics.lithosphere_hash;
    let pre_hash = model.pre_orogenic.metrics.pre_orogenic_hash;
    let strength = model.pre_orogenic.intrinsic_strength_index.clone();
    let weakness = model.pre_orogenic.intrinsic_weakness_index.clone();
    let elastic_thickness = model.pre_orogenic.effective_elastic_thickness_km.clone();
    let fabric = model.pre_orogenic.inherited_fabric_strength.clone();
    let fragmentation = model.pre_orogenic.fragmentation_propensity.clone();
    let structure = model
        .pre_orogenic
        .inherited_structure_kind
        .iter()
        .map(|kind| match *kind {
            value if value == InheritedStructureKind::PaleoSuture as u8 => {
                StructuralZoneKind::Suture as u8
            }
            value if value == InheritedStructureKind::InheritedRift as u8 => {
                StructuralZoneKind::Rift as u8
            }
            value if value == InheritedStructureKind::ShearZone as u8 => {
                StructuralZoneKind::Transform as u8
            }
            value if value == InheritedStructureKind::ContinentalMargin as u8 => {
                StructuralZoneKind::ContinentalMargin as u8
            }
            _ => StructuralZoneKind::None as u8,
        })
        .collect::<Vec<_>>();

    model.strength_index = strength;
    model.weakness_index = weakness;
    model.effective_elastic_thickness_km = elastic_thickness;
    model.structural_fabric_strength = fabric;
    model.structural_zone_kind = structure;
    model.fragmentation_propensity = fragmentation;

    model.metrics.mean_strength_index = area_weighted_mean(topology, &model.strength_index);
    model.metrics.mean_weakness_index = area_weighted_mean(topology, &model.weakness_index);
    model.metrics.mean_effective_elastic_thickness_km =
        area_weighted_mean(topology, &model.effective_elastic_thickness_km);
    model.metrics.suture_sample_count = model
        .structural_zone_kind
        .iter()
        .filter(|kind| **kind == StructuralZoneKind::Suture as u8)
        .count() as u32;
    model.metrics.rift_zone_sample_count = model
        .structural_zone_kind
        .iter()
        .filter(|kind| **kind == StructuralZoneKind::Rift as u8)
        .count() as u32;
    model.metrics.transform_zone_sample_count = model
        .structural_zone_kind
        .iter()
        .filter(|kind| **kind == StructuralZoneKind::Transform as u8)
        .count() as u32;
    model.metrics.continental_margin_sample_count = model
        .structural_zone_kind
        .iter()
        .filter(|kind| **kind == StructuralZoneKind::ContinentalMargin as u8)
        .count() as u32;

    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, b"geology:lithosphere-from-material-history:v1\0");
    hash = fnv_update(hash, &base_hash.to_le_bytes());
    hash = fnv_update(hash, &pre_hash.to_le_bytes());
    hash = hash_f32(hash, &model.strength_index);
    hash = hash_f32(hash, &model.weakness_index);
    hash = hash_f32(hash, &model.effective_elastic_thickness_km);
    hash = hash_f32(hash, &model.structural_fabric_strength);
    hash = hash_u8(hash, &model.structural_zone_kind);
    hash = hash_f32(hash, &model.fragmentation_propensity);
    model.metrics.lithosphere_hash = hash;
}

/// History-aware WG-3.5/WG-3.6 authority when the caller already retains the PR-A material model.
pub fn generate_lithosphere_from_history<T: PlanetTopology>(
    topology: &T,
    historical: &HistoricalLithosphereModel,
    tectonics: &TectonicModel,
    geology: &crate::CrustalModel,
    request: &LithosphereRequest,
) -> Result<LithosphericModel, WorldgenError> {
    if historical.current_plate_ids != tectonics.plate_ids {
        return Err(WorldgenError::InvalidLithosphere(
            "history-aware lithosphere requires tectonics projected from the supplied material history",
        ));
    }
    if historical.crust_kind != geology.crust_kind {
        return Err(WorldgenError::InvalidLithosphere(
            "history-aware lithosphere requires geology projected from the supplied material history",
        ));
    }

    let mut model = crate::causal_pipeline::generate_lithosphere(
        topology,
        tectonics,
        geology,
        request,
    )?;
    let morphology = build_historical_tectonic_morphology(
        topology,
        historical,
        tectonics,
        request.seed.as_str(),
    )?;
    model.pre_orogenic = generate_pre_orogenic_lithosphere_from_history(
        topology,
        tectonics,
        &model.tectonic_history,
        geology,
        &morphology,
        &PreOrogenicLithosphereRequest::new(request.seed.as_str()),
    )?;
    project_historical_substrate(topology, &mut model);
    model.orogen_provinces = generate_event_driven_orogen_provinces(
        topology,
        tectonics,
        &model.tectonic_history,
        geology,
        &model.pre_orogenic,
        &morphology,
        &OrogenProvinceRequest::new(request.seed.as_str()),
        PlanetPhysicalParameters::earthlike_reference(),
    )?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_icosphere, generate_historical_frontend, HistoricalLithosphereRequest};

    #[test]
    fn history_aware_lithosphere_changes_inherited_structural_authority() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let seed = "historical-causal-lithosphere";
        let frontend = generate_historical_frontend(
            &topology,
            &HistoricalLithosphereRequest::new(seed, 16),
            planet,
        )
        .unwrap();
        let legacy_causal = crate::causal_pipeline::generate_lithosphere(
            &topology,
            &frontend.tectonics,
            &frontend.geology,
            &LithosphereRequest::new(seed),
        )
        .unwrap();
        let historical_causal = generate_lithosphere_from_history(
            &topology,
            &frontend.historical,
            &frontend.tectonics,
            &frontend.geology,
            &LithosphereRequest::new(seed),
        )
        .unwrap();
        assert_ne!(
            legacy_causal.pre_orogenic.metrics.pre_orogenic_hash,
            historical_causal.pre_orogenic.metrics.pre_orogenic_hash
        );
        assert_ne!(
            legacy_causal.orogen_provinces.metrics.province_hash,
            historical_causal.orogen_provinces.metrics.province_hash
        );
        assert_ne!(legacy_causal.metrics.lithosphere_hash, historical_causal.metrics.lithosphere_hash);
        assert_eq!(
            historical_causal.metrics.suture_sample_count,
            historical_causal.pre_orogenic.metrics.paleo_suture_sample_count
        );
        assert_eq!(
            historical_causal.metrics.rift_zone_sample_count,
            historical_causal.pre_orogenic.metrics.inherited_rift_sample_count
        );
        assert!(historical_causal.pre_orogenic.metrics.paleo_suture_sample_count > 0);
        assert!(historical_causal.pre_orogenic.metrics.inherited_rift_sample_count > 0);
    }
}
