from pathlib import Path

p = Path('rust/interlink-worldgen/src/boundary_plate_geometry.rs')
s = p.read_text()

# The continuous field should remember broad material identity without tracing every
# inherited cell edge. Soften the sharp same-owner lock and retain a compact core basin.
s = s.replace(
    '        0.26\n    } else if crust_kind == CrustKind::Transitional as u8 {\n        0.175\n    } else {\n        0.058',
    '        0.20\n    } else if crust_kind == CrustKind::Transitional as u8 {\n        0.135\n    } else {\n        0.050',
)
s = s.replace(
    'let core_support = ((0.30 - distance).max(0.0) / 0.30).powi(2) * 0.34;',
    'let core_support = ((0.28 - distance).max(0.0) / 0.28).powi(2) * 0.16;',
)

old_struct = '''    structure_release: Vec<f64>,
    plate_adjacency: Vec<Vec<bool>>,'''
new_struct = '''    structure_release: Vec<f64>,
    nearby_plate_mask: Vec<u64>,
    plate_adjacency: Vec<Vec<bool>>,'''
if old_struct in s:
    s = s.replace(old_struct, new_struct, 1)
elif 'nearby_plate_mask: Vec<u64>' not in s:
    raise SystemExit('MaterialSignals fields not found')

# Build a compact, local two-ring mask of which inherited modern plates actually occur
# near each sample. Unlike a global adjacency boolean, this preserves the location of
# inherited contacts and discourages a plate from becoming a one-neighbor island.
construction = '''    let rift_release = diffuse_seed_strength(topology, &rift_seeds, 4);
    let structure_release = diffuse_seed_strength(topology, &structure_seeds, 3);
    let total_area = total_area.max(1.0e-12);'''
replacement = '''    let rift_release = diffuse_seed_strength(topology, &rift_seeds, 4);
    let structure_release = diffuse_seed_strength(topology, &structure_seeds, 3);
    let mut nearby_plate_mask = vec![0_u64; count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let owner = initial[index];
        if owner < 64 {
            nearby_plate_mask[index] |= 1_u64 << owner;
        }
        for neighbor in topology.neighbors(sample) {
            let neighbor_owner = initial[*neighbor as usize];
            if neighbor_owner < 64 {
                nearby_plate_mask[index] |= 1_u64 << neighbor_owner;
            }
        }
    }
    for _ in 0..2 {
        let previous = nearby_plate_mask.clone();
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            let mut mask = previous[index];
            for neighbor in topology.neighbors(sample) {
                mask |= previous[*neighbor as usize];
            }
            nearby_plate_mask[index] = mask;
        }
    }
    let total_area = total_area.max(1.0e-12);'''
if construction in s:
    s = s.replace(construction, replacement, 1)
elif 'let mut nearby_plate_mask = vec![0_u64; count];' not in s:
    raise SystemExit('material signal construction point not found')

old_init = '''        rift_release,
        structure_release,
        plate_adjacency,'''
new_init = '''        rift_release,
        structure_release,
        nearby_plate_mask,
        plate_adjacency,'''
if old_init in s:
    s = s.replace(old_init, new_init, 1)
elif 'nearby_plate_mask,' not in s:
    raise SystemExit('MaterialSignals initializer not found')

old_candidate = '''        if signals.plate_adjacency[initial][candidate] {
            // Neighboring provisional domains may advance preferentially through explicit rift,
            // suture, capture and collision corridors, but receive no such help in stable interiors.
            signals.rift_release[sample_index] * 0.085
                + signals.structure_release[sample_index] * 0.035
        } else {
            0.0
        }'''
new_candidate = '''        if signals.plate_adjacency[initial][candidate] {
            // Preserve the *location* of inherited contacts with a compact neighborhood mask.
            // Event corridors can still move the boundary, while unrelated adjacent plates do
            // not gain a global license to cut through the whole material domain.
            let local_contact = if candidate_plate < 64
                && (signals.nearby_plate_mask[sample_index] & (1_u64 << candidate_plate)) != 0
            {
                let depth = f64::from(signals.owner_depth[sample_index].min(MAX_OWNER_DEPTH))
                    / f64::from(MAX_OWNER_DEPTH);
                (1.0 - depth).powi(2) * 0.11
            } else {
                0.0
            };
            local_contact
                + signals.rift_release[sample_index] * 0.075
                + signals.structure_release[sample_index] * 0.030
        } else {
            0.0
        }'''
if old_candidate in s:
    s = s.replace(old_candidate, new_candidate, 1)
elif 'let local_contact = if candidate_plate < 64' not in s:
    raise SystemExit('candidate ancestry branch not found')

p.write_text(s)
