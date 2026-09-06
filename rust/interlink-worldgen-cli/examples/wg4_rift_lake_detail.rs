use interlink_worldgen::{
    build_icosphere, generate_bounded_terrain_evolution, generate_coupled_climate_with_diagnostics,
    generate_crust_and_history, generate_drainage_topology, generate_fluvial_erosion_sediment,
    generate_initial_topography, generate_lake_sediment_infill, generate_lakes_closed_basins,
    generate_lithosphere, generate_post_erosion_hydrology, generate_runoff_discharge,
    generate_seasonal_hydrology, generate_tectonics, inherit_boundary_interfaces,
    inherit_physical_state, ClimateRequest, DrainageRequest, FluvialErosionRequest,
    GeologicalBoundaryRegime, GeologyRequest, LakeRequest, LakeSedimentInfillRequest,
    LithosphereRequest, PlanetPhysicalParameters, PostErosionHydrologyRequest, RunoffRequest,
    SeasonalHydrologyRequest, TectonicsRequest, TerrainEvolutionRequest, TopographyRequest,
};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

const CRUST_CONTINENTAL: u8 = 3;
const RIFT_RADIUS_M: f64 = 300_000.0;

#[derive(Clone, Copy, Debug)]
struct QueueEntry {
    distance_m: f64,
    sample: u32,
}
impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.distance_m.to_bits() == other.distance_m.to_bits() && self.sample == other.sample
    }
}
impl Eq for QueueEntry {}
impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance_m.total_cmp(&self.distance_m).then_with(|| other.sample.cmp(&self.sample))
    }
}

fn rift_distances(
    topology: &interlink_worldgen::GeodesicTopology,
    boundaries: &interlink_worldgen::InheritedBoundarySet,
    radius_m: f64,
) -> Vec<f64> {
    let count = topology.metrics().sample_count as usize;
    let mut distance = vec![f64::INFINITY; count];
    let mut queue = BinaryHeap::new();
    for edge in &boundaries.boundaries {
        if edge.geological_regime != GeologicalBoundaryRegime::ContinentalRift { continue; }
        for sample in [edge.sample_a, edge.sample_b] {
            let i = sample as usize;
            if distance[i] != 0.0 {
                distance[i] = 0.0;
                queue.push(QueueEntry { distance_m: 0.0, sample });
            }
        }
    }
    while let Some(entry) = queue.pop() {
        let i = entry.sample as usize;
        if entry.distance_m > distance[i] + 1e-6 || entry.distance_m > RIFT_RADIUS_M { continue; }
        for (neighbor, arc) in topology.neighbors_of(entry.sample).iter().zip(topology.neighbor_arc_lengths_of(entry.sample)) {
            let candidate = entry.distance_m + *arc * radius_m;
            let n = *neighbor as usize;
            if candidate <= RIFT_RADIUS_M && candidate + 1e-6 < distance[n] {
                distance[n] = candidate;
                queue.push(QueueEntry { distance_m: candidate, sample: *neighbor });
            }
        }
    }
    distance
}

