use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, generate_initial_topography,
    generate_lithosphere_from_history, inherit_boundary_interfaces, inherit_physical_state,
    HistoricalLithosphereRequest, LithosphereRequest, PlanetPhysicalParameters, TopographyRequest,
};

fn main() -> Result<(), String> {
    let seed = "interlink-wg7c";
    let coarse_level = 6;
    let fine_level = 8;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;
    let frontend = generate_historical_frontend(
        &coarse,
        &HistoricalLithosphereRequest::new(seed, 16),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let lithosphere = generate_lithosphere_from_history(
        &coarse,
        &frontend.historical,
        &frontend.tectonics,
        &frontend.geology,
        &LithosphereRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;
    let inherited = inherit_physical_state(
        &fine,
        coarse_level,
        &frontend.tectonics,
        &frontend.geology,
        &lithosphere,
        planet,
    )
    .map_err(|error| error.to_string())?;
    let boundaries = inherit_boundary_interfaces(
        &coarse,
        &fine,
        &frontend.tectonics,
        &frontend.geology,
        &inherited.plate_ids,
    )
    .map_err(|error| error.to_string())?;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;

    let mut land_area = 0.0_f64;
    let mut above_1km = 0.0_f64;
    let mut above_2km = 0.0_f64;
    let mut above_3km = 0.0_f64;
    let mut above_4km = 0.0_f64;
    for sample in 0..terrain.solid_elevation_m.len() {
        if terrain.submerged_mask[sample] != 0 {
            continue;
        }
        let area = fine.dual_area_steradians()[sample];
        let elevation = f64::from(terrain.elevation_above_sea_level_m[sample]);
        land_area += area;
        above_1km += area * f64::from(elevation >= 1_000.0);
        above_2km += area * f64::from(elevation >= 2_000.0);
        above_3km += area * f64::from(elevation >= 3_000.0);
        above_4km += area * f64::from(elevation >= 4_000.0);
    }

    println!(
        "hypsometry-highres seed={seed} L{coarse_level}->L{fine_level} samples={} land={:.1}% mean-land={:.0}m ocean-depth={:.0}m solid-p05/p50/p95={:.0}/{:.0}/{:.0}m range={:.0}..{:.0}m land>1/2/3/4km={:.1}/{:.1}/{:.1}/{:.1}% closure={:.3e}",
        terrain.metrics.sample_count,
        terrain.metrics.land_area_fraction * 100.0,
        terrain.metrics.mean_land_elevation_m,
        terrain.metrics.mean_water_depth_m,
        terrain.metrics.p05_solid_elevation_m,
        terrain.metrics.median_solid_elevation_m,
        terrain.metrics.p95_solid_elevation_m,
        terrain.metrics.minimum_solid_elevation_m,
        terrain.metrics.maximum_solid_elevation_m,
        above_1km / land_area.max(1.0e-12) * 100.0,
        above_2km / land_area.max(1.0e-12) * 100.0,
        above_3km / land_area.max(1.0e-12) * 100.0,
        above_4km / land_area.max(1.0e-12) * 100.0,
        terrain.metrics.water_volume_relative_error,
    );
    Ok(())
}
