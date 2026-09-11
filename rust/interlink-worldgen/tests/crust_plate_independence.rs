use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_tectonics, CrustKind, GeologyRequest,
    PlanetPhysicalParameters, TectonicsRequest,
};

fn mixed_plate_count(
    plate_ids: &[u16],
    crust_kind: &[u8],
    plate_count: usize,
) -> usize {
    let mut seen_continental = vec![false; plate_count];
    let mut seen_oceanic = vec![false; plate_count];
    for (plate_id, kind) in plate_ids.iter().zip(crust_kind.iter()) {
        let plate = *plate_id as usize;
        if *kind == CrustKind::Oceanic as u8 {
            seen_oceanic[plate] = true;
        } else {
            seen_continental[plate] = true;
        }
    }
    (0..plate_count)
        .filter(|plate| seen_continental[*plate] && seen_oceanic[*plate])
        .count()
}

#[test]
fn continental_crust_is_not_plate_identity_but_responds_to_accepted_tectonic_layout() {
    let topology = build_icosphere(4).expect("WG-3 crust/plate coupling topology");
    let parameters = PlanetPhysicalParameters::earthlike_reference();
    let seed = "wg3-inherited-crust";

    let tectonics_10 = generate_tectonics(&topology, &TectonicsRequest::new(seed, 10), parameters)
        .expect("10-plate tectonics");
    let tectonics_22 = generate_tectonics(&topology, &TectonicsRequest::new(seed, 22), parameters)
        .expect("22-plate tectonics");
    let geology_10 = generate_crust_and_history(
        &topology,
        &tectonics_10,
        &GeologyRequest::new(seed),
        parameters,
    )
    .expect("10-plate geology");
    let geology_22 = generate_crust_and_history(
        &topology,
        &tectonics_22,
        &GeologyRequest::new(seed),
        parameters,
    )
    .expect("22-plate geology");

    let changed_fraction = geology_10
        .crust_kind
        .iter()
        .zip(geology_22.crust_kind.iter())
        .filter(|(left, right)| left != right)
        .count() as f64
        / geology_10.crust_kind.len() as f64;
    assert!(
        changed_fraction >= 0.03,
        "WG-3 continental assembly must respond materially to a different accepted tectonic layout: changed={changed_fraction:.4}",
    );

    let mixed_10 = mixed_plate_count(
        &tectonics_10.plate_ids,
        &geology_10.crust_kind,
        tectonics_10.plates.len(),
    );
    let mixed_22 = mixed_plate_count(
        &tectonics_22.plate_ids,
        &geology_22.crust_kind,
        tectonics_22.plates.len(),
    );
    assert!(
        mixed_10 > 0 && mixed_22 > 0,
        "plate ownership must not become synonymous with crust type: mixed plates 10={mixed_10}, 22={mixed_22}",
    );

    assert_ne!(
        geology_10.metrics.geology_hash, geology_22.metrics.geology_hash,
        "a different accepted tectonic layout must produce a different WG-3 geological identity",
    );
}
