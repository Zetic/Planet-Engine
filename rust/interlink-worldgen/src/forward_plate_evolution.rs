use crate::{
    derive_stage_seed, CrustFragment, CrustKind, HistoricalEventKind,
    HistoricalLithosphereModel, HistoricalTectonicEvent, PlanetPhysicalParameters, PlanetTopology,
    WorldgenError, MAX_TECTONIC_PLATES,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const FORWARD_PLATE_NAMESPACE: &str = "worldgen:geology:forward-plate-evolution:v1";
const FORWARD_EPOCHS: usize = 8;
const SUBSTEPS_PER_EPOCH: usize = 4;
const EPOCH_DURATION_MYR: f64 = 20.0;
const SUBSTEP_MYR: f64 = EPOCH_DURATION_MYR / SUBSTEPS_PER_EPOCH as f64;
const RIFT_STRAIN_NUCLEATION_MYR: f32 = 26.0;
const RIFT_STRAIN_RELIEF_FACTOR: f32 = 0.22;
const FORWARD_TRANSITION_MATURATION_MYR: f32 = 30.0;
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

fn interpolate_owned_scalar<T: PlanetTopology>(
    topology: &T,
    target: [f64; 3],
    source: u32,
    owner: u16,
    kind: u8,
    owners: &[u16],
    kinds: &[u8],
    values: &[f32],
) -> f32 {
    let source_lengths = topology.neighbor_arc_lengths_rad(source);
    let local_scale = if source_lengths.is_empty() {
        0.05
    } else {
        source_lengths.iter().copied().sum::<f64>() / source_lengths.len() as f64
    }
    .max(1.0e-6);
    let sigma = local_scale * 0.78;

    let mut samples = Vec::<u32>::with_capacity(topology.neighbors(source).len() + 1);
    samples.push(source);
    samples.extend(
        topology
            .neighbors(source)
            .iter()
            .copied()
            .filter(|sample| {
                let index = *sample as usize;
                owners[index] == owner && kinds[index] == kind
            }),
    );

    let mut weighted = 0.0_f64;
    let mut weight_sum = 0.0_f64;
    for sample in samples {
        let index = sample as usize;
        if owners[index] != owner || kinds[index] != kind {
            continue;
        }
        let distance = dot(topology.unit_position(sample), target)
            .clamp(-1.0, 1.0)
            .acos();
        let normalized = distance / sigma;
        let weight = (-0.5 * normalized * normalized).exp();
        weighted += f64::from(values[index]) * weight;
        weight_sum += weight;
    }
    if weight_sum > 1.0e-12 {
        (weighted / weight_sum) as f32
    } else {
        values[source as usize]
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

#[derive(Clone, Copy, Default)]
struct ConnectivityResolution {
    accreted_samples: usize,
    microplate_births: usize,
}

fn resolve_plate_connectivity<T: PlanetTopology>(
    topology: &T,
    owners: &mut [u16],
    velocities: &mut Vec<[f64; 3]>,
    planet: PlanetPhysicalParameters,
) -> ConnectivityResolution {
    let initial_plate_count = velocities.len();
    let mut resolution = ConnectivityResolution::default();

    // A rigid plate should occupy one connected surface domain. If advection/subduction cuts a
    // plate into disconnected pieces, do not silently repaint every detached component onto the
    // largest neighboring owner. Small scraps may physically accrete, but a substantial detached
    // remnant becomes its own microplate and keeps the parent's instantaneous Euler motion.
    for plate in 0..initial_plate_count {
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

        components.sort_by(|left, right| {
            right
                .len()
                .cmp(&left.len())
                .then_with(|| left[0].cmp(&right[0]))
        });
        let primary_size = components[0].len().max(1);

        for component in components.into_iter().skip(1) {
            let mut contacts = BTreeMap::<u16, (usize, f64)>::new();
            for sample in &component {
                let sample_position = topology.unit_position(*sample);
                for neighbor in topology.neighbors(*sample) {
                    let ni = *neighbor as usize;
                    let candidate = owners[ni];
                    if candidate == plate_id || candidate as usize >= velocities.len() {
                        continue;
                    }
                    let neighbor_position = topology.unit_position(*neighbor);
                    let midpoint = normalize_or(add(sample_position, neighbor_position), sample_position);
                    let normal = normalize_or(sub(neighbor_position, sample_position), sample_position);
                    let source_velocity = scale(
                        cross(velocities[plate], midpoint),
                        planet.radius_m / 1_000_000.0,
                    );
                    let candidate_velocity = scale(
                        cross(velocities[candidate as usize], midpoint),
                        planet.radius_m / 1_000_000.0,
                    );
                    let normal_rate = dot(sub(candidate_velocity, source_velocity), normal);
                    let entry = contacts.entry(candidate).or_insert((0, 0.0));
                    entry.0 += 1;
                    if normal_rate < 0.0 {
                        entry.1 += -normal_rate;
                    }
                }
            }

            let best_convergent = contacts
                .iter()
                .filter(|(_, (_, convergence))| *convergence > 1.0e-9)
                .max_by(|left, right| {
                    left.1
                        .1
                        .total_cmp(&right.1.1)
                        .then_with(|| left.1.0.cmp(&right.1.0))
                        .then_with(|| right.0.cmp(left.0))
                })
                .map(|(owner, _)| *owner);
            let best_contact = contacts
                .iter()
                .max_by(|left, right| {
                    left.1
                        .0
                        .cmp(&right.1.0)
                        .then_with(|| left.1.1.total_cmp(&right.1.1))
                        .then_with(|| right.0.cmp(left.0))
                })
                .map(|(owner, _)| *owner);

            let substantial_remnant =
                component.len() >= 16 && component.len().saturating_mul(4) >= primary_size;
            let recipient = if substantial_remnant {
                // A sizeable detached lithospheric body is not erased just because one edge is
                // convergent. Preserve it as a microplate; only genuinely small scraps accrete
                // onto a neighboring plate.
                None
            } else if let Some(owner) = best_convergent {
                Some(owner)
            } else {
                best_contact
            };

            if let Some(recipient) = recipient {
                for sample in component {
                    owners[sample as usize] = recipient;
                    resolution.accreted_samples += 1;
                }
                continue;
            }

            if velocities.len() < usize::from(MAX_TECTONIC_PLATES) {
                let child = velocities.len() as u16;
                velocities.push(velocities[plate]);
                for sample in component {
                    owners[sample as usize] = child;
                }
                resolution.microplate_births += 1;
            } else if let Some(recipient) = best_contact {
                for sample in component {
                    owners[sample as usize] = recipient;
                    resolution.accreted_samples += 1;
                }
            }
        }
    }

    resolution
}
fn gap_is_divergent<T: PlanetTopology>(
    topology: &T,
    sample: u32,
    choices: &[usize],
    owners: &[u16],
    velocities: &[[f64; 3]],
) -> bool {
    let position = topology.unit_position(sample);
    for left in 0..choices.len() {
        for right in (left + 1)..choices.len() {
            let left_index = choices[left];
            let right_index = choices[right];
            let owner_left = owners[left_index] as usize;
            let owner_right = owners[right_index] as usize;
            if owner_left == owner_right
                || owner_left >= velocities.len()
                || owner_right >= velocities.len()
            {
                continue;
            }
            let left_position = topology.unit_position(left_index as u32);
            let right_position = topology.unit_position(right_index as u32);
            let normal = normalize_or(sub(right_position, left_position), position);
            let left_velocity = cross(velocities[owner_left], position);
            let right_velocity = cross(velocities[owner_right], position);
            if dot(sub(right_velocity, left_velocity), normal) > 1.0e-8 {
                return true;
            }
        }
    }
    false
}

fn compact_extinct_plates(
    owners: &mut [u16],
    velocities: &mut Vec<[f64; 3]>,
) -> usize {
    let plate_count = velocities.len();
    let mut active = vec![false; plate_count];
    for owner in owners.iter().copied() {
        let index = owner as usize;
        if index < plate_count {
            active[index] = true;
        }
    }
    if active.iter().all(|value| *value) {
        return 0;
    }

    let mut remap = vec![u16::MAX; plate_count];
    let mut compact_velocities = Vec::with_capacity(plate_count);
    for old in 0..plate_count {
        if !active[old] {
            continue;
        }
        remap[old] = compact_velocities.len() as u16;
        compact_velocities.push(velocities[old]);
    }
    for owner in owners {
        *owner = remap[*owner as usize];
    }
    let removed = plate_count.saturating_sub(compact_velocities.len());
    *velocities = compact_velocities;
    removed
}

#[derive(Clone, Copy)]
struct ConvergentConsumptionProposal {
    strength: f64,
    winner: u16,
    donor: usize,
}

fn convergent_consumption_priority(kind: u8, age_myr: f32, weakness: f32) -> f64 {
    if kind == CrustKind::Oceanic as u8 {
        3.0 + (f64::from(age_myr) / 220.0).clamp(0.0, 1.0)
    } else if kind == CrustKind::Transitional as u8 {
        1.4 + 0.5 * f64::from(weakness).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn consume_convergent_boundary_band<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
    velocities: &mut Vec<[f64; 3]>,
    generation_fragments: &mut BTreeMap<(u8, u16, u8, u16), u16>,
    extensional_strain_myr: &mut [f32],
    planet: PlanetPhysicalParameters,
    dt_myr: f64,
) {
    let plate_count = model.metrics.modern_plate_count as usize;
    if plate_count <= 1 || velocities.len() != plate_count {
        return;
    }

    let count = topology.sample_count() as usize;
    let mut plate_samples = vec![0usize; plate_count];
    for owner in &model.current_plate_ids {
        let index = *owner as usize;
        if index < plate_count {
            plate_samples[index] += 1;
        }
    }

    let mut proposals = vec![None::<ConvergentConsumptionProposal>; count];
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

            let position_a = topology.unit_position(sample_a);
            let position_b = topology.unit_position(*sample_b);
            let midpoint = normalize_or(add(position_a, position_b), position_a);
            let normal = normalize_or(sub(position_b, position_a), position_a);
            let velocity_a = scale(
                cross(velocities[owner_a as usize], midpoint),
                planet.radius_m / 1_000_000.0,
            );
            let velocity_b = scale(
                cross(velocities[owner_b as usize], midpoint),
                planet.radius_m / 1_000_000.0,
            );
            let normal_rate = dot(sub(velocity_b, velocity_a), normal);
            if normal_rate >= -1.0e-9 {
                continue;
            }

            let arc_rad = dot(position_a, position_b).clamp(-1.0, 1.0).acos();
            let edge_km = (arc_rad * planet.radius_m / 1000.0).max(1.0);
            let convergence_km = -normal_rate * dt_myr * 1000.0;
            let convergence_fraction = convergence_km / edge_km;
            if convergence_fraction < 0.16 {
                continue;
            }

            let priority_a = convergent_consumption_priority(
                model.crust_kind[a],
                model.crust_birth_age_myr[a],
                model.lithospheric_weakness_index[a],
            );
            let priority_b = convergent_consumption_priority(
                model.crust_kind[b],
                model.crust_birth_age_myr[b],
                model.lithospheric_weakness_index[b],
            );
            if priority_a <= 0.0 && priority_b <= 0.0 {
                // Continental collision shortens/welds rather than subducting a surface band.
                continue;
            }

            let (victim, donor, winner, material_priority) =
                if priority_a > priority_b + 1.0e-9 {
                    (a, b, owner_b, priority_a)
                } else if priority_b > priority_a + 1.0e-9 {
                    (b, a, owner_a, priority_b)
                } else if model.crust_birth_age_myr[a] > model.crust_birth_age_myr[b] + 1.0e-6 {
                    (a, b, owner_b, priority_a)
                } else if model.crust_birth_age_myr[b] > model.crust_birth_age_myr[a] + 1.0e-6 {
                    (b, a, owner_a, priority_b)
                } else if owner_a > owner_b {
                    (a, b, owner_b, priority_a)
                } else {
                    (b, a, owner_a, priority_b)
                };

            let strength = convergence_fraction.min(2.5) * material_priority;
            let candidate = ConvergentConsumptionProposal {
                strength,
                winner,
                donor,
            };
            let replace = proposals[victim]
                .map(|current| {
                    candidate.strength > current.strength + 1.0e-12
                        || ((candidate.strength - current.strength).abs() <= 1.0e-12
                            && candidate.winner < current.winner)
                })
                .unwrap_or(true);
            if replace {
                proposals[victim] = Some(candidate);
            }
        }
    }

    let mut by_plate = vec![Vec::<(usize, ConvergentConsumptionProposal)>::new(); plate_count];
    for (sample, proposal) in proposals.into_iter().enumerate() {
        let Some(proposal) = proposal else { continue };
        let victim_plate = model.current_plate_ids[sample] as usize;
        if victim_plate < plate_count && proposal.winner as usize != victim_plate {
            by_plate[victim_plate].push((sample, proposal));
        }
    }

    // Surface subduction removes the victim parcel from the exposed lithosphere. Preserve a
    // snapshot so simultaneous boundary-band consumption can advance the overriding material into
    // the vacated surface cell without reading fields already modified earlier in this substep.
    let previous_origin = model.origin_plate_ids.clone();
    let previous_fragment = model.fragment_ids.clone();
    let previous_kind = model.crust_kind.clone();
    let previous_age = model.crust_birth_age_myr.clone();
    let previous_weakness = model.lithospheric_weakness_index.clone();
    let previous_strain = extensional_strain_myr.to_vec();

    let mut consumed = 0usize;
    for plate in 0..plate_count {
        if by_plate[plate].is_empty() {
            continue;
        }
        by_plate[plate].sort_by(|left, right| {
            right
                .1
                .strength
                .total_cmp(&left.1.strength)
                .then_with(|| left.0.cmp(&right.0))
        });
        // Consume a finite boundary band per integration step. This lets persistent convergence
        // remove oceanic plates over geological time without instant whole-plate deletion.
        let limit = ((plate_samples[plate] as f64 * 0.045).ceil() as usize)
            .clamp(1, plate_samples[plate].saturating_sub(1).max(1));
        for (sample, proposal) in by_plate[plate].iter().take(limit) {
            if model.current_plate_ids[*sample] as usize != plate {
                continue;
            }
            model.current_plate_ids[*sample] = proposal.winner;

            let victim_kind = previous_kind[*sample];
            let donor_kind = previous_kind[proposal.donor];
            if victim_kind != CrustKind::Continental as u8
                && donor_kind == CrustKind::Continental as u8
            {
                // A 2-D surface raster cannot represent the descending slab and the overriding
                // forearc as two stacked lithospheres. Do not solve that limitation by cloning
                // continental basement or by turning every consumed trench cell into new
                // transitional crust. Transfer plate ownership while retaining the exposed
                // non-continental veneer and its age/provenance; a future 3-D slab reservoir can
                // remove the buried parcel explicitly without corrupting surface hypsometry.
                model.origin_plate_ids[*sample] = previous_origin[*sample];
                model.fragment_ids[*sample] = previous_fragment[*sample];
                model.crust_kind[*sample] = victim_kind;
                model.crust_birth_age_myr[*sample] = previous_age[*sample];
                model.lithospheric_weakness_index[*sample] =
                    previous_weakness[*sample].max(0.48);
                extensional_strain_myr[*sample] = previous_strain[*sample] * 0.70;
            } else {
                model.origin_plate_ids[*sample] = previous_origin[proposal.donor];
                model.fragment_ids[*sample] = previous_fragment[proposal.donor];
                model.crust_kind[*sample] = donor_kind;
                model.crust_birth_age_myr[*sample] = previous_age[proposal.donor];
                model.lithospheric_weakness_index[*sample] =
                    previous_weakness[proposal.donor];
                extensional_strain_myr[*sample] = previous_strain[proposal.donor] * 0.65;
            }
            consumed += 1;
        }
    }

    if consumed == 0 {
        return;
    }

    let connectivity = resolve_plate_connectivity(
        topology,
        &mut model.current_plate_ids,
        velocities,
        planet,
    );
    model.metrics.detached_accretion_sample_count = model
        .metrics
        .detached_accretion_sample_count
        .saturating_add(connectivity.accreted_samples as u32);
    model.metrics.detached_microplate_birth_count = model
        .metrics
        .detached_microplate_birth_count
        .saturating_add(connectivity.microplate_births as u16);
    if connectivity.microplate_births > 0 {
        generation_fragments.clear();
    }
    let extinct = compact_extinct_plates(
        &mut model.current_plate_ids,
        velocities,
    );
    model.metrics.modern_plate_count = velocities.len() as u16;
    model.metrics.convergent_consumed_sample_count = model
        .metrics
        .convergent_consumed_sample_count
        .saturating_add(consumed as u32);
    if extinct > 0 {
        generation_fragments.clear();
        model.metrics.natural_extinction_count = model
            .metrics
            .natural_extinction_count
            .saturating_add(extinct as u16);
    }
}

fn advect_substep<T: PlanetTopology>(
    topology: &T,
    model: &mut HistoricalLithosphereModel,
    velocities: &mut Vec<[f64; 3]>,
    epoch: u8,
    generation_fragments: &mut BTreeMap<(u8, u16, u8, u16), u16>,
    extensional_strain_myr: &mut Vec<f32>,
    planet: PlanetPhysicalParameters,
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
    let old_strain = extensional_strain_myr.clone();
    debug_assert_eq!(old_strain.len(), count);

    let mut new_origin = vec![u16::MAX; count];
    let mut new_fragment = vec![u16::MAX; count];
    let mut new_owner = vec![u16::MAX; count];
    let mut new_kind = vec![CrustKind::Oceanic as u8; count];
    let mut new_age = vec![0.0_f32; count];
    let mut new_weakness = vec![0.0_f32; count];
    let mut new_strain = vec![0.0_f32; count];
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
        let preimage = rotate_by_angular_velocity(
            destination_position,
            velocities[plate as usize],
            -dt_myr,
        );
        new_origin[destination_index] = old_origin[source];
        new_fragment[destination_index] = old_fragment[source];
        new_owner[destination_index] = plate;
        new_kind[destination_index] = old_kind[source];
        let interpolated_age = interpolate_owned_scalar(
            topology,
            preimage,
            source as u32,
            plate,
            old_kind[source],
            &old_owner,
            &old_kind,
            &old_age,
        );
        new_weakness[destination_index] = interpolate_owned_scalar(
            topology,
            preimage,
            source as u32,
            plate,
            old_kind[source],
            &old_owner,
            &old_kind,
            &old_weakness,
        )
        .clamp(0.0, 1.0);
        new_strain[destination_index] = interpolate_owned_scalar(
            topology,
            preimage,
            source as u32,
            plate,
            old_kind[source],
            &old_owner,
            &old_kind,
            &old_strain,
        )
        .clamp(0.0, 160.0);
        new_age[destination_index] = if old_kind[source] == CrustKind::Oceanic as u8
            || (old_kind[source] == CrustKind::Transitional as u8
                && interpolated_age < 100.0)
        {
            (interpolated_age + dt_myr as f32).clamp(0.0, 220.0)
        } else {
            interpolated_age
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
        let previous_age = new_age.clone();
        let previous_weakness = new_weakness.clone();
        let previous_strain = new_strain.clone();
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
            // Assign the uncovered cell to the neighboring plate whose inverse-advected
            // preimage best lands on that margin. This is a geometric/kinematic decision only:
            // provenance and genealogy are not allowed to decide physical ownership.
            let sample_position = topology.unit_position(sample);
            choices.sort_by(|left, right| {
                let score = |neighbor: usize| {
                    let owner = previous_owner[neighbor] as usize;
                    if owner >= velocities.len() {
                        return f64::INFINITY;
                    }
                    let preimage =
                        rotate_by_angular_velocity(sample_position, velocities[owner], -dt_myr);
                    dot(preimage, topology.unit_position(neighbor as u32))
                        .clamp(-1.0, 1.0)
                        .acos()
                };
                score(*left)
                    .total_cmp(&score(*right))
                    .then_with(|| previous_owner[*left].cmp(&previous_owner[*right]))
                    .then_with(|| left.cmp(right))
            });
            let donor = choices[0];
            let divergent_gap =
                gap_is_divergent(topology, sample, &choices, &previous_owner, velocities);
            new_owner[index] = previous_owner[donor];
            new_origin[index] = previous_origin[donor];
            new_fragment[index] = previous_fragment[donor];

            if divergent_gap {
                let has_oceanic_margin = choices
                    .iter()
                    .any(|neighbor| previous_kind[*neighbor] == CrustKind::Oceanic as u8);
                let has_transitional_margin = choices
                    .iter()
                    .any(|neighbor| previous_kind[*neighbor] == CrustKind::Transitional as u8);
                new_kind[index] = if has_oceanic_margin || has_transitional_margin {
                    CrustKind::Oceanic as u8
                } else {
                    CrustKind::Transitional as u8
                };
                // Newly opened lithosphere is mechanically weak/hot. This is physical state
                // attached to generated material, not a property selected by its genealogy id.
                new_weakness[index] = if new_kind[index] == CrustKind::Oceanic as u8 {
                    0.48
                } else {
                    0.74
                };
                new_strain[index] = 0.0;
                new_age[index] = 0.0;
                generated[index] = true;
            } else {
                // A semi-Lagrangian raster can leave a one-cell sampling hole even inside a rigid
                // or convergent domain. Close that numerical hole by extending transported
                // material; only a kinematically divergent gap is allowed to manufacture crust.
                new_kind[index] = previous_kind[donor];
                new_age[index] = previous_age[donor];
                new_weakness[index] = previous_weakness[donor];
                new_strain[index] = previous_strain[donor];
            }
            changed += 1;
        }
        if changed == 0 {
            break;
        }
        remaining = remaining.saturating_sub(changed);
    }

    let connectivity =
        resolve_plate_connectivity(topology, &mut new_owner, velocities, planet);
    model.metrics.detached_accretion_sample_count = model
        .metrics
        .detached_accretion_sample_count
        .saturating_add(connectivity.accreted_samples as u32);
    model.metrics.detached_microplate_birth_count = model
        .metrics
        .detached_microplate_birth_count
        .saturating_add(connectivity.microplate_births as u16);
    if connectivity.microplate_births > 0 {
        generation_fragments.clear();
    }
    if new_owner.iter().any(|owner| *owner == u16::MAX) {
        return Err(WorldgenError::InvalidTectonics(
            "forward transport left an unresolved surface ownership hole",
        ));
    }

    // A plate that has no remaining surface samples after overlap resolution is genuinely extinct.
    // Compact it out of the evolving kinematic state instead of resurrecting a one-cell core or
    // waiting for a later whole-plate relabel operation.
    let extinct = compact_extinct_plates(&mut new_owner, velocities);
    if extinct > 0 {
        generation_fragments.clear();
        model.metrics.modern_plate_count = velocities.len() as u16;
        model.metrics.natural_extinction_count = model
            .metrics
            .natural_extinction_count
            .saturating_add(extinct as u16);
    }

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
    model.metrics.modern_plate_count = velocities.len() as u16;
    model.lithospheric_weakness_index = new_weakness;
    *extensional_strain_myr = new_strain;
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



#[derive(Clone)]
struct FlowEdge {
    to: usize,
    rev: usize,
    capacity: u64,
}

fn add_flow_edge(graph: &mut [Vec<FlowEdge>], from: usize, to: usize, capacity: u64) {
    let forward_rev = graph[to].len();
    let reverse_rev = graph[from].len();
    graph[from].push(FlowEdge {
        to,
        rev: forward_rev,
        capacity,
    });
    graph[to].push(FlowEdge {
        to: from,
        rev: reverse_rev,
        capacity: 0,
    });
}

fn add_undirected_flow_edge(
    graph: &mut [Vec<FlowEdge>],
    left: usize,
    right: usize,
    capacity: u64,
) {
    add_flow_edge(graph, left, right, capacity);
    add_flow_edge(graph, right, left, capacity);
}

fn flow_levels(graph: &[Vec<FlowEdge>], source: usize, sink: usize) -> Option<Vec<i32>> {
    let mut level = vec![-1_i32; graph.len()];
    let mut queue = VecDeque::from([source]);
    level[source] = 0;
    while let Some(node) = queue.pop_front() {
        for edge in &graph[node] {
            if edge.capacity > 0 && level[edge.to] < 0 {
                level[edge.to] = level[node] + 1;
                queue.push_back(edge.to);
            }
        }
    }
    (level[sink] >= 0).then_some(level)
}

fn flow_dfs(
    node: usize,
    sink: usize,
    pushed: u64,
    level: &[i32],
    next_edge: &mut [usize],
    graph: &mut [Vec<FlowEdge>],
) -> u64 {
    if pushed == 0 {
        return 0;
    }
    if node == sink {
        return pushed;
    }
    while next_edge[node] < graph[node].len() {
        let edge_index = next_edge[node];
        let to = graph[node][edge_index].to;
        let reverse = graph[node][edge_index].rev;
        let capacity = graph[node][edge_index].capacity;
        if capacity > 0 && level[to] == level[node] + 1 {
            let sent = flow_dfs(
                to,
                sink,
                pushed.min(capacity),
                level,
                next_edge,
                graph,
            );
            if sent > 0 {
                graph[node][edge_index].capacity -= sent;
                graph[to][reverse].capacity += sent;
                return sent;
            }
        }
        next_edge[node] += 1;
    }
    0
}

fn max_flow(graph: &mut [Vec<FlowEdge>], source: usize, sink: usize) {
    const FLOW_INF: u64 = 1_u64 << 60;
    while let Some(level) = flow_levels(graph, source, sink) {
        let mut next_edge = vec![0_usize; graph.len()];
        loop {
            let pushed = flow_dfs(source, sink, FLOW_INF, &level, &mut next_edge, graph);
            if pushed == 0 {
                break;
            }
        }
    }
}

fn spatial_side_connected<T: PlanetTopology>(
    topology: &T,
    plate: u16,
    owners: &[u16],
    side: &[bool],
    expected_side: bool,
    expected_count: usize,
) -> bool {
    let Some(start) = (0..topology.sample_count()).find(|sample| {
        let index = *sample as usize;
        owners[index] == plate && side[index] == expected_side
    }) else {
        return false;
    };
    let mut seen = BTreeSet::<u32>::new();
    let mut queue = VecDeque::from([start]);
    seen.insert(start);
    while let Some(sample) = queue.pop_front() {
        for neighbor in topology.neighbors(sample) {
            let index = *neighbor as usize;
            if owners[index] == plate
                && side[index] == expected_side
                && seen.insert(*neighbor)
            {
                queue.push_back(*neighbor);
            }
        }
    }
    seen.len() == expected_count
}

fn weak_corridor_partition<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    candidate: RiftCandidate,
    samples: &[u32],
) -> Option<(Vec<bool>, u32, u32, f64, f64)> {
    if samples.len() < 48 {
        return None;
    }
    let mut local_index = vec![usize::MAX; topology.sample_count() as usize];
    for (local, sample) in samples.iter().copied().enumerate() {
        local_index[sample as usize] = local;
    }

    let mut projections = samples
        .iter()
        .copied()
        .map(|sample| {
            (
                dot(topology.unit_position(sample), candidate.plane_normal),
                sample,
            )
        })
        .collect::<Vec<_>>();
    projections.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
    });
    let terminal_count = (samples.len() / 10).clamp(3, 24);
    if terminal_count * 2 >= samples.len() {
        return None;
    }

    let source = samples.len();
    let sink = source + 1;
    let mut graph = vec![Vec::<FlowEdge>::new(); samples.len() + 2];
    const TERMINAL_CAPACITY: u64 = 1_u64 << 50;
    for (_, sample) in projections.iter().take(terminal_count) {
        add_flow_edge(
            &mut graph,
            local_index[*sample as usize],
            sink,
            TERMINAL_CAPACITY,
        );
    }
    for (_, sample) in projections.iter().rev().take(terminal_count) {
        add_flow_edge(
            &mut graph,
            source,
            local_index[*sample as usize],
            TERMINAL_CAPACITY,
        );
    }

    for sample in samples.iter().copied() {
        let a = sample as usize;
        let local_a = local_index[a];
        for neighbor in topology.neighbors(sample) {
            if *neighbor <= sample || model.current_plate_ids[*neighbor as usize] != candidate.plate {
                continue;
            }
            let b = *neighbor as usize;
            let local_b = local_index[b];
            if local_b == usize::MAX {
                continue;
            }
            let weakness = (f64::from(model.lithospheric_weakness_index[a])
                + f64::from(model.lithospheric_weakness_index[b]))
                * 0.5;
            let strength = (1.0 - weakness.clamp(0.0, 1.0)).powi(2);
            let both_non_oceanic = model.crust_kind[a] != CrustKind::Oceanic as u8
                && model.crust_kind[b] != CrustKind::Oceanic as u8;
            let both_oceanic = model.crust_kind[a] == CrustKind::Oceanic as u8
                && model.crust_kind[b] == CrustKind::Oceanic as u8;
            let material_factor = if both_non_oceanic {
                0.78
            } else if both_oceanic {
                1.22
            } else {
                0.92
            };
            let tie = ((u64::from(sample)
                .wrapping_mul(0x9e37_79b9)
                ^ u64::from(*neighbor).wrapping_mul(0x85eb_ca6b))
                % 7) as f64;
            let capacity = (80.0 + strength * material_factor * 10_000.0 + tie)
                .round()
                .max(1.0) as u64;
            add_undirected_flow_edge(&mut graph, local_a, local_b, capacity);
        }
    }

    max_flow(&mut graph, source, sink);
    let mut reachable_local = vec![false; graph.len()];
    let mut queue = VecDeque::from([source]);
    reachable_local[source] = true;
    while let Some(node) = queue.pop_front() {
        for edge in &graph[node] {
            if edge.capacity > 0 && !reachable_local[edge.to] {
                reachable_local[edge.to] = true;
                queue.push_back(edge.to);
            }
        }
    }

    let mut side = vec![false; topology.sample_count() as usize];
    let mut child_count = 0usize;
    for sample in samples.iter().copied() {
        let child_side = reachable_local[local_index[sample as usize]];
        side[sample as usize] = child_side;
        child_count += usize::from(child_side);
    }
    let parent_count = samples.len().saturating_sub(child_count);
    let minimum_side = child_count.min(parent_count);
    if minimum_side < 16 || minimum_side * 5 < samples.len() {
        return None;
    }
    if !spatial_side_connected(
        topology,
        candidate.plate,
        &model.current_plate_ids,
        &side,
        true,
        child_count,
    ) || !spatial_side_connected(
        topology,
        candidate.plate,
        &model.current_plate_ids,
        &side,
        false,
        parent_count,
    ) {
        return None;
    }

    let mut cut_weakness_sum = 0.0_f64;
    let mut cut_edges = 0usize;
    let mut geometry = None::<(f64, u32, u32)>;
    for sample in samples.iter().copied() {
        let a = sample as usize;
        for neighbor in topology.neighbors(sample) {
            if *neighbor <= sample || model.current_plate_ids[*neighbor as usize] != candidate.plate {
                continue;
            }
            let b = *neighbor as usize;
            if side[a] == side[b] {
                continue;
            }
            let weakness = (f64::from(model.lithospheric_weakness_index[a])
                + f64::from(model.lithospheric_weakness_index[b]))
                * 0.5;
            cut_weakness_sum += weakness;
            cut_edges += 1;
            if geometry
                .map(|current| {
                    weakness > current.0 + 1.0e-12
                        || ((weakness - current.0).abs() <= 1.0e-12
                            && (sample, *neighbor) < (current.1, current.2))
                })
                .unwrap_or(true)
            {
                geometry = Some((weakness, sample, *neighbor));
            }
        }
    }
    let (_, geometry_a, geometry_b) = geometry?;
    let mean_cut_weakness = cut_weakness_sum / cut_edges.max(1) as f64;
    let balance = minimum_side as f64 / child_count.max(parent_count) as f64;
    Some((side, geometry_a, geometry_b, mean_cut_weakness, balance))
}

