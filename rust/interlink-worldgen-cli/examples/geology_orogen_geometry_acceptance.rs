use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_tectonics, GeologicalBoundaryRegime,
    GeologyRequest, PlanetPhysicalParameters, PlanetTopology, SubductionPolarity, TectonicsRequest,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, Debug, Default)]
struct WeightedMoments {
    weight: f64,
    sum: f64,
    sum_sq: f64,
}

impl WeightedMoments {
    fn add(&mut self, value: f64, weight: f64) {
        if value.is_finite() && weight.is_finite() && weight > 0.0 {
            self.weight += weight;
            self.sum += value * weight;
            self.sum_sq += value * value * weight;
        }
    }

    fn mean(self) -> f64 {
        if self.weight > 0.0 { self.sum / self.weight } else { 0.0 }
    }

    fn cv(self) -> f64 {
        let mean = self.mean();
        if self.weight <= 0.0 || mean.abs() <= 1.0e-12 {
            return 0.0;
        }
        let variance = (self.sum_sq / self.weight - mean * mean).max(0.0);
        variance.sqrt() / mean.abs()
    }
}

#[derive(Clone, Copy, Debug)]
struct Frontier {
    distance_rad: f64,
    source: u32,
    sample: u32,
}

impl PartialEq for Frontier {
    fn eq(&self, other: &Self) -> bool {
        self.distance_rad.to_bits() == other.distance_rad.to_bits()
            && self.source == other.source
            && self.sample == other.sample
    }
}
impl Eq for Frontier {}
impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for Frontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance_rad
            .total_cmp(&self.distance_rad)
            .then_with(|| other.source.cmp(&self.source))
            .then_with(|| other.sample.cmp(&self.sample))
    }
}

#[derive(Clone, Debug, Default)]
struct WorldProfile {
    source_count: usize,
    measured_widths: usize,
    core_fraction: f64,
    width: WeightedMoments,
    endpoint_width: WeightedMoments,
    interior_width: WeightedMoments,
    collision_asymmetry: WeightedMoments,
    integrated_history: f64,
}

fn orogen_sources(
    geology: &interlink_worldgen::CrustalModel,
) -> (Vec<u32>, Vec<bool>, Vec<bool>) {
    let count = geology.orogenic_history.len();
    let mut mask = vec![false; count];
    let mut collision = vec![false; count];
    let mut subduction = vec![false; count];
    for boundary in &geology.boundaries {
        match boundary.regime {
            GeologicalBoundaryRegime::ContinentalCollision => {
                for sample in [boundary.sample_a, boundary.sample_b] {
                    mask[sample as usize] = true;
                    collision[sample as usize] = true;
                }
            }
            GeologicalBoundaryRegime::OceanicSubduction
            | GeologicalBoundaryRegime::OceanContinentSubduction => match boundary.subduction_polarity {
                SubductionPolarity::PlateA => {
                    mask[boundary.sample_b as usize] = true;
                    subduction[boundary.sample_b as usize] = true;
                }
                SubductionPolarity::PlateB => {
                    mask[boundary.sample_a as usize] = true;
                    subduction[boundary.sample_a as usize] = true;
                }
                SubductionPolarity::None => {}
            },
            _ => {}
        }
    }
    let sources = mask
        .iter()
        .enumerate()
        .filter_map(|(index, active)| active.then_some(index as u32))
        .collect::<Vec<_>>();
    (sources, collision, subduction)
}

