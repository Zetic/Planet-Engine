use interlink_worldgen::{
    build_icosphere, generate_historical_lithosphere, CrustKind, HistoricalLithosphereRequest,
    PlanetPhysicalParameters, PlanetTopology,
};
use std::collections::{BTreeSet, VecDeque};

fn verify_seed(seed: &str) -> Result<(), String> {
    let topology = build_icosphere(4).map_err(|error| error.to_string())?;
    let history = generate_historical_lithosphere(
        &topology,
        &HistoricalLithosphereRequest::new(seed, 16),
        PlanetPhysicalParameters::earthlike_reference(),
    )
    .map_err(|error| error.to_string())?;

    let mut modern_boundary_edges = 0usize;
    let mut boundary_inside_origin = 0usize;
    let mut origin_to_current = vec![BTreeSet::<u16>::new(); history.metrics.ancestral_plate_count as usize];
    let mut current_area = vec![0.0_f64; history.metrics.modern_plate_count as usize];
    let mut total_area = 0.0_f64;

    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let current = history.current_plate_ids[index];
        let origin = history.origin_plate_ids[index];
        let area = topology.area_steradians(sample);
        total_area += area;
        current_area[current as usize] += area;
        origin_to_current[origin as usize].insert(current);
        for neighbor in topology.neighbors(sample) {
            if *neighbor <= sample {
                continue;
            }
            let ni = *neighbor as usize;
            if history.current_plate_ids[ni] != current {
                modern_boundary_edges += 1;
                if history.origin_plate_ids[ni] == origin {
                    boundary_inside_origin += 1;
                }
            }
        }
    }

    if modern_boundary_edges == 0 || boundary_inside_origin == 0 {
        return Err(format!(
            "{seed}: modern boundaries never migrated through ancestral material"
        ));
    }
    let migrated_fraction = boundary_inside_origin as f64 / modern_boundary_edges as f64;
    if migrated_fraction < 0.025 {
        return Err(format!(
            "{seed}: only {:.2}% of modern boundary edges cut ancestral material",
            migrated_fraction * 100.0
        ));
    }

    let split_origins = origin_to_current.iter().filter(|owners| owners.len() > 1).count();
    if split_origins < 2 {
        return Err(format!(
            "{seed}: modern evolution split only {split_origins} ancestral plates across current owners"
        ));
    }

    for plate in 0..history.metrics.modern_plate_count {
        let Some(start) = history.current_plate_ids.iter().position(|owner| *owner == plate) else {
            return Err(format!("{seed}: modern plate {plate} is empty"));
        };
        let mut seen = vec![false; history.current_plate_ids.len()];
        let mut queue = VecDeque::from([start as u32]);
        seen[start] = true;
        let mut reached = 0usize;
        while let Some(sample) = queue.pop_front() {
            reached += 1;
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                if !seen[ni] && history.current_plate_ids[ni] == plate {
                    seen[ni] = true;
                    queue.push_back(*neighbor);
                }
            }
        }
        let expected = history.current_plate_ids.iter().filter(|owner| **owner == plate).count();
        if reached != expected {
            return Err(format!(
                "{seed}: modern plate {plate} is disconnected: reached={reached} expected={expected}"
            ));
        }
    }

    let maximum_plate_fraction = current_area
        .iter()
        .map(|area| *area / total_area.max(1.0e-12))
        .fold(0.0_f64, f64::max);
    if maximum_plate_fraction > 0.30 {
        return Err(format!(
            "{seed}: a modern plate dominates {:.1}% of the planet",
            maximum_plate_fraction * 100.0
        ));
    }

    let mut visited = vec![false; topology.sample_count() as usize];
    let mut continental_components = Vec::<f64>::new();
    for start in 0..topology.sample_count() {
        let si = start as usize;
        if visited[si] || history.crust_kind[si] == CrustKind::Oceanic as u8 {
            continue;
        }
        visited[si] = true;
        let mut queue = VecDeque::from([start]);
        let mut area = 0.0_f64;
        while let Some(sample) = queue.pop_front() {
            area += topology.area_steradians(sample);
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                if !visited[ni] && history.crust_kind[ni] != CrustKind::Oceanic as u8 {
                    visited[ni] = true;
                    queue.push_back(*neighbor);
                }
            }
        }
        continental_components.push(area);
    }
    continental_components.sort_by(|left, right| right.total_cmp(left));
    let continental_area = continental_components.iter().sum::<f64>().max(1.0e-12);
    let major_area = continental_components.iter().take(6).sum::<f64>();
    let satellite_fraction = (continental_area - major_area) / continental_area;
    if continental_components.len() > 10 || satellite_fraction > 0.08 {
        return Err(format!(
            "{seed}: continental material remains archipelago-dominated: components={} satellite={:.1}%",
            continental_components.len(),
            satellite_fraction * 100.0
        ));
    }

    println!(
        "dynamic-plate-geometry seed={seed} migrated-boundary={:.1}% split-origins={} max-plate={:.1}% continental-components={} satellite={:.1}%",
        migrated_fraction * 100.0,
        split_origins,
        maximum_plate_fraction * 100.0,
        continental_components.len(),
        satellite_fraction * 100.0,
    );
    Ok(())
}

fn main() -> Result<(), String> {
    for seed in ["interlink-wg7c", "1", "2", "dynamic-geometry-holdout"] {
        verify_seed(seed)?;
    }
    Ok(())
}
