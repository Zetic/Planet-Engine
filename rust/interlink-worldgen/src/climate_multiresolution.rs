use crate::climate::{
    build_scalar_gradient_geometry, effective_shortwave_albedo,
    generate_coupled_climate_reference_internal, scalar_gradient_cached,
};
use crate::{
    build_icosphere, build_refinement_map, refine_scalar_f32, tangent_basis,
    ClimateGenerationDiagnostics, ClimateRequest, ClimateState, GeodesicTopology,
    PlanetPhysicalParameters, TopographyMetrics, TopographyState, WorldgenError,
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn hash_f32_slice(mut hash: u64, values: &[f32]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    for value in values {
        hash = fnv_update(hash, &value.to_bits().to_le_bytes());
    }
    hash
}

fn area_weighted_mean(topology: &GeodesicTopology, values: &[f32]) -> f64 {
    values
        .iter()
        .enumerate()
        .map(|(sample, value)| f64::from(*value) * topology.dual_area_steradians()[sample])
        .sum::<f64>()
        / topology.metrics().total_area_steradians.max(1.0e-18)
}

fn subset_area_weighted_mean(
    topology: &GeodesicTopology,
    values: &[f32],
    include: impl Fn(usize) -> bool,
) -> f64 {
    let mut weighted = 0.0;
    let mut area = 0.0;
    for (sample, value) in values.iter().enumerate() {
        if include(sample) {
            let weight = topology.dual_area_steradians()[sample];
            weighted += f64::from(*value) * weight;
            area += weight;
        }
    }
    weighted / area.max(1.0e-18)
}

fn percentile(values: &[f32], fraction: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let index =
        ((sorted.len().saturating_sub(1)) as f64 * fraction.clamp(0.0, 1.0)).round() as usize;
    f64::from(sorted[index])
}

fn radiative_target_k(
    annual_mean_insolation_w_m2: f64,
    surface_height_m: f64,
    submerged: bool,
    temperature_k: f64,
    sea_surface_temperature_k: f64,
    pressure_pa: f64,
    planet: PlanetPhysicalParameters,
    request: &ClimateRequest,
) -> f64 {
    let parameters = request.parameters;
    let physical = request.physical;
    let base_albedo = if submerged {
        parameters.ocean_albedo
    } else {
        parameters.land_albedo
    };
    let cold_state = if submerged {
        sea_surface_temperature_k < parameters.sea_ice_temperature_k
    } else {
        temperature_k < parameters.snow_temperature_k
    };
    let albedo = if cold_state {
        base_albedo + parameters.snow_albedo_feedback * (parameters.snow_ice_albedo - base_albedo)
    } else {
        base_albedo
    }
    .clamp(0.0, 0.95);
    let effective_albedo = if planet.reference_surface_pressure_pa > 0.0 {
        effective_shortwave_albedo(
            physical.atmospheric_shortwave_reflectivity,
            parameters.surface_albedo_shortwave_coupling,
            albedo,
        )
    } else {
        albedo
    };
    let absorbed = (annual_mean_insolation_w_m2 * (1.0 - effective_albedo)
        + planet.internal_heat_flux_w_per_m2)
        .max(0.0);
    let effective_temperature = if absorbed > 0.0 {
        (absorbed / STEFAN_BOLTZMANN).powf(0.25)
    } else {
        120.0
    };
    let pressure_ratio = if planet.reference_surface_pressure_pa > 0.0 {
        (pressure_pa / planet.reference_surface_pressure_pa).clamp(0.0, 2.0)
    } else {
        0.0
    };
    let greenhouse =
        (1.0 + 0.75 * physical.atmospheric_longwave_optical_depth * pressure_ratio).powf(0.25);
    (effective_temperature * greenhouse - parameters.lapse_rate_k_per_m * surface_height_m)
        .clamp(120.0, 355.0)
}

fn aggregate_topography(
    fine_topology: &GeodesicTopology,
    terrain: &TopographyState,
    climate_level: u8,
) -> Result<(GeodesicTopology, TopographyState), WorldgenError> {
    let climate_topology = build_icosphere(climate_level)?;
    let map = build_refinement_map(fine_topology, climate_level)?;
    let coarse_count = climate_topology.metrics().sample_count as usize;
    let mut area_sum = vec![0.0; coarse_count];
    let mut solid_sum = vec![0.0; coarse_count];
    let mut land_area = vec![0.0; coarse_count];
    let mut land_height_sum = vec![0.0; coarse_count];
    let mut ocean_area = vec![0.0; coarse_count];
    let mut ocean_depth_sum = vec![0.0; coarse_count];

    for fine in 0..fine_topology.metrics().sample_count as usize {
        let coarse = map.nearest_coarse_source[fine] as usize;
        let area = fine_topology.dual_area_steradians()[fine];
        area_sum[coarse] += area;
        solid_sum[coarse] += f64::from(terrain.solid_elevation_m[fine]) * area;
        if terrain.submerged_mask[fine] == 0 {
            land_area[coarse] += area;
            land_height_sum[coarse] +=
                f64::from(terrain.elevation_above_sea_level_m[fine]).max(0.0) * area;
        } else {
            ocean_area[coarse] += area;
            ocean_depth_sum[coarse] += f64::from(terrain.water_depth_m[fine]).max(0.0) * area;
        }
    }

    let mut solid_elevation_m = vec![0.0_f32; coarse_count];
    let mut elevation_above_sea_level_m = vec![0.0_f32; coarse_count];
    let mut water_depth_m = vec![0.0_f32; coarse_count];
    let mut submerged_mask = vec![0_u8; coarse_count];
    for coarse in 0..coarse_count {
        solid_elevation_m[coarse] = (solid_sum[coarse] / area_sum[coarse].max(1.0e-15)) as f32;
        if ocean_area[coarse] > land_area[coarse] {
            submerged_mask[coarse] = 1;
            water_depth_m[coarse] =
                (ocean_depth_sum[coarse] / ocean_area[coarse].max(1.0e-15)) as f32;
            elevation_above_sea_level_m[coarse] = -water_depth_m[coarse];
        } else {
            elevation_above_sea_level_m[coarse] =
                (land_height_sum[coarse] / land_area[coarse].max(1.0e-15)) as f32;
        }
    }

    let mut topography_hash = FNV_OFFSET_BASIS;
    topography_hash = fnv_update(topography_hash, b"climate:aggregated-topography:v1");
    topography_hash = fnv_update(topography_hash, &[climate_level, fine_topology.level()]);
    topography_hash = fnv_update(
        topography_hash,
        &terrain.metrics.topography_hash.to_le_bytes(),
    );
    topography_hash = hash_f32_slice(topography_hash, &solid_elevation_m);
    topography_hash = hash_f32_slice(topography_hash, &elevation_above_sea_level_m);
    topography_hash = hash_f32_slice(topography_hash, &water_depth_m);
    topography_hash = fnv_update(topography_hash, &submerged_mask);

    let zero = vec![0.0_f32; coarse_count];
    let mut metrics: TopographyMetrics = terrain.metrics.clone();
    metrics.sample_count = coarse_count as u32;
    metrics.topography_hash = topography_hash;
    Ok((
        climate_topology,
        TopographyState {
            stage: terrain.stage.clone(),
            metrics,
            isostatic_elevation_m: zero.clone(),
            thermal_elevation_m: zero.clone(),
            orogenic_elevation_m: zero.clone(),
            ridge_elevation_m: zero.clone(),
            rift_basin_elevation_m: zero.clone(),
            trench_elevation_m: zero.clone(),
            arc_elevation_m: zero.clone(),
            mantle_dynamic_elevation_m: zero,
            solid_elevation_m,
            elevation_above_sea_level_m,
            water_depth_m,
            submerged_mask,
        },
    ))
}

fn refine_climate_state(
    fine_topology: &GeodesicTopology,
    fine_terrain: &TopographyState,
    coarse_terrain: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &ClimateRequest,
    climate_level: u8,
    coarse: ClimateState,
) -> Result<(ClimateState, Vec<f32>), WorldgenError> {
    macro_rules! refine {
        ($field:ident) => {
            refine_scalar_f32(fine_topology, climate_level, &coarse.$field)?
        };
    }

    let mut output = ClimateState {
        stage: coarse.stage.clone(),
        metrics: coarse.metrics.clone(),
        annual_mean_insolation_w_m2: refine!(annual_mean_insolation_w_m2),
        seasonal_insolation_amplitude_w_m2: refine!(seasonal_insolation_amplitude_w_m2),
        temperature_mean_k: refine!(temperature_mean_k),
        temperature_annual_cos_k: refine!(temperature_annual_cos_k),
        temperature_annual_sin_k: refine!(temperature_annual_sin_k),
        temperature_min_k: refine!(temperature_min_k),
        temperature_max_k: refine!(temperature_max_k),
        local_pressure_pa: refine!(local_pressure_pa),
        wind_east_mean_m_s: refine!(wind_east_mean_m_s),
        wind_north_mean_m_s: refine!(wind_north_mean_m_s),
        wind_east_annual_cos_m_s: refine!(wind_east_annual_cos_m_s),
        wind_east_annual_sin_m_s: refine!(wind_east_annual_sin_m_s),
        wind_north_annual_cos_m_s: refine!(wind_north_annual_cos_m_s),
        wind_north_annual_sin_m_s: refine!(wind_north_annual_sin_m_s),
        sea_surface_temperature_mean_k: refine!(sea_surface_temperature_mean_k),
        sea_surface_temperature_annual_cos_k: refine!(sea_surface_temperature_annual_cos_k),
        sea_surface_temperature_annual_sin_k: refine!(sea_surface_temperature_annual_sin_k),
        current_east_mean_m_s: refine!(current_east_mean_m_s),
        current_north_mean_m_s: refine!(current_north_mean_m_s),
        current_east_annual_cos_m_s: refine!(current_east_annual_cos_m_s),
        current_east_annual_sin_m_s: refine!(current_east_annual_sin_m_s),
        current_north_annual_cos_m_s: refine!(current_north_annual_cos_m_s),
        current_north_annual_sin_m_s: refine!(current_north_annual_sin_m_s),
        current_speed_mean_m_s: refine!(current_speed_mean_m_s),
        ocean_heat_transport_index: refine!(ocean_heat_transport_index),
        specific_humidity_mean: refine!(specific_humidity_mean),
        annual_precipitation_mm: refine!(annual_precipitation_mm),
        precipitation_seasonality: refine!(precipitation_seasonality),
        potential_evaporation_mm: refine!(potential_evaporation_mm),
        moisture_balance_mm: refine!(moisture_balance_mm),
        aridity_index: refine!(aridity_index),
        snowfall_fraction: refine!(snowfall_fraction),
        persistent_snow_potential: refine!(persistent_snow_potential),
        sea_ice_potential: refine!(sea_ice_potential),
    };
    // Orographic downscaling redistributes precipitation at fine resolution. Preserve the
    // area-integrated amount inherited from the conservative global solve so that local relief
    // cannot create or destroy water.
    let inherited_precipitation_mean_mm = coarse.metrics.mean_annual_precipitation_mm;

    let coarse_surface_height_m = coarse_terrain
        .elevation_above_sea_level_m
        .iter()
        .enumerate()
        .map(|(sample, value)| {
            if coarse_terrain.submerged_mask[sample] == 0 {
                value.max(0.0)
            } else {
                0.0
            }
        })
        .collect::<Vec<_>>();
    let inherited_surface_height_m =
        refine_scalar_f32(fine_topology, climate_level, &coarse_surface_height_m)?;
    let coarse_radiative_target_k = (0..coarse_terrain.metrics.sample_count as usize)
        .map(|sample| {
            radiative_target_k(
                f64::from(coarse.annual_mean_insolation_w_m2[sample]),
                f64::from(coarse_surface_height_m[sample]),
                coarse_terrain.submerged_mask[sample] != 0,
                f64::from(coarse.temperature_mean_k[sample]),
                f64::from(coarse.sea_surface_temperature_mean_k[sample]),
                f64::from(coarse.local_pressure_pa[sample]),
                planet,
                request,
            ) as f32
        })
        .collect::<Vec<_>>();
    let inherited_radiative_target_k =
        refine_scalar_f32(fine_topology, climate_level, &coarse_radiative_target_k)?;
    let fine_surface_height_m = fine_terrain
        .elevation_above_sea_level_m
        .iter()
        .enumerate()
        .map(|(sample, value)| {
            if fine_terrain.submerged_mask[sample] == 0 {
                value.max(0.0)
            } else {
                0.0
            }
        })
        .collect::<Vec<_>>();
    let east_bases = fine_topology
        .positions()
        .iter()
        .map(|position| tangent_basis(*position).map(|basis| basis.east))
        .collect::<Result<Vec<_>, _>>()?;
    let north_bases = fine_topology
        .positions()
        .iter()
        .map(|position| tangent_basis(*position).map(|basis| basis.north))
        .collect::<Result<Vec<_>, _>>()?;
    let gradient_geometry =
        build_scalar_gradient_geometry(fine_topology, planet.radius_m, &east_bases, &north_bases);
    let fine_surface_f64 = fine_surface_height_m
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    let inherited_surface_f64 = inherited_surface_height_m
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    let mut precipitation_downscale_ratio = vec![1.0_f32; fine_topology.positions().len()];

    for sample in 0..fine_topology.metrics().sample_count as usize {
        let height_delta_m =
            f64::from(fine_surface_height_m[sample] - inherited_surface_height_m[sample]);
        let fine_target_k = radiative_target_k(
            f64::from(output.annual_mean_insolation_w_m2[sample]),
            f64::from(fine_surface_height_m[sample]),
            fine_terrain.submerged_mask[sample] != 0,
            f64::from(output.temperature_mean_k[sample]),
            f64::from(output.sea_surface_temperature_mean_k[sample]),
            f64::from(output.local_pressure_pa[sample]),
            planet,
            request,
        );
        let local_response = if fine_terrain.submerged_mask[sample] != 0 {
            request.parameters.ocean_thermal_relaxation
        } else {
            request.parameters.land_thermal_relaxation
        };
        let temperature_delta_k = (local_response
            * (fine_target_k - f64::from(inherited_radiative_target_k[sample])))
        .clamp(-20.0, 20.0);
        output.temperature_mean_k[sample] = (f64::from(output.temperature_mean_k[sample])
            + temperature_delta_k)
            .clamp(120.0, 355.0) as f32;
        output.temperature_min_k[sample] = (f64::from(output.temperature_min_k[sample])
            + temperature_delta_k)
            .clamp(120.0, 355.0) as f32;
        output.temperature_max_k[sample] = (f64::from(output.temperature_max_k[sample])
            + temperature_delta_k)
            .clamp(120.0, 355.0) as f32;
        if planet.reference_surface_pressure_pa > 0.0 {
            let specific_gas_constant =
                8.314_462_618 / request.physical.atmospheric_mean_molar_mass_kg_per_mol;
            let scale_height = (specific_gas_constant
                * f64::from(output.temperature_mean_k[sample])
                / planet.surface_gravity_m_s2)
                .max(1.0);
            output.local_pressure_pa[sample] = (f64::from(output.local_pressure_pa[sample])
                * (-height_delta_m / scale_height).exp())
            .max(0.0) as f32;
        }

        let (fine_gradient_east, fine_gradient_north) =
            scalar_gradient_cached(fine_topology, &gradient_geometry, &fine_surface_f64, sample);
        let (inherited_gradient_east, inherited_gradient_north) = scalar_gradient_cached(
            fine_topology,
            &gradient_geometry,
            &inherited_surface_f64,
            sample,
        );
        let fine_slope = fine_gradient_east.hypot(fine_gradient_north);
        let inherited_slope = inherited_gradient_east.hypot(inherited_gradient_north);
        let drag_ratio = ((1.0 + request.parameters.topographic_wind_drag * inherited_slope)
            / (1.0 + request.parameters.topographic_wind_drag * fine_slope))
            .clamp(0.35, 2.0) as f32;
        output.wind_east_mean_m_s[sample] *= drag_ratio;
        output.wind_north_mean_m_s[sample] *= drag_ratio;
        output.wind_east_annual_cos_m_s[sample] *= drag_ratio;
        output.wind_east_annual_sin_m_s[sample] *= drag_ratio;
        output.wind_north_annual_cos_m_s[sample] *= drag_ratio;
        output.wind_north_annual_sin_m_s[sample] *= drag_ratio;

        let wind_east = f64::from(output.wind_east_mean_m_s[sample]);
        let wind_north = f64::from(output.wind_north_mean_m_s[sample]);
        let wind_speed = wind_east.hypot(wind_north);
        if wind_speed > 0.25 {
            let fine_uplift = ((wind_east * fine_gradient_east + wind_north * fine_gradient_north)
                / wind_speed)
                .max(0.0);
            let inherited_uplift = ((wind_east * inherited_gradient_east
                + wind_north * inherited_gradient_north)
                / wind_speed)
                .max(0.0);
            let fine_orographic_fraction = (fine_uplift
                * request.parameters.orographic_precipitation_strength)
                .clamp(0.0, request.parameters.maximum_orographic_fraction);
            let inherited_orographic_fraction = (inherited_uplift
                * request.parameters.orographic_precipitation_strength)
                .clamp(0.0, request.parameters.maximum_orographic_fraction);
            let precipitation_ratio = ((1.0 + fine_orographic_fraction)
                / (1.0 + inherited_orographic_fraction))
                .clamp(0.5, 2.0) as f32;
            precipitation_downscale_ratio[sample] = precipitation_ratio;
            output.annual_precipitation_mm[sample] *= precipitation_ratio;
        }

        let pet_temperature_ratio = (0.06 * temperature_delta_k).exp().clamp(0.35, 2.5) as f32;
        output.potential_evaporation_mm[sample] *= pet_temperature_ratio;
        output.moisture_balance_mm[sample] =
            output.annual_precipitation_mm[sample] - output.potential_evaporation_mm[sample];
        output.aridity_index[sample] = if output.potential_evaporation_mm[sample] > 1.0 {
            (output.annual_precipitation_mm[sample] / output.potential_evaporation_mm[sample])
                .clamp(0.0, 4.0)
        } else {
            4.0
        };

        if fine_terrain.submerged_mask[sample] == 0 {
            output.current_east_mean_m_s[sample] = 0.0;
            output.current_north_mean_m_s[sample] = 0.0;
            output.current_east_annual_cos_m_s[sample] = 0.0;
            output.current_east_annual_sin_m_s[sample] = 0.0;
            output.current_north_annual_cos_m_s[sample] = 0.0;
            output.current_north_annual_sin_m_s[sample] = 0.0;
            output.current_speed_mean_m_s[sample] = 0.0;
            output.ocean_heat_transport_index[sample] = 0.0;
            output.sea_ice_potential[sample] = 0.0;
            output.sea_surface_temperature_mean_k[sample] = output.temperature_mean_k[sample];
            output.sea_surface_temperature_annual_cos_k[sample] =
                output.temperature_annual_cos_k[sample];
            output.sea_surface_temperature_annual_sin_k[sample] =
                output.temperature_annual_sin_k[sample];
        }
    }

    let downscaled_precipitation_mean_mm =
        area_weighted_mean(fine_topology, &output.annual_precipitation_mm);
    let precipitation_normalization =
        (inherited_precipitation_mean_mm / downscaled_precipitation_mean_mm.max(1.0e-18)) as f32;
    for sample in 0..fine_topology.metrics().sample_count as usize {
        output.annual_precipitation_mm[sample] *= precipitation_normalization;
        precipitation_downscale_ratio[sample] *= precipitation_normalization;
        output.moisture_balance_mm[sample] =
            output.annual_precipitation_mm[sample] - output.potential_evaporation_mm[sample];
        output.aridity_index[sample] = if output.potential_evaporation_mm[sample] > 1.0 {
            (output.annual_precipitation_mm[sample] / output.potential_evaporation_mm[sample])
                .clamp(0.0, 4.0)
        } else {
            4.0
        };
    }

    output.metrics.sample_count = fine_topology.metrics().sample_count;
    output.metrics.global_solver_level = climate_level;
    output.metrics.global_solver_sample_count = coarse.metrics.sample_count;
    output.metrics.mean_temperature_k =
        area_weighted_mean(fine_topology, &output.temperature_mean_k);
    output.metrics.minimum_temperature_k = output
        .temperature_min_k
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min) as f64;
    output.metrics.maximum_temperature_k = output
        .temperature_max_k
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max) as f64;
    output.metrics.mean_land_temperature_k =
        subset_area_weighted_mean(fine_topology, &output.temperature_mean_k, |sample| {
            fine_terrain.submerged_mask[sample] == 0
        });
    output.metrics.mean_ocean_temperature_k =
        subset_area_weighted_mean(fine_topology, &output.temperature_mean_k, |sample| {
            fine_terrain.submerged_mask[sample] != 0
        });
    output.metrics.mean_sea_surface_temperature_k = subset_area_weighted_mean(
        fine_topology,
        &output.sea_surface_temperature_mean_k,
        |sample| fine_terrain.submerged_mask[sample] != 0,
    );
    let phase_count = usize::from(request.parameters.orbital_phase_count);
    let mut wind_speed_mean = vec![0.0_f32; fine_topology.positions().len()];
    let mut maximum_wind_speed = 0.0_f64;
    for phase in 0..phase_count {
        let angle = std::f64::consts::TAU * phase as f64 / phase_count as f64;
        let phase_cos = angle.cos();
        let phase_sin = angle.sin();
        for (sample, mean_speed) in wind_speed_mean.iter_mut().enumerate() {
            let east = f64::from(output.wind_east_mean_m_s[sample])
                + f64::from(output.wind_east_annual_cos_m_s[sample]) * phase_cos
                + f64::from(output.wind_east_annual_sin_m_s[sample]) * phase_sin;
            let north = f64::from(output.wind_north_mean_m_s[sample])
                + f64::from(output.wind_north_annual_cos_m_s[sample]) * phase_cos
                + f64::from(output.wind_north_annual_sin_m_s[sample]) * phase_sin;
            let speed = east.hypot(north);
            *mean_speed += (speed / phase_count as f64) as f32;
            maximum_wind_speed = maximum_wind_speed.max(speed);
        }
    }
    output.metrics.mean_wind_speed_m_s = area_weighted_mean(fine_topology, &wind_speed_mean);
    output.metrics.maximum_wind_speed_m_s = maximum_wind_speed;
    output.metrics.mean_surface_current_m_s =
        subset_area_weighted_mean(fine_topology, &output.current_speed_mean_m_s, |sample| {
            fine_terrain.submerged_mask[sample] != 0
        });
    output.metrics.maximum_surface_current_m_s = output
        .current_speed_mean_m_s
        .iter()
        .copied()
        .fold(0.0_f32, f32::max) as f64;
    output.metrics.mean_annual_precipitation_mm =
        area_weighted_mean(fine_topology, &output.annual_precipitation_mm);
    output.metrics.p95_annual_precipitation_mm = percentile(&output.annual_precipitation_mm, 0.95);
    let radius_squared = planet.radius_m * planet.radius_m;
    output.metrics.global_precipitation_kg = output
        .annual_precipitation_mm
        .iter()
        .enumerate()
        .map(|(sample, precipitation_mm)| {
            f64::from(*precipitation_mm)
                * fine_topology.dual_area_steradians()[sample]
                * radius_squared
        })
        .sum();
    let total_area = fine_topology.metrics().total_area_steradians.max(1.0e-18);
    output.metrics.persistent_snow_area_fraction = output
        .persistent_snow_potential
        .iter()
        .enumerate()
        .map(|(sample, value)| f64::from(*value) * fine_topology.dual_area_steradians()[sample])
        .sum::<f64>()
        / total_area;
    output.metrics.sea_ice_area_fraction = output
        .sea_ice_potential
        .iter()
        .enumerate()
        .map(|(sample, value)| f64::from(*value) * fine_topology.dual_area_steradians()[sample])
        .sum::<f64>()
        / total_area;
    output.metrics.climate_physical_parameter_hash = request.physical.parameter_hash();
    output.metrics.climate_model_parameter_hash = request.parameters.parameter_hash();
    let mut climate_hash = FNV_OFFSET_BASIS;
    climate_hash = fnv_update(climate_hash, b"climate:multiresolution:v1");
    climate_hash = fnv_update(climate_hash, &[climate_level, fine_topology.level()]);
    climate_hash = fnv_update(
        climate_hash,
        &fine_terrain.metrics.topography_hash.to_le_bytes(),
    );
    climate_hash = fnv_update(climate_hash, &coarse.metrics.climate_hash.to_le_bytes());
    for values in [
        &output.annual_mean_insolation_w_m2,
        &output.seasonal_insolation_amplitude_w_m2,
        &output.temperature_mean_k,
        &output.temperature_annual_cos_k,
        &output.temperature_annual_sin_k,
        &output.temperature_min_k,
        &output.temperature_max_k,
        &output.local_pressure_pa,
        &output.wind_east_mean_m_s,
        &output.wind_north_mean_m_s,
        &output.wind_east_annual_cos_m_s,
        &output.wind_east_annual_sin_m_s,
        &output.wind_north_annual_cos_m_s,
        &output.wind_north_annual_sin_m_s,
        &output.sea_surface_temperature_mean_k,
        &output.sea_surface_temperature_annual_cos_k,
        &output.sea_surface_temperature_annual_sin_k,
        &output.current_east_mean_m_s,
        &output.current_north_mean_m_s,
        &output.current_east_annual_cos_m_s,
        &output.current_east_annual_sin_m_s,
        &output.current_north_annual_cos_m_s,
        &output.current_north_annual_sin_m_s,
        &output.current_speed_mean_m_s,
        &output.ocean_heat_transport_index,
        &output.specific_humidity_mean,
        &output.annual_precipitation_mm,
        &output.precipitation_seasonality,
        &output.potential_evaporation_mm,
        &output.moisture_balance_mm,
        &output.aridity_index,
        &output.snowfall_fraction,
        &output.persistent_snow_potential,
        &output.sea_ice_potential,
    ] {
        climate_hash = hash_f32_slice(climate_hash, values);
    }
    output.metrics.climate_hash = climate_hash;
    Ok((output, precipitation_downscale_ratio))
}

