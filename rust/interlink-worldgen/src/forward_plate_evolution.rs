use crate::{
    derive_stage_seed, CrustFragment, CrustKind, HistoricalEventKind,
    HistoricalLithosphereModel, HistoricalTectonicEvent, PlanetPhysicalParameters, PlanetTopology,
    WorldgenError,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const FORWARD_PLATE_NAMESPACE: &str = "worldgen:geology:forward-plate-evolution:v1";
const FORWARD_EPOCHS: usize = 8;
const SUBSTEPS_PER_EPOCH: usize = 4;
const EPOCH_DURATION_MYR: f64 = 20.0;
const SUBSTEP_MYR: f64 = EPOCH_DURATION_MYR / SUBSTEPS_PER_EPOCH as f64;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
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

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f64; 3], factor: f64) -> [f64; 3] {
    [a[0] * factor, a[1] * factor, a[2] * factor]
}

fn norm(value: [f64; 3]) -> f64 {
    dot(value, value).sqrt()
}

fn normalize_or(value: [f64; 3], fallback: [f64; 3]) -> [f64; 3] {
    let length = norm(value);
    if length > 1.0e-14 {
        scale(value, 1.0 / length)
    } else {
        fallback
    }
}

fn rotate_by_angular_velocity(
    position: [f64; 3],
    angular_velocity_rad_per_myr: [f64; 3],
    dt_myr: f64,
) -> [f64; 3] {
    let speed = norm(angular_velocity_rad_per_myr);
    if speed <= 1.0e-14 {
        return position;
    }
    let axis = scale(angular_velocity_rad_per_myr, 1.0 / speed);
    let angle = speed * dt_myr;
    let (sin_angle, cos_angle) = angle.sin_cos();
    normalize_or(
        add(
            add(
                scale(position, cos_angle),
                scale(cross(axis, position), sin_angle),
            ),
            scale(axis, dot(axis, position) * (1.0 - cos_angle)),
        ),
        position,
    )
}

fn nearest_sample<T: PlanetTopology>(
    topology: &T,
    target: [f64; 3],
    start: u32,
) -> u32 {
    let mut current = start;
    let mut current_alignment = dot(topology.unit_position(current), target);
    for _ in 0..32 {
        let mut best = current;
        let mut best_alignment = current_alignment;
        for neighbor in topology.neighbors(current) {
            let alignment = dot(topology.unit_position(*neighbor), target);
            if alignment > best_alignment + 1.0e-14
                || ((alignment - best_alignment).abs() <= 1.0e-14 && *neighbor < best)
            {
                best = *neighbor;
                best_alignment = alignment;
            }
        }
        if best == current {
            break;
        }
        current = best;
        current_alignment = best_alignment;
    }
    current
}

fn plate_core_samples<T: PlanetTopology>(
    topology: &T,
    owners: &[u16],
    plate_count: usize,
) -> Vec<Option<u32>> {
    let mut result = vec![None; plate_count];
    let mut best_support = vec![0usize; plate_count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let owner = owners[index] as usize;
        if owner >= plate_count {
            continue;
        }
        let support = topology
            .neighbors(sample)
            .iter()
            .filter(|neighbor| owners[**neighbor as usize] as usize == owner)
            .count();
        if result[owner].is_none()
            || support > best_support[owner]
            || (support == best_support[owner] && sample < result[owner].unwrap())
        {
            result[owner] = Some(sample);
            best_support[owner] = support;
        }
    }
    result
}

fn material_survival_bias(kind: u8, age_myr: f32) -> f64 {
    if kind == CrustKind::Continental as u8 {
        0.34
    } else if kind == CrustKind::Transitional as u8 {
        0.16
    } else {
        0.02 + 0.03 * (1.0 - (f64::from(age_myr) / 220.0).clamp(0.0, 1.0))
    }
}

fn nearest_owned_sample<T: PlanetTopology>(
    topology: &T,
    target: [f64; 3],
    start: u32,
    owners: &[u16],
    plate: u16,
) -> (u32, f64) {
    let mut current = start;
    let mut current_alignment = dot(topology.unit_position(current), target);
    for _ in 0..64 {
        let mut best = current;
        let mut best_alignment = current_alignment;
        for neighbor in topology.neighbors(current) {
            if owners[*neighbor as usize] != plate {
                continue;
            }
            let alignment = dot(topology.unit_position(*neighbor), target);
            if alignment > best_alignment + 1.0e-14
                || ((alignment - best_alignment).abs() <= 1.0e-14 && *neighbor < best)
            {
                best = *neighbor;
                best_alignment = alignment;
            }
        }
        if best == current {
            break;
        }
        current = best;
        current_alignment = best_alignment;
    }
    (current, current_alignment)
}

fn refresh_active_fragment_summaries<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
) {
    let active = model.fragment_ids.iter().copied().collect::<BTreeSet<_>>();
    let mut counts = BTreeMap::<u16, u32>::new();
    let mut areas = BTreeMap::<u16, f64>::new();
    let mut kinds = BTreeMap::<u16, [u32; 3]>::new();
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let fragment = model.fragment_ids[index];
        *counts.entry(fragment).or_insert(0) += 1;
        *areas.entry(fragment).or_insert(0.0) += topology.area_steradians(sample);
        let bucket = if model.crust_kind[index] == CrustKind::Continental as u8 {
            2
        } else if model.crust_kind[index] == CrustKind::Transitional as u8 {
            1
        } else {
            0
        };
        kinds.entry(fragment).or_insert([0; 3])[bucket] += 1;
    }
    for fragment_id in active {
        if let Some(fragment) = model.fragments.get_mut(fragment_id as usize) {
            fragment.sample_count = counts.get(&fragment_id).copied().unwrap_or(0);
            fragment.area_steradians = areas.get(&fragment_id).copied().unwrap_or(0.0);
            if let Some(counts) = kinds.get(&fragment_id) {
                fragment.dominant_crust_kind = counts
                    .iter()
                    .enumerate()
                    .max_by_key(|(kind, count)| (**count, *kind))
                    .map(|(kind, _)| match kind {
                        2 => CrustKind::Continental as u8,
                        1 => CrustKind::Transitional as u8,
                        _ => CrustKind::Oceanic as u8,
                    })
                    .unwrap_or(fragment.dominant_crust_kind);
            }
        }
    }
}


