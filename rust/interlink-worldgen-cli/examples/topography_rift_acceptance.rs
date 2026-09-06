use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    GeologicalBoundaryRegime, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters,
    TectonicsRequest, TopographyRequest,
};

#[derive(Clone, Debug, Default)]
struct RiftStats {
    edges: u64,
    endpoints: u64,
    submerged_endpoints: u64,
    both_submerged_edges: u64,
    depth_gt_500_endpoints: u64,
    depth_gt_1000_endpoints: u64,
}

impl RiftStats {
    fn flooded_fraction(&self) -> f64 {
        self.submerged_endpoints as f64 / self.endpoints.max(1) as f64
    }

    fn both_flooded_fraction(&self) -> f64 {
        self.both_submerged_edges as f64 / self.edges.max(1) as f64
    }

    fn deep_fraction(&self, count: u64) -> f64 {
        count as f64 / self.endpoints.max(1) as f64
    }

    fn absorb(&mut self, other: &Self) {
        self.edges += other.edges;
        self.endpoints += other.endpoints;
        self.submerged_endpoints += other.submerged_endpoints;
        self.both_submerged_edges += other.both_submerged_edges;
        self.depth_gt_500_endpoints += other.depth_gt_500_endpoints;
        self.depth_gt_1000_endpoints += other.depth_gt_1000_endpoints;
    }
}

fn accumulate_edge(stats: &mut RiftStats, a: usize, b: usize, submerged: &[u8], depth: &[f32]) {
    stats.edges += 1;
    stats.endpoints += 2;
    let a_wet = submerged[a] != 0;
    let b_wet = submerged[b] != 0;
    stats.submerged_endpoints += u64::from(a_wet) + u64::from(b_wet);
    stats.both_submerged_edges += u64::from(a_wet && b_wet);
    for sample in [a, b] {
        if submerged[sample] == 0 {
            continue;
        }
        let water_depth_m = f64::from(depth[sample]);
        stats.depth_gt_500_endpoints += u64::from(water_depth_m > 500.0);
        stats.depth_gt_1000_endpoints += u64::from(water_depth_m > 1_000.0);
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
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;

    let mut continental = RiftStats::default();
    let mut transitional = RiftStats::default();
    let mut visible_seed_continental = None;

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

        let mut seed_continental = RiftStats::default();
        let mut seed_transitional = RiftStats::default();
        for edge in &boundaries.boundaries {
            let a = edge.sample_a as usize;
            let b = edge.sample_b as usize;
            match edge.geological_regime {
                GeologicalBoundaryRegime::ContinentalRift => {
                    accumulate_edge(
                        &mut seed_continental,
                        a,
                        b,
                        &terrain.submerged_mask,
                        &terrain.water_depth_m,
                    );
                }
                GeologicalBoundaryRegime::TransitionalDivergence => {
                    accumulate_edge(
                        &mut seed_transitional,
                        a,
                        b,
                        &terrain.submerged_mask,
                        &terrain.water_depth_m,
                    );
                }
                _ => {}
            }
        }
        if seed == "interlink-wg7c" {
            visible_seed_continental = Some(seed_continental.clone());
        }
        continental.absorb(&seed_continental);
        transitional.absorb(&seed_transitional);
    }

    let visible = visible_seed_continental.ok_or("missing interlink-wg7c rift calibration seed")?;
    let continental_flooded = continental.flooded_fraction();
    let continental_both = continental.both_flooded_fraction();
    let continental_deep_500 = continental.deep_fraction(continental.depth_gt_500_endpoints);
    let continental_deep_1000 = continental.deep_fraction(continental.depth_gt_1000_endpoints);
    let visible_flooded = visible.flooded_fraction();
    let visible_deep_500 = visible.deep_fraction(visible.depth_gt_500_endpoints);
    let transitional_flooded = transitional.flooded_fraction();
    let transitional_both = transitional.both_flooded_fraction();

    println!(
        "WG-4 rift acceptance: continental_flooded={:.2}% continental_both={:.2}% continental_gt500={:.2}% continental_gt1000={:.2}% interlink_wg7c_flooded={:.2}% interlink_wg7c_gt500={:.2}% transitional_flooded={:.2}% transitional_both={:.2}%",
        continental_flooded * 100.0,
        continental_both * 100.0,
        continental_deep_500 * 100.0,
        continental_deep_1000 * 100.0,
        visible_flooded * 100.0,
        visible_deep_500 * 100.0,
        transitional_flooded * 100.0,
        transitional_both * 100.0,
    );

    if continental.edges == 0 || transitional.edges == 0 {
        return Err("rift acceptance ensemble did not produce both continental and transitional divergent boundaries".to_owned());
    }
    if !(0.25..=0.50).contains(&continental_flooded) {
        return Err(format!(
            "continental-rift flooding outside calibrated range: {:.2}%",
            continental_flooded * 100.0
        ));
    }
    if continental_both > 0.50 {
        return Err(format!(
            "continuous flooded continental-rift edges too common: {:.2}%",
            continental_both * 100.0
        ));
    }
    if continental_deep_500 > 0.35 {
        return Err(format!(
            "too many continental-rift endpoints are deeper than 500 m: {:.2}%",
            continental_deep_500 * 100.0
        ));
    }
    if continental_deep_1000 > 0.22 {
        return Err(format!(
            "too many continental-rift endpoints are deeper than 1 km: {:.2}%",
            continental_deep_1000 * 100.0
        ));
    }
    if visible_flooded > 0.40 {
        return Err(format!(
            "interlink-wg7c continental-rift flooding too high: {:.2}%",
            visible_flooded * 100.0
        ));
    }
    if visible_deep_500 > 0.05 {
        return Err(format!(
            "interlink-wg7c retains broad deep continental-rift seaways: {:.2}% deeper than 500 m",
            visible_deep_500 * 100.0
        ));
    }
    if transitional_flooded < 0.70 {
        return Err(format!(
            "transitional-divergence seaways became too dry: {:.2}%",
            transitional_flooded * 100.0
        ));
    }
    if transitional_both < 0.55 {
        return Err(format!(
            "continuous transitional-divergence seaways became too rare: {:.2}%",
            transitional_both * 100.0
        ));
    }

    Ok(())
}
