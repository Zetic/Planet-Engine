use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, CrustKind,
    GeologyRequest, LithosphereRequest, OrogenProvinceKind, PlanetPhysicalParameters,
    TectonicsRequest, TopographyRequest, TOPOGRAPHY_STAGE_VERSION,
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
    let displaced_mountain_core_samples = inherited
        .mountain_core_index
        .iter()
        .zip(inherited.boundary_distance_km.iter())
        .filter(|(core, distance)| **core > 0.20 && **distance >= 180.0)
        .count();
    let runaway_mountain_core_samples = inherited
        .mountain_core_index
        .iter()
        .zip(inherited.boundary_distance_km.iter())
        .filter(|(core, distance)| **core > 0.20 && **distance >= 2200.0)
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
    let mut continental_foreland = 0usize;
    let mut flooded_continental_foreland = 0usize;
    let mut continental_orogen = 0usize;
    let mut flooded_continental_orogen = 0usize;
    let mut kind_samples = [0usize; 7];
    let mut kind_flooded = [0usize; 7];
    let mut kind_intensity_sum = [0.0_f64; 7];
    let mut kind_core_sum = [0.0_f64; 7];
    let mut kind_root_sum = [0.0_f64; 7];
    let mut kind_solid_sum = [0.0_f64; 7];
    let mut kind_flooded_solid_sum = [0.0_f64; 7];
    for sample in 0..terrain.solid_elevation_m.len() {
        let continental = inherited.crust_kind[sample] == CrustKind::Continental as u8;
        if continental && inherited.foreland_basin_index[sample] > 0.20 {
            continental_foreland += 1;
            if terrain.submerged_mask[sample] != 0 {
                flooded_continental_foreland += 1;
            }
        }
        let kind = inherited.province_kind[sample];
        let collision_orogen = kind == OrogenProvinceKind::ContinentalCollision as u8
            || kind == OrogenProvinceKind::CollisionalPlateau as u8
            || kind == OrogenProvinceKind::TerraneAccretion as u8
            || kind == OrogenProvinceKind::TranspressionalOrogen as u8;
        if continental && collision_orogen {
            continental_orogen += 1;
            let kind_index = kind as usize;
            if kind_index < kind_samples.len() {
                kind_samples[kind_index] += 1;
                kind_intensity_sum[kind_index] += f64::from(inherited.orogenic_history[sample]);
                kind_core_sum[kind_index] += f64::from(inherited.mountain_core_index[sample]);
                kind_root_sum[kind_index] += f64::from(inherited.crustal_root_index[sample]);
                kind_solid_sum[kind_index] += terrain.solid_elevation_m[sample];
            }
            if terrain.submerged_mask[sample] != 0 {
                flooded_continental_orogen += 1;
                if kind_index < kind_flooded.len() {
                    kind_flooded[kind_index] += 1;
                    kind_flooded_solid_sum[kind_index] += terrain.solid_elevation_m[sample];
                }
            }
        }
    }

    println!(
        "WG-4 orogen topology: stage=v{} provinces={} active_samples={} core={} displaced_core={} runaway_core={} relief(+/-)={}/{} foreland_flood={}/{} orogen_flood={}/{} solid={:.0}..{:.0}m clamped={} land={:.1}% province_hash={} topo_hash={}",
        terrain.stage.version,
        lithosphere.orogen_provinces.metrics.province_count,
        active_samples,
        mountain_core_samples,
        displaced_mountain_core_samples,
        runaway_mountain_core_samples,
        positive_orogen,
        negative_orogen,
        flooded_continental_foreland,
        continental_foreland,
        flooded_continental_orogen,
        continental_orogen,
        terrain.metrics.minimum_solid_elevation_m,
        terrain.metrics.maximum_solid_elevation_m,
        terrain.metrics.clamped_sample_count,
        terrain.metrics.land_area_fraction * 100.0,
        inherited.orogen_province_hash_hex(),
        terrain.metrics.topography_hash_hex(),
    );
    for kind in [
        OrogenProvinceKind::ContinentalCollision as usize,
        OrogenProvinceKind::CollisionalPlateau as usize,
        OrogenProvinceKind::TerraneAccretion as usize,
        OrogenProvinceKind::TranspressionalOrogen as usize,
    ] {
        let count = kind_samples[kind].max(1) as f64;
        let flooded = kind_flooded[kind].max(1) as f64;
        println!(
            "WG-4 orogen kind={} flood={}/{} ({:.1}%) mean_intensity={:.3} mean_core={:.3} mean_root={:.3} mean_solid={:.0}m flooded_mean_solid={:.0}m",
            kind,
            kind_flooded[kind],
            kind_samples[kind],
            kind_flooded[kind] as f64 / count * 100.0,
            kind_intensity_sum[kind] / count,
            kind_core_sum[kind] / count,
            kind_root_sum[kind] / count,
            kind_solid_sum[kind] / count,
            kind_flooded_solid_sum[kind] / flooded,
        );
    }

    if terrain.stage.version != TOPOGRAPHY_STAGE_VERSION || terrain.stage.version != 15 {
        return Err("WG-4 did not route through historical-material topography".to_string());
    }
    if active_samples == 0
        || mountain_core_samples == 0
        || displaced_mountain_core_samples == 0
        || runaway_mountain_core_samples != 0
        || positive_orogen == 0
    {
        return Err(
            "tectonic province topography did not exercise structural-strand relief".to_string(),
        );
    }
    if continental_foreland > 0 && flooded_continental_foreland * 20 > continental_foreland {
        return Err(format!(
            "continental foreland flooding is still systematic: {}/{}",
            flooded_continental_foreland, continental_foreland
        ));
    }
    if continental_orogen > 0 && flooded_continental_orogen * 10 > continental_orogen {
        return Err(format!(
            "continental orogen flooding is still excessive: {}/{}",
            flooded_continental_orogen, continental_orogen
        ));
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
