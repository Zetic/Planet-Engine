use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, CrustKind, HistoricalLithosphereRequest,
    PlanetPhysicalParameters, PlanetTopology, PlateBoundaryKind,
};
use std::collections::VecDeque;

fn is_continental(kind: u8) -> bool {
    kind == CrustKind::Continental as u8 || kind == CrustKind::Transitional as u8
}

fn verify_seed(seed: &str) -> Result<(), String> {
    let topology = build_icosphere(4).map_err(|error| error.to_string())?;
    let frontend = generate_historical_frontend(
        &topology,
        &HistoricalLithosphereRequest::new(seed, 16),
        PlanetPhysicalParameters::earthlike_reference(),
    )
    .map_err(|error| error.to_string())?;
    let history = &frontend.historical;
    let tectonics = &frontend.tectonics;
    let plate_count = history.metrics.modern_plate_count as usize;
    let origin_count = history.ancestral_tectonics.plates.len();

    let mut total_area = 0.0_f64;
    let mut plate_area = vec![0.0_f64; plate_count];
    let mut plate_continental = vec![0.0_f64; plate_count];
    let mut plate_oceanic = vec![0.0_f64; plate_count];
    let mut origin_continental = vec![vec![0.0_f64; plate_count]; origin_count];

    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let area = topology.area_steradians(sample);
        let current = history.current_plate_ids[index] as usize;
        let origin = history.origin_plate_ids[index] as usize;
        total_area += area;
        plate_area[current] += area;
        if is_continental(history.crust_kind[index]) {
            plate_continental[current] += area;
            origin_continental[origin][current] += area;
        } else {
            plate_oceanic[current] += area;
        }
    }

    let mut mixed_plates = 0usize;
    let mut area_fractions = Vec::with_capacity(plate_count);
    for plate in 0..plate_count {
        let area = plate_area[plate].max(1.0e-12);
        area_fractions.push(area / total_area.max(1.0e-12));
        if plate_continental[plate] / area >= 0.08 && plate_oceanic[plate] / area >= 0.08 {
            mixed_plates += 1;
        }
    }
    let mean_area = 1.0 / plate_count as f64;
    let area_cv = (area_fractions
        .iter()
        .map(|fraction| (fraction - mean_area).powi(2))
        .sum::<f64>()
        / plate_count as f64)
        .sqrt()
        / mean_area;

    // Material-retention gate: each ancestral continental block may participate in later captures
    // and rifts, but its continental area should normally retain a dominant present-day owner.
    let mut retained_weight = 0.0_f64;
    let mut continental_weight = 0.0_f64;
    let mut retained_origins = 0usize;
    let mut continental_origins = 0usize;
    for ownership in &origin_continental {
        let area = ownership.iter().sum::<f64>();
        if area <= 1.0e-12 {
            continue;
        }
        continental_origins += 1;
        let dominant = ownership.iter().copied().fold(0.0_f64, f64::max);
        let retention = dominant / area;
        retained_weight += dominant;
        continental_weight += area;
        if retention >= 0.62 {
            retained_origins += 1;
        }
    }
    let weighted_retention = retained_weight / continental_weight.max(1.0e-12);
    let retained_origin_fraction = retained_origins as f64 / continental_origins.max(1) as f64;

    // Coast/plate relationship: ocean-continent crust transitions must include both passive
    // margins internal to plates and active margins between different plates.
    let mut passive_margin_edges = 0usize;
    let mut active_margin_edges = 0usize;
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        for neighbor in topology.neighbors(sample) {
            if *neighbor <= sample {
                continue;
            }
            let ni = *neighbor as usize;
            let continental_a = is_continental(history.crust_kind[index]);
            let continental_b = is_continental(history.crust_kind[ni]);
            if continental_a == continental_b {
                continue;
            }
            if history.current_plate_ids[index] == history.current_plate_ids[ni] {
                passive_margin_edges += 1;
            } else {
                active_margin_edges += 1;
            }
        }
    }
    let margin_total = passive_margin_edges + active_margin_edges;
    let passive_margin_fraction = passive_margin_edges as f64 / margin_total.max(1) as f64;

    let mut ocean_continent_convergent = 0usize;
    let mut convergent_total = 0usize;
    for boundary in &tectonics.boundaries {
        if boundary.kind != PlateBoundaryKind::Convergent {
            continue;
        }
        convergent_total += 1;
        let kind_a = history.crust_kind[boundary.sample_a as usize];
        let kind_b = history.crust_kind[boundary.sample_b as usize];
        if is_continental(kind_a) != is_continental(kind_b) {
            ocean_continent_convergent += 1;
        }
    }
    let cordilleran_fraction = ocean_continent_convergent as f64 / convergent_total.max(1) as f64;

    // Connected continental bodies may legitimately span several present plates (supercontinents
    // and active rifts). This metric is therefore diagnostic and only rejects near-total loss of
    // any dominant current-plate relationship.
    let mut seen = vec![false; topology.sample_count() as usize];
    let mut major_components = 0usize;
    let mut weakest_major_dominance = 1.0_f64;
    for start in 0..topology.sample_count() {
        let start_index = start as usize;
        if seen[start_index] || !is_continental(history.crust_kind[start_index]) {
            continue;
        }
        seen[start_index] = true;
        let mut queue = VecDeque::from([start]);
        let mut component_area = 0.0_f64;
        let mut by_plate = vec![0.0_f64; plate_count];
        while let Some(sample) = queue.pop_front() {
            let index = sample as usize;
            let area = topology.area_steradians(sample);
            component_area += area;
            by_plate[history.current_plate_ids[index] as usize] += area;
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                if !seen[ni] && is_continental(history.crust_kind[ni]) {
                    seen[ni] = true;
                    queue.push_back(*neighbor);
                }
            }
        }
        if component_area / total_area >= 0.018 {
            major_components += 1;
            let dominant = by_plate.iter().copied().fold(0.0_f64, f64::max) / component_area;
            weakest_major_dominance = weakest_major_dominance.min(dominant);
        }
    }

    println!(
        "plate-crust seed={seed} retention={:.1}% retained-origins={:.1}% mixed={} passive={:.1}% cordilleran={:.1}% area-cv={:.2} major={} weakest-major={:.1}%",
        weighted_retention * 100.0,
        retained_origin_fraction * 100.0,
        mixed_plates,
        passive_margin_fraction * 100.0,
        cordilleran_fraction * 100.0,
        area_cv,
        major_components,
        weakest_major_dominance * 100.0,
    );

    if weighted_retention < 0.58 {
        return Err(format!(
            "{seed}: continental ancestral material lost modern plate identity: {:.1}% weighted retention",
            weighted_retention * 100.0
        ));
    }
    if retained_origin_fraction < 0.50 {
        return Err(format!(
            "{seed}: only {:.1}% of continental ancestral blocks retain a >=62% dominant owner",
            retained_origin_fraction * 100.0
        ));
    }
    // Mixed-plate count is diagnostic only. A fixed minimum encoded the superseded final-state
    // ownership synthesizer's shape. The physical invariant is expressed below instead: the
    // evolved crust must contain both passive within-plate margins and active cross-plate margins,
    // plus ocean-continent convergent opportunities.
    if margin_total == 0 || passive_margin_edges == 0 || active_margin_edges == 0 {
        return Err(format!(
            "{seed}: crust margins do not include both passive and active plate relationships"
        ));
    }
    if !(0.10..=0.94).contains(&passive_margin_fraction) {
        return Err(format!(
            "{seed}: passive-margin share is implausibly one-sided at {:.1}%",
            passive_margin_fraction * 100.0
        ));
    }
    if ocean_continent_convergent == 0 || cordilleran_fraction < 0.025 {
        return Err(format!(
            "{seed}: convergent boundaries lost ocean-continent cordilleran opportunities: {ocean_continent_convergent}/{convergent_total}"
        ));
    }
    if area_cv < 0.12 {
        return Err(format!(
            "{seed}: modern plates remain too uniformly sized: area CV {area_cv:.3}"
        ));
    }
    if major_components > 0 && weakest_major_dominance < 0.12 {
        return Err(format!(
            "{seed}: a major continental body has essentially no dominant current plate: {:.1}%",
            weakest_major_dominance * 100.0
        ));
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let mut failures = Vec::<String>::new();
    for seed in ["interlink-wg7c", "1", "2", "plate-crust-holdout"] {
        if let Err(error) = verify_seed(seed) {
            failures.push(error);
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