fn materialize_generated_crust<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
    epoch: u8,
    generation_fragments: &mut BTreeMap<(u8, u16, u8, u16), u16>,
    generated: &[bool],
    origin_plate_ids: &[u16],
    fragment_ids: &mut [u16],
    owners: &[u16],
    crust_kind: &[u8],
) -> Result<(), WorldgenError> {
    if generated.len() != topology.sample_count() as usize {
        return Err(WorldgenError::InvalidLithosphere(
            "generated-crust mask does not match topology",
        ));
    }
    let mut seen = vec![false; generated.len()];
    for start in 0..topology.sample_count() {
        let si = start as usize;
        if seen[si] || !generated[si] {
            continue;
        }
        let owner = owners[si];
        let kind = crust_kind[si];
        let origin = origin_plate_ids[si];
        let mut queue = VecDeque::from([start]);
        let mut component = Vec::<u32>::new();
        seen[si] = true;
        while let Some(sample) = queue.pop_front() {
            component.push(sample);
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                if !seen[ni]
                    && generated[ni]
                    && owners[ni] == owner
                    && crust_kind[ni] == kind
                    && origin_plate_ids[ni] == origin
                {
                    seen[ni] = true;
                    queue.push_back(*neighbor);
                }
            }
        }
        let key = (epoch, owner, kind, origin);
        let id = if let Some(existing) = generation_fragments.get(&key).copied() {
            existing
        } else {
            if model.fragments.len() >= usize::from(u16::MAX) {
                return Err(WorldgenError::InvalidLithosphere(
                    "forward spreading exhausted fragment id capacity",
                ));
            }
            let id = model.fragments.len() as u16;
            let area = component
                .iter()
                .map(|sample| topology.area_steradians(*sample))
                .sum::<f64>();
            let (thickness_km, density_kg_per_m3) =
                if kind == CrustKind::Oceanic as u8 {
                    (7.0, 2935.0)
                } else {
                    (19.0, 2875.0)
                };
            model.fragments.push(CrustFragment {
                id,
                parent_fragment_id: None,
                origin_plate_id: origin,
                current_plate_id: owner,
                seed_sample: component[0],
                birth_age_myr: 0.0,
                capture_age_myr: None,
                accretion_age_myr: None,
                dominant_crust_kind: kind,
                sample_count: component.len() as u32,
                area_steradians: area,
                mean_thickness_km: thickness_km,
                mean_density_kg_per_m3: density_kg_per_m3,
                inherited_fabric: 0.15,
            });
            generation_fragments.insert(key, id);
            id
        };
        for sample in component {
            fragment_ids[sample as usize] = id;
        }
    }
    Ok(())
}

