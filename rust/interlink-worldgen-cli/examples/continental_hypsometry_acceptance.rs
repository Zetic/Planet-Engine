use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, generate_initial_topography,
    generate_lithosphere_from_history, inherit_boundary_interfaces, inherit_physical_state,
    CrustKind, HistoricalLithosphereRequest, InheritedStructureKind, LithosphereRequest,
    PlanetPhysicalParameters, TopographyRequest,
};

#[derive(Default)]
struct AreaBucket {
    total: f64,
    submerged: f64,
    shallow_submerged: f64,
    depth_area_sum: f64,
}

impl AreaBucket {
    fn submerged_fraction(&self) -> f64 {
        self.submerged / self.total.max(1.0e-12)
    }

    fn shallow_fraction_of_submerged(&self) -> f64 {
        self.shallow_submerged / self.submerged.max(1.0e-12)
    }

    fn mean_submerged_depth_m(&self) -> f64 {
        self.depth_area_sum / self.submerged.max(1.0e-12)
    }
}

fn add_sample(bucket: &mut AreaBucket, area: f64, submerged: bool, depth_m: f64) {
    bucket.total += area;
    if submerged {
        bucket.submerged += area;
        bucket.depth_area_sum += area * depth_m;
        if depth_m <= 500.0 {
            bucket.shallow_submerged += area;
        }
    }
}

fn build_world(
    seed: &str,
    coarse_level: u8,
    fine_level: u8,
) -> Result<
    (
        interlink_worldgen::GeodesicTopology,
        interlink_worldgen::InheritedPhysicalState,
        interlink_worldgen::TopographyState,
    ),
    String,
> {
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
    Ok((fine, inherited, terrain))
}

