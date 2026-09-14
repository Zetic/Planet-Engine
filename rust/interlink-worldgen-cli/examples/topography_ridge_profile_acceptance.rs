use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, GeodesicTopology,
    GeologicalBoundaryRegime, GeologyRequest, InheritedBoundarySet, LithosphereRequest,
    PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

const DISTANCE_EPSILON_M: f64 = 1.0e-6;
const BAND_EDGES_M: [f64; 6] = [
    0.0,
    100_000.0,
    250_000.0,
    500_000.0,
    750_000.0,
    1_000_000.0,
];

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

fn boundary_distances(
    topology: &GeodesicTopology,
    boundaries: &InheritedBoundarySet,
    radius_m: f64,
    target_regime: GeologicalBoundaryRegime,
) -> (Vec<f64>, u32) {
    let count = topology.metrics().sample_count as usize;
    let mut distances = vec![f64::INFINITY; count];
    let mut queue = BinaryHeap::new();
    let mut edge_count = 0_u32;

    for edge in &boundaries.boundaries {
        if edge.geological_regime != target_regime {
            continue;
        }
        edge_count += 1;
        for sample in [edge.sample_a, edge.sample_b] {
            let index = sample as usize;
            if distances[index] > 0.0 {
                distances[index] = 0.0;
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
struct RidgeProfile {
    source_edge_count: u32,
    peak_mean_m: f64,
    peak_distance_m: f64,
    half_peak_width_m: f64,
    mid_tail_ratio: f64,
    far_tail_ratio: f64,
}

fn attributed_ridge_profile(
    topology: &GeodesicTopology,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    ridge_elevation_m: &[f32],
) -> RidgeProfile {
    let (ridge_distance, source_edge_count) = boundary_distances(
        topology,
        boundaries,
        planet.radius_m,
        GeologicalBoundaryRegime::OceanicRidge,
    );
    let (transition_distance, _) = boundary_distances(
        topology,
        boundaries,
        planet.radius_m,
        GeologicalBoundaryRegime::TransitionalDivergence,
    );

    let mut band_means = [0.0_f64; 5];
    for (band_index, pair) in BAND_EDGES_M.windows(2).enumerate() {
        let mut weighted_sum = 0.0_f64;
        let mut area_sum = 0.0_f64;
        for index in 0..ridge_distance.len() {
            let distance = ridge_distance[index];
            if distance < pair[0]
                || distance >= pair[1]
                || distance > transition_distance[index] + DISTANCE_EPSILON_M
            {
                continue;
            }
            let area = topology.dual_area_steradians()[index] * planet.radius_m * planet.radius_m;
            weighted_sum += f64::from(ridge_elevation_m[index]).abs() * area;
            area_sum += area;
        }
        band_means[band_index] = if area_sum > 0.0 {
            weighted_sum / area_sum
        } else {
            0.0
        };
    }

    let mut peak_index = 0_usize;
    let mut peak_mean_m = 0.0_f64;
    for (index, value) in band_means.iter().copied().enumerate() {
        if value > peak_mean_m {
            peak_mean_m = value;
            peak_index = index;
        }
    }
    let peak_distance_m = if peak_mean_m > 0.0 {
        0.5 * (BAND_EDGES_M[peak_index] + BAND_EDGES_M[peak_index + 1])
    } else {
        0.0
    };
    let half_peak = peak_mean_m * 0.5;
    let half_peak_width_m = band_means
        .iter()
        .enumerate()
        .filter(|(_, value)| half_peak > 0.0 && **value >= half_peak)
        .map(|(index, _)| BAND_EDGES_M[index + 1])
        .fold(0.0_f64, f64::max);

    RidgeProfile {
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

    let mut peak_mean_sum_m = 0.0_f64;
    let mut maximum_peak_distance_m = 0.0_f64;
    let mut maximum_half_peak_width_m = 0.0_f64;
    let mut maximum_mid_tail_ratio = 0.0_f64;
    let mut maximum_far_tail_ratio = 0.0_f64;
    let mut measured_seed_count = 0_u32;

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

        let profile = attributed_ridge_profile(
            &fine,
            &boundaries,
            planet,
            &terrain.ridge_elevation_m,
        );
        if profile.source_edge_count == 0 {
            continue;
        }

        println!(
            "WG-4 attributed ridge profile seed={seed} edges={} peak={:.3}m peak_distance={:.1}km half_peak_width={:.1}km tail500_750={:.4} tail750_1000={:.4}",
            profile.source_edge_count,
            profile.peak_mean_m,
            profile.peak_distance_m / 1_000.0,
            profile.half_peak_width_m / 1_000.0,
            profile.mid_tail_ratio,
            profile.far_tail_ratio,
        );

        peak_mean_sum_m += profile.peak_mean_m;
        maximum_peak_distance_m = maximum_peak_distance_m.max(profile.peak_distance_m);
        maximum_half_peak_width_m = maximum_half_peak_width_m.max(profile.half_peak_width_m);
        maximum_mid_tail_ratio = maximum_mid_tail_ratio.max(profile.mid_tail_ratio);
        maximum_far_tail_ratio = maximum_far_tail_ratio.max(profile.far_tail_ratio);
        measured_seed_count += 1;
    }

    if measured_seed_count == 0 {
        return Err("ridge profile acceptance found no oceanic-ridge boundaries".to_string());
    }

    let mean_peak_m = peak_mean_sum_m / f64::from(measured_seed_count);
    println!(
        "WG-4 attributed ridge profile acceptance: seeds={} mean_peak={:.3}m max_peak_distance={:.1}km max_half_peak_width={:.1}km max_tail500_750={:.4} max_tail750_1000={:.4}",
        measured_seed_count,
        mean_peak_m,
        maximum_peak_distance_m / 1_000.0,
        maximum_half_peak_width_m / 1_000.0,
        maximum_mid_tail_ratio,
        maximum_far_tail_ratio,
    );

    if mean_peak_m < 3.0 {
        return Err(format!(
            "localized oceanic-ridge response became negligible: mean peak {mean_peak_m:.3} m"
        ));
    }
    if maximum_peak_distance_m > 250_000.0 {
        return Err(format!(
            "oceanic-ridge component peaks too far from the active boundary: {:.1} km",
            maximum_peak_distance_m / 1_000.0
        ));
    }
    if maximum_half_peak_width_m > 500_000.0 {
        return Err(format!(
            "oceanic-ridge component remains too broad: {:.1} km effective half-peak width",
            maximum_half_peak_width_m / 1_000.0
        ));
    }
    if maximum_mid_tail_ratio > 0.25 {
        return Err(format!(
            "oceanic-ridge 500-750 km tail remains too strong: ratio {maximum_mid_tail_ratio:.4}"
        ));
    }
    if maximum_far_tail_ratio > 0.10 {
        return Err(format!(
            "oceanic-ridge 750-1000 km tail remains too strong: ratio {maximum_far_tail_ratio:.4}"
        ));
    }

    Ok(())
}
