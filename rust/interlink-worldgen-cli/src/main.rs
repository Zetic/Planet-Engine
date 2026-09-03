use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_lithosphere, generate_synthetic,
    generate_tectonics, inherit_physical_state, GeologyRequest, LithosphereRequest,
    PlanetPhysicalParameters, PlateScaleClass, SyntheticRequest, TectonicFragmentKind,
    TectonicsRequest, WORLDGEN_ENGINE_VERSION,
};
use std::{env, process, time::Instant};

#[derive(Debug)]
struct Options {
    command: String,
    seed: String,
    width: u32,
    height: u32,
    iterations: u32,
    level: u8,
    coarse_level: u8,
    plates: u16,
}

fn usage() -> &'static str {
    "interlink-worldgen-cli <generate|benchmark|topology|tectonics|geology|lithosphere|inheritance|profile> [--seed TEXT] [--width N] [--height N] [--iterations N] [--level N] [--coarse-level N] [--plates N]"
}
fn parse_u32(name: &str, value: Option<String>) -> Result<u32, String> {
    value
        .ok_or_else(|| format!("{name} requires a value"))?
        .parse::<u32>()
        .map_err(|_| format!("{name} requires an unsigned integer"))
}

fn parse_options() -> Result<Options, String> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| usage().to_owned())?;
    if !matches!(
        command.as_str(),
        "generate"
            | "benchmark"
            | "topology"
            | "tectonics"
            | "geology"
            | "lithosphere"
            | "inheritance"
            | "profile"
    ) {
        return Err(format!("unsupported command '{command}'\n{}", usage()));
    }
    let mut options = Options {
        command,
        seed: "worldgen-cli".to_owned(),
        width: 512,
        height: 256,
        iterations: 5,
        level: 6,
        coarse_level: 4,
        plates: 16,
    };
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--seed" => {
                options.seed = args
                    .next()
                    .ok_or_else(|| "--seed requires a value".to_owned())?
            }
            "--width" => options.width = parse_u32("--width", args.next())?,
            "--height" => options.height = parse_u32("--height", args.next())?,
            "--iterations" => options.iterations = parse_u32("--iterations", args.next())?,
            "--level" => {
                options.level = u8::try_from(parse_u32("--level", args.next())?)
                    .map_err(|_| "--level exceeds u8 range".to_owned())?
            }
            "--coarse-level" => {
                options.coarse_level = u8::try_from(parse_u32("--coarse-level", args.next())?)
                    .map_err(|_| "--coarse-level exceeds u8 range".to_owned())?
            }
            "--plates" => {
                options.plates = u16::try_from(parse_u32("--plates", args.next())?)
                    .map_err(|_| "--plates exceeds u16 range".to_owned())?
            }
            _ => return Err(format!("unsupported option '{flag}'\n{}", usage())),
        }
    }
    if options.iterations == 0 {
        return Err("--iterations must be at least 1".to_owned());
    }
    Ok(options)
}

fn generate_once(options: &Options) -> Result<(), String> {
    let started = Instant::now();
    let result = generate_synthetic(&SyntheticRequest::new(
        options.seed.as_str(),
        options.width,
        options.height,
    ))
    .map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();
    println!("Project Interlink Planet Engine WG-0 transport diagnostic");
    println!("engine_version={}", WORLDGEN_ENGINE_VERSION);
    println!("stage={}@{}", result.stage.id, result.stage.version);
    println!("seed={}", options.seed);
    println!(
        "field={}x{} samples={}",
        result.field.width(),
        result.field.height(),
        result.statistics.sample_count
    );
    println!(
        "min={} max={} mean={:.3}",
        result.statistics.minimum, result.statistics.maximum, result.statistics.mean
    );
    println!("field_hash={}", result.statistics.hash_hex());
    println!("stage_seed={:016x}", result.stage.derived_seed);
    println!("elapsed_ms={:.3}", elapsed.as_secs_f64() * 1_000.0);
    Ok(())
}

fn benchmark(options: &Options) -> Result<(), String> {
    let mut elapsed_ms = 0.0;
    let mut hash = String::new();
    for _ in 0..options.iterations {
        let started = Instant::now();
        let result = generate_synthetic(&SyntheticRequest::new(
            options.seed.as_str(),
            options.width,
            options.height,
        ))
        .map_err(|error| error.to_string())?;
        elapsed_ms += started.elapsed().as_secs_f64() * 1_000.0;
        if hash.is_empty() {
            hash = result.statistics.hash_hex();
        } else if hash != result.statistics.hash_hex() {
            return Err("determinism failure during benchmark".to_owned());
        }
    }
    println!("engine_version={}", WORLDGEN_ENGINE_VERSION);
    println!("iterations={}", options.iterations);
    println!("field={}x{}", options.width, options.height);
    println!("field_hash={hash}");
    println!(
        "mean_elapsed_ms={:.3}",
        elapsed_ms / f64::from(options.iterations)
    );
    Ok(())
}