fn main() -> Result<(), String> {
    let seed = "interlink-wg7c";
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|e| e.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|e| e.to_string())?;
    let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, plates), planet).map_err(|e| e.to_string())?;
    let geology = generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet).map_err(|e| e.to_string())?;
    let lithosphere = generate_lithosphere(&coarse, &tectonics, &geology, &LithosphereRequest::new(seed)).map_err(|e| e.to_string())?;
    let inherited = inherit_physical_state(&fine, coarse_level, &tectonics, &geology, &lithosphere, planet).map_err(|e| e.to_string())?;
    let boundaries = inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids).map_err(|e| e.to_string())?;
    let terrain = generate_initial_topography(&fine, &inherited, &boundaries, planet, &TopographyRequest::new(seed)).map_err(|e| e.to_string())?;
    let mut progress = |_done: u8, _max: u8| {};
    let (climate, climate_diagnostics) = generate_coupled_climate_with_diagnostics(
        &fine, &terrain, planet, &ClimateRequest::new(seed), &mut progress,
    ).map_err(|e| e.to_string())?;
    let drainage = generate_drainage_topology(&fine, &terrain, planet, &DrainageRequest::new(seed)).map_err(|e| e.to_string())?;
    let runoff = generate_runoff_discharge(&fine, &terrain, &climate, &drainage, planet, &RunoffRequest::new(seed)).map_err(|e| e.to_string())?;
    let lakes = generate_lakes_closed_basins(&fine, &terrain, &climate, &drainage, &runoff, planet, &LakeRequest::new(seed)).map_err(|e| e.to_string())?;
    let seasonal = generate_seasonal_hydrology(&fine, &terrain, &climate, &climate_diagnostics, &drainage, &runoff, &lakes, planet, &SeasonalHydrologyRequest::new(seed)).map_err(|e| e.to_string())?;
    let erosion = generate_fluvial_erosion_sediment(&fine, &inherited, &terrain, &drainage, &lakes, &seasonal, planet, &FluvialErosionRequest::new(seed)).map_err(|e| e.to_string())?;
    let evolution = generate_bounded_terrain_evolution(&fine, &terrain, &drainage, &runoff, &lakes, &erosion, planet, &TerrainEvolutionRequest::new(seed)).map_err(|e| e.to_string())?;
    let reconciliation = generate_post_erosion_hydrology(&fine, &terrain, &climate, &climate_diagnostics, &drainage, &runoff, &lakes, &seasonal, &evolution, planet, &PostErosionHydrologyRequest::new(seed)).map_err(|e| e.to_string())?;
    let infill = generate_lake_sediment_infill(&fine, &terrain, &climate, &climate_diagnostics, &drainage, &lakes, &erosion, &evolution, &reconciliation, planet, &LakeSedimentInfillRequest::new(seed)).map_err(|e| e.to_string())?;

    let rift_distance = rift_distances(&fine, &boundaries, planet.radius_m);
    let final_lakes = &infill.reconciled_lakes;
    let mut total_area_by_id: HashMap<u32, f64> = HashMap::new();
    let mut rift_area_by_id: HashMap<u32, f64> = HashMap::new();
    let mut min_final_surface_by_id: HashMap<u32, f64> = HashMap::new();
    let mut max_final_surface_by_id: HashMap<u32, f64> = HashMap::new();
    for i in 0..final_lakes.lake_fraction.len() {
        let fraction = f64::from(final_lakes.lake_fraction[i]);
        if fraction <= 0.0 { continue; }
        let id = final_lakes.lake_id[i];
        let area = fine.dual_area_steradians()[i] * planet.radius_m * planet.radius_m * fraction;
        *total_area_by_id.entry(id).or_insert(0.0) += area;
        if inherited.crust_kind[i] == CRUST_CONTINENTAL && rift_distance[i] <= RIFT_RADIUS_M {
            *rift_area_by_id.entry(id).or_insert(0.0) += area;
        }
        let z = f64::from(infill.post_infill_solid_elevation_m[i]);
        min_final_surface_by_id.entry(id).and_modify(|v| *v = v.min(z)).or_insert(z);
        max_final_surface_by_id.entry(id).and_modify(|v| *v = v.max(z)).or_insert(z);
    }

    let mut rows = final_lakes.lakes.iter().filter_map(|record| {
        let total = *total_area_by_id.get(&record.id)?;
        let rift = rift_area_by_id.get(&record.id).copied().unwrap_or(0.0);
        if total <= 0.0 || rift / total < 0.50 { return None; }
        Some((total, rift / total, record))
    }).collect::<Vec<_>>();
    rows.sort_by(|a, b| b.0.total_cmp(&a.0));

    println!("interlink-wg7c final rift-dominated lakes: count={}", rows.len());
    for (rank, (area, rift_share, record)) in rows.iter().take(10).enumerate() {
        println!(
            "rank={} id={} kind={} area_km2={:.0} rift_share={:.1}% volume_km3={:.0} max_depth_m={:.1} surface_m={:.1} floor_min_m={:.1} floor_max_m={:.1} inflow_m3_s={:.1} evap_m3_s={:.1} outflow_m3_s={:.1} storage_m3_s={:.1} spill_sample={} spill_receiver={}",
            rank + 1,
            record.id,
            record.kind,
            area / 1.0e6,
            rift_share * 100.0,
            record.volume_m3 / 1.0e9,
            record.maximum_depth_m,
            record.surface_elevation_m,
            min_final_surface_by_id.get(&record.id).copied().unwrap_or(f64::NAN),
            max_final_surface_by_id.get(&record.id).copied().unwrap_or(f64::NAN),
            record.gross_land_inflow_m3_s,
            record.lake_evaporation_m3_s,
            record.outflow_m3_s,
            record.unreleased_storage_m3_s,
            record.spill_sample,
            record.spill_receiver,
        );
    }
    Ok(())
}
