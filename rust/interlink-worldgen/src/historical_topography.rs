use crate::{
    causal_pipeline, CrustKind, GeodesicTopology, InheritedBoundarySet, InheritedPhysicalState,
    InheritedStructureKind, PlanetPhysicalParameters, TopographyRequest, TopographyState,
    WorldgenError,
};

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

/// WG-4 material-history adapter.
///
/// PR-B establishes passive margins from persistent rift/spreading history in WG-3.5. The
/// accepted WG-4 solver already knows how to turn inherited basin/subsidence/rift memory into
/// shelf/basin relief, so translate the historical `ContinentalMargin` fabric into those bounded
/// physical channels immediately before topography. This keeps passive shelves causally tied to
/// rifted material ancestry without adding another dense long-lived raster to WG-3.75.
pub fn generate_initial_topography(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    request: &TopographyRequest,
) -> Result<TopographyState, WorldgenError> {
    let count = topology.metrics().sample_count as usize;
    if inherited.structural_zone_kind.len() != count
        || inherited.structural_fabric_strength.len() != count
        || inherited.weakness_index.len() != count
        || inherited.crust_kind.len() != count
        || inherited.rift_history.len() != count
        || inherited.subsidence_history.len() != count
        || inherited.basin_potential.len() != count
    {
        return Err(WorldgenError::InvalidTopography(
            "historical passive-margin inputs are not aligned to WG-4 topology",
        ));
    }

    let margin_kind = InheritedStructureKind::ContinentalMargin as u8;
    if !inherited
        .structural_zone_kind
        .iter()
        .any(|kind| *kind == margin_kind)
    {
        return causal_pipeline::generate_initial_topography(
            topology, inherited, boundaries, planet, request,
        );
    }

    let mut adjusted = inherited.clone();
    for sample in 0..count {
        if adjusted.structural_zone_kind[sample] != margin_kind
            || adjusted.crust_kind[sample] == CrustKind::Oceanic as u8
        {
            continue;
        }

        let fabric = f64::from(adjusted.structural_fabric_strength[sample]).clamp(0.0, 1.0);
        let weakness = f64::from(adjusted.weakness_index[sample]).clamp(0.0, 1.0);
        // A weak, inherited rift fabric is the physical memory of stretched continental crust.
        // Transitional crust receives the stronger shelf/basin expression; intact continental
        // material gets a shallower shoulder so passive margins do not become active-rift troughs.
        let margin_memory = clamp01(fabric * (0.68 + 0.32 * weakness));
        let transitional = adjusted.crust_kind[sample] == CrustKind::Transitional as u8;
        let rift_gain = if transitional { 0.72 } else { 0.32 };
        let subsidence_gain = if transitional { 0.82 } else { 0.50 };
        let basin_gain = if transitional { 0.80 } else { 0.46 };

        adjusted.rift_history[sample] = f64::from(adjusted.rift_history[sample])
            .max(margin_memory * rift_gain)
            .clamp(0.0, 1.0) as f32;
        adjusted.subsidence_history[sample] = f64::from(adjusted.subsidence_history[sample])
            .max(margin_memory * subsidence_gain)
            .clamp(0.0, 1.0) as f32;
        adjusted.basin_potential[sample] = f64::from(adjusted.basin_potential[sample])
            .max(margin_memory * basin_gain)
            .clamp(0.0, 1.0) as f32;
    }

    causal_pipeline::generate_initial_topography(
        topology, &adjusted, boundaries, planet, request,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_historical_tectonic_morphology, build_icosphere, generate_historical_frontend,
        generate_lithosphere_from_history, inherit_boundary_interfaces, inherit_physical_state,
        HistoricalLithosphereRequest, LithosphereRequest,
    };

    #[test]
    fn historical_passive_margins_create_bounded_shelf_subsidence() {
        let seed = "historical-passive-margin-topography";
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let coarse_level = 4;
        let coarse = build_icosphere(coarse_level).unwrap();
        let fine = build_icosphere(6).unwrap();
        let frontend = generate_historical_frontend(
            &coarse,
            &HistoricalLithosphereRequest::new(seed, 16),
            planet,
        )
        .unwrap();
        let morphology = build_historical_tectonic_morphology(
            &coarse,
            &frontend.historical,
            &frontend.tectonics,
            seed,
        )
        .unwrap();
        assert!(morphology.metrics.passive_margin_sample_count > 0);

        let lithosphere = generate_lithosphere_from_history(
            &coarse,
            &frontend.historical,
            &frontend.tectonics,
            &frontend.geology,
            &LithosphereRequest::new(seed),
        )
        .unwrap();
        let inherited = inherit_physical_state(
            &fine,
            coarse_level,
            &frontend.tectonics,
            &frontend.geology,
            &lithosphere,
            planet,
        )
        .unwrap();
        let boundaries = inherit_boundary_interfaces(
            &coarse,
            &fine,
            &frontend.tectonics,
            &frontend.geology,
            &inherited.plate_ids,
        )
        .unwrap();

        let legacy = causal_pipeline::generate_initial_topography(
            &fine,
            &inherited,
            &boundaries,
            planet,
            &TopographyRequest::new(seed),
        )
        .unwrap();
        let historical = generate_initial_topography(
            &fine,
            &inherited,
            &boundaries,
            planet,
            &TopographyRequest::new(seed),
        )
        .unwrap();

        let mut margin_samples = 0usize;
        let mut legacy_margin_rift = 0.0_f64;
        let mut historical_margin_rift = 0.0_f64;
        for sample in 0..fine.metrics().sample_count as usize {
            if inherited.structural_zone_kind[sample] == InheritedStructureKind::ContinentalMargin as u8
                && inherited.crust_kind[sample] != CrustKind::Oceanic as u8
            {
                margin_samples += 1;
                legacy_margin_rift += f64::from(legacy.rift_basin_elevation_m[sample]);
                historical_margin_rift += f64::from(historical.rift_basin_elevation_m[sample]);
            }
        }
        assert!(margin_samples > 0);
        let legacy_mean = legacy_margin_rift / margin_samples as f64;
        let historical_mean = historical_margin_rift / margin_samples as f64;
        assert!(historical_mean < legacy_mean - 25.0);
        assert!(historical_mean > -2_500.0);
        assert!(historical.metrics.clamped_sample_count == 0);
    }
}
