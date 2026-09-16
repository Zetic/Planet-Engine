use interlink_worldgen::{
    build_icosphere, generate_historical_lithosphere, HistoricalLithosphereRequest,
    PlanetPhysicalParameters, PlanetTopology,
};
use std::collections::{BTreeMap, VecDeque};

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn arc(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(a, b).clamp(-1.0, 1.0).acos()
}

fn verify_seed(seed: &str) -> Result<(), String> {
    let topology = build_icosphere(4).map_err(|error| error.to_string())?;
    let history = generate_historical_lithosphere(
        &topology,
        &HistoricalLithosphereRequest::new(seed, 16),
        PlanetPhysicalParameters::earthlike_reference(),
    )
    .map_err(|error| error.to_string())?;

    let plate_count = history.metrics.modern_plate_count as usize;
    let mut area = vec![0.0_f64; plate_count];
    let mut perimeter = vec![0.0_f64; plate_count];
    let mut boundary_contacts = vec![BTreeMap::<u16, f64>::new(); plate_count];
    let mut plate_samples = vec![Vec::<u32>::new(); plate_count];
    let mut total_area = 0.0_f64;
    let mut modern_boundary_edges = 0usize;
    let mut boundary_inside_origin = 0usize;
    let mut neck_samples = vec![0usize; plate_count];

    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let owner = history.current_plate_ids[index] as usize;
        let sample_area = topology.area_steradians(sample);
        total_area += sample_area;
        area[owner] += sample_area;
        plate_samples[owner].push(sample);

        let same_neighbors = topology
            .neighbors(sample)
            .iter()
            .filter(|neighbor| history.current_plate_ids[**neighbor as usize] as usize == owner)
            .count();
        if same_neighbors <= 2 {
            neck_samples[owner] += 1;
        }

        for neighbor in topology.neighbors(sample) {
            if *neighbor <= sample {
                continue;
            }
            let ni = *neighbor as usize;
            let other = history.current_plate_ids[ni] as usize;
            if other == owner {
                continue;
            }
            modern_boundary_edges += 1;
            if history.origin_plate_ids[ni] == history.origin_plate_ids[index] {
                boundary_inside_origin += 1;
            }
            let edge_length = arc(
                topology.unit_position(sample),
                topology.unit_position(*neighbor),
            );
            perimeter[owner] += edge_length;
            perimeter[other] += edge_length;
            *boundary_contacts[owner]
                .entry(other as u16)
                .or_insert(0.0) += edge_length;
            *boundary_contacts[other]
                .entry(owner as u16)
                .or_insert(0.0) += edge_length;
        }
    }

    if modern_boundary_edges == 0 {
        return Err(format!("{seed}: no modern boundary network"));
    }
    let migrated_fraction = boundary_inside_origin as f64 / modern_boundary_edges as f64;
    if migrated_fraction < 0.10 {
        return Err(format!(
            "{seed}: only {:.1}% of modern boundaries cut ancestral material",
            migrated_fraction * 100.0
        ));
    }

    let mut maximum_plate_fraction = 0.0_f64;
    let mut maximum_compactness = 0.0_f64;
    let mut maximum_covering_radius = 0.0_f64;
    let mut maximum_contact_dominance = 0.0_f64;
    let mut maximum_neck_fraction = 0.0_f64;

    for plate in 0..plate_count {
        if plate_samples[plate].is_empty() {
            return Err(format!("{seed}: modern plate {plate} is empty"));
        }
        let mut seen = vec![false; topology.sample_count() as usize];
        let start = plate_samples[plate][0];
        seen[start as usize] = true;
        let mut queue = VecDeque::from([start]);
        let mut reached = 0usize;
        while let Some(sample) = queue.pop_front() {
            reached += 1;
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                if !seen[ni] && history.current_plate_ids[ni] as usize == plate {
                    seen[ni] = true;
                    queue.push_back(*neighbor);
                }
            }
        }
        if reached != plate_samples[plate].len() {
            return Err(format!(
                "{seed}: modern plate {plate} is disconnected: reached={reached} expected={}",
                plate_samples[plate].len()
            ));
        }

        let fraction = area[plate] / total_area.max(1.0e-12);
        maximum_plate_fraction = maximum_plate_fraction.max(fraction);
        let spherical_denom = (area[plate] * (4.0 * std::f64::consts::PI - area[plate]))
            .max(1.0e-12);
        let compactness = perimeter[plate] * perimeter[plate] / spherical_denom;
        maximum_compactness = maximum_compactness.max(compactness);

        // Approximate the minimum spherical cap containing the plate by searching actual plate
        // samples as candidate centers. This detects horseshoes/wraps that have modest area but
        // span an implausibly large angular extent.
        let stride = (plate_samples[plate].len() / 96).max(1);
        let mut covering_radius = std::f64::consts::PI;
        for center in plate_samples[plate].iter().step_by(stride) {
            let center_position = topology.unit_position(*center);
            let radius = plate_samples[plate]
                .iter()
                .map(|sample| arc(center_position, topology.unit_position(*sample)))
                .fold(0.0_f64, f64::max);
            covering_radius = covering_radius.min(radius);
        }
        maximum_covering_radius = maximum_covering_radius.max(covering_radius);

        let contact_total = boundary_contacts[plate].values().sum::<f64>().max(1.0e-12);
        let contact_dominance = boundary_contacts[plate]
            .values()
            .copied()
            .fold(0.0_f64, f64::max)
            / contact_total;
        maximum_contact_dominance = maximum_contact_dominance.max(contact_dominance);

        let neck_fraction = neck_samples[plate] as f64 / plate_samples[plate].len() as f64;
        maximum_neck_fraction = maximum_neck_fraction.max(neck_fraction);
    }

    if maximum_plate_fraction > 0.26 {
        return Err(format!(
            "{seed}: modern plate dominates {:.1}% of the planet",
            maximum_plate_fraction * 100.0
        ));
    }
    if maximum_covering_radius > 1.95 {
        return Err(format!(
            "{seed}: plate covering radius remains wrap-prone at {:.1} degrees",
            maximum_covering_radius.to_degrees()
        ));
    }
    if maximum_compactness > 8.0 {
        return Err(format!(
            "{seed}: plate perimeter/area compactness remains excessive at {:.2}",
            maximum_compactness
        ));
    }
    if maximum_contact_dominance > 0.92 {
        return Err(format!(
            "{seed}: one plate boundary is {:.1}% dominated by a single neighbor",
            maximum_contact_dominance * 100.0
        ));
    }
    if maximum_neck_fraction > 0.08 {
        return Err(format!(
            "{seed}: narrow-neck samples reach {:.1}% of a plate",
            maximum_neck_fraction * 100.0
        ));
    }

    println!(
        "boundary-network seed={seed} migrated={:.1}% max-plate={:.1}% cap={:.1}deg compactness={:.2} contact={:.1}% neck={:.1}%",
        migrated_fraction * 100.0,
        maximum_plate_fraction * 100.0,
        maximum_covering_radius.to_degrees(),
        maximum_compactness,
        maximum_contact_dominance * 100.0,
        maximum_neck_fraction * 100.0,
    );
    Ok(())
}

fn main() -> Result<(), String> {
    for seed in [
        "interlink-wg7c",
        "1",
        "2",
        "boundary-network-holdout",
    ] {
        verify_seed(seed)?;
    }
    Ok(())
}
