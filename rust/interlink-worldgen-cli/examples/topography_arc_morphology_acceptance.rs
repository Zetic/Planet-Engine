use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    GeologicalBoundaryRegime, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters,
    SubductionPolarity, TectonicsRequest, TopographyRequest,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

const CRUST_CONTINENTAL: u8 = 3;
const ARC_BAND_MIN_M: f64 = 100_000.0;
const ARC_BAND_MAX_M: f64 = 450_000.0;
const MINIMUM_AGGREGATE_ARC_LAND_FRACTION: f64 = 0.0010;
const MAXIMUM_AGGREGATE_ARC_LAND_FRACTION: f64 = 0.0100;
const MAXIMUM_SEED_ARC_LAND_FRACTION: f64 = 0.0150;
const MINIMUM_SEEDS_WITH_ARC_LAND: u32 = 3;

#[derive(Clone, Copy, Debug)]
struct Entry {
    distance_m: f64,
    sample: u32,
    plate: u16,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.distance_m.to_bits() == other.distance_m.to_bits()
            && self.sample == other.sample
            && self.plate == other.plate
    }
}
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance_m
            .total_cmp(&self.distance_m)
            .then_with(|| other.sample.cmp(&self.sample))
            .then_with(|| other.plate.cmp(&self.plate))
    }
}

fn oceanic_arc_distances(
    topology: &interlink_worldgen::GeodesicTopology,
    inherited: &interlink_worldgen::InheritedPhysicalState,
    boundaries: &interlink_worldgen::InheritedBoundarySet,
    radius_m: f64,
) -> Vec<f64> {
    let count = topology.metrics().sample_count as usize;
    let mut distance = vec![f64::INFINITY; count];
    let mut source_plate = vec![u16::MAX; count];
    let mut queue = BinaryHeap::new();

    for edge in &boundaries.boundaries {
        if edge.geological_regime != GeologicalBoundaryRegime::OceanicSubduction {
            continue;
        }
        let (sample, plate) = match edge.subduction_polarity {
            SubductionPolarity::PlateA => (edge.sample_b, edge.plate_b),
            SubductionPolarity::PlateB => (edge.sample_a, edge.plate_a),
            SubductionPolarity::None => continue,
        };
        let i = sample as usize;
        if inherited.crust_kind[i] == CRUST_CONTINENTAL {
            continue;
        }
        if distance[i] > 0.0 || plate < source_plate[i] {
            distance[i] = 0.0;
            source_plate[i] = plate;
            queue.push(Entry {
                distance_m: 0.0,
                sample,
                plate,
            });
        }
    }

    while let Some(entry) = queue.pop() {
        let i = entry.sample as usize;
        if entry.distance_m > distance[i] + 1.0e-6 || entry.plate != source_plate[i] {
            continue;
        }
        if entry.distance_m > ARC_BAND_MAX_M {
            continue;
        }
        for (neighbor, arc) in topology
            .neighbors_of(entry.sample)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(entry.sample).iter())
        {
            let ni = *neighbor as usize;
            if inherited.plate_ids[ni] != entry.plate {
                continue;
            }
            let candidate = entry.distance_m + *arc * radius_m;
            if candidate > ARC_BAND_MAX_M {
                continue;
            }
            let better = candidate + 1.0e-6 < distance[ni]
                || ((candidate - distance[ni]).abs() <= 1.0e-6 && entry.plate < source_plate[ni]);
            if better {
                distance[ni] = candidate;
                source_plate[ni] = entry.plate;
                queue.push(Entry {
                    distance_m: candidate,
                    sample: *neighbor,
                    plate: entry.plate,
                });
            }
        }
    }
    distance
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
    let mut total_band_area = 0.0;
    let mut total_land_area = 0.0;
    let mut maximum_seed_fraction = 0.0_f64;
    let mut seeds_with_arc_land = 0_u32;

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
        let distances = oceanic_arc_distances(&fine, &inherited, &boundaries, planet.radius_m);
        let mut band_area = 0.0;
        let mut land_area = 0.0;
        for i in 0..distances.len() {
            if distances[i] < ARC_BAND_MIN_M
                || distances[i] > ARC_BAND_MAX_M
                || inherited.crust_kind[i] == CRUST_CONTINENTAL
            {
                continue;
            }
            let area = fine.dual_area_steradians()[i] * planet.radius_m * planet.radius_m;
            band_area += area;
            if terrain.submerged_mask[i] == 0 {
                land_area += area;
            }
        }
        let fraction = if band_area > 0.0 {
            land_area / band_area
        } else {
            0.0
        };
        if land_area > 0.0 {
            seeds_with_arc_land += 1;
        }
        maximum_seed_fraction = maximum_seed_fraction.max(fraction);
        total_band_area += band_area;
        total_land_area += land_area;
        println!(
            "{seed}: oceanic_arc_band_land={:.3}% arc_land_km2={:.0}",
            fraction * 100.0,
            land_area / 1.0e6
        );
    }

    let aggregate_fraction = if total_band_area > 0.0 {
        total_land_area / total_band_area
    } else {
        0.0
    };
    println!(
        "WG-4 oceanic arc morphology acceptance: aggregate={:.3}% max_seed={:.3}% seeds_with_land={} land_km2={:.0}",
        aggregate_fraction * 100.0,
        maximum_seed_fraction * 100.0,
        seeds_with_arc_land,
        total_land_area / 1.0e6
    );

    if aggregate_fraction < MINIMUM_AGGREGATE_ARC_LAND_FRACTION {
        return Err(format!(
            "oceanic arc emergence {:.3}% is below preservation floor {:.3}%",
            aggregate_fraction * 100.0,
            MINIMUM_AGGREGATE_ARC_LAND_FRACTION * 100.0
        ));
    }
    if aggregate_fraction > MAXIMUM_AGGREGATE_ARC_LAND_FRACTION {
        return Err(format!(
            "oceanic arc emergence {:.3}% exceeds {:.3}%",
            aggregate_fraction * 100.0,
            MAXIMUM_AGGREGATE_ARC_LAND_FRACTION * 100.0
        ));
    }
    if maximum_seed_fraction > MAXIMUM_SEED_ARC_LAND_FRACTION {
        return Err(format!(
            "single-seed oceanic arc emergence {:.3}% exceeds {:.3}%",
            maximum_seed_fraction * 100.0,
            MAXIMUM_SEED_ARC_LAND_FRACTION * 100.0
        ));
    }
    if seeds_with_arc_land < MINIMUM_SEEDS_WITH_ARC_LAND {
        return Err(format!(
            "only {seeds_with_arc_land} seeds retain oceanic arc land; expected at least {MINIMUM_SEEDS_WITH_ARC_LAND}"
        ));
    }

    Ok(())
}
