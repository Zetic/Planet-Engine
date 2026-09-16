use crate::{
    derive_stage_seed, CrustFragment, HistoricalEventKind, HistoricalLithosphereModel,
    HistoricalTectonicEvent, PlanetTopology, WorldgenError,
};

pub const HISTORICAL_EPOCH_COUNT: u8 = 8;
const HISTORICAL_EPOCH_NAMESPACE: &str = "worldgen:geology:historical-lithosphere:epochs:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const MIN_SPLIT_SAMPLES: usize = 8;
const MIN_CHILD_FRACTION: f64 = 0.16;
// Epoch work remains deliberately bounded so historical lineage stays sparse and production-safe.

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

fn arc_radians(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(a, b).clamp(-1.0, 1.0).acos()
}

#[derive(Clone, Debug)]
struct SplitPlan {
    parent_id: u16,
    epoch: u8,
    age_myr: f32,
    stream: u64,
}

fn candidate_split_plans(model: &HistoricalLithosphereModel, stage_seed: u64) -> Vec<SplitPlan> {
    let original_fragment_count = model.fragments.len();
    let mut plans = Vec::new();
    let mut fallback: Option<(u32, u16)> = None;

    for fragment in model.fragments.iter().take(original_fragment_count) {
        if fragment.sample_count as usize >= MIN_SPLIT_SAMPLES {
            if fallback
                .map(|(count, id)| (fragment.sample_count, fragment.id) > (count, id))
                .unwrap_or(true)
            {
                fallback = Some((fragment.sample_count, fragment.id));
            }
        }

        if (fragment.sample_count as usize) < MIN_SPLIT_SAMPLES {
            continue;
        }
        let stream = stage_seed
            ^ u64::from(fragment.id).wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ u64::from(fragment.origin_plate_id).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        let inherited_weakness = f64::from(fragment.inherited_fabric).clamp(0.0, 1.0);
        let split_score = unit_random(stream ^ 0xa076_1d64_78bd_642f) * 0.68
            + inherited_weakness * 0.32;
        if split_score < 0.54 {
            continue;
        }
        let epoch = 1 + (mix64(stream ^ 0xe703_7ed1_a0b4_28db) % 6) as u8;
        let age_myr = (330.0 - f32::from(epoch) * 42.0
            + (unit_random(stream ^ 0x8ebc_6af0_9c88_c6e3) * 24.0) as f32)
            .max(18.0);
        plans.push(SplitPlan {
            parent_id: fragment.id,
            epoch,
            age_myr,
            stream,
        });
    }

    if plans.is_empty() {
        if let Some((_, parent_id)) = fallback {
            let stream = stage_seed ^ u64::from(parent_id).wrapping_mul(0xd6e8_feb8_6659_fd93);
            plans.push(SplitPlan {
                parent_id,
                epoch: 4,
                age_myr: 162.0,
                stream,
            });
        }
    }

    plans.sort_by(|left, right| {
        left.epoch
            .cmp(&right.epoch)
            .then_with(|| left.parent_id.cmp(&right.parent_id))
    });
    plans
}