#[derive(Clone, Copy, Default)]
struct PlateRiftForcing {
    divergent_rate_length: f64,
    convergent_rate_length: f64,
    boundary_length: f64,
}

impl PlateRiftForcing {
    fn mean_extension(self) -> f64 {
        self.divergent_rate_length / self.boundary_length.max(1.0e-12)
    }

    fn mean_compression(self) -> f64 {
        self.convergent_rate_length / self.boundary_length.max(1.0e-12)
    }

    fn net_tension(self) -> f64 {
        self.mean_extension() - 0.45 * self.mean_compression()
    }

    fn can_nucleate_rift(self) -> bool {
        let extension = self.mean_extension();
        let compression = self.mean_compression();
        extension >= 0.006
            && self.net_tension() >= 0.0035
            && extension >= compression * 0.65
    }
}

fn plate_rift_forcing<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    velocities: &[[f64; 3]],
    planet: PlanetPhysicalParameters,
) -> Vec<PlateRiftForcing> {
    let plate_count = model.metrics.modern_plate_count as usize;
    let mut forcing = vec![PlateRiftForcing::default(); plate_count];
    for sample_a in 0..topology.sample_count() {
        let a = sample_a as usize;
        for sample_b in topology.neighbors(sample_a) {
            if *sample_b <= sample_a {
                continue;
            }
            let b = *sample_b as usize;
            let owner_a = model.current_plate_ids[a] as usize;
            let owner_b = model.current_plate_ids[b] as usize;
            if owner_a == owner_b || owner_a >= plate_count || owner_b >= plate_count {
                continue;
            }

            let position_a = topology.unit_position(sample_a);
            let position_b = topology.unit_position(*sample_b);
            let midpoint = normalize_or(add(position_a, position_b), position_a);
            let normal = normalize_or(sub(position_b, position_a), position_a);
            let velocity_a = scale(
                cross(velocities[owner_a], midpoint),
                planet.radius_m / 1_000_000.0,
            );
            let velocity_b = scale(
                cross(velocities[owner_b], midpoint),
                planet.radius_m / 1_000_000.0,
            );
            let normal_rate = dot(sub(velocity_b, velocity_a), normal);
            let edge_length = dot(position_a, position_b)
                .clamp(-1.0, 1.0)
                .acos()
                .max(1.0e-9);

            for owner in [owner_a, owner_b] {
                forcing[owner].boundary_length += edge_length;
                if normal_rate > 0.0 {
                    forcing[owner].divergent_rate_length += normal_rate * edge_length;
                } else {
                    forcing[owner].convergent_rate_length += -normal_rate * edge_length;
                }
            }
        }
    }
    forcing
}

