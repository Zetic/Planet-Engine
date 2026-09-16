use crate::{
    geology, historical_frontend, historical_lithosphere, tectonics, CrustalModel, GeologyRequest,
    HistoricalLithosphereRequest, PlanetPhysicalParameters, PlanetTopology, TectonicModel,
    TectonicsRequest, WorldgenError,
};
use std::cell::Cell;

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

/// Public WG-2 authority after the historical-lithosphere cutover.
///
/// The existing spherical partition/motion solver remains the ancestral-domain primitive. Public
/// callers now receive the modern kinematic projection produced after persistent material ancestry
/// and deterministic consolidation have been established.
pub fn generate_tectonics<T: PlanetTopology>(
    topology: &T,
    request: &TectonicsRequest,
    parameters: PlanetPhysicalParameters,
) -> Result<TectonicModel, WorldgenError> {
    if raw_tectonics_requested() {
        return tectonics::generate_tectonics(topology, request, parameters);
    }

    let historical = with_raw_tectonics_scope(|| {
        historical_lithosphere::generate_historical_lithosphere(
            topology,
            &HistoricalLithosphereRequest::new(request.seed.as_str(), request.plate_count),
            parameters,
        )
    })?;
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

    let historical = with_raw_tectonics_scope(|| {
        historical_lithosphere::generate_historical_lithosphere(
            topology,
            &HistoricalLithosphereRequest::new(
                request.seed.as_str(),
                tectonics_model.metrics.plate_count,
            ),
            parameters,
        )
    })?;
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
        let frontend = with_raw_tectonics_scope(|| {
            historical_frontend::generate_historical_frontend(
                &topology,
                &HistoricalLithosphereRequest::new(seed, 10),
                planet,
            )
        })
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
