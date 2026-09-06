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
    depth_gt_100_endpoints: u64,
    depth_gt_500_endpoints: u64,
    depth_gt_1000_endpoints: u64,
    immature_endpoints: u64,
    immature_submerged: u64,
    mature_endpoints: u64,
    mature_submerged: u64,
    crust_kind_counts: [u64; 4],
    submerged_depths_m: Vec<f64>,
}

impl RiftStats {
    fn endpoint_submerged_fraction(&self) -> f64 {
        self.submerged_endpoints as f64 / self.endpoints.max(1) as f64
    }
    fn both_submerged_fraction(&self) -> f64 {
        self.both_submerged_edges as f64 / self.edges.max(1) as f64
    }
    fn depth_fraction(&self, count: u64) -> f64 {
        count as f64 / self.endpoints.max(1) as f64
    }
    fn mean_submerged_depth_m(&self) -> f64 {
        if self.submerged_depths_m.is_empty() { 0.0 } else {
            self.submerged_depths_m.iter().sum::<f64>() / self.submerged_depths_m.len() as f64
        }
    }
    fn median_submerged_depth_m(&self) -> f64 {
        if self.submerged_depths_m.is_empty() { return 0.0; }
        let mut values = self.submerged_depths_m.clone();
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    }
    fn immature_submerged_fraction(&self) -> f64 {
        self.immature_submerged as f64 / self.immature_endpoints.max(1) as f64
    }
    fn mature_submerged_fraction(&self) -> f64 {
        self.mature_submerged as f64 / self.mature_endpoints.max(1) as f64
    }
    fn absorb(&mut self, other: &RiftStats) {
        self.edges += other.edges;
        self.endpoints += other.endpoints;
        self.submerged_endpoints += other.submerged_endpoints;
        self.both_submerged_edges += other.both_submerged_edges;
        self.depth_gt_100_endpoints += other.depth_gt_100_endpoints;
        self.depth_gt_500_endpoints += other.depth_gt_500_endpoints;
        self.depth_gt_1000_endpoints += other.depth_gt_1000_endpoints;
        self.immature_endpoints += other.immature_endpoints;
        self.immature_submerged += other.immature_submerged;
        self.mature_endpoints += other.mature_endpoints;
        self.mature_submerged += other.mature_submerged;
        for i in 0..self.crust_kind_counts.len() { self.crust_kind_counts[i] += other.crust_kind_counts[i]; }
        self.submerged_depths_m.extend(other.submerged_depths_m.iter().copied());
    }
}

fn collect(seed: &str, coarse_level: u8, fine_level: u8, plates: u16) -> Result<(RiftStats, f64, f64, f64), String> {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|e| e.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|e| e.to_string())?;
    let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, plates), planet).map_err(|e| e.to_string())?;
    let geology = generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet).map_err(|e| e.to_string())?;
    let lithosphere = generate_lithosphere(&coarse, &tectonics, &geology, &LithosphereRequest::new(seed)).map_err(|e| e.to_string())?;
    let inherited = inherit_physical_state(&fine, coarse_level, &tectonics, &geology, &lithosphere, planet).map_err(|e| e.to_string())?;
    let boundaries = inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids).map_err(|e| e.to_string())?;
    let terrain = generate_initial_topography(&fine, &inherited, &boundaries, planet, &TopographyRequest::new(seed)).map_err(|e| e.to_string())?;

    let mut stats = RiftStats::default();
    for edge in &boundaries.boundaries {
        if edge.geological_regime != GeologicalBoundaryRegime::ContinentalRift { continue; }
        stats.edges += 1;
        let mut submerged_pair = true;
        for sample in [edge.sample_a as usize, edge.sample_b as usize] {
            stats.endpoints += 1;
            let history = f64::from(inherited.rift_history[sample]);
            if history < 0.45 { stats.immature_endpoints += 1; } else if history >= 0.70 { stats.mature_endpoints += 1; }
            let kind = inherited.crust_kind[sample] as usize;
            if kind < stats.crust_kind_counts.len() { stats.crust_kind_counts[kind] += 1; }
            if terrain.submerged_mask[sample] != 0 {
                stats.submerged_endpoints += 1;
                if history < 0.45 { stats.immature_submerged += 1; } else if history >= 0.70 { stats.mature_submerged += 1; }
                let depth = f64::from(terrain.water_depth_m[sample]);
                stats.submerged_depths_m.push(depth);
                stats.depth_gt_100_endpoints += u64::from(depth > 100.0);
                stats.depth_gt_500_endpoints += u64::from(depth > 500.0);
                stats.depth_gt_1000_endpoints += u64::from(depth > 1_000.0);
            } else {
                submerged_pair = false;
            }
        }
        stats.both_submerged_edges += u64::from(submerged_pair);
    }
    Ok((stats, terrain.metrics.land_area_fraction, terrain.metrics.mean_land_elevation_m, terrain.metrics.mean_water_depth_m))
}

fn print_stats(label: &str, s: &RiftStats, land: f64, mean_land: f64, ocean_depth: f64) {
    println!(
        "{label}: edges={} endpoints={} flooded={:.2}% both_flooded={:.2}% depth>100={:.2}% depth>500={:.2}% depth>1000={:.2}% submerged_mean={:.1}m submerged_median={:.1}m immature_flooded={:.2}% mature_flooded={:.2}% crust0/1/2/3={}/{}/{}/{} land={:.2}% mean_land={:.1}m ocean_depth={:.1}m",
        s.edges,
        s.endpoints,
        100.0 * s.endpoint_submerged_fraction(),
        100.0 * s.both_submerged_fraction(),
        100.0 * s.depth_fraction(s.depth_gt_100_endpoints),
        100.0 * s.depth_fraction(s.depth_gt_500_endpoints),
        100.0 * s.depth_fraction(s.depth_gt_1000_endpoints),
        s.mean_submerged_depth_m(),
        s.median_submerged_depth_m(),
        100.0 * s.immature_submerged_fraction(),
        100.0 * s.mature_submerged_fraction(),
        s.crust_kind_counts[0], s.crust_kind_counts[1], s.crust_kind_counts[2], s.crust_kind_counts[3],
        100.0 * land, mean_land, ocean_depth,
    );
}

fn main() -> Result<(), String> {
    let seeds = ["3", "interlink-wg7c", "wg4-boundary-a", "wg4-boundary-b", "wg4-boundary-c", "wg4-boundary-d"];
    let mut aggregate = RiftStats::default();
    let mut land_sum = 0.0;
    let mut mean_land_sum = 0.0;
    let mut ocean_depth_sum = 0.0;
    for seed in seeds {
        let (stats, land, mean_land, ocean_depth) = collect(seed, 5, 7, 16)?;
        print_stats(seed, &stats, land, mean_land, ocean_depth);
        aggregate.absorb(&stats);
        land_sum += land;
        mean_land_sum += mean_land;
        ocean_depth_sum += ocean_depth;
    }
    let n = seeds.len() as f64;
    print_stats("AGGREGATE", &aggregate, land_sum / n, mean_land_sum / n, ocean_depth_sum / n);
    Ok(())
}
