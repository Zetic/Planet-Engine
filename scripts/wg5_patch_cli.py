from pathlib import Path

path = Path('rust/interlink-worldgen-cli/src/main.rs')
text = path.read_text()

old_import = '''use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_synthetic, generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    GeologyRequest, LithosphereRequest, PlanetPhysicalParameters, PlateScaleClass,
    SyntheticRequest, TectonicFragmentKind, TectonicsRequest, TopographyRequest,
    WORLDGEN_ENGINE_VERSION,
};'''
new_import = '''use interlink_worldgen::{
    build_icosphere, generate_coupled_climate, generate_crust_and_history,
    generate_initial_topography, generate_lithosphere, generate_synthetic, generate_tectonics,
    inherit_boundary_interfaces, inherit_physical_state, ClimateRequest, GeologyRequest,
    LithosphereRequest, PlanetPhysicalParameters, PlateScaleClass, SyntheticRequest,
    TectonicFragmentKind, TectonicsRequest, TopographyRequest, WORLDGEN_ENGINE_VERSION,
};'''
if old_import not in text:
    raise SystemExit('WG-5 CLI import marker not found')
text = text.replace(old_import, new_import, 1)

old_usage = '<generate|benchmark|topology|tectonics|geology|lithosphere|inheritance|topography|profile>'
if old_usage not in text:
    raise SystemExit('WG-5 CLI usage marker not found')
text = text.replace(old_usage, '<generate|benchmark|topology|tectonics|geology|lithosphere|inheritance|topography|climate|profile>', 1)

old_match = '''            | "topography"
            | "profile"'''
if old_match not in text:
    raise SystemExit('WG-5 CLI command marker not found')
text = text.replace(old_match, '''            | "topography"
            | "climate"
            | "profile"''', 1)

marker = 'fn profile(_options: &Options) -> Result<(), String> {'
if marker not in text:
    raise SystemExit('WG-5 CLI profile marker not found')

climate_fn = '''fn climate(options: &Options) -> Result<(), String> {
    if options.coarse_level > options.level {
        return Err("--coarse-level cannot exceed --level".to_owned());
    }
    let started = Instant::now();
    let coarse = build_icosphere(options.coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(options.level).map_err(|error| error.to_string())?;
    let planet = PlanetPhysicalParameters::earthlike_reference();
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
    let boundaries = inherit_boundary_interfaces(
        &coarse,
        &fine,
        &tectonics,
        &geology,
        &inherited.plate_ids,
    )
    .map_err(|error| error.to_string())?;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let climate = generate_coupled_climate(
        &fine,
        &terrain,
        planet,
        &ClimateRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let metrics = &climate.metrics;
    println!("Project Interlink Planet Engine WG-5 Coupled Planetary Climate");
    println!("engine_version={}", WORLDGEN_ENGINE_VERSION);
    println!("stage={}@{}", climate.stage.id, climate.stage.version);
    println!("seed={} macro_plates={}", options.seed, options.plates);
    println!("levels coarse={} fine={}", options.coarse_level, options.level);
    println!(
        "samples={} phases={} spinup_years={}",
        metrics.sample_count, metrics.orbital_phase_count, metrics.spinup_years
    );
    println!(
        "climate_hash={} climate_physical_parameter_hash={} climate_model_parameter_hash={}",
        metrics.climate_hash_hex(),
        metrics.climate_physical_parameter_hash_hex(),
        metrics.climate_model_parameter_hash_hex()
    );
    println!(
        "topography_hash={} planet_parameter_hash={}",
        terrain.metrics.topography_hash_hex(),
        planet.parameter_hash_hex()
    );
    println!(
        "temperature_k min={:.3} mean={:.3} max={:.3} land_mean={:.3} ocean_mean={:.3} final_rms_change={:.6}",
        metrics.minimum_temperature_k,
        metrics.mean_temperature_k,
        metrics.maximum_temperature_k,
        metrics.mean_land_temperature_k,
        metrics.mean_ocean_temperature_k,
        metrics.final_temperature_rms_change_k
    );
    println!(
        "winds_m_s mean={:.4} max={:.4}",
        metrics.mean_wind_speed_m_s, metrics.maximum_wind_speed_m_s
    );
    println!(
        "surface_currents_m_s mean={:.5} max={:.5} divergence_residual={:.8}",
        metrics.mean_surface_current_m_s,
        metrics.maximum_surface_current_m_s,
        metrics.ocean_divergence_residual_m_s
    );
    println!("sst_mean_k={:.3}", metrics.mean_sea_surface_temperature_k);
    println!(
        "precipitation_mm_year mean={:.3} p95={:.3}",
        metrics.mean_annual_precipitation_mm, metrics.p95_annual_precipitation_mm
    );
    println!(
        "moisture_budget evaporation_kg={:.6e} precipitation_kg={:.6e} relative_error={:.6e}",
        metrics.global_evaporation_kg,
        metrics.global_precipitation_kg,
        metrics.moisture_budget_relative_error
    );
    println!(
        "cryosphere_potential persistent_snow_area_fraction={:.6} sea_ice_area_fraction={:.6}",
        metrics.persistent_snow_area_fraction, metrics.sea_ice_area_fraction
    );
    println!("elapsed_ms={:.3}", started.elapsed().as_secs_f64() * 1_000.0);
    Ok(())
}

'''
text = text.replace(marker, climate_fn + marker, 1)

old_dispatch = '''        "topography" => topography(&options),
        "profile" => profile(&options),'''
if old_dispatch not in text:
    raise SystemExit('WG-5 CLI dispatch marker not found')
text = text.replace(old_dispatch, '''        "topography" => topography(&options),
        "climate" => climate(&options),
        "profile" => profile(&options),''', 1)

path.write_text(text)