fn accumulate_extensional_strain<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    velocities: &[[f64; 3]],
    planet: PlanetPhysicalParameters,
    extensional_strain_myr: &mut [f32],
    dt_myr: f64,
) {
    let forcing = plate_rift_forcing(topology, model, velocities, planet);
    for sample in 0..topology.sample_count() as usize {
        let owner = model.current_plate_ids[sample] as usize;
        if owner >= forcing.len() {
            continue;
        }
        let tension = forcing[owner].net_tension();
        if tension > 0.002 {
            let loading = ((tension - 0.002) / 0.008).clamp(0.0, 1.5);
            extensional_strain_myr[sample] =
                (extensional_strain_myr[sample] + (dt_myr * loading) as f32)
                    .clamp(0.0, 160.0);
        } else {
            // Extension memory is material state, but it relaxes when the regional stress field
            // is no longer tensile. This prevents a once-stretched plate from spawning repeated
            // ruptures indefinitely after its boundary conditions have changed.
            let decay = (-dt_myr / 28.0).exp() as f32;
            extensional_strain_myr[sample] *= decay;
        }
    }
}

fn mature_forward_transitional_crust(
    model: &mut HistoricalLithosphereModel,
    extensional_strain_myr: &mut [f32],
) {
    for sample in 0..model.crust_kind.len() {
        if model.crust_kind[sample] != CrustKind::Transitional as u8 {
            continue;
        }
        let age = model.crust_birth_age_myr[sample];
        // Initial passive-margin transitional crust is old (>100 Myr in the initializer). Only
        // young material created by this forward solver can cross the breakup threshold here;
        // inherited continental margins remain transitional unless a new physical opening forms.
        if age < 100.0
            && age >= FORWARD_TRANSITION_MATURATION_MYR
            && extensional_strain_myr[sample] >= 18.0
        {
            model.crust_kind[sample] = CrustKind::Oceanic as u8;
            model.crust_birth_age_myr[sample] = 0.0;
            model.lithospheric_weakness_index[sample] =
                model.lithospheric_weakness_index[sample].min(0.55);
            extensional_strain_myr[sample] *= 0.35;
        }
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
    planet: PlanetPhysicalParameters,
    extensional_strain_myr: &mut [f32],
) -> bool {
    let plate_count = model.metrics.modern_plate_count as usize;
    if velocities.len() != plate_count || plate_count >= usize::from(MAX_TECTONIC_PLATES) {
        return false;
    }

    let rift_forcing = plate_rift_forcing(topology, model, velocities, planet);
    if !rift_forcing.iter().any(|forcing| forcing.can_nucleate_rift()) {
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
    let mut candidate_edges_by_plate = vec![Vec::<RiftCandidate>::new(); plate_count];
    for sample_a in 0..topology.sample_count() {
        let a = sample_a as usize;
        let owner = model.current_plate_ids[a] as usize;
        if owner >= plate_count
            || !rift_forcing[owner].can_nucleate_rift()
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
            let strain_myr = (f64::from(extensional_strain_myr[a])
                + f64::from(extensional_strain_myr[b]))
                * 0.5;
            if weakness < 0.42 || strain_myr < f64::from(RIFT_STRAIN_NUCLEATION_MYR) {
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
            let tension = rift_forcing[owner].net_tension();
            let stress_gain = 1.0 + (tension / 0.02).clamp(0.0, 2.0);
            let strain_gain =
                1.0 + (strain_myr / f64::from(RIFT_STRAIN_NUCLEATION_MYR) - 1.0)
                    .clamp(0.0, 1.5)
                    * 0.35;
            let score = (weakness + material_bonus) * stress_gain * strain_gain + tie;
            let candidate = RiftCandidate {
                plate: owner as u16,
                sample_a,
                sample_b: *sample_b,
                plane_normal,
                weakness,
                score,
            };
            candidate_edges_by_plate[owner].push(candidate);
        }
    }

    let mut best: Option<(f64, RiftCandidate, Vec<bool>, u32, u32)> = None;
    for (plate, candidates) in candidate_edges_by_plate.iter_mut().enumerate() {
        candidates.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.sample_a.cmp(&right.sample_a))
                .then_with(|| left.sample_b.cmp(&right.sample_b))
        });
        candidates.truncate(8);
        let samples = &samples_by_plate[plate];
        for candidate in candidates.iter().copied() {
            let Some((side, geometry_a, geometry_b, cut_weakness, balance)) =
                weak_corridor_partition(topology, model, candidate, samples)
            else {
                continue;
            };
            if cut_weakness < 0.46 {
                continue;
            }
            let size_weight = (samples.len() as f64 / 64.0).sqrt().min(3.0);
            let score = candidate.score
                * (0.55 + 0.45 * balance)
                * (0.65 + 0.35 * cut_weakness.clamp(0.0, 1.0))
                * size_weight;
            if best
                .as_ref()
                .map(|current| {
                    score > current.0 + 1.0e-12
                        || ((score - current.0).abs() <= 1.0e-12
                            && candidate.plate < current.1.plate)
                })
                .unwrap_or(true)
            {
                best = Some((score, candidate, side, geometry_a, geometry_b));
            }
        }
    }

    let Some((_score, candidate, child_side, geometry_a, geometry_b)) = best else {
        return false;
    };
    let parent = candidate.plate as usize;
    let child = plate_count as u16;
    for sample in &samples_by_plate[parent] {
        if child_side[*sample as usize] {
            model.current_plate_ids[*sample as usize] = child;
        }
        // Rupture releases most of the accumulated plate-scale extensional strain. Continued
        // spreading must reload the system before another internal rupture can nucleate.
        extensional_strain_myr[*sample as usize] *= RIFT_STRAIN_RELIEF_FACTOR;
    }

    // Give the two new rigid bodies a divergent velocity component normal to the inherited weak
    // contact. This creates an actual opening boundary that can generate new lithosphere during
    // subsequent advection instead of recording a rift label on a static partition.
    let geometry_a_position = topology.unit_position(geometry_a);
    let geometry_b_position = topology.unit_position(geometry_b);
    let midpoint = normalize_or(
        add(geometry_a_position, geometry_b_position),
        geometry_a_position,
    );
    let opening_normal = normalize_or(
        sub(geometry_b_position, geometry_a_position),
        candidate.plane_normal,
    );
    let angular_offset =
        (0.07 + 0.11 * candidate.weakness.clamp(0.0, 1.0)).to_radians();
    let delta = scale(cross(midpoint, opening_normal), angular_offset);
    let base_velocity = velocities[parent];
    velocities[parent] = sub(base_velocity, delta);
    velocities.push(add(base_velocity, delta));
    model.metrics.modern_plate_count = child + 1;
    model.metrics.rift_birth_count = model.metrics.rift_birth_count.saturating_add(1);

    let a = geometry_a as usize;
    let b = geometry_b as usize;
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
        geometry_sample_a: geometry_a,
        geometry_sample_b: geometry_b,
    });
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
            "forward plate evolution retained an empty plate id after compaction",
        ));
    }
    Ok(())
}

