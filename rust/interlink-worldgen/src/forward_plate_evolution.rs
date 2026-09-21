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

fn owner_angular_velocities<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
) -> Vec<[f64; 3]> {
    let plate_count = model.metrics.modern_plate_count as usize;
    let ancestral_count = model.ancestral_tectonics.plates.len();
    let mut sums = vec![[0.0_f64; 3]; plate_count];
    let mut areas = vec![0.0_f64; plate_count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let owner = model.current_plate_ids[index] as usize;
        let origin = model.origin_plate_ids[index] as usize;
        if owner >= plate_count || origin >= ancestral_count {
            continue;
        }
        let area = topology.area_steradians(sample);
        let omega = model.ancestral_tectonics.plates[origin].angular_velocity_rad_per_myr;
        for axis in 0..3 {
            sums[owner][axis] += omega[axis] * area;
        }
        areas[owner] += area;
    }
    for plate in 0..plate_count {
        if areas[plate] > 0.0 {
            for axis in 0..3 {
                sums[plate][axis] /= areas[plate];
            }
        }
    }
    sums
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
        0.020
    } else if kind == CrustKind::Transitional as u8 {
        0.012
    } else {
        0.004 + 0.004 * (1.0 - (f64::from(age_myr) / 220.0).clamp(0.0, 1.0))
    }
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
    dt_myr: f64,
) {
    let count = topology.sample_count() as usize;
    let plate_count = model.metrics.modern_plate_count as usize;
    let velocities = owner_angular_velocities(topology, model);
    let cores = plate_core_samples(topology, &model.current_plate_ids, plate_count);

    let old_origin = model.origin_plate_ids.clone();
    let old_fragment = model.fragment_ids.clone();
    let old_owner = model.current_plate_ids.clone();
    let old_kind = model.crust_kind.clone();
    let old_age = model.crust_birth_age_myr.clone();

    let mut selected_source = vec![usize::MAX; count];
    let mut selected_score = vec![f64::NEG_INFINITY; count];
    let mut mapped_core_destination = vec![None; plate_count];

    for source in 0..topology.sample_count() {
        let source_index = source as usize;
        let owner = old_owner[source_index] as usize;
        if owner >= plate_count {
            continue;
        }
        let target = rotate_by_angular_velocity(
            topology.unit_position(source),
            velocities[owner],
            dt_myr,
        );
        let destination = nearest_sample(topology, target, source) as usize;
        let alignment = dot(topology.unit_position(destination as u32), target);
        let score = alignment + material_survival_bias(old_kind[source_index], old_age[source_index]);
        if score > selected_score[destination] + 1.0e-14
            || ((score - selected_score[destination]).abs() <= 1.0e-14
                && source_index < selected_source[destination])
        {
            selected_score[destination] = score;
            selected_source[destination] = source_index;
        }
        if cores[owner] == Some(source) {
            mapped_core_destination[owner] = Some(destination);
        }
    }

    let mut new_origin = vec![u16::MAX; count];
    let mut new_fragment = vec![u16::MAX; count];
    let mut new_owner = vec![u16::MAX; count];
    let mut new_kind = vec![CrustKind::Oceanic as u8; count];
    let mut new_age = vec![0.0_f32; count];

    for destination in 0..count {
        let source = selected_source[destination];
        if source == usize::MAX {
            continue;
        }
        new_origin[destination] = old_origin[source];
        new_fragment[destination] = old_fragment[source];
        new_owner[destination] = old_owner[source];
        new_kind[destination] = old_kind[source];
        new_age[destination] = if old_kind[source] == CrustKind::Oceanic as u8 {
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
                    previous_fragment[*neighbor],
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
            new_age[index] = 0.0;
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
        counts[plate] += 1;
    }

    repair_plate_connectivity(topology, &mut new_owner, plate_count);

    model.origin_plate_ids = new_origin;
    model.fragment_ids = new_fragment;
    model.current_plate_ids = new_owner;
    model.crust_kind = new_kind;
    model.crust_birth_age_myr = new_age;
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
    epoch: usize,
    planet: PlanetPhysicalParameters,
) {
    let velocities = owner_angular_velocities(topology, model);
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


fn merge_one_converging_plate_pair<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
    epoch: usize,
    planet: PlanetPhysicalParameters,
) -> bool {
    let plate_count = model.metrics.modern_plate_count as usize;
    if plate_count <= 1 {
        return false;
    }
    let velocities = owner_angular_velocities(topology, model);
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

    let Some((_score, keep, remove, summary)) = best else {
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
            let previous_owner = model.fragments[fragment_id as usize].current_plate_id;
            model.fragments[fragment_id as usize].current_plate_id = owner;
            if owner != previous_owner {
                let geometry = model.fragments[fragment_id as usize].seed_sample;
                model.fragments[fragment_id as usize].capture_age_myr = Some(0.0);
                model.events.push(HistoricalTectonicEvent {
                    id: model.events.len() as u32,
                    kind: HistoricalEventKind::Capture,
                    epoch: crate::HISTORICAL_EPOCH_COUNT - 1,
                    age_myr: 0.0,
                    plate_a: parent.origin_plate_id,
                    plate_b: parent.origin_plate_id,
                    fragment_a: fragment_id,
                    fragment_b: fragment_id,
                    displacement_km: 0.0,
                    strength: 0.20,
                    geometry_sample_a: geometry,
                    geometry_sample_b: geometry,
                });
            }
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
                capture_age_myr: if owner != parent.current_plate_id {
                    Some(0.0)
                } else {
                    parent.capture_age_myr
                },
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
                    epoch: crate::HISTORICAL_EPOCH_COUNT - 1,
                    age_myr: 0.0,
                    plate_a: parent.origin_plate_id,
                    plate_b: parent.origin_plate_id,
                    fragment_a: parent.id,
                    fragment_b: child_id,
                    displacement_km: 0.0,
                    strength: 0.24,
                    geometry_sample_a: parent.seed_sample,
                    geometry_sample_b: seed_sample,
                });
            }
        }
    }
    Ok(())
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
    let starting_plate_count = model.metrics.modern_plate_count;
    let merges_needed = starting_plate_count.saturating_sub(target_plate_count) as usize;
    let mut merges_completed = 0usize;
    for epoch in 0..FORWARD_EPOCHS {
        record_epoch_events(topology, &mut model, epoch, planet);
        for _ in 0..SUBSTEPS_PER_EPOCH {
            advect_substep(topology, &mut model, SUBSTEP_MYR);
        }
        let expected_merges = ((epoch + 1) * merges_needed) / FORWARD_EPOCHS;
        while merges_completed < expected_merges
            && model.metrics.modern_plate_count > target_plate_count
        {
            if !merge_one_converging_plate_pair(topology, &mut model, epoch, planet) {
                break;
            }
            merges_completed += 1;
        }
    }
    if model.metrics.modern_plate_count != target_plate_count {
        return Err(WorldgenError::InvalidTectonics(
            "forward plate evolution did not reach the requested present plate count through convergence",
        ));
    }

    split_fragments_at_final_boundaries(topology, &mut model)?;
    refresh_active_fragment_summaries(topology, &mut model);
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