fn repair_plate_connectivity<T: PlanetTopology>(
    topology: &T,
    owners: &mut [u16],
    plate_count: usize,
) {
    // Reassign an entire detached component as one tectonic fragment. Cell-by-cell repair can
    // manufacture new disconnected islands on the receiving plate and was one of the pathologies
    // of the superseded final-state ownership synthesizer.
    for _ in 0..8 {
        let mut changed = false;
        for plate in 0..plate_count {
            let plate_id = plate as u16;
            let mut seen = vec![false; owners.len()];
            let mut components = Vec::<Vec<u32>>::new();
            for start in 0..topology.sample_count() {
                let start_index = start as usize;
                if seen[start_index] || owners[start_index] != plate_id {
                    continue;
                }
                seen[start_index] = true;
                let mut queue = VecDeque::from([start]);
                let mut component = Vec::new();
                while let Some(sample) = queue.pop_front() {
                    component.push(sample);
                    for neighbor in topology.neighbors(sample) {
                        let ni = *neighbor as usize;
                        if !seen[ni] && owners[ni] == plate_id {
                            seen[ni] = true;
                            queue.push_back(*neighbor);
                        }
                    }
                }
                components.push(component);
            }
            if components.len() <= 1 {
                continue;
            }
            components.sort_by(|left, right| right.len().cmp(&left.len()));
            for component in components.into_iter().skip(1) {
                let mut candidates = BTreeMap::<u16, usize>::new();
                for sample in &component {
                    for neighbor in topology.neighbors(*sample) {
                        let candidate = owners[*neighbor as usize];
                        if candidate != plate_id {
                            *candidates.entry(candidate).or_insert(0) += 1;
                        }
                    }
                }
                let Some((winner, _)) = candidates
                    .into_iter()
                    .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
                else {
                    continue;
                };
                for sample in component {
                    owners[sample as usize] = winner;
                }
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}

fn advect_substep<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
    velocities: &[[f64; 3]],
    epoch: u8,
    generation_fragments: &mut BTreeMap<(u8, u16, u8, u16), u16>,
    dt_myr: f64,
) -> Result<(), WorldgenError> {
    let count = topology.sample_count() as usize;
    let plate_count = model.metrics.modern_plate_count as usize;
    debug_assert_eq!(velocities.len(), plate_count);
    let cores = plate_core_samples(topology, &model.current_plate_ids, plate_count);

    let old_origin = model.origin_plate_ids.clone();
    let old_fragment = model.fragment_ids.clone();
    let old_owner = model.current_plate_ids.clone();
    let old_kind = model.crust_kind.clone();
    let old_age = model.crust_birth_age_myr.clone();
    let old_weakness = model.lithospheric_weakness_index.clone();

    let mut mapped_core_destination = vec![None; plate_count];
    for plate in 0..plate_count {
        if let Some(core) = cores[plate] {
            let target = rotate_by_angular_velocity(
                topology.unit_position(core),
                velocities[plate],
                dt_myr,
            );
            mapped_core_destination[plate] =
                Some(nearest_sample(topology, target, core) as usize);
        }
    }

    let mut new_origin = vec![u16::MAX; count];
    let mut new_fragment = vec![u16::MAX; count];
    let mut new_owner = vec![u16::MAX; count];
    let mut new_kind = vec![CrustKind::Oceanic as u8; count];
    let mut new_age = vec![0.0_f32; count];
    let mut new_weakness = vec![0.0_f32; count];
    let mut generated = vec![false; count];

    // Semi-Lagrangian rigid transport: for each destination cell, back-rotate through every
    // moving plate and ask whether the preimage lies inside that plate's previous material
    // domain. This transports coherent plate shapes. It replaces the source-cell scatter that
    // shredded continental blocks whenever several parcels rounded onto the same raster cell.
    for destination in 0..topology.sample_count() {
        let destination_index = destination as usize;
        let destination_position = topology.unit_position(destination);
        let mut candidate_plates = BTreeSet::<u16>::new();
        candidate_plates.insert(old_owner[destination_index]);
        for neighbor in topology.neighbors(destination) {
            candidate_plates.insert(old_owner[*neighbor as usize]);
            for second in topology.neighbors(*neighbor) {
                candidate_plates.insert(old_owner[*second as usize]);
            }
        }
        let mut best: Option<(f64, usize, u16)> = None;
        for plate_id in candidate_plates {
            let plate = plate_id as usize;
            if plate >= plate_count {
                continue;
            }
            let Some(core) = cores[plate] else { continue };
            let preimage = rotate_by_angular_velocity(
                destination_position,
                velocities[plate],
                -dt_myr,
            );
            let (source, alignment) =
                nearest_owned_sample(topology, preimage, core, &old_owner, plate as u16);
            let source_index = source as usize;
            let distance = alignment.clamp(-1.0, 1.0).acos();
            let lengths = topology.neighbor_arc_lengths_rad(source);
            let local_scale = if lengths.is_empty() {
                0.0
            } else {
                lengths.iter().copied().sum::<f64>() / lengths.len() as f64
            };
            if local_scale <= 0.0 || distance > local_scale * 0.82 {
                continue;
            }
            let normalized_distance = distance / local_scale;
            let score = -normalized_distance
                + material_survival_bias(old_kind[source_index], old_age[source_index]);
            let candidate = (score, source_index, plate as u16);
            if best
                .as_ref()
                .map(|current| {
                    candidate.0 > current.0 + 1.0e-14
                        || ((candidate.0 - current.0).abs() <= 1.0e-14
                            && (candidate.2, candidate.1) < (current.2, current.1))
                })
                .unwrap_or(true)
            {
                best = Some(candidate);
            }
        }
        let Some((_score, source, plate)) = best else {
            continue;
        };
        new_origin[destination_index] = old_origin[source];
        new_fragment[destination_index] = old_fragment[source];
        new_owner[destination_index] = plate;
        new_kind[destination_index] = old_kind[source];
        new_weakness[destination_index] = old_weakness[source];
        new_age[destination_index] = if old_kind[source] == CrustKind::Oceanic as u8 {
            (old_age[source] + dt_myr as f32).clamp(0.0, 220.0)
        } else {
            old_age[source]
        };
    }

    // Forward advection can leave uncovered raster cells where neighboring material moved apart.
    // Those are genuine extensional gaps, not ownership-score holes. Fill them outward from the
    // advected margins and create transitional/oceanic material rather than repainting old crust.
    let mut remaining = new_owner.iter().filter(|owner| **owner == u16::MAX).count();
    for _ in 0..count {
        if remaining == 0 {
            break;
        }
        let previous_owner = new_owner.clone();
        let previous_origin = new_origin.clone();
        let previous_fragment = new_fragment.clone();
        let previous_kind = new_kind.clone();
        let mut changed = 0usize;
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            if previous_owner[index] != u16::MAX {
                continue;
            }
            let mut choices = Vec::<usize>::new();
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                if previous_owner[ni] != u16::MAX {
                    choices.push(ni);
                }
            }
            if choices.is_empty() {
                continue;
            }
            choices.sort_by_key(|neighbor| {
                (
                    previous_owner[*neighbor],
                    previous_origin[*neighbor],
                    *neighbor,
                )
            });
            let donor = choices[0];
            let has_oceanic_margin = choices
                .iter()
                .any(|neighbor| previous_kind[*neighbor] == CrustKind::Oceanic as u8);
            new_owner[index] = previous_owner[donor];
            new_origin[index] = previous_origin[donor];
            new_fragment[index] = previous_fragment[donor];
            new_kind[index] = if has_oceanic_margin {
                CrustKind::Oceanic as u8
            } else {
                CrustKind::Transitional as u8
            };
            // Newly opened lithosphere is mechanically weak/hot. This is physical state attached
            // to the generated material, not a property selected by its new genealogy id.
            new_weakness[index] = if new_kind[index] == CrustKind::Oceanic as u8 {
                0.48
            } else {
                0.74
            };
            new_age[index] = 0.0;
            generated[index] = true;
            changed += 1;
        }
        if changed == 0 {
            break;
        }
        remaining = remaining.saturating_sub(changed);
    }

    // Preserve one advected material core per requested plate. This is only a raster-degeneracy
    // guard; ordinary geometry comes from rigid forward transport and collision/gap resolution.
    let mut counts = vec![0usize; plate_count];
    for owner in &new_owner {
        if (*owner as usize) < plate_count {
            counts[*owner as usize] += 1;
        }
    }
    for plate in 0..plate_count {
        if counts[plate] > 0 {
            continue;
        }
        let Some(source) = cores[plate] else { continue };
        let preferred = mapped_core_destination[plate].unwrap_or(source as usize);
        let mut destination = preferred;
        let donor = new_owner[destination];
        if (donor as usize) < plate_count && counts[donor as usize] <= 1 {
            // Do not preserve one plate by deleting another. Walk outward from the advected
            // core target until a donor with more than one raster sample is found.
            let mut seen = vec![false; count];
            let mut queue = VecDeque::from([preferred as u32]);
            seen[preferred] = true;
            while let Some(sample) = queue.pop_front() {
                let index = sample as usize;
                let owner = new_owner[index] as usize;
                if owner < plate_count && counts[owner] > 1 {
                    destination = index;
                    break;
                }
                for neighbor in topology.neighbors(sample) {
                    let ni = *neighbor as usize;
                    if !seen[ni] {
                        seen[ni] = true;
                        queue.push_back(*neighbor);
                    }
                }
            }
        }
        let previous = new_owner[destination] as usize;
        if previous < plate_count {
            counts[previous] = counts[previous].saturating_sub(1);
        }
        let source_index = source as usize;
        new_origin[destination] = old_origin[source_index];
        new_fragment[destination] = old_fragment[source_index];
        new_owner[destination] = plate as u16;
        new_kind[destination] = old_kind[source_index];
        new_age[destination] = old_age[source_index];
        new_weakness[destination] = old_weakness[source_index];
        counts[plate] += 1;
    }

    repair_plate_connectivity(topology, &mut new_owner, plate_count);

    materialize_generated_crust(
        topology,
        model,
        epoch,
        generation_fragments,
        &generated,
        &new_origin,
        &mut new_fragment,
        &new_owner,
        &new_kind,
    )?;

    model.origin_plate_ids = new_origin;
    model.fragment_ids = new_fragment;
    model.current_plate_ids = new_owner;
    model.lithospheric_weakness_index = new_weakness;
    model.crust_kind = new_kind;
    model.crust_birth_age_myr = new_age;
    Ok(())
}

