use crate::{
    derive_stage_seed, CrustKind, HistoricalEventKind, HistoricalLithosphereModel, PlanetTopology,
    WorldgenError,
};
use std::collections::{BTreeMap, VecDeque};

const BOUNDARY_FIELD_NAMESPACE: &str = "worldgen:geology:material-aware-boundary-plates:v1";
const KINEMATIC_EPOCHS: usize = 10;
const EPOCH_DURATION_MYR: f64 = 3.0;
const MAX_OWNER_DEPTH: u16 = 10;

#[derive(Clone, Copy, Debug)]
struct PlateField {
    center: [f64; 3],
    tangent_u: [f64; 3],
    tangent_v: [f64; 3],
    angular_velocity: [f64; 3],
    area_bias: f64,
    mobility: f64,
    stretch: f64,
    bend: f64,
}

#[derive(Clone, Debug)]
struct MaterialSignals {
    owner_depth: Vec<u16>,
    rift_release: Vec<f64>,
    structure_release: Vec<f64>,
    plate_adjacency: Vec<Vec<bool>>,
    plate_area_fraction: Vec<f64>,
    plate_continental_fraction: Vec<f64>,
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

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn normalize_or(value: [f64; 3], fallback: [f64; 3]) -> [f64; 3] {
    let magnitude = norm(value);
    if magnitude > 1.0e-15 {
        scale(value, 1.0 / magnitude)
    } else {
        fallback
    }
}

fn unit_random(value: u64) -> f64 {
    let mut value = value;
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    ((value >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0)
}

fn random_unit_vector(seed: u64, stream: u64) -> [f64; 3] {
    let z = unit_random(seed ^ stream ^ 0x65b5_61f5_1b6b_42d1) * 2.0 - 1.0;
    let theta = unit_random(seed ^ stream ^ 0xa076_1d64_78bd_642f) * std::f64::consts::TAU;
    let radial = (1.0 - z * z).max(0.0).sqrt();
    [radial * theta.cos(), radial * theta.sin(), z]
}

fn rotate_vector(value: [f64; 3], angular_velocity: [f64; 3], dt_myr: f64) -> [f64; 3] {
    let speed = norm(angular_velocity);
    if speed <= 1.0e-15 {
        return value;
    }
    let axis = scale(angular_velocity, 1.0 / speed);
    let angle = speed * dt_myr;
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    let rotated = add(
        add(
            scale(value, cos_angle),
            scale(cross(axis, value), sin_angle),
        ),
        scale(axis, dot(axis, value) * (1.0 - cos_angle)),
    );
    normalize_or(rotated, value)
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

fn diffuse_seed_strength<T: PlanetTopology>(
    topology: &T,
    seeds: &[bool],
    max_steps: u16,
) -> Vec<f64> {
    let count = topology.sample_count() as usize;
    let mut distance = vec![u16::MAX; count];
    let mut queue = VecDeque::<u32>::new();
    for sample in 0..topology.sample_count() {
        if seeds[sample as usize] {
            distance[sample as usize] = 0;
            queue.push_back(sample);
        }
    }
    while let Some(sample) = queue.pop_front() {
        let next = distance[sample as usize].saturating_add(1);
        if next > max_steps {
            continue;
        }
        for neighbor in topology.neighbors(sample) {
            let index = *neighbor as usize;
            if distance[index] > next {
                distance[index] = next;
                queue.push_back(*neighbor);
            }
        }
    }
    distance
        .into_iter()
        .map(|steps| {
            if steps == u16::MAX || steps > max_steps {
                0.0
            } else {
                1.0 - f64::from(steps) / f64::from(max_steps.saturating_add(1))
            }
        })
        .collect()
}

fn build_material_signals<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    initial: &[u16],
) -> MaterialSignals {
    let count = topology.sample_count() as usize;
    let plate_count = model.metrics.modern_plate_count as usize;
    let mut owner_depth = vec![u16::MAX; count];
    let mut queue = VecDeque::<u32>::new();
    let mut plate_adjacency = vec![vec![false; plate_count]; plate_count];
    let mut rift_seeds = vec![false; count];
    let mut structure_seeds = vec![false; count];
    let mut plate_area = vec![0.0_f64; plate_count];
    let mut plate_continental_area = vec![0.0_f64; plate_count];
    let mut total_area = 0.0_f64;

    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let owner = initial[index] as usize;
        let area = topology.area_steradians(sample);
        total_area += area;
        plate_area[owner] += area;
        if model.crust_kind[index] != CrustKind::Oceanic as u8 {
            plate_continental_area[owner] += area;
        }

        let mut boundary = false;
        for neighbor in topology.neighbors(sample) {
            let ni = *neighbor as usize;
            let other = initial[ni] as usize;
            if other != owner {
                boundary = true;
                plate_adjacency[owner][other] = true;
                plate_adjacency[other][owner] = true;
            }

            if model.fragment_ids[ni] != model.fragment_ids[index] {
                let fragment_a = &model.fragments[model.fragment_ids[index] as usize];
                let fragment_b = &model.fragments[model.fragment_ids[ni] as usize];
                if fragment_a.parent_fragment_id.is_some()
                    && fragment_a.parent_fragment_id == fragment_b.parent_fragment_id
                {
                    rift_seeds[index] = true;
                    rift_seeds[ni] = true;
                }
                if model.origin_plate_ids[ni] != model.origin_plate_ids[index] {
                    structure_seeds[index] = true;
                    structure_seeds[ni] = true;
                }
            }
        }
        if boundary {
            owner_depth[index] = 0;
            queue.push_back(sample);
        }
    }

    while let Some(sample) = queue.pop_front() {
        let index = sample as usize;
        let next = owner_depth[index].saturating_add(1);
        if next > MAX_OWNER_DEPTH {
            continue;
        }
        let owner = initial[index];
        for neighbor in topology.neighbors(sample) {
            let ni = *neighbor as usize;
            if initial[ni] == owner && owner_depth[ni] > next {
                owner_depth[ni] = next;
                queue.push_back(*neighbor);
            }
        }
    }
    for depth in &mut owner_depth {
        if *depth == u16::MAX {
            *depth = MAX_OWNER_DEPTH;
        }
    }

    for event in &model.events {
        let strength = f64::from(event.strength).clamp(0.0, 1.0);
        let mark = |seeds: &mut [bool], sample: u32| {
            if (sample as usize) < seeds.len() {
                seeds[sample as usize] = true;
            }
        };
        match event.kind {
            HistoricalEventKind::Rift | HistoricalEventKind::Spreading => {
                if strength >= 0.18 {
                    mark(&mut rift_seeds, event.geometry_sample_a);
                    mark(&mut rift_seeds, event.geometry_sample_b);
                }
            }
            HistoricalEventKind::Collision
            | HistoricalEventKind::Transform
            | HistoricalEventKind::Accretion
            | HistoricalEventKind::Capture
            | HistoricalEventKind::Subduction => {
                if strength >= 0.18 {
                    mark(&mut structure_seeds, event.geometry_sample_a);
                    mark(&mut structure_seeds, event.geometry_sample_b);
                }
            }
        }
    }

    let rift_release = diffuse_seed_strength(topology, &rift_seeds, 4);
    let structure_release = diffuse_seed_strength(topology, &structure_seeds, 3);
    let total_area = total_area.max(1.0e-12);
    let plate_area_fraction = plate_area
        .iter()
        .map(|area| *area / total_area)
        .collect::<Vec<_>>();
    let plate_continental_fraction = plate_area
        .iter()
        .zip(plate_continental_area.iter())
        .map(|(area, continental)| {
            if *area > 0.0 {
                (*continental / *area).clamp(0.0, 1.0)
            } else {
                0.0
            }
        })
        .collect::<Vec<_>>();

    MaterialSignals {
        owner_depth,
        rift_release,
        structure_release,
        plate_adjacency,
        plate_area_fraction,
        plate_continental_fraction,
    }
}

fn choose_plate_cores<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    owners: &[u16],
    signals: &MaterialSignals,
    plate_count: usize,
    seed: u64,
) -> Result<Vec<u32>, WorldgenError> {
    if (topology.sample_count() as usize) < plate_count || plate_count == 0 {
        return Err(WorldgenError::InvalidTectonics(
            "material-aware boundary synthesis cannot place the requested plate cores",
        ));
    }

    let mut cores = Vec::with_capacity(plate_count);
    for plate in 0..plate_count {
        let mut best = None::<(f64, u32)>;
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            if owners[index] as usize != plate {
                continue;
            }
            let crust_bonus = match model.crust_kind[index] {
                value if value == CrustKind::Continental as u8 => 0.80,
                value if value == CrustKind::Transitional as u8 => 0.45,
                _ => 0.0,
            };
            let depth_bonus = f64::from(signals.owner_depth[index].min(MAX_OWNER_DEPTH)) * 0.16;
            let corridor_penalty = signals.rift_release[index] * 0.70
                + signals.structure_release[index] * 0.20;
            let jitter = unit_random(
                seed ^ u64::from(sample).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                    ^ (plate as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9),
            ) * 0.025;
            let score = depth_bonus + crust_bonus - corridor_penalty + jitter;
            let candidate = (score, sample);
            if best
                .map(|current| {
                    candidate.0 > current.0
                        || (candidate.0 == current.0 && candidate.1 < current.1)
                })
                .unwrap_or(true)
            {
                best = Some(candidate);
            }
        }
        let Some((_, sample)) = best else {
            return Err(WorldgenError::InvalidTectonics(
                "material-aware boundary synthesis found an empty provisional plate",
            ));
        };
        cores.push(sample);
    }
    Ok(cores)
}

