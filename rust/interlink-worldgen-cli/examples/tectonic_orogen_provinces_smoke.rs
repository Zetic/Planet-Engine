use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_pre_orogenic_lithosphere,
    generate_tectonic_history, generate_tectonic_orogen_provinces, generate_tectonics,
    GeologyRequest, OrogenProvinceRequest, PlanetPhysicalParameters, PreOrogenicLithosphereRequest,
    TectonicHistoryRequest, TectonicsRequest,
};

fn main() -> Result<(), String> {
    let topology = build_icosphere(5).map_err(|error| error.to_string())?;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let seed = "wg36-orogen-provinces-smoke";
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
    let orogens = generate_tectonic_orogen_provinces(
        &topology,
        &tectonics,
        &history,
        &geology,
        &pre,
        &OrogenProvinceRequest::new(seed),
        planet,
    )
    .map_err(|error| error.to_string())?;

    let mountain_core_samples = orogens
        .mountain_core_index
        .iter()
        .filter(|value| **value > 0.20)
        .count();
    let far_core_samples = orogens
        .mountain_core_index
        .iter()
        .zip(orogens.boundary_distance_km.iter())
        .filter(|(core, distance)| **core > 0.20 && **distance >= 800.0)
        .count();

    println!(
        "WG-3.6 tectonic orogens: provinces={} collision={} plateau={} cordilleran={} island_arc={} terrane={} transpressional={} coverage={:.1}% width={:.0}/{:.0}/{:.0}km core={} far_core={} max_shortening={:.3} max_intensity={:.3} hash={}",
        orogens.metrics.province_count,
        orogens.metrics.continental_collision_count,
        orogens.metrics.collisional_plateau_count,
        orogens.metrics.cordilleran_arc_count,
        orogens.metrics.island_arc_count,
        orogens.metrics.terrane_accretion_count,
        orogens.metrics.transpressional_count,
        orogens.metrics.orogenic_area_fraction * 100.0,
        orogens.metrics.minimum_source_width_km,
        orogens.metrics.mean_source_width_km,
        orogens.metrics.maximum_source_width_km,
        mountain_core_samples,
        far_core_samples,
        orogens.metrics.maximum_shortening_index,
        orogens.metrics.maximum_orogenic_intensity,
        orogens.metrics.province_hash_hex(),
    );

    if mountain_core_samples == 0 || far_core_samples != 0 {
        return Err("mountain-core localization invariant failed".to_string());
    }
    if orogens.provinces.is_empty() {
        return Err("tectonic orogen stage produced no convergent provinces".to_string());
    }
    if orogens
        .orogenic_intensity
        .iter()
        .any(|value| !value.is_finite())
    {
        return Err("tectonic orogen stage produced non-finite state".to_string());
    }
    Ok(())
}
