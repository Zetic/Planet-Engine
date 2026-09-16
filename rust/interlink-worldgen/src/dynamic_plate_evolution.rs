use crate::{
    derive_stage_seed, CrustFragment, HistoricalEventKind, HistoricalLithosphereModel,
    HistoricalTectonicEvent, PlanetPhysicalParameters, PlanetTopology, WorldgenError,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const DYNAMIC_PLATE_NAMESPACE: &str = "worldgen:geology:dynamic-modern-plates:v1";
const DYNAMIC_EPOCHS: usize = 18;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn unit_random(value: u64) -> f64 {
    ((mix64(value) >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f64; 3], factor: f64) -> [f64; 3] {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn normalize_or(a: [f64; 3], fallback: [f64; 3]) -> [f64; 3] {
    let magnitude = norm(a);
    if magnitude > 1.0e-15 {
        scale(a, 1.0 / magnitude)
    } else {
        fallback
    }
}

fn random_unit_vector(seed: u64, stream: u64) -> [f64; 3] {
    let z = unit_random(seed ^ stream ^ 0x65b5_61f5_1b6b_42d1) * 2.0 - 1.0;
    let theta = unit_random(seed ^ stream ^ 0xa076_1d64_78bd_642f) * std::f64::consts::TAU;
    let radial = (1.0 - z * z).max(0.0).sqrt();
    [radial * theta.cos(), radial * theta.sin(), z]
}

fn plate_angular_velocities<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    owners: &[u16],
) -> Vec<[f64; 3]> {
    let plate_count = model.metrics.modern_plate_count as usize;
    let mut sums = vec![[0.0_f64; 3]; plate_count];
    let mut areas = vec![0.0_f64; plate_count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let owner = owners[index] as usize;
        let origin = model.origin_plate_ids[index] as usize;
        let area = topology.area_steradians(sample);
        let velocity = model.ancestral_tectonics.plates[origin].angular_velocity_rad_per_myr;
        for axis in 0..3 {
            sums[owner][axis] += velocity[axis] * area;
        }
        areas[owner] += area;
    }
    for plate in 0..plate_count {
        if areas[plate] > 0.0 {
            sums[plate] = scale(sums[plate], 1.0 / areas[plate]);
        }
    }
    sums
}

fn choose_anchors<T: PlanetTopology>(
    topology: &T,
    owners: &[u16],
    plate_count: usize,
    seed: u64,
) -> Result<Vec<u32>, WorldgenError> {
    let mut anchors = vec![u32::MAX; plate_count];
    let mut best_score = vec![f64::NEG_INFINITY; plate_count];
    for sample in 0..topology.sample_count() {
        let plate = owners[sample as usize] as usize;
        if plate >= plate_count {
            return Err(WorldgenError::InvalidTectonics(
                "dynamic modern plate evolution received an invalid owner id",
            ));
        }
        let neighbors = topology.neighbors(sample);
        let same = neighbors
            .iter()
            .filter(|neighbor| owners[**neighbor as usize] as usize == plate)
            .count() as f64;
        let score = same
            + unit_random(seed ^ u64::from(sample).wrapping_mul(0x9e37_79b9_7f4a_7c15)) * 0.01;
        if score > best_score[plate] {
            best_score[plate] = score;
            anchors[plate] = sample;
        }
    }
    if anchors.iter().any(|sample| *sample == u32::MAX) {
        return Err(WorldgenError::InvalidTectonics(
            "dynamic modern plate evolution received an empty plate",
        ));
    }
    Ok(anchors)
}

fn candidate_score<T: PlanetTopology>(
    topology: &T,
    sample: u32,
    candidate: u16,
    current: u16,
    owners: &[u16],
    initial_owners: &[u16],
    angular_velocity: &[[f64; 3]],
    shape_axes: &[[[f64; 3]; 3]],
    epoch: usize,
) -> f64 {
    let position = topology.unit_position(sample);
    let neighbors = topology.neighbors(sample);
    let immediate = neighbors
        .iter()
        .filter(|neighbor| owners[**neighbor as usize] == candidate)
        .count() as f64
        / neighbors.len().max(1) as f64;

    let mut second_total = 0usize;
    let mut second_same = 0usize;
    let mut pressure = 0.0_f64;
    let mut pressure_count = 0usize;
    let velocity = cross(angular_velocity[candidate as usize], position);
    let speed = norm(velocity);
    let velocity_dir = if speed > 1.0e-15 {
        scale(velocity, 1.0 / speed)
    } else {
        [0.0, 0.0, 0.0]
    };

    for neighbor in neighbors {
        for second in topology.neighbors(*neighbor) {
            if *second == sample {
                continue;
            }
            second_total += 1;
            if owners[*second as usize] == candidate {
                second_same += 1;
            }
        }
        if owners[*neighbor as usize] == candidate && speed > 1.0e-15 {
            let direction = normalize_or(
                sub(position, topology.unit_position(*neighbor)),
                velocity_dir,
            );
            pressure += dot(velocity_dir, direction);
            pressure_count += 1;
        }
    }
    let second_ring = if second_total > 0 {
        second_same as f64 / second_total as f64
    } else {
        0.0
    };
    let kinematic_pressure = if pressure_count > 0 {
        pressure / pressure_count as f64
    } else {
        0.0
    };

    let axes = shape_axes[candidate as usize];
    let low_frequency_shape = dot(position, axes[0]) * 0.58
        + (dot(position, axes[1]).powi(3)) * 0.26
        + dot(position, axes[2]) * (0.08 + epoch as f64 / DYNAMIC_EPOCHS as f64 * 0.08);
    let inertia = if candidate == current { 0.10 } else { 0.0 };
    let ancestry_affinity = if initial_owners[sample as usize] == candidate {
        0.055
    } else {
        0.0
    };

    immediate * 0.76
        + second_ring * 0.24
        + kinematic_pressure * 0.24
        + low_frequency_shape * 0.24
        + inertia
        + ancestry_affinity
}

fn repair_connectivity<T: PlanetTopology>(
    topology: &T,
    owners: &mut [u16],
    anchors: &[u32],
    plate_count: usize,
) {
    for _ in 0..4 {
        let mut changed = false;
        for plate in 0..plate_count {
            let anchor = anchors[plate];
            if owners[anchor as usize] != plate as u16 {
                owners[anchor as usize] = plate as u16;
                changed = true;
            }
            let mut connected = vec![false; owners.len()];
            let mut queue = VecDeque::from([anchor]);
            connected[anchor as usize] = true;
            while let Some(sample) = queue.pop_front() {
                for neighbor in topology.neighbors(sample) {
                    let index = *neighbor as usize;
                    if !connected[index] && owners[index] == plate as u16 {
                        connected[index] = true;
                        queue.push_back(*neighbor);
                    }
                }
            }

            for sample in 0..topology.sample_count() {
                let index = sample as usize;
                if owners[index] != plate as u16 || connected[index] || sample == anchor {
                    continue;
                }
                let mut candidates = BTreeMap::<u16, usize>::new();
                for neighbor in topology.neighbors(sample) {
                    let owner = owners[*neighbor as usize];
                    if owner != plate as u16 {
                        *candidates.entry(owner).or_insert(0) += 1;
                    }
                }
                if let Some((&replacement, _)) = candidates
                    .iter()
                    .max_by_key(|(owner, count)| (**count, std::cmp::Reverse(**owner)))
                {
                    owners[index] = replacement;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
}

fn evolve_ownership<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    seed: u64,
) -> Result<Vec<u16>, WorldgenError> {
    let plate_count = model.metrics.modern_plate_count as usize;
    let initial = model.current_plate_ids.clone();
    let mut owners = initial.clone();
    let angular_velocity = plate_angular_velocities(topology, model, &initial);
    let anchors = choose_anchors(topology, &initial, plate_count, seed)?;
    let shape_axes = (0..plate_count)
        .map(|plate| {
            let stream = (plate as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
            [
                random_unit_vector(seed, stream ^ 0xa076_1d64_78bd_642f),
                random_unit_vector(seed, stream ^ 0xe703_7ed1_a0b4_28db),
                random_unit_vector(seed, stream ^ 0x8ebc_6af0_9c88_c6e3),
            ]
        })
        .collect::<Vec<_>>();

    let sample_count = topology.sample_count() as usize;
    let minimum_plate_samples = (sample_count / plate_count.max(1) / 10).max(6);
    let maximum_plate_samples = ((sample_count as f64) * 0.28).ceil() as usize;

    for epoch in 0..DYNAMIC_EPOCHS {
        let mut plate_sizes = vec![0usize; plate_count];
        for owner in &owners {
            plate_sizes[*owner as usize] += 1;
        }
        let mut proposals = Vec::<(f64, u32, u16, u16)>::new();
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            let current = owners[index];
            if anchors[current as usize] == sample {
                continue;
            }
            let mut candidates = BTreeSet::<u16>::new();
            candidates.insert(current);
            for neighbor in topology.neighbors(sample) {
                candidates.insert(owners[*neighbor as usize]);
            }
            if candidates.len() <= 1 {
                continue;
            }
            let current_score = candidate_score(
                topology,
                sample,
                current,
                current,
                &owners,
                &initial,
                &angular_velocity,
                &shape_axes,
                epoch,
            );
            let mut best_owner = current;
            let mut best_score = current_score;
            for candidate in candidates {
                if candidate == current {
                    continue;
                }
                let score = candidate_score(
                    topology,
                    sample,
                    candidate,
                    current,
                    &owners,
                    &initial,
                    &angular_velocity,
                    &shape_axes,
                    epoch,
                );
                if score > best_score {
                    best_score = score;
                    best_owner = candidate;
                }
            }
            let advantage = best_score - current_score;
            if best_owner != current && advantage > 0.075 {
                proposals.push((advantage, sample, current, best_owner));
            }
        }

        proposals.sort_by(|left, right| {
            right
                .0
                .total_cmp(&left.0)
                .then_with(|| left.1.cmp(&right.1))
        });
        let move_limit = (proposals.len() / 3).max(1);
        let mut moved = 0usize;
        for (_, sample, from, to) in proposals {
            if moved >= move_limit || owners[sample as usize] != from {
                continue;
            }
            if plate_sizes[from as usize] <= minimum_plate_samples
                || plate_sizes[to as usize] >= maximum_plate_samples
            {
                continue;
            }
            owners[sample as usize] = to;
            plate_sizes[from as usize] -= 1;
            plate_sizes[to as usize] += 1;
            moved += 1;
        }

        repair_connectivity(topology, &mut owners, &anchors, plate_count);
        if moved == 0 {
            break;
        }
    }

    for (plate, anchor) in anchors.iter().enumerate() {
        owners[*anchor as usize] = plate as u16;
    }
    repair_connectivity(topology, &mut owners, &anchors, plate_count);
    Ok(owners)
}

fn split_fragments_at_modern_boundaries<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
    seed: u64,
) -> Result<(), WorldgenError> {
    let active_fragments = model.fragment_ids.iter().copied().collect::<BTreeSet<_>>();
    for fragment_id in active_fragments {
        let parent = model.fragments[fragment_id as usize].clone();
        let mut by_owner = BTreeMap::<u16, Vec<u32>>::new();
        for (sample, active_fragment) in model.fragment_ids.iter().copied().enumerate() {
            if active_fragment == fragment_id {
                by_owner
                    .entry(model.current_plate_ids[sample])
                    .or_default()
                    .push(sample as u32);
            }
        }
        if by_owner.is_empty() {
            continue;
        }
        if by_owner.len() == 1 {
            model.fragments[fragment_id as usize].current_plate_id = *by_owner.keys().next().unwrap();
            continue;
        }
        if model.fragments.len() + by_owner.len() >= usize::from(u16::MAX) {
            return Err(WorldgenError::InvalidLithosphere(
                "dynamic modern plate evolution exhausted fragment id capacity",
            ));
        }

        let dominant_owner = by_owner
            .iter()
            .max_by_key(|(owner, samples)| (samples.len(), std::cmp::Reverse(**owner)))
            .map(|(owner, _)| *owner)
            .unwrap_or(parent.current_plate_id);
        model.fragments[fragment_id as usize].current_plate_id = dominant_owner;

        for (owner, samples) in by_owner {
            let child_id = model.fragments.len() as u16;
            let area = samples
                .iter()
                .map(|sample| topology.area_steradians(*sample))
                .sum::<f64>();
            let capture_age = if owner != parent.current_plate_id {
                Some(
                    (6.0
                        + unit_random(
                            seed ^ u64::from(child_id).wrapping_mul(0xd6e8_feb8_6659_fd93),
                        ) * 54.0) as f32,
                )
            } else {
                parent.capture_age_myr
            };
            let seed_sample = samples[0];
            model.fragments.push(CrustFragment {
                id: child_id,
                parent_fragment_id: Some(parent.id),
                origin_plate_id: parent.origin_plate_id,
                current_plate_id: owner,
                seed_sample,
                birth_age_myr: parent.birth_age_myr,
                capture_age_myr: capture_age,
                accretion_age_myr: parent.accretion_age_myr,
                dominant_crust_kind: parent.dominant_crust_kind,
                sample_count: samples.len() as u32,
                area_steradians: area,
                mean_thickness_km: parent.mean_thickness_km,
                mean_density_kg_per_m3: parent.mean_density_kg_per_m3,
                inherited_fabric: parent.inherited_fabric,
            });
            for sample in &samples {
                model.fragment_ids[*sample as usize] = child_id;
            }
            if owner != parent.current_plate_id {
                model.events.push(HistoricalTectonicEvent {
                    id: model.events.len() as u32,
                    kind: HistoricalEventKind::Capture,
                    epoch: 7,
                    age_myr: capture_age.unwrap_or(18.0),
                    plate_a: parent.origin_plate_id,
                    plate_b: parent.origin_plate_id,
                    fragment_a: parent.id,
                    fragment_b: child_id,
                    displacement_km: 0.0,
                    strength: 0.22,
                    geometry_sample_a: parent.seed_sample,
                    geometry_sample_b: seed_sample,
                });
            }
        }
    }
    Ok(())
}

fn dynamic_history_hash(model: &HistoricalLithosphereModel, stage_seed: u64) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, DYNAMIC_PLATE_NAMESPACE.as_bytes());
    hash = fnv_update(hash, &stage_seed.to_le_bytes());
    hash = fnv_update(hash, &model.metrics.history_hash.to_le_bytes());
    for owner in &model.current_plate_ids {
        hash = fnv_update(hash, &owner.to_le_bytes());
    }
    for fragment in &model.fragment_ids {
        hash = fnv_update(hash, &fragment.to_le_bytes());
    }
    for fragment in &model.fragments {
        hash = fnv_update(hash, &fragment.id.to_le_bytes());
        hash = fnv_update(hash, &fragment.origin_plate_id.to_le_bytes());
        hash = fnv_update(hash, &fragment.current_plate_id.to_le_bytes());
        hash = fnv_update(
            hash,
            &fragment
                .parent_fragment_id
                .unwrap_or(u16::MAX)
                .to_le_bytes(),
        );
    }
    for event in &model.events {
        hash = fnv_update(hash, &event.id.to_le_bytes());
        hash = fnv_update(hash, &[event.kind as u8, event.epoch]);
        hash = fnv_update(hash, &event.fragment_a.to_le_bytes());
        hash = fnv_update(hash, &event.fragment_b.to_le_bytes());
    }
    hash
}

fn validate_dynamic_state<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
) -> Result<(), WorldgenError> {
    let plate_count = model.metrics.modern_plate_count;
    let mut plate_samples = vec![0usize; plate_count as usize];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let owner = model.current_plate_ids[index];
        if owner >= plate_count {
            return Err(WorldgenError::InvalidTectonics(
                "dynamic modern plate evolution produced an invalid owner",
            ));
        }
        plate_samples[owner as usize] += 1;
        let fragment_id = model.fragment_ids[index] as usize;
        if fragment_id >= model.fragments.len() {
            return Err(WorldgenError::InvalidLithosphere(
                "dynamic modern plate evolution produced an invalid fragment",
            ));
        }
        let fragment = &model.fragments[fragment_id];
        if fragment.current_plate_id != owner
            || fragment.origin_plate_id != model.origin_plate_ids[index]
        {
            return Err(WorldgenError::InvalidLithosphere(
                "dynamic modern plate evolution lost material/current ownership consistency",
            ));
        }
    }
    if plate_samples.iter().any(|count| *count == 0) {
        return Err(WorldgenError::InvalidTectonics(
            "dynamic modern plate evolution eliminated a modern plate",
        ));
    }
    Ok(())
}

