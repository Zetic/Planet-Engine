from pathlib import Path
import re

path = Path('rust/interlink-worldgen/src/historical_lithosphere.rs')
text = path.read_text()
text = text.replace('use std::collections::{BTreeMap, BTreeSet};', 'use std::collections::{BTreeMap, BTreeSet, VecDeque};')
text = text.replace('pub const HISTORICAL_LITHOSPHERE_STAGE_VERSION: u32 = 1;', 'pub const HISTORICAL_LITHOSPHERE_STAGE_VERSION: u32 = 2;')
text = text.replace('worldgen:geology:historical-lithosphere:v1', 'worldgen:geology:historical-lithosphere:v2')
text = text.replace('worldgen:geology:historical-lithosphere:crust:v1', 'worldgen:geology:historical-lithosphere:crust:v2')
text = text.replace('worldgen:geology:historical-lithosphere:modern:v1', 'worldgen:geology:historical-lithosphere:modern:v2')

new_crust = r'''fn build_continental_assemblies(
    ancestral: &TectonicModel,
    seed: u64,
) -> (Vec<bool>, Vec<u16>) {
    let count = ancestral.plates.len();
    let mut carriers = (0..count)
        .map(|plate| {
            unit_random(seed ^ (plate as u64).wrapping_mul(0xa076_1d64_78bd_642f)) > 0.55
        })
        .collect::<Vec<_>>();
    if !carriers.iter().any(|carrier| *carrier) {
        if let Some((largest, _)) = ancestral
            .plates
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.area_steradians.total_cmp(&right.area_steradians))
        {
            carriers[largest] = true;
        }
    }

    let mut pair_kinds = BTreeMap::<(u16, u16), [u32; 3]>::new();
    for boundary in &ancestral.boundaries {
        let pair = if boundary.plate_a < boundary.plate_b {
            (boundary.plate_a, boundary.plate_b)
        } else {
            (boundary.plate_b, boundary.plate_a)
        };
        let counts = pair_kinds.entry(pair).or_insert([0; 3]);
        match boundary.kind {
            PlateBoundaryKind::Convergent => counts[0] += 1,
            PlateBoundaryKind::Divergent => counts[1] += 1,
            PlateBoundaryKind::Transform => counts[2] += 1,
        }
    }

    let mut assemblies = (0..count as u16).collect::<Vec<_>>();
    for ((plate_a, plate_b), counts) in pair_kinds {
        if !carriers[plate_a as usize] || !carriers[plate_b as usize] {
            continue;
        }
        let total = f64::from(counts.iter().sum::<u32>().max(1));
        let convergence_fraction = f64::from(counts[0]) / total;
        let divergence_fraction = f64::from(counts[1]) / total;
        let transform_fraction = f64::from(counts[2]) / total;
        let stream = seed
            ^ u64::from(plate_a).wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ u64::from(plate_b).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        let weld = divergence_fraction < 0.34
            && (convergence_fraction >= 0.34
                || (transform_fraction >= 0.55 && unit_random(stream) > 0.32));
        if !weld {
            continue;
        }
        let keep = assemblies[plate_a as usize].min(assemblies[plate_b as usize]);
        let remove = assemblies[plate_a as usize].max(assemblies[plate_b as usize]);
        if keep == remove {
            continue;
        }
        for assembly in &mut assemblies {
            if *assembly == remove {
                *assembly = keep;
            }
        }
    }

    let mut compact = BTreeMap::<u16, u16>::new();
    for assembly in &assemblies {
        if !compact.contains_key(assembly) {
            let next = compact.len() as u16;
            compact.insert(*assembly, next);
        }
    }
    for assembly in &mut assemblies {
        *assembly = compact[assembly];
    }
    (carriers, assemblies)
}

fn build_plate_owned_crust<T: PlanetTopology>(
    topology: &T,
    fragment_ids: &[u16],
    fragments: &mut [CrustFragment],
    seed: u64,
    planet: PlanetPhysicalParameters,
    ancestral: &TectonicModel,
) -> (Vec<u8>, Vec<f32>) {
    let (plate_carriers, plate_assemblies) = build_continental_assemblies(ancestral, seed);
    let count = topology.sample_count() as usize;
    let sample_origin = fragment_ids
        .iter()
        .map(|fragment| fragments[*fragment as usize].origin_plate_id)
        .collect::<Vec<_>>();
    let sample_assembly = sample_origin
        .iter()
        .map(|origin| plate_assemblies[*origin as usize])
        .collect::<Vec<_>>();
    let sample_carrier = sample_origin
        .iter()
        .map(|origin| plate_carriers[*origin as usize])
        .collect::<Vec<_>>();

    // Distance inward from the edge of an assembled continental material domain. Internal
    // fragment and welded ancestral-plate contacts are deliberately ignored here: they remain
    // provenance/suture structure, not automatic seaways.
    let mut depth = vec![u16::MAX; count];
    let mut queue = VecDeque::<u32>::new();
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        if !sample_carrier[index] {
            continue;
        }
        let assembly = sample_assembly[index];
        let edge = topology.neighbors(sample).iter().any(|neighbor| {
            let ni = *neighbor as usize;
            !sample_carrier[ni] || sample_assembly[ni] != assembly
        });
        if edge {
            depth[index] = 0;
            queue.push_back(sample);
        }
    }
    while let Some(sample) = queue.pop_front() {
        let index = sample as usize;
        let assembly = sample_assembly[index];
        let next_depth = depth[index].saturating_add(1);
        for neighbor in topology.neighbors(sample) {
            let ni = *neighbor as usize;
            if !sample_carrier[ni] || sample_assembly[ni] != assembly || depth[ni] != u16::MAX {
                continue;
            }
            depth[ni] = next_depth;
            queue.push_back(*neighbor);
        }
    }

    let assembly_count = plate_assemblies.iter().copied().max().map(|value| value as usize + 1).unwrap_or(0);
    let mut max_depth = vec![0_u16; assembly_count];
    for sample in 0..count {
        if sample_carrier[sample] && depth[sample] != u16::MAX {
            let assembly = sample_assembly[sample] as usize;
            max_depth[assembly] = max_depth[assembly].max(depth[sample]);
        }
    }

    let divergent_samples = ancestral
        .boundaries
        .iter()
        .filter(|boundary| boundary.kind == PlateBoundaryKind::Divergent)
        .flat_map(|boundary| [boundary.sample_a, boundary.sample_b])
        .collect::<Vec<_>>();

    let mut crust_kind = vec![CrustKind::Oceanic as u8; count];
    let mut birth_age = vec![0.0_f32; count];
    let mut kind_counts = vec![[0_u32; 3]; fragments.len()];
    let mut thickness_sums = vec![0.0_f64; fragments.len()];
    let mut density_sums = vec![0.0_f64; fragments.len()];

    for sample in 0..topology.sample_count() {
        let sample_index = sample as usize;
        let fragment = fragment_ids[sample_index] as usize;
        let edge_warp = unit_random(
            seed ^ u64::from(sample).wrapping_mul(0xe703_7ed1_a0b4_28db) ^ (fragment as u64),
        );
        let kind = if sample_carrier[sample_index] {
            let assembly = sample_assembly[sample_index] as usize;
            let maximum = max_depth[assembly].max(1);
            let inward = if depth[sample_index] == u16::MAX {
                1.0
            } else {
                f64::from(depth[sample_index]) / f64::from(maximum)
            };
            let continental_cut = (0.18 + (edge_warp - 0.5) * 0.08).clamp(0.12, 0.24);
            let transitional_cut = (0.045 + (edge_warp - 0.5) * 0.035).clamp(0.02, 0.08);
            if inward >= continental_cut {
                CrustKind::Continental
            } else if inward >= transitional_cut {
                CrustKind::Transitional
            } else {
                CrustKind::Oceanic
            }
        } else {
            CrustKind::Oceanic
        };
        crust_kind[sample_index] = kind as u8;

        let local_random = unit_random(
            seed ^ u64::from(sample).wrapping_mul(0x8ebc_6af0_9c88_c6e3) ^ 0x243f_6a88_85a3_08d3,
        );
        let (age, thickness, density, kind_index) = match kind {
            CrustKind::Continental => (
                650.0 + 2800.0 * unit_random(seed ^ (fragment as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)),
                31.0 + 13.0 * local_random,
                2720.0 + 90.0 * (1.0 - local_random),
                2,
            ),
            CrustKind::Transitional => (
                120.0 + 900.0 * unit_random(seed ^ (fragment as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9)),
                15.0 + 14.0 * local_random,
                2820.0 + 110.0 * (1.0 - local_random),
                1,
            ),
            CrustKind::Oceanic => {
                let position = topology.unit_position(sample);
                let nearest_ridge_rad = divergent_samples
                    .iter()
                    .map(|ridge| arc_radians(position, topology.unit_position(*ridge)))
                    .fold(f64::INFINITY, f64::min);
                let age = if nearest_ridge_rad.is_finite() {
                    let distance_km = nearest_ridge_rad * planet.radius_m / 1000.0;
                    (distance_km / 32.0).clamp(0.0, 220.0)
                } else {
                    110.0 + 90.0 * local_random
                };
                (age, 5.8 + 1.8 * local_random, 2870.0 + 120.0 * (1.0 - local_random), 0)
            }
        };
        birth_age[sample_index] = age as f32;
        kind_counts[fragment][kind_index] += 1;
        thickness_sums[fragment] += thickness;
        density_sums[fragment] += density;
    }

    for (index, fragment) in fragments.iter_mut().enumerate() {
        let dominant = kind_counts[index]
            .iter()
            .enumerate()
            .max_by_key(|(kind, count)| (**count, *kind))
            .map(|(kind, _)| kind)
            .unwrap_or(0);
        fragment.dominant_crust_kind = match dominant {
            2 => CrustKind::Continental as u8,
            1 => CrustKind::Transitional as u8,
            _ => CrustKind::Oceanic as u8,
        };
        let count = f64::from(fragment.sample_count.max(1));
        fragment.mean_thickness_km = (thickness_sums[index] / count) as f32;
        fragment.mean_density_kg_per_m3 = (density_sums[index] / count) as f32;
        fragment.birth_age_myr = if fragment.dominant_crust_kind == CrustKind::Continental as u8 {
            (650.0 + 2800.0 * unit_random(seed ^ (index as u64).wrapping_mul(0x517c_c1b7_2722_0a95))) as f32
        } else {
            (80.0 + 140.0 * unit_random(seed ^ (index as u64).wrapping_mul(0x6a09_e667_f3bc_c909))) as f32
        };
    }

    (crust_kind, birth_age)
}

'''
text, count = re.subn(r'fn build_plate_owned_crust<T: PlanetTopology>\(.*?\n}\n\nfn group_ancestral_plates', new_crust + 'fn group_ancestral_plates', text, flags=re.S)
assert count == 1, count