fn topology(options: &Options) -> Result<(), String> {
    let started = Instant::now();
    let topology = build_icosphere(options.level).map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();
    let metrics = topology.metrics();
    println!("Project Interlink Planet Engine WG-1 topology");
    println!("engine_version={}", WORLDGEN_ENGINE_VERSION);
    println!("level={}", topology.level());
    println!(
        "samples={} edges={} faces={}",
        metrics.sample_count, metrics.edge_count, metrics.face_count
    );
    println!(
        "pentavalent={} hexavalent={}",
        metrics.five_neighbor_count, metrics.six_neighbor_count
    );
    println!("area_sum_sr={:.15}", metrics.total_area_steradians);
    println!(
        "area_min_sr={:.15} area_max_sr={:.15} area_cv={:.9}",
        metrics.minimum_area_steradians,
        metrics.maximum_area_steradians,
        metrics.area_coefficient_of_variation
    );
    println!(
        "edge_min_rad={:.15} edge_max_rad={:.15} edge_cv={:.9}",
        metrics.minimum_edge_arc_radians,
        metrics.maximum_edge_arc_radians,
        metrics.edge_coefficient_of_variation
    );
    println!(
        "interface_min_rad={:.15} interface_max_rad={:.15} interface_cv={:.9}",
        metrics.minimum_interface_arc_radians,
        metrics.maximum_interface_arc_radians,
        metrics.interface_coefficient_of_variation
    );
    println!("topology_hash={}", metrics.topology_hash_hex());
    println!("elapsed_ms={:.3}", elapsed.as_secs_f64() * 1_000.0);
    Ok(())
}

fn tectonics(options: &Options) -> Result<(), String> {
    let started = Instant::now();
    let topology = build_icosphere(options.level).map_err(|error| error.to_string())?;
    let parameters = PlanetPhysicalParameters::earthlike_reference();
    let model = generate_tectonics(
        &topology,
        &TectonicsRequest::new(options.seed.as_str(), options.plates),
        parameters,
    )
    .map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();
    let metrics = &model.metrics;
    println!("Project Interlink Planet Engine WG-2 spherical plate tectonics");
    println!("engine_version={}", WORLDGEN_ENGINE_VERSION);
    println!("stage={}@{}", model.stage.id, model.stage.version);
    println!(
        "seed={} stage_seed={:016x}",
        options.seed, model.stage.derived_seed
    );
    println!(
        "topology_level={} samples={} plates={}",
        options.level, metrics.sample_count, metrics.plate_count
    );
    println!(
        "boundaries={} convergent={} divergent={} transform={}",
        metrics.boundary_edge_count,
        metrics.convergent_edge_count,
        metrics.divergent_edge_count,
        metrics.transform_edge_count
    );
    println!(
        "plate_area_fraction_min={:.6} mean={:.6} max={:.6}",
        metrics.minimum_plate_area_fraction,
        metrics.mean_plate_area_fraction,
        metrics.maximum_plate_area_fraction
    );
    println!(
        "minimum_seed_separation_deg={:.3}",
        metrics.minimum_seed_separation_rad.to_degrees()
    );
    println!(
        "mean_reference_speed_mm_per_year={:.3}",
        metrics.mean_reference_speed_mm_per_year
    );
    println!("tectonic_hash={}", metrics.tectonic_hash_hex());
    println!("elapsed_ms={:.3}", elapsed.as_secs_f64() * 1_000.0);
    Ok(())
}