fn tangent_frame(center: [f64; 3], random_axis: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let projected = sub(random_axis, scale(center, dot(random_axis, center)));
    let fallback = if center[2].abs() < 0.8 {
        normalize_or(cross([0.0, 0.0, 1.0], center), [1.0, 0.0, 0.0])
    } else {
        normalize_or(cross([0.0, 1.0, 0.0], center), [1.0, 0.0, 0.0])
    };
    let tangent_u = normalize_or(projected, fallback);
    let tangent_v = normalize_or(cross(center, tangent_u), fallback);
    (tangent_u, tangent_v)
}

fn build_fields<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    initial: &[u16],
    cores: &[u32],
    signals: &MaterialSignals,
    seed: u64,
) -> Vec<PlateField> {
    let angular_velocity = plate_angular_velocities(topology, model, initial);
    let mean_area = 1.0 / cores.len().max(1) as f64;
    cores
        .iter()
        .enumerate()
        .map(|(plate, core)| {
            let stream = (plate as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
            let center = topology.unit_position(*core);
            let (tangent_u, tangent_v) = tangent_frame(
                center,
                random_unit_vector(seed, stream ^ 0xa076_1d64_78bd_642f),
            );
            let area_ratio = (signals.plate_area_fraction[plate] / mean_area.max(1.0e-12))
                .max(1.0e-6);
            let inherited_area_bias = (area_ratio.ln() * 0.105).clamp(-0.18, 0.22);
            let random_area_bias =
                (unit_random(seed ^ stream ^ 0xe703_7ed1_a0b4_28db) - 0.5) * 0.025;
            let continental_fraction = signals.plate_continental_fraction[plate];
            PlateField {
                center,
                tangent_u,
                tangent_v,
                angular_velocity: angular_velocity[plate],
                area_bias: inherited_area_bias + random_area_bias,
                mobility: (0.94 - continental_fraction * 0.24).clamp(0.62, 0.94),
                stretch: (unit_random(seed ^ stream ^ 0x8ebc_6af0_9c88_c6e3) - 0.5) * 0.24,
                bend: (unit_random(seed ^ stream ^ 0xd6e8_feb8_6659_fd93) - 0.5) * 0.18,
            }
        })
        .collect()
}

fn evolve_fields(fields: &mut [PlateField]) {
    const MIN_CORE_SEPARATION_RAD: f64 = 0.20;
    for _ in 0..KINEMATIC_EPOCHS {
        let proposals = fields
            .iter()
            .map(|field| {
                let dt = EPOCH_DURATION_MYR * field.mobility;
                (
                    rotate_vector(field.center, field.angular_velocity, dt),
                    rotate_vector(field.tangent_u, field.angular_velocity, dt),
                )
            })
            .collect::<Vec<_>>();
        let mut blocked = vec![false; fields.len()];
        for left in 0..fields.len() {
            for right in (left + 1)..fields.len() {
                let separation = dot(proposals[left].0, proposals[right].0)
                    .clamp(-1.0, 1.0)
                    .acos();
                if separation < MIN_CORE_SEPARATION_RAD {
                    blocked[left] = true;
                    blocked[right] = true;
                }
            }
        }
        for (index, field) in fields.iter_mut().enumerate() {
            if blocked[index] {
                continue;
            }
            field.center = proposals[index].0;
            field.tangent_u = proposals[index].1;
            field.tangent_v = normalize_or(cross(field.center, field.tangent_u), field.tangent_v);
        }
    }
}

fn warp_position(position: [f64; 3], stress_axes: [[f64; 3]; 2]) -> [f64; 3] {
    let a = stress_axes[0];
    let b = stress_axes[1];
    let da = dot(position, a);
    let db = dot(position, b);
    let tangent_a = sub(a, scale(position, da));
    let tangent_b = sub(b, scale(position, db));
    let warp = add(scale(tangent_a, 0.072 * db), scale(tangent_b, -0.058 * da));
    normalize_or(add(position, warp), position)
}

fn material_retention_strength(crust_kind: u8, owner_depth: u16) -> f64 {
    let base = if crust_kind == CrustKind::Continental as u8 {
        0.34
    } else if crust_kind == CrustKind::Transitional as u8 {
        0.23
    } else {
        0.075
    };
    let depth = f64::from(owner_depth.min(MAX_OWNER_DEPTH)) / f64::from(MAX_OWNER_DEPTH);
    base * (0.18 + 0.82 * depth.sqrt())
}

fn field_score(
    sample_index: usize,
    position: [f64; 3],
    candidate_plate: u16,
    initial_owner: u16,
    crust_kind: u8,
    field: PlateField,
    stress_axes: [[f64; 3]; 2],
    signals: &MaterialSignals,
) -> f64 {
    let warped = warp_position(position, stress_axes);
    let distance = dot(warped, field.center).clamp(-1.0, 1.0).acos();
    let far_cap = if distance > 1.72 {
        (distance - 1.72) * 4.0
    } else {
        0.0
    };

    // Low-order anisotropy keeps boundaries freeform without reverting to local cellular growth.
    let u = dot(warped, field.tangent_u);
    let v = dot(warped, field.tangent_v);
    let shape = field.stretch * (u * u - v * v) + field.bend * (2.0 * u * v);

    let corridor_release = (1.0
        - signals.rift_release[sample_index] * 0.82
        - signals.structure_release[sample_index] * 0.38)
        .clamp(0.12, 1.0);
    let retention = material_retention_strength(crust_kind, signals.owner_depth[sample_index])
        * corridor_release;
    let ancestry = if candidate_plate == initial_owner {
        retention
    } else {
        let initial = initial_owner as usize;
        let candidate = candidate_plate as usize;
        if signals.plate_adjacency[initial][candidate] {
            // Neighboring provisional domains may advance preferentially through explicit rift,
            // suture, capture and collision corridors, but receive no such help in stable interiors.
            signals.rift_release[sample_index] * 0.085
                + signals.structure_release[sample_index] * 0.035
        } else {
            0.0
        }
    };

    -distance + field.area_bias + shape + ancestry - far_cap
}

fn choose_field_cores<T: PlanetTopology>(
    topology: &T,
    fields: &[PlateField],
    initial: &[u16],
    model: &HistoricalLithosphereModel,
    signals: &MaterialSignals,
) -> Vec<u32> {
    let mut used = vec![false; topology.sample_count() as usize];
    let mut cores = Vec::with_capacity(fields.len());
    for (plate, field) in fields.iter().enumerate() {
        let mut best_sample = None::<u32>;
        let mut best_score = f64::NEG_INFINITY;
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            if used[index] {
                continue;
            }
            let alignment = dot(topology.unit_position(sample), field.center);
            let material_bonus = if initial[index] as usize == plate {
                material_retention_strength(model.crust_kind[index], signals.owner_depth[index]) * 0.55
            } else {
                0.0
            };
            let corridor_penalty = signals.rift_release[index] * 0.12;
            let score = alignment + material_bonus - corridor_penalty;
            if score > best_score
                || (score == best_score && best_sample.map(|current| sample < current).unwrap_or(true))
            {
                best_score = score;
                best_sample = Some(sample);
            }
        }
        let best_sample = best_sample.unwrap_or(0);
        used[best_sample as usize] = true;
        cores.push(best_sample);
    }
    cores
}

