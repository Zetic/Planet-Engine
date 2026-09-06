use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    GeologicalBoundaryRegime, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters,
    TectonicsRequest, TopographyRequest,
};

#[derive(Clone, Debug, Default)]
struct BoundaryStats {
    edges: u64,
    endpoints: u64,
    land_endpoints: u64,
    both_land_edges: u64,
    submerged_depths_m: Vec<f64>,
}

impl BoundaryStats {
    fn endpoint_land_fraction(&self) -> f64 {
        self.land_endpoints as f64 / self.endpoints.max(1) as f64
    }

    fn both_land_fraction(&self) -> f64 {
        self.both_land_edges as f64 / self.edges.max(1) as f64
    }

    fn mean_submerged_depth_m(&self) -> f64 {
        if self.submerged_depths_m.is_empty() {
            0.0
        } else {
            self.submerged_depths_m.iter().sum::<f64>() / self.submerged_depths_m.len() as f64
        }
    }

    fn median_submerged_depth_m(&self) -> f64 {
        if self.submerged_depths_m.is_empty() {
            return 0.0;
        }
        let mut values = self.submerged_depths_m.clone();
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    }

    fn submerged_shallower_fraction(&self, threshold_m: f64) -> f64 {
        if self.submerged_depths_m.is_empty() {
            return 0.0;
        }
        self.submerged_depths_m
            .iter()
            .filter(|&&depth| depth < threshold_m)
            .count() as f64
            / self.submerged_depths_m.len() as f64
    }
}