#[derive(Clone, Copy, Default)]
struct BoundaryEpochSummary {
    convergence: f64,
    divergence: f64,
    shear: f64,
    speed_sum: f64,
    edges: u32,
    sample_a: u32,
    sample_b: u32,
    initialized: bool,
}

fn record_epoch_events<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
    velocities: &[[f64; 3]],
    epoch: usize,
    planet: PlanetPhysicalParameters,
) {
    debug_assert_eq!(velocities.len(), model.metrics.modern_plate_count as usize);
    let mut summaries = BTreeMap::<(u16, u16), BoundaryEpochSummary>::new();
    for sample_a in 0..topology.sample_count() {
        let index_a = sample_a as usize;
        for sample_b in topology.neighbors(sample_a) {
            if *sample_b <= sample_a {
                continue;
            }
            let index_b = *sample_b as usize;
            let owner_a = model.current_plate_ids[index_a];
            let owner_b = model.current_plate_ids[index_b];
            if owner_a == owner_b {
                continue;
            }
            let pair = if owner_a < owner_b {
                (owner_a, owner_b)
            } else {
                (owner_b, owner_a)
            };
            let position_a = topology.unit_position(sample_a);
            let position_b = topology.unit_position(*sample_b);
            let midpoint = normalize_or(add(position_a, position_b), position_a);
            let normal = normalize_or(sub(position_b, position_a), position_a);
            let tangent = normalize_or(cross(midpoint, normal), [1.0, 0.0, 0.0]);
            let velocity = |owner: u16| {
                scale(
                    cross(velocities[owner as usize], midpoint),
                    planet.radius_m / 1_000_000.0,
                )
            };
            let relative = sub(velocity(owner_b), velocity(owner_a));
            let normal_rate = dot(relative, normal);
            let shear_rate = dot(relative, tangent);
            let summary = summaries.entry(pair).or_default();
            if !summary.initialized {
                summary.sample_a = sample_a;
                summary.sample_b = *sample_b;
                summary.initialized = true;
            }
            if normal_rate < -1.0e-9 {
                summary.convergence += -normal_rate;
            } else if normal_rate > 1.0e-9 {
                summary.divergence += normal_rate;
            }
            summary.shear += shear_rate.abs();
            summary.speed_sum += normal_rate.hypot(shear_rate);
            summary.edges += 1;
        }
    }

    for ((_plate_low, _plate_high), summary) in summaries {
        if !summary.initialized || summary.edges == 0 {
            continue;
        }
        let a = summary.sample_a as usize;
        let b = summary.sample_b as usize;
        let dominant_normal = summary.convergence.max(summary.divergence);
        let kind = if summary.shear > dominant_normal * 1.15 {
            HistoricalEventKind::Transform
        } else if summary.divergence >= summary.convergence {
            if model.crust_kind[a] == CrustKind::Oceanic as u8
                && model.crust_kind[b] == CrustKind::Oceanic as u8
            {
                HistoricalEventKind::Spreading
            } else {
                HistoricalEventKind::Rift
            }
        } else if model.crust_kind[a] != CrustKind::Oceanic as u8
            && model.crust_kind[b] != CrustKind::Oceanic as u8
        {
            HistoricalEventKind::Collision
        } else if model.crust_kind[a] == CrustKind::Oceanic as u8
            && model.crust_kind[b] == CrustKind::Oceanic as u8
        {
            HistoricalEventKind::Subduction
        } else {
            HistoricalEventKind::Accretion
        };
        let mean_speed = summary.speed_sum / f64::from(summary.edges);
        let age_myr = ((FORWARD_EPOCHS - epoch) as f64 * EPOCH_DURATION_MYR) as f32;
        model.events.push(HistoricalTectonicEvent {
            id: model.events.len() as u32,
            kind,
            epoch: epoch.min(usize::from(crate::HISTORICAL_EPOCH_COUNT - 1)) as u8,
            age_myr,
            plate_a: model.origin_plate_ids[a],
            plate_b: model.origin_plate_ids[b],
            fragment_a: model.fragment_ids[a],
            fragment_b: model.fragment_ids[b],
            displacement_km: (mean_speed * EPOCH_DURATION_MYR * 1000.0)
                .clamp(0.0, 5000.0) as f32,
            strength: (0.20 + (mean_speed / 0.08).clamp(0.0, 1.0) * 0.80) as f32,
            geometry_sample_a: summary.sample_a,
            geometry_sample_b: summary.sample_b,
        });
    }
}



#[derive(Clone, Copy)]
struct RiftCandidate {
    plate: u16,
    sample_a: u32,
    sample_b: u32,
    plane_normal: [f64; 3],
    weakness: f64,
    score: f64,
}

