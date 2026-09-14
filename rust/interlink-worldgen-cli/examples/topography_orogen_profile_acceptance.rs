use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, GeodesicTopology,
    GeologicalBoundaryRegime, GeologyRequest, InheritedBoundarySet, LithosphereRequest,
    PlanetPhysicalParameters, TectonicsRequest, TopographyRequest, TopographyState,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

const DISTANCE_EPSILON_M: f64 = 1.0e-6;
const BAND_EDGES_M: [f64; 6] = [0.0, 100_000.0, 250_000.0, 500_000.0, 750_000.0, 1_000_000.0];

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

fn collision_distances(
    topology: &GeodesicTopology,
    boundaries: &InheritedBoundarySet,
    radius_m: f64,
) -> (Vec<f64>, u32) {
    let count = topology.metrics().sample_count as usize;
    let mut distances = vec![f64::INFINITY; count];
    let mut queue = BinaryHeap::new();
    let mut edge_count = 0_u32;
    for edge in &boundaries.boundaries {
        if edge.geological_regime != GeologicalBoundaryRegime::ContinentalCollision {
            continue;
        }
        edge_count += 1;
        for sample in [edge.sample_a, edge.sample_b] {
            if distances[sample as usize] > 0.0 {
                distances[sample as usize] = 0.0;
                queue.push(QueueEntry {
                    distance_m: 0.0,
                    sample,
                });
            }
        }
    }
    while let Some(entry) = queue.pop() {
        let index = entry.sample as usize;
        if entry.distance_m > distances[index] + DISTANCE_EPSILON_M {
            continue;
        }
        for (neighbor, arc) in topology
            .neighbors_of(entry.sample)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(entry.sample).iter())
        {
            let candidate = entry.distance_m + *arc * radius_m;
            let target = *neighbor as usize;
            if candidate + DISTANCE_EPSILON_M < distances[target] {
                distances[target] = candidate;
                queue.push(QueueEntry {
                    distance_m: candidate,
                    sample: *neighbor,
                });
            }
        }
    }
    (distances, edge_count)
}

#[derive(Clone, Copy, Debug)]
struct ResponseProfile {
    source_edge_count: u32,
    peak_mean_m: f64,
    peak_distance_m: f64,
    half_peak_width_m: f64,
    mid_tail_ratio: f64,
    far_tail_ratio: f64,
}

fn response_profile(
    topology: &GeodesicTopology,
    planet: PlanetPhysicalParameters,
    source_edge_count: u32,
    distances: &[f64],
    values_m: &[f64],
) -> ResponseProfile {
    let mut band_means = [0.0_f64; 5];
    for (band_index, pair) in BAND_EDGES_M.windows(2).enumerate() {
        let mut weighted_sum = 0.0;
        let mut area_sum = 0.0;
        for index in 0..distances.len() {
            if distances[index] < pair[0] || distances[index] >= pair[1] {
                continue;
            }
            let area = topology.dual_area_steradians()[index] * planet.radius_m * planet.radius_m;
            weighted_sum += values_m[index].abs() * area;
            area_sum += area;
        }
        if area_sum > 0.0 {
            band_means[band_index] = weighted_sum / area_sum;
        }
    }
    let (peak_index, peak_mean_m) = band_means
        .iter()
        .copied()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap();
    let peak_distance_m = if peak_mean_m > 0.0 {
        0.5 * (BAND_EDGES_M[peak_index] + BAND_EDGES_M[peak_index + 1])
    } else {
        0.0
    };
    let half_peak = 0.5 * peak_mean_m;
    let half_peak_width_m = band_means
        .iter()
        .enumerate()
        .filter(|(_, value)| half_peak > 0.0 && **value >= half_peak)
        .map(|(index, _)| BAND_EDGES_M[index + 1])
        .fold(0.0_f64, f64::max);
    ResponseProfile {
        source_edge_count,
        peak_mean_m,
        peak_distance_m,
        half_peak_width_m,
        mid_tail_ratio: if peak_mean_m > 0.0 {
            band_means[3] / peak_mean_m
        } else {
            0.0
        },
        far_tail_ratio: if peak_mean_m > 0.0 {
            band_means[4] / peak_mean_m
        } else {
            0.0
        },
    }
}

fn attributed_profile(
    topology: &GeodesicTopology,
    inherited: &interlink_worldgen::InheritedPhysicalState,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    request: &TopographyRequest,
    terrain: &TopographyState,
) -> Result<ResponseProfile, String> {
    let (distances, source_edge_count) = collision_distances(topology, boundaries, planet.radius_m);
    if source_edge_count == 0 {
        return Ok(ResponseProfile {
            source_edge_count,
            peak_mean_m: 0.0,
            peak_distance_m: 0.0,
            half_peak_width_m: 0.0,
            mid_tail_ratio: 0.0,
            far_tail_ratio: 0.0,
        });
    }
    // Keep inherited orogenic history and every other physical driver fixed.
    // Removing only active ContinentalCollision interfaces isolates the direct
    // collision response from broad inherited mountain-belt memory.
    let mut without_collisions = boundaries.clone();
    without_collisions
        .boundaries
        .retain(|edge| edge.geological_regime != GeologicalBoundaryRegime::ContinentalCollision);
    without_collisions.boundary_hash ^= 0x4d71_9c23_a5e8_6bf1;
    let counterfactual =
        generate_initial_topography(topology, inherited, &without_collisions, planet, request)
            .map_err(|error| error.to_string())?;
    let values = terrain
        .orogenic_elevation_m
        .iter()
        .zip(counterfactual.orogenic_elevation_m.iter())
        .map(|(accepted, without)| f64::from(*accepted) - f64::from(*without))
        .collect::<Vec<_>>();
    Ok(response_profile(
        topology,
        planet,
        source_edge_count,
        &distances,
        &values,
    ))
}