pub fn evolve_modern_plate_geometry<T: PlanetTopology>(
    topology: &T,
    mut model: HistoricalLithosphereModel,
    seed: &str,
    requested_plate_scale: u16,
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

    if requested_plate_scale == 0 || requested_plate_scale > model.metrics.modern_plate_count {
        return Err(WorldgenError::InvalidTectonics(
            "forward plate evolution requested scale must be within the ancestral plate count",
        ));
    }

    let stage_seed = derive_stage_seed(seed, FORWARD_PLATE_NAMESPACE);
    let mut velocities = model.current_plate_angular_velocities_rad_per_myr.clone();
    let mut extensional_strain_myr = vec![0.0_f32; topology.sample_count() as usize];
    if velocities.len() != model.metrics.modern_plate_count as usize {
        return Err(WorldgenError::InvalidTectonics(
            "historical state is missing current plate kinematics",
        ));
    }
    for epoch in 0..FORWARD_EPOCHS {
        let mut generation_fragments = BTreeMap::<(u8, u16, u8, u16), u16>::new();
        record_epoch_events(topology, &mut model, &velocities, epoch, planet);
        for _ in 0..SUBSTEPS_PER_EPOCH {
            advect_substep(
                topology,
                &mut model,
                &mut velocities,
                epoch as u8,
                &mut generation_fragments,
                &mut extensional_strain_myr,
                planet,
                SUBSTEP_MYR,
            )?;
            consume_convergent_boundary_band(
                topology,
                &mut model,
                &mut velocities,
                &mut generation_fragments,
                &mut extensional_strain_myr,
                planet,
                SUBSTEP_MYR,
            );
            accumulate_extensional_strain(
                topology,
                &model,
                &velocities,
                planet,
                &mut extensional_strain_myr,
                SUBSTEP_MYR,
            );
            mature_forward_transitional_crust(
                &mut model,
                &mut extensional_strain_myr,
            );
        }

        // Plate birth is stress-triggered rather than scheduled from the requested plate count.
        // At most one rupture is resolved per coarse epoch so the new boundary can evolve before
        // another internal rupture is considered.
        if epoch >= 1 && epoch <= FORWARD_EPOCHS.saturating_sub(2) {
            let _ = split_one_rifting_plate(
                topology,
                &mut model,
                &mut velocities,
                epoch,
                stage_seed,
                planet,
                &mut extensional_strain_myr,
            );
        }

        // The requested scale does not schedule births or deaths. The epoch ends with whatever
        // topology the physical rifting, advection, convergence, accretion, and extinction produced.
    }

    // Present plate cardinality is an output of physical births and extinctions. The request sets
    // the initial tectonic scale; it must not become a hidden final-state objective.
    if model.metrics.modern_plate_count == 0
        || model.metrics.modern_plate_count > MAX_TECTONIC_PLATES
    {
        return Err(WorldgenError::InvalidTectonics(
            "forward plate evolution produced an unsupported emergent plate count",
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
    fn genealogy_partitions_are_inert_to_forward_plate_physics() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let request = HistoricalLithosphereRequest::new("forward-genealogy-inert", 16);
        let base = crate::historical_lithosphere::generate_historical_lithosphere(
            &topology,
            &request,
            planet,
        )
        .unwrap();

        let mut collapsed = base.clone();
        let mut representative = BTreeMap::<u16, u16>::new();
        for fragment in &collapsed.fragments {
            representative
                .entry(fragment.origin_plate_id)
                .or_insert(fragment.id);
        }
        for sample in 0..topology.sample_count() as usize {
            collapsed.fragment_ids[sample] =
                representative[&collapsed.origin_plate_ids[sample]];
        }
        assert_ne!(
            base.fragment_ids, collapsed.fragment_ids,
            "regression did not alter genealogy partition geometry"
        );

        let reference = evolve_modern_plate_geometry(
            &topology,
            base,
            &request.seed,
            request.modern_plate_count,
            planet,
        )
        .unwrap();
        let relabeled = evolve_modern_plate_geometry(
            &topology,
            collapsed,
            &request.seed,
            request.modern_plate_count,
            planet,
        )
        .unwrap();

        assert_eq!(reference.current_plate_ids, relabeled.current_plate_ids);
        assert_eq!(reference.crust_kind, relabeled.crust_kind);
        assert_eq!(reference.crust_birth_age_myr, relabeled.crust_birth_age_myr);
        assert_eq!(
            reference.lithospheric_weakness_index,
            relabeled.lithospheric_weakness_index
        );
        assert_eq!(
            reference.current_plate_angular_velocities_rad_per_myr,
            relabeled.current_plate_angular_velocities_rad_per_myr
        );

        let mut provenance_relabel = crate::historical_lithosphere::generate_historical_lithosphere(
            &topology,
            &request,
            planet,
        )
        .unwrap();
        let origin_count = provenance_relabel.ancestral_tectonics.plates.len() as u16;
        let relabel_origin = |origin: u16| {
            ((u32::from(origin) * 7 + 3) % u32::from(origin_count)) as u16
        };
        for origin in &mut provenance_relabel.origin_plate_ids {
            *origin = relabel_origin(*origin);
        }
        for fragment in &mut provenance_relabel.fragments {
            fragment.origin_plate_id = relabel_origin(fragment.origin_plate_id);
        }

        let provenance_intervened = evolve_modern_plate_geometry(
            &topology,
            provenance_relabel,
            &request.seed,
            request.modern_plate_count,
            planet,
        )
        .unwrap();
        assert_eq!(
            reference.current_plate_ids,
            provenance_intervened.current_plate_ids,
            "ancestral provenance labels changed physical present ownership"
        );
        assert_eq!(reference.crust_kind, provenance_intervened.crust_kind);
        assert_eq!(
            reference.crust_birth_age_myr,
            provenance_intervened.crust_birth_age_myr
        );
        assert_eq!(
            reference.lithospheric_weakness_index,
            provenance_intervened.lithospheric_weakness_index
        );
        assert_eq!(
            reference.current_plate_angular_velocities_rad_per_myr,
            provenance_intervened.current_plate_angular_velocities_rad_per_myr
        );
    }

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
