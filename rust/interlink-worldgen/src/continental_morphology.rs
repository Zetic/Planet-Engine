use crate::{CrustKind, GeodesicTopology, WorldgenError};
use std::collections::{BTreeSet, VecDeque};
use std::f64::consts::PI;

pub const CONTINENTAL_MORPHOLOGY_SIGNIFICANT_AREA_FRACTION: f64 = 0.0025;
pub const CONTINENTAL_MORPHOLOGY_MAJOR_AREA_FRACTION: f64 = 0.015;
pub const CONTINENTAL_MORPHOLOGY_RANKED_LIMIT: usize = 8;

#[derive(Clone, Debug, PartialEq)]
pub struct ContinentalMorphologyComponent {
    pub anchor_sample: u32,
    pub sample_count: u32,
    pub area_fraction: f64,
    pub perimeter_rad: f64,
    pub diameter_rad: f64,
    pub plate_count: u32,
    pub elongation: f64,
    pub compactness: f64,
    pub constricted_sample_fraction: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContinentalMorphologySummary {
    pub all_component_count: u32,
    pub significant_component_count: u32,
    pub component_area_coefficient_of_variation: f64,
    pub largest_to_median_area_ratio: f64,
    pub maximum_elongation: f64,
    pub maximum_compactness: f64,
    pub largest_component_plate_count: u32,
    pub has_major_multiplate_component: bool,
    pub satellite_component_count: u32,
    pub satellite_area_fraction: f64,
    pub secondary_complement_component_count: u32,
    pub secondary_complement_area_fraction: f64,
    pub constricted_sample_fraction: f64,
    pub coastline_complexity_fine: f64,
    pub coastline_complexity_medium: f64,
    pub coastline_complexity_coarse: f64,
    pub medium_smoothing_rounds: u32,
    pub coarse_smoothing_rounds: u32,
    pub ranked_components: Vec<ContinentalMorphologyComponent>,
}

#[derive(Clone, Debug)]
struct RawComponent {
    anchor_sample: u32,
    samples: Vec<usize>,
    area_sr: f64,
    perimeter_rad: f64,
    diameter_rad: f64,
    plate_count: u32,
}

fn great_circle_arc(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
        .clamp(-1.0, 1.0)
        .acos()
}

fn connected_components(
    topology: &GeodesicTopology,
    mask: &[bool],
    plate_ids: &[u16],
) -> Vec<RawComponent> {
    let mut visited = vec![false; mask.len()];
    let mut components = Vec::new();
    for start in 0..mask.len() {
        if !mask[start] || visited[start] {
            continue;
        }
        visited[start] = true;
        let mut queue = VecDeque::from([start]);
        let mut samples = Vec::new();
        while let Some(sample) = queue.pop_front() {
            samples.push(sample);
            for neighbor in topology.neighbors_of(sample as u32) {
                let index = *neighbor as usize;
                if mask[index] && !visited[index] {
                    visited[index] = true;
                    queue.push_back(index);
                }
            }
        }

        let mut area_sr = 0.0_f64;
        let mut perimeter_rad = 0.0_f64;
        let mut plates = BTreeSet::new();
        for sample in &samples {
            area_sr += topology.dual_area_steradians()[*sample];
            plates.insert(plate_ids[*sample]);
            for (neighbor, arc_length) in topology
                .neighbors_of(*sample as u32)
                .iter()
                .zip(topology.neighbor_arc_lengths_of(*sample as u32).iter())
            {
                if !mask[*neighbor as usize] {
                    perimeter_rad += *arc_length;
                }
            }
        }

        let positions = topology.positions();
        let first = samples[0];
        let farthest_from_first = samples
            .iter()
            .copied()
            .max_by(|a, b| {
                great_circle_arc(positions[first], positions[*a])
                    .total_cmp(&great_circle_arc(positions[first], positions[*b]))
            })
            .unwrap_or(first);
        let farthest = samples
            .iter()
            .copied()
            .max_by(|a, b| {
                great_circle_arc(positions[farthest_from_first], positions[*a]).total_cmp(
                    &great_circle_arc(positions[farthest_from_first], positions[*b]),
                )
            })
            .unwrap_or(farthest_from_first);

        components.push(RawComponent {
            anchor_sample: start as u32,
            samples,
            area_sr,
            perimeter_rad,
            diameter_rad: great_circle_arc(positions[farthest_from_first], positions[farthest]),
            plate_count: plates.len() as u32,
        });
    }
    components
}

fn coefficient_of_variation(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    if mean <= 0.0 {
        return 0.0;
    }
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;
    variance.sqrt() / mean
}

fn component_compactness(component: &RawComponent) -> f64 {
    component.perimeter_rad.powi(2) / (4.0 * PI * component.area_sr.max(1.0e-18))
}

fn component_elongation(component: &RawComponent) -> f64 {
    let equivalent_radius = (component.area_sr / PI).sqrt().max(1.0e-9);
    component.diameter_rad / (2.0 * equivalent_radius)
}

fn component_constricted_fraction(
    topology: &GeodesicTopology,
    mask: &[bool],
    component: &RawComponent,
) -> f64 {
    if component.samples.is_empty() {
        return 0.0;
    }
    let constricted = component
        .samples
        .iter()
        .filter(|sample| {
            topology
                .neighbors_of(**sample as u32)
                .iter()
                .filter(|neighbor| mask[**neighbor as usize])
                .count()
                <= 2
        })
        .count();
    constricted as f64 / component.samples.len() as f64
}

fn area_weighted_complexity(
    topology: &GeodesicTopology,
    mask: &[bool],
    plate_ids: &[u16],
    significant_area_sr: f64,
) -> f64 {
    let components = connected_components(topology, mask, plate_ids);
    let mut weighted = 0.0_f64;
    let mut area = 0.0_f64;
    for component in components
        .iter()
        .filter(|component| component.area_sr >= significant_area_sr)
    {
        weighted += component_compactness(component) * component.area_sr;
        area += component.area_sr;
    }
    if area > 0.0 { weighted / area } else { 0.0 }
}

fn smooth_mask(topology: &GeodesicTopology, mask: &[bool], rounds: usize) -> Vec<bool> {
    let mut current = mask.to_vec();
    for _ in 0..rounds {
        let mut next = current.clone();
        for sample in 0..current.len() {
            let neighbors = topology.neighbors_of(sample as u32);
            let land_votes = usize::from(current[sample])
                + neighbors
                    .iter()
                    .filter(|neighbor| current[**neighbor as usize])
                    .count();
            let total_votes = neighbors.len() + 1;
            next[sample] = if land_votes * 2 > total_votes {
                true
            } else if land_votes * 2 < total_votes {
                false
            } else {
                current[sample]
            };
        }
        current = next;
    }
    current
}

fn smoothing_rounds(level: u8) -> (usize, usize) {
    let exponent = level.saturating_sub(4).min(3) as u32;
    let scale = 1usize << exponent;
    (scale, scale * 3)
}

fn complement_component_areas(topology: &GeodesicTopology, mask: &[bool]) -> Vec<f64> {
    let mut visited = vec![false; mask.len()];
    let mut areas = Vec::new();
    for start in 0..mask.len() {
        if mask[start] || visited[start] {
            continue;
        }
        visited[start] = true;
        let mut queue = VecDeque::from([start]);
        let mut area_sr = 0.0_f64;
        while let Some(sample) = queue.pop_front() {
            area_sr += topology.dual_area_steradians()[sample];
            for neighbor in topology.neighbors_of(sample as u32) {
                let index = *neighbor as usize;
                if !mask[index] && !visited[index] {
                    visited[index] = true;
                    queue.push_back(index);
                }
            }
        }
        areas.push(area_sr);
    }
    areas.sort_by(|a, b| b.total_cmp(a));
    areas
}

pub fn analyze_mask_morphology(
    topology: &GeodesicTopology,
    mask: &[bool],
    plate_ids: &[u16],
    significant_area_fraction: f64,
) -> Result<ContinentalMorphologySummary, WorldgenError> {
    let count = topology.metrics().sample_count as usize;
    if mask.len() != count || plate_ids.len() != count {
        return Err(WorldgenError::InvalidGeology(
            "continental morphology inputs must align with topology sample count",
        ));
    }
    if !significant_area_fraction.is_finite()
        || significant_area_fraction <= 0.0
        || significant_area_fraction >= 1.0
    {
        return Err(WorldgenError::InvalidGeology(
            "continental morphology significant area fraction must lie in (0, 1)",
        ));
    }

    let total_area_sr = topology.metrics().total_area_steradians.max(1.0e-18);
    let significant_area_sr = total_area_sr * significant_area_fraction;
    let mut all_components = connected_components(topology, mask, plate_ids);
    all_components.sort_by(|a, b| b.area_sr.total_cmp(&a.area_sr));
    let significant = all_components
        .iter()
        .filter(|component| component.area_sr >= significant_area_sr)
        .collect::<Vec<_>>();
    let satellites = all_components
        .iter()
        .filter(|component| component.area_sr < significant_area_sr)
        .collect::<Vec<_>>();

    let areas = significant
        .iter()
        .map(|component| component.area_sr)
        .collect::<Vec<_>>();
    let area_cv = coefficient_of_variation(&areas);
    let median = if areas.is_empty() {
        0.0
    } else {
        areas[areas.len() / 2]
    };
    let hierarchy = areas
        .first()
        .copied()
        .map(|largest| largest / median.max(1.0e-18))
        .unwrap_or(0.0);

    let mut maximum_elongation = 0.0_f64;
    let mut maximum_compactness = 0.0_f64;
    let mut significant_samples = 0_usize;
    let mut constricted_samples = 0_usize;
    let mut has_major_multiplate_component = false;
    let mut ranked_components = Vec::new();
    for component in &significant {
        let elongation = component_elongation(component);
        let compactness = component_compactness(component);
        let constricted_fraction = component_constricted_fraction(topology, mask, component);
        maximum_elongation = maximum_elongation.max(elongation);
        maximum_compactness = maximum_compactness.max(compactness);
        significant_samples += component.samples.len();
        constricted_samples += (constricted_fraction * component.samples.len() as f64).round() as usize;
        has_major_multiplate_component |= component.area_sr
            >= total_area_sr * CONTINENTAL_MORPHOLOGY_MAJOR_AREA_FRACTION
            && component.plate_count >= 2;
        if ranked_components.len() < CONTINENTAL_MORPHOLOGY_RANKED_LIMIT {
            ranked_components.push(ContinentalMorphologyComponent {
                anchor_sample: component.anchor_sample,
                sample_count: component.samples.len() as u32,
                area_fraction: component.area_sr / total_area_sr,
                perimeter_rad: component.perimeter_rad,
                diameter_rad: component.diameter_rad,
                plate_count: component.plate_count,
                elongation,
                compactness,
                constricted_sample_fraction: constricted_fraction,
            });
        }
    }

    let satellite_area_sr = satellites.iter().map(|component| component.area_sr).sum::<f64>();
    let complement_areas = complement_component_areas(topology, mask);
    let secondary_complement_area_sr = complement_areas.iter().skip(1).sum::<f64>();
    let (medium_rounds, coarse_rounds) = smoothing_rounds(topology.level());
    let medium_mask = smooth_mask(topology, mask, medium_rounds);
    let coarse_mask = smooth_mask(topology, mask, coarse_rounds);

    Ok(ContinentalMorphologySummary {
        all_component_count: all_components.len() as u32,
        significant_component_count: significant.len() as u32,
        component_area_coefficient_of_variation: area_cv,
        largest_to_median_area_ratio: hierarchy,
        maximum_elongation,
        maximum_compactness,
        largest_component_plate_count: significant
            .first()
            .map(|component| component.plate_count)
            .unwrap_or(0),
        has_major_multiplate_component,
        satellite_component_count: satellites.len() as u32,
        satellite_area_fraction: satellite_area_sr / total_area_sr,
        secondary_complement_component_count: complement_areas.len().saturating_sub(1) as u32,
        secondary_complement_area_fraction: secondary_complement_area_sr / total_area_sr,
        constricted_sample_fraction: if significant_samples > 0 {
            constricted_samples as f64 / significant_samples as f64
        } else {
            0.0
        },
        coastline_complexity_fine: area_weighted_complexity(
            topology,
            mask,
            plate_ids,
            significant_area_sr,
        ),
        coastline_complexity_medium: area_weighted_complexity(
            topology,
            &medium_mask,
            plate_ids,
            significant_area_sr,
        ),
        coastline_complexity_coarse: area_weighted_complexity(
            topology,
            &coarse_mask,
            plate_ids,
            significant_area_sr,
        ),
        medium_smoothing_rounds: medium_rounds as u32,
        coarse_smoothing_rounds: coarse_rounds as u32,
        ranked_components,
    })
}

pub fn analyze_continental_morphology(
    topology: &GeodesicTopology,
    crust_kind: &[u8],
    plate_ids: &[u16],
) -> Result<ContinentalMorphologySummary, WorldgenError> {
    let count = topology.metrics().sample_count as usize;
    if crust_kind.len() != count {
        return Err(WorldgenError::InvalidGeology(
            "continental morphology crust-kind input must align with topology sample count",
        ));
    }
    let mask = crust_kind
        .iter()
        .map(|kind| *kind == CrustKind::Continental as u8)
        .collect::<Vec<_>>();
    analyze_mask_morphology(
        topology,
        &mask,
        plate_ids,
        CONTINENTAL_MORPHOLOGY_SIGNIFICANT_AREA_FRACTION,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_icosphere;

    fn zero_plates(count: usize) -> Vec<u16> {
        vec![0; count]
    }

    #[test]
    fn morphology_analysis_is_deterministic() {
        let topology = build_icosphere(3).unwrap();
        let mask = topology
            .positions()
            .iter()
            .map(|position| position[2] > 0.2)
            .collect::<Vec<_>>();
        let plates = zero_plates(mask.len());
        let a = analyze_mask_morphology(&topology, &mask, &plates, 0.0025).unwrap();
        let b = analyze_mask_morphology(&topology, &mask, &plates, 0.0025).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.significant_component_count, 1);
        assert!(a.maximum_compactness.is_finite());
        assert!(a.maximum_elongation.is_finite());
    }

    #[test]
    fn tiny_remote_island_is_reported_as_satellite() {
        let topology = build_icosphere(3).unwrap();
        let mut mask = topology
            .positions()
            .iter()
            .map(|position| position[2] > 0.2)
            .collect::<Vec<_>>();
        let south = topology
            .positions()
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a[2].total_cmp(&b[2]))
            .map(|(index, _)| index)
            .unwrap();
        mask[south] = true;
        let plates = zero_plates(mask.len());
        let summary = analyze_mask_morphology(&topology, &mask, &plates, 0.0025).unwrap();
        assert!(summary.all_component_count >= 2);
        assert!(summary.satellite_component_count >= 1);
        assert!(summary.satellite_area_fraction > 0.0);
    }

    #[test]
    fn elongated_band_scores_above_compact_cap() {
        let topology = build_icosphere(3).unwrap();
        let compact = topology
            .positions()
            .iter()
            .map(|position| position[2] > 0.35)
            .collect::<Vec<_>>();
        let elongated = topology
            .positions()
            .iter()
            .map(|position| position[0] > 0.0 && position[2].abs() < 0.22)
            .collect::<Vec<_>>();
        let plates = zero_plates(compact.len());
        let compact_summary =
            analyze_mask_morphology(&topology, &compact, &plates, 0.0025).unwrap();
        let elongated_summary =
            analyze_mask_morphology(&topology, &elongated, &plates, 0.0025).unwrap();
        assert!(elongated_summary.maximum_elongation > compact_summary.maximum_elongation);
    }

    #[test]
    fn length_mismatch_is_rejected() {
        let topology = build_icosphere(2).unwrap();
        let mask = vec![false; topology.metrics().sample_count as usize - 1];
        let plates = zero_plates(topology.metrics().sample_count as usize);
        let error = analyze_mask_morphology(&topology, &mask, &plates, 0.0025).unwrap_err();
        assert!(matches!(error, WorldgenError::InvalidGeology(_)));
    }
}