#[derive(Default)]
struct EnsembleSummary {
    measured_seed_count: u32,
    peak_sum_m: f64,
    maximum_peak_distance_m: f64,
    maximum_half_peak_width_m: f64,
    maximum_mid_tail_ratio: f64,
    maximum_far_tail_ratio: f64,
}
impl EnsembleSummary {
    fn absorb(&mut self, profile: ResponseProfile) {
        if profile.source_edge_count == 0 {
            return;
        }
        self.measured_seed_count += 1;
        self.peak_sum_m += profile.peak_mean_m;
        self.maximum_peak_distance_m = self.maximum_peak_distance_m.max(profile.peak_distance_m);
        self.maximum_half_peak_width_m = self
            .maximum_half_peak_width_m
            .max(profile.half_peak_width_m);
        self.maximum_mid_tail_ratio = self.maximum_mid_tail_ratio.max(profile.mid_tail_ratio);
        self.maximum_far_tail_ratio = self.maximum_far_tail_ratio.max(profile.far_tail_ratio);
    }
    fn mean_peak_m(&self) -> f64 {
        self.peak_sum_m / f64::from(self.measured_seed_count.max(1))
    }
}

fn main() -> Result<(), String> {
    let calibration = [
        "3",
        "interlink-wg7c",
        "wg4-boundary-a",
        "wg4-boundary-b",
        "wg4-boundary-c",
        "wg4-boundary-d",
    ];
    let holdout = [
        "wg4-morphology-holdout-a",
        "wg4-morphology-holdout-b",
        "wg4-morphology-holdout-c",
        "wg4-morphology-holdout-d",
    ];
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;
    let mut summary = EnsembleSummary::default();
    for (cohort, seeds) in [
        ("calibration", calibration.as_slice()),
        ("holdout", holdout.as_slice()),
    ] {
        for seed in seeds {
            let tectonics =
                generate_tectonics(&coarse, &TectonicsRequest::new(*seed, plates), planet)
                    .map_err(|e| e.to_string())?;
            let geology = generate_crust_and_history(
                &coarse,
                &tectonics,
                &GeologyRequest::new(*seed),
                planet,
            )
            .map_err(|e| e.to_string())?;
            let lithosphere = generate_lithosphere(
                &coarse,
                &tectonics,
                &geology,
                &LithosphereRequest::new(*seed),
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
            let request = TopographyRequest::new(*seed);
            let terrain =
                generate_initial_topography(&fine, &inherited, &boundaries, planet, &request)
                    .map_err(|e| e.to_string())?;
            let profile =
                attributed_profile(&fine, &inherited, &boundaries, planet, &request, &terrain)?;
            println!("WG-4 attributed orogen profile cohort={cohort} seed={seed} collision_edges={} peak={:.3}m peak_distance={:.1}km half_peak_width={:.1}km tail500_750={:.4} tail750_1000={:.4}",
                profile.source_edge_count, profile.peak_mean_m, profile.peak_distance_m / 1_000.0,
                profile.half_peak_width_m / 1_000.0, profile.mid_tail_ratio, profile.far_tail_ratio);
            summary.absorb(profile);
        }
    }
    println!("WG-4 attributed orogen response acceptance: seeds={} mean_peak={:.3}m max_peak_distance={:.1}km max_half_peak_width={:.1}km max_tail500_750={:.4} max_tail750_1000={:.4}",
        summary.measured_seed_count, summary.mean_peak_m(), summary.maximum_peak_distance_m / 1_000.0,
        summary.maximum_half_peak_width_m / 1_000.0, summary.maximum_mid_tail_ratio, summary.maximum_far_tail_ratio);
    if summary.measured_seed_count < 6 {
        return Err(format!(
            "orogen response acceptance exercised only {} worlds",
            summary.measured_seed_count
        ));
    }
    if summary.mean_peak_m() < 2_000.0 {
        return Err(format!(
            "active collision response became too weak: mean peak {:.3} m",
            summary.mean_peak_m()
        ));
    }
    if summary.maximum_peak_distance_m > 100_000.0 {
        return Err(format!(
            "orogen response peaks too far from the active collision boundary: {:.1} km",
            summary.maximum_peak_distance_m / 1_000.0
        ));
    }
    if summary.maximum_half_peak_width_m > 250_000.0 {
        return Err(format!(
            "orogen response remains too broad near the active collision boundary: {:.1} km effective half-peak width",
            summary.maximum_half_peak_width_m / 1_000.0
        ));
    }
    if summary.maximum_mid_tail_ratio > 0.07 {
        return Err(format!(
            "orogen 500-750 km active-collision tail remains too strong: ratio {:.4}",
            summary.maximum_mid_tail_ratio
        ));
    }
    if summary.maximum_far_tail_ratio > 0.01 {
        return Err(format!(
            "orogen 750-1000 km active-collision tail remains too strong: ratio {:.4}",
            summary.maximum_far_tail_ratio
        ));
    }
    Ok(())
}
