use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    GeologicalBoundaryRegime, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters,
    TectonicsRequest, TopographyRequest,
};

#[derive(Clone, Debug, Default)]
struct RidgeStats {
    edges: u64,
    endpoints: u64,
    land_endpoints: u64,
    both_land_edges: u64,
    mixed_edges: u64,
    submerged_depths_m: Vec<f64>,
}

impl RidgeStats {
    fn record_edge(&mut self, a_land: bool, b_land: bool, a_depth_m: f64, b_depth_m: f64) {
        self.edges += 1;
        self.endpoints += 2;
        self.land_endpoints += u64::from(a_land) + u64::from(b_land);
        self.both_land_edges += u64::from(a_land && b_land);
        self.mixed_edges += u64::from(a_land != b_land);
        if !a_land {
            self.submerged_depths_m.push(a_depth_m);
        }
        if !b_land {
            self.submerged_depths_m.push(b_depth_m);
        }
    }

    fn endpoint_land_fraction(&self) -> f64 {
        self.land_endpoints as f64 / self.endpoints.max(1) as f64
    }

    fn both_land_fraction(&self) -> f64 {
        self.both_land_edges as f64 / self.edges.max(1) as f64
    }

    fn mixed_edge_fraction(&self) -> f64 {
        self.mixed_edges as f64 / self.edges.max(1) as f64
    }

    fn mean_submerged_depth_m(&self) -> f64 {
        if self.submerged_depths_m.is_empty() {
            return 0.0;
        }
        self.submerged_depths_m.iter().sum::<f64>() / self.submerged_depths_m.len() as f64
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

#[derive(Clone, Copy, Debug, Default)]
struct BoundaryEmergence {
    endpoints: u64,
    land_endpoints: u64,
}

impl BoundaryEmergence {
    fn record(&mut self, a_land: bool, b_land: bool) {
        self.endpoints += 2;
        self.land_endpoints += u64::from(a_land) + u64::from(b_land);
    }

    fn land_fraction(self) -> f64 {
        self.land_endpoints as f64 / self.endpoints.max(1) as f64
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

    let mut ridge = RidgeStats::default();
    let mut collision = BoundaryEmergence::default();
    let mut transitional = BoundaryEmergence::default();
    let mut land_fraction_sum = 0.0_f64;
    let mut mean_land_elevation_sum_m = 0.0_f64;
    let mut mean_ocean_depth_sum_m = 0.0_f64;
    let mut maximum_seed_ridge_land_fraction = 0.0_f64;
    let mut maximum_seed_shallow_1km_fraction = 0.0_f64;

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

        let mut seed_ridge = RidgeStats::default();
        for edge in &boundaries.boundaries {
            let a = edge.sample_a as usize;
            let b = edge.sample_b as usize;
            let a_land = terrain.submerged_mask[a] == 0;
            let b_land = terrain.submerged_mask[b] == 0;
            match edge.geological_regime {
                GeologicalBoundaryRegime::OceanicRidge => {
                    let a_depth = f64::from(terrain.water_depth_m[a]);
                    let b_depth = f64::from(terrain.water_depth_m[b]);
                    ridge.record_edge(a_land, b_land, a_depth, b_depth);
                    seed_ridge.record_edge(a_land, b_land, a_depth, b_depth);
                }
                GeologicalBoundaryRegime::ContinentalCollision => {
                    collision.record(a_land, b_land);
                }
                GeologicalBoundaryRegime::TransitionalDivergence => {
                    transitional.record(a_land, b_land);
                }
                _ => {}
            }
        }

        maximum_seed_ridge_land_fraction =
            maximum_seed_ridge_land_fraction.max(seed_ridge.endpoint_land_fraction());
        maximum_seed_shallow_1km_fraction = maximum_seed_shallow_1km_fraction
            .max(seed_ridge.submerged_shallower_fraction(1_000.0));

        if seed == "3" || seed == "interlink-wg7c" {
            println!(
                "seed={seed} land={:.2}% ridge_land={:.2}% ridge_both_land={:.2}% ridge_mixed={:.2}% ridge_submerged_mean={:.0}m ridge_submerged_median={:.0}m shallow500={:.2}% shallow1000={:.2}% shallow2000={:.2}%",
                terrain.metrics.land_area_fraction * 100.0,
                seed_ridge.endpoint_land_fraction() * 100.0,
                seed_ridge.both_land_fraction() * 100.0,
                seed_ridge.mixed_edge_fraction() * 100.0,
                seed_ridge.mean_submerged_depth_m(),
                seed_ridge.median_submerged_depth_m(),
                seed_ridge.submerged_shallower_fraction(500.0) * 100.0,
                seed_ridge.submerged_shallower_fraction(1_000.0) * 100.0,
                seed_ridge.submerged_shallower_fraction(2_000.0) * 100.0,
            );
        }
    }

    let count = seeds.len() as f64;
    println!(
        "aggregate seeds={} land={:.3}% mean_land={:.1}m mean_ocean_depth={:.1}m ridge_land={:.2}% ridge_both_land={:.2}% ridge_mixed={:.2}% ridge_submerged_mean={:.0}m ridge_submerged_median={:.0}m shallow500={:.2}% shallow1000={:.2}% shallow2000={:.2}% max_seed_ridge_land={:.2}% max_seed_shallow1000={:.2}% collision_land={:.2}% transitional_land={:.2}%",
        seeds.len(),
        land_fraction_sum * 100.0 / count,
        mean_land_elevation_sum_m / count,
        mean_ocean_depth_sum_m / count,
        ridge.endpoint_land_fraction() * 100.0,
        ridge.both_land_fraction() * 100.0,
        ridge.mixed_edge_fraction() * 100.0,
        ridge.mean_submerged_depth_m(),
        ridge.median_submerged_depth_m(),
        ridge.submerged_shallower_fraction(500.0) * 100.0,
        ridge.submerged_shallower_fraction(1_000.0) * 100.0,
        ridge.submerged_shallower_fraction(2_000.0) * 100.0,
        maximum_seed_ridge_land_fraction * 100.0,
        maximum_seed_shallow_1km_fraction * 100.0,
        collision.land_fraction() * 100.0,
        transitional.land_fraction() * 100.0,
    );
    Ok(())
}