fn split_fragment<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
    plan: &SplitPlan,
) -> Result<bool, WorldgenError> {
    let parent_index = usize::from(plan.parent_id);
    let Some(parent) = model.fragments.get(parent_index).cloned() else {
        return Err(WorldgenError::InvalidLithosphere(
            "historical epoch split references an invalid parent fragment",
        ));
    };

    let samples = model
        .fragment_ids
        .iter()
        .enumerate()
        .filter_map(|(sample, fragment)| (*fragment == plan.parent_id).then_some(sample as u32))
        .collect::<Vec<_>>();
    if samples.len() < MIN_SPLIT_SAMPLES {
        return Ok(false);
    }

    let seed_a = if samples.contains(&parent.seed_sample) {
        parent.seed_sample
    } else {
        samples[0]
    };
    let position_a = topology.unit_position(seed_a);
    let seed_b = samples
        .iter()
        .copied()
        .max_by(|left, right| {
            let left_distance = arc_radians(position_a, topology.unit_position(*left));
            let right_distance = arc_radians(position_a, topology.unit_position(*right));
            left_distance
                .total_cmp(&right_distance)
                .then_with(|| left.cmp(right))
        })
        .unwrap_or(seed_a);
    if seed_a == seed_b {
        return Ok(false);
    }

    let position_b = topology.unit_position(seed_b);
    let mut side_a = Vec::new();
    let mut side_b = Vec::new();
    for sample in samples {
        let position = topology.unit_position(sample);
        let distance_a = arc_radians(position, position_a);
        let distance_b = arc_radians(position, position_b);
        if distance_a < distance_b {
            side_a.push(sample);
        } else if distance_b < distance_a {
            side_b.push(sample);
        } else if mix64(plan.stream ^ u64::from(sample)) & 1 == 0 {
            side_a.push(sample);
        } else {
            side_b.push(sample);
        }
    }

    let total = side_a.len() + side_b.len();
    if total == 0 {
        return Ok(false);
    }
    let smaller_fraction = side_a.len().min(side_b.len()) as f64 / total as f64;
    if side_a.is_empty() || side_b.is_empty() || smaller_fraction < MIN_CHILD_FRACTION {
        return Ok(false);
    }
    if model.fragments.len() > usize::from(u16::MAX) - 2 {
        return Err(WorldgenError::InvalidLithosphere(
            "historical epoch fragment lineage exceeds u16 capacity",
        ));
    }

    let child_a_id = model.fragments.len() as u16;
    let child_b_id = child_a_id + 1;
    let mut area_a = 0.0_f64;
    let mut area_b = 0.0_f64;
    for sample in &side_a {
        model.fragment_ids[*sample as usize] = child_a_id;
        area_a += topology.area_steradians(*sample);
    }
    for sample in &side_b {
        model.fragment_ids[*sample as usize] = child_b_id;
        area_b += topology.area_steradians(*sample);
    }

    let child = |id: u16, seed_sample: u32, sample_count: usize, area_steradians: f64, fabric_shift: f32| {
        CrustFragment {
            id,
            parent_fragment_id: Some(parent.id),
            origin_plate_id: parent.origin_plate_id,
            current_plate_id: parent.current_plate_id,
            seed_sample,
            birth_age_myr: parent.birth_age_myr,
            capture_age_myr: parent.capture_age_myr,
            accretion_age_myr: parent.accretion_age_myr,
            dominant_crust_kind: parent.dominant_crust_kind,
            sample_count: sample_count as u32,
            area_steradians,
            mean_thickness_km: parent.mean_thickness_km,
            mean_density_kg_per_m3: parent.mean_density_kg_per_m3,
            inherited_fabric: (parent.inherited_fabric + fabric_shift).clamp(0.0, 1.0),
        }
    };
    model.fragments.push(child(
        child_a_id,
        seed_a,
        side_a.len(),
        area_a,
        0.06,
    ));
    model.fragments.push(child(
        child_b_id,
        seed_b,
        side_b.len(),
        area_b,
        -0.04,
    ));

    model.events.push(HistoricalTectonicEvent {
        id: model.events.len() as u32,
        kind: HistoricalEventKind::Rift,
        epoch: plan.epoch,
        age_myr: plan.age_myr,
        plate_a: parent.origin_plate_id,
        plate_b: parent.origin_plate_id,
        fragment_a: child_a_id,
        fragment_b: child_b_id,
        displacement_km: (35.0 + unit_random(plan.stream ^ 0x243f_6a88_85a3_08d3) * 260.0) as f32,
        strength: (0.35 + unit_random(plan.stream ^ 0x1319_8a2e_0370_7344) * 0.55) as f32,
        geometry_sample_a: seed_a,
        geometry_sample_b: seed_b,
    });
    Ok(true)
}

fn lineage_hash(model: &HistoricalLithosphereModel, stage_seed: u64) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, b"geology:historical-lithosphere:epochs:v1\0");
    hash = fnv_update(hash, &stage_seed.to_le_bytes());
    hash = fnv_update(hash, &model.stage.derived_seed.to_le_bytes());
    for values in [
        &model.origin_plate_ids,
        &model.fragment_ids,
        &model.current_plate_ids,
    ] {
        for value in values {
            hash = fnv_update(hash, &value.to_le_bytes());
        }
    }
    hash = fnv_update(hash, &model.crust_kind);
    for age in &model.crust_birth_age_myr {
        hash = fnv_update(hash, &age.to_bits().to_le_bytes());
    }
    for fragment in &model.fragments {
        hash = fnv_update(hash, &fragment.id.to_le_bytes());
        hash = fnv_update(
            hash,
            &fragment.parent_fragment_id.unwrap_or(u16::MAX).to_le_bytes(),
        );
        hash = fnv_update(hash, &fragment.origin_plate_id.to_le_bytes());
        hash = fnv_update(hash, &fragment.current_plate_id.to_le_bytes());
        hash = fnv_update(hash, &fragment.seed_sample.to_le_bytes());
        hash = fnv_update(hash, &fragment.sample_count.to_le_bytes());
        hash = fnv_update(hash, &fragment.area_steradians.to_bits().to_le_bytes());
    }
    for event in &model.events {
        hash = fnv_update(hash, &event.id.to_le_bytes());
        hash = fnv_update(hash, &[event.kind as u8, event.epoch]);
        hash = fnv_update(hash, &event.age_myr.to_bits().to_le_bytes());
        hash = fnv_update(hash, &event.plate_a.to_le_bytes());
        hash = fnv_update(hash, &event.plate_b.to_le_bytes());
        hash = fnv_update(hash, &event.fragment_a.to_le_bytes());
        hash = fnv_update(hash, &event.fragment_b.to_le_bytes());
        hash = fnv_update(hash, &event.geometry_sample_a.to_le_bytes());
        hash = fnv_update(hash, &event.geometry_sample_b.to_le_bytes());
    }
    hash
}

