use interlink_worldgen::{
    analyze_topography_morphology, build_icosphere, generate_crust_and_history,
    generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, CrustKind,
    GeologyRequest, LithosphereRequest, OrogenProvinceKind, PlanetPhysicalParameters,
    TectonicsRequest, TopographyRequest, TOPOGRAPHY_STAGE_VERSION,
};

fn main() -> Result<(), String> {
    let seed = "interlink-wg7c";
    let coarse_level = 5;
    let fine_level = 7;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;
    let tectonics = generate_tectonics(&coarse, &TectonicsRequest::new(seed, 16), planet)
        .map_err(|error| error.to_string())?;
    let geology =
        generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet)
            .map_err(|error| error.to_string())?;
    let lithosphere = generate_lithosphere(
        &coarse,
        &tectonics,
        &geology,
        &LithosphereRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;
    let inherited = inherit_physical_state(
        &fine,
        coarse_level,
        &tectonics,
        &geology,
        &lithosphere,
        planet,
    )
    .map_err(|error| error.to_string())?;
    let boundaries =
        inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids)
            .map_err(|error| error.to_string())?;
    let terrain = generate_initial_topography(
        &fine,
        &inherited,
        &boundaries,
        planet,
        &TopographyRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;
    let morphology =
        analyze_topography_morphology(&fine, &inherited, &boundaries, planet, &terrain)
            .map_err(|error| error.to_string())?;

    let active_samples = inherited
        .orogenic_history
        .iter()
        .filter(|value| **value > 0.05)
        .count();
    let mountain_core_samples = inherited
        .mountain_core_index
        .iter()
        .filter(|value| **value > 0.20)
        .count();
    let displaced_mountain_core_samples = inherited
        .mountain_core_index
        .iter()
        .zip(inherited.boundary_distance_km.iter())
        .filter(|(core, distance)| **core > 0.20 && **distance >= 180.0)
        .count();
    let runaway_mountain_core_samples = inherited
        .mountain_core_index
        .iter()
        .zip(inherited.boundary_distance_km.iter())
        .filter(|(core, distance)| **core > 0.20 && **distance >= 2200.0)
        .count();
    let positive_orogen = terrain
        .orogenic_elevation_m
        .iter()
        .filter(|value| **value > 250.0)
        .count();
    let negative_orogen = terrain
        .orogenic_elevation_m
        .iter()
        .filter(|value| **value < -100.0)
        .count();
    let mut continental_foreland = 0usize;
    let mut flooded_continental_foreland = 0usize;
    let mut continental_orogen = 0usize;
    let mut flooded_continental_orogen = 0usize;
    for sample in 0..terrain.solid_elevation_m.len() {
        let continental = inherited.crust_kind[sample] == CrustKind::Continental as u8;
        if continental && inherited.foreland_basin_index[sample] > 0.20 {
            continental_foreland += 1;
            if terrain.submerged_mask[sample] != 0 {
                flooded_continental_foreland += 1;
            }
        }
        let kind = inherited.province_kind[sample];
        let collision_orogen = kind == OrogenProvinceKind::ContinentalCollision as u8
            || kind == OrogenProvinceKind::CollisionalPlateau as u8
            || kind == OrogenProvinceKind::TerraneAccretion as u8
            || kind == OrogenProvinceKind::TranspressionalOrogen as u8;
        if continental && collision_orogen {
            continental_orogen += 1;
            if terrain.submerged_mask[sample] != 0 {
                flooded_continental_orogen += 1;
            }
        }
    }

    println!(
        "WG-4 orogen topology: stage=v{} provinces={} active_samples={} core={} displaced_core={} runaway_core={} relief(+/-)={}/{} foreland_flood={}/{} orogen_flood={}/{} solid={:.0}..{:.0}m clamped={} land={:.1}% province_hash={} topo_hash={}",
        terrain.stage.version,
        lithosphere.orogen_provinces.metrics.province_count,
        active_samples,
        mountain_core_samples,
        displaced_mountain_core_samples,
        runaway_mountain_core_samples,
        positive_orogen,
        negative_orogen,
        flooded_continental_foreland,
        continental_foreland,
        flooded_continental_orogen,
        continental_orogen,
        terrain.metrics.minimum_solid_elevation_m,
        terrain.metrics.maximum_solid_elevation_m,
        terrain.metrics.clamped_sample_count,
        terrain.metrics.land_area_fraction * 100.0,
        inherited.orogen_province_hash_hex(),
        terrain.metrics.topography_hash_hex(),
    );

    println!(
        "PR81 morphology: provenance_edges={} provenance_ratio={:.4} boundary_isolated={}/{} ({:.4}) boundary_agreement={:.4} ocean_age_monotonic={:.4} inversions={} inland_non_oceanic_km2={:.0} unsupported_km2={:.0} unsupported_fraction={:.4} max_unsupported_distance_km={:.0} quiet_ocean_gradient={:.4}",
        morphology.quiet_provenance_contacts.contact_edge_count,
        morphology.quiet_provenance_contacts.contact_to_interior_gradient_ratio,
        morphology.boundary_regime_coherence.isolated_regime_edge_count,
        morphology.boundary_regime_coherence.edge_with_same_pair_neighbors_count,
        morphology.boundary_regime_coherence.isolated_regime_edge_fraction,
        morphology.boundary_regime_coherence.mean_same_pair_neighbor_agreement,
        morphology.ocean_age_depth_monotonic_pair_fraction,
        morphology.ocean_age_depth_inversion_count,
        morphology.inland_marine.non_oceanic_marine_area_m2 / 1.0e6,
        morphology.inland_marine.unsupported_inland_area_m2 / 1.0e6,
        morphology.inland_marine.unsupported_inland_area_fraction,
        morphology.inland_marine.maximum_unsupported_distance_from_oceanic_crust_m / 1_000.0,
        morphology.quiet_ocean.mean_gradient_m_per_km,
    );

    if terrain.stage.version != TOPOGRAPHY_STAGE_VERSION || terrain.stage.version != 17 {
        return Err("WG-4 did not route through historical-material topography".to_string());
    }
    if active_samples == 0
        || mountain_core_samples == 0
        || displaced_mountain_core_samples == 0
        || runaway_mountain_core_samples != 0
        || positive_orogen == 0
    {
        return Err(
            "tectonic province topography did not exercise structural-strand relief".to_string(),
        );
    }
    if continental_foreland > 0 && flooded_continental_foreland * 20 > continental_foreland {
        return Err(format!(
            "continental foreland flooding is still systematic: {}/{}",
            flooded_continental_foreland, continental_foreland
        ));
    }
    if continental_orogen > 0 && flooded_continental_orogen * 10 > continental_orogen {
        return Err(format!(
            "continental orogen flooding is still excessive: {}/{}",
            flooded_continental_orogen, continental_orogen
        ));
    }
    let provenance = &morphology.quiet_provenance_contacts;
    if provenance.contact_edge_count == 0 || provenance.interior_edge_count == 0 {
        return Err("quiet provenance morphology was not exercised".to_string());
    }
    if provenance.contact_to_interior_gradient_ratio > 2.0 {
        return Err(format!(
            "quiet provenance contacts still over-express as terrain seams: ratio={:.3}",
            provenance.contact_to_interior_gradient_ratio
        ));
    }

    let coherence = &morphology.boundary_regime_coherence;
    if coherence.isolated_regime_edge_fraction > 0.01
        || coherence.mean_same_pair_neighbor_agreement < 0.75
    {
        return Err(format!(
            "modern boundary regimes remain locally incoherent: isolated={:.3} agreement={:.3}",
            coherence.isolated_regime_edge_fraction,
            coherence.mean_same_pair_neighbor_agreement
        ));
    }

    if morphology.ocean_age_depth_inversion_count != 0
        || morphology.ocean_age_depth_monotonic_pair_fraction < 0.99
    {
        return Err(format!(
            "oceanic age/depth relation is not monotonic: fraction={:.3} inversions={}",
            morphology.ocean_age_depth_monotonic_pair_fraction,
            morphology.ocean_age_depth_inversion_count
        ));
    }

    let inland = &morphology.inland_marine;
    if inland.unsupported_inland_sample_count != 0
        || inland.unsupported_inland_area_fraction > 1.0e-12
        || inland.maximum_unsupported_distance_from_oceanic_crust_m > 1.0
    {
        return Err(format!(
            "marine mask contains unsupported inland penetration: samples={} fraction={:.6} max_distance_km={:.1}",
            inland.unsupported_inland_sample_count,
            inland.unsupported_inland_area_fraction,
            inland.maximum_unsupported_distance_from_oceanic_crust_m / 1_000.0
        ));
    }

    if morphology.quiet_ocean.mean_gradient_m_per_km > 1.5 {
        return Err(format!(
            "quiet-ocean bathymetry remains too rough: mean_gradient={:.3} m/km",
            morphology.quiet_ocean.mean_gradient_m_per_km
        ));
    }

    if terrain
        .solid_elevation_m
        .iter()
        .any(|value| !value.is_finite())
    {
        return Err("tectonic province topography produced non-finite relief".to_string());
    }
    Ok(())
}
