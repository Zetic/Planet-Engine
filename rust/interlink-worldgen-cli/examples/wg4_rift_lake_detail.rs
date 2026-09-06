use interlink_worldgen::{
    build_icosphere, generate_bounded_terrain_evolution, generate_coupled_climate_with_diagnostics,
    generate_crust_and_history, generate_drainage_topology, generate_fluvial_erosion_sediment,
    generate_initial_topography, generate_lake_sediment_infill, generate_lakes_closed_basins,
    generate_lithosphere, generate_post_erosion_hydrology, generate_runoff_discharge,
    generate_seasonal_hydrology, generate_tectonics, inherit_boundary_interfaces,
    inherit_physical_state, ClimateRequest, DrainageRequest, FluvialErosionRequest,
    GeologicalBoundaryRegime, GeologyRequest, LakeRequest, LakeSedimentInfillRequest, LakeState,
    LithosphereRequest, PlanetPhysicalParameters, PostErosionHydrologyRequest, RunoffRequest,
    SeasonalHydrologyRequest, TectonicsRequest, TerrainEvolutionRequest, TopographyRequest,
};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance_m
            .total_cmp(&self.distance_m)
            .then_with(|| other.sample.cmp(&self.sample))
    }
}

#[derive(Clone, Copy, Debug)]
struct RiftLakeRow {
    depression_id: u32,
    record_id: u32,
    kind: u8,
    area_m2: f64,
    rift_share: f64,
    volume_m3: f64,
    maximum_depth_m: f64,
    surface_elevation_m: f64,
    outflow_m3_s: f64,
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
        if edge.geological_regime != GeologicalBoundaryRegime::ContinentalRift {
            continue;
        }
        for sample in [edge.sample_a, edge.sample_b] {
            let i = sample as usize;
            if distance[i] != 0.0 {
                distance[i] = 0.0;
                queue.push(QueueEntry {
                    distance_m: 0.0,
                    sample,
                });
            }
        }
    }
    while let Some(entry) = queue.pop() {
        let i = entry.sample as usize;
        if entry.distance_m > distance[i] + 1.0e-6 || entry.distance_m > RIFT_RADIUS_M {
            continue;
        }
        for (neighbor, arc) in topology
            .neighbors_of(entry.sample)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(entry.sample))
        {
            let candidate = entry.distance_m + *arc * radius_m;
            let n = *neighbor as usize;
            if candidate <= RIFT_RADIUS_M && candidate + 1.0e-6 < distance[n] {
                distance[n] = candidate;
                queue.push(QueueEntry {
                    distance_m: candidate,
                    sample: *neighbor,
                });
            }
        }
    }
    distance
}

