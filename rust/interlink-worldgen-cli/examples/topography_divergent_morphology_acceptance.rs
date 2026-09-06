use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    GeologicalBoundaryRegime, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters,
    TectonicsRequest, TopographyRequest,
};

const CRUST_CONTINENTAL: u8 = 3;
const LARGE_RIDGE_COMPONENT_AREA_M2: f64 = 25_000.0 * 1_000_000.0;

#[derive(Clone, Debug, Default)]
struct RidgeIslandStats {
    component_count: u64,
    large_component_count: u64,
    total_area_m2: f64,
    maximum_area_m2: f64,
    ridge_land_endpoints: u64,
}

impl RidgeIslandStats {
    fn absorb(&mut self, other: &Self) {
        self.component_count += other.component_count;
        self.large_component_count += other.large_component_count;
        self.total_area_m2 += other.total_area_m2;
        self.maximum_area_m2 = self.maximum_area_m2.max(other.maximum_area_m2);
        self.ridge_land_endpoints += other.ridge_land_endpoints;
    }
}

fn ridge_island_stats(
    topology: &interlink_worldgen::GeodesicTopology,
    inherited: &interlink_worldgen::InheritedPhysicalState,
    boundaries: &interlink_worldgen::InheritedBoundarySet,
    terrain: &interlink_worldgen::TopographyState,
    radius_m: f64,
) -> RidgeIslandStats {
    let count = topology.metrics().sample_count as usize;
    let mut ridge_endpoint = vec![false; count];
    for edge in &boundaries.boundaries {
        if edge.geological_regime == GeologicalBoundaryRegime::OceanicRidge {
            ridge_endpoint[edge.sample_a as usize] = true;
            ridge_endpoint[edge.sample_b as usize] = true;
        }
    }

    let mut visited = vec![false; count];
    let mut stats = RidgeIslandStats::default();
    for start in 0..count {
        if visited[start] || terrain.submerged_mask[start] != 0 {
            continue;
        }
        let mut stack = vec![start as u32];
        visited[start] = true;
        let mut area_m2 = 0.0_f64;
        let mut has_continental = false;
        let mut touches_ridge = false;
        let mut ridge_endpoints = 0_u64;

        while let Some(sample) = stack.pop() {
            let index = sample as usize;
            area_m2 += topology.dual_area_steradians()[index] * radius_m * radius_m;
            has_continental |= inherited.crust_kind[index] == CRUST_CONTINENTAL;
            if ridge_endpoint[index] {
                touches_ridge = true;
                ridge_endpoints += 1;
            }
            for neighbor in topology.neighbors_of(sample) {
                let neighbor_index = *neighbor as usize;
                if !visited[neighbor_index] && terrain.submerged_mask[neighbor_index] == 0 {
                    visited[neighbor_index] = true;
                    stack.push(*neighbor);
                }
            }
        }

        if touches_ridge && !has_continental {
            stats.component_count += 1;
            stats.total_area_m2 += area_m2;
            stats.maximum_area_m2 = stats.maximum_area_m2.max(area_m2);
            stats.ridge_land_endpoints += ridge_endpoints;
            if area_m2 >= LARGE_RIDGE_COMPONENT_AREA_M2 {
                stats.large_component_count += 1;
            }
        }
    }
    stats
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
    let mut aggregate = RidgeIslandStats::default();

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
        let stats = ridge_island_stats(&fine, &inherited, &boundaries, &terrain, planet.radius_m);
        println!(
            "{seed}: ridge_islands={} ridge_large={} ridge_area_km2={:.0} ridge_max_km2={:.0} ridge_land_endpoints={}",
            stats.component_count,
            stats.large_component_count,
            stats.total_area_m2 / 1.0e6,
            stats.maximum_area_m2 / 1.0e6,
            stats.ridge_land_endpoints,
        );
        aggregate.absorb(&stats);
    }

    println!(
        "WG-4 divergent morphology acceptance: ridge_islands={} ridge_large={} ridge_area_km2={:.0} ridge_max_km2={:.0} ridge_land_endpoints={}",
        aggregate.component_count,
        aggregate.large_component_count,
        aggregate.total_area_m2 / 1.0e6,
        aggregate.maximum_area_m2 / 1.0e6,
        aggregate.ridge_land_endpoints,
    );

    if aggregate.component_count > 155 {
        return Err(
            "WG-4 pure-oceanic ridge land remains too fragmented into emergent island components"
                .into(),
        );
    }
    if aggregate.large_component_count > 90 {
        return Err("WG-4 produces too many large emergent pure-oceanic ridge components".into());
    }
    if aggregate.total_area_m2 > 7_500_000.0 * 1.0e6 {
        return Err("WG-4 emergent pure-oceanic ridge area remains too large across the calibration ensemble".into());
    }
    if aggregate.maximum_area_m2 > 275_000.0 * 1.0e6 {
        return Err("WG-4 permits an oversized noncontinental ridge-island component".into());
    }
    if aggregate.ridge_land_endpoints > 700 {
        return Err("WG-4 leaves too many pure-oceanic ridge endpoints emergent".into());
    }

    Ok(())
}
