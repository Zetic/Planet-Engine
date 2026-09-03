use interlink_worldgen_wasm::WasmWorldgenTopography;

#[test]
fn topography_bridge_exposes_physical_surface_and_water_solution() {
    let output = WasmWorldgenTopography::new("wg4-wasm".to_owned(), 3, 4, 12).unwrap();
    assert_eq!(output.generator_version(), 7);
    assert_eq!(output.stage_id(), "terrain:initial-topography");
    assert_eq!(output.stage_version(), 1);
    assert_eq!(output.coarse_level(), 3);
    assert_eq!(output.fine_level(), 4);
    assert_eq!(
        output.fine_sample_count() as usize,
        output.solid_elevation_m().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.water_depth_m().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.submerged_mask().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.nearest_coarse_source().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.inherited_sample_mask().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.crust_age_myr().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.crust_thickness_km().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.strength_index().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.weakness_index().len()
    );
    assert_eq!(
        output.fine_sample_count() as usize,
        output.kinematic_domain_ids().len()
    );
    assert_eq!(
        output.fine_boundary_edge_count() as usize,
        output.boundary_kinds().len()
    );
    assert_eq!(
        output.fine_boundary_edge_count() as usize,
        output.boundary_coarse_source_indices().len()
    );
    assert!(output.minimum_solid_elevation_m() < output.maximum_solid_elevation_m());
    assert!(output.has_sea_level());
    assert!(output.land_area_fraction() > 0.0);
    assert!(output.ocean_area_fraction() > 0.0);
    assert!(output.water_volume_relative_error() < 1.0e-10);
    assert_eq!(output.topography_hash_hex().len(), 16);
    assert_eq!(output.inheritance_hash_hex().len(), 16);
    assert_eq!(output.boundary_hash_hex().len(), 16);
}
