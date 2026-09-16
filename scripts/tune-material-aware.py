from pathlib import Path

p = Path('rust/interlink-worldgen/src/boundary_plate_geometry.rs')
s = p.read_text()

# Reduce purely decorative anisotropy. Large-scale irregularity should come from
# inherited material and tectonic corridors, not arbitrary field distortion.
s = s.replace(
    '(area_ratio.ln() * 0.105).clamp(-0.18, 0.22)',
    '(area_ratio.ln() * 0.050).clamp(-0.10, 0.14)',
)
s = s.replace(
    '* 0.24,\n                bend:',
    '* 0.10,\n                bend:',
)
s = s.replace(
    '* 0.18,\n            }',
    '* 0.07,\n            }',
)
s = s.replace(
    '        0.34\n    } else if crust_kind == CrustKind::Transitional as u8 {\n        0.23\n    } else {\n        0.075',
    '        0.30\n    } else if crust_kind == CrustKind::Transitional as u8 {\n        0.20\n    } else {\n        0.065',
)

# A modern kinematic field may move relative to its inherited material, but its
# authoritative connectivity seed must remain inside the provisional material domain.
# Otherwise repair_connectivity can erase the inherited domain and leave only the
# forced single-sample core, which is the collapse seen in the geometry gate.
old_core_guard = '''            if used[index] {
                continue;
            }
            let alignment = dot(topology.unit_position(sample), field.center);'''
new_core_guard = '''            if used[index] || initial[index] as usize != plate {
                continue;
            }
            let alignment = dot(topology.unit_position(sample), field.center);'''
if old_core_guard in s:
    s = s.replace(old_core_guard, new_core_guard, 1)
elif new_core_guard not in s:
    raise SystemExit('choose_field_cores guard not found')

marker = '\nfn repair_connectivity<T: PlanetTopology>(\n'
if 'fn calibrate_area_biases<T: PlanetTopology>' not in s:
    insert = '''
fn target_plate_fractions(signals: &MaterialSignals, plate_count: usize) -> Vec<f64> {
    let mean = 1.0 / plate_count.max(1) as f64;
    let mut targets = signals
        .plate_area_fraction
        .iter()
        .map(|fraction| (fraction * 0.75 + mean * 0.25).max(0.018))
        .collect::<Vec<_>>();
    let total = targets.iter().sum::<f64>().max(1.0e-12);
    for target in &mut targets {
        *target /= total;
    }
    targets
}

fn connected_plate_fractions<T: PlanetTopology>(
    topology: &T,
    owners: &[u16],
    cores: &[u32],
) -> Vec<f64> {
    let mut total_area = 0.0_f64;
    for sample in 0..topology.sample_count() {
        total_area += topology.area_steradians(sample);
    }
    let total_area = total_area.max(1.0e-12);
    let mut fractions = vec![0.0_f64; cores.len()];
    let mut seen = vec![false; owners.len()];

    for (plate, core) in cores.iter().copied().enumerate() {
        let plate_id = plate as u16;
        if owners[core as usize] != plate_id {
            continue;
        }
        seen.fill(false);
        seen[core as usize] = true;
        let mut queue = VecDeque::from([core]);
        let mut area = 0.0_f64;
        while let Some(sample) = queue.pop_front() {
            area += topology.area_steradians(sample);
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                if !seen[ni] && owners[ni] == plate_id {
                    seen[ni] = true;
                    queue.push_back(*neighbor);
                }
            }
        }
        fractions[plate] = area / total_area;
    }
    fractions
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
    for _ in 0..120 {
        let owners = assign_from_fields(
            topology,
            model,
            fields,
            initial,
            cores,
            stress_axes,
            signals,
        );
        let current = connected_plate_fractions(topology, &owners, cores);
        let mut maximum_error = 0.0_f64;
        let mut minimum_fraction = 1.0_f64;
        for plate in 0..fields.len() {
            let error = targets[plate] - current[plate];
            maximum_error = maximum_error.max(error.abs());
            minimum_fraction = minimum_fraction.min(current[plate]);
            let mut correction = (error * 1.8).clamp(-0.018, 0.018);
            if current[plate] < 0.008 {
                correction = correction.max(0.014);
            }
            fields[plate].area_bias =
                (fields[plate].area_bias + correction).clamp(-0.90, 0.90);
        }
        if minimum_fraction >= 0.012 && maximum_error < 0.007 {
            break;
        }
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

p.write_text(s)
