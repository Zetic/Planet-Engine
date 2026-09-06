use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, GeologicalBoundaryRegime,
    GeologyRequest, LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest,
    TopographyRequest,
};
use std::collections::{HashMap, HashSet};

const CRUST_CONTINENTAL: u8 = 3;

#[derive(Clone, Debug, Default)]
struct Stats {
    edges: u64,
    either_land: u64,
    both_land: u64,
    oceanic_context_edges: u64,
    oceanic_context_both_land: u64,
    oceanic_context_either_land: u64,
    land_endpoints: u64,
    endpoint_count: u64,
    chain_components: u64,
    max_chain_edges: u64,
    chain_edges_total: u64,
}

fn label(regime: GeologicalBoundaryRegime) -> &'static str {
    match regime {
        GeologicalBoundaryRegime::OceanicSubduction => "oceanic_subduction",
        GeologicalBoundaryRegime::OceanContinentSubduction => "ocean_continent_subduction",
        GeologicalBoundaryRegime::ContinentalCollision => "continental_collision",
        GeologicalBoundaryRegime::OceanicRidge => "oceanic_ridge",
        GeologicalBoundaryRegime::ContinentalRift => "continental_rift",
        GeologicalBoundaryRegime::TransitionalDivergence => "transitional_divergence",
        GeologicalBoundaryRegime::Transform => "transform",
    }
}

fn regimes() -> [GeologicalBoundaryRegime; 7] {
    [
        GeologicalBoundaryRegime::OceanicSubduction,
        GeologicalBoundaryRegime::OceanContinentSubduction,
        GeologicalBoundaryRegime::ContinentalCollision,
        GeologicalBoundaryRegime::OceanicRidge,
        GeologicalBoundaryRegime::ContinentalRift,
        GeologicalBoundaryRegime::TransitionalDivergence,
        GeologicalBoundaryRegime::Transform,
    ]
}