fn assign_from_fields<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    fields: &[PlateField],
    initial: &[u16],
    cores: &[u32],
    stress_axes: [[f64; 3]; 2],
    signals: &MaterialSignals,
) -> Vec<u16> {
    let mut owners = vec![0_u16; topology.sample_count() as usize];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let position = topology.unit_position(sample);
        let mut best_plate = 0_u16;
        let mut best_score = f64::NEG_INFINITY;
        for (plate, field) in fields.iter().copied().enumerate() {
            let candidate = plate as u16;
            let score = field_score(
                index,
                position,
                candidate,
                initial[index],
                model.crust_kind[index],
                field,
                stress_axes,
                signals,
            );
            if score > best_score || (score == best_score && candidate < best_plate) {
                best_score = score;
                best_plate = candidate;
            }
        }
        owners[index] = best_plate;
    }
    for (plate, core) in cores.iter().copied().enumerate() {
        owners[core as usize] = plate as u16;
    }
    owners
}

fn repair_connectivity<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    owners: &mut [u16],
    fields: &[PlateField],
    initial: &[u16],
    cores: &[u32],
    stress_axes: [[f64; 3]; 2],
    signals: &MaterialSignals,
) {
    for _ in 0..6 {
        let mut changed = false;
        for (plate, core) in cores.iter().copied().enumerate() {
            let plate_id = plate as u16;
            owners[core as usize] = plate_id;
            let mut connected = vec![false; owners.len()];
            connected[core as usize] = true;
            let mut queue = VecDeque::from([core]);
            while let Some(sample) = queue.pop_front() {
                for neighbor in topology.neighbors(sample) {
                    let ni = *neighbor as usize;
                    if !connected[ni] && owners[ni] == plate_id {
                        connected[ni] = true;
                        queue.push_back(*neighbor);
                    }
                }
            }

            for sample in 0..topology.sample_count() {
                let index = sample as usize;
                if owners[index] != plate_id || connected[index] || sample == core {
                    continue;
                }
                let mut candidates = BTreeMap::<u16, usize>::new();
                for neighbor in topology.neighbors(sample) {
                    let owner = owners[*neighbor as usize];
                    if owner != plate_id {
                        *candidates.entry(owner).or_insert(0) += 1;
                    }
                }
                if candidates.is_empty() {
                    continue;
                }
                let position = topology.unit_position(sample);
                let mut best = None::<(f64, usize, u16)>;
                for (candidate, contact) in candidates {
                    let score = field_score(
                        index,
                        position,
                        candidate,
                        initial[index],
                        model.crust_kind[index],
                        fields[candidate as usize],
                        stress_axes,
                        signals,
                    );
                    let proposal = (score, contact, candidate);
                    if best
                        .map(|current| {
                            proposal.0 > current.0
                                || (proposal.0 == current.0 && proposal.1 > current.1)
                                || (proposal.0 == current.0
                                    && proposal.1 == current.1
                                    && proposal.2 < current.2)
                        })
                        .unwrap_or(true)
                    {
                        best = Some(proposal);
                    }
                }
                if let Some((_, _, replacement)) = best {
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

fn validate_nonempty(owners: &[u16], plate_count: usize) -> Result<(), WorldgenError> {
    let mut counts = vec![0usize; plate_count];
    for owner in owners {
        let index = *owner as usize;
        if index >= plate_count {
            return Err(WorldgenError::InvalidTectonics(
                "material-aware boundary synthesis produced an invalid owner",
            ));
        }
        counts[index] += 1;
    }
    if counts.iter().any(|count| *count == 0) {
        return Err(WorldgenError::InvalidTectonics(
            "material-aware boundary synthesis eliminated a modern plate",
        ));
    }
    Ok(())
}

/// Synthesizes present plate geometry from material-aware, finite-rotation plate potentials.
///
/// Historical material ownership supplies one provisional domain per modern plate. Continuous
/// spherical fields regularize those domains into sane freeform geometry, but stable continental
/// interiors retain strong ownership while explicit rift/suture/capture/collision corridors release
/// that constraint. Geometry therefore remains free to leave ancestral Voronoi edges without
/// becoming independent of the crustal history it is supposed to carry.
pub(crate) fn synthesize_boundary_first_ownership<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    seed: &str,
) -> Result<Vec<u16>, WorldgenError> {
    let plate_count = model.metrics.modern_plate_count as usize;
    let initial = &model.current_plate_ids;
    let stage_seed = derive_stage_seed(seed, BOUNDARY_FIELD_NAMESPACE);
    let signals = build_material_signals(topology, model, initial);
    let initial_cores = choose_plate_cores(
        topology,
        model,
        initial,
        &signals,
        plate_count,
        stage_seed,
    )?;
    let mut fields = build_fields(topology, model, initial, &initial_cores, &signals, stage_seed);
    evolve_fields(&mut fields);
    let cores = choose_field_cores(topology, &fields, initial, model, &signals);
    let stress_axes = [
        random_unit_vector(stage_seed, 0xd6e8_feb8_6659_fd93),
        random_unit_vector(stage_seed, 0xa5a3_56d5_2f62_56d5),
    ];
    let mut owners = assign_from_fields(
        topology,
        model,
        &fields,
        initial,
        &cores,
        stress_axes,
        &signals,
    );
    repair_connectivity(
        topology,
        model,
        &mut owners,
        &fields,
        initial,
        &cores,
        stress_axes,
        &signals,
    );
    validate_nonempty(&owners, plate_count)?;
    Ok(owners)
}
