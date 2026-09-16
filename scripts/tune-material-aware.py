from pathlib import Path

p = Path('rust/interlink-worldgen/src/boundary_plate_geometry.rs')
s = p.read_text()

# Keep the material field authoritative, but reduce purely decorative anisotropy so
# boundary complexity comes primarily from tectonic/material corridors rather than noise.
s = s.replace(
    '(area_ratio.ln() * 0.105).clamp(-0.18, 0.22)',
    '(area_ratio.ln() * 0.050).clamp(-0.10, 0.14)',
)
s = s.replace(
    '* 0.24,\n                bend:',
    '* 0.08,\n                bend:',
)
s = s.replace(
    '* 0.18,\n            }',
    '* 0.05,\n            }',
)
s = s.replace(
    '        0.34\n    } else if crust_kind == CrustKind::Transitional as u8 {\n        0.23\n    } else {\n        0.075',
    '        0.29\n    } else if crust_kind == CrustKind::Transitional as u8 {\n        0.19\n    } else {\n        0.060',
)

marker = '\nfn repair_connectivity<T: PlanetTopology>(\n'
if 'fn calibrate_area_biases<T: PlanetTopology>' not in s:
    insert = '''
fn target_plate_fractions(signals: &MaterialSignals, plate_count: usize) -> Vec<f64> {
    let mean = 1.0 / plate_count.max(1) as f64;
    let mut targets = signals
        .plate_area_fraction
        .iter()
        .map(|fraction| (fraction * 0.68 + mean * 0.32).max(0.015))
        .collect::<Vec<_>>();
    let total = targets.iter().sum::<f64>().max(1.0e-12);
    for target in &mut targets {
        *target /= total;
    }
    targets
}

fn measured_plate_fractions<T: PlanetTopology>(
    topology: &T,
    owners: &[u16],
    plate_count: usize,
) -> Vec<f64> {
    let mut areas = vec![0.0_f64; plate_count];
    let mut total_area = 0.0_f64;
    for sample in 0..topology.sample_count() {
        let area = topology.area_steradians(sample);
        total_area += area;
        areas[owners[sample as usize] as usize] += area;
    }
    let total_area = total_area.max(1.0e-12);
    areas.into_iter().map(|area| area / total_area).collect()
}

fn calibrate_area_biases<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    fields: &mut [PlateField],
    initial: &[u16],
    cores: &[u32],
    stress_axes: [[f64; 3]; 2],
    signals: &MaterialSignals,
) {
    let targets = target_plate_fractions(signals, fields.len());
    for _ in 0..28 {
        let owners = assign_from_fields(
            topology,
            model,
            fields,
            initial,
            cores,
            stress_axes,
            signals,
        );
        let current = measured_plate_fractions(topology, &owners, fields.len());
        let mut maximum_log_error = 0.0_f64;
        for plate in 0..fields.len() {
            let safe_current = current[plate].max(2.5e-5);
            let log_error = (targets[plate] / safe_current).ln();
            maximum_log_error = maximum_log_error.max(log_error.abs());
            let gain = if current[plate] < 0.008 { 0.16 } else { 0.095 };
            let correction = log_error.clamp(-1.5, 1.5) * gain;
            fields[plate].area_bias =
                (fields[plate].area_bias + correction).clamp(-0.48, 0.58);
        }
        if maximum_log_error < 0.13 {
            break;
        }
    }
}

fn regularize_boundaries<T: PlanetTopology>(
    topology: &T,
    model: &HistoricalLithosphereModel,
    owners: &mut [u16],
    fields: &[PlateField],
    initial: &[u16],
    cores: &[u32],
    stress_axes: [[f64; 3]; 2],
    signals: &MaterialSignals,
) {
    let mut core_owner = vec![None::<u16>; owners.len()];
    for (plate, core) in cores.iter().copied().enumerate() {
        core_owner[core as usize] = Some(plate as u16);
    }

    for _ in 0..5 {
        let fractions = measured_plate_fractions(topology, owners, fields.len());
        let mut proposals = owners.to_vec();
        let mut changed = false;

        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            if core_owner[index].is_some() {
                continue;
            }
            let current = owners[index];
            let mut contacts = BTreeMap::<u16, usize>::new();
            for neighbor in topology.neighbors(sample) {
                *contacts.entry(owners[*neighbor as usize]).or_insert(0) += 1;
            }
            let same_contact = contacts.get(&current).copied().unwrap_or(0);
            if same_contact >= 3 || fractions[current as usize] <= 0.0105 {
                continue;
            }

            let candidate = contacts
                .iter()
                .filter(|(owner, _)| **owner != current)
                .max_by(|(owner_a, count_a), (owner_b, count_b)| {
                    count_a.cmp(count_b).then_with(|| owner_b.cmp(owner_a))
                })
                .map(|(owner, count)| (*owner, *count));
            let Some((candidate, candidate_contact)) = candidate else {
                continue;
            };
            if candidate_contact < 3 {
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
            let coherence_bonus =
                (candidate_contact as f64 - same_contact as f64).max(0.0) * 0.055;
            if candidate_score + coherence_bonus >= current_score {
                proposals[index] = candidate;
                changed = true;
            }
        }

        if !changed {
            break;
        }
        owners.copy_from_slice(&proposals);
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
    }
}
'''
    if marker not in s:
        raise SystemExit('repair_connectivity marker not found')
    s = s.replace(marker, '\n' + insert + marker, 1)

needle = '''    let stress_axes = [
        random_unit_vector(stage_seed, 0xd6e8_feb8_6659_fd93),
        random_unit_vector(stage_seed, 0xa5a3_56d5_2f62_56d5),
    ];
    let mut owners = assign_from_fields('''
replacement = '''    let stress_axes = [
        random_unit_vector(stage_seed, 0xd6e8_feb8_6659_fd93),
        random_unit_vector(stage_seed, 0xa5a3_56d5_2f62_56d5),
    ];
    calibrate_area_biases(
        topology,
        model,
        &mut fields,
        initial,
        &cores,
        stress_axes,
        &signals,
    );
    let mut owners = assign_from_fields('''
if 'calibrate_area_biases(\n        topology,' not in s:
    if needle not in s:
        raise SystemExit('synthesis calibration insertion point not found')
    s = s.replace(needle, replacement, 1)

repair_call = '''    repair_connectivity(
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
regularized_call = '''    repair_connectivity(
        topology,
        model,
        &mut owners,
        &fields,
        initial,
        &cores,
        stress_axes,
        &signals,
    );
    regularize_boundaries(
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
if 'regularize_boundaries(\n        topology,' not in s:
    if repair_call not in s:
        raise SystemExit('final connectivity call not found')
    s = s.replace(repair_call, regularized_call, 1)

p.write_text(s)