fn validate_lineage(model: &HistoricalLithosphereModel) -> Result<(), WorldgenError> {
    for (index, fragment) in model.fragments.iter().enumerate() {
        if fragment.id as usize != index {
            return Err(WorldgenError::InvalidLithosphere(
                "historical fragment ids are not dense lineage indices",
            ));
        }
        if let Some(parent) = fragment.parent_fragment_id {
            if parent >= fragment.id || usize::from(parent) >= model.fragments.len() {
                return Err(WorldgenError::InvalidLithosphere(
                    "historical fragment parent graph is not acyclic",
                ));
            }
            let parent_fragment = &model.fragments[parent as usize];
            if parent_fragment.origin_plate_id != fragment.origin_plate_id {
                return Err(WorldgenError::InvalidLithosphere(
                    "historical fragment split changed ancestral material provenance",
                ));
            }
        }
    }
    if model
        .fragment_ids
        .iter()
        .any(|fragment| usize::from(*fragment) >= model.fragments.len())
    {
        return Err(WorldgenError::InvalidLithosphere(
            "historical epoch split produced an invalid sample fragment id",
        ));
    }
    if model.events.iter().any(|event| event.epoch >= HISTORICAL_EPOCH_COUNT) {
        return Err(WorldgenError::InvalidLithosphere(
            "historical event lies outside the bounded epoch schedule",
        ));
    }
    Ok(())
}

pub fn evolve_historical_lithosphere<T: PlanetTopology>(
    topology: &T,
    mut model: HistoricalLithosphereModel,
    seed: &str,
) -> Result<HistoricalLithosphereModel, WorldgenError> {
    if topology.sample_count() as usize != model.fragment_ids.len() {
        return Err(WorldgenError::InvalidLithosphere(
            "historical epoch topology does not match material state",
        ));
    }
    let stage_seed = derive_stage_seed(seed, HISTORICAL_EPOCH_NAMESPACE);
    let plans = candidate_split_plans(&model, stage_seed);
    let mut split_count = 0_u32;
    for plan in &plans {
        split_count += u32::from(split_fragment(topology, &mut model, plan)?);
    }

    model.metrics.fragment_count = model.fragments.len() as u16;
    model.metrics.event_count = model.events.len() as u32;
    model.metrics.history_hash = lineage_hash(&model, stage_seed);
    validate_lineage(&model)?;

    if plans.iter().any(|plan| {
        model.fragments[usize::from(plan.parent_id)].sample_count as usize >= MIN_SPLIT_SAMPLES
    }) && split_count == 0
    {
        return Err(WorldgenError::InvalidLithosphere(
            "historical epoch schedule found splittable material but produced no lineage split",
        ));
    }
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_icosphere, historical_lithosphere, HistoricalLithosphereRequest,
        PlanetPhysicalParameters,
    };

    #[test]
    fn bounded_epochs_create_deterministic_parented_fragment_lineage() {
        let topology = build_icosphere(3).unwrap();
        let request = HistoricalLithosphereRequest::new("historical-epochs", 10);
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let base = historical_lithosphere::generate_historical_lithosphere(
            &topology,
            &request,
            planet,
        )
        .unwrap();
        let repeat_base = base.clone();
        let first = evolve_historical_lithosphere(&topology, base, &request.seed).unwrap();
        let second =
            evolve_historical_lithosphere(&topology, repeat_base, &request.seed).unwrap();
        assert_eq!(first.metrics.history_hash, second.metrics.history_hash);
        assert_eq!(first.fragment_ids, second.fragment_ids);
        assert!(first
            .fragments
            .iter()
            .any(|fragment| fragment.parent_fragment_id.is_some()));
        assert!(first.events.iter().any(|event| {
            event.kind == HistoricalEventKind::Rift && event.fragment_a != event.fragment_b
        }));
        validate_lineage(&first).unwrap();
    }
}
