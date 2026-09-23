use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, inherit_historical_identity, CrustKind,
    HistoricalEventKind, HistoricalLithosphereRequest, PlanetPhysicalParameters, PlanetTopology,
    HISTORICAL_EPOCH_COUNT,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

// Permanent historical-material acceptance: material identity must survive the complete
// historical front end, forward plate evolution, compatibility geology projection, and
// coarse-to-fine inheritance. Spatial migration of present boundaries is gated separately by
// forward_plate_evolution_acceptance; ancestry labels move with material and therefore are not a
// valid fixed spatial reference frame for measuring boundary migration.
fn verify_seed(seed: &str) -> Result<(), String> {
    let coarse_level = 4;
    let topology = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(6).map_err(|error| error.to_string())?;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let request = HistoricalLithosphereRequest::new(seed, 16);
    let frontend = generate_historical_frontend(&topology, &request, planet)
        .map_err(|error| error.to_string())?;
    let repeat = generate_historical_frontend(&topology, &request, planet)
        .map_err(|error| error.to_string())?;

    let history = &frontend.historical;
    let tectonics = &frontend.tectonics;
    let geology = &frontend.geology;
    let count = topology.sample_count() as usize;

    if history.metrics.history_hash != repeat.historical.metrics.history_hash
        || tectonics.metrics.tectonic_hash != repeat.tectonics.metrics.tectonic_hash
        || geology.metrics.geology_hash != repeat.geology.metrics.geology_hash
    {
        return Err(format!("{seed}: historical frontend is not deterministic"));
    }
    if history.origin_plate_ids.len() != count
        || history.fragment_ids.len() != count
        || history.current_plate_ids.len() != count
        || history.crust_kind.len() != count
        || history.crust_birth_age_myr.len() != count
    {
        return Err(format!(
            "{seed}: historical material fields do not cover the sphere"
        ));
    }
    if history.metrics.fragment_count < history.metrics.ancestral_plate_count {
        return Err(format!(
            "{seed}: material lineage lost ancestral plate representation"
        ));
    }
    if history.metrics.modern_plate_count == 0 {
        return Err(format!("{seed}: forward history produced no present tectonic plates"));
    }
    if tectonics.plate_ids != history.current_plate_ids {
        return Err(format!(
            "{seed}: modern tectonic ownership diverged from historical current ownership"
        ));
    }
    if geology.crust_kind != history.crust_kind {
        return Err(format!(
            "{seed}: geology regenerated crust kind instead of consuming historical material"
        ));
    }

    let crust_fraction_sum = history.metrics.continental_area_fraction
        + history.metrics.transitional_area_fraction
        + history.metrics.oceanic_area_fraction;
    if (crust_fraction_sum - 1.0).abs() > 1.0e-10 {
        return Err(format!(
            "{seed}: crust material-area fractions do not close: {crust_fraction_sum:.12}"
        ));
    }

    let mut parented_fragment_ids = BTreeSet::<u16>::new();
    for (index, fragment) in history.fragments.iter().enumerate() {
        if usize::from(fragment.id) != index {
            return Err(format!(
                "{seed}: fragment ids are not dense lineage indices"
            ));
        }
        if let Some(parent_id) = fragment.parent_fragment_id {
            parented_fragment_ids.insert(fragment.id);
            if parent_id >= fragment.id || usize::from(parent_id) >= history.fragments.len() {
                return Err(format!(
                    "{seed}: fragment {} has an invalid/cyclic parent {parent_id}",
                    fragment.id
                ));
            }
            let parent = &history.fragments[parent_id as usize];
            if parent.origin_plate_id != fragment.origin_plate_id {
                return Err(format!(
                    "{seed}: fragment {} changed ancestral provenance across its parent edge",
                    fragment.id
                ));
            }
        }
    }
    if parented_fragment_ids.is_empty() {
        return Err(format!(
            "{seed}: forward tectonic history produced no parent/child material lineage"
        ));
    }

    let mut split_events = 0_usize;
    let mut split_event_children = BTreeSet::<u16>::new();
    for (event_index, event) in history.events.iter().enumerate() {
        if event.id as usize != event_index {
            return Err(format!(
                "{seed}: event ids are not dense deterministic indices"
            ));
        }
        if event.epoch >= HISTORICAL_EPOCH_COUNT {
            return Err(format!(
                "{seed}: event {} lies outside the bounded epoch schedule",
                event.id
            ));
        }
        if usize::from(event.fragment_a) >= history.fragments.len()
            || usize::from(event.fragment_b) >= history.fragments.len()
        {
            return Err(format!(
                "{seed}: event {} references an invalid fragment",
                event.id
            ));
        }
        if event.kind == HistoricalEventKind::Rift && event.fragment_a != event.fragment_b {
            split_events += 1;
            split_event_children.insert(event.fragment_a);
            split_event_children.insert(event.fragment_b);
        }
    }
    if split_events == 0 {
        return Err(format!(
            "{seed}: forward tectonic history produced no explicit split/rift event"
        ));
    }
    if split_event_children.is_empty() {
        return Err(format!(
            "{seed}: forward history produced no fragment-resolving rift geometry"
        ));
    }

    // Ocean chronology is now generated by forward opening rather than distance to a frozen
    // ancestral divergent graph. Seed distance from actually young present oceanic material and
    // require older oceanic crust away from those creation corridors on the same moving plate.
    let mut spreading_distance_km = vec![f64::INFINITY; count];
    let mut spreading_queue = VecDeque::<u32>::new();
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        if history.crust_kind[index] == CrustKind::Oceanic as u8
            && history.crust_birth_age_myr[index] <= 10.0
        {
            spreading_distance_km[index] = 0.0;
            spreading_queue.push_back(sample);
        }
    }
    if spreading_queue.is_empty() {
        return Err(format!(
            "{seed}: forward evolution produced no young oceanic spreading material"
        ));
    }
    while let Some(sample) = spreading_queue.pop_front() {
        let index = sample as usize;
        let owner = history.current_plate_ids[index];
        let neighbors = topology.neighbors(sample);
        let lengths = topology.neighbor_arc_lengths_rad(sample);
        for neighbor_index in 0..neighbors.len() {
            let neighbor = neighbors[neighbor_index];
            let ni = neighbor as usize;
            if history.crust_kind[ni] != CrustKind::Oceanic as u8
                || history.current_plate_ids[ni] != owner
            {
                continue;
            }
            let candidate = spreading_distance_km[index]
                + lengths[neighbor_index] * planet.radius_m / 1000.0;
            if candidate + 1.0e-9 < spreading_distance_km[ni] {
                spreading_distance_km[ni] = candidate;
                spreading_queue.push_back(neighbor);
            }
        }
    }

    let mut modern_origins = BTreeMap::<u16, BTreeSet<u16>>::new();
    let mut modern_fragments = BTreeMap::<u16, BTreeSet<u16>>::new();
    let mut origin_to_current =
        vec![BTreeSet::<u16>::new(); history.ancestral_tectonics.plates.len()];
    let mut current_area = vec![0.0_f64; history.metrics.modern_plate_count as usize];
    let mut active_fragment_area = vec![0.0_f64; history.fragments.len()];
    let mut internal_fragment_edges = 0_u32;
    let mut origin_discontinuity_edges = 0_u32;
    let mut oceanic_samples = 0_u32;
    let mut oceanic_age_min = f32::INFINITY;
    let mut oceanic_age_max = f32::NEG_INFINITY;
    let mut oceanic_distance_age = Vec::<(f64, f64)>::new();
    let mut total_area = 0.0_f64;

    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let origin = history.origin_plate_ids[index];
        let fragment = history.fragment_ids[index];
        let current = history.current_plate_ids[index];
        if usize::from(origin) >= history.ancestral_tectonics.plates.len()
            || usize::from(fragment) >= history.fragments.len()
            || current >= history.metrics.modern_plate_count
        {
            return Err(format!("{seed}: invalid ancestry id at sample {sample}"));
        }
        let fragment_record = &history.fragments[fragment as usize];
        if fragment_record.id != fragment
            || fragment_record.origin_plate_id != origin
            || fragment_record.current_plate_id != current
        {
            return Err(format!(
                "{seed}: fragment metadata disagrees with material identity at sample {sample}"
            ));
        }
        origin_to_current[origin as usize].insert(current);
        modern_origins.entry(current).or_default().insert(origin);
        modern_fragments
            .entry(current)
            .or_default()
            .insert(fragment);

        let area = topology.area_steradians(sample);
        total_area += area;
        current_area[current as usize] += area;
        active_fragment_area[fragment as usize] += area;

        let province = geology.crust_province_id[index] & 0x7fff;
        if province != (fragment & 0x7fff) {
            return Err(format!(
                "{seed}: crust provenance is not fragment-owned at sample {sample}"
            ));
        }

        if history.crust_kind[index] == CrustKind::Oceanic as u8 {
            let age = history.crust_birth_age_myr[index];
            if !age.is_finite() || !(0.0..=220.0).contains(&age) {
                return Err(format!(
                    "{seed}: invalid oceanic birth age at sample {sample}: {age}"
                ));
            }
            oceanic_samples += 1;
            oceanic_age_min = oceanic_age_min.min(age);
            oceanic_age_max = oceanic_age_max.max(age);
            if spreading_distance_km[index].is_finite() {
                oceanic_distance_age.push((spreading_distance_km[index], f64::from(age)));
            }
        }

        for neighbor in topology.neighbors(sample) {
            if *neighbor <= sample {
                continue;
            }
            let ni = *neighbor as usize;
            if history.current_plate_ids[ni] == current && history.fragment_ids[ni] != fragment {
                internal_fragment_edges += 1;
            }
            if history.current_plate_ids[ni] == current && history.origin_plate_ids[ni] != origin {
                origin_discontinuity_edges += 1;
            }
        }
    }

    let current_area_sum = current_area.iter().sum::<f64>();
    let fragment_area_sum = active_fragment_area.iter().sum::<f64>();
    let projected_plate_area_sum = tectonics
        .plates
        .iter()
        .map(|plate| plate.area_steradians)
        .sum::<f64>();
    for (label, area_sum) in [
        ("current ownership", current_area_sum),
        ("active fragment ownership", fragment_area_sum),
        ("projected tectonic plates", projected_plate_area_sum),
    ] {
        let relative_error = (area_sum - total_area).abs() / total_area.max(1.0e-12);
        if relative_error > 1.0e-12 {
            return Err(format!(
                "{seed}: {label} area does not close on the sphere: relative error {relative_error:.3e}"
            ));
        }
    }

    let multi_origin_modern_plates = modern_origins
        .values()
        .filter(|origins| origins.len() > 1)
        .count();
    let multi_fragment_modern_plates = modern_fragments
        .values()
        .filter(|fragments| fragments.len() > 1)
        .count();
    if multi_origin_modern_plates == 0 || multi_fragment_modern_plates == 0 {
        return Err(format!(
            "{seed}: modern plates did not consolidate persistent ancestry"
        ));
    }
    if internal_fragment_edges == 0 || origin_discontinuity_edges == 0 {
        return Err(format!(
            "{seed}: no fossil material discontinuities survived inside modern plates"
        ));
    }
    if oceanic_samples == 0 || oceanic_age_max - oceanic_age_min < 20.0 {
        return Err(format!(
            "{seed}: oceanic chronology lacks a meaningful birth-age gradient"
        ));
    }

    if oceanic_distance_age.len() < 8 {
        return Err(format!(
            "{seed}: too little oceanic crust is connected to forward-created spreading material"
        ));
    }
    oceanic_distance_age.sort_by(|left, right| left.0.total_cmp(&right.0));
    let quartile = (oceanic_distance_age.len() / 4).max(1);
    let near_mean_age = oceanic_distance_age[..quartile]
        .iter()
        .map(|(_, age)| *age)
        .sum::<f64>()
        / quartile as f64;
    let far_mean_age = oceanic_distance_age[oceanic_distance_age.len() - quartile..]
        .iter()
        .map(|(_, age)| *age)
        .sum::<f64>()
        / quartile as f64;
    if far_mean_age <= near_mean_age + 2.0 {
        return Err(format!(
            "{seed}: oceanic age does not increase away from forward-created spreading material: near={near_mean_age:.1}Myr far={far_mean_age:.1}Myr"
        ));
    }

    for boundary in &tectonics.boundaries {
        if history.current_plate_ids[boundary.sample_a as usize]
            == history.current_plate_ids[boundary.sample_b as usize]
            || history.current_plate_ids[boundary.sample_a as usize] != boundary.plate_a
            || history.current_plate_ids[boundary.sample_b as usize] != boundary.plate_b
            || !boundary.normal_rate_m_per_year.is_finite()
            || !boundary.shear_rate_m_per_year.is_finite()
        {
            return Err(format!(
                "{seed}: modern boundary does not match finite ownership/kinematic state"
            ));
        }
    }

    let inherited = inherit_historical_identity(&fine, coarse_level, history)
        .map_err(|error| error.to_string())?;
    if inherited.origin_plate_ids.len() != fine.sample_count() as usize
        || inherited.fragment_ids.len() != fine.sample_count() as usize
        || inherited.current_plate_ids.len() != fine.sample_count() as usize
        || inherited.crust_birth_age_myr.len() != fine.sample_count() as usize
    {
        return Err(format!("{seed}: fine historical inheritance is incomplete"));
    }
    for sample in 0..topology.sample_count() as usize {
        if inherited.origin_plate_ids[sample] != history.origin_plate_ids[sample]
            || inherited.fragment_ids[sample] != history.fragment_ids[sample]
            || inherited.current_plate_ids[sample] != history.current_plate_ids[sample]
        {
            return Err(format!(
                "{seed}: fine inheritance rewrote coarse material identity"
            ));
        }
    }

    println!(
        "historical-lithosphere seed={seed} old={} fragments={} parented={} split-events={} modern={} events={} continent={:.1}% transitional={:.1}% ocean={:.1}% ocean-age={:.1}..{:.1}Myr spreading-path-near={:.1} spreading-path-far={:.1} multi-origin-modern={} internal-fragment-edges={} internal-origin-edges={} area-closure=ok history={} tectonics={} geology={} inheritance={}",
        history.metrics.ancestral_plate_count,
        history.metrics.fragment_count,
        parented_fragment_ids.len(),
        split_events,
        history.metrics.modern_plate_count,
        history.metrics.event_count,
        history.metrics.continental_area_fraction * 100.0,
        history.metrics.transitional_area_fraction * 100.0,
        history.metrics.oceanic_area_fraction * 100.0,
        oceanic_age_min,
        oceanic_age_max,
        near_mean_age,
        far_mean_age,
        multi_origin_modern_plates,
        internal_fragment_edges,
        origin_discontinuity_edges,
        history.metrics.history_hash_hex(),
        tectonics.metrics.tectonic_hash_hex(),
        geology.metrics.geology_hash_hex(),
        inherited.identity_hash_hex(),
    );
    Ok(())
}

fn main() -> Result<(), String> {
    for seed in ["interlink-wg7c", "1", "2", "historical-holdout-a"] {
        verify_seed(seed)?;
    }
    Ok(())
}
