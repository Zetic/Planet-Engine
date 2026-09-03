use crate::{
    CrustalModel, GeodesicTopology, GeologicalBoundaryRegime, PlateBoundaryKind,
    SubductionPolarity, TectonicModel, WorldgenError,
};
use std::collections::BTreeMap;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const DISTANCE_TIE_EPSILON: f64 = 1.0e-15;

#[derive(Clone, Debug, PartialEq)]
pub struct InheritedBoundaryEdge {
    pub sample_a: u32,
    pub sample_b: u32,
    pub plate_a: u16,
    pub plate_b: u16,
    pub tectonic_kind: PlateBoundaryKind,
    pub geological_regime: GeologicalBoundaryRegime,
    pub subduction_polarity: SubductionPolarity,
    pub normal_rate_m_per_year: f64,
    pub shear_rate_m_per_year: f64,
    pub coarse_boundary_index: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InheritedBoundarySet {
    pub boundaries: Vec<InheritedBoundaryEdge>,
    pub boundary_hash: u64,
}

impl InheritedBoundarySet {
    pub fn boundary_hash_hex(&self) -> String {
        format!("{:016x}", self.boundary_hash)
    }

    pub fn flattened_samples(&self) -> Vec<u32> {
        let mut output = Vec::with_capacity(self.boundaries.len() * 2);
        for boundary in &self.boundaries {
            output.extend_from_slice(&[boundary.sample_a, boundary.sample_b]);
        }
        output
    }

    pub fn coarse_boundary_indices(&self) -> Vec<u32> {
        self.boundaries
            .iter()
            .map(|boundary| boundary.coarse_boundary_index)
            .collect()
    }

    pub fn tectonic_kinds(&self) -> Vec<u8> {
        self.boundaries
            .iter()
            .map(|boundary| boundary.tectonic_kind as u8)
            .collect()
    }

    pub fn geological_regimes(&self) -> Vec<u8> {
        self.boundaries
            .iter()
            .map(|boundary| boundary.geological_regime as u8)
            .collect()
    }

    pub fn subduction_polarities(&self) -> Vec<u8> {
        self.boundaries
            .iter()
            .map(|boundary| boundary.subduction_polarity as u8)
            .collect()
    }

    pub fn normal_rates_m_per_year(&self) -> Vec<f64> {
        self.boundaries
            .iter()
            .map(|boundary| boundary.normal_rate_m_per_year)
            .collect()
    }

    pub fn shear_rates_m_per_year(&self) -> Vec<f64> {
        self.boundaries
            .iter()
            .map(|boundary| boundary.shear_rate_m_per_year)
            .collect()
    }
}

fn ordered_pair(a: u16, b: u16) -> (u16, u16) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn normalize(value: [f64; 3]) -> [f64; 3] {
    let magnitude = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2])
        .sqrt()
        .max(1.0e-15);
    [
        value[0] / magnitude,
        value[1] / magnitude,
        value[2] / magnitude,
    ]
}

fn midpoint(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    normalize([a[0] + b[0], a[1] + b[1], a[2] + b[2]])
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0)
}

