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
    eligible: &[bool],
) -> ResponseProfile {
    let mut band_means = [0.0_f64; 5];
    for (band_index, pair) in BAND_EDGES_M.windows(2).enumerate() {
        let mut weighted_sum = 0.0_f64;
        let mut area_sum = 0.0_f64;
        for index in 0..distances.len() {
            let distance = distances[index];
            if !eligible[index] || distance < pair[0] || distance >= pair[1] {
                continue;
            }
            let area = topology.dual_area_steradians()[index] * planet.radius_m * planet.radius_m;
            weighted_sum += values_m[index].abs() * area;
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

fn attributed_continental_rift_profile(
    topology: &GeodesicTopology,
    inherited: &interlink_worldgen::InheritedPhysicalState,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    request: &TopographyRequest,
    terrain: &TopographyState,
) -> Result<ResponseProfile, String> {
    let (rift_distance, source_edge_count) = boundary_distances(
        topology,
        boundaries,
        planet.radius_m,
        GeologicalBoundaryRegime::ContinentalRift,
    );
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

    // Hold inherited rift history, basin potential, subsidence history, crust state,
    // planet parameters and every non-rift boundary fixed. Removing only active
    // ContinentalRift interfaces makes the component difference a causal measure
    // of the direct boundary response rather than the broad inherited basin field.
    let mut without_active_rifts = boundaries.clone();
    without_active_rifts
        .boundaries
        .retain(|edge| edge.geological_regime != GeologicalBoundaryRegime::ContinentalRift);
    // The counterfactual is diagnostic-only, but keep its identity distinct from
    // the accepted boundary set so an accidental hash comparison cannot alias it.
    without_active_rifts.boundary_hash ^= 0x8f3d_94a2_771c_5b6d;
    let counterfactual =
        generate_initial_topography(topology, inherited, &without_active_rifts, planet, request)
            .map_err(|error| error.to_string())?;

    let values = terrain
        .rift_basin_elevation_m
        .iter()
        .zip(counterfactual.rift_basin_elevation_m.iter())
        .map(|(accepted, without)| f64::from(*accepted) - f64::from(*without))
        .collect::<Vec<_>>();
    let eligible = vec![true; values.len()];
    Ok(response_profile(
        topology,
        planet,
        source_edge_count,
        &rift_distance,
        &values,
        &eligible,
    ))
}

fn attributed_transitional_divergence_profile(
    topology: &GeodesicTopology,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    terrain: &TopographyState,
) -> ResponseProfile {
    let (transition_distance, source_edge_count) = boundary_distances(
        topology,
        boundaries,
        planet.radius_m,
        GeologicalBoundaryRegime::TransitionalDivergence,
    );
    let (oceanic_ridge_distance, _) = boundary_distances(
        topology,
        boundaries,
        planet.radius_m,
        GeologicalBoundaryRegime::OceanicRidge,
    );
    let values = terrain
        .ridge_elevation_m
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    // WG-4 chooses between pure-oceanic and transitional divergent sources by
    // nearest distance (then deterministic source ID on exact ties). Excluding
    // equal-distance ties means every retained sample is unambiguously sourced
    // by TransitionalDivergence rather than measuring nearby pure-ridge relief.
    let eligible = transition_distance
        .iter()
        .zip(oceanic_ridge_distance.iter())
        .map(|(transition, ridge)| *transition + DISTANCE_EPSILON_M < *ridge)
        .collect::<Vec<_>>();
    response_profile(
        topology,
        planet,
        source_edge_count,
        &transition_distance,
        &values,
        &eligible,
    )
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

    let mut continental = EnsembleSummary::default();
    let mut transitional = EnsembleSummary::default();

    for (cohort, seeds) in [
        ("calibration", calibration.as_slice()),
        ("holdout", holdout.as_slice()),
    ] {
        for seed in seeds {
            let tectonics =
                generate_tectonics(&coarse, &TectonicsRequest::new(*seed, plates), planet)
                    .map_err(|error| error.to_string())?;
            let geology = generate_crust_and_history(
                &coarse,
                &tectonics,
                &GeologyRequest::new(*seed),
                planet,
            )
            .map_err(|error| error.to_string())?;
            let lithosphere = generate_lithosphere(
                &coarse,
                &tectonics,
                &geology,
                &LithosphereRequest::new(*seed),
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
            let boundaries = inherit_boundary_interfaces(
                &coarse,
                &fine,
                &tectonics,
                &geology,
                &inherited.plate_ids,
            )
            .map_err(|error| error.to_string())?;
            let request = TopographyRequest::new(*seed);
            let terrain =
                generate_initial_topography(&fine, &inherited, &boundaries, planet, &request)
                    .map_err(|error| error.to_string())?;

            let continental_profile = attributed_continental_rift_profile(
                &fine,
                &inherited,
                &boundaries,
                planet,
                &request,
                &terrain,
            )?;
            let transitional_profile =
                attributed_transitional_divergence_profile(&fine, &boundaries, planet, &terrain);

            println!(
                "WG-4 attributed rift profile cohort={cohort} seed={seed} continental_edges={} continental_peak={:.3}m continental_peak_distance={:.1}km continental_half_peak_width={:.1}km continental_tail500_750={:.4} continental_tail750_1000={:.4} transitional_edges={} transitional_peak={:.3}m transitional_peak_distance={:.1}km transitional_half_peak_width={:.1}km transitional_tail500_750={:.4} transitional_tail750_1000={:.4}",
                continental_profile.source_edge_count,
                continental_profile.peak_mean_m,
                continental_profile.peak_distance_m / 1_000.0,
                continental_profile.half_peak_width_m / 1_000.0,
                continental_profile.mid_tail_ratio,
                continental_profile.far_tail_ratio,
                transitional_profile.source_edge_count,
                transitional_profile.peak_mean_m,
                transitional_profile.peak_distance_m / 1_000.0,
                transitional_profile.half_peak_width_m / 1_000.0,
                transitional_profile.mid_tail_ratio,
                transitional_profile.far_tail_ratio,
            );

            continental.absorb(continental_profile);
            transitional.absorb(transitional_profile);
        }
    }

    println!(
        "WG-4 attributed rift response acceptance: continental_seeds={} continental_mean_peak={:.3}m continental_max_peak_distance={:.1}km continental_max_half_peak_width={:.1}km continental_max_tail500_750={:.4} continental_max_tail750_1000={:.4} transitional_seeds={} transitional_mean_peak={:.3}m transitional_max_peak_distance={:.1}km transitional_max_half_peak_width={:.1}km transitional_max_tail500_750={:.4} transitional_max_tail750_1000={:.4}",
        continental.measured_seed_count,
        continental.mean_peak_m(),
        continental.maximum_peak_distance_m / 1_000.0,
        continental.maximum_half_peak_width_m / 1_000.0,
        continental.maximum_mid_tail_ratio,
        continental.maximum_far_tail_ratio,
        transitional.measured_seed_count,
        transitional.mean_peak_m(),
        transitional.maximum_peak_distance_m / 1_000.0,
        transitional.maximum_half_peak_width_m / 1_000.0,
        transitional.maximum_mid_tail_ratio,
        transitional.maximum_far_tail_ratio,
    );

    if continental.measured_seed_count < 6 || transitional.measured_seed_count < 6 {
        return Err(format!(
            "rift response acceptance did not exercise enough worlds: continental={} transitional={}",
            continental.measured_seed_count, transitional.measured_seed_count
        ));
    }
    if continental.mean_peak_m() < 50.0 {
        return Err(format!(
            "active continental-rift response became negligible: mean peak {:.3} m",
            continental.mean_peak_m()
        ));
    }
    if transitional.mean_peak_m() < 150.0 {
        return Err(format!(
            "transitional-divergence response became negligible: mean peak {:.3} m",
            transitional.mean_peak_m()
        ));
    }
    for (label, summary) in [
        ("continental-rift", &continental),
        ("transitional-divergence", &transitional),
    ] {
        if summary.maximum_peak_distance_m > 100_000.0 {
            return Err(format!(
                "{label} response peaks too far from the active boundary: {:.1} km",
                summary.maximum_peak_distance_m / 1_000.0
            ));
        }
        if summary.maximum_half_peak_width_m > 250_000.0 {
            return Err(format!(
                "{label} response remains too broad: {:.1} km effective half-peak width",
                summary.maximum_half_peak_width_m / 1_000.0
            ));
        }
        if summary.maximum_mid_tail_ratio > 0.06 {
            return Err(format!(
                "{label} 500-750 km tail remains too strong: ratio {:.4}",
                summary.maximum_mid_tail_ratio
            ));
        }
        if summary.maximum_far_tail_ratio > 0.01 {
            return Err(format!(
                "{label} 750-1000 km tail remains too strong: ratio {:.4}",
                summary.maximum_far_tail_ratio
            ));
        }
    }

    Ok(())
}
