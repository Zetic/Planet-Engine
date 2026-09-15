use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, GeologyRequest,
    LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,
    TOPOGRAPHY_STAGE_VERSION,
};

fn main() -> Result<(), String> {
    let seed = "interlink-wg7c";
    let coarse_level = 5;
    let fine_level = 7;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;
    let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, 16), planet)
        .map_err(|error| error.to_string())?;
    let geology =
        generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet)
            .map_err(|error| error.to_string())?;
    let lithosphere = generate_lithosphere(
        &coarse,
        &tectonics,
        &geology,
        &LithosphereRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;
    let inherited = inherit_physical_state(
        &fine,
        coarse_level,
        &tectonics,
        &geology,
        &lithosphere,
        planet,
    )
    .map_err(|error| error.to_string())?;
    let boundaries =
        inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids)
            .map_err(|error| error.to_string())?;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;

    let active_samples = inherited
        .orogenic_history
        .iter()
        .filter(|value| **value > 0.05)
        .count();
    let mountain_core_samples = inherited
        .mountain_core_index
        .iter()
        .filter(|value| **value > 0.20)
        .count();
    let far_mountain_core_samples = inherited
        .mountain_core_index
        .iter()
        .zip(inherited.boundary_distance_km.iter())
        .filter(|(core, distance)| **core > 0.20 && **distance >= 800.0)
        .count();
    let positive_orogen = terrain
        .orogenic_elevation_m
        .iter()
        .filter(|value| **value > 250.0)
        .count();
    let negative_orogen = terrain
        .orogenic_elevation_m
        .iter()
        .filter(|value| **value < -100.0)
        .count();

    println!(
        "WG-4 boundary-localized topography: stage=v{} provinces={} active_samples={} core={} far_core={} relief(+/-)={}/{} solid={:.0}..{:.0}m clamped={} land={:.1}% province_hash={} topo_hash={}",
        terrain.stage.version,
        lithosphere.orogen_provinces.metrics.province_count,
        active_samples,
        mountain_core_samples,
        far_mountain_core_samples,
        positive_orogen,
        negative_orogen,
        terrain.metrics.minimum_solid_elevation_m,
        terrain.metrics.maximum_solid_elevation_m,
        terrain.metrics.clamped_sample_count,
        terrain.metrics.land_area_fraction * 100.0,
        inherited.orogen_province_hash_hex(),
        terrain.metrics.topography_hash_hex(),
    );

    if terrain.stage.version != TOPOGRAPHY_STAGE_VERSION || terrain.stage.version != 13 {
        return Err("WG-4 did not route through tectonic-province topography".to_string());
    }
    if active_samples == 0
        || mountain_core_samples == 0
        || far_mountain_core_samples != 0
        || positive_orogen == 0
        || negative_orogen == 0
    {
        return Err("tectonic province topography did not exercise zoned relief".to_string());
    }
    if terrain
        .solid_elevation_m
        .iter()
        .any(|value| !value.is_finite())
    {
        return Err("tectonic province topography produced non-finite relief".to_string());
    }
    Ok(())
}
