use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_tectonics, CrustKind, GeodesicTopology,
    GeologyRequest, PlanetPhysicalParameters, TectonicsRequest,
};
use std::collections::{BTreeSet, VecDeque};
use std::f64::consts::PI;

#[derive(Clone, Debug)]
struct ComponentStats {
    area_sr: f64,
    perimeter_rad: f64,
    diameter_rad: f64,
    plate_count: usize,
}

fn arc(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
        .clamp(-1.0, 1.0)
        .acos()
}

fn component_stats(
    topology: &GeodesicTopology,
    mask: &[bool],
    plate_ids: &[u16],
) -> Vec<ComponentStats> {
    let mut visited = vec![false; mask.len()];
    let mut components = Vec::new();
    for start in 0..mask.len() {
        if !mask[start] || visited[start] {
            continue;
        }
        visited[start] = true;
        let mut queue = VecDeque::from([start]);
        let mut samples = Vec::new();
        while let Some(sample) = queue.pop_front() {
            samples.push(sample);
            for neighbor in topology.neighbors_of(sample as u32) {
                let index = *neighbor as usize;
                if mask[index] && !visited[index] {
                    visited[index] = true;
                    queue.push_back(index);
                }
            }
        }

        let mut area_sr = 0.0_f64;
        let mut perimeter_rad = 0.0_f64;
        let mut plates = BTreeSet::new();
        for sample in &samples {
            area_sr += topology.dual_area_steradians()[*sample];
            plates.insert(plate_ids[*sample]);
            for (neighbor, arc_length) in topology
                .neighbors_of(*sample as u32)
                .iter()
                .zip(topology.neighbor_arc_lengths_of(*sample as u32).iter())
            {
                if !mask[*neighbor as usize] {
                    perimeter_rad += *arc_length;
                }
            }
        }

        let positions = topology.positions();
        let first = samples[0];
        let farthest_from_first = samples
            .iter()
            .copied()
            .max_by(|a, b| {
                arc(positions[first], positions[*a])
                    .total_cmp(&arc(positions[first], positions[*b]))
            })
            .unwrap_or(first);
        let farthest = samples
            .iter()
            .copied()
            .max_by(|a, b| {
                arc(positions[farthest_from_first], positions[*a])
                    .total_cmp(&arc(positions[farthest_from_first], positions[*b]))
            })
            .unwrap_or(farthest_from_first);
        let diameter_rad = arc(positions[farthest_from_first], positions[farthest]);
        components.push(ComponentStats {
            area_sr,
            perimeter_rad,
            diameter_rad,
            plate_count: plates.len(),
        });
    }
    components
}