fn verify_seed(seed: &str) -> Result<(), String> {
    let (fine, inherited, terrain) = build_world(seed, 4, 6)?;

    let mut continental = AreaBucket::default();
    let mut stable_continental = AreaBucket::default();
    let mut modified_continental = AreaBucket::default();
    let mut transitional = AreaBucket::default();
    let mut oceanic = AreaBucket::default();
    let mut modified_isostatic_sum = 0.0_f64;
    let mut modified_rift_basin_sum = 0.0_f64;
    let mut modified_orogen_sum = 0.0_f64;
    let mut modified_mantle_sum = 0.0_f64;
    let total_area = fine.dual_area_steradians().iter().sum::<f64>();
    let mut land_area = 0.0_f64;
    let mut land_above_2km = 0.0_f64;
    let mut land_above_3km = 0.0_f64;

    for sample in 0..terrain.solid_elevation_m.len() {
        let area = fine.dual_area_steradians()[sample];
        let submerged = terrain.submerged_mask[sample] != 0;
        let depth_m = f64::from(terrain.water_depth_m[sample]);
        if !submerged {
            let elevation = f64::from(terrain.elevation_above_sea_level_m[sample]);
            land_area += area;
            land_above_2km += area * f64::from(elevation >= 2_000.0);
            land_above_3km += area * f64::from(elevation >= 3_000.0);
        }

        match inherited.crust_kind[sample] {
            value if value == CrustKind::Continental as u8 => {
                add_sample(&mut continental, area, submerged, depth_m);
                let stable = inherited.structural_zone_kind[sample]
                    != InheritedStructureKind::ContinentalMargin as u8
                    && inherited.structural_zone_kind[sample]
                        != InheritedStructureKind::InheritedRift as u8
                    && inherited.rift_history[sample] < 0.22
                    && inherited.subsidence_history[sample] < 0.28
                    && inherited.basin_potential[sample] < 0.32;
                if stable {
                    add_sample(&mut stable_continental, area, submerged, depth_m);
                } else {
                    add_sample(&mut modified_continental, area, submerged, depth_m);
                    modified_isostatic_sum +=
                        area * f64::from(terrain.isostatic_elevation_m[sample]);
                    modified_rift_basin_sum +=
                        area * f64::from(terrain.rift_basin_elevation_m[sample]);
                    modified_orogen_sum +=
                        area * f64::from(terrain.orogenic_elevation_m[sample]);
                    modified_mantle_sum +=
                        area * f64::from(terrain.mantle_dynamic_elevation_m[sample]);
                }
            }
            value if value == CrustKind::Transitional as u8 => {
                add_sample(&mut transitional, area, submerged, depth_m);
            }
            _ => add_sample(&mut oceanic, area, submerged, depth_m),
        }
    }

    let highland_2km = land_above_2km / land_area.max(1.0e-12);
    let highland_3km = land_above_3km / land_area.max(1.0e-12);
    println!(
        "forward-hypsometry seed={seed} land={:.1}% mean-land={:.0}m ocean-depth={:.0}m solid-p95={:.0}m highland2/3={:.1}/{:.1}% continental-submerged={:.1}% stable={:.1}% modified={:.1}% transitional={:.1}% oceanic={:.1}% shallow-continental={:.1}% continental-depth={:.0}m",
        terrain.metrics.land_area_fraction * 100.0,
        terrain.metrics.mean_land_elevation_m,
        terrain.metrics.mean_water_depth_m,
        terrain.metrics.p95_solid_elevation_m,
        highland_2km * 100.0,
        highland_3km * 100.0,
        continental.submerged_fraction() * 100.0,
        stable_continental.submerged_fraction() * 100.0,
        modified_continental.submerged_fraction() * 100.0,
        transitional.submerged_fraction() * 100.0,
        oceanic.submerged_fraction() * 100.0,
        continental.shallow_fraction_of_submerged() * 100.0,
        continental.mean_submerged_depth_m(),
    );
    let modified_area = modified_continental.total.max(1.0e-12);
    println!(
        "forward-hypsometry-components seed={seed} modified(isostatic/rift/orogen/mantle)={:.0}/{:.0}/{:.0}/{:.0}m",
        modified_isostatic_sum / modified_area,
        modified_rift_basin_sum / modified_area,
        modified_orogen_sum / modified_area,
        modified_mantle_sum / modified_area,
    );

    if terrain.metrics.water_volume_relative_error.abs() > 1.0e-9 {
        return Err(format!(
            "{seed}: hydrostatic water closure regressed: {:.3e}",
            terrain.metrics.water_volume_relative_error
        ));
    }
    if continental.total <= 0.0 || stable_continental.total <= 0.0 || transitional.total <= 0.0 {
        return Err(format!(
            "{seed}: forward hypsometry diagnostic lacked required crust classes"
        ));
    }
    if !(0.28..=0.42).contains(&terrain.metrics.land_area_fraction) {
        return Err(format!(
            "{seed}: L4->L6 emergent land escaped the forward calibration envelope: {:.1}%",
            terrain.metrics.land_area_fraction * 100.0
        ));
    }
    if !(800.0..=1_900.0).contains(&terrain.metrics.mean_land_elevation_m) {
        return Err(format!(
            "{seed}: mean land elevation escaped the forward calibration envelope: {:.0} m",
            terrain.metrics.mean_land_elevation_m
        ));
    }
    if !(3_700.0..=4_500.0).contains(&terrain.metrics.mean_water_depth_m) {
        return Err(format!(
            "{seed}: mean ocean depth escaped the forward calibration envelope: {:.0} m",
            terrain.metrics.mean_water_depth_m
        ));
    }
    if !(3_500.0..=5_500.0).contains(&terrain.metrics.p95_solid_elevation_m) {
        return Err(format!(
            "{seed}: solid-elevation p95 escaped the forward calibration envelope: {:.0} m",
            terrain.metrics.p95_solid_elevation_m
        ));
    }
    if highland_2km > 0.40 || highland_3km > 0.20 {
        return Err(format!(
            "{seed}: continental highlands became too spatially broad: {:.1}% above 2 km, {:.1}% above 3 km",
            highland_2km * 100.0,
            highland_3km * 100.0
        ));
    }
    if stable_continental.submerged_fraction() > 0.15 {
        return Err(format!(
            "{seed}: stable continental interiors are systematically drowned at {:.1}%",
            stable_continental.submerged_fraction() * 100.0
        ));
    }
    if continental.submerged_fraction() > 0.20 {
        return Err(format!(
            "{seed}: total continental crust is excessively submerged at {:.1}%",
            continental.submerged_fraction() * 100.0
        ));
    }
    if modified_continental.submerged_fraction() + 0.01
        < stable_continental.submerged_fraction()
    {
        return Err(format!(
            "{seed}: modified continental crust stands systematically higher than stable interiors: {:.1}% vs {:.1}% submerged",
            modified_continental.submerged_fraction() * 100.0,
            stable_continental.submerged_fraction() * 100.0
        ));
    }
    if continental.submerged_fraction() > 0.01
        && continental.shallow_fraction_of_submerged() < 0.50
    {
        return Err(format!(
            "{seed}: submerged continental material lost its shallow-shelf character: {:.1}% shallow",
            continental.shallow_fraction_of_submerged() * 100.0
        ));
    }
    if transitional.submerged_fraction() < 0.80 {
        return Err(format!(
            "{seed}: transitional crust lost its marine margin character at {:.1}% submerged",
            transitional.submerged_fraction() * 100.0
        ));
    }
    if oceanic.submerged_fraction() < 0.95 {
        return Err(format!(
            "{seed}: oceanic crust is insufficiently marine at {:.1}% submerged",
            oceanic.submerged_fraction() * 100.0
        ));
    }
    Ok(())
}