fn geology(options: &Options) -> Result<(), String> {
    let started = Instant::now();
    let topology = build_icosphere(options.level).map_err(|error| error.to_string())?;
    let parameters = PlanetPhysicalParameters::earthlike_reference();
    let tectonics = generate_tectonics(
        &topology,
        &TectonicsRequest::new(options.seed.as_str(), options.plates),
        parameters,
    )
    .map_err(|error| error.to_string())?;
    let geology = generate_crust_and_history(
        &topology,
        &tectonics,
        &GeologyRequest::new(options.seed.as_str()),
        parameters,
    )
    .map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();
    let metrics = &geology.metrics;
    let major = geology
        .plate_summaries
        .iter()
        .filter(|plate| plate.scale_class == PlateScaleClass::Major)
        .count();
    let intermediate = geology
        .plate_summaries
        .iter()
        .filter(|plate| plate.scale_class == PlateScaleClass::Intermediate)
        .count();
    let minor = geology
        .plate_summaries
        .iter()
        .filter(|plate| plate.scale_class == PlateScaleClass::Minor)
        .count();
    println!("Project Interlink Planet Engine WG-3 crustal state and geological history");
    println!("engine_version={}", WORLDGEN_ENGINE_VERSION);
    println!("stage={}@{}", geology.stage.id, geology.stage.version);
    println!("seed={} stage_seed={:016x} province_seed={:016x} property_seed={:016x} history_seed={:016x}", options.seed, geology.stage.derived_seed, geology.province_seed, geology.property_seed, geology.history_seed);
    println!(
        "topology_level={} samples={} plates={}",
        options.level, metrics.sample_count, options.plates
    );
    println!(
        "crust_area_fraction continental={:.6} transitional={:.6} oceanic={:.6}",
        metrics.continental_area_fraction,
        metrics.transitional_area_fraction,
        metrics.oceanic_area_fraction
    );
    println!(
        "mean_crust_age_myr continental={:.3} oceanic={:.3}",
        metrics.mean_continental_age_myr, metrics.mean_oceanic_age_myr
    );
    println!(
        "mean_crust_thickness_km continental={:.3} oceanic={:.3}",
        metrics.mean_continental_thickness_km, metrics.mean_oceanic_thickness_km
    );
    println!(
        "plate_scale_classes major={} intermediate={} minor={}",
        major, intermediate, minor
    );
    println!("boundary_regimes oceanic_subduction={} ocean_continent_subduction={} continental_collision={} oceanic_ridge={} continental_rift={} transitional_divergence={} transform={}", metrics.oceanic_subduction_edges, metrics.ocean_continent_subduction_edges, metrics.continental_collision_edges, metrics.oceanic_ridge_edges, metrics.continental_rift_edges, metrics.transitional_divergence_edges, metrics.transform_edges);
    println!(
        "tectonic_hash={} geology_hash={}",
        tectonics.metrics.tectonic_hash_hex(),
        metrics.geology_hash_hex()
    );
    println!("elapsed_ms={:.3}", elapsed.as_secs_f64() * 1_000.0);
    Ok(())
}

