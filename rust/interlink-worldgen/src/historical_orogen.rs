use crate::{
    generate_tectonic_orogen_provinces, HistoricalMorphologyModel, OrogenProvinceKind,
    OrogenProvinceModel, OrogenProvinceRequest, PlanetPhysicalParameters, PlanetTopology,
    PreOrogenicLithosphereModel, TectonicHistoryModel, TectonicModel, WorldgenError,
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
fn event_driven_hash(base_hash: u64, morphology_hash: u64, model: &OrogenProvinceModel) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, b"geology:event-driven-orogen-provinces:v1\0");
    hash = fnv_update(hash, &base_hash.to_le_bytes());
    hash = fnv_update(hash, &morphology_hash.to_le_bytes());
    hash = hash_u8(hash, &model.province_kind);
    for values in [
        &model.orogenic_intensity,
        &model.mountain_core_index,
        &model.crustal_root_index,
        &model.plateau_index,
        &model.fold_thrust_index,
        &model.foreland_basin_index,
        &model.suture_index,
        &model.transpression_index,
        &model.maturity_index,
        &model.shortening_index,
        &model.local_width_km,
    ] {
        hash = hash_f32(hash, values);
    }
    hash
}

pub fn generate_event_driven_orogen_provinces<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    active_history: &TectonicHistoryModel,
    geology: &crate::CrustalModel,
    pre: &PreOrogenicLithosphereModel,
    morphology: &HistoricalMorphologyModel,
    request: &OrogenProvinceRequest,
    planet: PlanetPhysicalParameters,
) -> Result<OrogenProvinceModel, WorldgenError> {
    let mut model = generate_tectonic_orogen_provinces(
        topology,
        tectonics,
        active_history,
        geology,
        pre,
        request,
        planet,
    )?;
    let count = topology.sample_count() as usize;
    let fields = [
        morphology.active_orogen_intensity.len(),
        morphology.fossil_orogen_intensity.len(),
        morphology.suture_intensity.len(),
        morphology.suture_age_myr.len(),
        morphology.shear_intensity.len(),
        morphology.accretion_intensity.len(),
        morphology.cumulative_shortening_km.len(),
    ];
    if fields.iter().any(|length| *length != count) {
        return Err(WorldgenError::InvalidLithosphere(
            "historical morphology does not align with orogen topology",
        ));
    }

    let base_hash = model.metrics.province_hash;
    for sample in 0..count {
        let active = f64::from(morphology.active_orogen_intensity[sample]);
        let fossil = f64::from(morphology.fossil_orogen_intensity[sample]);
        let suture = f64::from(morphology.suture_intensity[sample]);
        let shear = f64::from(morphology.shear_intensity[sample]);
        let accretion = f64::from(morphology.accretion_intensity[sample]);
        let shortening = clamp01(f64::from(morphology.cumulative_shortening_km[sample]) / 1200.0);
        let age = f64::from(morphology.suture_age_myr[sample]);
        let maturity = if age >= 0.0 {
            clamp01(0.25 + age / 280.0)
        } else {
            0.0
        };

        model.orogenic_intensity[sample] = f64::from(model.orogenic_intensity[sample])
            .max(active * 0.76)
            .max(fossil * (0.44 + 0.20 * shortening))
            .clamp(0.0, 1.0) as f32;
        model.suture_index[sample] = f64::from(model.suture_index[sample])
            .max(suture)
            .clamp(0.0, 1.0) as f32;
        model.transpression_index[sample] = f64::from(model.transpression_index[sample])
            .max(shear * (0.38 + 0.42 * fossil))
            .clamp(0.0, 1.0) as f32;

        // Keep broad fossil memory, but reserve explicit high-relief structural fields for the
        // stronger core of the old belt. This avoids turning diffuse event-memory fringes into a
        // huge low-elevation collision province while retaining interior fossil ranges.
        if fossil >= 0.20 {
            let inherited_weakness = f64::from(pre.intrinsic_weakness_index[sample]);
            let core = fossil
                * (0.24 + 0.60 * suture + 0.22 * shear)
                * (0.40 + 0.60 * shortening.max(0.18))
                * (0.72 + 0.28 * inherited_weakness);
            let root = fossil * (0.30 + 0.56 * shortening + 0.24 * suture);
            let plateau = fossil * root * (0.16 + 0.30 * maturity);
            let fold = fossil * (0.20 + 0.50 * shortening + 0.18 * accretion);
            let foreland = fossil * fold * 0.10;

            model.mountain_core_index[sample] = f64::from(model.mountain_core_index[sample])
                .max(core)
                .clamp(0.0, 1.0) as f32;
            model.crustal_root_index[sample] = f64::from(model.crustal_root_index[sample])
                .max(root)
                .clamp(0.0, 1.0) as f32;
            model.plateau_index[sample] = f64::from(model.plateau_index[sample])
                .max(plateau)
                .clamp(0.0, 1.0) as f32;
            model.fold_thrust_index[sample] = f64::from(model.fold_thrust_index[sample])
                .max(fold)
                .clamp(0.0, 1.0) as f32;
            model.foreland_basin_index[sample] = f64::from(model.foreland_basin_index[sample])
                .max(foreland)
                .clamp(0.0, 1.0) as f32;
            model.maturity_index[sample] = f64::from(model.maturity_index[sample])
                .max(maturity * fossil)
                .clamp(0.0, 1.0) as f32;
            model.shortening_index[sample] = f64::from(model.shortening_index[sample])
                .max(shortening * fossil)
                .clamp(0.0, 1.0) as f32;
            model.local_width_km[sample] = f64::from(model.local_width_km[sample])
                .max(65.0 + 330.0 * fossil + 130.0 * suture)
                .min(850.0) as f32;

            if fossil >= 0.28 && model.province_kind[sample] == 0 {
                model.province_kind[sample] = if accretion >= suture.max(shear) && accretion >= 0.30
                {
                    OrogenProvinceKind::TerraneAccretion as u8
                } else if shear > suture && shear >= 0.30 {
                    OrogenProvinceKind::TranspressionalOrogen as u8
                } else {
                    OrogenProvinceKind::ContinentalCollision as u8
                };
            }
        }
    }

    let total_area = (0..topology.sample_count())
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>()
        .max(1.0e-12);
    let orogenic_area = (0..topology.sample_count())
        .filter(|sample| model.orogenic_intensity[*sample as usize] >= 0.10)
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>();
    model.metrics.orogenic_area_fraction = orogenic_area / total_area;
    model.metrics.maximum_orogenic_intensity = model
        .orogenic_intensity
        .iter()
        .map(|value| f64::from(*value))
        .fold(0.0_f64, f64::max);
    model.metrics.maximum_shortening_index = model
        .shortening_index
        .iter()
        .map(|value| f64::from(*value))
        .fold(0.0_f64, f64::max);
    model.metrics.province_hash =
        event_driven_hash(base_hash, morphology.metrics.morphology_hash, &model);
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_historical_tectonic_morphology, build_icosphere, generate_historical_frontend,
        generate_pre_orogenic_lithosphere_from_history, generate_tectonic_history,
        HistoricalLithosphereRequest, PreOrogenicLithosphereRequest, TectonicHistoryRequest,
    };

    #[test]
    fn fossil_orogens_survive_away_from_present_boundaries() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let seed = "historical-orogen-test";
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
        let pre = generate_pre_orogenic_lithosphere_from_history(
            &topology,
            &frontend.tectonics,
            &active_history,
            &frontend.geology,
            &morphology,
            &PreOrogenicLithosphereRequest::new(seed),
        )
        .unwrap();
        let model = generate_event_driven_orogen_provinces(
            &topology,
            &frontend.tectonics,
            &active_history,
            &frontend.geology,
            &pre,
            &morphology,
            &OrogenProvinceRequest::new(seed),
            planet,
        )
        .unwrap();
        let mut boundary_mask = vec![false; topology.sample_count() as usize];
        for boundary in &frontend.tectonics.boundaries {
            boundary_mask[boundary.sample_a as usize] = true;
            boundary_mask[boundary.sample_b as usize] = true;
        }
        let internal_fossil_relief = model
            .orogenic_intensity
            .iter()
            .enumerate()
            .filter(|(sample, value)| **value >= 0.16 && !boundary_mask[*sample])
            .count();
        assert!(internal_fossil_relief > 0);
    }
}
