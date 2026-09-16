use crate::{derive_stage_seed, HistoricalLithosphereModel, PlanetTopology, WorldgenError};
use std::collections::{BTreeMap, VecDeque};

const BOUNDARY_FIELD_NAMESPACE: &str = "worldgen:geology:boundary-first-plates:v1";
const KINEMATIC_EPOCHS: usize = 10;
const EPOCH_DURATION_MYR: f64 = 3.0;

#[derive(Clone, Copy, Debug)]
struct PlateField {
    center: [f64; 3],
    tangent_u: [f64; 3],
    tangent_v: [f64; 3],
    angular_velocity: [f64; 3],
    area_bias: f64,
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

fn choose_plate_cores<T: PlanetTopology>(
    topology: &T,
    _owners: &[u16],
    plate_count: usize,
    seed: u64,
) -> Result<Vec<u32>, WorldgenError> {
    if (topology.sample_count() as usize) < plate_count || plate_count == 0 {
        return Err(WorldgenError::InvalidTectonics(
            "boundary-first synthesis cannot place the requested plate cores",
        ));
    }
    let first = (0..topology.sample_count())
        .max_by(|left, right| {
            let left_score =
                unit_random(seed ^ u64::from(*left).wrapping_mul(0x9e37_79b9_7f4a_7c15));
            let right_score =
                unit_random(seed ^ u64::from(*right).wrapping_mul(0x9e37_79b9_7f4a_7c15));
            left_score
                .total_cmp(&right_score)
                .then_with(|| right.cmp(left))
        })
        .unwrap_or(0);
    let mut cores = vec![first];
    while cores.len() < plate_count {
        let mut best_sample = None;
        let mut best_score = f64::NEG_INFINITY;
        for sample in 0..topology.sample_count() {
            if cores.contains(&sample) {
                continue;
            }
            let position = topology.unit_position(sample);
            let separation = cores
                .iter()
                .map(|core| {
                    dot(position, topology.unit_position(*core))
                        .clamp(-1.0, 1.0)
                        .acos()
                })
                .fold(std::f64::consts::PI, f64::min);
            let jitter = unit_random(
                seed ^ u64::from(sample).wrapping_mul(0xbf58_476d_1ce4_e5b9)
                    ^ (cores.len() as u64).wrapping_mul(0x94d0_49bb_1331_11eb),
            ) * 0.045;
            let score = separation + jitter;
            if score > best_score || (score == best_score && Some(sample) < best_sample) {
                best_score = score;
                best_sample = Some(sample);
            }
        }
        let Some(sample) = best_sample else {
            return Err(WorldgenError::InvalidTectonics(
                "boundary-first synthesis exhausted spherical core candidates",
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
    seed: u64,
) -> Vec<PlateField> {
    let angular_velocity = plate_angular_velocities(topology, model, initial);
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
            PlateField {
                center,
                tangent_u,
                tangent_v,
                angular_velocity: angular_velocity[initial[*core as usize] as usize],
                area_bias: (unit_random(seed ^ stream ^ 0xe703_7ed1_a0b4_28db) - 0.5) * 0.04,
            }
        })
        .collect()
}

fn evolve_fields(fields: &mut [PlateField]) {
    const MIN_CORE_SEPARATION_RAD: f64 = 0.34;
    for _ in 0..KINEMATIC_EPOCHS {
        let proposals = fields
            .iter()
            .map(|field| {
                (
                    rotate_vector(field.center, field.angular_velocity, EPOCH_DURATION_MYR),
                    rotate_vector(field.tangent_u, field.angular_velocity, EPOCH_DURATION_MYR),
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
    let warp = add(scale(tangent_a, 0.085 * db), scale(tangent_b, -0.065 * da));
    normalize_or(add(position, warp), position)
}

fn field_score(
    position: [f64; 3],
    field: PlateField,
    stress_axes: [[f64; 3]; 2],
    _ancestry_match: bool,
) -> f64 {
    let warped = warp_position(position, stress_axes);
    let distance = dot(warped, field.center).clamp(-1.0, 1.0).acos();
    let far_cap = if distance > 1.65 {
        (distance - 1.65) * 4.0
    } else {
        0.0
    };
    -distance + field.area_bias - far_cap
}

fn choose_field_cores<T: PlanetTopology>(topology: &T, fields: &[PlateField]) -> Vec<u32> {
    let mut used = vec![false; topology.sample_count() as usize];
    let mut cores = Vec::with_capacity(fields.len());
    for field in fields {
        let mut best_sample = 0_u32;
        let mut best_alignment = f64::NEG_INFINITY;
        for sample in 0..topology.sample_count() {
            if used[sample as usize] {
                continue;
            }
            let alignment = dot(topology.unit_position(sample), field.center);
            if alignment > best_alignment {
                best_alignment = alignment;
                best_sample = sample;
            }
        }
        used[best_sample as usize] = true;
        cores.push(best_sample);
    }
    cores
}

fn assign_from_fields<T: PlanetTopology>(
    topology: &T,
    fields: &[PlateField],
    initial: &[u16],
    cores: &[u32],
    stress_axes: [[f64; 3]; 2],
) -> Vec<u16> {
    let mut owners = vec![0_u16; topology.sample_count() as usize];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let position = topology.unit_position(sample);
        let mut best_plate = 0_u16;
        let mut best_score = f64::NEG_INFINITY;
        for (plate, field) in fields.iter().copied().enumerate() {
            let score = field_score(position, field, stress_axes, initial[index] == plate as u16);
            if score > best_score || (score == best_score && (plate as u16) < best_plate) {
                best_score = score;
                best_plate = plate as u16;
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
    owners: &mut [u16],
    fields: &[PlateField],
    initial: &[u16],
    cores: &[u32],
    stress_axes: [[f64; 3]; 2],
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
                        position,
                        fields[candidate as usize],
                        stress_axes,
                        initial[index] == candidate,
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
                "boundary-first synthesis produced an invalid owner",
            ));
        }
        counts[index] += 1;
    }
    if counts.iter().any(|count| *count == 0) {
        return Err(WorldgenError::InvalidTectonics(
            "boundary-first synthesis eliminated a modern plate",
        ));
    }
    Ok(())
}

/// Synthesizes present plate geometry from smooth, finite-rotation plate potentials.
///
/// The equal-potential loci form the tectonic boundary network; plate ownership is rasterized
/// from those continuous boundaries after the field geometry has been advected through a bounded
/// kinematic history. This deliberately replaces sample-by-sample territorial growth as the
/// authority for final plate shape.
pub(crate) fn synthesize_boundary_first_ownership<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    seed: &str,
) -> Result<Vec<u16>, WorldgenError> {
    let plate_count = model.metrics.modern_plate_count as usize;
    let initial = &model.current_plate_ids;
    let stage_seed = derive_stage_seed(seed, BOUNDARY_FIELD_NAMESPACE);
    let initial_cores = choose_plate_cores(topology, initial, plate_count, stage_seed)?;
    let mut fields = build_fields(topology, model, initial, &initial_cores, stage_seed);
    evolve_fields(&mut fields);
    let cores = choose_field_cores(topology, &fields);
    let stress_axes = [
        random_unit_vector(stage_seed, 0xd6e8_feb8_6659_fd93),
        random_unit_vector(stage_seed, 0xa5a3_56d5_2f62_56d5),
    ];
    let mut owners = assign_from_fields(topology, &fields, initial, &cores, stress_axes);
    repair_connectivity(topology, &mut owners, &fields, initial, &cores, stress_axes);
    validate_nonempty(&owners, plate_count)?;
    Ok(owners)
}
