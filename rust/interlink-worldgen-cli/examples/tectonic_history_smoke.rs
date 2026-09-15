use interlink_worldgen::{
    build_icosphere, generate_tectonic_history, generate_tectonics, PlanetPhysicalParameters,
    TectonicHistoryRequest, TectonicsRequest,
};

fn main() -> Result<(), String> {
    let topology = build_icosphere(5).map_err(|error| error.to_string())?;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let seed = "wg25-history-smoke";
    let tectonics = generate_tectonics(&topology, &TectonicsRequest::new(seed, 16), planet)
        .map_err(|error| error.to_string())?;
    let history = generate_tectonic_history(
        &topology,
        &tectonics,
        &TectonicHistoryRequest::new(seed),
        planet,
    )
    .map_err(|error| error.to_string())?;

    println!(
        "WG-2.5 tectonic history: systems={} convergent={} divergent={} transform={} max_age={:.2}Myr max_convergence={:.1}km max_extension={:.1}km mean_obliquity={:.2}deg hash={}",
        history.metrics.boundary_system_count,
        history.metrics.convergent_system_count,
        history.metrics.divergent_system_count,
        history.metrics.transform_system_count,
        history.metrics.maximum_event_age_myr,
        history.metrics.maximum_cumulative_convergence_km,
        history.metrics.maximum_cumulative_extension_km,
        history.metrics.mean_obliquity_deg,
        history.metrics.history_hash_hex(),
    );

    if history.boundary_state.len() != tectonics.boundaries.len() {
        return Err("tectonic history did not cover every tectonic boundary".to_string());
    }
    if history.metrics.boundary_system_count == 0 {
        return Err("tectonic history produced no connected boundary systems".to_string());
    }
    if history
        .boundary_state
        .iter()
        .any(|state| !state.event_age_myr.is_finite() || state.event_age_myr <= 0.0)
    {
        return Err("tectonic history produced invalid chronology".to_string());
    }
    Ok(())
}
