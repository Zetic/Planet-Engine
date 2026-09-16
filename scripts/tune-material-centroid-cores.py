from pathlib import Path

p = Path('rust/interlink-worldgen/src/boundary_plate_geometry.rs')
s = p.read_text()

needle = '''    let mut cores = Vec::with_capacity(plate_count);
    for plate in 0..plate_count {'''
replacement = '''    // Use the inherited material-domain centroid as the primary kinematic anchor.
    // This keeps mixed continental/oceanic plate extent represented in the field center
    // instead of pinning the plate to whichever continental interior happens to be deepest.
    let mut centroid_sums = vec![[0.0_f64; 3]; plate_count];
    let mut centroid_area = vec![0.0_f64; plate_count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let plate = owners[index] as usize;
        let area = topology.area_steradians(sample);
        let position = topology.unit_position(sample);
        centroid_sums[plate] = add(centroid_sums[plate], scale(position, area));
        centroid_area[plate] += area;
    }
    let centroids = centroid_sums
        .iter()
        .enumerate()
        .map(|(plate, sum)| {
            if centroid_area[plate] > 0.0 {
                normalize_or(*sum, [1.0, 0.0, 0.0])
            } else {
                [1.0, 0.0, 0.0]
            }
        })
        .collect::<Vec<_>>();

    let mut cores = Vec::with_capacity(plate_count);
    for plate in 0..plate_count {'''
if 'let mut centroid_sums' not in s:
    if needle not in s:
        raise SystemExit('choose_plate_cores insertion point not found')
    s = s.replace(needle, replacement, 1)

old_score = '''            let score = depth_bonus + crust_bonus - corridor_penalty + separation_bonus
                - crowd_penalty + jitter;'''
new_score = '''            let centroid_alignment = dot(position, centroids[plate]);
            let score = centroid_alignment * 0.85
                + depth_bonus * 0.55
                + crust_bonus * 0.35
                - corridor_penalty * 0.75
                + separation_bonus
                - crowd_penalty
                + jitter;'''
if old_score in s:
    s = s.replace(old_score, new_score, 1)
elif 'centroid_alignment * 0.85' not in s:
    raise SystemExit('material core score not found')

p.write_text(s)
