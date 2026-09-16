#!/usr/bin/env python3
from pathlib import Path

path = Path('rust/interlink-worldgen/src/dynamic_plate_evolution.rs')
text = path.read_text()

insert = r'''
fn rebalance_dominant_plates<T: PlanetTopology>(
    topology: &T,
    owners: &mut [u16],
    anchors: &[u32],
    plate_count: usize,
    seed: u64,
) {
    let target_samples = ((owners.len() as f64) * 0.27).ceil() as usize;
    for pass in 0..12 {
        let mut sizes = vec![0usize; plate_count];
        for owner in owners.iter().copied() {
            sizes[owner as usize] += 1;
        }
        let oversized = (0..plate_count)
            .filter(|plate| sizes[*plate] > target_samples)
            .collect::<Vec<_>>();
        if oversized.is_empty() {
            break;
        }

        let mut proposals = Vec::<(f64, u32, u16, u16)>::new();
        for plate in oversized {
            for sample in 0..topology.sample_count() {
                let index = sample as usize;
                if owners[index] != plate as u16 || anchors[plate] == sample {
                    continue;
                }
                let mut neighbor_counts = BTreeMap::<u16, usize>::new();
                for neighbor in topology.neighbors(sample) {
                    let other = owners[*neighbor as usize];
                    if other != plate as u16 {
                        *neighbor_counts.entry(other).or_insert(0) += 1;
                    }
                }
                if neighbor_counts.is_empty() {
                    continue;
                }
                let (&target, &contact) = neighbor_counts
                    .iter()
                    .min_by(|(owner_a, contact_a), (owner_b, contact_b)| {
                        sizes[**owner_a as usize]
                            .cmp(&sizes[**owner_b as usize])
                            .then_with(|| contact_b.cmp(contact_a))
                            .then_with(|| owner_a.cmp(owner_b))
                    })
                    .unwrap();
                let target_fraction = sizes[target as usize] as f64 / owners.len() as f64;
                let score = contact as f64 * 0.28
                    + (0.27 - target_fraction).max(0.0) * 2.2
                    + unit_random(
                        seed ^ u64::from(sample).wrapping_mul(0x517c_c1b7_2722_0a95)
                            ^ (pass as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
                    ) * 0.03;
                proposals.push((score, sample, plate as u16, target));
            }
        }
        proposals.sort_by(|left, right| {
            right
                .0
                .total_cmp(&left.0)
                .then_with(|| left.1.cmp(&right.1))
        });

        let mut moved = 0usize;
        for (_, sample, from, to) in proposals {
            if owners[sample as usize] != from
                || sizes[from as usize] <= target_samples
                || sizes[to as usize] >= target_samples
            {
                continue;
            }
            owners[sample as usize] = to;
            sizes[from as usize] -= 1;
            sizes[to as usize] += 1;
            moved += 1;
        }
        repair_connectivity(topology, owners, anchors, plate_count);
        if moved == 0 {
            break;
        }
    }
}

'''
marker = 'fn split_fragments_at_modern_boundaries<T: PlanetTopology>('
if 'fn rebalance_dominant_plates<T: PlanetTopology>(' not in text:
    if marker not in text:
        raise SystemExit('missing fragment split marker')
    text = text.replace(marker, insert + marker, 1)

old = '''    for (plate, anchor) in anchors.iter().enumerate() {
        owners[*anchor as usize] = plate as u16;
    }
    repair_connectivity(topology, &mut owners, &anchors, plate_count);
    Ok(owners)
}'''
new = '''    for (plate, anchor) in anchors.iter().enumerate() {
        owners[*anchor as usize] = plate as u16;
    }
    repair_connectivity(topology, &mut owners, &anchors, plate_count);
    rebalance_dominant_plates(topology, &mut owners, &anchors, plate_count, seed);
    repair_connectivity(topology, &mut owners, &anchors, plate_count);
    Ok(owners)
}'''
if new not in text:
    if old not in text:
        raise SystemExit('missing evolve ownership return block')
    text = text.replace(old, new, 1)

path.write_text(text)
print('dynamic plate dominance balancing applied')
