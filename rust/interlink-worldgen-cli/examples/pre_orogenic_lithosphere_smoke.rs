use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_pre_orogenic_lithosphere,
    generate_tectonic_history, generate_tectonics, GeologyRequest, PlanetPhysicalParameters,
    PlanetTopology, PreOrogenicLithosphereRequest, TectonicHistoryRequest, TectonicsRequest,
};

fn main() -> Result<(), String> {
    let topology = build_icosphere(5).map_err(|error| error.to_string())?;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let seed = "wg35-pre-orogenic-smoke";
    let tectonics = generate_tectonics(&topology, &TectonicsRequest::new(seed, 16), planet)
        .map_err(|error| error.to_string())?;
    let history = generate_tectonic_history(
        &topology,
        &tectonics,
        &TectonicHistoryRequest::new(seed),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let geology =
        generate_crust_and_history(&topology, &tectonics, &GeologyRequest::new(seed), planet)
            .map_err(|error| error.to_string())?;
    let pre = generate_pre_orogenic_lithosphere(
        &topology,
        &tectonics,
        &history,
        &geology,
        &PreOrogenicLithosphereRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;

    println!(
        "WG-3.5 pre-orogenic lithosphere: strength={:.3} weakness={:.3} Te={:.1}km thermal={:.3} fabric={:.3} fragments={} microplates={} terranes={} contacts={} fragmented={:.2}% hash={}",
        pre.metrics.mean_intrinsic_strength_index,
        pre.metrics.mean_intrinsic_weakness_index,
        pre.metrics.mean_effective_elastic_thickness_km,
        pre.metrics.mean_thermal_state_index,
        pre.metrics.mean_inherited_fabric_strength,
        pre.metrics.fragment_count,
        pre.metrics.microplate_count,
        pre.metrics.terrane_count,
        pre.metrics.contact_count,
        pre.metrics.fragmented_area_fraction * 100.0,
        pre.metrics.pre_orogenic_hash_hex(),
    );

    if pre.intrinsic_strength_index.len() != topology.sample_count() as usize {
        return Err("pre-orogenic lithosphere did not cover the topology".to_string());
    }
    if pre
        .effective_elastic_thickness_km
        .iter()
        .any(|value| !value.is_finite())
    {
        return Err("pre-orogenic lithosphere produced invalid mechanics".to_string());
    }
    Ok(())
}
