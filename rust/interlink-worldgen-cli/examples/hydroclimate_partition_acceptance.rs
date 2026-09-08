use interlink_worldgen::{
    build_icosphere, generate_coupled_climate, generate_crust_and_history,
    generate_drainage_topology, generate_initial_topography, generate_lithosphere,
    generate_runoff_discharge, generate_tectonics, inherit_boundary_interfaces,
    inherit_physical_state, ClimateRequest, DrainageRequest, GeologyRequest, LithosphereRequest,
    PlanetPhysicalParameters, RunoffRequest, TectonicsRequest, TopographyRequest,
};

const SEEDS: &[&str] = &[
    "3",
    "interlink-wg7c",
    "wg4-boundary-a",
    "wg4-boundary-b",
    "wg4-boundary-c",
    "wg4-boundary-d",
];
const COARSE_LEVEL: u8 = 5;
const FINE_LEVEL: u8 = 7;
const PLATES: u16 = 16;
const MIN_AGGREGATE_RUNOFF_FRACTION: f64 = 0.30;
const MAX_AGGREGATE_RUNOFF_FRACTION: f64 = 0.55;
const MAXIMUM_SEED_RUNOFF_FRACTION: f64 = 0.62;
const MIN_AGGREGATE_AET_FRACTION: f64 = 0.40;
const MAXIMUM_DISCHARGE_CLOSURE_ERROR: f64 = 1.0e-10;

fn main() -> Result<(), String> {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(COARSE_LEVEL).map_err(|error| error.to_string())?;
    let fine = build_icosphere(FINE_LEVEL).map_err(|error| error.to_string())?;
    let mut total_land_area_m2 = 0.0_f64;
    let mut precipitation_area_sum = 0.0_f64;
    let mut aet_area_sum = 0.0_f64;
    let mut runoff_area_sum = 0.0_f64;
    let mut maximum_seed_runoff_fraction = 0.0_f64;

    for seed in SEEDS {
        let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(*seed, PLATES), planet)
            .map_err(|error| error.to_string())?;
        let geology =
            generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(*seed), planet)
                .map_err(|error| error.to_string())?;
        let lithosphere = generate_lithosphere(
            &coarse,
            &tectonics,
            &geology,
            &LithosphereRequest::new(*seed),
        )
        .map_err(|error| error.to_string())?;
        let inherited = inherit_physical_state(
            &fine,
            COARSE_LEVEL,
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
            &TopographyRequest::new(*seed),
        )
        .map_err(|error| error.to_string())?;
        let climate =
            generate_coupled_climate(&fine, &terrain, planet, &ClimateRequest::new(*seed))
                .map_err(|error| error.to_string())?;
        let drainage =
            generate_drainage_topology(&fine, &terrain, planet, &DrainageRequest::new(*seed))
                .map_err(|error| error.to_string())?;
        let runoff = generate_runoff_discharge(
            &fine,
            &terrain,
            &climate,
            &drainage,
            planet,
            &RunoffRequest::new(*seed),
        )
        .map_err(|error| error.to_string())?;
        let metrics = &runoff.metrics;
        if metrics.discharge_conservation_relative_error > MAXIMUM_DISCHARGE_CLOSURE_ERROR {
            return Err(format!(
                "{seed}: discharge closure {:.3e} exceeds {:.3e}",
                metrics.discharge_conservation_relative_error, MAXIMUM_DISCHARGE_CLOSURE_ERROR,
            ));
        }
        if metrics.land_runoff_fraction > MAXIMUM_SEED_RUNOFF_FRACTION {
            return Err(format!(
                "{seed}: runoff fraction {:.3} exceeds Earth-like ensemble ceiling {:.3}",
                metrics.land_runoff_fraction, MAXIMUM_SEED_RUNOFF_FRACTION,
            ));
        }

        let area = metrics.land_area_m2;
        total_land_area_m2 += area;
        precipitation_area_sum += metrics.mean_land_precipitation_mm * area;
        aet_area_sum += metrics.mean_land_actual_evapotranspiration_mm * area;
        runoff_area_sum += metrics.mean_land_runoff_mm * area;
        maximum_seed_runoff_fraction =
            maximum_seed_runoff_fraction.max(metrics.land_runoff_fraction);
        println!(
            "{seed}: P={:.1} AET={:.1} R={:.1} runoff={:.1}%",
            metrics.mean_land_precipitation_mm,
            metrics.mean_land_actual_evapotranspiration_mm,
            metrics.mean_land_runoff_mm,
            metrics.land_runoff_fraction * 100.0,
        );
    }

    let precipitation = precipitation_area_sum / total_land_area_m2;
    let aet = aet_area_sum / total_land_area_m2;
    let runoff = runoff_area_sum / total_land_area_m2;
    let runoff_fraction = runoff_area_sum / precipitation_area_sum.max(1.0e-12);
    let aet_fraction = aet_area_sum / precipitation_area_sum.max(1.0e-12);
    println!(
        "WG-5/WG-6 hydroclimate partition acceptance: P={precipitation:.1} AET={aet:.1} R={runoff:.1} runoff={:.1}% aet={:.1}% max_seed_runoff={:.1}%",
        runoff_fraction * 100.0,
        aet_fraction * 100.0,
        maximum_seed_runoff_fraction * 100.0,
    );

    if !(MIN_AGGREGATE_RUNOFF_FRACTION..=MAX_AGGREGATE_RUNOFF_FRACTION).contains(&runoff_fraction) {
        return Err(format!(
            "aggregate runoff fraction {:.3} escaped Earth-like calibration envelope [{:.3}, {:.3}]",
            runoff_fraction,
            MIN_AGGREGATE_RUNOFF_FRACTION,
            MAX_AGGREGATE_RUNOFF_FRACTION,
        ));
    }
    if aet_fraction < MIN_AGGREGATE_AET_FRACTION {
        return Err(format!(
            "aggregate AET fraction {:.3} is below minimum {:.3}",
            aet_fraction, MIN_AGGREGATE_AET_FRACTION,
        ));
    }
    if ((aet + runoff) - precipitation).abs() > 1.0e-6 {
        return Err(format!(
            "aggregate annual water balance did not close: P={precipitation} AET={aet} R={runoff}",
        ));
    }
    Ok(())
}