pub fn evolve_modern_plate_geometry<T: PlanetTopology>(
    topology: &T,
    mut model: HistoricalLithosphereModel,
    seed: &str,
    planet: PlanetPhysicalParameters,
) -> Result<HistoricalLithosphereModel, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    if model.current_plate_ids.len() != topology.sample_count() as usize
        || model.origin_plate_ids.len() != topology.sample_count() as usize
        || model.fragment_ids.len() != topology.sample_count() as usize
    {
        return Err(WorldgenError::InvalidTectonics(
            "dynamic modern plate evolution topology does not match historical state",
        ));
    }

    let stage_seed = derive_stage_seed(seed, DYNAMIC_PLATE_NAMESPACE);
    model.current_plate_ids = evolve_ownership(topology, &model, stage_seed)?;
    split_fragments_at_modern_boundaries(topology, &mut model, stage_seed)?;
    model.metrics.fragment_count = model.fragments.len() as u16;
    model.metrics.event_count = model.events.len() as u32;
    model.metrics.history_hash = dynamic_history_hash(&model, stage_seed);
    validate_dynamic_state(topology, &model)?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_icosphere, HistoricalLithosphereRequest};

    fn connected_plate_count<T: PlanetTopology>(
        topology: &T,
        owners: &[u16],
        plate_count: u16,
    ) -> usize {
        let mut connected = 0usize;
        for plate in 0..plate_count {
            let Some(start) = owners.iter().position(|owner| *owner == plate) else {
                continue;
            };
            let mut seen = vec![false; owners.len()];
            let mut queue = VecDeque::from([start as u32]);
            seen[start] = true;
            let mut reached = 0usize;
            while let Some(sample) = queue.pop_front() {
                reached += 1;
                for neighbor in topology.neighbors(sample) {
                    let index = *neighbor as usize;
                    if !seen[index] && owners[index] == plate {
                        seen[index] = true;
                        queue.push_back(*neighbor);
                    }
                }
            }
            let expected = owners.iter().filter(|owner| **owner == plate).count();
            if reached == expected {
                connected += 1;
            }
        }
        connected
    }

    #[test]
    fn dynamic_modern_boundaries_cut_across_ancestral_cells() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let model = crate::generate_historical_lithosphere(
            &topology,
            &HistoricalLithosphereRequest::new("dynamic-plate-regression", 16),
            planet,
        )
        .unwrap();

        let mut inherited_edges = 0usize;
        let mut evolved_edges = 0usize;
        for sample in 0..topology.sample_count() {
            for neighbor in topology.neighbors(sample) {
                if *neighbor <= sample {
                    continue;
                }
                let a = sample as usize;
                let b = *neighbor as usize;
                if model.current_plate_ids[a] != model.current_plate_ids[b] {
                    if model.origin_plate_ids[a] == model.origin_plate_ids[b] {
                        evolved_edges += 1;
                    } else {
                        inherited_edges += 1;
                    }
                }
            }
        }
        assert!(evolved_edges > 0, "modern boundaries never cut ancestral material");
        assert!(
            evolved_edges * 20 >= inherited_edges.max(1),
            "modern geometry remained overwhelmingly inherited from ancestral cell edges"
        );
        assert_eq!(
            connected_plate_count(&topology, &model.current_plate_ids, 16),
            16,
            "dynamic evolution produced a disconnected modern plate"
        );
    }
}
