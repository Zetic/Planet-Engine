use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_tectonics, GeologyRequest,
    PlanetPhysicalParameters, TectonicsRequest,
};

#[test]
fn continental_crust_partition_is_inherited_truth_not_a_present_day_plate_count_artifact() {
    let topology = build_icosphere(4).expect("WG-3 crust-independence topology");
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

    assert_eq!(
        geology_10.crust_kind,
        geology_22.crust_kind,
        "continental/transitional/oceanic inherited crust mask must not be selected from current plate count",
    );

    for sample in 0..geology_10.crust_kind.len() {
        if geology_10.crust_kind[sample] != interlink_worldgen::CrustKind::Oceanic as u8 {
            assert_eq!(
                geology_10.crust_province_id[sample],
                geology_22.crust_province_id[sample],
                "inherited continental/transitional province identity must remain stable across present-day plate counts",
            );
        }
    }
}
