use crate::{
    derive_stage_seed, forward_plate_evolution, historical_causal, historical_epochs,
    historical_frontend, historical_lithosphere, tectonics, CrustalModel, GeologyRequest,
    HistoricalLithosphereModel, HistoricalLithosphereRequest, LithosphereRequest,
    LithosphericModel, PlanetPhysicalParameters, PlanetTopology, TectonicModel, TectonicsRequest,
    WorldgenError,
};
use std::cell::Cell;

const ANCESTRAL_TECTONICS_NAMESPACE: &str = "worldgen:geology:historical-lithosphere:ancestral:v1";

thread_local! {
    static RAW_TECTONICS_DEPTH: Cell<u32> = const { Cell::new(0) };
}

fn with_raw_tectonics_scope<R>(operation: impl FnOnce() -> R) -> R {
    RAW_TECTONICS_DEPTH.with(|depth| {
        let previous = depth.get();
        depth.set(previous.saturating_add(1));
        struct Restore<'a> {
            depth: &'a Cell<u32>,
            previous: u32,
        }
        impl Drop for Restore<'_> {
            fn drop(&mut self) {
                self.depth.set(self.previous);
            }
        }
        let _restore = Restore { depth, previous };
        operation()
    })
}

fn raw_tectonics_requested() -> bool {
    RAW_TECTONICS_DEPTH.with(|depth| depth.get() > 0)
}

fn is_ancestral_partition_request(seed: &str) -> bool {
    let Some((base_seed, encoded)) = seed.rsplit_once(':') else {
        return false;
    };
    if encoded.len() != 16 || !encoded.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return false;
    }
    let Ok(encoded_seed) = u64::from_str_radix(encoded, 16) else {
        return false;
    };
    encoded_seed == derive_stage_seed(base_seed, ANCESTRAL_TECTONICS_NAMESPACE)
}

pub fn generate_historical_lithosphere<T: PlanetTopology>(
    topology: &T,
    request: &HistoricalLithosphereRequest,
    parameters: PlanetPhysicalParameters,
) -> Result<HistoricalLithosphereModel, WorldgenError> {
    let base = with_raw_tectonics_scope(|| {
        historical_lithosphere::generate_historical_lithosphere(topology, request, parameters)
    })?;
    let lineage =
        historical_epochs::evolve_historical_lithosphere(topology, base, request.seed.as_str())?;
    forward_plate_evolution::evolve_modern_plate_geometry(
        topology,
        lineage,
        request.seed.as_str(),
        parameters,
    )
}

pub fn generate_tectonics<T: PlanetTopology>(
    topology: &T,
    request: &TectonicsRequest,
    parameters: PlanetPhysicalParameters,
) -> Result<TectonicModel, WorldgenError> {
    if raw_tectonics_requested() || is_ancestral_partition_request(request.seed.as_str()) {
        return tectonics::generate_tectonics(topology, request, parameters);
    }

    let historical = generate_historical_lithosphere(
        topology,
        &HistoricalLithosphereRequest::new(request.seed.as_str(), request.plate_count),
        parameters,
    )?;
    historical_frontend::project_historical_modern_tectonics(
        topology,
        &historical,
        request.seed.as_str(),
        parameters,
    )
}

pub fn generate_crust_and_history<T: PlanetTopology>(
    topology: &T,
    tectonics_model: &TectonicModel,
    request: &GeologyRequest,
    parameters: PlanetPhysicalParameters,
) -> Result<CrustalModel, WorldgenError> {
    if request.seed.trim().is_empty() {
        return Err(WorldgenError::InvalidGeology(
            "geology seed must not be empty",
        ));
    }
    parameters
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;

    let historical = generate_historical_lithosphere(
        topology,
        &HistoricalLithosphereRequest::new(
            request.seed.as_str(),
            tectonics_model.metrics.plate_count,
        ),
        parameters,
    )?;
    let projected_tectonics = historical_frontend::project_historical_modern_tectonics(
        topology,
        &historical,
        request.seed.as_str(),
        parameters,
    )?;

    if projected_tectonics.metrics.tectonic_hash != tectonics_model.metrics.tectonic_hash
        || projected_tectonics.plate_ids != tectonics_model.plate_ids
    {
        return Err(WorldgenError::InvalidGeology(
            "WG-3 historical material does not match supplied modern tectonic ancestry",
        ));
    }

    historical_frontend::project_historical_crust(
        topology,
        &historical,
        tectonics_model,
        request.seed.as_str(),
    )
}

