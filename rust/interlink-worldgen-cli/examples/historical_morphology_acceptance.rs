use interlink_worldgen::{
    build_historical_tectonic_morphology, build_icosphere, generate_historical_frontend,
    generate_lithosphere_from_history, CrustKind, HistoricalEventKind, HistoricalLithosphereRequest,
    LithosphereRequest, PlanetPhysicalParameters, PlanetTopology, PlateBoundaryKind,
};

fn verify_seed(seed: &str) -> Result<(), String> {
    let topology = build_icosphere(4).map_err(|error| error.to_string())?;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let frontend = generate_historical_frontend(
        &topology,
        &HistoricalLithosphereRequest::new(seed, 16),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let morphology = build_historical_tectonic_morphology(
        &topology,
        &frontend.historical,
        &frontend.tectonics,
        seed,
    )
    .map_err(|error| error.to_string())?;
    let repeat = build_historical_tectonic_morphology(
        &topology,
        &frontend.historical,
        &frontend.tectonics,
        seed,
    )
    .map_err(|error| error.to_string())?;
    if morphology.metrics.morphology_hash != repeat.metrics.morphology_hash {
        return Err(format!("{seed}: historical morphology is not deterministic"));
    }

    let count = topology.sample_count() as usize;
    for (name, length) in [
        ("latest-event", morphology.latest_event_kind.len()),
        ("rift", morphology.rift_intensity.len()),
        ("shear", morphology.shear_intensity.len()),
        ("suture", morphology.suture_intensity.len()),
        ("passive-margin", morphology.passive_margin_index.len()),
        ("active-orogen", morphology.active_orogen_intensity.len()),
        ("fossil-orogen", morphology.fossil_orogen_intensity.len()),
    ] {
        if length != count {
            return Err(format!("{seed}: {name} morphology does not cover the coarse sphere"));
        }
    }

    let mut current_boundary_mask = vec![false; count];
    let mut active_convergent_samples = 0_usize;
    let mut active_convergent_supported = 0_usize;
    for boundary in &frontend.tectonics.boundaries {
        current_boundary_mask[boundary.sample_a as usize] = true;
        current_boundary_mask[boundary.sample_b as usize] = true;
        if boundary.kind == PlateBoundaryKind::Convergent {
            for sample in [boundary.sample_a, boundary.sample_b] {
                active_convergent_samples += 1;
                if morphology.active_orogen_intensity[sample as usize] >= 0.20 {
                    active_convergent_supported += 1;
                }
            }
        }
    }
    if active_convergent_samples == 0 || active_convergent_supported == 0 {
        return Err(format!("{seed}: active convergence is absent from active-orogen morphology"));
    }

    let internal_fossil_samples = morphology
        .fossil_orogen_intensity
        .iter()
        .enumerate()
        .filter(|(sample, value)| **value >= 0.20 && !current_boundary_mask[*sample])
        .count();
    if internal_fossil_samples == 0 {
        return Err(format!("{seed}: fossil orogens did not survive inside present plates"));
    }

    let collision_events = frontend
        .historical
        .events
        .iter()
        .filter(|event| event.kind == HistoricalEventKind::Collision)
        .collect::<Vec<_>>();
    if collision_events.is_empty() {
        return Err(format!("{seed}: historical ledger contains no collision event"));
    }
    let collision_suture_support = collision_events
        .iter()
        .filter(|event| {
            morphology.suture_intensity[event.geometry_sample_a as usize] >= 0.20
                || morphology.suture_intensity[event.geometry_sample_b as usize] >= 0.20
        })
        .count();
    if collision_suture_support == 0 {
        return Err(format!("{seed}: collision events did not seed persistent sutures"));
    }

    let mut rifted_continent_ocean_contacts = 0_usize;
    let mut passive_margin_contacts = 0_usize;
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        if frontend.historical.crust_kind[index] == CrustKind::Oceanic as u8 {
            continue;
        }
        for neighbor in topology.neighbors(sample) {
            let ni = *neighbor as usize;
            if frontend.historical.crust_kind[ni] == CrustKind::Oceanic as u8
                && morphology.rift_intensity[index].max(morphology.rift_intensity[ni]) >= 0.20
            {
                rifted_continent_ocean_contacts += 1;
                if morphology.passive_margin_index[index] >= 0.20 {
                    passive_margin_contacts += 1;
                }
                break;
            }
        }
    }
    if rifted_continent_ocean_contacts == 0 || passive_margin_contacts == 0 {
        return Err(format!("{seed}: rifted continental edges did not produce passive-margin state"));
    }

    let lithosphere = generate_lithosphere_from_history(
        &topology,
        &frontend.historical,
        &frontend.tectonics,
        &frontend.geology,
        &LithosphereRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;
    if lithosphere.pre_orogenic.metrics.paleo_suture_sample_count == 0
        || lithosphere.pre_orogenic.metrics.inherited_rift_sample_count == 0
    {
        return Err(format!("{seed}: WG-3.5 did not consume historical structures"));
    }

    let mut internal_fossil_relief = 0_usize;
    let mut active_relief = 0_usize;
    for sample in 0..count {
        if morphology.fossil_orogen_intensity[sample] >= 0.20
            && !current_boundary_mask[sample]
            && lithosphere.orogen_provinces.orogenic_intensity[sample] >= 0.12
        {
            internal_fossil_relief += 1;
        }
        if morphology.active_orogen_intensity[sample] >= 0.20
            && lithosphere.orogen_provinces.orogenic_intensity[sample] >= 0.12
        {
            active_relief += 1;
        }
    }
    if internal_fossil_relief == 0 {
        return Err(format!("{seed}: WG-3.6 produced no event-driven fossil relief inside modern plates"));
    }
    if active_relief == 0 {
        return Err(format!("{seed}: WG-3.6 lost active convergent relief"));
    }

    println!(
        "historical-morphology seed={seed} events={} rift={} shear={} sutures={} passive={} active={} fossil={} internal-fossil={} collision-suture={}/{} passive-contact={}/{} fossil-relief={} active-relief={} morphology={} pre={} orogen={}",
        morphology.metrics.event_count,
        morphology.metrics.rift_sample_count,
        morphology.metrics.shear_sample_count,
        morphology.metrics.suture_sample_count,
        morphology.metrics.passive_margin_sample_count,
        morphology.metrics.active_orogen_sample_count,
        morphology.metrics.fossil_orogen_sample_count,
        internal_fossil_samples,
        collision_suture_support,
        collision_events.len(),
        passive_margin_contacts,
        rifted_continent_ocean_contacts,
        internal_fossil_relief,
        active_relief,
        morphology.metrics.morphology_hash_hex(),
        lithosphere.pre_orogenic.metrics.pre_orogenic_hash_hex(),
        lithosphere.orogen_provinces.metrics.province_hash_hex(),
    );
    Ok(())
}

fn main() -> Result<(), String> {
    for seed in ["interlink-wg7c", "1", "2", "historical-morphology-holdout"] {
        verify_seed(seed)?;
    }
    Ok(())
}