pub(crate) fn generate_multiresolution_climate(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &ClimateRequest,
) -> Result<ClimateState, WorldgenError> {
    let mut progress = |_completed_years: u8, _maximum_years: u8| {};
    let (climate, _) = generate_multiresolution_climate_internal(
        topology,
        terrain,
        planet,
        request,
        false,
        &mut progress,
    )?;
    Ok(climate)
}

pub(crate) fn generate_multiresolution_climate_with_diagnostics(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &ClimateRequest,
    progress: &mut dyn FnMut(u8, u8),
) -> Result<(ClimateState, ClimateGenerationDiagnostics), WorldgenError> {
    generate_multiresolution_climate_internal(topology, terrain, planet, request, true, progress)
}

fn generate_multiresolution_climate_internal(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &ClimateRequest,
    capture_precipitation_phases: bool,
    progress: &mut dyn FnMut(u8, u8),
) -> Result<(ClimateState, ClimateGenerationDiagnostics), WorldgenError> {
    request
        .parameters
        .validate()
        .map_err(WorldgenError::InvalidClimate)?;
    let maximum_global_climate_level = request.parameters.maximum_global_climate_level;
    if maximum_global_climate_level >= topology.level() {
        return generate_coupled_climate_reference_internal(
            topology,
            terrain,
            planet,
            request,
            capture_precipitation_phases,
            progress,
        );
    }
    let (climate_topology, aggregated_terrain) =
        aggregate_topography(topology, terrain, maximum_global_climate_level)?;
    let mut coarse_request = request.clone();
    let level_delta = topology.level() - maximum_global_climate_level;
    let linear_spacing_ratio = 1_u16 << level_delta;
    coarse_request.parameters.atmospheric_heat_solver_iterations =
        ((u16::from(request.parameters.atmospheric_heat_solver_iterations) + linear_spacing_ratio
            - 1)
            / linear_spacing_ratio)
            .max(1) as u8;
    let (coarse_climate, coarse_diagnostics) = generate_coupled_climate_reference_internal(
        &climate_topology,
        &aggregated_terrain,
        planet,
        &coarse_request,
        capture_precipitation_phases,
        progress,
    )?;
    let (climate, precipitation_downscale_ratio) = refine_climate_state(
        topology,
        terrain,
        &aggregated_terrain,
        planet,
        request,
        maximum_global_climate_level,
        coarse_climate,
    )?;
    let phase_count = usize::from(request.parameters.orbital_phase_count);
    let fine_count = topology.metrics().sample_count as usize;
    let coarse_count = climate_topology.metrics().sample_count as usize;
    let mut precipitation_phase_rate_mm_year = if capture_precipitation_phases {
        Vec::with_capacity(phase_count * fine_count)
    } else {
        Vec::new()
    };
    if capture_precipitation_phases {
        for phase in 0..phase_count {
            let start = phase * coarse_count;
            let mut refined = refine_scalar_f32(
                topology,
                maximum_global_climate_level,
                &coarse_diagnostics.precipitation_phase_rate_mm_year[start..start + coarse_count],
            )?;
            for (value, ratio) in refined.iter_mut().zip(&precipitation_downscale_ratio) {
                *value *= *ratio;
            }
            precipitation_phase_rate_mm_year.extend(refined);
        }
    }
    Ok((
        climate,
        ClimateGenerationDiagnostics {
            precipitation_phase_rate_mm_year,
        },
    ))
}
