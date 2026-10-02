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
    if !(0.10..=0.42).contains(&terrain.metrics.land_area_fraction) {
        return Err(format!(
            "{seed}: L4->L6 emergent land escaped the forward calibration envelope: {:.1}%",
            terrain.metrics.land_area_fraction * 100.0
        ));
    }
    if !(0.0..=2_200.0).contains(&terrain.metrics.mean_land_elevation_m) {
        return Err(format!(
            "{seed}: mean land elevation escaped the forward calibration envelope: {:.0} m",
            terrain.metrics.mean_land_elevation_m
        ));
    }
    if !(3_200.0..=4_800.0).contains(&terrain.metrics.mean_water_depth_m) {
        return Err(format!(
            "{seed}: mean ocean depth escaped the forward calibration envelope: {:.0} m",
            terrain.metrics.mean_water_depth_m
        ));
    }
    if !(2_800.0..=5_500.0).contains(&terrain.metrics.p95_solid_elevation_m) {
        return Err(format!(
            "{seed}: solid-elevation p95 escaped the forward calibration envelope: {:.0} m",
            terrain.metrics.p95_solid_elevation_m
        ));
    }
    // The L4->L6 pass is a cheap smoke check; physical shelf width and highland-area authority
    // live in the production L6->L8 gate below, where those features are actually resolved.
    if highland_2km > 0.45 || highland_3km > 0.25 {
        return Err(format!(
            "{seed}: continental highlands became too spatially broad: {:.1}% above 2 km, {:.1}% above 3 km",
            highland_2km * 100.0,
            highland_3km * 100.0
        ));
    }
    if stable_continental.submerged_fraction() > 0.55 {
        return Err(format!(
            "{seed}: stable continental interiors are systematically drowned at {:.1}%",
            stable_continental.submerged_fraction() * 100.0
        ));
    }
    if continental.submerged_fraction() > 0.65 {
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
        && continental.shallow_fraction_of_submerged() < 0.25
    {
        return Err(format!(
            "{seed}: submerged continental material lost all shallow-shelf character: {:.1}% shallow",
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

fn verify_production_seed(seed: &str) -> Result<(), String> {
    let (fine, inherited, terrain) = build_world(seed, 6, 8)?;
    let mut land_area = 0.0_f64;
    let mut land_above_2km = 0.0_f64;
    let mut land_above_3km = 0.0_f64;
    let mut quiet_area = 0.0_f64;
    let mut quiet_land_area = 0.0_f64;
    let mut quiet_land_elevation_sum = 0.0_f64;
    let mut quiet_below_1500m = 0.0_f64;
    let mut quiet_above_2000m = 0.0_f64;
    let mut orogenic_land_area = 0.0_f64;
    let mut orogenic_land_elevation_sum = 0.0_f64;
    let mut crust_area = [0.0_f64; 3];
    let mut submerged_crust_area = [0.0_f64; 3];
    let mut shallow_submerged_crust_area = [0.0_f64; 3];
    let mut thinned_continental_area = 0.0_f64;

    for sample in 0..terrain.solid_elevation_m.len() {
        let area = fine.dual_area_steradians()[sample];
        let land = terrain.submerged_mask[sample] == 0;
        let elevation = if land {
            f64::from(terrain.elevation_above_sea_level_m[sample])
        } else {
            0.0
        };
        if land {
            land_area += area;
            land_above_2km += area * f64::from(elevation >= 2_000.0);
            land_above_3km += area * f64::from(elevation >= 3_000.0);
        }

        let crust_bucket = if inherited.crust_kind[sample] == CrustKind::Continental as u8 {
            0
        } else if inherited.crust_kind[sample] == CrustKind::Transitional as u8 {
            1
        } else {
            2
        };
        crust_area[crust_bucket] += area;
        if !land {
            submerged_crust_area[crust_bucket] += area;
            if f64::from(terrain.water_depth_m[sample]) <= 500.0 {
                shallow_submerged_crust_area[crust_bucket] += area;
            }
        }
        if crust_bucket == 0 && inherited.crust_thickness_km[sample] < 36.0 {
            thinned_continental_area += area;
        }

        if inherited.crust_kind[sample] != CrustKind::Continental as u8 {
            continue;
        }
        let structural_modified =
            inherited.structural_zone_kind[sample] == InheritedStructureKind::ContinentalMargin as u8
                || inherited.structural_zone_kind[sample]
                    == InheritedStructureKind::InheritedRift as u8
                || inherited.crust_thickness_km[sample] < 36.0;
        let modified = structural_modified
            || inherited.rift_history[sample] >= 0.22
            || inherited.subsidence_history[sample] >= 0.28
            || inherited.basin_potential[sample] >= 0.32;
        let orogenic = inherited.province_kind[sample] != 0
            || inherited.orogenic_history[sample] >= 0.45
            || terrain.orogenic_elevation_m[sample] >= 800.0;

        if orogenic {
            if land {
                orogenic_land_area += area;
                orogenic_land_elevation_sum += area * elevation;
            }
        } else if !modified {
            quiet_area += area;
            if land {
                quiet_land_area += area;
                quiet_land_elevation_sum += area * elevation;
                quiet_below_1500m += area * f64::from(elevation < 1_500.0);
                quiet_above_2000m += area * f64::from(elevation >= 2_000.0);
            }
        }
    }

    let highland_2km = land_above_2km / land_area.max(1.0e-12);
    let highland_3km = land_above_3km / land_area.max(1.0e-12);
    let quiet_emergent = quiet_land_area / quiet_area.max(1.0e-12);
    let quiet_lowland = quiet_below_1500m / quiet_land_area.max(1.0e-12);
    let quiet_highland = quiet_above_2000m / quiet_land_area.max(1.0e-12);
    let quiet_mean = quiet_land_elevation_sum / quiet_land_area.max(1.0e-12);
    let orogenic_mean = orogenic_land_elevation_sum / orogenic_land_area.max(1.0e-12);

    let total_crust_area = crust_area.iter().sum::<f64>().max(1.0e-12);
    println!(
        "forward-hypsometry-production seed={seed} L6->L8 samples={} land={:.1}% mean-land={:.0}m ocean-depth={:.0}m solid-p95={:.0}m highland2/3={:.1}/{:.1}% quiet-emergent={:.1}% quiet-mean={:.0}m quiet<1.5km={:.1}% quiet>=2km={:.1}% orogenic-mean={:.0}m crust(c/t/o)={:.1}/{:.1}/{:.1}% submerged(c/t/o)={:.1}/{:.1}/{:.1}% shallow-submerged-c={:.1}% thinned-c={:.1}%",
        terrain.metrics.sample_count,
        terrain.metrics.land_area_fraction * 100.0,
        terrain.metrics.mean_land_elevation_m,
        terrain.metrics.mean_water_depth_m,
        terrain.metrics.p95_solid_elevation_m,
        highland_2km * 100.0,
        highland_3km * 100.0,
        quiet_emergent * 100.0,
        quiet_mean,
        quiet_lowland * 100.0,
        quiet_highland * 100.0,
        orogenic_mean,
        crust_area[0] / total_crust_area * 100.0,
        crust_area[1] / total_crust_area * 100.0,
        crust_area[2] / total_crust_area * 100.0,
        submerged_crust_area[0] / crust_area[0].max(1.0e-12) * 100.0,
        submerged_crust_area[1] / crust_area[1].max(1.0e-12) * 100.0,
        submerged_crust_area[2] / crust_area[2].max(1.0e-12) * 100.0,
        shallow_submerged_crust_area[0] / submerged_crust_area[0].max(1.0e-12) * 100.0,
        thinned_continental_area / total_crust_area * 100.0,
    );

    if terrain.metrics.sample_count != 655_362 {
        return Err(format!(
            "{seed}: production hypsometry used unexpected sample count {}",
            terrain.metrics.sample_count
        ));
    }
    if terrain.metrics.water_volume_relative_error.abs() > 1.0e-9 {
        return Err(format!(
            "{seed}: production water closure regressed: {:.3e}",
            terrain.metrics.water_volume_relative_error
        ));
    }
    if !(0.14..=0.24).contains(&terrain.metrics.land_area_fraction) {
        return Err(format!(
            "{seed}: production land fraction escaped the calibration envelope: {:.1}%",
            terrain.metrics.land_area_fraction * 100.0
        ));
    }
    if !(250.0..=1_400.0).contains(&terrain.metrics.mean_land_elevation_m) {
        return Err(format!(
            "{seed}: production mean land elevation escaped the calibration envelope: {:.0} m",
            terrain.metrics.mean_land_elevation_m
        ));
    }
    if !(2_800.0..=4_000.0).contains(&terrain.metrics.mean_water_depth_m) {
        return Err(format!(
            "{seed}: production mean ocean depth escaped the calibration envelope: {:.0} m",
            terrain.metrics.mean_water_depth_m
        ));
    }
    if !(2_800.0..=4_500.0).contains(&terrain.metrics.p95_solid_elevation_m) {
        return Err(format!(
            "{seed}: production solid-elevation p95 escaped the calibration envelope: {:.0} m",
            terrain.metrics.p95_solid_elevation_m
        ));
    }
    if highland_2km > 0.20 || highland_3km > 0.10 {
        return Err(format!(
            "{seed}: production highlands became too spatially broad: {:.1}% above 2 km, {:.1}% above 3 km",
            highland_2km * 100.0,
            highland_3km * 100.0
        ));
    }
    if quiet_area <= 0.02 * fine.dual_area_steradians().iter().sum::<f64>() {
        return Err(format!(
            "{seed}: production world did not expose a measurable quiet continental interior"
        ));
    }
    if quiet_emergent < 0.85 {
        return Err(format!(
            "{seed}: quiet continental interiors are excessively drowned at {:.1}% emergent",
            quiet_emergent * 100.0
        ));
    }
    if quiet_lowland < 0.70 || quiet_highland > 0.05 {
        return Err(format!(
            "{seed}: quiet continental interior is not lowland-dominated: {:.1}% below 1.5 km, {:.1}% at/above 2 km",
            quiet_lowland * 100.0,
            quiet_highland * 100.0
        ));
    }
    if orogenic_land_area > 1.0e-6 && orogenic_mean < quiet_mean + 250.0 {
        return Err(format!(
            "{seed}: localized orogenic terrain lost relief contrast: quiet {:.0} m vs orogenic {:.0} m",
            quiet_mean,
            orogenic_mean
        ));
    }
    let continental_shallow_fraction =
        shallow_submerged_crust_area[0] / submerged_crust_area[0].max(1.0e-12);
    if submerged_crust_area[0] > 0.01 * crust_area[0]
        && continental_shallow_fraction < 0.30
    {
        return Err(format!(
            "{seed}: production submerged continental margin lost shallow-shelf character: {:.1}% shallow",
            continental_shallow_fraction * 100.0
        ));
    }
    let transitional_fraction = crust_area[1] / total_crust_area;
    if transitional_fraction > 0.18 {
        return Err(format!(
            "{seed}: production transitional crust remains too spatially broad: {:.1}% of surface",
            transitional_fraction * 100.0
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
    for seed in [
        "interlink-wg7c",
        "1",
        "2",
        "continental-freeboard-holdout",
        "444",
    ] {
        if let Err(error) = verify_production_seed(seed) {
            failures.push(error);
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
