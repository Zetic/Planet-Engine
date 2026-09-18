use crate::{
    derive_stage_seed, CrustFragment, HistoricalEventKind, HistoricalLithosphereModel,
    HistoricalTectonicEvent, PlanetPhysicalParameters, PlanetTopology, WorldgenError,
};
use std::collections::{BTreeMap, BTreeSet};

const DYNAMIC_PLATE_NAMESPACE: &str = "worldgen:geology:dynamic-modern-plates:v2";
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
            let owner = *by_owner.keys().next().unwrap();
            let index = fragment_id as usize;
            let previous_owner = model.fragments[index].current_plate_id;
            if owner != previous_owner {
                let capture_age = (6.0
                    + unit_random(
                        seed ^ u64::from(fragment_id).wrapping_mul(0xd6e8_feb8_6659_fd93),
                    ) * 54.0) as f32;
                model.fragments[index].current_plate_id = owner;
                model.fragments[index].capture_age_myr = Some(capture_age);
                let origin = model.fragments[index].origin_plate_id;
                let geometry = model.fragments[index].seed_sample;
                model.events.push(HistoricalTectonicEvent {
                    id: model.events.len() as u32,
                    kind: HistoricalEventKind::Capture,
                    epoch: 7,
                    age_myr: capture_age,
                    plate_a: origin,
                    plate_b: origin,
                    fragment_a: fragment_id,
                    fragment_b: fragment_id,
                    displacement_km: 0.0,
                    strength: 0.18,
                    geometry_sample_a: geometry,
                    geometry_sample_b: geometry,
                });
            }
            continue;
        }
        if model.fragments.len() + by_owner.len() >= usize::from(u16::MAX) {
            return Err(WorldgenError::InvalidLithosphere(
                "dynamic modern plate evolution exhausted fragment id capacity",
            ));
        }

        for (owner, samples) in by_owner {
            let child_id = model.fragments.len() as u16;
            let area = samples
                .iter()
                .map(|sample| topology.area_steradians(*sample))
                .sum::<f64>();
            let capture_age = if owner != parent.current_plate_id {
                Some(
                    (6.0 + unit_random(
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
    model.current_plate_ids = crate::boundary_plate_geometry::synthesize_boundary_first_ownership(
        topology, &model, seed,
    )?;
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
    use std::collections::VecDeque;

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
        assert!(
            evolved_edges > 0,
            "modern boundaries never cut ancestral material"
        );
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