fn main() -> Result<(), String> {
    let topology = build_icosphere(4).map_err(|error| error.to_string())?;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let seeds = [
        "continental-shape-a",
        "continental-shape-b",
        "continental-shape-c",
        "continental-shape-d",
        "interlink-wg7c",
        "3",
    ];
    let total_area = 4.0 * PI;
    let mut hierarchy_worlds = 0_usize;
    let mut elongated_worlds = 0_usize;
    let mut noncompact_worlds = 0_usize;
    let mut multiplate_worlds = 0_usize;
    let mut cv_sum = 0.0_f64;

    for seed in seeds {
        let tectonics = generate_tectonics(&topology, &TectonicsRequest::new(seed, 16), planet)
            .map_err(|error| error.to_string())?;
        let geology =
            generate_crust_and_history(&topology, &tectonics, &GeologyRequest::new(seed), planet)
                .map_err(|error| error.to_string())?;
        let continental = geology
            .crust_kind
            .iter()
            .map(|kind| *kind == CrustKind::Continental as u8)
            .collect::<Vec<_>>();
        let mut components = component_stats(&topology, &continental, &tectonics.plate_ids)
            .into_iter()
            .filter(|component| component.area_sr >= total_area * 0.0025)
            .collect::<Vec<_>>();
        components.sort_by(|a, b| b.area_sr.total_cmp(&a.area_sr));
        if components.len() < 2 {
            return Err(format!(
                "{seed}: continental assembly collapsed to fewer than two significant components"
            ));
        }

        let areas = components
            .iter()
            .map(|component| component.area_sr)
            .collect::<Vec<_>>();
        let mean = areas.iter().sum::<f64>() / areas.len() as f64;
        let variance =
            areas.iter().map(|area| (area - mean).powi(2)).sum::<f64>() / areas.len() as f64;
        let cv = variance.sqrt() / mean.max(1.0e-12);
        cv_sum += cv;
        let median = areas[areas.len() / 2];
        let hierarchy = areas[0] / median.max(1.0e-12);
        let max_elongation = components
            .iter()
            .map(|component| {
                let equivalent_radius = (component.area_sr / PI).sqrt().max(1.0e-6);
                component.diameter_rad / (2.0 * equivalent_radius)
            })
            .fold(0.0_f64, f64::max);
        let max_compactness = components
            .iter()
            .map(|component| {
                component.perimeter_rad.powi(2) / (4.0 * PI * component.area_sr.max(1.0e-12))
            })
            .fold(0.0_f64, f64::max);
        let multiplate = components
            .iter()
            .any(|component| component.area_sr >= total_area * 0.015 && component.plate_count >= 2);

        hierarchy_worlds += usize::from(hierarchy >= 1.80);
        elongated_worlds += usize::from(max_elongation >= 1.25);
        noncompact_worlds += usize::from(max_compactness >= 1.15);
        multiplate_worlds += usize::from(multiplate);
        println!(
            "{seed}: components={} CV={cv:.3} largest/median={hierarchy:.3} max_elong={max_elongation:.3} max_compact={max_compactness:.3} multiplate={multiplate}",
            components.len()
        );
    }

    let coupling_seed = "continental-tectonic-coupling";
    let tectonics_12 =
        generate_tectonics(&topology, &TectonicsRequest::new(coupling_seed, 12), planet)
            .map_err(|error| error.to_string())?;
    let tectonics_20 =
        generate_tectonics(&topology, &TectonicsRequest::new(coupling_seed, 20), planet)
            .map_err(|error| error.to_string())?;
    let geology_12 = generate_crust_and_history(
        &topology,
        &tectonics_12,
        &GeologyRequest::new(coupling_seed),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let geology_20 = generate_crust_and_history(
        &topology,
        &tectonics_20,
        &GeologyRequest::new(coupling_seed),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let changed_fraction = geology_12
        .crust_kind
        .iter()
        .zip(geology_20.crust_kind.iter())
        .filter(|(a, b)| a != b)
        .count() as f64
        / geology_12.crust_kind.len() as f64;
    println!("plate-layout coupling changed crust-kind fraction={changed_fraction:.4}");

    let world_count = seeds.len() as f64;
    let mean_cv = cv_sum / world_count;
    if hierarchy_worlds < 4 {
        return Err(format!(
            "only {hierarchy_worlds}/6 worlds show a material continental size hierarchy"
        ));
    }
    if elongated_worlds < 4 {
        return Err(format!(
            "only {elongated_worlds}/6 worlds contain a materially elongated continental component"
        ));
    }
    if noncompact_worlds < 4 {
        return Err(format!(
            "only {noncompact_worlds}/6 worlds contain a non-circular continental outline"
        ));
    }
    if multiplate_worlds < 3 {
        return Err(format!(
            "only {multiplate_worlds}/6 worlds assemble a major continent across tectonic domains"
        ));
    }
    if mean_cv < 0.55 {
        return Err(format!(
            "ensemble continental component-size CV remained too uniform: {mean_cv:.3}"
        ));
    }
    if changed_fraction < 0.03 {
        return Err(format!("continental partition remains effectively independent of tectonic plate layout: changed={changed_fraction:.4}"));
    }
    Ok(())
}