fn main() -> Result<(), String> {
    let seeds = [
        "3",
        "interlink-wg7c",
        "wg4-boundary-a",
        "wg4-boundary-b",
        "wg4-boundary-c",
        "wg4-boundary-d",
    ];
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;

    let mut ridge = BoundaryStats::default();
    let mut collision = BoundaryStats::default();
    let mut land_fraction_sum = 0.0_f64;
    let mut mean_land_elevation_sum_m = 0.0_f64;
    let mut mean_ocean_depth_sum_m = 0.0_f64;
    let mut maximum_seed_ridge_land_fraction = 0.0_f64;
    let mut maximum_seed_ridge_both_land_fraction = 0.0_f64;

    for seed in seeds {
        let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, plates), planet)
            .map_err(|error| error.to_string())?;
        let geology =
            generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet)
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
        let boundaries =
            inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids)
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
        mean_land_elevation_sum_m += terrain.metrics.mean_land_elevation_m;
        mean_ocean_depth_sum_m += terrain.metrics.mean_water_depth_m;

        let mut seed_ridge = BoundaryStats::default();
        for edge in &boundaries.boundaries {
            let a = edge.sample_a as usize;
            let b = edge.sample_b as usize;
            let a_land = terrain.submerged_mask[a] == 0;
            let b_land = terrain.submerged_mask[b] == 0;
            let target = match edge.geological_regime {
                GeologicalBoundaryRegime::OceanicRidge => Some((&mut ridge, &mut seed_ridge)),
                GeologicalBoundaryRegime::ContinentalCollision => {
                    collision.edges += 1;
                    collision.endpoints += 2;
                    collision.land_endpoints += u64::from(a_land) + u64::from(b_land);
                    collision.both_land_edges += u64::from(a_land && b_land);
                    None
                }
                _ => None,
            };
            let Some((aggregate, seed_stats)) = target else {
                continue;
            };
            for stats in [aggregate, seed_stats] {
                stats.edges += 1;
                stats.endpoints += 2;
                stats.land_endpoints += u64::from(a_land) + u64::from(b_land);
                stats.both_land_edges += u64::from(a_land && b_land);
                for sample in [a, b] {
                    if terrain.submerged_mask[sample] != 0 {
                        stats.submerged_depths_m
                            .push(f64::from(terrain.water_depth_m[sample]));
                    }
                }
            }
        }
        maximum_seed_ridge_land_fraction =
            maximum_seed_ridge_land_fraction.max(seed_ridge.endpoint_land_fraction());
        maximum_seed_ridge_both_land_fraction =
            maximum_seed_ridge_both_land_fraction.max(seed_ridge.both_land_fraction());
    }

    let count = seeds.len() as f64;
    let mean_land_fraction = land_fraction_sum / count;
    let mean_land_elevation_m = mean_land_elevation_sum_m / count;
    let mean_ocean_depth_m = mean_ocean_depth_sum_m / count;
    let ridge_land_fraction = ridge.endpoint_land_fraction();
    let ridge_both_land_fraction = ridge.both_land_fraction();
    let ridge_submerged_depth_m = ridge.mean_submerged_depth_m();
    let ridge_submerged_median_m = ridge.median_submerged_depth_m();
    let ridge_shallow_500_fraction = ridge.submerged_shallower_fraction(500.0);
    let ridge_shallow_1000_fraction = ridge.submerged_shallower_fraction(1_000.0);
    let ridge_shallow_2000_fraction = ridge.submerged_shallower_fraction(2_000.0);
    let collision_land_fraction = collision.endpoint_land_fraction();

    println!(
        "WG-4 boundary acceptance: land={:.3}% mean_land={:.1}m mean_ocean_depth={:.1}m ridge_land={:.2}% ridge_both_land={:.2}% ridge_submerged_mean={:.1}m ridge_submerged_median={:.1}m ridge_shallow500={:.2}% ridge_shallow1000={:.2}% ridge_shallow2000={:.2}% max_seed_ridge_land={:.2}% max_seed_ridge_both_land={:.2}% collision_land={:.2}%",
        mean_land_fraction * 100.0,
        mean_land_elevation_m,
        mean_ocean_depth_m,
        ridge_land_fraction * 100.0,
        ridge_both_land_fraction * 100.0,
        ridge_submerged_depth_m,
        ridge_submerged_median_m,
        ridge_shallow_500_fraction * 100.0,
        ridge_shallow_1000_fraction * 100.0,
        ridge_shallow_2000_fraction * 100.0,
        maximum_seed_ridge_land_fraction * 100.0,
        maximum_seed_ridge_both_land_fraction * 100.0,
        collision_land_fraction * 100.0,
    );

    if !(0.20..=0.33).contains(&mean_land_fraction) {
        return Err(format!(
            "mean land fraction out of calibrated range: {mean_land_fraction:.4}"
        ));
    }
    if !(1_000.0..=1_800.0).contains(&mean_land_elevation_m) {
        return Err(format!(
            "mean land elevation out of calibrated range: {mean_land_elevation_m:.1} m"
        ));
    }
    if !(3_200.0..=4_100.0).contains(&mean_ocean_depth_m) {
        return Err(format!(
            "mean ocean depth out of calibrated range: {mean_ocean_depth_m:.1} m"
        ));
    }
    if ridge_land_fraction > 0.18 {
        return Err(format!(
            "oceanic-ridge endpoint emergence too high: {:.2}%",
            ridge_land_fraction * 100.0
        ));
    }
    if ridge_both_land_fraction > 0.08 {
        return Err(format!(
            "continuous emergent oceanic-ridge edges too common: {:.2}%",
            ridge_both_land_fraction * 100.0
        ));
    }
    if ridge_submerged_depth_m < 1_200.0 {
        return Err(format!(
            "submerged oceanic ridge crests are too shallow on average: {ridge_submerged_depth_m:.1} m"
        ));
    }
    if ridge_submerged_median_m < 1_100.0 {
        return Err(format!(
            "median submerged oceanic ridge crest is too shallow: {ridge_submerged_median_m:.1} m"
        ));
    }
    if ridge_shallow_500_fraction > 0.15 {
        return Err(format!(
            "too much submerged oceanic ridge is shallower than 500 m: {:.2}%",
            ridge_shallow_500_fraction * 100.0
        ));
    }
    if ridge_shallow_1000_fraction > 0.45 {
        return Err(format!(
            "too much submerged oceanic ridge is shallower than 1 km: {:.2}%",
            ridge_shallow_1000_fraction * 100.0
        ));
    }
    if ridge_shallow_2000_fraction > 0.92 {
        return Err(format!(
            "too much submerged oceanic ridge is shallower than 2 km: {:.2}%",
            ridge_shallow_2000_fraction * 100.0
        ));
    }
    if maximum_seed_ridge_land_fraction > 0.25 {
        return Err(format!(
            "single-seed oceanic-ridge emergence too high: {:.2}%",
            maximum_seed_ridge_land_fraction * 100.0
        ));
    }
    if maximum_seed_ridge_both_land_fraction > 0.15 {
        return Err(format!(
            "single-seed continuous emergent ridge fraction too high: {:.2}%",
            maximum_seed_ridge_both_land_fraction * 100.0
        ));
    }
    if collision_land_fraction < 0.85 {
        return Err(format!(
            "continental collision belts lost expected emergence: {:.2}%",
            collision_land_fraction * 100.0
        ));
    }

    Ok(())
}