fn split_one_rifting_plate<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
    velocities: &mut Vec<[f64; 3]>,
    epoch: usize,
    stage_seed: u64,
) -> bool {
    let plate_count = model.metrics.modern_plate_count as usize;
    if velocities.len() != plate_count || plate_count >= usize::from(u16::MAX) {
        return false;
    }

    let mut samples_by_plate = vec![Vec::<u32>::new(); plate_count];
    let mut non_oceanic_by_plate = vec![0usize; plate_count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let owner = model.current_plate_ids[index] as usize;
        if owner >= plate_count {
            continue;
        }
        samples_by_plate[owner].push(sample);
        if model.crust_kind[index] != CrustKind::Oceanic as u8 {
            non_oceanic_by_plate[owner] += 1;
        }
    }

    // Rift nucleation follows the advected physical weakness field. Genealogical fragment
    // boundaries are deliberately absent from candidate selection: ancestry may describe the
    // material later, but it may not decide where a plate physically breaks.
    let mut best_edge_by_plate = vec![None::<RiftCandidate>; plate_count];
    for sample_a in 0..topology.sample_count() {
        let a = sample_a as usize;
        let owner = model.current_plate_ids[a] as usize;
        if owner >= plate_count
            || samples_by_plate[owner].len() < 48
            || non_oceanic_by_plate[owner] * 100 < samples_by_plate[owner].len() * 30
        {
            continue;
        }
        for sample_b in topology.neighbors(sample_a) {
            if *sample_b <= sample_a {
                continue;
            }
            let b = *sample_b as usize;
            if model.current_plate_ids[b] as usize != owner {
                continue;
            }
            if model.crust_kind[a] == CrustKind::Oceanic as u8
                && model.crust_kind[b] == CrustKind::Oceanic as u8
            {
                continue;
            }

            let weakness = (f64::from(model.lithospheric_weakness_index[a])
                + f64::from(model.lithospheric_weakness_index[b]))
                * 0.5;
            if weakness < 0.30 {
                continue;
            }
            let material_bonus = if model.crust_kind[a] != CrustKind::Oceanic as u8
                && model.crust_kind[b] != CrustKind::Oceanic as u8
            {
                0.30
            } else {
                0.08
            };
            let tie = ((stage_seed
                ^ u64::from(sample_a).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                ^ u64::from(*sample_b).wrapping_mul(0xbf58_476d_1ce4_e5b9)
                ^ (epoch as u64).wrapping_mul(0x94d0_49bb_1331_11eb))
                .rotate_left(17) as f64
                / u64::MAX as f64)
                * 1.0e-6;
            let plane_normal = normalize_or(
                sub(topology.unit_position(*sample_b), topology.unit_position(sample_a)),
                [1.0, 0.0, 0.0],
            );
            let score = weakness + material_bonus + tie;
            let candidate = RiftCandidate {
                plate: owner as u16,
                sample_a,
                sample_b: *sample_b,
                plane_normal,
                weakness,
                score,
            };
            if best_edge_by_plate[owner]
                .map(|current| {
                    candidate.score > current.score + 1.0e-12
                        || ((candidate.score - current.score).abs() <= 1.0e-12
                            && (candidate.sample_a, candidate.sample_b)
                                < (current.sample_a, current.sample_b))
                })
                .unwrap_or(true)
            {
                best_edge_by_plate[owner] = Some(candidate);
            }
        }
    }

    let mut best: Option<(f64, RiftCandidate)> = None;
    for candidate in best_edge_by_plate.into_iter().flatten() {
        let plate = candidate.plate as usize;
        let samples = &samples_by_plate[plate];
        let mut negative = 0usize;
        let mut positive = 0usize;
        for sample in samples {
            let side = dot(topology.unit_position(*sample), candidate.plane_normal);
            if side >= 0.0 {
                positive += 1;
            } else {
                negative += 1;
            }
        }
        let minimum_side = negative.min(positive);
        if minimum_side < 16 || minimum_side * 5 < samples.len() {
            continue;
        }

        // Both child bodies must already be connected under the proposed rift. Reject a split that
        // would require the old ownership-repair machinery to manufacture connectivity afterward.
        let mut connected = true;
        for positive_side in [false, true] {
            let expected = if positive_side { positive } else { negative };
            let Some(start) = samples.iter().copied().find(|sample| {
                (dot(topology.unit_position(*sample), candidate.plane_normal) >= 0.0)
                    == positive_side
            }) else {
                connected = false;
                break;
            };
            let mut seen = BTreeSet::<u32>::new();
            let mut queue = VecDeque::from([start]);
            seen.insert(start);
            while let Some(sample) = queue.pop_front() {
                for neighbor in topology.neighbors(sample) {
                    let ni = *neighbor as usize;
                    if model.current_plate_ids[ni] != candidate.plate {
                        continue;
                    }
                    let neighbor_side =
                        dot(topology.unit_position(*neighbor), candidate.plane_normal) >= 0.0;
                    if neighbor_side == positive_side && seen.insert(*neighbor) {
                        queue.push_back(*neighbor);
                    }
                }
            }
            if seen.len() != expected {
                connected = false;
                break;
            }
        }
        if !connected {
            continue;
        }

        let balance = minimum_side as f64 / negative.max(positive) as f64;
        let size_weight = (samples.len() as f64 / 64.0).sqrt().min(3.0);
        let score = candidate.score * (0.55 + 0.45 * balance) * size_weight;
        if best
            .as_ref()
            .map(|current| {
                score > current.0 + 1.0e-12
                    || ((score - current.0).abs() <= 1.0e-12
                        && candidate.plate < current.1.plate)
            })
            .unwrap_or(true)
        {
            best = Some((score, candidate));
        }
    }

    let Some((_score, candidate)) = best else {
        return false;
    };
    let parent = candidate.plate as usize;
    let child = plate_count as u16;
    for sample in &samples_by_plate[parent] {
        if dot(topology.unit_position(*sample), candidate.plane_normal) >= 0.0 {
            model.current_plate_ids[*sample as usize] = child;
        }
    }

    // Give the two new rigid bodies a divergent velocity component normal to the inherited weak
    // contact. This creates an actual opening boundary that can generate new lithosphere during
    // subsequent advection instead of recording a rift label on a static partition.
    let midpoint = normalize_or(
        add(
            topology.unit_position(candidate.sample_a),
            topology.unit_position(candidate.sample_b),
        ),
        topology.unit_position(candidate.sample_a),
    );
    let angular_offset =
        (0.07 + 0.11 * candidate.weakness.clamp(0.0, 1.0)).to_radians();
    let delta = scale(
        cross(midpoint, candidate.plane_normal),
        angular_offset,
    );
    let base_velocity = velocities[parent];
    velocities[parent] = sub(base_velocity, delta);
    velocities.push(add(base_velocity, delta));
    model.metrics.modern_plate_count = child + 1;

    let a = candidate.sample_a as usize;
    let b = candidate.sample_b as usize;
    model.events.push(HistoricalTectonicEvent {
        id: model.events.len() as u32,
        kind: HistoricalEventKind::Rift,
        epoch: epoch.min(usize::from(crate::HISTORICAL_EPOCH_COUNT - 1)) as u8,
        age_myr: ((FORWARD_EPOCHS - epoch) as f64 * EPOCH_DURATION_MYR) as f32,
        plate_a: model.origin_plate_ids[a],
        plate_b: model.origin_plate_ids[b],
        fragment_a: model.fragment_ids[a],
        fragment_b: model.fragment_ids[b],
        displacement_km: 0.0,
        strength: (0.45 + 0.50 * candidate.weakness.clamp(0.0, 1.0)) as f32,
        geometry_sample_a: candidate.sample_a,
        geometry_sample_b: candidate.sample_b,
    });
    true
}

