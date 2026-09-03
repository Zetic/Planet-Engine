use crate::{
    expected_sample_count, CrustalModel, GeodesicTopology, LithosphericModel,
    PlanetPhysicalParameters, TectonicModel, WorldgenError,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub const MULTIRES_STAGE_ID: &str = "foundation:multires-inheritance";
pub const MULTIRES_STAGE_VERSION: u32 = 1;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const DISTANCE_EPSILON: f64 = 1.0e-15;

#[derive(Clone, Debug, PartialEq)]
pub struct RefinementMetrics {
    pub coarse_level: u8,
    pub fine_level: u8,
    pub coarse_sample_count: u32,
    pub fine_sample_count: u32,
    pub added_sample_count: u32,
    pub provenance_hash: u64,
}

impl RefinementMetrics {
    pub fn provenance_hash_hex(&self) -> String {
        format!("{:016x}", self.provenance_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefinementMap {
    pub metrics: RefinementMetrics,
    pub nearest_coarse_source: Vec<u32>,
    pub inherited_sample_mask: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
struct QueueEntry {
    distance: f64,
    source: u32,
    sample: u32,
}

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.distance.to_bits() == other.distance.to_bits()
            && self.source == other.source
            && self.sample == other.sample
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
            .distance
            .total_cmp(&self.distance)
            .then_with(|| other.source.cmp(&self.source))
            .then_with(|| other.sample.cmp(&self.sample))
    }
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn validate_levels(
    fine_topology: &GeodesicTopology,
    coarse_level: u8,
) -> Result<u32, WorldgenError> {
    if coarse_level > fine_topology.level() {
        return Err(WorldgenError::InvalidRefinement(
            "coarse level cannot exceed fine topology level",
        ));
    }
    let coarse_count = expected_sample_count(coarse_level).ok_or(
        WorldgenError::InvalidRefinement("coarse topology level is unsupported"),
    )?;
    if coarse_count > fine_topology.metrics().sample_count {
        return Err(WorldgenError::InvalidRefinement(
            "coarse sample count exceeds fine topology",
        ));
    }
    for sample in 0..coarse_count as usize {
        if fine_topology.birth_levels()[sample] > coarse_level {
            return Err(WorldgenError::InvalidRefinement(
                "stable inherited sample prefix is not preserved",
            ));
        }
    }
    Ok(coarse_count)
}

pub fn build_refinement_map(
    fine_topology: &GeodesicTopology,
    coarse_level: u8,
) -> Result<RefinementMap, WorldgenError> {
    let coarse_count = validate_levels(fine_topology, coarse_level)?;
    let fine_count = fine_topology.metrics().sample_count;
    let mut distances = vec![f64::INFINITY; fine_count as usize];
    let mut sources = vec![u32::MAX; fine_count as usize];
    let mut queue = BinaryHeap::new();

    for source in 0..coarse_count {
        distances[source as usize] = 0.0;
        sources[source as usize] = source;
        queue.push(QueueEntry {
            distance: 0.0,
            source,
            sample: source,
        });
    }

    while let Some(entry) = queue.pop() {
        let index = entry.sample as usize;
        let known_distance = distances[index];
        let known_source = sources[index];
        if entry.distance > known_distance + DISTANCE_EPSILON
            || ((entry.distance - known_distance).abs() <= DISTANCE_EPSILON
                && entry.source != known_source)
        {
            continue;
        }
        let neighbors = fine_topology.neighbors_of(entry.sample);
        let lengths = fine_topology.neighbor_arc_lengths_of(entry.sample);
        for (neighbor, length) in neighbors.iter().zip(lengths.iter()) {
            let candidate_distance = entry.distance + *length;
            let neighbor_index = *neighbor as usize;
            let improves_distance =
                candidate_distance + DISTANCE_EPSILON < distances[neighbor_index];
            let ties_with_better_source = (candidate_distance - distances[neighbor_index]).abs()
                <= DISTANCE_EPSILON
                && entry.source < sources[neighbor_index];
            if improves_distance || ties_with_better_source {
                distances[neighbor_index] = candidate_distance;
                sources[neighbor_index] = entry.source;
                queue.push(QueueEntry {
                    distance: candidate_distance,
                    source: entry.source,
                    sample: *neighbor,
                });
            }
        }
    }

    if sources.iter().any(|source| *source == u32::MAX) {
        return Err(WorldgenError::InvalidRefinement(
            "refinement provenance does not cover the fine topology",
        ));
    }

    let inherited_sample_mask = (0..fine_count)
        .map(|sample| u8::from(sample < coarse_count))
        .collect::<Vec<_>>();
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, &[coarse_level, fine_topology.level()]);
    hash = fnv_update(hash, &coarse_count.to_le_bytes());
    hash = fnv_update(hash, &fine_count.to_le_bytes());
    for source in &sources {
        hash = fnv_update(hash, &source.to_le_bytes());
    }

    Ok(RefinementMap {
        metrics: RefinementMetrics {
            coarse_level,
            fine_level: fine_topology.level(),
            coarse_sample_count: coarse_count,
            fine_sample_count: fine_count,
            added_sample_count: fine_count - coarse_count,
            provenance_hash: hash,
        },
        nearest_coarse_source: sources,
        inherited_sample_mask,
    })
}

fn validate_scalar_len<T>(
    fine_topology: &GeodesicTopology,
    coarse_level: u8,
    coarse_values: &[T],
) -> Result<u32, WorldgenError> {
    let coarse_count = validate_levels(fine_topology, coarse_level)?;
    if coarse_values.len() != coarse_count as usize {
        return Err(WorldgenError::InvalidRefinement(
            "coarse field length does not match coarse topology",
        ));
    }
    Ok(coarse_count)
}

pub fn refine_scalar_f32(
    fine_topology: &GeodesicTopology,
    coarse_level: u8,
    coarse_values: &[f32],
) -> Result<Vec<f32>, WorldgenError> {
    let coarse_count = validate_scalar_len(fine_topology, coarse_level, coarse_values)?;
    if coarse_values.iter().any(|value| !value.is_finite()) {
        return Err(WorldgenError::InvalidRefinement(
            "coarse scalar field contains non-finite values",
        ));
    }
    let mut output = vec![0.0_f32; fine_topology.metrics().sample_count as usize];
    output[..coarse_count as usize].copy_from_slice(coarse_values);
    for sample in coarse_count as usize..output.len() {
        let [a, b] = fine_topology.parent_edges()[sample];
        if a == u32::MAX || b == u32::MAX || a as usize >= sample || b as usize >= sample {
            return Err(WorldgenError::InvalidRefinement(
                "fine sample has invalid hierarchical parents",
            ));
        }
        output[sample] = (output[a as usize] + output[b as usize]) * 0.5;
    }
    Ok(output)
}

pub fn refine_scalar_f64(
    fine_topology: &GeodesicTopology,
    coarse_level: u8,
    coarse_values: &[f64],
) -> Result<Vec<f64>, WorldgenError> {
    let coarse_count = validate_scalar_len(fine_topology, coarse_level, coarse_values)?;
    if coarse_values.iter().any(|value| !value.is_finite()) {
        return Err(WorldgenError::InvalidRefinement(
            "coarse scalar field contains non-finite values",
        ));
    }
    let mut output = vec![0.0_f64; fine_topology.metrics().sample_count as usize];
    output[..coarse_count as usize].copy_from_slice(coarse_values);
    for sample in coarse_count as usize..output.len() {
        let [a, b] = fine_topology.parent_edges()[sample];
        if a == u32::MAX || b == u32::MAX || a as usize >= sample || b as usize >= sample {
            return Err(WorldgenError::InvalidRefinement(
                "fine sample has invalid hierarchical parents",
            ));
        }
        output[sample] = (output[a as usize] + output[b as usize]) * 0.5;
    }
    Ok(output)
}

pub fn refine_vector3_f64(
    fine_topology: &GeodesicTopology,
    coarse_level: u8,
    coarse_values: &[[f64; 3]],
) -> Result<Vec<[f64; 3]>, WorldgenError> {
    let coarse_count = validate_scalar_len(fine_topology, coarse_level, coarse_values)?;
    if coarse_values
        .iter()
        .flatten()
        .any(|value| !value.is_finite())
    {
        return Err(WorldgenError::InvalidRefinement(
            "coarse vector field contains non-finite values",
        ));
    }
    let mut output = vec![[0.0_f64; 3]; fine_topology.metrics().sample_count as usize];
    output[..coarse_count as usize].copy_from_slice(coarse_values);
    for sample in coarse_count as usize..output.len() {
        let [a, b] = fine_topology.parent_edges()[sample];
        if a == u32::MAX || b == u32::MAX || a as usize >= sample || b as usize >= sample {
            return Err(WorldgenError::InvalidRefinement(
                "fine sample has invalid hierarchical parents",
            ));
        }
        for component in 0..3 {
            output[sample][component] =
                (output[a as usize][component] + output[b as usize][component]) * 0.5;
        }
    }
    Ok(output)
}

pub fn refine_categorical_u16(
    map: &RefinementMap,
    coarse_values: &[u16],
) -> Result<Vec<u16>, WorldgenError> {
    if coarse_values.len() != map.metrics.coarse_sample_count as usize {
        return Err(WorldgenError::InvalidRefinement(
            "coarse categorical field length does not match refinement map",
        ));
    }
    Ok(map
        .nearest_coarse_source
        .iter()
        .map(|source| coarse_values[*source as usize])
        .collect())
}

pub fn refine_categorical_u8(
    map: &RefinementMap,
    coarse_values: &[u8],
) -> Result<Vec<u8>, WorldgenError> {
    if coarse_values.len() != map.metrics.coarse_sample_count as usize {
        return Err(WorldgenError::InvalidRefinement(
            "coarse categorical field length does not match refinement map",
        ));
    }
    Ok(map
        .nearest_coarse_source
        .iter()
        .map(|source| coarse_values[*source as usize])
        .collect())
}

pub fn refine_scalar_f32_with_domains(
    fine_topology: &GeodesicTopology,
    coarse_level: u8,
    coarse_values: &[f32],
    map: &RefinementMap,
    coarse_domains: &[u16],
) -> Result<Vec<f32>, WorldgenError> {
    let coarse_count = validate_scalar_len(fine_topology, coarse_level, coarse_values)?;
    if map.metrics.coarse_level != coarse_level || map.metrics.fine_level != fine_topology.level() {
        return Err(WorldgenError::InvalidRefinement(
            "refinement map does not match scalar refinement levels",
        ));
    }
    if coarse_domains.len() != coarse_count as usize {
        return Err(WorldgenError::InvalidRefinement(
            "coarse domain field length does not match coarse topology",
        ));
    }
    if coarse_values.iter().any(|value| !value.is_finite()) {
        return Err(WorldgenError::InvalidRefinement(
            "coarse scalar field contains non-finite values",
        ));
    }
    let domains = refine_categorical_u16(map, coarse_domains)?;
    let mut output = vec![0.0_f32; fine_topology.metrics().sample_count as usize];
    output[..coarse_count as usize].copy_from_slice(coarse_values);
    for sample in coarse_count as usize..output.len() {
        let [a, b] = fine_topology.parent_edges()[sample];
        if a == u32::MAX || b == u32::MAX || a as usize >= sample || b as usize >= sample {
            return Err(WorldgenError::InvalidRefinement(
                "fine sample has invalid hierarchical parents",
            ));
        }
        let target_domain = domains[sample];
        let a_matches = domains[a as usize] == target_domain;
        let b_matches = domains[b as usize] == target_domain;
        output[sample] = match (a_matches, b_matches) {
            (true, true) => (output[a as usize] + output[b as usize]) * 0.5,
            (true, false) => output[a as usize],
            (false, true) => output[b as usize],
            (false, false) => coarse_values[map.nearest_coarse_source[sample] as usize],
        };
    }
    Ok(output)
}

#[derive(Clone, Debug, PartialEq)]
pub struct InheritedPhysicalState {
    pub map: RefinementMap,
    pub parameter_hash: u64,
    pub inheritance_hash: u64,
    pub plate_ids: Vec<u16>,
    pub crust_kind: Vec<u8>,
    pub crust_province_id: Vec<u16>,
    pub crust_age_myr: Vec<f32>,
    pub crust_thickness_km: Vec<f32>,
    pub crust_density_kg_per_m3: Vec<f32>,
    pub buoyancy_index: Vec<f32>,
    pub orogenic_history: Vec<f32>,
    pub rift_history: Vec<f32>,
    pub ridge_history: Vec<f32>,
    pub subduction_history: Vec<f32>,
    pub trench_history: Vec<f32>,
    pub volcanic_arc_history: Vec<f32>,
    pub transform_history: Vec<f32>,
    pub subsidence_history: Vec<f32>,
    pub basin_potential: Vec<f32>,
    pub crustal_strain: Vec<f32>,
    pub strength_index: Vec<f32>,
    pub weakness_index: Vec<f32>,
    pub effective_elastic_thickness_km: Vec<f32>,
    pub thermal_anomaly_index: Vec<f32>,
    pub mantle_upwelling_index: Vec<f32>,
    pub mantle_dynamic_support_index: Vec<f32>,
    pub compensated_buoyancy_index: Vec<f32>,
    pub structural_fabric_strength: Vec<f32>,
    pub structural_zone_kind: Vec<u8>,
    pub fragmentation_propensity: Vec<f32>,
    pub fragment_ids: Vec<u16>,
    pub kinematic_domain_ids: Vec<u16>,
}

impl InheritedPhysicalState {
    pub fn inheritance_hash_hex(&self) -> String {
        format!("{:016x}", self.inheritance_hash)
    }
    pub fn parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.parameter_hash)
    }
}

fn validate_upstream_lengths(
    coarse_count: usize,
    tectonics: &TectonicModel,
    geology: &CrustalModel,
    lithosphere: &LithosphericModel,
) -> Result<(), WorldgenError> {
    let geology_lengths = [
        geology.crust_kind.len(),
        geology.crust_province_id.len(),
        geology.crust_age_myr.len(),
        geology.crust_thickness_km.len(),
        geology.crust_density_kg_per_m3.len(),
        geology.buoyancy_index.len(),
        geology.orogenic_history.len(),
        geology.rift_history.len(),
        geology.ridge_history.len(),
        geology.subduction_history.len(),
        geology.trench_history.len(),
        geology.volcanic_arc_history.len(),
        geology.transform_history.len(),
        geology.subsidence_history.len(),
        geology.basin_potential.len(),
        geology.crustal_strain.len(),
    ];
    let lithosphere_lengths = [
        lithosphere.strength_index.len(),
        lithosphere.weakness_index.len(),
        lithosphere.effective_elastic_thickness_km.len(),
        lithosphere.thermal_anomaly_index.len(),
        lithosphere.mantle_upwelling_index.len(),
        lithosphere.mantle_dynamic_support_index.len(),
        lithosphere.compensated_buoyancy_index.len(),
        lithosphere.structural_fabric_strength.len(),
        lithosphere.structural_zone_kind.len(),
        lithosphere.fragmentation_propensity.len(),
        lithosphere.fragment_ids.len(),
        lithosphere.kinematic_domain_ids.len(),
    ];
    if tectonics.plate_ids.len() != coarse_count
        || geology_lengths.iter().any(|length| *length != coarse_count)
        || lithosphere_lengths
            .iter()
            .any(|length| *length != coarse_count)
    {
        return Err(WorldgenError::InvalidRefinement(
            "upstream physical state does not match the declared coarse topology",
        ));
    }
    Ok(())
}

pub fn inherit_physical_state(
    fine_topology: &GeodesicTopology,
    coarse_level: u8,
    tectonics: &TectonicModel,
    geology: &CrustalModel,
    lithosphere: &LithosphericModel,
    parameters: PlanetPhysicalParameters,
) -> Result<InheritedPhysicalState, WorldgenError> {
    parameters
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    let coarse_count = validate_levels(fine_topology, coarse_level)? as usize;
    validate_upstream_lengths(coarse_count, tectonics, geology, lithosphere)?;
    let map = build_refinement_map(fine_topology, coarse_level)?;

    let plate_ids = refine_categorical_u16(&map, &tectonics.plate_ids)?;
    let crust_kind = refine_categorical_u8(&map, &geology.crust_kind)?;
    let crust_province_id = refine_categorical_u16(&map, &geology.crust_province_id)?;
    let fragment_ids = refine_categorical_u16(&map, &lithosphere.fragment_ids)?;
    let kinematic_domain_ids = refine_categorical_u16(&map, &lithosphere.kinematic_domain_ids)?;
    let structural_zone_kind = refine_categorical_u8(&map, &lithosphere.structural_zone_kind)?;

    let crust_age_myr = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &geology.crust_age_myr,
        &map,
        &geology.crust_province_id,
    )?;
    let crust_thickness_km = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &geology.crust_thickness_km,
        &map,
        &geology.crust_province_id,
    )?;
    let crust_density_kg_per_m3 = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &geology.crust_density_kg_per_m3,
        &map,
        &geology.crust_province_id,
    )?;
    let buoyancy_index = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &geology.buoyancy_index,
        &map,
        &geology.crust_province_id,
    )?;

    let orogenic_history =
        refine_scalar_f32(fine_topology, coarse_level, &geology.orogenic_history)?;
    let rift_history = refine_scalar_f32(fine_topology, coarse_level, &geology.rift_history)?;
    let ridge_history = refine_scalar_f32(fine_topology, coarse_level, &geology.ridge_history)?;
    let subduction_history =
        refine_scalar_f32(fine_topology, coarse_level, &geology.subduction_history)?;
    let trench_history = refine_scalar_f32(fine_topology, coarse_level, &geology.trench_history)?;
    let volcanic_arc_history =
        refine_scalar_f32(fine_topology, coarse_level, &geology.volcanic_arc_history)?;
    let transform_history =
        refine_scalar_f32(fine_topology, coarse_level, &geology.transform_history)?;
    let subsidence_history =
        refine_scalar_f32(fine_topology, coarse_level, &geology.subsidence_history)?;
    let basin_potential = refine_scalar_f32(fine_topology, coarse_level, &geology.basin_potential)?;
    let crustal_strain = refine_scalar_f32(fine_topology, coarse_level, &geology.crustal_strain)?;

    let strength_index = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &lithosphere.strength_index,
        &map,
        &lithosphere.kinematic_domain_ids,
    )?;
    let weakness_index = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &lithosphere.weakness_index,
        &map,
        &lithosphere.kinematic_domain_ids,
    )?;
    let effective_elastic_thickness_km = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &lithosphere.effective_elastic_thickness_km,
        &map,
        &lithosphere.kinematic_domain_ids,
    )?;
    let structural_fabric_strength = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &lithosphere.structural_fabric_strength,
        &map,
        &lithosphere.kinematic_domain_ids,
    )?;
    let fragmentation_propensity = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &lithosphere.fragmentation_propensity,
        &map,
        &lithosphere.kinematic_domain_ids,
    )?;

    let thermal_anomaly_index = refine_scalar_f32(
        fine_topology,
        coarse_level,
        &lithosphere.thermal_anomaly_index,
    )?;
    let mantle_upwelling_index = refine_scalar_f32(
        fine_topology,
        coarse_level,
        &lithosphere.mantle_upwelling_index,
    )?;
    let mantle_dynamic_support_index = refine_scalar_f32(
        fine_topology,
        coarse_level,
        &lithosphere.mantle_dynamic_support_index,
    )?;
    let compensated_buoyancy_index = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &lithosphere.compensated_buoyancy_index,
        &map,
        &geology.crust_province_id,
    )?;

    let parameter_hash = parameters.parameter_hash();
    let mut inheritance_hash = FNV_OFFSET_BASIS;
    inheritance_hash = fnv_update(inheritance_hash, MULTIRES_STAGE_ID.as_bytes());
    inheritance_hash = fnv_update(inheritance_hash, &MULTIRES_STAGE_VERSION.to_le_bytes());
    inheritance_hash = fnv_update(inheritance_hash, &map.metrics.provenance_hash.to_le_bytes());
    inheritance_hash = fnv_update(inheritance_hash, &parameter_hash.to_le_bytes());
    inheritance_hash = fnv_update(
        inheritance_hash,
        &tectonics.metrics.tectonic_hash.to_le_bytes(),
    );
    inheritance_hash = fnv_update(
        inheritance_hash,
        &geology.metrics.geology_hash.to_le_bytes(),
    );
    inheritance_hash = fnv_update(
        inheritance_hash,
        &lithosphere.metrics.lithosphere_hash.to_le_bytes(),
    );

    Ok(InheritedPhysicalState {
        map,
        parameter_hash,
        inheritance_hash,
        plate_ids,
        crust_kind,
        crust_province_id,
        crust_age_myr,
        crust_thickness_km,
        crust_density_kg_per_m3,
        buoyancy_index,
        orogenic_history,
        rift_history,
        ridge_history,
        subduction_history,
        trench_history,
        volcanic_arc_history,
        transform_history,
        subsidence_history,
        basin_potential,
        crustal_strain,
        strength_index,
        weakness_index,
        effective_elastic_thickness_km,
        thermal_anomaly_index,
        mantle_upwelling_index,
        mantle_dynamic_support_index,
        compensated_buoyancy_index,
        structural_fabric_strength,
        structural_zone_kind,
        fragmentation_propensity,
        fragment_ids,
        kinematic_domain_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_icosphere;

    #[test]
    fn scalar_refinement_preserves_every_coarse_sample_exactly() {
        let fine = build_icosphere(5).unwrap();
        let coarse_count = expected_sample_count(3).unwrap() as usize;
        let coarse = (0..coarse_count)
            .map(|sample| sample as f64 * 0.125 - 7.0)
            .collect::<Vec<_>>();
        let refined = refine_scalar_f64(&fine, 3, &coarse).unwrap();
        assert_eq!(&refined[..coarse_count], coarse.as_slice());
        assert_eq!(refined.len(), expected_sample_count(5).unwrap() as usize);
    }

    #[test]
    fn direct_and_staged_scalar_refinement_are_identical() {
        let level4 = build_icosphere(4).unwrap();
        let level6 = build_icosphere(6).unwrap();
        let coarse_count = expected_sample_count(3).unwrap() as usize;
        let coarse = (0..coarse_count)
            .map(|sample| ((sample * 17) % 101) as f32 / 100.0)
            .collect::<Vec<_>>();
        let level4_values = refine_scalar_f32(&level4, 3, &coarse).unwrap();
        let staged = refine_scalar_f32(&level6, 4, &level4_values).unwrap();
        let direct = refine_scalar_f32(&level6, 3, &coarse).unwrap();
        assert_eq!(direct, staged);
    }

    #[test]
    fn categorical_refinement_is_deterministic_complete_and_preserves_coarse_truth() {
        let fine = build_icosphere(5).unwrap();
        let coarse_count = expected_sample_count(3).unwrap() as usize;
        let coarse = (0..coarse_count)
            .map(|sample| (sample % 13) as u16)
            .collect::<Vec<_>>();
        let a = build_refinement_map(&fine, 3).unwrap();
        let b = build_refinement_map(&fine, 3).unwrap();
        assert_eq!(a, b);
        let refined = refine_categorical_u16(&a, &coarse).unwrap();
        assert_eq!(&refined[..coarse_count], coarse.as_slice());
        assert!(a
            .nearest_coarse_source
            .iter()
            .all(|source| (*source as usize) < coarse_count));
        assert_eq!(
            a.inherited_sample_mask
                .iter()
                .map(|value| u32::from(*value))
                .sum::<u32>(),
            coarse_count as u32
        );
    }

    #[test]
    fn backwards_refinement_is_rejected() {
        let level3 = build_icosphere(3).unwrap();
        assert!(build_refinement_map(&level3, 4).is_err());
    }
}

#[cfg(test)]
mod physical_inheritance_tests {
    use super::*;
    use crate::{
        build_icosphere, generate_crust_and_history, generate_lithosphere, generate_tectonics,
        GeologyRequest, LithosphereRequest, TectonicsRequest,
    };

    fn coarse_world(
        level: u8,
    ) -> (
        GeodesicTopology,
        TectonicModel,
        CrustalModel,
        LithosphericModel,
        PlanetPhysicalParameters,
    ) {
        let topology = build_icosphere(level).unwrap();
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let tectonics = generate_tectonics(
            &topology,
            &TectonicsRequest::new("wg375-inheritance", 14),
            parameters,
        )
        .unwrap();
        let geology = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new("wg375-inheritance"),
            parameters,
        )
        .unwrap();
        let lithosphere = generate_lithosphere(
            &topology,
            &tectonics,
            &geology,
            &LithosphereRequest::new("wg375-inheritance"),
        )
        .unwrap();
        (topology, tectonics, geology, lithosphere, parameters)
    }

    #[test]
    fn accepted_wg2_wg3_wg35_truth_is_exact_on_inherited_samples() {
        let (_coarse, tectonics, geology, lithosphere, parameters) = coarse_world(3);
        let fine = build_icosphere(5).unwrap();
        let state =
            inherit_physical_state(&fine, 3, &tectonics, &geology, &lithosphere, parameters)
                .unwrap();
        let count = expected_sample_count(3).unwrap() as usize;
        assert_eq!(&state.plate_ids[..count], tectonics.plate_ids.as_slice());
        assert_eq!(&state.crust_kind[..count], geology.crust_kind.as_slice());
        assert_eq!(
            &state.crust_age_myr[..count],
            geology.crust_age_myr.as_slice()
        );
        assert_eq!(
            &state.strength_index[..count],
            lithosphere.strength_index.as_slice()
        );
        assert_eq!(
            &state.kinematic_domain_ids[..count],
            lithosphere.kinematic_domain_ids.as_slice()
        );
    }

    #[test]
    fn physical_inheritance_is_deterministic_and_profile_identity_is_explicit() {
        let (_coarse, tectonics, geology, lithosphere, parameters) = coarse_world(3);
        let fine = build_icosphere(5).unwrap();
        let a = inherit_physical_state(&fine, 3, &tectonics, &geology, &lithosphere, parameters)
            .unwrap();
        let b = inherit_physical_state(&fine, 3, &tectonics, &geology, &lithosphere, parameters)
            .unwrap();
        assert_eq!(a, b);
        let mut dry = parameters;
        dry.surface_water_mass_kg = 0.0;
        let dry_state =
            inherit_physical_state(&fine, 3, &tectonics, &geology, &lithosphere, dry).unwrap();
        assert_ne!(a.parameter_hash, dry_state.parameter_hash);
        assert_ne!(a.inheritance_hash, dry_state.inheritance_hash);
        assert_eq!(a.plate_ids, dry_state.plate_ids);
    }

    #[test]
    fn domain_constrained_interpolation_does_not_average_across_a_categorical_boundary() {
        let fine = build_icosphere(2).unwrap();
        let coarse_count = expected_sample_count(1).unwrap() as usize;
        let map = build_refinement_map(&fine, 1).unwrap();
        let mut domains = vec![0_u16; coarse_count];
        let mut values = vec![10.0_f32; coarse_count];
        domains[1] = 1;
        values[1] = 100.0;
        let refined = refine_scalar_f32_with_domains(&fine, 1, &values, &map, &domains).unwrap();
        for sample in coarse_count..refined.len() {
            assert!(refined[sample] >= 10.0 && refined[sample] <= 100.0);
        }
        assert_eq!(&refined[..coarse_count], values.as_slice());
    }
}
