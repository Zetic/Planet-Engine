from pathlib import Path

p = Path('rust/interlink-worldgen/src/boundary_plate_geometry.rs')
s = p.read_text()

marker = '\nfn validate_nonempty(owners: &[u16], plate_count: usize) -> Result<(), WorldgenError> {\n'
if 'fn regularize_plate_domains<T: PlanetTopology>' not in s:
    insert = '''
fn removal_preserves_local_connectivity<T: PlanetTopology>(
    topology: &T,
    owners: &[u16],
    sample: u32,
    owner: u16,
) -> bool {
    let same = topology
        .neighbors(sample)
        .iter()
        .copied()
        .filter(|neighbor| owners[*neighbor as usize] == owner)
        .collect::<Vec<_>>();
    if same.len() <= 1 {
        return true;
    }

    let mut reached = vec![false; same.len()];
    reached[0] = true;
    let mut queue = VecDeque::from([0usize]);
    while let Some(local_index) = queue.pop_front() {
        let node = same[local_index];
        for neighbor in topology.neighbors(node) {
            if *neighbor == sample {
                continue;
            }
            if let Some(next_index) = same.iter().position(|candidate| candidate == neighbor) {
                if !reached[next_index] {
                    reached[next_index] = true;
                    queue.push_back(next_index);
                }
            }
        }
    }
    reached.into_iter().all(|value| value)
}

fn plate_area_fractions<T: PlanetTopology>(
    topology: &T,
    owners: &[u16],
    plate_count: usize,
) -> Vec<f64> {
    let total_area = (0..topology.sample_count())
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>()
        .max(1.0e-12);
    let mut fractions = vec![0.0_f64; plate_count];
    for sample in 0..topology.sample_count() {
        fractions[owners[sample as usize] as usize] +=
            topology.area_steradians(sample) / total_area;
    }
    fractions
}

fn stable_material_interior(model: &HistoricalLithosphereModel, signals: &MaterialSignals, index: usize, owner: u16, initial: &[u16]) -> bool {
    model.crust_kind[index] != CrustKind::Oceanic as u8
        && initial[index] == owner
        && signals.owner_depth[index] >= 4
        && signals.rift_release[index] < 0.22
        && signals.structure_release[index] < 0.22
}

fn regularize_plate_domains<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    owners: &mut [u16],
    fields: &[PlateField],
    initial: &[u16],
    cores: &[u32],
    stress_axes: [[f64; 3]; 2],
    signals: &MaterialSignals,
) {
    const MIN_FRACTION: f64 = 0.012;
    const SOFT_MIN_FRACTION: f64 = 0.016;
    const MAX_FRACTION: f64 = 0.252;
    let targets = target_plate_fractions(signals, fields.len());
    let total_area = (0..topology.sample_count())
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>()
        .max(1.0e-12);
    let mut protected = vec![false; owners.len()];
    for core in cores {
        protected[*core as usize] = true;
    }

    for _ in 0..12 {
        let mut fractions = plate_area_fractions(topology, owners, fields.len());
        let mut changed = 0usize;

        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            if protected[index] {
                continue;
            }
            let current = owners[index];
            let current_index = current as usize;
            let sample_fraction = topology.area_steradians(sample) / total_area;
            if fractions[current_index] - sample_fraction < MIN_FRACTION {
                continue;
            }
            if !removal_preserves_local_connectivity(topology, owners, sample, current) {
                continue;
            }

            let mut contacts = BTreeMap::<u16, usize>::new();
            for neighbor in topology.neighbors(sample) {
                *contacts.entry(owners[*neighbor as usize]).or_insert(0) += 1;
            }
            let same_contact = contacts.get(&current).copied().unwrap_or(0);
            if contacts.len() <= 1 {
                continue;
            }

            let position = topology.unit_position(sample);
            let current_score = field_score(
                index,
                position,
                current,
                initial[index],
                model.crust_kind[index],
                fields[current_index],
                stress_axes,
                signals,
            );
            let mut best = None::<(f64, usize, u16)>;
            for (candidate, candidate_contact) in contacts {
                if candidate == current {
                    continue;
                }
                let candidate_index = candidate as usize;
                if fractions[candidate_index] + sample_fraction > MAX_FRACTION {
                    continue;
                }
                let candidate_score = field_score(
                    index,
                    position,
                    candidate,
                    initial[index],
                    model.crust_kind[index],
                    fields[candidate_index],
                    stress_axes,
                    signals,
                );

                let candidate_deficit = targets[candidate_index] - fractions[candidate_index];
                let current_deficit = targets[current_index] - fractions[current_index];
                let balance_pressure = (candidate_deficit - current_deficit) * 3.0;
                let viability_bonus = if fractions[candidate_index] < 0.008 {
                    1.45
                } else if fractions[candidate_index] < SOFT_MIN_FRACTION {
                    0.55
                } else {
                    0.0
                };
                let oversize_bonus = if fractions[current_index] > 0.245 {
                    0.50
                } else {
                    0.0
                };
                let shape_pressure =
                    (candidate_contact as f64 - same_contact as f64) * 0.18;
                let stable_penalty = if stable_material_interior(model, signals, index, current, initial) {
                    0.22
                } else {
                    0.0
                };
                let transfer_score = candidate_score - current_score
                    + balance_pressure
                    + viability_bonus
                    + oversize_bonus
                    + shape_pressure
                    - stable_penalty;
                let proposal = (transfer_score, candidate_contact, candidate);
                if best
                    .map(|current_best| {
                        proposal.0 > current_best.0
                            || (proposal.0 == current_best.0 && proposal.1 > current_best.1)
                            || (proposal.0 == current_best.0
                                && proposal.1 == current_best.1
                                && proposal.2 < current_best.2)
                    })
                    .unwrap_or(true)
                {
                    best = Some(proposal);
                }
            }

            if let Some((score, _, replacement)) = best {
                let urgent = fractions[replacement as usize] < MIN_FRACTION
                    || fractions[current_index] > MAX_FRACTION;
                let threshold = if urgent { -0.08 } else { 0.10 };
                if score >= threshold {
                    owners[index] = replacement;
                    fractions[current_index] -= sample_fraction;
                    fractions[replacement as usize] += sample_fraction;
                    changed += 1;
                }
            }
        }

        repair_connectivity(
            topology,
            model,
            owners,
            fields,
            initial,
            cores,
            stress_axes,
            signals,
        );
        if changed == 0 {
            break;
        }
    }
}

fn smooth_plate_boundaries<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    owners: &mut [u16],
    fields: &[PlateField],
    initial: &[u16],
    cores: &[u32],
    stress_axes: [[f64; 3]; 2],
    signals: &MaterialSignals,
) {
    const MIN_FRACTION: f64 = 0.012;
    const MAX_FRACTION: f64 = 0.252;
    let total_area = (0..topology.sample_count())
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>()
        .max(1.0e-12);
    let mut protected = vec![false; owners.len()];
    for core in cores {
        protected[*core as usize] = true;
    }

    for _ in 0..10 {
        let mut fractions = plate_area_fractions(topology, owners, fields.len());
        let mut changed = 0usize;
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            if protected[index] {
                continue;
            }
            let current = owners[index];
            let sample_fraction = topology.area_steradians(sample) / total_area;
            if fractions[current as usize] - sample_fraction < MIN_FRACTION
                || !removal_preserves_local_connectivity(topology, owners, sample, current)
            {
                continue;
            }

            let mut contacts = BTreeMap::<u16, usize>::new();
            for neighbor in topology.neighbors(sample) {
                *contacts.entry(owners[*neighbor as usize]).or_insert(0) += 1;
            }
            let same_contact = contacts.get(&current).copied().unwrap_or(0);
            let Some((candidate, candidate_contact)) = contacts
                .iter()
                .filter(|(owner, _)| **owner != current)
                .map(|(owner, count)| (*owner, *count))
                .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
            else {
                continue;
            };
            if candidate_contact < 4 || same_contact > 2 {
                continue;
            }
            if fractions[candidate as usize] + sample_fraction > MAX_FRACTION {
                continue;
            }

            let position = topology.unit_position(sample);
            let current_score = field_score(
                index,
                position,
                current,
                initial[index],
                model.crust_kind[index],
                fields[current as usize],
                stress_axes,
                signals,
            );
            let candidate_score = field_score(
                index,
                position,
                candidate,
                initial[index],
                model.crust_kind[index],
                fields[candidate as usize],
                stress_axes,
                signals,
            );
            let stable_penalty = if stable_material_interior(model, signals, index, current, initial) {
                0.16
            } else {
                0.0
            };
            let shape_gain = (candidate_contact as f64 - same_contact as f64) * 0.16;
            if candidate_score - current_score + shape_gain - stable_penalty >= -0.02 {
                owners[index] = candidate;
                fractions[current as usize] -= sample_fraction;
                fractions[candidate as usize] += sample_fraction;
                changed += 1;
            }
        }
        repair_connectivity(
            topology,
            model,
            owners,
            fields,
            initial,
            cores,
            stress_axes,
            signals,
        );
        if changed == 0 {
            break;
        }
    }
}
'''
    if marker not in s:
        raise SystemExit('validate_nonempty marker not found')
    s = s.replace(marker, '\n' + insert + marker, 1)

old_tail = '''    repair_connectivity(
        topology,
        model,
        &mut owners,
        &fields,
        initial,
        &cores,
        stress_axes,
        &signals,
    );
    validate_nonempty(&owners, plate_count)?;'''
new_tail = '''    repair_connectivity(
        topology,
        model,
        &mut owners,
        &fields,
        initial,
        &cores,
        stress_axes,
        &signals,
    );
    regularize_plate_domains(
        topology,
        model,
        &mut owners,
        &fields,
        initial,
        &cores,
        stress_axes,
        &signals,
    );
    smooth_plate_boundaries(
        topology,
        model,
        &mut owners,
        &fields,
        initial,
        &cores,
        stress_axes,
        &signals,
    );
    validate_nonempty(&owners, plate_count)?;'''
if old_tail in s:
    s = s.replace(old_tail, new_tail, 1)
elif 'smooth_plate_boundaries(\n        topology,' not in s:
    raise SystemExit('final connectivity call not found')

p.write_text(s)