new_group = r'''fn group_ancestral_plates(
    ancestral: &TectonicModel,
    modern_plate_count: u16,
    seed: u64,
) -> Vec<u16> {
    let count = ancestral.plates.len();
    let mut groups = (0..count as u16).collect::<Vec<_>>();
    let mut active = count;
    let total_area = ancestral
        .plates
        .iter()
        .map(|plate| plate.area_steradians)
        .sum::<f64>()
        .max(1.0e-12);

    while active > modern_plate_count as usize {
        let mut area = BTreeMap::<u16, f64>::new();
        let mut velocity_sum = BTreeMap::<u16, [f64; 3]>::new();
        let mut members = BTreeMap::<u16, u32>::new();
        for (plate_index, plate) in ancestral.plates.iter().enumerate() {
            let group = groups[plate_index];
            *area.entry(group).or_insert(0.0) += plate.area_steradians;
            *members.entry(group).or_insert(0) += 1;
            let sum = velocity_sum.entry(group).or_insert([0.0; 3]);
            for axis in 0..3 {
                sum[axis] += plate.angular_velocity_rad_per_myr[axis] * plate.area_steradians;
            }
        }

        let mut contacts = BTreeMap::<(u16, u16), u32>::new();
        let mut perimeter = BTreeMap::<u16, u32>::new();
        for boundary in &ancestral.boundaries {
            let group_a = groups[boundary.plate_a as usize];
            let group_b = groups[boundary.plate_b as usize];
            if group_a == group_b {
                continue;
            }
            let pair = if group_a < group_b { (group_a, group_b) } else { (group_b, group_a) };
            *contacts.entry(pair).or_insert(0) += 1;
            *perimeter.entry(group_a).or_insert(0) += 1;
            *perimeter.entry(group_b).or_insert(0) += 1;
        }

        let enclosed_after_merge = |merge_a: u16, merge_b: u16| -> usize {
            let keep = merge_a.min(merge_b);
            let remove = merge_a.max(merge_b);
            let mut neighbors = BTreeMap::<u16, BTreeSet<u16>>::new();
            for &(left, right) in contacts.keys() {
                let mapped_left = if left == remove { keep } else { left };
                let mapped_right = if right == remove { keep } else { right };
                if mapped_left == mapped_right {
                    continue;
                }
                neighbors.entry(mapped_left).or_default().insert(mapped_right);
                neighbors.entry(mapped_right).or_default().insert(mapped_left);
            }
            neighbors
                .iter()
                .filter(|(group, adjacent)| **group != keep && adjacent.len() == 1 && adjacent.contains(&keep))
                .count()
        };

        let mut best: Option<(f64, u16, u16)> = None;
        for (&(group_a, group_b), &shared_contact) in &contacts {
            let area_a = area[&group_a];
            let area_b = area[&group_b];
            let velocity_a_sum = velocity_sum[&group_a];
            let velocity_b_sum = velocity_sum[&group_b];
            let velocity_a = [
                velocity_a_sum[0] / area_a.max(1.0e-12),
                velocity_a_sum[1] / area_a.max(1.0e-12),
                velocity_a_sum[2] / area_a.max(1.0e-12),
            ];
            let velocity_b = [
                velocity_b_sum[0] / area_b.max(1.0e-12),
                velocity_b_sum[1] / area_b.max(1.0e-12),
                velocity_b_sum[2] / area_b.max(1.0e-12),
            ];
            let velocity_cost = vector_distance(velocity_a, velocity_b);
            let merged_area = area_a + area_b;
            let merged_fraction = merged_area / total_area;
            let balance_cost = (area_a - area_b).abs() / merged_area.max(1.0e-12);
            let contact_fraction = f64::from(shared_contact)
                / f64::from(perimeter[&group_a].min(perimeter[&group_b]).max(1));
            let compactness_cost = 1.0 - contact_fraction.clamp(0.0, 1.0);
            let dominance_cost = ((merged_fraction - 0.30).max(0.0) / 0.20).powi(2);
            let enclosure_cost = enclosed_after_merge(group_a, group_b) as f64;
            let member_total = f64::from(members[&group_a] + members[&group_b]);
            let member_imbalance = f64::from(members[&group_a].max(members[&group_b])) / member_total.max(1.0);
            let tie = unit_random(
                seed ^ u64::from(group_a).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                    ^ u64::from(group_b).wrapping_mul(0xbf58_476d_1ce4_e5b9),
            ) * 1.0e-6;
            let score = velocity_cost * 32.0
                + compactness_cost * 0.30
                + balance_cost * 0.05
                + dominance_cost * 1.80
                + enclosure_cost * 4.0
                + member_imbalance * 0.04
                + tie;
            let candidate = (score, group_a.min(group_b), group_a.max(group_b));
            if best.map(|current| candidate < current).unwrap_or(true) {
                best = Some(candidate);
            }
        }
        let Some((_, keep, remove)) = best else {
            break;
        };
        for group in &mut groups {
            if *group == remove {
                *group = keep;
            }
        }
        active -= 1;
    }

    let mut compact = BTreeMap::<u16, u16>::new();
    for group in &groups {
        if !compact.contains_key(group) {
            let next = compact.len() as u16;
            compact.insert(*group, next);
        }
    }
    groups.into_iter().map(|group| compact[&group]).collect()
}

'''
text, count = re.subn(r'fn group_ancestral_plates\(.*?\n}\n\nfn event_kind_for_boundary', new_group + 'fn event_kind_for_boundary', text, flags=re.S)
assert count == 1, count