fn nearest_sources<T: PlanetTopology>(topology: &T, sources: &[u32]) -> (Vec<f64>, Vec<u32>) {
    let count = topology.sample_count() as usize;
    let mut distance = vec![f64::INFINITY; count];
    let mut source_id = vec![u32::MAX; count];
    let mut frontier = BinaryHeap::new();
    for (source, sample) in sources.iter().enumerate() {
        distance[*sample as usize] = 0.0;
        source_id[*sample as usize] = source as u32;
        frontier.push(Frontier { distance_rad: 0.0, source: source as u32, sample: *sample });
    }
    while let Some(current) = frontier.pop() {
        let index = current.sample as usize;
        if current.distance_rad > distance[index] + 1.0e-14 || current.source != source_id[index] {
            continue;
        }
        let neighbors = topology.neighbors(current.sample);
        let lengths = topology.neighbor_arc_lengths_rad(current.sample);
        for cursor in 0..neighbors.len() {
            let neighbor = neighbors[cursor];
            let candidate = current.distance_rad + lengths[cursor];
            let target = neighbor as usize;
            if candidate + 1.0e-14 < distance[target] {
                distance[target] = candidate;
                source_id[target] = current.source;
                frontier.push(Frontier { distance_rad: candidate, source: current.source, sample: neighbor });
            }
        }
    }
    (distance, source_id)
}

