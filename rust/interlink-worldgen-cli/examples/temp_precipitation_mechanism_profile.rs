use interlink_worldgen::{
    build_hydroclimate_closure_report, build_icosphere, generate_coupled_climate,
    generate_crust_and_history, generate_drainage_topology, generate_initial_topography,
    generate_lithosphere, generate_runoff_discharge, generate_tectonics,
    inherit_boundary_interfaces, inherit_physical_state, ClimateRequest, DrainageRequest,
    GeologyRequest, LithosphereRequest, PlanetPhysicalParameters, RunoffRequest,
    TectonicsRequest, TopographyRequest,
};

const SEEDS: &[&str] = &[
    "3", "interlink-wg7c", "wg4-boundary-a", "wg4-boundary-b", "wg4-boundary-c", "wg4-boundary-d",
];

fn wet_share(climate: &interlink_worldgen::ClimateState, terrain: &interlink_worldgen::TopographyState, topo: &interlink_worldgen::GeodesicTopology, threshold: f64) -> (f64, f64) {
    let mut land_area = 0.0;
    let mut wet_area = 0.0;
    let mut total_p = 0.0;
    let mut wet_p = 0.0;
    for i in 0..topo.metrics().sample_count as usize {
        if terrain.submerged_mask[i] != 0 { continue; }
        let a = topo.dual_area_steradians()[i];
        let p = f64::from(climate.annual_precipitation_mm[i]);
        land_area += a;
        total_p += p * a;
        if p >= threshold {
            wet_area += a;
            wet_p += p * a;
        }
    }
    (wet_area / land_area.max(1e-18), wet_p / total_p.max(1e-18))
}

fn main() -> Result<(), String> {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;

    for seed in SEEDS {
        let coarse = build_icosphere(coarse_level).map_err(|e| e.to_string())?;
        let fine = build_icosphere(fine_level).map_err(|e| e.to_string())?;
        let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(*seed, plates), planet).map_err(|e| e.to_string())?;
        let geology = generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(*seed), planet).map_err(|e| e.to_string())?;
        let lithosphere = generate_lithosphere(&coarse, &tectonics, &geology, &LithosphereRequest::new(*seed)).map_err(|e| e.to_string())?;
        let inherited = inherit_physical_state(&fine, coarse_level, &tectonics, &geology, &lithosphere, planet).map_err(|e| e.to_string())?;
        let boundaries = inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids).map_err(|e| e.to_string())?;
        let terrain = generate_initial_topography(&fine, &inherited, &boundaries, planet, &TopographyRequest::new(*seed)).map_err(|e| e.to_string())?;
        let drainage = generate_drainage_topology(&fine, &terrain, planet, &DrainageRequest::new(*seed)).map_err(|e| e.to_string())?;

        println!("SEED {seed}");
        for scenario in ["base", "orog_half", "orog_zero", "conv_low", "conv_high_rh", "combo"] {
            let mut req = ClimateRequest::new(*seed);
            match scenario {
                "orog_half" => {
                    req.parameters.orographic_precipitation_strength = 8.0;
                    req.parameters.maximum_orographic_fraction = 0.16;
                }
                "orog_zero" => {
                    req.parameters.maximum_orographic_fraction = 0.0;
                }
                "conv_low" => {
                    req.parameters.convergence_precipitation_efficiency = 0.15;
                }
                "conv_high_rh" => {
                    req.parameters.convergence_precipitation_relative_humidity = 0.75;
                }
                "combo" => {
                    req.parameters.orographic_precipitation_strength = 8.0;
                    req.parameters.maximum_orographic_fraction = 0.16;
                    req.parameters.convergence_precipitation_efficiency = 0.18;
                    req.parameters.convergence_precipitation_relative_humidity = 0.70;
                }
                _ => {}
            }
            let climate = generate_coupled_climate(&fine, &terrain, planet, &req).map_err(|e| format!("{scenario}: {e}"))?;
            let report = build_hydroclimate_closure_report(&fine, &terrain, &climate, None).map_err(|e| e.to_string())?;
            let runoff = generate_runoff_discharge(&fine, &terrain, &climate, &drainage, planet, &RunoffRequest::new(*seed)).map_err(|e| e.to_string())?;
            let (wet1_area, wet1_p) = wet_share(&climate, &terrain, &fine, 1000.0);
            let (wet2_area, wet2_p) = wet_share(&climate, &terrain, &fine, 2000.0);
            println!(
                "  {scenario:<12} P={:.0} p50={:.0} p95={:.0} cv={:.2} PET={:.0} AET={:.0} R={:.0} runoff={:.1}% wet1_area={:.1}% wet1_P={:.1}% wet2_area={:.1}% wet2_P={:.1}% mb={:.1e}",
                report.mean_land_precipitation_mm,
                report.land_precipitation_p50_mm,
                report.land_precipitation_p95_mm,
                report.land_precipitation_spatial_cv,
                report.mean_land_potential_evaporation_mm,
                runoff.metrics.mean_land_actual_evapotranspiration_mm,
                runoff.metrics.mean_land_runoff_mm,
                runoff.metrics.land_runoff_fraction * 100.0,
                wet1_area * 100.0,
                wet1_p * 100.0,
                wet2_area * 100.0,
                wet2_p * 100.0,
                climate.metrics.moisture_budget_relative_error,
            );
        }
    }
    Ok(())
}
