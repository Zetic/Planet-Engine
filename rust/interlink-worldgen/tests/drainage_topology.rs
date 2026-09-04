use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_drainage_topology,
    generate_initial_topography, generate_lithosphere, generate_tectonics,
    inherit_boundary_interfaces, inherit_physical_state, DrainageRequest, GeologyRequest,
    LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,
    DRAINAGE_OUTLET_INTERNAL, INVALID_SAMPLE_ID,
};

fn generated(
    seed: &str,
    water_mass_kg: Option<f64>,
) -> (
    interlink_worldgen::GeodesicTopology,
    interlink_worldgen::TopographyState,
    interlink_worldgen::DrainageState,
) {
    let mut planet = PlanetPhysicalParameters::earthlike_reference();
    if let Some(value) = water_mass_kg {
        planet.surface_water_mass_kg = value;
    }
    let coarse = build_icosphere(3).unwrap();
    let fine = build_icosphere(4).unwrap();
    let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, 12), planet).unwrap();
    let geology =
        generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet)
            .unwrap();
    let lithosphere = generate_lithosphere(
        &coarse,
        &tectonics,
        &geology,
        &LithosphereRequest::new(seed),
    )
    .unwrap();
    let inherited =
        inherit_physical_state(&fine, 3, &tectonics, &geology, &lithosphere, planet).unwrap();
    let boundaries =
        inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids)
            .unwrap();
    let topography = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(seed),
    )
    .unwrap();
    let drainage =
        generate_drainage_topology(&fine, &topography, planet, &DrainageRequest::new(seed))
            .unwrap();
    (fine, topography, drainage)
}

#[test]
fn generated_planet_drainage_is_deterministic_aligned_and_conservative() {
    let (topology_a, topography_a, drainage_a) = generated("wg6a-planet", None);
    let (_, _, drainage_b) = generated("wg6a-planet", None);

    assert_eq!(
        drainage_a.metrics.drainage_hash,
        drainage_b.metrics.drainage_hash
    );
    assert_eq!(drainage_a.receiver, drainage_b.receiver);
    assert_eq!(drainage_a.basin_id, drainage_b.basin_id);
    assert_eq!(
        drainage_a.metrics.sample_count,
        topography_a.metrics.sample_count
    );
    assert_eq!(
        drainage_a.receiver.len(),
        topology_a.metrics().sample_count as usize
    );
    assert!(drainage_a.metrics.land_sample_count > 0);
    assert!(drainage_a.metrics.ocean_sample_count > 0);
    assert!(drainage_a.metrics.basin_count > 0);
    assert!(drainage_a.metrics.area_conservation_relative_error < 1.0e-12);
    assert!(drainage_a
        .hydrologic_escape_elevation_m
        .iter()
        .all(|value| value.is_finite()));
    assert!(drainage_a
        .depression_depth_m
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0));

    for sample in 0..drainage_a.receiver.len() {
        if topography_a.submerged_mask[sample] != 0 {
            continue;
        }
        assert_ne!(drainage_a.outlet_sample[sample], INVALID_SAMPLE_ID);
        let outlet = drainage_a.outlet_sample[sample] as usize;
        assert_eq!(
            drainage_a.outlet_kind[sample],
            drainage_a.outlet_kind[outlet]
        );
        let receiver = drainage_a.receiver[sample];
        if receiver != INVALID_SAMPLE_ID {
            assert!(topology_a.neighbors_of(sample as u32).contains(&receiver));
        }
    }
}

#[test]
fn changing_water_inventory_changes_outlets_without_changing_wg4_solid_surface() {
    let (_, wet_topography, wet) = generated("wg6a-water", None);
    let (_, drier_topography, drier) = generated("wg6a-water", Some(7.0e20));

    assert_eq!(
        wet_topography.solid_elevation_m,
        drier_topography.solid_elevation_m
    );
    assert_ne!(
        wet_topography.submerged_mask,
        drier_topography.submerged_mask
    );
    assert_ne!(wet.metrics.drainage_hash, drier.metrics.drainage_hash);
    assert_ne!(wet.outlet_sample, drier.outlet_sample);
}

#[test]
fn dry_planet_resolves_to_an_explicit_internal_terminal() {
    let (_, topography, drainage) = generated("wg6a-dry", Some(0.0));
    assert_eq!(drainage.metrics.ocean_sample_count, 0);
    assert!(topography.submerged_mask.iter().all(|value| *value == 0));

    let internal_terminals = drainage
        .receiver
        .iter()
        .enumerate()
        .filter(|(index, receiver)| {
            **receiver == INVALID_SAMPLE_ID
                && drainage.outlet_kind[*index] == DRAINAGE_OUTLET_INTERNAL
        })
        .map(|(index, _)| index as u32)
        .collect::<Vec<_>>();
    assert_eq!(internal_terminals.len(), 1);
    let internal = internal_terminals[0];
    assert!(drainage
        .outlet_sample
        .iter()
        .all(|outlet| *outlet == internal));
    assert!(drainage
        .outlet_kind
        .iter()
        .all(|kind| *kind == DRAINAGE_OUTLET_INTERNAL));
    assert!(drainage.metrics.area_conservation_relative_error < 1.0e-12);
}