fn lithosphere(options: &Options) -> Result<(), String> {
    let started = Instant::now();
    let topology = build_icosphere(options.level).map_err(|error| error.to_string())?;
    let parameters = PlanetPhysicalParameters::earthlike_reference();
    let tectonics = generate_tectonics(
        &topology,
        &TectonicsRequest::new(options.seed.as_str(), options.plates),
        parameters,
    )
    .map_err(|error| error.to_string())?;
    let geology = generate_crust_and_history(
        &topology,
        &tectonics,
        &GeologyRequest::new(options.seed.as_str()),
        parameters,
    )
    .map_err(|error| error.to_string())?;
    let lithosphere = generate_lithosphere(
        &topology,
        &tectonics,
        &geology,
        &LithosphereRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();
    let metrics = &lithosphere.metrics;
    let microplates = lithosphere
        .fragments
        .iter()
        .filter(|fragment| fragment.kind == TectonicFragmentKind::Microplate)
        .count();
    let terranes = lithosphere.fragments.len() - microplates;
    println!("Project Interlink Planet Engine WG-3.5 lithosphere and tectonic refinement");
    println!("engine_version={}", WORLDGEN_ENGINE_VERSION);
    println!(
        "stage={}@{}",
        lithosphere.stage.id, lithosphere.stage.version
    );
    println!("seed={} stage_seed={:016x} mechanical_seed={:016x} mantle_seed={:016x} refinement_seed={:016x}", options.seed, lithosphere.stage.derived_seed, lithosphere.mechanical_seed, lithosphere.mantle_seed, lithosphere.refinement_seed);
    println!(
        "topology_level={} samples={} macro_plates={}",
        options.level, metrics.sample_count, options.plates
    );
    println!("mechanics mean_strength={:.5} mean_weakness={:.5} mean_effective_elastic_thickness_km={:.3}", metrics.mean_strength_index, metrics.mean_weakness_index, metrics.mean_effective_elastic_thickness_km);
    println!(
        "mantle mean_upwelling={:.5} mean_dynamic_support={:.5}",
        metrics.mean_mantle_upwelling_index, metrics.mean_dynamic_support_index
    );
    println!(
        "structural_zones sutures={} rifts={} transforms={} continental_margins={}",
        metrics.suture_sample_count,
        metrics.rift_zone_sample_count,
        metrics.transform_zone_sample_count,
        metrics.continental_margin_sample_count
    );
    println!("tectonic_refinement fragments={} microplates={} terranes={} fragmented_area_fraction={:.6}", metrics.tectonic_fragment_count, microplates, terranes, metrics.fragmented_area_fraction);
    for fragment in lithosphere.fragments.iter().take(12) {
        println!("fragment id={} kind={:?} parent_plate={} samples={} parent_area_fraction={:.5} weakness={:.4} propensity={:.4}", fragment.id, fragment.kind, fragment.parent_plate_id, fragment.sample_count, fragment.area_fraction_of_parent, fragment.mean_weakness, fragment.mean_fragmentation_propensity);
    }
    println!(
        "tectonic_hash={} geology_hash={} lithosphere_hash={}",
        tectonics.metrics.tectonic_hash_hex(),
        geology.metrics.geology_hash_hex(),
        metrics.lithosphere_hash_hex()
    );
    println!("elapsed_ms={:.3}", elapsed.as_secs_f64() * 1_000.0);
    Ok(())
}

fn inheritance(options: &Options) -> Result<(), String> {
    if options.coarse_level > options.level {
        return Err("--coarse-level cannot exceed --level".to_owned());
    }
    let started = Instant::now();
    let coarse = build_icosphere(options.coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(options.level).map_err(|error| error.to_string())?;
    let parameters = PlanetPhysicalParameters::earthlike_reference();
    let tectonics = generate_tectonics(
        &coarse,
        &TectonicsRequest::new(options.seed.as_str(), options.plates),
        parameters,
    )
    .map_err(|error| error.to_string())?;
    let geology = generate_crust_and_history(
        &coarse,
        &tectonics,
        &GeologyRequest::new(options.seed.as_str()),
        parameters,
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
        parameters,
    )
    .map_err(|error| error.to_string())?;
    println!("Project Interlink Planet Engine WG-3.75 multiresolution inheritance");
    println!("engine_version={}", WORLDGEN_ENGINE_VERSION);
    println!("stage=foundation:multires-inheritance@1");
    println!("seed={} macro_plates={}", options.seed, options.plates);
    println!(
        "levels coarse={} fine={}",
        options.coarse_level, options.level
    );
    println!(
        "samples coarse={} fine={} added={}",
        inherited.map.metrics.coarse_sample_count,
        inherited.map.metrics.fine_sample_count,
        inherited.map.metrics.added_sample_count
    );
    println!(
        "provenance_hash={} parameter_hash={} inheritance_hash={}",
        inherited.map.metrics.provenance_hash_hex(),
        inherited.parameter_hash_hex(),
        inherited.inheritance_hash_hex()
    );
    println!(
        "upstream tectonic_hash={} geology_hash={} lithosphere_hash={}",
        tectonics.metrics.tectonic_hash_hex(),
        geology.metrics.geology_hash_hex(),
        lithosphere.metrics.lithosphere_hash_hex()
    );
    println!(
        "earthlike equivalent_global_water_depth_m={:.3}",
        parameters.equivalent_global_water_depth_m()
    );
    println!(
        "elapsed_ms={:.3}",
        started.elapsed().as_secs_f64() * 1_000.0
    );
    Ok(())
}

fn profile(_options: &Options) -> Result<(), String> {
    let parameters = PlanetPhysicalParameters::earthlike_reference();
    parameters.validate().map_err(str::to_owned)?;
    println!("Project Interlink Planet Engine physical profile");
    println!(
        "profile=earthlike-default hash={}",
        parameters.parameter_hash_hex()
    );
    println!(
        "radius_m={:.3} gravity_m_s2={:.6} mass_kg={:.6e} mean_bulk_density_kg_m3={:.3}",
        parameters.radius_m,
        parameters.surface_gravity_m_s2,
        parameters.mass_kg(),
        parameters.mean_bulk_density_kg_per_m3()
    );
    println!("rotation_period_s={:.4} axial_tilt_deg={:.6} orbital_period_s={:.3} stellar_flux_w_m2={:.3}", parameters.rotation_period_s, parameters.axial_tilt_rad.to_degrees(), parameters.orbital_period_s, parameters.stellar_flux_w_m2);
    println!("surface_pressure_pa={:.3} surface_water_mass_kg={:.6e} ocean_density_kg_m3={:.3} equivalent_global_water_depth_m={:.3}", parameters.reference_surface_pressure_pa, parameters.surface_water_mass_kg, parameters.ocean_water_density_kg_per_m3, parameters.equivalent_global_water_depth_m());
    println!("isostatic_mantle_density_kg_m3={:.3} internal_heat_flux_w_m2={:.6} mantle_thermal_expansivity_per_k={:.8}", parameters.isostatic_mantle_density_kg_per_m3, parameters.internal_heat_flux_w_per_m2, parameters.mantle_thermal_expansivity_per_k);
    Ok(())
}

fn main() {
    let options = match parse_options() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            process::exit(2);
        }
    };
    let result = match options.command.as_str() {
        "benchmark" => benchmark(&options),
        "topology" => topology(&options),
        "tectonics" => tectonics(&options),
        "geology" => geology(&options),
        "lithosphere" => lithosphere(&options),
        "inheritance" => inheritance(&options),
        "profile" => profile(&options),
        _ => generate_once(&options),
    };
    if let Err(message) = result {
        eprintln!("worldgen error: {message}");
        process::exit(1);
    }
}