fn profile(
    inherited: &interlink_worldgen::InheritedPhysicalState,
    boundaries: &interlink_worldgen::InheritedBoundarySet,
    terrain: &interlink_worldgen::TopographyState,
) -> HashMap<u8, Stats> {
    let mut out = HashMap::<u8, Stats>::new();
    for regime in regimes() {
        out.insert(regime as u8, Stats::default());
    }

    // Graph only the *oceanic-context, both-land* boundary edges. This directly measures the
    // dotted/continuous island-chain artifact visible from final coastline morphology.
    let mut graph_edges: HashMap<u8, Vec<(u32, u32)>> = HashMap::new();

    for edge in &boundaries.boundaries {
        let a = edge.sample_a as usize;
        let b = edge.sample_b as usize;
        let land_a = terrain.submerged_mask[a] == 0;
        let land_b = terrain.submerged_mask[b] == 0;
        let oceanic_context = inherited.crust_kind[a] != CRUST_CONTINENTAL
            && inherited.crust_kind[b] != CRUST_CONTINENTAL;
        let stats = out.get_mut(&(edge.geological_regime as u8)).unwrap();
        stats.edges += 1;
        stats.endpoint_count += 2;
        stats.land_endpoints += u64::from(land_a) + u64::from(land_b);
        stats.either_land += u64::from(land_a || land_b);
        stats.both_land += u64::from(land_a && land_b);
        if oceanic_context {
            stats.oceanic_context_edges += 1;
            stats.oceanic_context_either_land += u64::from(land_a || land_b);
            stats.oceanic_context_both_land += u64::from(land_a && land_b);
            if land_a && land_b {
                graph_edges
                    .entry(edge.geological_regime as u8)
                    .or_default()
                    .push((edge.sample_a, edge.sample_b));
            }
        }
    }

    for regime in regimes() {
        let key = regime as u8;
        let edges = graph_edges.remove(&key).unwrap_or_default();
        let mut adjacency: HashMap<u32, Vec<(u32, usize)>> = HashMap::new();
        for (index, (a, b)) in edges.iter().copied().enumerate() {
            adjacency.entry(a).or_default().push((b, index));
            adjacency.entry(b).or_default().push((a, index));
        }
        let mut seen_edges = HashSet::<usize>::new();
        let mut components = 0_u64;
        let mut max_edges = 0_u64;
        for start_edge in 0..edges.len() {
            if seen_edges.contains(&start_edge) {
                continue;
            }
            components += 1;
            let mut stack = vec![start_edge];
            let mut count = 0_u64;
            while let Some(edge_index) = stack.pop() {
                if !seen_edges.insert(edge_index) {
                    continue;
                }
                count += 1;
                let (a, b) = edges[edge_index];
                for sample in [a, b] {
                    if let Some(next) = adjacency.get(&sample) {
                        for (_, next_edge) in next {
                            if !seen_edges.contains(next_edge) {
                                stack.push(*next_edge);
                            }
                        }
                    }
                }
            }
            max_edges = max_edges.max(count);
        }
        let stats = out.get_mut(&key).unwrap();
        stats.chain_components = components;
        stats.max_chain_edges = max_edges;
        stats.chain_edges_total = edges.len() as u64;
    }

    out
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
    let coarse = build_icosphere(coarse_level).map_err(|e| e.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|e| e.to_string())?;
    let mut aggregate: HashMap<u8, Stats> = HashMap::new();

    for seed in seeds {
        let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, plates), planet)
            .map_err(|e| e.to_string())?;
        let geology = generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet)
            .map_err(|e| e.to_string())?;
        let lithosphere = generate_lithosphere(
            &coarse,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed),
        )
        .map_err(|e| e.to_string())?;
        let inherited = inherit_physical_state(
            &fine,
            coarse_level,
            &tectonics,
            &geology,
            &lithosphere,
            planet,
        )
        .map_err(|e| e.to_string())?;
        let boundaries = inherit_boundary_interfaces(
            &coarse,
            &fine,
            &tectonics,
            &geology,
            &inherited.plate_ids,
        )
        .map_err(|e| e.to_string())?;
        let terrain = generate_initial_topography(
            &fine,
            &inherited,
            &boundaries,
            planet,
            &TopographyRequest::new(seed),
        )
        .map_err(|e| e.to_string())?;
        let stats = profile(&inherited, &boundaries, &terrain);

        println!("SEED {seed} land={:.2}%", terrain.metrics.land_area_fraction * 100.0);
        for regime in regimes() {
            let s = stats.get(&(regime as u8)).unwrap();
            let all_both = if s.edges > 0 { 100.0 * s.both_land as f64 / s.edges as f64 } else { 0.0 };
            let ocean_both = if s.oceanic_context_edges > 0 { 100.0 * s.oceanic_context_both_land as f64 / s.oceanic_context_edges as f64 } else { 0.0 };
            let ocean_any = if s.oceanic_context_edges > 0 { 100.0 * s.oceanic_context_either_land as f64 / s.oceanic_context_edges as f64 } else { 0.0 };
            println!(
                "  {:28} edges={} both_land={:.1}% ocean_edges={} ocean_any={:.1}% ocean_both={:.1}% chain_edges={} chains={} max_chain={}",
                label(regime), s.edges, all_both, s.oceanic_context_edges, ocean_any, ocean_both,
                s.chain_edges_total, s.chain_components, s.max_chain_edges
            );
            let a = aggregate.entry(regime as u8).or_default();
            a.edges += s.edges;
            a.either_land += s.either_land;
            a.both_land += s.both_land;
            a.oceanic_context_edges += s.oceanic_context_edges;
            a.oceanic_context_both_land += s.oceanic_context_both_land;
            a.oceanic_context_either_land += s.oceanic_context_either_land;
            a.land_endpoints += s.land_endpoints;
            a.endpoint_count += s.endpoint_count;
            a.chain_components += s.chain_components;
            a.max_chain_edges = a.max_chain_edges.max(s.max_chain_edges);
            a.chain_edges_total += s.chain_edges_total;
        }
    }

    println!("AGGREGATE");
    for regime in regimes() {
        let s = aggregate.get(&(regime as u8)).unwrap();
        let ocean_both = if s.oceanic_context_edges > 0 { 100.0 * s.oceanic_context_both_land as f64 / s.oceanic_context_edges as f64 } else { 0.0 };
        let ocean_any = if s.oceanic_context_edges > 0 { 100.0 * s.oceanic_context_either_land as f64 / s.oceanic_context_edges as f64 } else { 0.0 };
        println!(
            "  {:28} ocean_edges={} ocean_any={:.2}% ocean_both={:.2}% chain_edges={} chains={} max_chain={}",
            label(regime), s.oceanic_context_edges, ocean_any, ocean_both,
            s.chain_edges_total, s.chain_components, s.max_chain_edges
        );
    }

    Ok(())
}
