use interlink_worldgen_wasm::WasmWorldgenDrainage;

#[test]
fn drainage_bridge_exposes_routing_topology_and_conservation() {
    let output = WasmWorldgenDrainage::new("wg6a-wasm".to_owned(), 3, 4, 12).unwrap();
    assert_eq!(output.generator_version(), 10);
    assert_eq!(output.stage_id(), "hydrology:drainage-topology");
    assert_eq!(output.stage_version(), 1);
    assert_eq!(output.coarse_level(), 3);
    assert_eq!(output.fine_level(), 4);
    assert_eq!(output.sample_count() as usize, output.receiver().len());
    assert_eq!(output.sample_count() as usize, output.outlet_sample().len());
    assert_eq!(output.sample_count() as usize, output.basin_id().len());
    assert_eq!(output.sample_count() as usize, output.depression_id().len());
    assert_eq!(
        output.sample_count() as usize,
        output.hydrologic_escape_elevation_m().len()
    );
    assert_eq!(
        output.sample_count() as usize,
        output.contributing_area_m2().len()
    );
    assert_eq!(
        output.land_sample_count() + output.ocean_sample_count(),
        output.sample_count()
    );
    assert!(output.basin_count() > 0);
    assert!(output.area_conservation_relative_error() < 1.0e-12);
    assert_eq!(output.drainage_hash_hex().len(), 16);
    assert_eq!(output.topography_hash_hex().len(), 16);
}