/// Public WG-3.5/WG-3.6 authority after the material-history morphology cutover.
///
/// Compatibility callers that only retain modern tectonics/geology deterministically reconstruct
/// the same PR-A historical material model, verify both projections, then route through the
/// history-aware lithosphere/orogen implementation. Callers already holding `HistoricalFrontend`
/// should use `generate_lithosphere_from_history` directly to avoid this reconstruction.
pub fn generate_lithosphere<T: PlanetTopology>(
    topology: &T,
    tectonics_model: &TectonicModel,
    geology_model: &CrustalModel,
    request: &LithosphereRequest,
) -> Result<LithosphericModel, WorldgenError> {
    if request.seed.trim().is_empty() {
        return Err(WorldgenError::InvalidLithosphere(
            "lithosphere seed must not be empty",
        ));
    }
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let historical = generate_historical_lithosphere(
        topology,
        &HistoricalLithosphereRequest::new(
            request.seed.as_str(),
            tectonics_model.metrics.plate_count,
        ),
        planet,
    )?;
    let projected_tectonics = historical_frontend::project_historical_modern_tectonics(
        topology,
        &historical,
        request.seed.as_str(),
        planet,
    )?;
    if projected_tectonics.metrics.tectonic_hash != tectonics_model.metrics.tectonic_hash
        || projected_tectonics.plate_ids != tectonics_model.plate_ids
    {
        return Err(WorldgenError::InvalidLithosphere(
            "WG-3.5 historical material does not match supplied modern tectonic ancestry",
        ));
    }
    let projected_geology = historical_frontend::project_historical_crust(
        topology,
        &historical,
        tectonics_model,
        request.seed.as_str(),
    )?;
    if projected_geology.metrics.geology_hash != geology_model.metrics.geology_hash
        || projected_geology.crust_kind != geology_model.crust_kind
        || projected_geology.crust_province_id != geology_model.crust_province_id
    {
        return Err(WorldgenError::InvalidLithosphere(
            "WG-3.5 historical material does not match supplied WG-3 crust projection",
        ));
    }
    historical_causal::generate_lithosphere_from_history(
        topology,
        &historical,
        tectonics_model,
        geology_model,
        request,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_icosphere;

    #[test]
    fn ancestral_request_encoding_resolves_to_raw_partition() {
        let seed = "ancestral-detection";
        let derived = derive_stage_seed(seed, ANCESTRAL_TECTONICS_NAMESPACE);
        assert!(is_ancestral_partition_request(&format!(
            "{seed}:{derived:016x}"
        )));
        assert!(!is_ancestral_partition_request(seed));
        assert!(!is_ancestral_partition_request(&format!(
            "{seed}:0000000000000000"
        )));
    }

    #[test]
    fn public_historical_authority_contains_parented_fragment_lineage() {
        let topology = build_icosphere(3).unwrap();
        let model = generate_historical_lithosphere(
            &topology,
            &HistoricalLithosphereRequest::new("historical-public-lineage", 10),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
        assert!(model
            .fragments
            .iter()
            .any(|fragment| fragment.parent_fragment_id.is_some()));
        assert!(model
            .events
            .iter()
            .all(|event| { event.epoch < historical_epochs::HISTORICAL_EPOCH_COUNT }));
    }

    #[test]
    fn public_wg2_and_wg3_are_the_historical_frontend_projection() {
        let topology = build_icosphere(3).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let seed = "historical-public-cutover";
        let tectonics =
            generate_tectonics(&topology, &TectonicsRequest::new(seed, 10), planet).unwrap();
        let geology =
            generate_crust_and_history(&topology, &tectonics, &GeologyRequest::new(seed), planet)
                .unwrap();
        let frontend = historical_frontend::generate_historical_frontend(
            &topology,
            &HistoricalLithosphereRequest::new(seed, 10),
            planet,
        )
        .unwrap();
        assert_eq!(
            tectonics.metrics.tectonic_hash,
            frontend.tectonics.metrics.tectonic_hash
        );
        assert_eq!(tectonics.plate_ids, frontend.historical.current_plate_ids);
        assert_eq!(
            geology.metrics.geology_hash,
            frontend.geology.metrics.geology_hash
        );
        assert_eq!(geology.crust_kind, frontend.historical.crust_kind);
    }

    #[test]
    fn public_lithosphere_consumes_historical_morphology() {
        let topology = build_icosphere(3).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let seed = "historical-public-morphology";
        let tectonics =
            generate_tectonics(&topology, &TectonicsRequest::new(seed, 10), planet).unwrap();
        let geology =
            generate_crust_and_history(&topology, &tectonics, &GeologyRequest::new(seed), planet)
                .unwrap();
        let lithosphere = generate_lithosphere(
            &topology,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed),
        )
        .unwrap();
        assert!(lithosphere.pre_orogenic.metrics.paleo_suture_sample_count > 0);
        assert!(lithosphere.pre_orogenic.metrics.inherited_rift_sample_count > 0);
    }

    #[test]
    fn mismatched_modern_tectonics_are_rejected_by_historical_wg3() {
        let topology = build_icosphere(3).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let tectonics = generate_tectonics(
            &topology,
            &TectonicsRequest::new("historical-a", 10),
            planet,
        )
        .unwrap();
        let error = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new("historical-b"),
            planet,
        )
        .unwrap_err();
        assert!(matches!(error, WorldgenError::InvalidGeology(_)));
    }
}
