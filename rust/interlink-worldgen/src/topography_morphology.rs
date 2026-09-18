use crate::{
    GeodesicTopology, GeologicalBoundaryRegime, InheritedBoundarySet, InheritedPhysicalState,
    InheritedStructureKind, PlanetPhysicalParameters, TopographyState, WorldgenError,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub const TOPOGRAPHY_MORPHOLOGY_DISTANCE_BAND_EDGES_M: [f64; 6] =
    [0.0, 100_000.0, 250_000.0, 500_000.0, 750_000.0, 1_000_000.0];
pub const TOPOGRAPHY_MORPHOLOGY_OCEAN_AGE_BAND_EDGES_MYR: [f64; 7] =
    [0.0, 20.0, 50.0, 100.0, 150.0, 250.0, 500.0];
pub const TOPOGRAPHY_MORPHOLOGY_QUIET_OCEAN_MIN_BOUNDARY_DISTANCE_M: f64 = 750_000.0;

const CRUST_OCEANIC: u8 = 1;
const CRUST_TRANSITIONAL: u8 = 2;
const DISTANCE_EPSILON_M: f64 = 1.0e-6;
const GRADIENT_EPSILON: f64 = 1.0e-12;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopographyMorphologyFeature {
    OceanicRidge = 1,
    ContinentalRift = 2,
    TransitionalDivergence = 3,
    SubductionTrench = 4,
    VolcanicArc = 5,
    ContinentalOrogen = 6,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopographyDistanceBandMorphology {
    pub minimum_distance_m: f64,
    pub maximum_distance_m: f64,
    pub sample_count: u32,
    pub area_m2: f64,
    pub mean_component_relief_m: f64,
    pub mean_absolute_component_relief_m: f64,
    pub mean_solid_elevation_m: f64,
    pub submerged_area_fraction: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopographyReliefMorphology {
    pub feature: TopographyMorphologyFeature,
    pub source_edge_count: u32,
    pub source_sample_count: u32,
    pub peak_absolute_component_relief_m: f64,
    pub peak_mean_absolute_component_relief_m: f64,
    pub peak_distance_m: f64,
    pub effective_half_peak_width_m: f64,
    pub bands: Vec<TopographyDistanceBandMorphology>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OceanAgeDepthMorphology {
    pub minimum_age_myr: f64,
    pub maximum_age_myr: f64,
    pub sample_count: u32,
    pub area_m2: f64,
    pub mean_solid_elevation_m: f64,
    pub mean_water_depth_m: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuietOceanMorphology {
    pub sample_count: u32,
    pub area_m2: f64,
    pub oceanic_submerged_area_fraction: f64,
    pub mean_gradient_m_per_km: f64,
    pub gradient_edge_count: u64,
    pub mean_gradient_turn_degrees: f64,
    pub rms_gradient_turn_degrees: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuietProvenanceContactMorphology {
    pub contact_edge_count: u64,
    pub interior_edge_count: u64,
    pub mean_contact_gradient_m_per_km: f64,
    pub mean_interior_gradient_m_per_km: f64,
    pub rms_contact_gradient_m_per_km: f64,
    pub rms_interior_gradient_m_per_km: f64,
    pub contact_to_interior_gradient_ratio: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoundaryRegimeCoherence {
    pub edge_count: u32,
    pub edge_with_same_pair_neighbors_count: u32,
    pub isolated_regime_edge_count: u32,
    pub isolated_regime_edge_fraction: f64,
    pub mean_same_pair_neighbor_agreement: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InlandMarineMorphology {
    pub non_oceanic_marine_sample_count: u32,
    pub non_oceanic_marine_area_m2: f64,
    pub maximum_distance_from_oceanic_crust_m: f64,
    pub unsupported_inland_sample_count: u32,
    pub unsupported_inland_area_m2: f64,
    pub unsupported_inland_area_fraction: f64,
    pub maximum_unsupported_distance_from_oceanic_crust_m: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopographyMorphologyReport {
    pub topology_level: u8,
    pub sample_count: u32,
    pub topography_hash: u64,
    pub boundary_hash: u64,
    pub ridge: TopographyReliefMorphology,
    pub continental_rift: TopographyReliefMorphology,
    pub transitional_divergence: TopographyReliefMorphology,
    pub trench: TopographyReliefMorphology,
    pub arc: TopographyReliefMorphology,
    pub orogen: TopographyReliefMorphology,
    pub ocean_age_depth: Vec<OceanAgeDepthMorphology>,
    pub ocean_age_depth_monotonic_pair_fraction: f64,
    pub ocean_age_depth_inversion_count: u32,
    pub quiet_ocean: QuietOceanMorphology,
    pub quiet_provenance_contacts: QuietProvenanceContactMorphology,
    pub boundary_regime_coherence: BoundaryRegimeCoherence,
    pub inland_marine: InlandMarineMorphology,
}

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

fn validate_inputs(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    terrain: &TopographyState,
) -> Result<(), WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    let count = topology.metrics().sample_count as usize;
    let lengths = [
        inherited.crust_kind.len(),
        inherited.crust_age_myr.len(),
        inherited.crust_province_id.len(),
        inherited.plate_ids.len(),
        inherited.province_kind.len(),
        inherited.structural_zone_kind.len(),
        inherited.rift_history.len(),
        inherited.subsidence_history.len(),
        inherited.basin_potential.len(),
        terrain.solid_elevation_m.len(),
        terrain.water_depth_m.len(),
        terrain.submerged_mask.len(),
        terrain.orogenic_elevation_m.len(),
        terrain.ridge_elevation_m.len(),
        terrain.rift_basin_elevation_m.len(),
        terrain.trench_elevation_m.len(),
        terrain.arc_elevation_m.len(),
    ];
    if lengths.iter().any(|length| *length != count) {
        return Err(WorldgenError::InvalidTopography(
            "WG-4 morphology fields are not aligned to the fine topology",
        ));
    }
    if boundaries
        .boundaries
        .iter()
        .any(|edge| edge.sample_a as usize >= count || edge.sample_b as usize >= count)
    {
        return Err(WorldgenError::InvalidTopography(
            "WG-4 morphology boundary sample is outside the fine topology",
        ));
    }
    if inherited
        .crust_age_myr
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
        || terrain
            .solid_elevation_m
            .iter()
            .chain(terrain.water_depth_m.iter())
            .chain(terrain.orogenic_elevation_m.iter())
            .chain(terrain.ridge_elevation_m.iter())
            .chain(terrain.rift_basin_elevation_m.iter())
            .chain(terrain.trench_elevation_m.iter())
            .chain(terrain.arc_elevation_m.iter())
            .any(|value| !value.is_finite())
    {
        return Err(WorldgenError::InvalidTopography(
            "WG-4 morphology input contains non-finite physical values",
        ));
    }
    Ok(())
}

fn boundary_distances<F>(
    topology: &GeodesicTopology,
    boundaries: &InheritedBoundarySet,
    radius_m: f64,
    include: F,
) -> (Vec<f64>, u32, u32)
where
    F: Fn(GeologicalBoundaryRegime) -> bool,
{
    let count = topology.metrics().sample_count as usize;
    let mut distance = vec![f64::INFINITY; count];
    let mut queue = BinaryHeap::new();
    let mut source_mask = vec![false; count];
    let mut source_edge_count = 0_u32;

    for edge in &boundaries.boundaries {
        if !include(edge.geological_regime) {
            continue;
        }
        source_edge_count += 1;
        for sample in [edge.sample_a, edge.sample_b] {
            let index = sample as usize;
            source_mask[index] = true;
            if distance[index] > 0.0 {
                distance[index] = 0.0;
                queue.push(QueueEntry {
                    distance_m: 0.0,
                    sample,
                });
            }
        }
    }

    while let Some(entry) = queue.pop() {
        let index = entry.sample as usize;
        if entry.distance_m > distance[index] + DISTANCE_EPSILON_M {
            continue;
        }
        for (neighbor, arc) in topology
            .neighbors_of(entry.sample)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(entry.sample).iter())
        {
            let candidate = entry.distance_m + *arc * radius_m;
            let target = *neighbor as usize;
            if candidate + DISTANCE_EPSILON_M < distance[target] {
                distance[target] = candidate;
                queue.push(QueueEntry {
                    distance_m: candidate,
                    sample: *neighbor,
                });
            }
        }
    }

    let source_sample_count = source_mask.iter().filter(|value| **value).count() as u32;
    (distance, source_edge_count, source_sample_count)
}

fn relief_profile(
    feature: TopographyMorphologyFeature,
    topology: &GeodesicTopology,
    planet: PlanetPhysicalParameters,
    terrain: &TopographyState,
    component: &[f32],
    distances: &[f64],
    source_edge_count: u32,
    source_sample_count: u32,
) -> TopographyReliefMorphology {
    let mut bands = Vec::with_capacity(TOPOGRAPHY_MORPHOLOGY_DISTANCE_BAND_EDGES_M.len() - 1);
    let mut peak_absolute_component_relief_m = 0.0_f64;

    for pair in TOPOGRAPHY_MORPHOLOGY_DISTANCE_BAND_EDGES_M.windows(2) {
        let minimum_distance_m = pair[0];
        let maximum_distance_m = pair[1];
        let mut sample_count = 0_u32;
        let mut area_m2 = 0.0_f64;
        let mut component_sum = 0.0_f64;
        let mut absolute_component_sum = 0.0_f64;
        let mut solid_sum = 0.0_f64;
        let mut submerged_area_m2 = 0.0_f64;

        for index in 0..distances.len() {
            let distance = distances[index];
            if distance < minimum_distance_m || distance >= maximum_distance_m {
                continue;
            }
            let area = topology.dual_area_steradians()[index] * planet.radius_m * planet.radius_m;
            let relief = component[index] as f64;
            sample_count += 1;
            area_m2 += area;
            component_sum += relief * area;
            absolute_component_sum += relief.abs() * area;
            solid_sum += terrain.solid_elevation_m[index] as f64 * area;
            if terrain.submerged_mask[index] != 0 {
                submerged_area_m2 += area;
            }
            peak_absolute_component_relief_m = peak_absolute_component_relief_m.max(relief.abs());
        }

        bands.push(TopographyDistanceBandMorphology {
            minimum_distance_m,
            maximum_distance_m,
            sample_count,
            area_m2,
            mean_component_relief_m: if area_m2 > 0.0 {
                component_sum / area_m2
            } else {
                0.0
            },
            mean_absolute_component_relief_m: if area_m2 > 0.0 {
                absolute_component_sum / area_m2
            } else {
                0.0
            },
            mean_solid_elevation_m: if area_m2 > 0.0 {
                solid_sum / area_m2
            } else {
                0.0
            },
            submerged_area_fraction: if area_m2 > 0.0 {
                submerged_area_m2 / area_m2
            } else {
                0.0
            },
        });
    }

    let mut peak_band_index = 0_usize;
    let mut peak_mean_absolute_component_relief_m = 0.0_f64;
    for (index, band) in bands.iter().enumerate() {
        if band.mean_absolute_component_relief_m > peak_mean_absolute_component_relief_m {
            peak_mean_absolute_component_relief_m = band.mean_absolute_component_relief_m;
            peak_band_index = index;
        }
    }
    let peak_distance_m = if peak_mean_absolute_component_relief_m > 0.0 {
        0.5 * (bands[peak_band_index].minimum_distance_m
            + bands[peak_band_index].maximum_distance_m)
    } else {
        0.0
    };
    let half_peak = peak_mean_absolute_component_relief_m * 0.5;
    let effective_half_peak_width_m = bands
        .iter()
        .filter(|band| half_peak > 0.0 && band.mean_absolute_component_relief_m >= half_peak)
        .map(|band| band.maximum_distance_m)
        .fold(0.0_f64, f64::max);

    TopographyReliefMorphology {
        feature,
        source_edge_count,
        source_sample_count,
        peak_absolute_component_relief_m,
        peak_mean_absolute_component_relief_m,
        peak_distance_m,
        effective_half_peak_width_m,
        bands,
    }
}

fn ocean_age_depth_profile(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    planet: PlanetPhysicalParameters,
    terrain: &TopographyState,
) -> Vec<OceanAgeDepthMorphology> {
    let mut output = Vec::with_capacity(TOPOGRAPHY_MORPHOLOGY_OCEAN_AGE_BAND_EDGES_MYR.len() - 1);
    for pair in TOPOGRAPHY_MORPHOLOGY_OCEAN_AGE_BAND_EDGES_MYR.windows(2) {
        let minimum_age_myr = pair[0];
        let maximum_age_myr = pair[1];
        let mut sample_count = 0_u32;
        let mut area_m2 = 0.0_f64;
        let mut solid_sum = 0.0_f64;
        let mut depth_sum = 0.0_f64;
        for index in 0..inherited.crust_age_myr.len() {
            let age = inherited.crust_age_myr[index] as f64;
            if inherited.crust_kind[index] != CRUST_OCEANIC
                || terrain.submerged_mask[index] == 0
                || age < minimum_age_myr
                || age >= maximum_age_myr
            {
                continue;
            }
            let area = topology.dual_area_steradians()[index] * planet.radius_m * planet.radius_m;
            sample_count += 1;
            area_m2 += area;
            solid_sum += terrain.solid_elevation_m[index] as f64 * area;
            depth_sum += terrain.water_depth_m[index] as f64 * area;
        }
        output.push(OceanAgeDepthMorphology {
            minimum_age_myr,
            maximum_age_myr,
            sample_count,
            area_m2,
            mean_solid_elevation_m: if area_m2 > 0.0 {
                solid_sum / area_m2
            } else {
                0.0
            },
            mean_water_depth_m: if area_m2 > 0.0 {
                depth_sum / area_m2
            } else {
                0.0
            },
        });
    }
    output
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn norm(value: [f64; 3]) -> f64 {
    dot(value, value).sqrt()
}
fn normalize(value: [f64; 3]) -> Option<[f64; 3]> {
    let magnitude = norm(value);
    if magnitude <= GRADIENT_EPSILON {
        None
    } else {
        Some([
            value[0] / magnitude,
            value[1] / magnitude,
            value[2] / magnitude,
        ])
    }
}

fn quiet_ocean_morphology(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    terrain: &TopographyState,
) -> QuietOceanMorphology {
    let (boundary_distance, _, _) =
        boundary_distances(topology, boundaries, planet.radius_m, |_| true);
    let count = topology.metrics().sample_count as usize;
    let mut eligible = vec![false; count];
    let mut oceanic_submerged_area_m2 = 0.0_f64;
    let mut quiet_area_m2 = 0.0_f64;
    let mut sample_count = 0_u32;

    for index in 0..count {
        if inherited.crust_kind[index] == CRUST_OCEANIC && terrain.submerged_mask[index] != 0 {
            let area = topology.dual_area_steradians()[index] * planet.radius_m * planet.radius_m;
            oceanic_submerged_area_m2 += area;
            if boundary_distance[index] >= TOPOGRAPHY_MORPHOLOGY_QUIET_OCEAN_MIN_BOUNDARY_DISTANCE_M
            {
                eligible[index] = true;
                quiet_area_m2 += area;
                sample_count += 1;
            }
        }
    }

    let mut gradients = vec![[0.0_f64; 3]; count];
    let mut gradient_area_sum = 0.0_f64;
    let mut gradient_magnitude_area_sum = 0.0_f64;
    for sample in 0..count as u32 {
        let index = sample as usize;
        if !eligible[index] {
            continue;
        }
        let origin = topology.positions()[index];
        let mut gradient = [0.0_f64; 3];
        let mut weight_sum = 0.0_f64;
        for ((neighbor, arc), interface_arc) in topology
            .neighbors_of(sample)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(sample).iter())
            .zip(topology.neighbor_interface_arc_lengths_of(sample).iter())
        {
            let target = topology.positions()[*neighbor as usize];
            let projection = [
                target[0] - dot(target, origin) * origin[0],
                target[1] - dot(target, origin) * origin[1],
                target[2] - dot(target, origin) * origin[2],
            ];
            let Some(direction) = normalize(projection) else {
                continue;
            };
            let distance_m = (*arc * planet.radius_m).max(1.0);
            let slope = (terrain.solid_elevation_m[*neighbor as usize] as f64
                - terrain.solid_elevation_m[index] as f64)
                / distance_m;
            let weight = *interface_arc;
            gradient[0] += direction[0] * slope * weight;
            gradient[1] += direction[1] * slope * weight;
            gradient[2] += direction[2] * slope * weight;
            weight_sum += weight;
        }
        if weight_sum > 0.0 {
            gradient[0] /= weight_sum;
            gradient[1] /= weight_sum;
            gradient[2] /= weight_sum;
        }
        gradients[index] = gradient;
        let area = topology.dual_area_steradians()[index] * planet.radius_m * planet.radius_m;
        gradient_area_sum += area;
        gradient_magnitude_area_sum += norm(gradient) * 1_000.0 * area;
    }

    let mut gradient_edge_count = 0_u64;
    let mut turn_weight_sum = 0.0_f64;
    let mut turn_sum = 0.0_f64;
    let mut turn_square_sum = 0.0_f64;
    for sample in 0..count as u32 {
        let index = sample as usize;
        if !eligible[index] {
            continue;
        }
        let Some(a) = normalize(gradients[index]) else {
            continue;
        };
        for (neighbor, interface_arc) in topology
            .neighbors_of(sample)
            .iter()
            .zip(topology.neighbor_interface_arc_lengths_of(sample).iter())
        {
            if sample >= *neighbor || !eligible[*neighbor as usize] {
                continue;
            }
            let Some(b) = normalize(gradients[*neighbor as usize]) else {
                continue;
            };
            let angle_degrees = dot(a, b).clamp(-1.0, 1.0).acos().to_degrees();
            let weight = *interface_arc * planet.radius_m;
            gradient_edge_count += 1;
            turn_weight_sum += weight;
            turn_sum += angle_degrees * weight;
            turn_square_sum += angle_degrees * angle_degrees * weight;
        }
    }

    QuietOceanMorphology {
        sample_count,
        area_m2: quiet_area_m2,
        oceanic_submerged_area_fraction: if oceanic_submerged_area_m2 > 0.0 {
            quiet_area_m2 / oceanic_submerged_area_m2
        } else {
            0.0
        },
        mean_gradient_m_per_km: if gradient_area_sum > 0.0 {
            gradient_magnitude_area_sum / gradient_area_sum
        } else {
            0.0
        },
        gradient_edge_count,
        mean_gradient_turn_degrees: if turn_weight_sum > 0.0 {
            turn_sum / turn_weight_sum
        } else {
            0.0
        },
        rms_gradient_turn_degrees: if turn_weight_sum > 0.0 {
            (turn_square_sum / turn_weight_sum).sqrt()
        } else {
            0.0
        },
    }
}


fn explicit_structure(kind: u8) -> bool {
    kind == InheritedStructureKind::PaleoSuture as u8
        || kind == InheritedStructureKind::InheritedRift as u8
        || kind == InheritedStructureKind::ShearZone as u8
        || kind == InheritedStructureKind::ContinentalMargin as u8
}

fn quiet_provenance_contact_morphology(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    planet: PlanetPhysicalParameters,
    terrain: &TopographyState,
) -> QuietProvenanceContactMorphology {
    let mut contact_edge_count = 0_u64;
    let mut interior_edge_count = 0_u64;
    let mut contact_weight = 0.0_f64;
    let mut interior_weight = 0.0_f64;
    let mut contact_sum = 0.0_f64;
    let mut interior_sum = 0.0_f64;
    let mut contact_square_sum = 0.0_f64;
    let mut interior_square_sum = 0.0_f64;

    for sample in 0..topology.metrics().sample_count {
        let a = sample as usize;
        if inherited.crust_kind[a] == CRUST_OCEANIC
            || inherited.province_kind[a] != 0
            || explicit_structure(inherited.structural_zone_kind[a])
            || f64::from(inherited.rift_history[a]) >= 0.18
            || f64::from(inherited.subsidence_history[a]) >= 0.22
            || f64::from(inherited.basin_potential[a]) >= 0.24
        {
            continue;
        }
        for ((neighbor, arc), interface_arc) in topology
            .neighbors_of(sample)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(sample).iter())
            .zip(topology.neighbor_interface_arc_lengths_of(sample).iter())
        {
            if *neighbor <= sample {
                continue;
            }
            let b = *neighbor as usize;
            if inherited.crust_kind[b] != inherited.crust_kind[a]
                || inherited.plate_ids[b] != inherited.plate_ids[a]
                || inherited.province_kind[b] != 0
                || explicit_structure(inherited.structural_zone_kind[b])
                || f64::from(inherited.rift_history[b]) >= 0.18
                || f64::from(inherited.subsidence_history[b]) >= 0.22
                || f64::from(inherited.basin_potential[b]) >= 0.24
            {
                continue;
            }

            let distance_m = (*arc * planet.radius_m).max(1.0);
            let gradient = ((f64::from(terrain.solid_elevation_m[a])
                - f64::from(terrain.solid_elevation_m[b]))
                .abs()
                / distance_m)
                * 1_000.0;
            let weight = (*interface_arc * planet.radius_m).max(1.0);
            if inherited.crust_province_id[a] != inherited.crust_province_id[b] {
                contact_edge_count += 1;
                contact_weight += weight;
                contact_sum += gradient * weight;
                contact_square_sum += gradient * gradient * weight;
            } else {
                interior_edge_count += 1;
                interior_weight += weight;
                interior_sum += gradient * weight;
                interior_square_sum += gradient * gradient * weight;
            }
        }
    }

    let mean_contact = if contact_weight > 0.0 {
        contact_sum / contact_weight
    } else {
        0.0
    };
    let mean_interior = if interior_weight > 0.0 {
        interior_sum / interior_weight
    } else {
        0.0
    };
    QuietProvenanceContactMorphology {
        contact_edge_count,
        interior_edge_count,
        mean_contact_gradient_m_per_km: mean_contact,
        mean_interior_gradient_m_per_km: mean_interior,
        rms_contact_gradient_m_per_km: if contact_weight > 0.0 {
            (contact_square_sum / contact_weight).sqrt()
        } else {
            0.0
        },
        rms_interior_gradient_m_per_km: if interior_weight > 0.0 {
            (interior_square_sum / interior_weight).sqrt()
        } else {
            0.0
        },
        contact_to_interior_gradient_ratio: if mean_interior > 1.0e-12 {
            mean_contact / mean_interior
        } else {
            0.0
        },
    }
}

fn normalized_plate_pair(a: u16, b: u16) -> (u16, u16) {
    if a < b { (a, b) } else { (b, a) }
}

fn boundary_regime_coherence(
    topology: &GeodesicTopology,
    boundaries: &InheritedBoundarySet,
) -> BoundaryRegimeCoherence {
    let mut incident = vec![Vec::<usize>::new(); topology.metrics().sample_count as usize];
    for (index, edge) in boundaries.boundaries.iter().enumerate() {
        incident[edge.sample_a as usize].push(index);
        incident[edge.sample_b as usize].push(index);
    }

    let mut edges_with_neighbors = 0_u32;
    let mut isolated = 0_u32;
    let mut agreement_sum = 0.0_f64;
    for (index, edge) in boundaries.boundaries.iter().enumerate() {
        let pair = normalized_plate_pair(edge.plate_a, edge.plate_b);
        let mut neighboring = Vec::<usize>::new();
        for sample in [edge.sample_a, edge.sample_b] {
            for candidate in &incident[sample as usize] {
                if *candidate != index
                    && normalized_plate_pair(
                        boundaries.boundaries[*candidate].plate_a,
                        boundaries.boundaries[*candidate].plate_b,
                    ) == pair
                    && !neighboring.contains(candidate)
                {
                    neighboring.push(*candidate);
                }
            }
        }
        if neighboring.is_empty() {
            continue;
        }
        edges_with_neighbors += 1;
        let agreeing = neighboring
            .iter()
            .filter(|candidate| {
                boundaries.boundaries[**candidate].geological_regime == edge.geological_regime
            })
            .count();
        agreement_sum += agreeing as f64 / neighboring.len() as f64;
        if neighboring.len() >= 2 && agreeing == 0 {
            isolated += 1;
        }
    }

    BoundaryRegimeCoherence {
        edge_count: boundaries.boundaries.len() as u32,
        edge_with_same_pair_neighbors_count: edges_with_neighbors,
        isolated_regime_edge_count: isolated,
        isolated_regime_edge_fraction: if edges_with_neighbors > 0 {
            f64::from(isolated) / f64::from(edges_with_neighbors)
        } else {
            0.0
        },
        mean_same_pair_neighbor_agreement: if edges_with_neighbors > 0 {
            agreement_sum / f64::from(edges_with_neighbors)
        } else {
            1.0
        },
    }
}

fn oceanic_crust_distances(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    terrain: &TopographyState,
    radius_m: f64,
) -> Vec<f64> {
    let count = topology.metrics().sample_count as usize;
    let mut distance = vec![f64::INFINITY; count];
    let mut queue = BinaryHeap::new();
    for sample in 0..topology.metrics().sample_count {
        let index = sample as usize;
        if inherited.crust_kind[index] == CRUST_OCEANIC && terrain.submerged_mask[index] != 0 {
            distance[index] = 0.0;
            queue.push(QueueEntry { distance_m: 0.0, sample });
        }
    }
    while let Some(entry) = queue.pop() {
        let index = entry.sample as usize;
        if entry.distance_m > distance[index] + DISTANCE_EPSILON_M {
            continue;
        }
        for (neighbor, arc) in topology
            .neighbors_of(entry.sample)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(entry.sample).iter())
        {
            let candidate = entry.distance_m + *arc * radius_m;
            let ni = *neighbor as usize;
            if candidate + DISTANCE_EPSILON_M < distance[ni] {
                distance[ni] = candidate;
                queue.push(QueueEntry {
                    distance_m: candidate,
                    sample: *neighbor,
                });
            }
        }
    }
    distance
}

fn inland_marine_morphology(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    planet: PlanetPhysicalParameters,
    terrain: &TopographyState,
) -> InlandMarineMorphology {
    let distances = oceanic_crust_distances(topology, inherited, terrain, planet.radius_m);
    let marine_access =
        crate::causal_pipeline::marine_connectivity_access_mask(topology, inherited, planet);
    let mut sample_count = 0_u32;
    let mut area_m2 = 0.0_f64;
    let mut max_distance = 0.0_f64;
    let mut unsupported_count = 0_u32;
    let mut unsupported_area = 0.0_f64;
    let mut max_unsupported_distance = 0.0_f64;

    for sample in 0..topology.metrics().sample_count as usize {
        if terrain.submerged_mask[sample] == 0 || inherited.crust_kind[sample] == CRUST_OCEANIC {
            continue;
        }
        let area = topology.dual_area_steradians()[sample] * planet.radius_m * planet.radius_m;
        sample_count += 1;
        area_m2 += area;
        max_distance = max_distance.max(distances[sample]);

        let causal_support = marine_access[sample] != 0;
        if !causal_support {
            unsupported_count += 1;
            unsupported_area += area;
            max_unsupported_distance = max_unsupported_distance.max(distances[sample]);
        }
    }

    InlandMarineMorphology {
        non_oceanic_marine_sample_count: sample_count,
        non_oceanic_marine_area_m2: area_m2,
        maximum_distance_from_oceanic_crust_m: max_distance,
        unsupported_inland_sample_count: unsupported_count,
        unsupported_inland_area_m2: unsupported_area,
        unsupported_inland_area_fraction: if area_m2 > 0.0 {
            unsupported_area / area_m2
        } else {
            0.0
        },
        maximum_unsupported_distance_from_oceanic_crust_m: max_unsupported_distance,
    }
}

fn ocean_age_depth_monotonicity(
    bands: &[OceanAgeDepthMorphology],
) -> (f64, u32) {
    let populated = bands
        .iter()
        .filter(|band| band.sample_count > 0)
        .collect::<Vec<_>>();
    if populated.len() < 2 {
        return (1.0, 0);
    }
    let mut comparisons = 0_u32;
    let mut monotonic = 0_u32;
    let mut inversions = 0_u32;
    for pair in populated.windows(2) {
        comparisons += 1;
        if pair[1].mean_water_depth_m + 50.0 >= pair[0].mean_water_depth_m {
            monotonic += 1;
        } else {
            inversions += 1;
        }
    }
    (f64::from(monotonic) / f64::from(comparisons.max(1)), inversions)
}

pub fn analyze_topography_morphology(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    terrain: &TopographyState,
) -> Result<TopographyMorphologyReport, WorldgenError> {
    validate_inputs(topology, inherited, boundaries, planet, terrain)?;

    let (ridge_distance, ridge_edges, ridge_samples) =
        boundary_distances(topology, boundaries, planet.radius_m, |regime| {
            regime == GeologicalBoundaryRegime::OceanicRidge
        });
    let (rift_distance, rift_edges, rift_samples) =
        boundary_distances(topology, boundaries, planet.radius_m, |regime| {
            regime == GeologicalBoundaryRegime::ContinentalRift
        });
    let (transition_distance, transition_edges, transition_samples) =
        boundary_distances(topology, boundaries, planet.radius_m, |regime| {
            regime == GeologicalBoundaryRegime::TransitionalDivergence
        });
    let (subduction_distance, subduction_edges, subduction_samples) =
        boundary_distances(topology, boundaries, planet.radius_m, |regime| {
            matches!(
                regime,
                GeologicalBoundaryRegime::OceanicSubduction
                    | GeologicalBoundaryRegime::OceanContinentSubduction
            )
        });
    let (collision_distance, collision_edges, collision_samples) =
        boundary_distances(topology, boundaries, planet.radius_m, |regime| {
            regime == GeologicalBoundaryRegime::ContinentalCollision
        });

    let ocean_age_depth = ocean_age_depth_profile(topology, inherited, planet, terrain);
    let (ocean_age_depth_monotonic_pair_fraction, ocean_age_depth_inversion_count) =
        ocean_age_depth_monotonicity(&ocean_age_depth);

    Ok(TopographyMorphologyReport {
        topology_level: topology.level(),
        sample_count: topology.metrics().sample_count,
        topography_hash: terrain.metrics.topography_hash,
        boundary_hash: boundaries.boundary_hash,
        ridge: relief_profile(
            TopographyMorphologyFeature::OceanicRidge,
            topology,
            planet,
            terrain,
            &terrain.ridge_elevation_m,
            &ridge_distance,
            ridge_edges,
            ridge_samples,
        ),
        continental_rift: relief_profile(
            TopographyMorphologyFeature::ContinentalRift,
            topology,
            planet,
            terrain,
            &terrain.rift_basin_elevation_m,
            &rift_distance,
            rift_edges,
            rift_samples,
        ),
        transitional_divergence: relief_profile(
            TopographyMorphologyFeature::TransitionalDivergence,
            topology,
            planet,
            terrain,
            &terrain.rift_basin_elevation_m,
            &transition_distance,
            transition_edges,
            transition_samples,
        ),
        trench: relief_profile(
            TopographyMorphologyFeature::SubductionTrench,
            topology,
            planet,
            terrain,
            &terrain.trench_elevation_m,
            &subduction_distance,
            subduction_edges,
            subduction_samples,
        ),
        arc: relief_profile(
            TopographyMorphologyFeature::VolcanicArc,
            topology,
            planet,
            terrain,
            &terrain.arc_elevation_m,
            &subduction_distance,
            subduction_edges,
            subduction_samples,
        ),
        orogen: relief_profile(
            TopographyMorphologyFeature::ContinentalOrogen,
            topology,
            planet,
            terrain,
            &terrain.orogenic_elevation_m,
            &collision_distance,
            collision_edges,
            collision_samples,
        ),
        ocean_age_depth,
        ocean_age_depth_monotonic_pair_fraction,
        ocean_age_depth_inversion_count,
        quiet_ocean: quiet_ocean_morphology(topology, inherited, boundaries, planet, terrain),
        quiet_provenance_contacts: quiet_provenance_contact_morphology(
            topology, inherited, planet, terrain,
        ),
        boundary_regime_coherence: boundary_regime_coherence(topology, boundaries),
        inland_marine: inland_marine_morphology(topology, inherited, planet, terrain),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_icosphere, generate_crust_and_history, generate_initial_topography,
        generate_lithosphere, generate_tectonics, inherit_boundary_interfaces,
        inherit_physical_state, GeologyRequest, LithosphereRequest, TectonicsRequest,
        TopographyRequest,
    };

    #[test]
    fn morphology_analysis_is_deterministic_and_observational() {
        let seed = "wg4-morphology-diagnostics";
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let coarse = build_icosphere(3).unwrap();
        let fine = build_icosphere(4).unwrap();
        let tectonics =
            generate_tectonics(&coarse, &TectonicsRequest::new(seed, 8), planet).unwrap();
        let geology =
            generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet)
                .unwrap();
        let lithosphere = generate_lithosphere(
            &coarse,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed),
        )
        .unwrap();
        let inherited =
            inherit_physical_state(&fine, 3, &tectonics, &geology, &lithosphere, planet).unwrap();
        let boundaries =
            inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids)
                .unwrap();
        let terrain = generate_initial_topography(
            &fine,
            &inherited,
            &boundaries,
            planet,
            &TopographyRequest::new(seed),
        )
        .unwrap();
        let original = terrain.clone();

        let first = analyze_topography_morphology(&fine, &inherited, &boundaries, planet, &terrain)
            .unwrap();
        let second =
            analyze_topography_morphology(&fine, &inherited, &boundaries, planet, &terrain)
                .unwrap();

        assert_eq!(first, second);
        assert_eq!(terrain, original);
        assert_eq!(first.topography_hash, terrain.metrics.topography_hash);
        assert_eq!(first.boundary_hash, boundaries.boundary_hash);
        assert_eq!(
            first.ridge.bands.len(),
            TOPOGRAPHY_MORPHOLOGY_DISTANCE_BAND_EDGES_M.len() - 1
        );
        assert_eq!(
            first.ocean_age_depth.len(),
            TOPOGRAPHY_MORPHOLOGY_OCEAN_AGE_BAND_EDGES_MYR.len() - 1
        );
        for profile in [
            &first.ridge,
            &first.continental_rift,
            &first.transitional_divergence,
            &first.trench,
            &first.arc,
            &first.orogen,
        ] {
            assert!(profile.peak_absolute_component_relief_m.is_finite());
            assert!(profile.peak_mean_absolute_component_relief_m.is_finite());
            assert!(profile.peak_distance_m.is_finite());
            assert!(profile.effective_half_peak_width_m.is_finite());
            assert!(profile.bands.iter().all(|band| {
                band.area_m2.is_finite()
                    && band.mean_component_relief_m.is_finite()
                    && band.mean_absolute_component_relief_m.is_finite()
                    && band.mean_solid_elevation_m.is_finite()
                    && band.submerged_area_fraction.is_finite()
            }));
        }
        assert!(first
            .quiet_ocean
            .oceanic_submerged_area_fraction
            .is_finite());
        assert!(first.quiet_ocean.mean_gradient_m_per_km.is_finite());
        assert!(first.quiet_ocean.mean_gradient_turn_degrees.is_finite());
        assert!(first.quiet_ocean.rms_gradient_turn_degrees.is_finite());
        assert!(first
            .quiet_provenance_contacts
            .mean_contact_gradient_m_per_km
            .is_finite());
        assert!(first
            .quiet_provenance_contacts
            .contact_to_interior_gradient_ratio
            .is_finite());
        assert!(first
            .boundary_regime_coherence
            .isolated_regime_edge_fraction
            .is_finite());
        assert!(first
            .boundary_regime_coherence
            .mean_same_pair_neighbor_agreement
            .is_finite());
        assert!(first.inland_marine.unsupported_inland_area_fraction.is_finite());
        assert!(first
            .inland_marine
            .maximum_unsupported_distance_from_oceanic_crust_m
            .is_finite());
        assert!((0.0..=1.0).contains(&first.ocean_age_depth_monotonic_pair_fraction));
    }
}
