use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, CrustKind, HistoricalEventKind,
    HistoricalLithosphereRequest, PlanetPhysicalParameters, PlanetTopology,
};
use std::collections::{BTreeSet, VecDeque};

fn connected_component_count<T: PlanetTopology>(
    topology: &T,
    owners: &[u16],
    plate: u16,
) -> usize {
    let mut seen = vec![false; owners.len()];
    let mut components = 0usize;
    for start in 0..topology.sample_count() {
        let index = start as usize;
        if seen[index] || owners[index] != plate {
            continue;
        }
        components += 1;
        seen[index] = true;
        let mut queue = VecDeque::from([start]);
        while let Some(sample) = queue.pop_front() {
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                if !seen[ni] && owners[ni] == plate {
                    seen[ni] = true;
                    queue.push_back(*neighbor);
                }
            }
        }
    }
    components
}

fn verify_seed(seed: &str) -> Result<(), String> {
    let topology = build_icosphere(4).map_err(|error| error.to_string())?;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let request = HistoricalLithosphereRequest::new(seed, 16);
    let first =
        generate_historical_frontend(&topology, &request, planet).map_err(|e| e.to_string())?;
    let second =
        generate_historical_frontend(&topology, &request, planet).map_err(|e| e.to_string())?;
    let history = &first.historical;

    if history.metrics.history_hash != second.historical.metrics.history_hash
        || history.current_plate_ids != second.historical.current_plate_ids
        || history.origin_plate_ids != second.historical.origin_plate_ids
    {
        return Err(format!("{seed}: forward plate evolution is not deterministic"));
    }
    if history.metrics.modern_plate_count != 16 {
        return Err(format!(
            "{seed}: requested 16 present plates but got {}",
            history.metrics.modern_plate_count
        ));
    }

    let mut plate_area = vec![0.0_f64; 16];
    let mut total_area = 0.0_f64;
    let mut migrated_boundary_edges = 0usize;
    let mut inherited_boundary_edges = 0usize;
    let mut young_created_crust = 0usize;
    let mut oceanic_samples = 0usize;
    let mut oceanic_age_min = f32::INFINITY;
    let mut oceanic_age_max = f32::NEG_INFINITY;

    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let owner = history.current_plate_ids[index] as usize;
        if owner >= 16 {
            return Err(format!("{seed}: invalid modern owner {owner}"));
        }
        let area = topology.area_steradians(sample);
        plate_area[owner] += area;
        total_area += area;

        if history.crust_kind[index] == CrustKind::Oceanic as u8 {
            oceanic_samples += 1;
            let age = history.crust_birth_age_myr[index];
            oceanic_age_min = oceanic_age_min.min(age);
            oceanic_age_max = oceanic_age_max.max(age);
        }
        if history.crust_kind[index] != CrustKind::Continental as u8
            && history.crust_birth_age_myr[index] <= 0.001
        {
            young_created_crust += 1;
        }

        for neighbor in topology.neighbors(sample) {
            if *neighbor <= sample {
                continue;
            }
            let ni = *neighbor as usize;
            if history.current_plate_ids[index] == history.current_plate_ids[ni] {
                continue;
            }
            if history.origin_plate_ids[index] == history.origin_plate_ids[ni] {
                migrated_boundary_edges += 1;
            } else {
                inherited_boundary_edges += 1;
            }
        }
    }

    if plate_area.iter().any(|area| *area <= 0.0) {
        return Err(format!("{seed}: forward evolution eliminated a present plate"));
    }
    for plate in 0..16_u16 {
        let components = connected_component_count(&topology, &history.current_plate_ids, plate);
        if components != 1 {
            return Err(format!(
                "{seed}: present plate {plate} has {components} disconnected components"
            ));
        }
    }

    let moved_fraction = migrated_boundary_edges as f64
        / (migrated_boundary_edges + inherited_boundary_edges).max(1) as f64;
    if moved_fraction < 0.08 {
        return Err(format!(
            "{seed}: only {:.1}% of final boundaries cut ancestral material; forward evolution did not move enough geometry",
            moved_fraction * 100.0
        ));
    }
    if young_created_crust == 0 {
        return Err(format!(
            "{seed}: forward extension created no new transitional/oceanic crust"
        ));
    }
    if oceanic_samples == 0 || oceanic_age_max - oceanic_age_min < 20.0 {
        return Err(format!("{seed}: oceanic chronology collapsed during forward evolution"));
    }

    let mut kinds = BTreeSet::new();
    for event in &history.events {
        kinds.insert(event.kind as u8);
    }
    let has_extension = kinds.contains(&(HistoricalEventKind::Rift as u8))
        || kinds.contains(&(HistoricalEventKind::Spreading as u8));
    let has_convergence = kinds.contains(&(HistoricalEventKind::Collision as u8))
        || kinds.contains(&(HistoricalEventKind::Subduction as u8))
        || kinds.contains(&(HistoricalEventKind::Accretion as u8));
    if !has_extension || !has_convergence {
        return Err(format!(
            "{seed}: forward event ledger lacks both extension and convergence"
        ));
    }

    let closure = plate_area.iter().sum::<f64>() / total_area.max(1.0e-12);
    if (closure - 1.0).abs() > 1.0e-12 {
        return Err(format!("{seed}: present plate area does not close"));
    }

    println!(
        "forward-plate-evolution seed={seed} migrated={:.1}% created={} ocean-age={:.1}..{:.1}Myr events={} history={}",
        moved_fraction * 100.0,
        young_created_crust,
        oceanic_age_min,
        oceanic_age_max,
        history.metrics.event_count,
        history.metrics.history_hash_hex(),
    );
    Ok(())
}

fn main() -> Result<(), String> {
    let mut failures = Vec::new();
    for seed in [
        "interlink-wg7c",
        "1",
        "2",
        "forward-plate-evolution-holdout",
    ] {
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