fn verify_high_resolution_reference() -> Result<(), String> {
    let seed = "interlink-wg7c";
    let (fine, _inherited, terrain) = build_world(seed, 6, 8)?;
    let mut land_area = 0.0_f64;
    let mut land_above_2km = 0.0_f64;
    let mut land_above_3km = 0.0_f64;
    for sample in 0..terrain.solid_elevation_m.len() {
        if terrain.submerged_mask[sample] != 0 {
            continue;
        }
        let area = fine.dual_area_steradians()[sample];
        let elevation = f64::from(terrain.elevation_above_sea_level_m[sample]);
        land_area += area;
        land_above_2km += area * f64::from(elevation >= 2_000.0);
        land_above_3km += area * f64::from(elevation >= 3_000.0);
    }
    let highland_2km = land_above_2km / land_area.max(1.0e-12);
    let highland_3km = land_above_3km / land_area.max(1.0e-12);
    println!(
        "forward-hypsometry-highres seed={seed} L6->L8 samples={} land={:.1}% mean-land={:.0}m ocean-depth={:.0}m solid-p95={:.0}m highland2/3={:.1}/{:.1}% range={:.0}..{:.0}m",
        terrain.metrics.sample_count,
        terrain.metrics.land_area_fraction * 100.0,
        terrain.metrics.mean_land_elevation_m,
        terrain.metrics.mean_water_depth_m,
        terrain.metrics.p95_solid_elevation_m,
        highland_2km * 100.0,
        highland_3km * 100.0,
        terrain.metrics.minimum_solid_elevation_m,
        terrain.metrics.maximum_solid_elevation_m,
    );

    if terrain.metrics.sample_count != 655_362 {
        return Err(format!(
            "high-resolution hypsometry used unexpected sample count {}",
            terrain.metrics.sample_count
        ));
    }
    if terrain.metrics.water_volume_relative_error.abs() > 1.0e-9 {
        return Err(format!(
            "high-resolution water closure regressed: {:.3e}",
            terrain.metrics.water_volume_relative_error
        ));
    }
    if !(0.16..=0.26).contains(&terrain.metrics.land_area_fraction) {
        return Err(format!(
            "high-resolution land fraction escaped the calibration envelope: {:.1}%",
            terrain.metrics.land_area_fraction * 100.0
        ));
    }
    if !(1_200.0..=2_200.0).contains(&terrain.metrics.mean_land_elevation_m) {
        return Err(format!(
            "high-resolution mean land elevation escaped the calibration envelope: {:.0} m",
            terrain.metrics.mean_land_elevation_m
        ));
    }
    if !(3_000.0..=3_800.0).contains(&terrain.metrics.mean_water_depth_m) {
        return Err(format!(
            "high-resolution mean ocean depth escaped the calibration envelope: {:.0} m",
            terrain.metrics.mean_water_depth_m
        ));
    }
    if !(3_500.0..=5_000.0).contains(&terrain.metrics.p95_solid_elevation_m) {
        return Err(format!(
            "high-resolution solid-elevation p95 escaped the calibration envelope: {:.0} m",
            terrain.metrics.p95_solid_elevation_m
        ));
    }
    if highland_2km > 0.55 || highland_3km > 0.12 {
        return Err(format!(
            "high-resolution continental highlands became too broad: {:.1}% above 2 km, {:.1}% above 3 km",
            highland_2km * 100.0,
            highland_3km * 100.0
        ));
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let mut failures = Vec::<String>::new();
    for seed in [
        "interlink-wg7c",
        "1",
        "2",
        "continental-hypsometry-holdout",
    ] {
        if let Err(error) = verify_seed(seed) {
            failures.push(error);
        }
    }
    if let Err(error) = verify_high_resolution_reference() {
        failures.push(error);
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
