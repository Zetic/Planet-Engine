from pathlib import Path

p = Path('rust/interlink-worldgen/src/boundary_plate_geometry.rs')
s = p.read_text()

s = s.replace(
    '(area_ratio.ln() * 0.105).clamp(-0.18, 0.22)',
    '(area_ratio.ln() * 0.055).clamp(-0.12, 0.16)',
)
s = s.replace(
    '* 0.24,\n                bend:',
    '* 0.12,\n                bend:',
)
s = s.replace(
    '* 0.18,\n            }',
    '* 0.08,\n            }',
)
s = s.replace(
    '        0.34\n    } else if crust_kind == CrustKind::Transitional as u8 {\n        0.23\n    } else {\n        0.075',
    '        0.30\n    } else if crust_kind == CrustKind::Transitional as u8 {\n        0.20\n    } else {\n        0.065',
)

marker = '\nfn repair_connectivity<T: PlanetTopology>(\n'
if 'fn calibrate_area_biases<T: PlanetTopology>' not in s:
    insert = '''
fn target_plate_fractions(signals: &MaterialSignals, plate_count: usize) -> Vec<f64> {
    let mean = 1.0 / plate_count.max(1) as f64;
    signals
        .plate_area_fraction
        .iter()
        .map(|fraction| fraction * 0.72 + mean * 0.28)
        .collect()
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
    for _ in 0..8 {
        let owners = assign_from_fields(
            topology,
            model,
            fields,
            initial,
            cores,
            stress_axes,
            signals,
        );
        let mut areas = vec![0.0_f64; fields.len()];
        let mut total_area = 0.0_f64;
        for sample in 0..topology.sample_count() {
            let area = topology.area_steradians(sample);
            total_area += area;
            areas[owners[sample as usize] as usize] += area;
        }
        let total_area = total_area.max(1.0e-12);
        for plate in 0..fields.len() {
            let current = (areas[plate] / total_area).max(1.0e-5);
            let correction = (targets[plate] / current).ln().clamp(-1.0, 1.0) * 0.070;
            fields[plate].area_bias = (fields[plate].area_bias + correction).clamp(-0.24, 0.26);
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