fn oriented_polarity(
    source: SubductionPolarity,
    source_plate_a: u16,
    source_plate_b: u16,
    fine_plate_a: u16,
    fine_plate_b: u16,
) -> Result<SubductionPolarity, WorldgenError> {
    if source_plate_a == fine_plate_a && source_plate_b == fine_plate_b {
        return Ok(source);
    }
    if source_plate_a == fine_plate_b && source_plate_b == fine_plate_a {
        return Ok(match source {
            SubductionPolarity::None => SubductionPolarity::None,
            SubductionPolarity::PlateA => SubductionPolarity::PlateB,
            SubductionPolarity::PlateB => SubductionPolarity::PlateA,
        });
    }
    Err(WorldgenError::InvalidRefinement(
        "fine boundary plate pair does not match coarse boundary provenance",
    ))
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub fn inherit_boundary_interfaces(
    coarse_topology: &GeodesicTopology,
    fine_topology: &GeodesicTopology,
    tectonics: &TectonicModel,
    geology: &CrustalModel,
    fine_plate_ids: &[u16],
) -> Result<InheritedBoundarySet, WorldgenError> {
    if fine_plate_ids.len() != fine_topology.metrics().sample_count as usize {
        return Err(WorldgenError::InvalidRefinement(
            "fine plate ownership does not match fine topology",
        ));
    }
    if tectonics.plate_ids.len() != coarse_topology.metrics().sample_count as usize {
        return Err(WorldgenError::InvalidRefinement(
            "tectonic ownership does not match coarse topology",
        ));
    }
    if tectonics.boundaries.len() != geology.boundaries.len() {
        return Err(WorldgenError::InvalidRefinement(
            "tectonic and geological boundary arrays are not aligned",
        ));
    }

    let mut candidates: BTreeMap<(u16, u16), Vec<(usize, [f64; 3])>> = BTreeMap::new();
    for (index, tectonic) in tectonics.boundaries.iter().enumerate() {
        let geological = &geology.boundaries[index];
        if tectonic.sample_a != geological.sample_a
            || tectonic.sample_b != geological.sample_b
            || tectonic.plate_a != geological.plate_a
            || tectonic.plate_b != geological.plate_b
        {
            return Err(WorldgenError::InvalidRefinement(
                "tectonic and geological boundary provenance is not edge-aligned",
            ));
        }
        let position = midpoint(
            coarse_topology.positions()[tectonic.sample_a as usize],
            coarse_topology.positions()[tectonic.sample_b as usize],
        );
        candidates
            .entry(ordered_pair(tectonic.plate_a, tectonic.plate_b))
            .or_default()
            .push((index, position));
    }

    let mut boundaries = Vec::new();
    for sample_a in 0..fine_topology.metrics().sample_count {
        for sample_b in fine_topology.neighbors_of(sample_a) {
            if sample_a >= *sample_b {
                continue;
            }
            let plate_a = fine_plate_ids[sample_a as usize];
            let plate_b = fine_plate_ids[*sample_b as usize];
            if plate_a == plate_b {
                continue;
            }
            let pair = ordered_pair(plate_a, plate_b);
            let pair_candidates = candidates
                .get(&pair)
                .ok_or(WorldgenError::InvalidRefinement(
                    "fine macro boundary has no compatible coarse boundary provenance",
                ))?;
            let fine_midpoint = midpoint(
                fine_topology.positions()[sample_a as usize],
                fine_topology.positions()[*sample_b as usize],
            );

            let mut best_index = usize::MAX;
            let mut best_similarity = f64::NEG_INFINITY;
            for (candidate_index, candidate_midpoint) in pair_candidates {
                let similarity = dot(fine_midpoint, *candidate_midpoint);
                if similarity > best_similarity + DISTANCE_TIE_EPSILON
                    || ((similarity - best_similarity).abs() <= DISTANCE_TIE_EPSILON
                        && *candidate_index < best_index)
                {
                    best_similarity = similarity;
                    best_index = *candidate_index;
                }
            }
            if best_index == usize::MAX {
                return Err(WorldgenError::InvalidRefinement(
                    "fine macro boundary provenance selection failed",
                ));
            }

            let tectonic = &tectonics.boundaries[best_index];
            let geological = &geology.boundaries[best_index];
            boundaries.push(InheritedBoundaryEdge {
                sample_a,
                sample_b: *sample_b,
                plate_a,
                plate_b,
                tectonic_kind: tectonic.kind,
                geological_regime: geological.regime,
                subduction_polarity: oriented_polarity(
                    geological.subduction_polarity,
                    tectonic.plate_a,
                    tectonic.plate_b,
                    plate_a,
                    plate_b,
                )?,
                normal_rate_m_per_year: tectonic.normal_rate_m_per_year,
                shear_rate_m_per_year: tectonic.shear_rate_m_per_year,
                coarse_boundary_index: best_index as u32,
            });
        }
    }

    let mut boundary_hash = FNV_OFFSET_BASIS;
    boundary_hash = fnv_update(boundary_hash, b"foundation:multires-boundaries:v1\0");
    for boundary in &boundaries {
        boundary_hash = fnv_update(boundary_hash, &boundary.sample_a.to_le_bytes());
        boundary_hash = fnv_update(boundary_hash, &boundary.sample_b.to_le_bytes());
        boundary_hash = fnv_update(boundary_hash, &boundary.plate_a.to_le_bytes());
        boundary_hash = fnv_update(boundary_hash, &boundary.plate_b.to_le_bytes());
        boundary_hash = fnv_update(boundary_hash, &[boundary.tectonic_kind as u8]);
        boundary_hash = fnv_update(boundary_hash, &[boundary.geological_regime as u8]);
        boundary_hash = fnv_update(boundary_hash, &[boundary.subduction_polarity as u8]);
        boundary_hash = fnv_update(
            boundary_hash,
            &boundary.normal_rate_m_per_year.to_bits().to_le_bytes(),
        );
        boundary_hash = fnv_update(
            boundary_hash,
            &boundary.shear_rate_m_per_year.to_bits().to_le_bytes(),
        );
        boundary_hash = fnv_update(boundary_hash, &boundary.coarse_boundary_index.to_le_bytes());
    }

    Ok(InheritedBoundarySet {
        boundaries,
        boundary_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_icosphere, build_refinement_map, generate_crust_and_history, generate_tectonics,
        refine_categorical_u16, GeologyRequest, PlanetPhysicalParameters, TectonicsRequest,
    };

    #[test]
    fn every_fine_macro_boundary_has_matching_coarse_provenance() {
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let coarse = build_icosphere(3).unwrap();
        let fine = build_icosphere(5).unwrap();
        let tectonics = generate_tectonics(
            &coarse,
            &TectonicsRequest::new("wg375-boundary", 12),
            parameters,
        )
        .unwrap();
        let geology = generate_crust_and_history(
            &coarse,
            &tectonics,
            &GeologyRequest::new("wg375-boundary"),
            parameters,
        )
        .unwrap();
        let map = build_refinement_map(&fine, 3).unwrap();
        let fine_plate_ids = refine_categorical_u16(&map, &tectonics.plate_ids).unwrap();
        let inherited =
            inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &fine_plate_ids)
                .unwrap();

        let expected_fine_edges = (0..fine.metrics().sample_count)
            .flat_map(|sample_a| {
                fine.neighbors_of(sample_a)
                    .iter()
                    .copied()
                    .filter(move |sample_b| sample_a < *sample_b)
                    .map(move |sample_b| (sample_a, sample_b))
            })
            .filter(|(a, b)| fine_plate_ids[*a as usize] != fine_plate_ids[*b as usize])
            .count();
        assert_eq!(inherited.boundaries.len(), expected_fine_edges);

        for boundary in &inherited.boundaries {
            let source = &tectonics.boundaries[boundary.coarse_boundary_index as usize];
            assert_eq!(
                ordered_pair(boundary.plate_a, boundary.plate_b),
                ordered_pair(source.plate_a, source.plate_b)
            );
            assert_eq!(boundary.tectonic_kind, source.kind);
            assert_eq!(
                boundary.normal_rate_m_per_year,
                source.normal_rate_m_per_year
            );
            assert_eq!(boundary.shear_rate_m_per_year, source.shear_rate_m_per_year);
        }
    }

    #[test]
    fn boundary_inheritance_is_deterministic() {
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let coarse = build_icosphere(3).unwrap();
        let fine = build_icosphere(4).unwrap();
        let tectonics = generate_tectonics(
            &coarse,
            &TectonicsRequest::new("wg375-boundary-determinism", 10),
            parameters,
        )
        .unwrap();
        let geology = generate_crust_and_history(
            &coarse,
            &tectonics,
            &GeologyRequest::new("wg375-boundary-determinism"),
            parameters,
        )
        .unwrap();
        let map = build_refinement_map(&fine, 3).unwrap();
        let fine_plate_ids = refine_categorical_u16(&map, &tectonics.plate_ids).unwrap();
        let a = inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &fine_plate_ids)
            .unwrap();
        let b = inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &fine_plate_ids)
            .unwrap();
        assert_eq!(a, b);
        assert_ne!(a.boundaries.len(), 0);
    }
}
