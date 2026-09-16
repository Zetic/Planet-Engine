use crate::{
    causal_pipeline, CrustKind, GeodesicTopology, InheritedBoundarySet, InheritedPhysicalState,
    InheritedStructureKind, PlanetPhysicalParameters, TopographyRequest, TopographyState,
    WorldgenError,
};

pub const HISTORICAL_TOPOGRAPHY_STAGE_ID: &str = "terrain:initial-topography";
pub const HISTORICAL_TOPOGRAPHY_STAGE_VERSION: u32 = 15;
const HISTORICAL_TOPOGRAPHY_NAMESPACE: &str = "terrain:historical-material-morphology:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn area_weighted_mean(values: &[f32], areas: &[f64]) -> f64 {
    let total_area = areas.iter().sum::<f64>().max(1.0e-12);
    values
        .iter()
        .zip(areas.iter())
        .map(|(value, area)| f64::from(*value) * *area)
        .sum::<f64>()
        / total_area
}

fn area_weighted_quantiles(values: &[f32], areas: &[f64]) -> (f64, f64, f64) {
    let mut pairs = values
        .iter()
        .copied()
        .zip(areas.iter().copied())
        .collect::<Vec<_>>();
    pairs.sort_by(|left, right| left.0.total_cmp(&right.0));
    let total = areas.iter().sum::<f64>().max(1.0e-12);
    let targets = [total * 0.05, total * 0.50, total * 0.95];
    let mut result = [0.0_f64; 3];
    let mut reached = 0usize;
    let mut cumulative = 0.0_f64;
    for (value, area) in pairs {
        cumulative += area;
        while reached < targets.len() && cumulative >= targets[reached] {
            result[reached] = f64::from(value);
            reached += 1;
        }
        if reached == targets.len() {
            break;
        }
    }
    if reached < targets.len() {
        let fallback = values.last().copied().map(f64::from).unwrap_or(0.0);
        while reached < targets.len() {
            result[reached] = fallback;
            reached += 1;
        }
    }
    (result[0], result[1], result[2])
}

fn passive_margin_deflection_m(inherited: &InheritedPhysicalState, sample: usize) -> f64 {
    if inherited.structural_zone_kind[sample] != InheritedStructureKind::ContinentalMargin as u8
        || inherited.crust_kind[sample] == CrustKind::Oceanic as u8
        || inherited.province_kind[sample] != 0
    {
        return 0.0;
    }

    let fabric = f64::from(inherited.structural_fabric_strength[sample]).clamp(0.0, 1.0);
    let weakness = f64::from(inherited.weakness_index[sample]).clamp(0.0, 1.0);
    let margin_memory = clamp01(fabric * (0.68 + 0.32 * weakness));
    let scale_m = if inherited.crust_kind[sample] == CrustKind::Transitional as u8 {
        900.0
    } else {
        420.0
    };
    -scale_m * margin_memory
}

fn finalize_historical_stage(state: &mut TopographyState, request: &TopographyRequest) {
    let stage_seed = crate::derive_stage_seed(&request.seed, HISTORICAL_TOPOGRAPHY_NAMESPACE);
    let prior_hash = state.metrics.topography_hash;
    let mut topography_hash = FNV_OFFSET_BASIS;
    topography_hash = fnv_update(topography_hash, HISTORICAL_TOPOGRAPHY_STAGE_ID.as_bytes());
    topography_hash = fnv_update(
        topography_hash,
        &HISTORICAL_TOPOGRAPHY_STAGE_VERSION.to_le_bytes(),
    );
    topography_hash = fnv_update(topography_hash, &stage_seed.to_le_bytes());
    topography_hash = fnv_update(topography_hash, &prior_hash.to_le_bytes());
    state.stage.id = HISTORICAL_TOPOGRAPHY_STAGE_ID;
    state.stage.version = HISTORICAL_TOPOGRAPHY_STAGE_VERSION;
    state.stage.derived_seed = stage_seed;
    state.metrics.topography_hash = topography_hash;
}

