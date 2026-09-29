use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, generate_initial_topography,
    generate_lithosphere_from_history, inherit_boundary_interfaces, inherit_physical_state,
    CrustKind, HistoricalLithosphereRequest, InheritedStructureKind, LithosphereRequest,
    PlanetPhysicalParameters, TopographyParameters, TopographyRequest,
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
    let mut parameters = TopographyParameters::default();
    parameters.isostatic_scale = 0.48;
    parameters.collision_uplift_scale_m = 2_200.0;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest {
            seed: seed.to_owned(),
            parameters,
        },
    )
    .map_err(|error| error.to_string())?;

    let mut land_area = 0.0_f64;
    let mut above_1km = 0.0_f64;
    let mut above_2km = 0.0_f64;
    let mut above_3km = 0.0_f64;
    let mut above_4km = 0.0_f64;
    let mut continental_area = 0.0_f64;
    let mut continental_base_iso = 0.0_f64;
    let mut continental_support = 0.0_f64;
    let mut continental_orogen = 0.0_f64;
    let mut continental_rift = 0.0_f64;
    let mut stable_area = 0.0_f64;
    let mut stable_land_area = 0.0_f64;
    let mut stable_land_elevation = 0.0_f64;
    let mut modified_land_area = 0.0_f64;
    let mut modified_land_elevation = 0.0_f64;
    let mut province_area = [0.0_f64; 7];
    let mut province_relief = [0.0_f64; 7];

    for sample in 0..terrain.solid_elevation_m.len() {
        let area = fine.dual_area_steradians()[sample];
        let province = usize::from(inherited.province_kind[sample]).min(6);
        province_area[province] += area;
        province_relief[province] += area * f64::from(terrain.orogenic_elevation_m[sample]);
        let continental = inherited.crust_kind[sample] == CrustKind::Continental as u8;
        let stable = continental
            && inherited.structural_zone_kind[sample]
                != InheritedStructureKind::ContinentalMargin as u8
            && inherited.structural_zone_kind[sample] != InheritedStructureKind::InheritedRift as u8
            && inherited.rift_history[sample] < 0.22
            && inherited.subsidence_history[sample] < 0.28
            && inherited.basin_potential[sample] < 0.32;
        if continental {
            let mantle_density = planet.isostatic_mantle_density_kg_per_m3;
            let thickness_m = f64::from(inherited.crust_thickness_km[sample]) * 1_000.0;
            let crust_density = f64::from(inherited.crust_density_kg_per_m3[sample]);
            let base_iso = thickness_m * (mantle_density - crust_density) / mantle_density
                * parameters.isostatic_scale;
            continental_area += area;
            continental_base_iso += area * base_iso;
            continental_support += area
                * (f64::from(terrain.isostatic_elevation_m[sample]) - base_iso);
            continental_orogen += area * f64::from(terrain.orogenic_elevation_m[sample]);
            continental_rift += area * f64::from(terrain.rift_basin_elevation_m[sample]);
            if stable {
                stable_area += area;
            }
        }

        if terrain.submerged_mask[sample] != 0 {
            continue;
        }
        let elevation = f64::from(terrain.elevation_above_sea_level_m[sample]);
        land_area += area;
        above_1km += area * f64::from(elevation >= 1_000.0);
        above_2km += area * f64::from(elevation >= 2_000.0);
        above_3km += area * f64::from(elevation >= 3_000.0);
        above_4km += area * f64::from(elevation >= 4_000.0);
        if continental {
            if stable {
                stable_land_area += area;
                stable_land_elevation += area * elevation;
            } else {
                modified_land_area += area;
                modified_land_elevation += area * elevation;
            }
        }
    }

    println!(
        "hypsometry-highres seed={seed} variant=iso48-c2200 L{coarse_level}->L{fine_level} samples={} land={:.1}% mean-land={:.0}m ocean-depth={:.0}m solid-p05/p50/p95={:.0}/{:.0}/{:.0}m range={:.0}..{:.0}m land>1/2/3/4km={:.1}/{:.1}/{:.1}/{:.1}% closure={:.3e}",
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
    println!(
        "hypsometry-highres-budget continental={:.1}% stable={:.1}% base-iso={:.0}m support={:.0}m orogen={:.0}m rift-basin={:.0}m stable-land={:.0}m modified-land={:.0}m sea-level={:.0}m",
        continental_area / fine.dual_area_steradians().iter().sum::<f64>() * 100.0,
        stable_area / fine.dual_area_steradians().iter().sum::<f64>() * 100.0,
        continental_base_iso / continental_area.max(1.0e-12),
        continental_support / continental_area.max(1.0e-12),
        continental_orogen / continental_area.max(1.0e-12),
        continental_rift / continental_area.max(1.0e-12),
        stable_land_elevation / stable_land_area.max(1.0e-12),
        modified_land_elevation / modified_land_area.max(1.0e-12),
        terrain.metrics.sea_level_m.unwrap_or(0.0),
    );
    for kind in 0..=6 {
        if province_area[kind] > 0.0 {
            println!(
                "hypsometry-highres-province kind={kind} area={:.1}% relief={:.0}m",
                province_area[kind] / fine.dual_area_steradians().iter().sum::<f64>() * 100.0,
                province_relief[kind] / province_area[kind],
            );
        }
    }
    Ok(())
}
