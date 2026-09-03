use interlink_worldgen::{build_icosphere, generate_tectonics, PlanetPhysicalParameters, TectonicsRequest};

#[test]
fn macro_plate_partition_retains_meaningful_area_variation_across_reference_seeds() {
    let topology = build_icosphere(5).expect("L5 topology should build");
    let parameters = PlanetPhysicalParameters::earthlike_reference();
    let seeds = ["ci-wg2", "ci-wg2-b", "ci-wg2-c", "ci-wg2-d", "ci-wg2-e"];
    let mut heterogeneous_worlds = 0usize;

    for seed in seeds {
        let model = generate_tectonics(&topology, &TectonicsRequest::new(seed, 18), parameters)
            .expect("reference tectonic model should generate");
        let mean = model.metrics.mean_plate_area_fraction;
        let has_major_plate = model.metrics.maximum_plate_area_fraction > mean * 1.5;
        let has_minor_plate = model.metrics.minimum_plate_area_fraction < mean * 0.65;
        if has_major_plate && has_minor_plate {
            heterogeneous_worlds += 1;
        }
    }

    assert!(
        heterogeneous_worlds >= 4,
        "macro-plate seeding regressed toward a near-equal tessellation: only {heterogeneous_worlds}/5 reference worlds contain both major and minor plates"
    );
}