fn merge_one_converging_plate_pair<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
    velocities: &mut Vec<[f64; 3]>,
    _epoch: usize,
    planet: PlanetPhysicalParameters,
) -> bool {
    let plate_count = model.metrics.modern_plate_count as usize;
    if plate_count <= 1 || velocities.len() != plate_count {
        return false;
    }
    let mut plate_area = vec![0.0_f64; plate_count];
    let mut continental_area = vec![0.0_f64; plate_count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let owner = model.current_plate_ids[index] as usize;
        if owner >= plate_count {
            continue;
        }
        let area = topology.area_steradians(sample);
        plate_area[owner] += area;
        if model.crust_kind[index] == CrustKind::Continental as u8 {
            continental_area[owner] += area;
        }
    }

    let mut summaries = BTreeMap::<(u16, u16), BoundaryEpochSummary>::new();
    for sample_a in 0..topology.sample_count() {
        let a = sample_a as usize;
        for sample_b in topology.neighbors(sample_a) {
            if *sample_b <= sample_a {
                continue;
            }
            let b = *sample_b as usize;
            let owner_a = model.current_plate_ids[a];
            let owner_b = model.current_plate_ids[b];
            if owner_a == owner_b {
                continue;
            }
            let pair = if owner_a < owner_b {
                (owner_a, owner_b)
            } else {
                (owner_b, owner_a)
            };
            let position_a = topology.unit_position(sample_a);
            let position_b = topology.unit_position(*sample_b);
            let midpoint = normalize_or(add(position_a, position_b), position_a);
            let normal = normalize_or(sub(position_b, position_a), position_a);
            let tangent = normalize_or(cross(midpoint, normal), [1.0, 0.0, 0.0]);
            let velocity = |owner: u16| {
                scale(
                    cross(velocities[owner as usize], midpoint),
                    planet.radius_m / 1_000_000.0,
                )
            };
            let relative = sub(velocity(owner_b), velocity(owner_a));
            let normal_rate = dot(relative, normal);
            let shear_rate = dot(relative, tangent);
            let summary = summaries.entry(pair).or_default();
            if !summary.initialized {
                summary.sample_a = sample_a;
                summary.sample_b = *sample_b;
                summary.initialized = true;
            }
            if normal_rate < -1.0e-9 {
                summary.convergence += -normal_rate;
            } else if normal_rate > 1.0e-9 {
                summary.divergence += normal_rate;
            }
            summary.shear += shear_rate.abs();
            summary.speed_sum += normal_rate.hypot(shear_rate);
            summary.edges += 1;
        }
    }

    let mut best: Option<(f64, u16, u16, BoundaryEpochSummary)> = None;
    for ((left, right), summary) in summaries {
        if !summary.initialized || summary.edges == 0 {
            continue;
        }
        let li = left as usize;
        let ri = right as usize;
        let mean_convergence = summary.convergence / f64::from(summary.edges);
        let mean_divergence = summary.divergence / f64::from(summary.edges);
        let net_convergence = (mean_convergence - mean_divergence * 0.65).max(0.0);
        if net_convergence <= 1.0e-6 {
            continue;
        }
        let left_cont = continental_area[li] / plate_area[li].max(1.0e-12);
        let right_cont = continental_area[ri] / plate_area[ri].max(1.0e-12);
        let (keep, remove, remove_cont) = if left_cont > right_cont + 0.05 {
            (left, right, right_cont)
        } else if right_cont > left_cont + 0.05 {
            (right, left, left_cont)
        } else if plate_area[li] >= plate_area[ri] {
            (left, right, right_cont)
        } else {
            (right, left, left_cont)
        };
        // Prefer sustained convergence, long interfaces and oceanic/small plates as extinction
        // candidates. This is a coarse plate-loss event, not an area-balancing objective.
        let contact_weight = (f64::from(summary.edges)).sqrt();
        let removable_fraction =
            plate_area[remove as usize] / plate_area.iter().sum::<f64>().max(1.0e-12);
        let score = net_convergence
            * contact_weight
            * (1.0 + (1.0 - remove_cont) * 0.8)
            * (1.0 + (0.12 - removable_fraction).max(0.0) * 2.0);
        let candidate = (score, keep, remove, summary);
        if best
            .as_ref()
            .map(|current| {
                candidate.0 > current.0 + 1.0e-12
                    || ((candidate.0 - current.0).abs() <= 1.0e-12
                        && (candidate.1, candidate.2) < (current.1, current.2))
            })
            .unwrap_or(true)
        {
            best = Some(candidate);
        }
    }

    let Some((_score, keep, remove, _summary)) = best else {
        return false;
    };
    // The convergent boundary event was already recorded before extinction. Do not emit a
    // synthetic Capture here: material fragments may straddle owners until the final lineage
    // repartition, so attaching a capture to pre-repartition fragment metadata would invent a
    // categorical ownership event that did not actually occur as a discrete parcel transfer.

    for owner in &mut model.current_plate_ids {
        if *owner == remove {
            *owner = keep;
        }
    }

    // Compact ids after plate extinction so every subsequent epoch can index plate state densely.
    let mut remap = vec![u16::MAX; plate_count];
    let mut next = 0_u16;
    for old in 0..plate_count as u16 {
        if old == remove {
            continue;
        }
        remap[old as usize] = next;
        next += 1;
    }
    for owner in &mut model.current_plate_ids {
        *owner = remap[*owner as usize];
    }

    let keep_index = keep as usize;
    let remove_index = remove as usize;
    let keep_area = plate_area[keep_index];
    let remove_area = plate_area[remove_index];
    let merged_area = (keep_area + remove_area).max(1.0e-12);
    let merged_velocity = [
        (velocities[keep_index][0] * keep_area + velocities[remove_index][0] * remove_area)
            / merged_area,
        (velocities[keep_index][1] * keep_area + velocities[remove_index][1] * remove_area)
            / merged_area,
        (velocities[keep_index][2] * keep_area + velocities[remove_index][2] * remove_area)
            / merged_area,
    ];
    let mut compact_velocities = Vec::with_capacity(plate_count - 1);
    for old in 0..plate_count {
        if old == remove_index {
            continue;
        }
        if old == keep_index {
            compact_velocities.push(merged_velocity);
        } else {
            compact_velocities.push(velocities[old]);
        }
    }
    *velocities = compact_velocities;
    model.metrics.modern_plate_count = next;
    true
}