fn rift_lake_rows(
    topology: &interlink_worldgen::GeodesicTopology,
    inherited: &interlink_worldgen::InheritedPhysicalState,
    rift_distance_m: &[f64],
    lakes: &LakeState,
    radius_m: f64,
) -> Vec<RiftLakeRow> {
    let mut total_area_by_depression = HashMap::<u32, f64>::new();
    let mut rift_area_by_depression = HashMap::<u32, f64>::new();
    for i in 0..lakes.lake_fraction.len() {
        let fraction = f64::from(lakes.lake_fraction[i]);
        if fraction <= 0.0 {
            continue;
        }
        let depression_id = lakes.lake_id[i];
        let area_m2 = topology.dual_area_steradians()[i] * radius_m * radius_m * fraction;
        *total_area_by_depression.entry(depression_id).or_default() += area_m2;
        if inherited.crust_kind[i] == CRUST_CONTINENTAL && rift_distance_m[i] <= RIFT_RADIUS_M {
            *rift_area_by_depression.entry(depression_id).or_default() += area_m2;
        }
    }

    let mut rows = lakes
        .lakes
        .iter()
        .filter_map(|record| {
            let area_m2 = *total_area_by_depression.get(&record.depression_id)?;
            let rift_area_m2 = rift_area_by_depression
                .get(&record.depression_id)
                .copied()
                .unwrap_or(0.0);
            let rift_share = rift_area_m2 / area_m2.max(1.0);
            if rift_share < 0.50 {
                return None;
            }
            Some(RiftLakeRow {
                depression_id: record.depression_id,
                record_id: record.id,
                kind: record.kind,
                area_m2,
                rift_share,
                volume_m3: record.volume_m3,
                maximum_depth_m: record.maximum_depth_m,
                surface_elevation_m: record.surface_elevation_m,
                outflow_m3_s: record.outflow_m3_s,
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| right.area_m2.total_cmp(&left.area_m2));
    rows
}

fn print_stage(label: &str, rows: &[RiftLakeRow]) {
    let total_area_m2 = rows.iter().map(|row| row.area_m2).sum::<f64>();
    println!(
        "{label}: rift_lakes={} rift_area_km2={:.0}",
        rows.len(),
        total_area_m2 / 1.0e6
    );
    for (rank, row) in rows.iter().take(5).enumerate() {
        println!(
            "  rank={} depression={} record={} kind={} area_km2={:.0} rift_share={:.1}% volume_km3={:.0} max_depth_m={:.1} surface_m={:.1} outflow_m3_s={:.1}",
            rank + 1,
            row.depression_id,
            row.record_id,
            row.kind,
            row.area_m2 / 1.0e6,
            row.rift_share * 100.0,
            row.volume_m3 / 1.0e9,
            row.maximum_depth_m,
            row.surface_elevation_m,
            row.outflow_m3_s,
        );
    }
}

fn footprint_overlap(
    label: &str,
    topology: &interlink_worldgen::GeodesicTopology,
    target_fraction: &[f32],
    lakes: &LakeState,
    radius_m: f64,
) {
    let mut wet_overlap_m2 = 0.0_f64;
    let mut target_area_m2 = 0.0_f64;
    let mut stage_ids = HashSet::<u32>::new();
    let mut by_stage_depression = HashMap::<u32, f64>::new();
    for i in 0..target_fraction.len() {
        let target = f64::from(target_fraction[i]);
        if target <= 0.0 {
            continue;
        }
        let cell_area = topology.dual_area_steradians()[i] * radius_m * radius_m;
        target_area_m2 += cell_area * target;
        let stage = f64::from(lakes.lake_fraction[i]);
        if stage <= 0.0 {
            continue;
        }
        let overlap = cell_area * target.min(stage);
        wet_overlap_m2 += overlap;
        let id = lakes.lake_id[i];
        stage_ids.insert(id);
        *by_stage_depression.entry(id).or_default() += overlap;
    }
    let dominant_m2 = by_stage_depression
        .values()
        .copied()
        .fold(0.0_f64, f64::max);
    println!(
        "{label}_on_final_footprint: wet_overlap_km2={:.0} target_km2={:.0} wet_share={:.1}% intersecting_depressions={} dominant_overlap_km2={:.0}",
        wet_overlap_m2 / 1.0e6,
        target_area_m2 / 1.0e6,
        100.0 * wet_overlap_m2 / target_area_m2.max(1.0),
        stage_ids.len(),
        dominant_m2 / 1.0e6,
    );
}

fn surface_stats(label: &str, values: &[f32], target_fraction: &[f32]) {
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    let mut weighted_sum = 0.0_f64;
    let mut weight_sum = 0.0_f64;
    for (value, fraction) in values.iter().zip(target_fraction.iter()) {
        let weight = f64::from(*fraction);
        if weight <= 0.0 {
            continue;
        }
        let value = f64::from(*value);
        minimum = minimum.min(value);
        maximum = maximum.max(value);
        weighted_sum += value * weight;
        weight_sum += weight;
    }
    println!(
        "{label}_on_final_footprint: min_m={:.1} mean_m={:.1} max_m={:.1}",
        minimum,
        weighted_sum / weight_sum.max(1.0e-12),
        maximum,
    );
}

fn main() -> Result<(), String> {
    let seed = "interlink-wg7c";
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|e| e.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|e| e.to_string())?;
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
    let mut progress = |_done: u8, _max: u8| {};
    let (climate, climate_diagnostics) = generate_coupled_climate_with_diagnostics(
        &fine,
        &terrain,
        planet,
        &ClimateRequest::new(seed),
        &mut progress,
    )
    .map_err(|e| e.to_string())?;
    let drainage = generate_drainage_topology(
        &fine,
        &terrain,
        planet,
        &DrainageRequest::new(seed),
    )
    .map_err(|e| e.to_string())?;
    let runoff = generate_runoff_discharge(
        &fine,
        &terrain,
        &climate,
        &drainage,
        planet,
        &RunoffRequest::new(seed),
    )
    .map_err(|e| e.to_string())?;
    let lakes = generate_lakes_closed_basins(
        &fine,
        &terrain,
        &climate,
        &drainage,
        &runoff,
        planet,
        &LakeRequest::new(seed),
    )
    .map_err(|e| e.to_string())?;
    let seasonal = generate_seasonal_hydrology(
        &fine,
        &terrain,
        &climate,
        &climate_diagnostics,
        &drainage,
        &runoff,
        &lakes,
        planet,
        &SeasonalHydrologyRequest::new(seed),
    )
    .map_err(|e| e.to_string())?;
    let erosion = generate_fluvial_erosion_sediment(
        &fine,
        &inherited,
        &terrain,
        &drainage,
        &lakes,
        &seasonal,
        planet,
        &FluvialErosionRequest::new(seed),
    )
    .map_err(|e| e.to_string())?;
    let evolution = generate_bounded_terrain_evolution(
        &fine,
        &terrain,
        &drainage,
        &runoff,
        &lakes,
        &erosion,
        planet,
        &TerrainEvolutionRequest::new(seed),
    )
    .map_err(|e| e.to_string())?;
    let reconciliation = generate_post_erosion_hydrology(
        &fine,
        &terrain,
        &climate,
        &climate_diagnostics,
        &drainage,
        &runoff,
        &lakes,
        &seasonal,
        &evolution,
        planet,
        &PostErosionHydrologyRequest::new(seed),
    )
    .map_err(|e| e.to_string())?;
    let infill = generate_lake_sediment_infill(
        &fine,
        &terrain,
        &climate,
        &climate_diagnostics,
        &drainage,
        &lakes,
        &erosion,
        &evolution,
        &reconciliation,
        planet,
        &LakeSedimentInfillRequest::new(seed),
    )
    .map_err(|e| e.to_string())?;

    let rift_distance = rift_distances(&fine, &boundaries, planet.radius_m);
    let wg6_rows = rift_lake_rows(&fine, &inherited, &rift_distance, &lakes, planet.radius_m);
    let wg7c_rows = rift_lake_rows(
        &fine,
        &inherited,
        &rift_distance,
        &reconciliation.reconciled_lakes,
        planet.radius_m,
    );
    let wg7d_rows = rift_lake_rows(
        &fine,
        &inherited,
        &rift_distance,
        &infill.reconciled_lakes,
        planet.radius_m,
    );

    print_stage("WG-6C", &wg6_rows);
    print_stage("WG-7C", &wg7c_rows);
    print_stage("WG-7D", &wg7d_rows);

    let target = wg7d_rows
        .first()
        .ok_or("WG-7D has no rift-dominated lake to trace")?;
    let target_fraction = infill
        .reconciled_lakes
        .lake_fraction
        .iter()
        .zip(infill.reconciled_lakes.lake_id.iter())
        .map(|(fraction, depression_id)| {
            if *depression_id == target.depression_id {
                *fraction
            } else {
                0.0
            }
        })
        .collect::<Vec<_>>();

    println!(
        "TRACE_TARGET: depression={} area_km2={:.0}",
        target.depression_id,
        target.area_m2 / 1.0e6
    );
    footprint_overlap("WG-6C", &fine, &target_fraction, &lakes, planet.radius_m);
    footprint_overlap(
        "WG-7C",
        &fine,
        &target_fraction,
        &reconciliation.reconciled_lakes,
        planet.radius_m,
    );
    footprint_overlap(
        "WG-7D",
        &fine,
        &target_fraction,
        &infill.reconciled_lakes,
        planet.radius_m,
    );
    surface_stats("WG-4", &terrain.solid_elevation_m, &target_fraction);
    surface_stats(
        "WG-7B",
        &evolution.evolved_solid_elevation_m,
        &target_fraction,
    );
    surface_stats(
        "WG-7D",
        &infill.post_infill_solid_elevation_m,
        &target_fraction,
    );
    surface_stats("WG-7D_FILL", &infill.lake_fill_depth_m, &target_fraction);

    Ok(())
}
