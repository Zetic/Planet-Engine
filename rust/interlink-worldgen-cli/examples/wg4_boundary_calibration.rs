use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, GeologicalBoundaryRegime,
    GeologyRequest, LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Default)]
struct RegimeStats {
    edges: u64,
    endpoints: u64,
    land_endpoints: u64,
    both_land_edges: u64,
    relative_elevation_sum_m: f64,
    submerged_depth_sum_m: f64,
    submerged_endpoints: u64,
}

fn label(regime: GeologicalBoundaryRegime) -> &'static str {
    match regime {
        GeologicalBoundaryRegime::ContinentalCollision => "continental_collision",
        GeologicalBoundaryRegime::OceanicRidge => "oceanic_ridge",
        GeologicalBoundaryRegime::TransitionalDivergence => "transitional_divergence",
        GeologicalBoundaryRegime::ContinentalRift => "continental_rift",
        GeologicalBoundaryRegime::OceanicSubduction => "oceanic_subduction",
        GeologicalBoundaryRegime::OceanContinentSubduction => "ocean_continent_subduction",
        GeologicalBoundaryRegime::Transform => "transform",
    }
}

fn main() -> Result<(), String> {
    let seeds = ["3", "wg4-boundary-a", "wg4-boundary-b", "wg4-boundary-c", "wg4-boundary-d"];
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;
    let mut aggregate: BTreeMap<&'static str, RegimeStats> = BTreeMap::new();
    let mut land_fraction_sum = 0.0_f64;
    let mut mean_land_elevation_sum = 0.0_f64;
    let mut mean_ocean_depth_sum = 0.0_f64;

    println!("WG-4 boundary-emergence calibration baseline");
    println!("coarse=L{coarse_level} fine=L{fine_level} plates={plates} seeds={}", seeds.len());

    for seed in seeds {
        let tectonics = generate_tectonics(
            &coarse,
            &TectonicsRequest::new(seed, plates),
            planet,
        )
        .map_err(|error| error.to_string())?;
        let geology = generate_crust_and_history(
            &coarse,
            &tectonics,
            &GeologyRequest::new(seed),
            planet,
        )
        .map_err(|error| error.to_string())?;
        let lithosphere = generate_lithosphere(
            &coarse,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;
        let inherited = inherit_physical_state(
            &fine,
            coarse_level,
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
            &TopographyRequest::new(seed),
        )
        .map_err(|error| error.to_string())?;

        land_fraction_sum += terrain.metrics.land_area_fraction;
        mean_land_elevation_sum += terrain.metrics.mean_land_elevation_m;
        mean_ocean_depth_sum += terrain.metrics.mean_water_depth_m;

        println!(
            "seed={seed} land={:.3}% mean_land={:.1}m mean_ocean_depth={:.1}m p95={:.1}m boundaries={}",
            terrain.metrics.land_area_fraction * 100.0,
            terrain.metrics.mean_land_elevation_m,
            terrain.metrics.mean_water_depth_m,
            terrain.metrics.p95_solid_elevation_m,
            boundaries.boundaries.len()
        );

        let mut per_seed: BTreeMap<&'static str, RegimeStats> = BTreeMap::new();
        let mut unique_boundary_samples = BTreeSet::new();
        let mut unique_land_boundary_samples = BTreeSet::new();

        for edge in &boundaries.boundaries {
            let name = label(edge.geological_regime);
            let stats = per_seed.entry(name).or_default();
            stats.edges += 1;
            let a = edge.sample_a as usize;
            let b = edge.sample_b as usize;
            let a_land = terrain.submerged_mask[a] == 0;
            let b_land = terrain.submerged_mask[b] == 0;
            stats.endpoints += 2;
            stats.land_endpoints += u64::from(a_land) + u64::from(b_land);
            stats.both_land_edges += u64::from(a_land && b_land);
            for sample in [a, b] {
                stats.relative_elevation_sum_m += f64::from(terrain.elevation_above_sea_level_m[sample]);
                if terrain.submerged_mask[sample] != 0 {
                    stats.submerged_depth_sum_m += f64::from(terrain.water_depth_m[sample]);
                    stats.submerged_endpoints += 1;
                }
                unique_boundary_samples.insert(sample);
                if terrain.submerged_mask[sample] == 0 {
                    unique_land_boundary_samples.insert(sample);
                }
            }
        }

        let unique_fraction = if unique_boundary_samples.is_empty() {
            0.0
        } else {
            unique_land_boundary_samples.len() as f64 / unique_boundary_samples.len() as f64
        };
        println!(
            "  unique_boundary_samples={} land_unique={:.2}%",
            unique_boundary_samples.len(),
            unique_fraction * 100.0
        );

        for (name, stats) in &per_seed {
            let land_pct = if stats.endpoints == 0 { 0.0 } else { stats.land_endpoints as f64 * 100.0 / stats.endpoints as f64 };
            let both_land_pct = if stats.edges == 0 { 0.0 } else { stats.both_land_edges as f64 * 100.0 / stats.edges as f64 };
            let mean_relative = if stats.endpoints == 0 { 0.0 } else { stats.relative_elevation_sum_m / stats.endpoints as f64 };
            let mean_submerged_depth = if stats.submerged_endpoints == 0 { 0.0 } else { stats.submerged_depth_sum_m / stats.submerged_endpoints as f64 };
            println!(
                "  {name}: edges={} endpoint_land={land_pct:.2}% both_land={both_land_pct:.2}% mean_rel={mean_relative:.1}m mean_submerged_depth={mean_submerged_depth:.1}m",
                stats.edges
            );
            let total = aggregate.entry(*name).or_default();
            total.edges += stats.edges;
            total.endpoints += stats.endpoints;
            total.land_endpoints += stats.land_endpoints;
            total.both_land_edges += stats.both_land_edges;
            total.relative_elevation_sum_m += stats.relative_elevation_sum_m;
            total.submerged_depth_sum_m += stats.submerged_depth_sum_m;
            total.submerged_endpoints += stats.submerged_endpoints;
        }
    }

    let n = seeds.len() as f64;
    println!("aggregate:");
    println!(
        "  land={:.3}% mean_land={:.1}m mean_ocean_depth={:.1}m",
        land_fraction_sum * 100.0 / n,
        mean_land_elevation_sum / n,
        mean_ocean_depth_sum / n
    );
    for (name, stats) in &aggregate {
        let land_pct = if stats.endpoints == 0 { 0.0 } else { stats.land_endpoints as f64 * 100.0 / stats.endpoints as f64 };
        let both_land_pct = if stats.edges == 0 { 0.0 } else { stats.both_land_edges as f64 * 100.0 / stats.edges as f64 };
        let mean_relative = if stats.endpoints == 0 { 0.0 } else { stats.relative_elevation_sum_m / stats.endpoints as f64 };
        let mean_submerged_depth = if stats.submerged_endpoints == 0 { 0.0 } else { stats.submerged_depth_sum_m / stats.submerged_endpoints as f64 };
        println!(
            "  {name}: edges={} endpoint_land={land_pct:.2}% both_land={both_land_pct:.2}% mean_rel={mean_relative:.1}m mean_submerged_depth={mean_submerged_depth:.1}m",
            stats.edges
        );
    }
    Ok(())
}