# Add topology/assembly regressions alongside the existing unit tests.
needle = '''    #[test]\n    fn historical_lithosphere_preserves_three_identity_levels() {'''
insert = r'''    fn single_neighbor_plate_count<T: PlanetTopology>(
        topology: &T,
        plate_ids: &[u16],
        plate_count: u16,
    ) -> usize {
        let mut neighbors = vec![BTreeSet::<u16>::new(); plate_count as usize];
        for sample in 0..topology.sample_count() {
            let plate = plate_ids[sample as usize];
            for neighbor in topology.neighbors(sample) {
                let other = plate_ids[*neighbor as usize];
                if other != plate {
                    neighbors[plate as usize].insert(other);
                }
            }
        }
        neighbors.iter().filter(|adjacent| adjacent.len() == 1).count()
    }

    fn continental_components<T: PlanetTopology>(topology: &T, crust_kind: &[u8]) -> (usize, f64) {
        let mut visited = vec![false; topology.sample_count() as usize];
        let mut components = 0usize;
        let mut largest_area = 0.0f64;
        let total_area = (0..topology.sample_count())
            .map(|sample| topology.area_steradians(sample))
            .sum::<f64>()
            .max(1.0e-12);
        for start in 0..topology.sample_count() {
            let si = start as usize;
            if visited[si] || crust_kind[si] == CrustKind::Oceanic as u8 {
                continue;
            }
            components += 1;
            visited[si] = true;
            let mut queue = VecDeque::from([start]);
            let mut area = 0.0;
            while let Some(sample) = queue.pop_front() {
                area += topology.area_steradians(sample);
                for neighbor in topology.neighbors(sample) {
                    let ni = *neighbor as usize;
                    if !visited[ni] && crust_kind[ni] != CrustKind::Oceanic as u8 {
                        visited[ni] = true;
                        queue.push_back(*neighbor);
                    }
                }
            }
            largest_area = largest_area.max(area);
        }
        (components, largest_area / total_area)
    }

    #[test]
    fn modern_plate_synthesis_avoids_enclosed_single_neighbor_plates() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        for seed in ["plate-topology-a", "plate-topology-b", "interlink-wg7c", "plate-topology-d"] {
            let model = generate_historical_lithosphere(
                &topology,
                &HistoricalLithosphereRequest::new(seed, 16),
                planet,
            )
            .unwrap();
            assert_eq!(
                single_neighbor_plate_count(&topology, &model.current_plate_ids, 16),
                0,
                "seed {seed} produced a modern plate enclosed by one neighbor"
            );
        }
    }

    #[test]
    fn continental_assemblies_are_broader_than_fragment_islands() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        for seed in ["continent-assembly-a", "interlink-wg7c", "continent-assembly-c"] {
            let model = generate_historical_lithosphere(
                &topology,
                &HistoricalLithosphereRequest::new(seed, 16),
                planet,
            )
            .unwrap();
            let (components, largest_fraction) = continental_components(&topology, &model.crust_kind);
            assert!(components < model.metrics.fragment_count as usize / 2, "seed {seed} remained fragment-island dominated");
            assert!(largest_fraction >= 0.06, "seed {seed} lacked a substantial assembled continent");
        }
    }

    #[test]
    fn historical_lithosphere_preserves_three_identity_levels() {'''
assert needle in text
text = text.replace(needle, insert)
path.write_text(text)