fn refresh_water_and_metrics(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    state: &mut TopographyState,
    planet: PlanetPhysicalParameters,
    prior_hash: u64,
    prior_clamped_sample_count: u32,
) -> Result<(), WorldgenError> {
    let count = topology.metrics().sample_count as usize;
    let areas = topology.dual_area_steradians();

    // The causal WG-4 solve already identified the connected global ocean. Reuse only submerged
    // oceanic-crust cells as seeds after the passive-margin deflection, so newly lowered shelves can
    // be flooded through a real marine path without reviving isolated inland/oceanic sliver seeds.
    let ocean_seed_mask = (0..count)
        .map(|sample| {
            u8::from(
                state.submerged_mask[sample] != 0
                    && inherited.crust_kind[sample] == CrustKind::Oceanic as u8,
            )
        })
        .collect::<Vec<_>>();

    // Release the old water rasters before allocating the replacement connected-ocean solve. This
    // keeps the adapter within the PR62 lifetime discipline instead of retaining duplicate L8 water
    // state during WG-4.
    state.elevation_above_sea_level_m = Vec::new();
    state.water_depth_m = Vec::new();
    state.submerged_mask = Vec::new();

    let water = crate::surface_water::solve_hydrostatic_surface_water_connected(
        topology,
        &state.solid_elevation_m,
        planet,
        &ocean_seed_mask,
    )?;

    let minimum_solid_elevation_m = state
        .solid_elevation_m
        .iter()
        .copied()
        .map(f64::from)
        .fold(f64::INFINITY, f64::min);
    let maximum_solid_elevation_m = state
        .solid_elevation_m
        .iter()
        .copied()
        .map(f64::from)
        .fold(f64::NEG_INFINITY, f64::max);
    let (p05, median, p95) = area_weighted_quantiles(&state.solid_elevation_m, areas);

    let mut land_area = 0.0_f64;
    let mut ocean_area = 0.0_f64;
    let mut land_elevation_area_sum = 0.0_f64;
    let mut water_depth_area_sum = 0.0_f64;
    let mut maximum_water_depth_m = 0.0_f64;
    for sample in 0..count {
        let area = areas[sample];
        if water.submerged_mask[sample] != 0 {
            ocean_area += area;
            let depth = f64::from(water.water_depth_m[sample]);
            water_depth_area_sum += depth * area;
            maximum_water_depth_m = maximum_water_depth_m.max(depth);
        } else {
            land_area += area;
            land_elevation_area_sum += f64::from(water.elevation_above_sea_level_m[sample]) * area;
        }
    }
    let total_area = (land_area + ocean_area).max(1.0e-12);

    let mut topography_hash = FNV_OFFSET_BASIS;
    topography_hash = fnv_update(
        topography_hash,
        b"terrain:historical-passive-margin-topography:v1\0",
    );
    topography_hash = fnv_update(topography_hash, &prior_hash.to_le_bytes());
    for value in &state.solid_elevation_m {
        topography_hash = fnv_update(topography_hash, &value.to_bits().to_le_bytes());
    }
    for value in &state.rift_basin_elevation_m {
        topography_hash = fnv_update(topography_hash, &value.to_bits().to_le_bytes());
    }
    for value in &water.water_depth_m {
        topography_hash = fnv_update(topography_hash, &value.to_bits().to_le_bytes());
    }
    topography_hash = fnv_update(topography_hash, &water.submerged_mask);

    state.metrics.minimum_solid_elevation_m = minimum_solid_elevation_m;
    state.metrics.maximum_solid_elevation_m = maximum_solid_elevation_m;
    state.metrics.mean_solid_elevation_m = area_weighted_mean(&state.solid_elevation_m, areas);
    state.metrics.p05_solid_elevation_m = p05;
    state.metrics.median_solid_elevation_m = median;
    state.metrics.p95_solid_elevation_m = p95;
    state.metrics.sea_level_m = water.metrics.sea_level_m;
    state.metrics.land_area_fraction = land_area / total_area;
    state.metrics.ocean_area_fraction = ocean_area / total_area;
    state.metrics.mean_land_elevation_m = if land_area > 0.0 {
        land_elevation_area_sum / land_area
    } else {
        0.0
    };
    state.metrics.mean_water_depth_m = if ocean_area > 0.0 {
        water_depth_area_sum / ocean_area
    } else {
        0.0
    };
    state.metrics.maximum_water_depth_m = maximum_water_depth_m;
    state.metrics.target_water_volume_m3 = water.metrics.target_water_volume_m3;
    state.metrics.solved_water_volume_m3 = water.metrics.solved_water_volume_m3;
    state.metrics.water_volume_relative_error = water.metrics.water_volume_relative_error;
    state.metrics.clamped_sample_count = prior_clamped_sample_count;
    state.metrics.topography_hash = topography_hash;
    state.elevation_above_sea_level_m = water.elevation_above_sea_level_m;
    state.water_depth_m = water.water_depth_m;
    state.submerged_mask = water.submerged_mask;
    Ok(())
}

