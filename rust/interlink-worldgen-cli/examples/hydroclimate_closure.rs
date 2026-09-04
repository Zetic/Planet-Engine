use interlink_worldgen::{
    build_hydroclimate_closure_report, build_icosphere, generate_coupled_climate,
    generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, ClimateRequest,
    GeologyRequest, LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest,
    TopographyRequest,
};
use std::env;
use std::time::Instant;

#[derive(Clone, Debug)]
struct Options {
    seed: String,
    coarse_level: u8,
    level: u8,
    plates: u16,
    skip_orography_intervention: bool,
}

fn parse_value<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> Result<T, String> {
    let Some(index) = args.iter().position(|arg| arg == name) else {
        return Ok(default);
    };
    let Some(value) = args.get(index + 1) else {
        return Err(format!("missing value after {name}"));
    };
    value
        .parse::<T>()
        .map_err(|_| format!("invalid value for {name}: {value}"))
}

fn parse_options() -> Result<Options, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let seed = args
        .iter()
        .position(|arg| arg == "--seed")
        .and_then(|index| args.get(index + 1))
        .cloned()
        .unwrap_or_else(|| "interlink-wg5".to_owned());
    Ok(Options {
        seed,
        coarse_level: parse_value(&args, "--coarse-level", 4_u8)?,
        level: parse_value(&args, "--level", 6_u8)?,
        plates: parse_value(&args, "--plates", 16_u16)?,
        skip_orography_intervention: args
            .iter()
            .any(|arg| arg == "--skip-orography-intervention"),
    })
}

fn option(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.6}"))
        .unwrap_or_else(|| "na".to_owned())
}

fn main() -> Result<(), String> {
    let options = parse_options()?;
    if options.coarse_level > options.level {
        return Err("--coarse-level cannot exceed --level".to_owned());
    }
    let started = Instant::now();
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(options.coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(options.level).map_err(|error| error.to_string())?;
    let tectonics = generate_tectonics(
        &coarse,
        &TectonicsRequest::new(options.seed.as_str(), options.plates),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let geology = generate_crust_and_history(
        &coarse,
        &tectonics,
        &GeologyRequest::new(options.seed.as_str()),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let lithosphere = generate_lithosphere(
        &coarse,
        &tectonics,
        &geology,
        &LithosphereRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let inherited = inherit_physical_state(
        &fine,
        options.coarse_level,
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
        &TopographyRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let request = ClimateRequest::new(options.seed.as_str());
    let climate = generate_coupled_climate(&fine, &terrain, planet, &request)
        .map_err(|error| error.to_string())?;
    let no_orography = if options.skip_orography_intervention {
        None
    } else {
        let mut intervention = request.clone();
        intervention.parameters.maximum_orographic_fraction = 0.0;
        Some(
            generate_coupled_climate(&fine, &terrain, planet, &intervention)
                .map_err(|error| error.to_string())?,
        )
    };
    let report = build_hydroclimate_closure_report(
        &fine,
        &terrain,
        &climate,
        no_orography.as_ref(),
    )
    .map_err(|error| error.to_string())?;

    println!("WG-5 hydroclimate closure");
    println!(
        "seed={} coarse_level={} fine_level={} plates={} samples={} climate_hash={}",
        options.seed,
        options.coarse_level,
        options.level,
        options.plates,
        report.sample_count,
        climate.metrics.climate_hash_hex()
    );
    println!(
        "land_precip_mm mean={:.3} p05={:.3} p50={:.3} p95={:.3} spatial_cv={:.6}",
        report.mean_land_precipitation_mm,
        report.land_precipitation_p05_mm,
        report.land_precipitation_p50_mm,
        report.land_precipitation_p95_mm,
        report.land_precipitation_spatial_cv
    );
    println!(
        "land_seasonality p50={:.6} p95={:.6}",
        report.land_precipitation_seasonality_p50,
        report.land_precipitation_seasonality_p95
    );
    println!(
        "land_pet_mm mean={:.3} p95={:.3}",
        report.mean_land_potential_evaporation_mm,
        report.land_potential_evaporation_p95_mm
    );
    println!(
        "land_aridity p05={:.6} p50={:.6} p95={:.6} below_0_2={:.6} below_0_5={:.6} at_least_1={:.6}",
        report.land_aridity_index_p05,
        report.land_aridity_index_p50,
        report.land_aridity_index_p95,
        report.land_aridity_below_0_2_fraction,
        report.land_aridity_below_0_5_fraction,
        report.land_aridity_at_least_1_fraction
    );
    println!(
        "latitude_regimes tropical_mm={} subtropical_mm={} midlatitude_mm={} polar_mm={} tropical_to_subtropical_ratio={}",
        option(report.tropical_land_precipitation_mm),
        option(report.subtropical_land_precipitation_mm),
        option(report.midlatitude_land_precipitation_mm),
        option(report.polar_land_precipitation_mm),
        option(report.tropical_to_subtropical_land_precipitation_ratio)
    );
    println!(
        "cryosphere land_snowfall_mm={:.3} persistent_snow_land_fraction={:.6} sea_ice_ocean_fraction={:.6}",
        report.mean_land_snowfall_mm,
        report.persistent_snow_land_area_fraction,
        report.sea_ice_ocean_area_fraction
    );
    if let (Some(rms), Some(fraction)) = (
        report.no_orography_land_precipitation_rms_difference_mm,
        report.no_orography_land_precipitation_rms_fraction_of_mean,
    ) {
        println!(
            "orography_spatial land_precip_rms_difference_mm={:.3} fraction_of_land_mean={:.6}",
            rms, fraction
        );
    }
    println!("hydroclimate_band,min_deg,max_deg,area_fraction,land_fraction,precip_mm,land_precip_mm,land_pet_mm,land_aridity,land_seasonality,snowfall_mm,persistent_snow,sea_ice");
    for band in &report.latitude_bands {
        println!(
            "hydroclimate_band,{:.0},{:.0},{:.8},{:.8},{:.3},{},{},{},{},{:.3},{:.6},{:.6}",
            band.minimum_latitude_deg,
            band.maximum_latitude_deg,
            band.area_fraction,
            band.land_area_fraction,
            band.mean_precipitation_mm,
            option(band.mean_land_precipitation_mm),
            option(band.mean_land_potential_evaporation_mm),
            option(band.mean_land_aridity_index),
            option(band.mean_land_precipitation_seasonality),
            band.mean_snowfall_mm,
            band.mean_persistent_snow_potential,
            band.mean_sea_ice_potential
        );
    }
    println!(
        "elapsed_ms={:.3}",
        started.elapsed().as_secs_f64() * 1_000.0
    );
    Ok(())
}
