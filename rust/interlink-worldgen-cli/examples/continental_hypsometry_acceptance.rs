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

fn verify_seed(seed: &str) -> Result<(), String> {
    let coarse_level = 4;
    let fine_level = 6;
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

    let mut continental = AreaBucket::default();
    let mut stable_continental = AreaBucket::default();
    let mut modified_continental = AreaBucket::default();
    let mut transitional = AreaBucket::default();
    let mut oceanic = AreaBucket::default();
    let mut modified_margin_area = 0.0_f64;
    let mut modified_rift_zone_area = 0.0_f64;
    let mut modified_rift_history_area = 0.0_f64;
    let mut modified_subsidence_area = 0.0_f64;
    let mut modified_basin_area = 0.0_f64;
    let mut modified_isostatic_sum = 0.0_f64;
    let mut modified_ridge_sum = 0.0_f64;
    let mut modified_rift_basin_sum = 0.0_f64;
    let mut modified_orogen_sum = 0.0_f64;
    let mut modified_mantle_sum = 0.0_f64;
    let total_area = fine.dual_area_steradians().iter().sum::<f64>();

    for sample in 0..terrain.solid_elevation_m.len() {
        let area = fine.dual_area_steradians()[sample];
        let submerged = terrain.submerged_mask[sample] != 0;
        let depth_m = f64::from(terrain.water_depth_m[sample]);
        let kind = inherited.crust_kind[sample];
        match kind {
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
                    modified_margin_area += area
                        * f64::from(
                            inherited.structural_zone_kind[sample]
                                == InheritedStructureKind::ContinentalMargin as u8,
                        );
                    modified_rift_zone_area += area
                        * f64::from(
                            inherited.structural_zone_kind[sample]
                                == InheritedStructureKind::InheritedRift as u8,
                        );
                    modified_rift_history_area +=
                        area * f64::from(inherited.rift_history[sample] >= 0.22);
                    modified_subsidence_area +=
                        area * f64::from(inherited.subsidence_history[sample] >= 0.28);
                    modified_basin_area +=
                        area * f64::from(inherited.basin_potential[sample] >= 0.32);
                    modified_isostatic_sum +=
                        area * f64::from(terrain.isostatic_elevation_m[sample]);
                    modified_ridge_sum += area * f64::from(terrain.ridge_elevation_m[sample]);
                    modified_rift_basin_sum +=
                        area * f64::from(terrain.rift_basin_elevation_m[sample]);
                    modified_orogen_sum += area * f64::from(terrain.orogenic_elevation_m[sample]);
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

    println!(
        "continental-hypsometry seed={seed} land={:.1}% continental-area={:.1}% continental-submerged={:.1}% stable-submerged={:.1}% modified-submerged={:.1}% transitional-submerged={:.1}% continental-shallow={:.1}% continental-depth={:.0}m oceanic-submerged={:.1}%",
        terrain.metrics.land_area_fraction * 100.0,
        continental.total / total_area * 100.0,
        continental.submerged_fraction() * 100.0,
        stable_continental.submerged_fraction() * 100.0,
        modified_continental.submerged_fraction() * 100.0,
        transitional.submerged_fraction() * 100.0,
        continental.shallow_fraction_of_submerged() * 100.0,
        continental.mean_submerged_depth_m(),
        oceanic.submerged_fraction() * 100.0,
    );
    let modified_area = modified_continental.total.max(1.0e-12);
    println!(
        "continental-modified seed={seed} margin={:.1}% rift-zone={:.1}% rift-history={:.1}% subsidence={:.1}% basin={:.1}% components(isostatic/ridge/rift/orogen/mantle)={:.0}/{:.0}/{:.0}/{:.0}/{:.0}m",
        modified_margin_area / modified_area * 100.0,
        modified_rift_zone_area / modified_area * 100.0,
        modified_rift_history_area / modified_area * 100.0,
        modified_subsidence_area / modified_area * 100.0,
        modified_basin_area / modified_area * 100.0,
        modified_isostatic_sum / modified_area,
        modified_ridge_sum / modified_area,
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
            "{seed}: continental hypsometry diagnostic lacked required crust classes"
        ));
    }
    if !(0.20..=0.38).contains(&terrain.metrics.land_area_fraction) {
        return Err(format!(
            "{seed}: emergent land moved outside the broad Earthlike calibration envelope: {:.1}%",
            terrain.metrics.land_area_fraction * 100.0
        ));
    }
    if stable_continental.submerged_fraction() > 0.24 {
        return Err(format!(
            "{seed}: stable continental interiors remain systematically drowned at {:.1}% submerged",
            stable_continental.submerged_fraction() * 100.0
        ));
    }
    if continental.submerged_fraction() > 0.44 {
        return Err(format!(
            "{seed}: total continental crust remains excessively submerged at {:.1}%",
            continental.submerged_fraction() * 100.0
        ));
    }
    if modified_continental.submerged_fraction()
        < stable_continental.submerged_fraction() + 0.08
    {
        return Err(format!(
            "{seed}: stable interiors are not measurably freer-standing than rifted/margin continental crust: {:.1}% vs {:.1}% submerged",
            stable_continental.submerged_fraction() * 100.0,
            modified_continental.submerged_fraction() * 100.0
        ));
    }
    if continental.submerged_fraction() < 0.20
        || continental.shallow_fraction_of_submerged() < 0.25
    {
        return Err(format!(
            "{seed}: calibration erased legitimate submerged continental shelves: {:.1}% submerged, {:.1}% of submerged area shallow",
            continental.submerged_fraction() * 100.0,
            continental.shallow_fraction_of_submerged() * 100.0
        ));
    }
    if transitional.submerged_fraction() < 0.75 {
        return Err(format!(
            "{seed}: transitional crust lost its shelf/margin character at {:.1}% submerged",
            transitional.submerged_fraction() * 100.0
        ));
    }
    if oceanic.submerged_fraction() < 0.90 {
        return Err(format!(
            "{seed}: oceanic crust is insufficiently marine at {:.1}% submerged",
            oceanic.submerged_fraction() * 100.0
        ));
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let mut failures = Vec::<String>::new();
    for seed in ["interlink-wg7c", "1", "2", "continental-hypsometry-holdout"] {
        if let Err(error) = verify_seed(seed) {
            failures.push(error);
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
