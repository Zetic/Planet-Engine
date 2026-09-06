use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    GeologicalBoundaryRegime, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters,
    TectonicsRequest, TopographyRequest,
};

fn main() -> Result<(), String> {
    let seeds = ["3", "interlink-wg7c", "wg4-boundary-a", "wg4-boundary-b", "wg4-boundary-c", "wg4-boundary-d"];
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(5).map_err(|e| e.to_string())?;
    let fine = build_icosphere(7).map_err(|e| e.to_string())?;
    let mut edges = 0_u64;
    let mut endpoints = 0_u64;
    let mut flooded = 0_u64;
    let mut both_flooded = 0_u64;
    let mut depths = Vec::new();
    let mut crust = [0_u64; 4];
    for seed in seeds {
        let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, 16), planet).map_err(|e| e.to_string())?;
        let geology = generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet).map_err(|e| e.to_string())?;
        let lithosphere = generate_lithosphere(&coarse, &tectonics, &geology, &LithosphereRequest::new(seed)).map_err(|e| e.to_string())?;
        let inherited = inherit_physical_state(&fine, 5, &tectonics, &geology, &lithosphere, planet).map_err(|e| e.to_string())?;
        let boundaries = inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids).map_err(|e| e.to_string())?;
        let terrain = generate_initial_topography(&fine, &inherited, &boundaries, planet, &TopographyRequest::new(seed)).map_err(|e| e.to_string())?;
        let mut seed_edges = 0_u64;
        let mut seed_endpoints = 0_u64;
        let mut seed_flooded = 0_u64;
        let mut seed_both = 0_u64;
        for edge in &boundaries.boundaries {
            if edge.geological_regime != GeologicalBoundaryRegime::TransitionalDivergence { continue; }
            edges += 1;
            seed_edges += 1;
            let mut pair = true;
            for sample in [edge.sample_a as usize, edge.sample_b as usize] {
                endpoints += 1;
                seed_endpoints += 1;
                let k = inherited.crust_kind[sample] as usize;
                if k < crust.len() { crust[k] += 1; }
                if terrain.submerged_mask[sample] != 0 {
                    flooded += 1;
                    seed_flooded += 1;
                    depths.push(f64::from(terrain.water_depth_m[sample]));
                } else { pair = false; }
            }
            both_flooded += u64::from(pair);
            seed_both += u64::from(pair);
        }
        println!("{seed}: transition_edges={seed_edges} flooded={:.2}% both_flooded={:.2}%", 100.0 * seed_flooded as f64 / seed_endpoints.max(1) as f64, 100.0 * seed_both as f64 / seed_edges.max(1) as f64);
    }
    depths.sort_by(f64::total_cmp);
    let mean = if depths.is_empty() { 0.0 } else { depths.iter().sum::<f64>() / depths.len() as f64 };
    let median = if depths.is_empty() { 0.0 } else { depths[depths.len()/2] };
    println!("AGGREGATE_TRANSITION: edges={edges} flooded={:.2}% both_flooded={:.2}% submerged_mean={mean:.1}m submerged_median={median:.1}m crust0/1/2/3={}/{}/{}/{}", 100.0 * flooded as f64 / endpoints.max(1) as f64, 100.0 * both_flooded as f64 / edges.max(1) as f64, crust[0], crust[1], crust[2], crust[3]);
    Ok(())
}
