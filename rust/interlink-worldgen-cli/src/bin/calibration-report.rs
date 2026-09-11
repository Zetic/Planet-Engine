use interlink_worldgen::{
    build_icosphere, build_world_calibration_report, generate_bounded_terrain_evolution,
    generate_coupled_climate_with_diagnostics, generate_crust_and_history,
    generate_drainage_topology, generate_fluvial_erosion_sediment, generate_initial_topography,
    generate_lake_sediment_infill, generate_lakes_closed_basins, generate_lithosphere,
    generate_post_erosion_hydrology, generate_runoff_discharge, generate_seasonal_hydrology,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, ClimateRequest,
    DrainageRequest, FluvialErosionRequest, GeologyRequest, LakeRequest,
    LakeSedimentInfillRequest, LithosphereRequest, PlanetPhysicalParameters,
    PostErosionHydrologyRequest, RunoffRequest, SeasonalHydrologyRequest, TectonicsRequest,
    TerrainEvolutionRequest, TopographyRequest,
};
use std::{env, process};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Format {
    Json,
    Markdown,
}

#[derive(Debug)]
struct Options {
    seed: String,
    level: u8,
    coarse_level: u8,
    plates: u16,
    format: Format,
}

fn usage() -> &'static str {
    "calibration-report [--seed TEXT] [--level N] [--coarse-level N] [--plates N] [--format json|markdown]"
}

fn next_u32(name: &str, value: Option<String>) -> Result<u32, String> {
    value
        .ok_or_else(|| format!("{name} requires a value"))?
        .parse::<u32>()
        .map_err(|_| format!("{name} requires an unsigned integer"))
}

fn parse_options() -> Result<Options, String> {
    let mut options = Options {
        seed: "worldgen-calibration".to_owned(),
        level: 6,
        coarse_level: 4,
        plates: 16,
        format: Format::Markdown,
    };
    let mut args = env::args().skip(1);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--seed" => {
                options.seed = args
                    .next()
                    .ok_or_else(|| "--seed requires a value".to_owned())?;
            }
            "--level" => {
                options.level = u8::try_from(next_u32("--level", args.next())?)
                    .map_err(|_| "--level exceeds u8 range".to_owned())?;
            }
            "--coarse-level" => {
                options.coarse_level = u8::try_from(next_u32("--coarse-level", args.next())?)
                    .map_err(|_| "--coarse-level exceeds u8 range".to_owned())?;
            }
            "--plates" => {
                options.plates = u16::try_from(next_u32("--plates", args.next())?)
                    .map_err(|_| "--plates exceeds u16 range".to_owned())?;
            }
            "--format" => {
                options.format = match args.next().as_deref() {
                    Some("json") => Format::Json,
                    Some("markdown") | Some("md") => Format::Markdown,
                    Some(value) => return Err(format!("unsupported --format '{value}'")),
                    None => return Err("--format requires json or markdown".to_owned()),
                };
            }
            "-h" | "--help" => return Err(usage().to_owned()),
            _ => return Err(format!("unsupported option '{flag}'\n{}", usage())),
        }
    }
    if options.coarse_level > options.level {
        return Err("--coarse-level cannot exceed --level".to_owned());
    }
    Ok(options)
}

fn run(options: &Options) -> Result<String, String> {
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
    let climate_request = ClimateRequest::new(options.seed.as_str());
    let mut climate_progress = |_completed: u8, _maximum: u8| {};
    let (climate, climate_diagnostics) = generate_coupled_climate_with_diagnostics(
        &fine,
        &terrain,
        planet,
        &climate_request,
        &mut climate_progress,
    )
    .map_err(|error| error.to_string())?;
    let drainage = generate_drainage_topology(
        &fine,
        &terrain,
        planet,
        &DrainageRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let runoff = generate_runoff_discharge(
        &fine,
        &terrain,
        &climate,
        &drainage,
        planet,
        &RunoffRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let lakes = generate_lakes_closed_basins(
        &fine,
        &terrain,
        &climate,
        &drainage,
        &runoff,
        planet,
        &LakeRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let seasonal = generate_seasonal_hydrology(
        &fine,
        &terrain,
        &climate,
        &climate_diagnostics,
        &drainage,
        &runoff,
        &lakes,
        planet,
        &SeasonalHydrologyRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let erosion = generate_fluvial_erosion_sediment(
        &fine,
        &inherited,
        &terrain,
        &drainage,
        &lakes,
        &seasonal,
        planet,
        &FluvialErosionRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let evolution = generate_bounded_terrain_evolution(
        &fine,
        &terrain,
        &drainage,
        &runoff,
        &lakes,
        &erosion,
        planet,
        &TerrainEvolutionRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let reconciliation = generate_post_erosion_hydrology(
        &fine,
        &terrain,
        &climate,
        &climate_diagnostics,
        &drainage,
        &runoff,
        &lakes,
        &seasonal,
        &evolution,
        planet,
        &PostErosionHydrologyRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let infill = generate_lake_sediment_infill(
        &fine,
        &terrain,
        &climate,
        &climate_diagnostics,
        &drainage,
        &lakes,
        &erosion,
        &evolution,
        &reconciliation,
        planet,
        &LakeSedimentInfillRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;

    let report = build_world_calibration_report(
        options.seed.as_str(),
        options.plates,
        &fine,
        &inherited,
        &terrain,
        &climate,
        &erosion,
        &evolution,
        &infill,
        planet,
        &coarse.metrics().topology_hash_hex(),
        &tectonics.metrics.tectonic_hash_hex(),
        &geology.metrics.geology_hash_hex(),
        &lithosphere.metrics.lithosphere_hash_hex(),
    )
    .map_err(|error| error.to_string())?;

    Ok(match options.format {
        Format::Json => report.to_json_pretty(),
        Format::Markdown => report.to_markdown(),
    })
}

fn main() {
    let options = match parse_options() {
        Ok(options) => options,
        Err(message) if message == usage() => {
            println!("{message}");
            process::exit(0);
        }
        Err(message) => {
            eprintln!("{message}");
            process::exit(2);
        }
    };
    match run(&options) {
        Ok(report) => print!("{report}"),
        Err(message) => {
            eprintln!("worldgen error: {message}");
            process::exit(1);
        }
    }
}