fn split_fragments_at_final_boundaries<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
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
            model.fragments[fragment_id as usize].current_plate_id = owner;
            continue;
        }
        if model.fragments.len() + by_owner.len() >= usize::from(u16::MAX) {
            return Err(WorldgenError::InvalidLithosphere(
                "forward plate evolution exhausted fragment id capacity",
            ));
        }
        for (owner, samples) in by_owner {
            let child_id = model.fragments.len() as u16;
            let area = samples
                .iter()
                .map(|sample| topology.area_steradians(*sample))
                .sum::<f64>();
            let seed_sample = samples[0];
            model.fragments.push(CrustFragment {
                id: child_id,
                parent_fragment_id: Some(parent.id),
                origin_plate_id: parent.origin_plate_id,
                current_plate_id: owner,
                seed_sample,
                birth_age_myr: parent.birth_age_myr,
                capture_age_myr: parent.capture_age_myr,
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
        }
    }
    Ok(())
}


fn refresh_material_metrics<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
) {
    let mut total_area = 0.0_f64;
    let mut continental_area = 0.0_f64;
    let mut transitional_area = 0.0_f64;
    let mut oceanic_area = 0.0_f64;
    let mut oceanic_age_area = 0.0_f64;
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let area = topology.area_steradians(sample);
        total_area += area;
        match model.crust_kind[index] {
            value if value == CrustKind::Continental as u8 => continental_area += area,
            value if value == CrustKind::Transitional as u8 => transitional_area += area,
            _ => {
                oceanic_area += area;
                oceanic_age_area += area * f64::from(model.crust_birth_age_myr[index]);
            }
        }
    }
    let total_area = total_area.max(1.0e-12);
    model.metrics.continental_area_fraction = continental_area / total_area;
    model.metrics.transitional_area_fraction = transitional_area / total_area;
    model.metrics.oceanic_area_fraction = oceanic_area / total_area;
    model.metrics.mean_oceanic_age_myr = if oceanic_area > 0.0 {
        oceanic_age_area / oceanic_area
    } else {
        0.0
    };
}

fn forward_history_hash(model: &HistoricalLithosphereModel, stage_seed: u64) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, FORWARD_PLATE_NAMESPACE.as_bytes());
    hash = fnv_update(hash, &stage_seed.to_le_bytes());
    hash = fnv_update(hash, &model.metrics.history_hash.to_le_bytes());
    for values in [
        &model.origin_plate_ids,
        &model.fragment_ids,
        &model.current_plate_ids,
    ] {
        for value in values {
            hash = fnv_update(hash, &value.to_le_bytes());
        }
    }
    for velocity in &model.current_plate_angular_velocities_rad_per_myr {
        for component in velocity {
            hash = fnv_update(hash, &component.to_bits().to_le_bytes());
        }
    }
    for weakness in &model.lithospheric_weakness_index {
        hash = fnv_update(hash, &weakness.to_bits().to_le_bytes());
    }
    hash = fnv_update(hash, &model.crust_kind);
    for age in &model.crust_birth_age_myr {
        hash = fnv_update(hash, &age.to_bits().to_le_bytes());
    }
    for event in &model.events {
        hash = fnv_update(hash, &event.id.to_le_bytes());
        hash = fnv_update(hash, &[event.kind as u8, event.epoch]);
        hash = fnv_update(hash, &event.age_myr.to_bits().to_le_bytes());
        hash = fnv_update(hash, &event.fragment_a.to_le_bytes());
        hash = fnv_update(hash, &event.fragment_b.to_le_bytes());
        hash = fnv_update(hash, &event.geometry_sample_a.to_le_bytes());
        hash = fnv_update(hash, &event.geometry_sample_b.to_le_bytes());
    }
    hash
}