fn profile_world(
    topology: &interlink_worldgen::GeodesicTopology,
    geology: &interlink_worldgen::CrustalModel,
    planet: PlanetPhysicalParameters,
) -> WorldProfile {
    let (sources, collision_source, subduction_source) = orogen_sources(geology);
    if sources.is_empty() {
        return WorldProfile::default();
    }
    let (distance_rad, nearest) = nearest_sources(topology, &sources);
    let mut shoulder = vec![WeightedMoments::default(); sources.len()];
    let mut source_core = 0_usize;
    let mut total_area = 0.0;
    let mut history_area = 0.0;

    for sample in 0..topology.sample_count() as usize {
        let area = topology.area_steradians(sample as u32);
        total_area += area;
        history_area += area * f64::from(geology.orogenic_history[sample]);
        let source = nearest[sample];
        if source == u32::MAX {
            continue;
        }
        let history = f64::from(geology.orogenic_history[sample]);
        if (0.42..=0.58).contains(&history) {
            shoulder[source as usize].add(distance_rad[sample] * planet.radius_m, area);
        }
    }
    for sample in &sources {
        if geology.orogenic_history[*sample as usize] >= 0.70 {
            source_core += 1;
        }
    }

    let mut source_mask = vec![false; geology.orogenic_history.len()];
    for sample in &sources { source_mask[*sample as usize] = true; }
    let mut profile = WorldProfile {
        source_count: sources.len(),
        core_fraction: source_core as f64 / sources.len() as f64,
        integrated_history: if total_area > 0.0 { history_area / total_area } else { 0.0 },
        ..WorldProfile::default()
    };
    let mut widths = vec![0.0_f64; sources.len()];
    for (source_index, sample) in sources.iter().enumerate() {
        let width = shoulder[source_index].mean();
        widths[source_index] = width;
        if width <= 0.0 {
            continue;
        }
        profile.measured_widths += 1;
        profile.width.add(width, 1.0);
        let neighbor_sources = topology
            .neighbors(*sample)
            .iter()
            .filter(|neighbor| source_mask[**neighbor as usize])
            .count();
        let is_collision = collision_source[*sample as usize];
        let is_subduction_only = subduction_source[*sample as usize] && !is_collision;
        let endpoint = if is_collision {
            neighbor_sources <= 2
        } else if is_subduction_only {
            neighbor_sources <= 1
        } else {
            false
        };
        let interior = if is_collision {
            neighbor_sources >= 3
        } else if is_subduction_only {
            neighbor_sources >= 2
        } else {
            false
        };
        if endpoint { profile.endpoint_width.add(width, 1.0); }
        if interior { profile.interior_width.add(width, 1.0); }
    }

    for boundary in &geology.boundaries {
        if boundary.regime != GeologicalBoundaryRegime::ContinentalCollision {
            continue;
        }
        let source_a = sources.iter().position(|sample| *sample == boundary.sample_a);
        let source_b = sources.iter().position(|sample| *sample == boundary.sample_b);
        let (Some(a), Some(b)) = (source_a, source_b) else { continue };
        let wa = widths[a];
        let wb = widths[b];
        let mean = 0.5 * (wa + wb);
        if wa > 0.0 && wb > 0.0 && mean > 0.0 {
            profile.collision_asymmetry.add((wa - wb).abs() / mean, 1.0);
        }
    }
    profile
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
        "1",
        "2",
        "wg4-morphology-holdout-a",
        "wg4-morphology-holdout-b",
        "wg4-morphology-holdout-c",
        "wg4-morphology-holdout-d",
    ];
    let level = 5_u8;
    let plates = 16_u16;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let topology = build_icosphere(level).map_err(|error| error.to_string())?;
    let mut exercised = 0_u32;
    let mut minimum_core = 1.0_f64;
    let mut minimum_width_cv = f64::INFINITY;
    let mut mean_width_cv = 0.0;
    let mut endpoint_ratios = WeightedMoments::default();
    let mut asymmetry = WeightedMoments::default();

    for (cohort, seeds) in [
        ("calibration", calibration.as_slice()),
        ("holdout", holdout.as_slice()),
    ] {
        for seed in seeds {
            let tectonics = generate_tectonics(&topology, &TectonicsRequest::new(*seed, plates), planet)
                .map_err(|error| error.to_string())?;
            let geology = generate_crust_and_history(
                &topology,
                &tectonics,
                &GeologyRequest::new(*seed),
                planet,
            )
            .map_err(|error| error.to_string())?;
            let profile = profile_world(&topology, &geology, planet);
            if profile.source_count == 0 || profile.measured_widths < 8 {
                println!("WG-3 orogen geometry cohort={cohort} seed={seed} skipped sources={} measured_widths={}", profile.source_count, profile.measured_widths);
                continue;
            }
            exercised += 1;
            minimum_core = minimum_core.min(profile.core_fraction);
            minimum_width_cv = minimum_width_cv.min(profile.width.cv());
            mean_width_cv += profile.width.cv();
            if profile.endpoint_width.weight > 0.0 && profile.interior_width.mean() > 0.0 {
                endpoint_ratios.add(profile.endpoint_width.mean() / profile.interior_width.mean(), 1.0);
            }
            if profile.collision_asymmetry.weight > 0.0 {
                asymmetry.add(profile.collision_asymmetry.mean(), 1.0);
            }
            println!(
                "WG-3 orogen geometry cohort={cohort} seed={seed} sources={} widths={} core={:.4} mean_width={:.0}km width_cv={:.4} endpoint/interior={:.4} collision_asym={:.4} integrated={:.4}",
                profile.source_count,
                profile.measured_widths,
                profile.core_fraction,
                profile.width.mean() / 1_000.0,
                profile.width.cv(),
                if profile.interior_width.mean() > 0.0 { profile.endpoint_width.mean() / profile.interior_width.mean() } else { 0.0 },
                profile.collision_asymmetry.mean(),
                profile.integrated_history,
            );
        }
    }

    let mean_cv = mean_width_cv / f64::from(exercised.max(1));
    println!(
        "WG-3 orogen geometry acceptance: worlds={} min_core={:.4} mean_width_cv={:.4} min_width_cv={:.4} mean_endpoint/interior={:.4} mean_collision_asym={:.4}",
        exercised,
        minimum_core,
        mean_cv,
        minimum_width_cv,
        endpoint_ratios.mean(),
        asymmetry.mean(),
    );

    if exercised < 10 {
        return Err(format!("orogen geometry exercised only {exercised} worlds"));
    }
    if minimum_core < 0.80 {
        return Err(format!("orogen core continuity fell too low: {minimum_core:.4}"));
    }
    if !minimum_width_cv.is_finite() || mean_cv <= 0.0 {
        return Err("orogen width diagnostics did not produce finite variation".to_string());
    }
    Ok(())
}