/// WG-4 material-history adapter.
///
/// Persistent rifting already produces `ContinentalMargin` structure in WG-3.5. Materialize that
/// inherited state as a bounded shelf/basin deflection after the accepted tectonic-province WG-4
/// solve. The operation is in-place: no second `InheritedPhysicalState` is retained at L8.
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
    {
        return Err(WorldgenError::InvalidTopography(
            "historical passive-margin inputs are not aligned to WG-4 topology",
        ));
    }

    let has_margin = inherited
        .structural_zone_kind
        .iter()
        .enumerate()
        .any(|(sample, kind)| {
            *kind == InheritedStructureKind::ContinentalMargin as u8
                && inherited.crust_kind[sample] != CrustKind::Oceanic as u8
        });
    let mut state = causal_pipeline::generate_initial_topography(
        topology, inherited, boundaries, planet, request,
    )?;
    if !has_margin {
        finalize_historical_stage(&mut state, request);
        return Ok(state);
    }

    let areas = topology.dual_area_steradians();
    let total_area = areas.iter().sum::<f64>().max(1.0e-12);
    let mut area_weighted_deflection = 0.0_f64;
    for sample in 0..count {
        let deflection = passive_margin_deflection_m(inherited, sample);
        if deflection == 0.0 {
            continue;
        }
        state.rift_basin_elevation_m[sample] += deflection as f32;
        state.solid_elevation_m[sample] += deflection as f32;
        area_weighted_deflection += deflection * areas[sample];
    }

    // Preserve the WG-4 global solid datum after adding the local shelf/basin term.
    let datum_shift = area_weighted_deflection / total_area;
    let mut newly_clamped = 0_u32;
    for value in &mut state.solid_elevation_m {
        let shifted = f64::from(*value) - datum_shift;
        let clamped = shifted.clamp(-20_000.0, 15_000.0);
        if clamped.to_bits() != shifted.to_bits() {
            newly_clamped += 1;
        }
        *value = clamped as f32;
    }

    let prior_hash = state.metrics.topography_hash;
    let prior_clamped = state
        .metrics
        .clamped_sample_count
        .saturating_add(newly_clamped);
    refresh_water_and_metrics(
        topology,
        inherited,
        &mut state,
        planet,
        prior_hash,
        prior_clamped,
    )?;
    finalize_historical_stage(&mut state, request);
    Ok(state)
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
            if inherited.structural_zone_kind[sample]
                == InheritedStructureKind::ContinentalMargin as u8
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
        assert_eq!(
            historical.metrics.clamped_sample_count,
            legacy.metrics.clamped_sample_count
        );
        assert_eq!(historical.stage.id, HISTORICAL_TOPOGRAPHY_STAGE_ID);
        assert_eq!(
            historical.stage.version,
            HISTORICAL_TOPOGRAPHY_STAGE_VERSION
        );
        assert_ne!(
            historical.metrics.topography_hash,
            legacy.metrics.topography_hash
        );
    }
}
