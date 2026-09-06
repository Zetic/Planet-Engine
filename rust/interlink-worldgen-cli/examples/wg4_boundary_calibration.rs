use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, GeologicalBoundaryRegime,
    GeologyRequest, LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyParameters,
    TopographyRequest,
};
use std::collections::BTreeMap;

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

#[derive(Clone, Copy, Debug, Default)]
struct GlobalStats {
    land_fraction_sum: f64,
    mean_land_elevation_sum_m: f64,
    mean_ocean_depth_sum_m: f64,
    p95_sum_m: f64,
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

    let mut profiles = Vec::new();
    let baseline = TopographyParameters::default();
    profiles.push(("baseline-2000x600", baseline));
    let mut moderate = baseline;
    moderate.ridge_uplift_scale_m = 1_500.0;
    moderate.ridge_width_m = 500_000.0;
    profiles.push(("ridge-1500x500", moderate));
    let mut low = baseline;
    low.ridge_uplift_scale_m = 1_200.0;
    low.ridge_width_m = 450_000.0;
    profiles.push(("ridge-1200x450", low));
    let mut narrow = baseline;
    narrow.ridge_uplift_scale_m = 1_400.0;
    narrow.ridge_width_m = 400_000.0;
    profiles.push(("ridge-1400x400", narrow));

    println!("WG-4 boundary-emergence ridge profile comparison");
    println!("coarse=L{coarse_level} fine=L{fine_level} plates={plates} seeds={}", seeds.len());

    let mut aggregate: BTreeMap<&'static str, (GlobalStats, BTreeMap<&'static str, RegimeStats>)> =
        BTreeMap::new();

    for seed in seeds {
        let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, plates), planet)
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

        for (profile, parameters) in &profiles {
            let request = TopographyRequest {
                seed: seed.to_owned(),
                parameters: *parameters,
            };
            let terrain = generate_initial_topography(&fine, &inherited, &boundaries, planet, &request)
                .map_err(|error| error.to_string())?;
            let entry = aggregate.entry(*profile).or_default();
            entry.0.land_fraction_sum += terrain.metrics.land_area_fraction;
            entry.0.mean_land_elevation_sum_m += terrain.metrics.mean_land_elevation_m;
            entry.0.mean_ocean_depth_sum_m += terrain.metrics.mean_water_depth_m;
            entry.0.p95_sum_m += terrain.metrics.p95_solid_elevation_m;

            let mut seed_ridge = RegimeStats::default();
            for edge in &boundaries.boundaries {
                let stats = entry.1.entry(label(edge.geological_regime)).or_default();
                stats.edges += 1;
                let a = edge.sample_a as usize;
                let b = edge.sample_b as usize;
                let a_land = terrain.submerged_mask[a] == 0;
                let b_land = terrain.submerged_mask[b] == 0;
                stats.endpoints += 2;
                stats.land_endpoints += u64::from(a_land) + u64::from(b_land);
                stats.both_land_edges += u64::from(a_land && b_land);
                for sample in [a, b] {
                    stats.relative_elevation_sum_m +=
                        f64::from(terrain.elevation_above_sea_level_m[sample]);
                    if terrain.submerged_mask[sample] != 0 {
                        stats.submerged_depth_sum_m += f64::from(terrain.water_depth_m[sample]);
                        stats.submerged_endpoints += 1;
                    }
                }
                if edge.geological_regime == GeologicalBoundaryRegime::OceanicRidge {
                    seed_ridge.edges += 1;
                    seed_ridge.endpoints += 2;
                    seed_ridge.land_endpoints += u64::from(a_land) + u64::from(b_land);
                    seed_ridge.both_land_edges += u64::from(a_land && b_land);
                }
            }
            if seed == "3" {
                println!(
                    "seed=3 profile={profile} land={:.2}% oceanic_ridge_endpoint_land={:.2}% both_land={:.2}%",
                    terrain.metrics.land_area_fraction * 100.0,
                    seed_ridge.land_endpoints as f64 * 100.0 / seed_ridge.endpoints as f64,
                    seed_ridge.both_land_edges as f64 * 100.0 / seed_ridge.edges as f64,
                );
            }
        }
    }

    let n = seeds.len() as f64;
    for (profile, (global, regimes)) in &aggregate {
        println!("profile={profile}");
        println!(
            "  global land={:.3}% mean_land={:.1}m mean_ocean_depth={:.1}m p95={:.1}m",
            global.land_fraction_sum * 100.0 / n,
            global.mean_land_elevation_sum_m / n,
            global.mean_ocean_depth_sum_m / n,
            global.p95_sum_m / n,
        );
        for (name, stats) in regimes {
            let land_pct = stats.land_endpoints as f64 * 100.0 / stats.endpoints as f64;
            let both_land_pct = stats.both_land_edges as f64 * 100.0 / stats.edges as f64;
            let mean_relative = stats.relative_elevation_sum_m / stats.endpoints as f64;
            let mean_submerged_depth = if stats.submerged_endpoints == 0 {
                0.0
            } else {
                stats.submerged_depth_sum_m / stats.submerged_endpoints as f64
            };
            println!(
                "  {name}: endpoint_land={land_pct:.2}% both_land={both_land_pct:.2}% mean_rel={mean_relative:.1}m mean_submerged_depth={mean_submerged_depth:.1}m"
            );
        }
    }
    Ok(())
}
