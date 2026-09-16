use crate::{
    build_historical_tectonic_morphology, generate_event_driven_orogen_provinces,
    generate_pre_orogenic_lithosphere_from_history, HistoricalLithosphereModel,
    LithosphereRequest, LithosphericModel, OrogenProvinceRequest, PlanetPhysicalParameters,
    PlanetTopology, PreOrogenicLithosphereRequest, TectonicModel, WorldgenError,
};

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

    // The old constructor remains the accepted mechanical/active-boundary compatibility kernel.
    // Call it directly through its module so the public `generate_lithosphere` adapter can become
    // history-aware without recursing back into this function.
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
        assert!(historical_causal.pre_orogenic.metrics.paleo_suture_sample_count > 0);
        assert!(historical_causal.pre_orogenic.metrics.inherited_rift_sample_count > 0);
    }
}
