#!/usr/bin/env python3
from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if new in text:
        return
    if old not in text:
        raise SystemExit(f"missing replacement anchor in {path}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


def regex_once(path: str, pattern: str, replacement: str) -> None:
    p = Path(path)
    text = p.read_text()
    if re.search(pattern, text, flags=re.S) is None:
        if replacement.strip() in text:
            return
        raise SystemExit(f"missing regex anchor in {path}: {pattern[:120]!r}")
    p.write_text(re.sub(pattern, replacement, text, count=1, flags=re.S))


replace_once(
    "rust/interlink-worldgen/src/lib.rs",
    "mod historical_causal;\nmod historical_epochs;",
    "mod dynamic_plate_evolution;\nmod historical_causal;\nmod historical_epochs;",
)
replace_once(
    "rust/interlink-worldgen/src/lib.rs",
    "pub const WORLDGEN_ENGINE_VERSION: u32 = 17;",
    "pub const WORLDGEN_ENGINE_VERSION: u32 = 18;",
)

replace_once(
    "rust/interlink-worldgen/src/historical_api.rs",
    "    derive_stage_seed, geology, historical_causal, historical_epochs, historical_frontend,\n",
    "    derive_stage_seed, dynamic_plate_evolution, geology, historical_causal, historical_epochs, historical_frontend,\n",
)
replace_once(
    "rust/interlink-worldgen/src/historical_api.rs",
    "    historical_epochs::evolve_historical_lithosphere(topology, base, request.seed.as_str())\n",
    "    let lineage = historical_epochs::evolve_historical_lithosphere(\n        topology,\n        base,\n        request.seed.as_str(),\n    )?;\n    dynamic_plate_evolution::evolve_modern_plate_geometry(\n        topology,\n        lineage,\n        request.seed.as_str(),\n        parameters,\n    )\n",
)

continental = r'''fn build_continental_assemblies<T: PlanetTopology>(
    topology: &T,
    ancestral: &TectonicModel,
    seed: u64,
) -> (Vec<bool>, Vec<u16>) {
    let count = ancestral.plates.len();
    let total_area = ancestral
        .plates
        .iter()
        .map(|plate| plate.area_steradians)
        .sum::<f64>()
        .max(1.0e-12);
    let target_fraction = 0.50 + (unit_random(seed ^ 0x3c79_ac49_2ba7_b653) - 0.5) * 0.08;
    let target_area = total_area * target_fraction.clamp(0.46, 0.54);

    let mut pair_kinds = BTreeMap::<(u16, u16), [u32; 3]>::new();
    let mut plate_perimeter = vec![0_u32; count];
    let mut adjacency = vec![BTreeSet::<usize>::new(); count];
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
        plate_perimeter[boundary.plate_a as usize] += 1;
        plate_perimeter[boundary.plate_b as usize] += 1;
        adjacency[boundary.plate_a as usize].insert(boundary.plate_b as usize);
        adjacency[boundary.plate_b as usize].insert(boundary.plate_a as usize);
    }

    // Continental material starts from a small number of coherent proto-continental nuclei.
    // Growth is contiguous across the ancestral plate graph; isolated random carrier plates are
    // not permitted. Later rifting may split these masses, but archipelagos are no longer an
    // initial-condition artifact.
    let nucleus_target = (3
        + (unit_random(seed ^ 0xa076_1d64_78bd_642f) * 4.0).floor() as usize)
        .clamp(2, 6)
        .min(count.max(1));
    let mut nuclei = Vec::<usize>::new();
    if count > 0 {
        let first = (0..count)
            .max_by(|left, right| {
                let left_score = (ancestral.plates[*left].area_steradians / total_area).sqrt() * 0.72
                    + unit_random(seed ^ (*left as u64).wrapping_mul(0xe703_7ed1_a0b4_28db)) * 0.28;
                let right_score = (ancestral.plates[*right].area_steradians / total_area).sqrt() * 0.72
                    + unit_random(seed ^ (*right as u64).wrapping_mul(0xe703_7ed1_a0b4_28db)) * 0.28;
                left_score.total_cmp(&right_score).then_with(|| right.cmp(left))
            })
            .unwrap_or(0);
        nuclei.push(first);
    }
    while nuclei.len() < nucleus_target {
        let mut best = None;
        let mut best_score = f64::NEG_INFINITY;
        for plate in 0..count {
            if nuclei.contains(&plate) {
                continue;
            }
            let position = topology.unit_position(ancestral.plates[plate].seed_sample);
            let separation = nuclei
                .iter()
                .map(|nucleus| {
                    arc_radians(
                        position,
                        topology.unit_position(ancestral.plates[*nucleus].seed_sample),
                    )
                })
                .fold(std::f64::consts::PI, f64::min);
            let area_bonus = (ancestral.plates[plate].area_steradians / total_area).sqrt() * 0.22;
            let jitter = unit_random(seed ^ (plate as u64).wrapping_mul(0x8ebc_6af0_9c88_c6e3)) * 0.04;
            let score = separation + area_bonus + jitter;
            if score > best_score || (score == best_score && Some(plate) < best) {
                best_score = score;
                best = Some(plate);
            }
        }
        let Some(plate) = best else { break };
        nuclei.push(plate);
    }

    let mut carriers = vec![false; count];
    let mut assemblies = vec![u16::MAX; count];
    let mut carrier_area = 0.0_f64;
    let mut assembly_area = vec![0.0_f64; nuclei.len()];
    for (assembly, plate) in nuclei.iter().copied().enumerate() {
        if !carriers[plate] {
            carriers[plate] = true;
            assemblies[plate] = assembly as u16;
            carrier_area += ancestral.plates[plate].area_steradians;
            assembly_area[assembly] += ancestral.plates[plate].area_steradians;
        }
    }

    while carrier_area < target_area {
        let mut best: Option<(f64, usize, u16)> = None;
        for plate in 0..count {
            if carriers[plate] {
                continue;
            }
            let mut by_assembly = BTreeMap::<u16, [u32; 4]>::new();
            for neighbor in &adjacency[plate] {
                if !carriers[*neighbor] {
                    continue;
                }
                let assembly = assemblies[*neighbor];
                let pair = if plate < *neighbor {
                    (plate as u16, *neighbor as u16)
                } else {
                    (*neighbor as u16, plate as u16)
                };
                let kinds = pair_kinds.get(&pair).copied().unwrap_or([0; 3]);
                let totals = by_assembly.entry(assembly).or_insert([0; 4]);
                totals[0] += kinds[0];
                totals[1] += kinds[1];
                totals[2] += kinds[2];
                totals[3] += kinds.iter().sum::<u32>();
            }
            for (assembly, totals) in by_assembly {
                let shared = f64::from(totals[3].max(1));
                let convergence = f64::from(totals[0]) / shared;
                let divergence = f64::from(totals[1]) / shared;
                let transform = f64::from(totals[2]) / shared;
                let contact = shared / f64::from(plate_perimeter[plate].max(1));
                let assembly_fraction = assembly_area[assembly as usize] / target_area.max(1.0e-12);
                let dominance_penalty = ((assembly_fraction - 0.52).max(0.0) / 0.48).powi(2);
                let jitter = unit_random(
                    seed ^ (plate as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15)
                        ^ u64::from(assembly).wrapping_mul(0xbf58_476d_1ce4_e5b9),
                ) * 0.025;
                let score = contact * 1.65
                    + convergence * 0.34
                    + transform * 0.10
                    - divergence * 0.58
                    - dominance_penalty * 0.25
                    + jitter;
                let candidate = (score, plate, assembly);
                if best
                    .map(|current| {
                        candidate.0 > current.0
                            || (candidate.0 == current.0
                                && (candidate.1, candidate.2) < (current.1, current.2))
                    })
                    .unwrap_or(true)
                {
                    best = Some(candidate);
                }
            }
        }
        let Some((_, plate, assembly)) = best else { break };
        carriers[plate] = true;
        assemblies[plate] = assembly;
        carrier_area += ancestral.plates[plate].area_steradians;
        assembly_area[assembly as usize] += ancestral.plates[plate].area_steradians;
    }

    // Non-carriers never participate in continental depth propagation, but give every ancestral
    // plate a dense assembly id for deterministic diagnostics and downstream indexing.
    let mut next_assembly = nuclei.len() as u16;
    for plate in 0..count {
        if assemblies[plate] == u16::MAX {
            assemblies[plate] = next_assembly;
            next_assembly = next_assembly.saturating_add(1);
        }
    }
    (carriers, assemblies)
}
'''
regex_once(
    "rust/interlink-worldgen/src/historical_lithosphere.rs",
    r"fn build_continental_assemblies\(ancestral: &TectonicModel, seed: u64\) -> \(Vec<bool>, Vec<u16>\) \{.*?\n\}\n\nfn build_plate_owned_crust",
    continental + "\nfn build_plate_owned_crust",
)
replace_once(
    "rust/interlink-worldgen/src/historical_lithosphere.rs",
    "    let (plate_carriers, plate_assemblies) = build_continental_assemblies(ancestral, seed);",
    "    let (plate_carriers, plate_assemblies) = build_continental_assemblies(topology, ancestral, seed);",
)

frontend_block = r'''    let ancestral_count = historical.ancestral_tectonics.plates.len();
    let mut velocity_sum = vec![[0.0_f64; 3]; modern_count];
    let mut ancestry_area = vec![0.0_f64; modern_count];
    let mut contribution_area = vec![vec![0.0_f64; ancestral_count]; modern_count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let origin = historical.origin_plate_ids[index] as usize;
        let current = historical.current_plate_ids[index] as usize;
        if origin >= ancestral_count || current >= modern_count {
            return Err(WorldgenError::InvalidTectonics(
                "historical ownership contains an invalid plate id",
            ));
        }
        let area = topology.area_steradians(sample);
        let ancestral = &historical.ancestral_tectonics.plates[origin];
        for component in 0..3 {
            velocity_sum[current][component] += ancestral.angular_velocity_rad_per_myr[component] * area;
        }
        ancestry_area[current] += area;
        contribution_area[current][origin] += area;
    }

    let mut representative_seed = vec![u32::MAX; modern_count];
    let mut representative_support = vec![f64::NEG_INFINITY; modern_count];
    let mut representative_pole = vec![[0.0_f64, 0.0, 1.0]; modern_count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let current = historical.current_plate_ids[index] as usize;
        let origin = historical.origin_plate_ids[index] as usize;
        let same_neighbors = topology
            .neighbors(sample)
            .iter()
            .filter(|neighbor| historical.current_plate_ids[**neighbor as usize] as usize == current)
            .count() as f64;
        let ancestry_support = contribution_area[current][origin] / ancestry_area[current].max(1.0e-12);
        let support = same_neighbors + ancestry_support * 0.45;
        if support > representative_support[current]
            || (support == representative_support[current] && sample < representative_seed[current])
        {
            representative_support[current] = support;
            representative_seed[current] = sample;
            representative_pole[current] = historical.ancestral_tectonics.plates[origin].euler_pole;
        }
    }
'''
regex_once(
    "rust/interlink-worldgen/src/historical_frontend.rs",
    r"    let ancestral_count = historical\.ancestral_tectonics\.plates\.len\(\);.*?\n    let mut plate_area = vec!\[0\.0_f64; modern_count\];",
    frontend_block + "\n    let mut plate_area = vec![0.0_f64; modern_count];",
)

replace_once(
    "rust/interlink-worldgen-cli/examples/historical_lithosphere_acceptance.rs",
    "    let mut origin_to_current = vec![u16::MAX; history.ancestral_tectonics.plates.len()];",
    "    let mut origin_to_current = vec![BTreeSet::<u16>::new(); history.ancestral_tectonics.plates.len()];",
)
replace_once(
    "rust/interlink-worldgen-cli/examples/historical_lithosphere_acceptance.rs",
    "        let mapped_current = &mut origin_to_current[origin as usize];\n        if *mapped_current == u16::MAX {\n            *mapped_current = current;\n        } else if *mapped_current != current {\n            return Err(format!(\"{seed}: one ancestral plate maps to multiple modern owners without split provenance\"));\n        }",
    "        origin_to_current[origin as usize].insert(current);",
)
replace_once(
    "rust/interlink-worldgen-cli/examples/historical_lithosphere_acceptance.rs",
    "            if parent.origin_plate_id != fragment.origin_plate_id\n                || parent.current_plate_id != fragment.current_plate_id\n            {\n                return Err(format!(\n                    \"{seed}: fragment {} changed material ownership across its parent edge\",\n                    fragment.id\n                ));\n            }",
    "            if parent.origin_plate_id != fragment.origin_plate_id {\n                return Err(format!(\n                    \"{seed}: fragment {} changed ancestral provenance across its parent edge\",\n                    fragment.id\n                ));\n            }\n            if parent.current_plate_id != fragment.current_plate_id\n                && fragment.capture_age_myr.is_none()\n            {\n                return Err(format!(\n                    \"{seed}: fragment {} changed modern ownership without capture age provenance\",\n                    fragment.id\n                ));\n            }",
)
replace_once(
    "rust/interlink-worldgen-cli/examples/historical_lithosphere_acceptance.rs",
    "    let mut origin_discontinuity_edges = 0_u32;",
    "    let mut origin_discontinuity_edges = 0_u32;\n    let mut modern_boundary_inside_origin_edges = 0_u32;",
)
replace_once(
    "rust/interlink-worldgen-cli/examples/historical_lithosphere_acceptance.rs",
    "            if history.current_plate_ids[ni] == current && history.origin_plate_ids[ni] != origin {\n                origin_discontinuity_edges += 1;\n            }",
    "            if history.current_plate_ids[ni] == current && history.origin_plate_ids[ni] != origin {\n                origin_discontinuity_edges += 1;\n            }\n            if history.current_plate_ids[ni] != current && history.origin_plate_ids[ni] == origin {\n                modern_boundary_inside_origin_edges += 1;\n            }",
)
replace_once(
    "rust/interlink-worldgen-cli/examples/historical_lithosphere_acceptance.rs",
    "    if internal_fragment_edges == 0 || origin_discontinuity_edges == 0 {\n        return Err(format!(\n            \"{seed}: no fossil material discontinuities survived inside modern plates\"\n        ));\n    }",
    "    if internal_fragment_edges == 0 || origin_discontinuity_edges == 0 {\n        return Err(format!(\n            \"{seed}: no fossil material discontinuities survived inside modern plates\"\n        ));\n    }\n    if modern_boundary_inside_origin_edges == 0 {\n        return Err(format!(\n            \"{seed}: modern plate boundaries remained locked to ancestral plate edges\"\n        ));\n    }",
)

capture_block = r'''    let mut capture_events_by_current = vec![0_usize; history.metrics.modern_plate_count as usize];
    for event in history
        .events
        .iter()
        .filter(|event| event.kind == HistoricalEventKind::Capture)
    {
        if usize::from(event.fragment_a) >= history.fragments.len()
            || usize::from(event.fragment_b) >= history.fragments.len()
        {
            return Err(format!(
                "{seed}: capture event {} references invalid fragments",
                event.id
            ));
        }
        let current = history.fragments[event.fragment_b as usize].current_plate_id;
        if current >= history.metrics.modern_plate_count {
            return Err(format!(
                "{seed}: capture event {} resolves to an invalid modern owner",
                event.id
            ));
        }
        capture_events_by_current[current as usize] += 1;
    }
    for (current, origins) in &modern_origins {
        if origins.len() > 1 && capture_events_by_current[*current as usize] == 0 {
            return Err(format!(
                "{seed}: modern plate {current} consolidates {} ancestral plates but has no capture provenance",
                origins.len()
            ));
        }
    }
'''
regex_once(
    "rust/interlink-worldgen-cli/examples/historical_lithosphere_acceptance.rs",
    r"    let mut capture_events_by_current = vec!\[0_usize; history\.metrics\.modern_plate_count as usize\];.*?\n    for boundary in &tectonics\.boundaries \{",
    capture_block + "\n    for boundary in &tectonics.boundaries {",
)

replace_once(
    ".github/workflows/ci.yml",
    "assert p['run']['engine_version']==17;",
    "assert p['run']['engine_version']==18;",
)

print("dynamic plate evolution cutover applied")
