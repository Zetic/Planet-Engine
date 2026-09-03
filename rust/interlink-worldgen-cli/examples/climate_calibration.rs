use interlink_worldgen::{
    build_climate_calibration_report, build_icosphere, generate_coupled_climate,
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
    let report = build_climate_calibration_report(
        &fine,
        &terrain,
        planet,
        &request,
        &climate,
        no_orography.as_ref(),
    )
    .map_err(|error| error.to_string())?;

    println!("WG-5 calibration baseline");
    println!(
        "seed={} coarse_level={} fine_level={} plates={}",
        options.seed, options.coarse_level, options.level, options.plates
    );
    println!(
        "samples={} phases={} climate_hash={}",
        report.sample_count,
        report.orbital_phase_count,
        climate.metrics.climate_hash_hex()
    );
    println!(
        "temperature_k global={:.3} land={:.3} ocean={:.3} contrast={:.3} sst={:.3}",
        report.mean_temperature_k,
        report.mean_land_temperature_k,
        report.mean_ocean_temperature_k,
        report.land_ocean_temperature_contrast_k,
        report.mean_sea_surface_temperature_k
    );
    println!("land_hypsometry_m mean={:.3} p50={:.3} p95={:.3} above_2km_fraction={:.6} above_4km_fraction={:.6}", report.mean_land_elevation_m, report.median_land_elevation_m, report.p95_land_elevation_m, report.land_area_above_2km_fraction, report.land_area_above_4km_fraction);
    println!(
        "effective_asr_w_m2 global={:.3} land={:.3} ocean={:.3}",
        report.effective_absorbed_shortwave_w_m2,
        report.effective_absorbed_shortwave_land_w_m2,
        report.effective_absorbed_shortwave_ocean_w_m2
    );
    println!(
        "olr_proxy_w_m2 global={:.3} land={:.3} ocean={:.3} imbalance_proxy={:.3}",
        report.outgoing_longwave_proxy_w_m2,
        report.outgoing_longwave_land_proxy_w_m2,
        report.outgoing_longwave_ocean_proxy_w_m2,
        report.toa_energy_imbalance_proxy_w_m2
    );
    println!(
        "transport_caps reconstructed_wind={:.6} reconstructed_moisture_edge={:.6}",
        report.reconstructed_wind_cap_fraction, report.reconstructed_moisture_edge_cap_fraction
    );
    println!(
        "relative_humidity_proxy p05={:.6} p50={:.6} p95={:.6}",
        report.mean_state_relative_humidity_p05,
        report.mean_state_relative_humidity_p50,
        report.mean_state_relative_humidity_p95
    );
    println!(
        "hydrology_mm_year precip_mean={:.3} precip_p95={:.3} pet_mean={:.3} p_over_e={:.6}",
        report.mean_annual_precipitation_mm,
        report.p95_annual_precipitation_mm,
        report.mean_potential_evaporation_mm,
        report.precipitation_to_evaporation_ratio
    );
    if let (Some(no_orography), Some(fraction)) = (
        report.no_orography_mean_annual_precipitation_mm,
        report.orographic_precipitation_causal_fraction,
    ) {
        println!(
            "orography_intervention no_orography_precip_mean_mm={:.3} causal_fraction={:.6}",
            no_orography, fraction
        );
    }
    println!("cryosphere mean_snowfall_mm={:.3} persistent_snow_area_fraction={:.6} sea_ice_area_fraction={:.6}", report.mean_snowfall_mm, report.persistent_snow_area_fraction, report.sea_ice_area_fraction);
    println!(
        "ocean_heat annual_mean_tendency_rms_index={:.6}",
        report.annual_mean_ocean_heat_tendency_rms_index
    );
    println!("latitude_band,min_deg,max_deg,area_fraction,temp_k,precip_mm,specific_humidity,rh_proxy,sst_k,snowfall_mm,sea_ice");
    for band in &report.latitude_bands {
        let sst = band
            .mean_sea_surface_temperature_k
            .map(|value| format!("{value:.3}"))
            .unwrap_or_default();
        println!(
            "latitude_band,{:.0},{:.0},{:.8},{:.3},{:.3},{:.8},{:.6},{},{:.3},{:.6}",
            band.minimum_latitude_deg,
            band.maximum_latitude_deg,
            band.area_fraction,
            band.mean_temperature_k,
            band.mean_precipitation_mm,
            band.mean_specific_humidity,
            band.mean_relative_humidity_proxy,
            sst,
            band.mean_snowfall_mm,
            band.mean_sea_ice_potential
        );
    }
    println!(
        "elapsed_ms={:.3}",
        started.elapsed().as_secs_f64() * 1_000.0
    );
    Ok(())
}
