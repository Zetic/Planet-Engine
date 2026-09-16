#!/usr/bin/env python3
from pathlib import Path

path = Path('rust/interlink-worldgen/src/dynamic_plate_evolution.rs')
text = path.read_text()

old_single = '''        if by_owner.len() == 1 {
            model.fragments[fragment_id as usize].current_plate_id = *by_owner.keys().next().unwrap();
            continue;
        }
'''
new_single = '''        if by_owner.len() == 1 {
            let owner = *by_owner.keys().next().unwrap();
            let index = fragment_id as usize;
            let previous_owner = model.fragments[index].current_plate_id;
            if owner != previous_owner {
                let capture_age = (6.0
                    + unit_random(
                        seed ^ u64::from(fragment_id).wrapping_mul(0xd6e8_feb8_6659_fd93),
                    ) * 54.0) as f32;
                model.fragments[index].current_plate_id = owner;
                model.fragments[index].capture_age_myr = Some(capture_age);
                let origin = model.fragments[index].origin_plate_id;
                let geometry = model.fragments[index].seed_sample;
                model.events.push(HistoricalTectonicEvent {
                    id: model.events.len() as u32,
                    kind: HistoricalEventKind::Capture,
                    epoch: 7,
                    age_myr: capture_age,
                    plate_a: origin,
                    plate_b: origin,
                    fragment_a: fragment_id,
                    fragment_b: fragment_id,
                    displacement_km: 0.0,
                    strength: 0.18,
                    geometry_sample_a: geometry,
                    geometry_sample_b: geometry,
                });
            }
            continue;
        }
'''
if new_single not in text:
    if old_single not in text:
        raise SystemExit('missing single-owner transfer block')
    text = text.replace(old_single, new_single, 1)

old_dominant = '''        let dominant_owner = by_owner
            .iter()
            .max_by_key(|(owner, samples)| (samples.len(), std::cmp::Reverse(**owner)))
            .map(|(owner, _)| *owner)
            .unwrap_or(parent.current_plate_id);
        model.fragments[fragment_id as usize].current_plate_id = dominant_owner;

'''
if old_dominant in text:
    text = text.replace(old_dominant, '', 1)

path.write_text(text)
print('dynamic fragment transfer provenance fixed')
