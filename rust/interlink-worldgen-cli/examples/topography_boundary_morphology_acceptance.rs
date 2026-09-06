use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    GeologicalBoundaryRegime, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters,
    TectonicsRequest, TopographyRequest,
};
use std::collections::{HashMap, HashSet};

const CRUST_CONTINENTAL: u8 = 3;

#[derive(Clone, Debug, Default)]
struct Stats {
    oceanic_context_edges: u64,
    oceanic_context_both_land: u64,
    oceanic_context_either_land: u64,
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

    let mut graph_edges: HashMap<u8, Vec<(u32, u32)>> = HashMap::new();
    for edge in &boundaries.boundaries {
        let a = edge.sample_a as usize;
        let b = edge.sample_b as usize;
        let oceanic_context = inherited.crust_kind[a] != CRUST_CONTINENTAL
            && inherited.crust_kind[b] != CRUST_CONTINENTAL;
        if !oceanic_context {
            continue;
        }
        let land_a = terrain.submerged_mask[a] == 0;
        let land_b = terrain.submerged_mask[b] == 0;
        let stats = out.get_mut(&(edge.geological_regime as u8)).unwrap();
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

    for regime in regimes() {
        let key = regime as u8;
        let edges = graph_edges.remove(&key).unwrap_or_default();
        let mut adjacency: HashMap<u32, Vec<usize>> = HashMap::new();
        for (index, (a, b)) in edges.iter().copied().enumerate() {
            adjacency.entry(a).or_default().push(index);
            adjacency.entry(b).or_default().push(index);
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
                        for next_edge in next {
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

fn percentage(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        100.0 * numerator as f64 / denominator as f64
    }
}

fn assert_regime(
    aggregate: &HashMap<u8, Stats>,
    regime: GeologicalBoundaryRegime,
    maximum_any_land_percent: f64,
    maximum_both_land_percent: f64,
    maximum_chain_edges: u64,
) -> Result<(), String> {
    let stats = aggregate.get(&(regime as u8)).unwrap();
    let any_land = percentage(
        stats.oceanic_context_either_land,
        stats.oceanic_context_edges,
    );
    let both_land = percentage(stats.oceanic_context_both_land, stats.oceanic_context_edges);
    println!(
        "{:28} ocean_edges={} ocean_any={:.2}% ocean_both={:.2}% chain_edges={} chains={} max_chain={}",
        label(regime),
        stats.oceanic_context_edges,
        any_land,
        both_land,
        stats.chain_edges_total,
        stats.chain_components,
        stats.max_chain_edges
    );
    if any_land > maximum_any_land_percent {
        return Err(format!(
            "{} oceanic-context emergence {:.2}% exceeds {:.2}%",
            label(regime),
            any_land,
            maximum_any_land_percent
        ));
    }
    if both_land > maximum_both_land_percent {
        return Err(format!(
            "{} both-land boundary rate {:.2}% exceeds {:.2}%",
            label(regime),
            both_land,
            maximum_both_land_percent
        ));
    }
    if stats.max_chain_edges > maximum_chain_edges {
        return Err(format!(
            "{} emergent boundary chain {} edges exceeds {}",
            label(regime),
            stats.max_chain_edges,
            maximum_chain_edges
        ));
    }
    Ok(())
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
    let coarse = build_icosphere(5).map_err(|error| error.to_string())?;
    let fine = build_icosphere(7).map_err(|error| error.to_string())?;
    let mut aggregate: HashMap<u8, Stats> = HashMap::new();

    for seed in seeds {
        let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, 16), planet)
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
        let inherited =
            inherit_physical_state(&fine, 5, &tectonics, &geology, &lithosphere, planet)
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
        let stats = profile(&inherited, &boundaries, &terrain);
        for regime in regimes() {
            let source = stats.get(&(regime as u8)).unwrap();
            let target = aggregate.entry(regime as u8).or_default();
            target.oceanic_context_edges += source.oceanic_context_edges;
            target.oceanic_context_both_land += source.oceanic_context_both_land;
            target.oceanic_context_either_land += source.oceanic_context_either_land;
            target.chain_components += source.chain_components;
            target.chain_edges_total += source.chain_edges_total;
            target.max_chain_edges = target.max_chain_edges.max(source.max_chain_edges);
        }
    }

    println!("WG-4 all-regime boundary morphology acceptance:");
    assert_regime(
        &aggregate,
        GeologicalBoundaryRegime::OceanicSubduction,
        0.50,
        0.25,
        2,
    )?;
    assert_regime(
        &aggregate,
        GeologicalBoundaryRegime::OceanContinentSubduction,
        1.00,
        0.25,
        2,
    )?;
    assert_regime(
        &aggregate,
        GeologicalBoundaryRegime::ContinentalCollision,
        15.0,
        0.50,
        2,
    )?;
    assert_regime(
        &aggregate,
        GeologicalBoundaryRegime::OceanicRidge,
        1.00,
        0.50,
        2,
    )?;
    assert_regime(
        &aggregate,
        GeologicalBoundaryRegime::TransitionalDivergence,
        1.00,
        0.50,
        2,
    )?;
    assert_regime(
        &aggregate,
        GeologicalBoundaryRegime::Transform,
        2.00,
        0.50,
        2,
    )?;

    Ok(())
}