fn validate_forward_state<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
) -> Result<(), WorldgenError> {
    let count = topology.sample_count() as usize;
    if model.origin_plate_ids.len() != count
        || model.fragment_ids.len() != count
        || model.current_plate_ids.len() != count
        || model.crust_kind.len() != count
        || model.crust_birth_age_myr.len() != count
        || model.lithospheric_weakness_index.len() != count
        || model
            .lithospheric_weakness_index
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        || model.current_plate_angular_velocities_rad_per_myr.len()
            != model.metrics.modern_plate_count as usize
        || model
            .current_plate_angular_velocities_rad_per_myr
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
    {
        return Err(WorldgenError::InvalidTectonics(
            "forward plate evolution produced incomplete material rasters",
        ));
    }
    let plate_count = model.metrics.modern_plate_count as usize;
    let mut samples = vec![0usize; plate_count];
    for sample in 0..count {
        let owner = model.current_plate_ids[sample] as usize;
        let origin = model.origin_plate_ids[sample] as usize;
        let fragment = model.fragment_ids[sample] as usize;
        if owner >= plate_count
            || origin >= model.ancestral_tectonics.plates.len()
            || fragment >= model.fragments.len()
        {
            return Err(WorldgenError::InvalidTectonics(
                "forward plate evolution produced an invalid material/owner id",
            ));
        }
        samples[owner] += 1;
        if model.fragments[fragment].origin_plate_id as usize != origin
            || model.fragments[fragment].current_plate_id as usize != owner
        {
            return Err(WorldgenError::InvalidLithosphere(
                "forward plate evolution lost fragment/material ownership consistency",
            ));
        }
    }
    if samples.iter().any(|count| *count == 0) {
        return Err(WorldgenError::InvalidTectonics(
            "forward plate evolution eliminated a requested modern plate",
        ));
    }
    Ok(())
}

pub fn evolve_modern_plate_geometry<T: PlanetTopology>(
    topology: &T,
    mut model: HistoricalLithosphereModel,
    seed: &str,
    target_plate_count: u16,
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
            "forward plate evolution topology does not match historical state",
        ));
    }

    if target_plate_count == 0 || target_plate_count > model.metrics.modern_plate_count {
        return Err(WorldgenError::InvalidTectonics(
            "forward plate evolution target must be within the ancestral plate count",
        ));
    }

    let stage_seed = derive_stage_seed(seed, FORWARD_PLATE_NAMESPACE);
    let mut velocities = model.current_plate_angular_velocities_rad_per_myr.clone();
    if velocities.len() != model.metrics.modern_plate_count as usize {
        return Err(WorldgenError::InvalidTectonics(
            "historical state is missing current plate kinematics",
        ));
    }
    let starting_plate_count = model.metrics.modern_plate_count;
    let merges_needed = starting_plate_count.saturating_sub(target_plate_count) as usize;
    let rift_budget = (usize::from(target_plate_count) / 8).clamp(1, 3);
    let mut splits_completed = 0usize;
    let mut merges_completed = 0usize;
    for epoch in 0..FORWARD_EPOCHS {
        let mut generation_fragments = BTreeMap::<(u8, u16, u8, u16), u16>::new();
        record_epoch_events(topology, &mut model, &velocities, epoch, planet);
        for _ in 0..SUBSTEPS_PER_EPOCH {
            advect_substep(
                topology,
                &mut model,
                &velocities,
                epoch as u8,
                &mut generation_fragments,
                SUBSTEP_MYR,
            )?;
        }

        if splits_completed < rift_budget
            && epoch >= 1
            && epoch <= FORWARD_EPOCHS.saturating_sub(3)
            && epoch % 2 == 1
            && split_one_rifting_plate(
                topology,
                &mut model,
                &mut velocities,
                epoch,
                stage_seed,
            )
        {
            splits_completed += 1;
        }

        let expected_merges = ((epoch + 1) * merges_needed) / FORWARD_EPOCHS;
        while merges_completed < expected_merges
            && model.metrics.modern_plate_count > target_plate_count
        {
            if !merge_one_converging_plate_pair(
                topology,
                &mut model,
                &mut velocities,
                epoch,
                planet,
            ) {
                break;
            }
            merges_completed += 1;
        }
    }

    // Plate births are allowed to survive for multiple epochs. Any remaining excess bodies must
    // disappear through actual convergent extinction, not by relabeling them to hit a count.
    while model.metrics.modern_plate_count > target_plate_count {
        if !merge_one_converging_plate_pair(
            topology,
            &mut model,
            &mut velocities,
            FORWARD_EPOCHS - 1,
            planet,
        ) {
            break;
        }
    }
    if model.metrics.modern_plate_count != target_plate_count {
        return Err(WorldgenError::InvalidTectonics(
            "forward plate evolution did not reach the requested present plate count through convergence",
        ));
    }

    split_fragments_at_final_boundaries(topology, &mut model)?;
    refresh_active_fragment_summaries(topology, &mut model);
    refresh_material_metrics(topology, &mut model);
    model.current_plate_angular_velocities_rad_per_myr = velocities;
    model.metrics.fragment_count = model.fragments.len() as u16;
    model.metrics.event_count = model.events.len() as u32;
    model.metrics.history_hash = forward_history_hash(&model, stage_seed);
    validate_forward_state(topology, &model)?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_icosphere, HistoricalLithosphereRequest};

    #[test]
    fn forward_evolution_moves_material_and_creates_new_spreading_crust() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let request = HistoricalLithosphereRequest::new("forward-plate-regression", 16);
        let base = crate::historical_lithosphere::generate_historical_lithosphere(
            &topology,
            &request,
            planet,
        )
        .unwrap();
        let before_owners = base.current_plate_ids.clone();
        let before_origins = base.origin_plate_ids.clone();
        let evolved = evolve_modern_plate_geometry(
            &topology,
            base,
            &request.seed,
            request.modern_plate_count,
            planet,
        )
        .unwrap();

        assert_ne!(
            before_owners, evolved.current_plate_ids,
            "forward integration did not move modern plate ownership"
        );
        assert_ne!(
            before_origins, evolved.origin_plate_ids,
            "forward integration did not advect material ancestry"
        );
        assert!(
            evolved
                .crust_birth_age_myr
                .iter()
                .zip(evolved.crust_kind.iter())
                .any(|(age, kind)| *kind != CrustKind::Continental as u8 && *age <= 0.001),
            "forward extension produced no newly created young crust"
        );
        assert!(
            evolved.events.iter().any(|event| {
                matches!(
                    event.kind,
                    HistoricalEventKind::Spreading
                        | HistoricalEventKind::Rift
                        | HistoricalEventKind::Subduction
                        | HistoricalEventKind::Collision
                )
            }),
            "forward integration recorded no active tectonic event history"
        );
    }
}
