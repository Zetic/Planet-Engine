use crate::{
    derive_stage_seed, geology, historical_epochs, historical_frontend, historical_lithosphere,
    tectonics, CrustalModel, GeologyRequest, HistoricalLithosphereModel,
    HistoricalLithosphereRequest, PlanetPhysicalParameters, PlanetTopology, TectonicModel,
    TectonicsRequest, WorldgenError,
};
use std::cell::Cell;

const ANCESTRAL_TECTONICS_NAMESPACE: &str =
    "worldgen:geology:historical-lithosphere:ancestral:v1";

thread_local! {
    // Historical generation reuses the deterministic WG-2 partition builder for ancestral plates.
    // Public WG-2 calls enter through this adapter; nested calls made while historical material is
    // being assembled must resolve to the raw partition builder rather than recurse into history.
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

/// Public historical-material authority.
///
/// The base material partition is assembled while WG-2 is forced into its raw ancestral mode,
/// then a bounded deterministic epoch pass creates persistent parent/child fragment lineage before
/// modern tectonic and crust compatibility projections consume the state.
pub fn generate_historical_lithosphere<T: PlanetTopology>(
    topology: &T,
    request: &HistoricalLithosphereRequest,
    parameters: PlanetPhysicalParameters,
) -> Result<HistoricalLithosphereModel, WorldgenError> {
    let base = with_raw_tectonics_scope(|| {
        historical_lithosphere::generate_historical_lithosphere(topology, request, parameters)
    })?;
    historical_epochs::evolve_historical_lithosphere(topology, base, request.seed.as_str())
}

/// Public WG-2 authority after the historical-lithosphere cutover.
///
/// The existing spherical partition/motion solver remains the ancestral-domain primitive. Public
/// callers now receive the modern kinematic projection produced after persistent material ancestry,
/// bounded fragment epochs, and deterministic consolidation have been established.
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

/// Public WG-3 authority after the historical-lithosphere cutover.
///
/// The legacy global craton-affinity implementation remains available inside `geology.rs` for
/// targeted historical regression tests, but normal engine callers no longer use it as the crust
/// authority. WG-3 deterministically reconstructs the same historical material state that produced
/// the supplied modern tectonic model and projects that material into the compatibility
/// `CrustalModel` consumed by WG-3.5+.
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

/// Explicit legacy WG-3 entrypoint retained only for source-level comparison and migration tests.
/// Product pipelines should call `generate_crust_and_history`, which is historical-material based.
pub fn generate_legacy_crust_and_history<T: PlanetTopology>(
    topology: &T,
    tectonics_model: &TectonicModel,
    request: &GeologyRequest,
    parameters: PlanetPhysicalParameters,
) -> Result<CrustalModel, WorldgenError> {
    geology::generate_crust_and_history(topology, tectonics_model, request, parameters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_icosphere;

    #[test]
    fn ancestral_request_encoding_resolves_to_raw_partition() {
        let seed = "ancestral-detection";
        let derived = derive_stage_seed(seed, ANCESTRAL_TECTONICS_NAMESPACE);
        assert!(is_ancestral_partition_request(&format!("{seed}:{derived:016x}")));
        assert!(!is_ancestral_partition_request(seed));
        assert!(!is_ancestral_partition_request(&format!("{seed}:0000000000000000")));
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
        assert!(model.events.iter().all(|event| {
            event.epoch < historical_epochs::HISTORICAL_EPOCH_COUNT
        }));
    }

    #[test]
    fn public_wg2_and_wg3_are_the_historical_frontend_projection() {
        let topology = build_icosphere(3).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let seed = "historical-public-cutover";
        let tectonics = generate_tectonics(
            &topology,
            &TectonicsRequest::new(seed, 10),
            planet,
        )
        .unwrap();
        let geology = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new(seed),
            planet,
        )
        .unwrap();
        let frontend = historical_frontend::generate_historical_frontend(
            &topology,
            &HistoricalLithosphereRequest::new(seed, 10),
            planet,
        )
        .unwrap();
        assert_eq!(tectonics.metrics.tectonic_hash, frontend.tectonics.metrics.tectonic_hash);
        assert_eq!(tectonics.plate_ids, frontend.historical.current_plate_ids);
        assert_eq!(geology.metrics.geology_hash, frontend.geology.metrics.geology_hash);
        assert_eq!(geology.crust_kind, frontend.historical.crust_kind);
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
