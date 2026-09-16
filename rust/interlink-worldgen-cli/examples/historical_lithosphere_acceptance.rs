use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, inherit_historical_identity, CrustKind,
    HistoricalEventKind, HistoricalLithosphereRequest, PlanetPhysicalParameters, PlanetTopology,
    HISTORICAL_EPOCH_COUNT,
};
use std::collections::{BTreeMap, BTreeSet};

// Permanent PR-A acceptance: material identity must survive the complete historical front end,
// bounded fragment epochs, modern-plate projection, compatibility geology projection, and
// coarse-to-fine inheritance.
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
        return Err(format!("{seed}: historical material fields do not cover the sphere"));
    }
    if history.metrics.ancestral_plate_count < history.metrics.modern_plate_count
        || history.metrics.fragment_count < history.metrics.ancestral_plate_count
    {
        return Err(format!("{seed}: ancestry hierarchy did not preserve old-plate -> fragment -> modern-plate structure"));
    }
    if tectonics.plate_ids != history.current_plate_ids {
        return Err(format!("{seed}: modern tectonic ownership diverged from historical current ownership"));
    }
    if geology.crust_kind != history.crust_kind {
        return Err(format!("{seed}: geology regenerated crust kind instead of consuming historical material"));
    }

    let mut parented_fragments = 0_usize;
    for (index, fragment) in history.fragments.iter().enumerate() {
        if usize::from(fragment.id) != index {
            return Err(format!("{seed}: fragment ids are not dense lineage indices"));
        }
        if let Some(parent_id) = fragment.parent_fragment_id {
            parented_fragments += 1;
            if parent_id >= fragment.id || usize::from(parent_id) >= history.fragments.len() {
                return Err(format!("{seed}: fragment {} has an invalid/cyclic parent {parent_id}", fragment.id));
            }
            let parent = &history.fragments[parent_id as usize];
            if parent.origin_plate_id != fragment.origin_plate_id
                || parent.current_plate_id != fragment.current_plate_id
            {
                return Err(format!("{seed}: fragment {} changed material ownership across its parent edge", fragment.id));
            }
        }
    }
    if parented_fragments == 0 {
        return Err(format!("{seed}: bounded historical epochs produced no parent/child fragment lineage"));
    }

    let mut split_events = 0_usize;
    for event in &history.events {
        if event.epoch >= HISTORICAL_EPOCH_COUNT {
            return Err(format!("{seed}: event {} lies outside the bounded epoch schedule", event.id));
        }
        if event.kind == HistoricalEventKind::Rift && event.fragment_a != event.fragment_b {
            split_events += 1;
        }
    }
    if split_events == 0 {
        return Err(format!("{seed}: bounded historical epochs produced no explicit split/rift event"));
    }

    let mut modern_origins = BTreeMap::<u16, BTreeSet<u16>>::new();
    let mut modern_fragments = BTreeMap::<u16, BTreeSet<u16>>::new();
    let mut internal_fragment_edges = 0_u32;
    let mut origin_discontinuity_edges = 0_u32;
    let mut oceanic_samples = 0_u32;
    let mut oceanic_age_min = f32::INFINITY;
    let mut oceanic_age_max = f32::NEG_INFINITY;

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
            return Err(format!("{seed}: fragment metadata disagrees with material identity at sample {sample}"));
        }
        modern_origins.entry(current).or_default().insert(origin);
        modern_fragments.entry(current).or_default().insert(fragment);

        let province = geology.crust_province_id[index] & 0x7fff;
        if province != (fragment & 0x7fff) {
            return Err(format!("{seed}: crust provenance is not fragment-owned at sample {sample}"));
        }

        if history.crust_kind[index] == CrustKind::Oceanic as u8 {
            let age = history.crust_birth_age_myr[index];
            if !age.is_finite() || !(0.0..=220.0).contains(&age) {
                return Err(format!("{seed}: invalid oceanic birth age at sample {sample}: {age}"));
            }
            oceanic_samples += 1;
            oceanic_age_min = oceanic_age_min.min(age);
            oceanic_age_max = oceanic_age_max.max(age);
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

    let multi_origin_modern_plates = modern_origins
        .values()
        .filter(|origins| origins.len() > 1)
        .count();
    let multi_fragment_modern_plates = modern_fragments
        .values()
        .filter(|fragments| fragments.len() > 1)
        .count();
    if multi_origin_modern_plates == 0 || multi_fragment_modern_plates == 0 {
        return Err(format!("{seed}: modern plates did not consolidate persistent ancestry"));
    }
    if internal_fragment_edges == 0 || origin_discontinuity_edges == 0 {
        return Err(format!("{seed}: no fossil material discontinuities survived inside modern plates"));
    }
    if oceanic_samples == 0 || oceanic_age_max - oceanic_age_min < 20.0 {
        return Err(format!("{seed}: oceanic chronology lacks a meaningful birth-age gradient"));
    }

    for boundary in &tectonics.boundaries {
        if history.current_plate_ids[boundary.sample_a as usize] == history.current_plate_ids[boundary.sample_b as usize]
            || history.current_plate_ids[boundary.sample_a as usize] != boundary.plate_a
            || history.current_plate_ids[boundary.sample_b as usize] != boundary.plate_b
        {
            return Err(format!("{seed}: modern boundary does not match ownership discontinuity"));
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
            return Err(format!("{seed}: fine inheritance rewrote coarse material identity"));
        }
    }

    println!(
        "historical-lithosphere seed={seed} old={} fragments={} parented={} split-events={} modern={} events={} continent={:.1}% transitional={:.1}% ocean={:.1}% ocean-age={:.1}..{:.1}Myr multi-origin-modern={} internal-fragment-edges={} internal-origin-edges={} history={} tectonics={} geology={} inheritance={}",
        history.metrics.ancestral_plate_count,
        history.metrics.fragment_count,
        parented_fragments,
        split_events,
        history.metrics.modern_plate_count,
        history.metrics.event_count,
        history.metrics.continental_area_fraction * 100.0,
        history.metrics.transitional_area_fraction * 100.0,
        history.metrics.oceanic_area_fraction * 100.0,
        oceanic_age_min,
        oceanic_age_max,
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
