use crate::{
    tangent_basis, ClimateRequest, ClimateState, GeodesicTopology,
    PlanetPhysicalParameters, TopographyState, WorldgenError,
};

const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;
const TWO_PI: f64 = std::f64::consts::PI * 2.0;
const LATITUDE_BAND_DEGREES: i32 = 15;

#[derive(Clone, Debug, PartialEq)]
pub struct ClimateLatitudeBand {
    pub minimum_latitude_deg: f64,
    pub maximum_latitude_deg: f64,
    pub area_fraction: f64,
    pub mean_temperature_k: f64,
    pub mean_precipitation_mm: f64,
    pub mean_specific_humidity: f64,
    pub mean_relative_humidity_proxy: f64,
    pub mean_sea_surface_temperature_k: Option<f64>,
    pub mean_snowfall_mm: f64,
    pub mean_sea_ice_potential: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClimateCalibrationReport {
    pub sample_count: u32,
    pub orbital_phase_count: u8,
    pub mean_temperature_k: f64,
    pub mean_land_temperature_k: f64,
    pub mean_ocean_temperature_k: f64,
    pub land_ocean_temperature_contrast_k: f64,
    pub mean_sea_surface_temperature_k: f64,
    pub mean_land_elevation_m: f64,
    pub median_land_elevation_m: f64,
    pub p95_land_elevation_m: f64,
    pub land_area_above_2km_fraction: f64,
    pub land_area_above_4km_fraction: f64,
    pub clear_surface_absorbed_shortwave_w_m2: f64,
    pub clear_surface_absorbed_shortwave_land_w_m2: f64,
    pub clear_surface_absorbed_shortwave_ocean_w_m2: f64,
    pub outgoing_longwave_proxy_w_m2: f64,
    pub outgoing_longwave_land_proxy_w_m2: f64,
    pub outgoing_longwave_ocean_proxy_w_m2: f64,
    pub toa_energy_imbalance_proxy_w_m2: f64,
    pub reconstructed_wind_cap_fraction: f64,
    pub reconstructed_moisture_edge_cap_fraction: f64,
    pub mean_state_relative_humidity_p05: f64,
    pub mean_state_relative_humidity_p50: f64,
    pub mean_state_relative_humidity_p95: f64,
    pub mean_annual_precipitation_mm: f64,
    pub p95_annual_precipitation_mm: f64,
    pub mean_potential_evaporation_mm: f64,
    pub precipitation_to_evaporation_ratio: f64,
    pub no_orography_mean_annual_precipitation_mm: Option<f64>,
    pub orographic_precipitation_causal_fraction: Option<f64>,
    pub mean_snowfall_mm: f64,
    pub persistent_snow_area_fraction: f64,
    pub sea_ice_area_fraction: f64,
    pub annual_mean_ocean_heat_tendency_rms_index: f64,
    pub latitude_bands: Vec<ClimateLatitudeBand>,
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn reconstructed(mean: f32, cosine: f32, sine: f32, phase_cos: f64, phase_sin: f64) -> f64 {
    f64::from(mean) + f64::from(cosine) * phase_cos + f64::from(sine) * phase_sin
}

fn saturation_specific_humidity(temperature_k: f64, pressure_pa: f64) -> f64 {
    if pressure_pa <= 1.0 || temperature_k <= 100.0 {
        return 0.0;
    }
    let celsius = temperature_k - 273.15;
    let exponent = (17.625 * celsius / (celsius + 243.04)).clamp(-60.0, 60.0);
    let vapor_pressure = (610.94 * exponent.exp()).min(pressure_pa * 0.95);
    let denominator = pressure_pa - 0.378 * vapor_pressure;
    if denominator <= 1.0 {
        0.2
    } else {
        (0.622 * vapor_pressure / denominator).clamp(0.0, 0.2)
    }
}

fn weighted_mean(
    topology: &GeodesicTopology,
    values: impl Iterator<Item = (usize, f64)>,
) -> f64 {
    let mut weighted = 0.0;
    let mut area = 0.0;
    for (index, value) in values {
        let sample_area = topology.dual_area_steradians()[index];
        weighted += value * sample_area;
        area += sample_area;
    }
    if area > 0.0 { weighted / area } else { 0.0 }
}

fn weighted_fraction(
    topology: &GeodesicTopology,
    indices: impl Iterator<Item = usize>,
    predicate: impl Fn(usize) -> bool,
) -> f64 {
    let mut matching = 0.0;
    let mut total = 0.0;
    for index in indices {
        let area = topology.dual_area_steradians()[index];
        total += area;
        if predicate(index) {
            matching += area;
        }
    }
    if total > 0.0 { matching / total } else { 0.0 }
}

fn weighted_percentile(mut values: Vec<(f64, f64)>, fraction: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.0.total_cmp(&b.0));
    let total_weight = values.iter().map(|(_, weight)| *weight).sum::<f64>().max(1.0e-18);
    let target = total_weight * fraction.clamp(0.0, 1.0);
    let mut cumulative = 0.0;
    for (value, weight) in values.iter().copied() {
        cumulative += weight;
        if cumulative >= target {
            return value;
        }
    }
    values.last().map(|(value, _)| *value).unwrap_or(0.0)
}

fn edge_direction(
    topology: &GeodesicTopology,
    from: usize,
    to: usize,
    east: [f64; 3],
    north: [f64; 3],
) -> Option<(f64, f64)> {
    let origin = topology.positions()[from];
    let target = topology.positions()[to];
    let radial = dot(target, origin);
    let tangent = [
        target[0] - origin[0] * radial,
        target[1] - origin[1] * radial,
        target[2] - origin[2] * radial,
    ];
    let magnitude = dot(tangent, tangent).sqrt();
    if magnitude <= 1.0e-15 {
        return None;
    }
    let direction = [
        tangent[0] / magnitude,
        tangent[1] / magnitude,
        tangent[2] / magnitude,
    ];
    Some((dot(direction, east), dot(direction, north)))
}

fn symmetric_edge_normal_wind(
    topology: &GeodesicTopology,
    a: usize,
    b: usize,
    east_bases: &[[f64; 3]],
    north_bases: &[[f64; 3]],
    wind_east: &[f64],
    wind_north: &[f64],
) -> Option<f64> {
    let (ae, an) = edge_direction(topology, a, b, east_bases[a], north_bases[a])?;
    let (be, bn) = edge_direction(topology, b, a, east_bases[b], north_bases[b])?;
    let outward_a = wind_east[a] * ae + wind_north[a] * an;
    let outward_b = wind_east[b] * be + wind_north[b] * bn;
    Some(0.5 * (outward_a - outward_b))
}

fn latitude_bands(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    climate: &ClimateState,
) -> Vec<ClimateLatitudeBand> {
    let mut result = Vec::new();
    let total_area = topology.metrics().total_area_steradians.max(1.0e-18);
    for minimum in (-90..90).step_by(LATITUDE_BAND_DEGREES as usize) {
        let maximum = minimum + LATITUDE_BAND_DEGREES;
        let indices = (0..climate.temperature_mean_k.len()).filter(|index| {
            let latitude = topology.positions()[*index][2].clamp(-1.0, 1.0).asin().to_degrees();
            latitude >= f64::from(minimum)
                && (latitude < f64::from(maximum) || maximum == 90 && latitude <= 90.0)
        }).collect::<Vec<_>>();
        let band_area = indices.iter().map(|index| topology.dual_area_steradians()[*index]).sum::<f64>();
        let mean_temperature_k = weighted_mean(
            topology,
            indices.iter().copied().map(|index| (index, f64::from(climate.temperature_mean_k[index]))),
        );
        let mean_precipitation_mm = weighted_mean(
            topology,
            indices.iter().copied().map(|index| (index, f64::from(climate.annual_precipitation_mm[index]))),
        );
        let mean_specific_humidity = weighted_mean(
            topology,
            indices.iter().copied().map(|index| (index, f64::from(climate.specific_humidity_mean[index]))),
        );
        let mean_relative_humidity_proxy = weighted_mean(
            topology,
            indices.iter().copied().map(|index| {
                let saturation = saturation_specific_humidity(
                    f64::from(climate.temperature_mean_k[index]),
                    f64::from(climate.local_pressure_pa[index]),
                );
                let rh = if saturation > 1.0e-12 {
                    f64::from(climate.specific_humidity_mean[index]) / saturation
                } else {
                    0.0
                };
                (index, rh.clamp(0.0, 2.0))
            }),
        );
        let ocean_indices = indices.iter().copied().filter(|index| terrain.submerged_mask[*index] != 0).collect::<Vec<_>>();
        let mean_sst = if ocean_indices.is_empty() {
            None
        } else {
            Some(weighted_mean(
                topology,
                ocean_indices.iter().copied().map(|index| (index, f64::from(climate.sea_surface_temperature_mean_k[index]))),
            ))
        };
        let mean_snowfall_mm = weighted_mean(
            topology,
            indices.iter().copied().map(|index| {
                (index, f64::from(climate.annual_precipitation_mm[index]) * f64::from(climate.snowfall_fraction[index]))
            }),
        );
        let mean_sea_ice_potential = weighted_mean(
            topology,
            indices.iter().copied().map(|index| (index, f64::from(climate.sea_ice_potential[index]))),
        );
        result.push(ClimateLatitudeBand {
            minimum_latitude_deg: f64::from(minimum),
            maximum_latitude_deg: f64::from(maximum),
            area_fraction: band_area / total_area,
            mean_temperature_k,
            mean_precipitation_mm,
            mean_specific_humidity,
            mean_relative_humidity_proxy,
            mean_sea_surface_temperature_k: mean_sst,
            mean_snowfall_mm,
            mean_sea_ice_potential,
        });
    }
    result
}

pub fn build_climate_calibration_report(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &ClimateRequest,
    climate: &ClimateState,
    no_orography_climate: Option<&ClimateState>,
) -> Result<ClimateCalibrationReport, WorldgenError> {
    let count = topology.metrics().sample_count as usize;
    if climate.temperature_mean_k.len() != count || terrain.submerged_mask.len() != count {
        return Err(WorldgenError::InvalidClimate(
            "WG-5 calibration inputs must match topology sample count",
        ));
    }
    let ocean = |index: usize| terrain.submerged_mask[index] != 0;
    let land_indices = (0..count).filter(|index| !ocean(*index)).collect::<Vec<_>>();
    let ocean_indices = (0..count).filter(|index| ocean(*index)).collect::<Vec<_>>();

    let land_elevation_values = land_indices.iter().map(|index| {
        (
            f64::from(terrain.elevation_above_sea_level_m[*index]),
            topology.dual_area_steradians()[*index],
        )
    }).collect::<Vec<_>>();
    let mean_land_elevation_m = weighted_mean(
        topology,
        land_indices.iter().copied().map(|index| (index, f64::from(terrain.elevation_above_sea_level_m[index]))),
    );
    let median_land_elevation_m = weighted_percentile(land_elevation_values.clone(), 0.50);
    let p95_land_elevation_m = weighted_percentile(land_elevation_values, 0.95);
    let land_area_above_2km_fraction = weighted_fraction(topology, land_indices.iter().copied(), |index| terrain.elevation_above_sea_level_m[index] >= 2_000.0);
    let land_area_above_4km_fraction = weighted_fraction(topology, land_indices.iter().copied(), |index| terrain.elevation_above_sea_level_m[index] >= 4_000.0);

    let clear_asr = |index: usize| {
        let albedo = if ocean(index) {
            request.parameters.ocean_albedo
        } else {
            request.parameters.land_albedo
        };
        f64::from(climate.annual_mean_insolation_w_m2[index]) * (1.0 - albedo)
    };
    let olr_proxy = |index: usize| {
        let pressure_ratio = if planet.reference_surface_pressure_pa > 0.0 {
            (f64::from(climate.local_pressure_pa[index]) / planet.reference_surface_pressure_pa).clamp(0.0, 2.0)
        } else {
            0.0
        };
        let greenhouse_denominator = 1.0
            + 0.75 * request.physical.atmospheric_longwave_optical_depth * pressure_ratio;
        STEFAN_BOLTZMANN * f64::from(climate.temperature_mean_k[index]).powi(4)
            / greenhouse_denominator.max(1.0e-12)
    };
    let clear_surface_absorbed_shortwave_w_m2 = weighted_mean(topology, (0..count).map(|index| (index, clear_asr(index))));
    let clear_surface_absorbed_shortwave_land_w_m2 = weighted_mean(topology, land_indices.iter().copied().map(|index| (index, clear_asr(index))));
    let clear_surface_absorbed_shortwave_ocean_w_m2 = weighted_mean(topology, ocean_indices.iter().copied().map(|index| (index, clear_asr(index))));
    let outgoing_longwave_proxy_w_m2 = weighted_mean(topology, (0..count).map(|index| (index, olr_proxy(index))));
    let outgoing_longwave_land_proxy_w_m2 = weighted_mean(topology, land_indices.iter().copied().map(|index| (index, olr_proxy(index))));
    let outgoing_longwave_ocean_proxy_w_m2 = weighted_mean(topology, ocean_indices.iter().copied().map(|index| (index, olr_proxy(index))));
    let toa_energy_imbalance_proxy_w_m2 = clear_surface_absorbed_shortwave_w_m2
        + planet.internal_heat_flux_w_per_m2
        - outgoing_longwave_proxy_w_m2;

    let mut east_bases = Vec::with_capacity(count);
    let mut north_bases = Vec::with_capacity(count);
    for position in topology.positions() {
        let basis = tangent_basis(*position)?;
        east_bases.push(basis.east);
        north_bases.push(basis.north);
    }
    let phase_count = usize::from(climate.metrics.orbital_phase_count);
    let phase_seconds = planet.orbital_period_s / phase_count as f64;
    let mut wind_east = vec![0.0; count];
    let mut wind_north = vec![0.0; count];
    let mut wind_cap_samples = 0_u64;
    let mut wind_samples = 0_u64;
    let mut moisture_cap_edges = 0_u64;
    let mut moisture_edges = 0_u64;
    for phase in 0..phase_count {
        let angle = TWO_PI * phase as f64 / phase_count as f64;
        let phase_cos = angle.cos();
        let phase_sin = angle.sin();
        for index in 0..count {
            wind_east[index] = reconstructed(
                climate.wind_east_mean_m_s[index],
                climate.wind_east_annual_cos_m_s[index],
                climate.wind_east_annual_sin_m_s[index],
                phase_cos,
                phase_sin,
            );
            wind_north[index] = reconstructed(
                climate.wind_north_mean_m_s[index],
                climate.wind_north_annual_cos_m_s[index],
                climate.wind_north_annual_sin_m_s[index],
                phase_cos,
                phase_sin,
            );
            let speed = (wind_east[index] * wind_east[index] + wind_north[index] * wind_north[index]).sqrt();
            if speed >= request.parameters.maximum_wind_speed_m_s * 0.98 {
                wind_cap_samples += 1;
            }
            wind_samples += 1;
        }
        for a in 0..count {
            for (neighbor, arc) in topology.neighbors_of(a as u32).iter().zip(topology.neighbor_arc_lengths_of(a as u32).iter()) {
                let b = *neighbor as usize;
                if b <= a {
                    continue;
                }
                let Some(projected) = symmetric_edge_normal_wind(
                    topology,
                    a,
                    b,
                    &east_bases,
                    &north_bases,
                    &wind_east,
                    &wind_north,
                ) else {
                    continue;
                };
                let distance_m = (*arc * planet.radius_m).max(1.0);
                let requested_fraction = projected.abs() * phase_seconds / distance_m
                    * request.parameters.moisture_transport_cfl;
                if requested_fraction >= 0.22 {
                    moisture_cap_edges += 1;
                }
                moisture_edges += 1;
            }
        }
    }

    let rh_values = (0..count).map(|index| {
        let saturation = saturation_specific_humidity(
            f64::from(climate.temperature_mean_k[index]),
            f64::from(climate.local_pressure_pa[index]),
        );
        let rh = if saturation > 1.0e-12 {
            f64::from(climate.specific_humidity_mean[index]) / saturation
        } else {
            0.0
        };
        (rh.clamp(0.0, 2.0), topology.dual_area_steradians()[index])
    }).collect::<Vec<_>>();

    let mean_potential_evaporation_mm = weighted_mean(
        topology,
        (0..count).map(|index| (index, f64::from(climate.potential_evaporation_mm[index]))),
    );
    let precipitation_to_evaporation_ratio = if climate.metrics.global_evaporation_kg.abs() > 1.0 {
        climate.metrics.global_precipitation_kg / climate.metrics.global_evaporation_kg
    } else {
        0.0
    };
    let (no_orography_mean_annual_precipitation_mm, orographic_precipitation_causal_fraction) =
        if let Some(no_orography) = no_orography_climate {
            let baseline = no_orography.metrics.mean_annual_precipitation_mm;
            let total = climate.metrics.mean_annual_precipitation_mm;
            let fraction = if total.abs() > 1.0e-12 { (total - baseline) / total } else { 0.0 };
            (Some(baseline), Some(fraction))
        } else {
            (None, None)
        };
    let mean_snowfall_mm = weighted_mean(
        topology,
        (0..count).map(|index| {
            (
                index,
                f64::from(climate.annual_precipitation_mm[index])
                    * f64::from(climate.snowfall_fraction[index]),
            )
        }),
    );
    let annual_mean_ocean_heat_tendency_rms_index = {
        let weighted_square = weighted_mean(
            topology,
            ocean_indices.iter().copied().map(|index| {
                let value = f64::from(climate.ocean_heat_transport_index[index]);
                (index, value * value)
            }),
        );
        weighted_square.max(0.0).sqrt()
    };

    Ok(ClimateCalibrationReport {
        sample_count: climate.metrics.sample_count,
        orbital_phase_count: climate.metrics.orbital_phase_count,
        mean_temperature_k: climate.metrics.mean_temperature_k,
        mean_land_temperature_k: climate.metrics.mean_land_temperature_k,
        mean_ocean_temperature_k: climate.metrics.mean_ocean_temperature_k,
        land_ocean_temperature_contrast_k: climate.metrics.mean_ocean_temperature_k - climate.metrics.mean_land_temperature_k,
        mean_sea_surface_temperature_k: climate.metrics.mean_sea_surface_temperature_k,
        mean_land_elevation_m,
        median_land_elevation_m,
        p95_land_elevation_m,
        land_area_above_2km_fraction,
        land_area_above_4km_fraction,
        clear_surface_absorbed_shortwave_w_m2,
        clear_surface_absorbed_shortwave_land_w_m2,
        clear_surface_absorbed_shortwave_ocean_w_m2,
        outgoing_longwave_proxy_w_m2,
        outgoing_longwave_land_proxy_w_m2,
        outgoing_longwave_ocean_proxy_w_m2,
        toa_energy_imbalance_proxy_w_m2,
        reconstructed_wind_cap_fraction: if wind_samples > 0 { wind_cap_samples as f64 / wind_samples as f64 } else { 0.0 },
        reconstructed_moisture_edge_cap_fraction: if moisture_edges > 0 { moisture_cap_edges as f64 / moisture_edges as f64 } else { 0.0 },
        mean_state_relative_humidity_p05: weighted_percentile(rh_values.clone(), 0.05),
        mean_state_relative_humidity_p50: weighted_percentile(rh_values.clone(), 0.50),
        mean_state_relative_humidity_p95: weighted_percentile(rh_values, 0.95),
        mean_annual_precipitation_mm: climate.metrics.mean_annual_precipitation_mm,
        p95_annual_precipitation_mm: climate.metrics.p95_annual_precipitation_mm,
        mean_potential_evaporation_mm,
        precipitation_to_evaporation_ratio,
        no_orography_mean_annual_precipitation_mm,
        orographic_precipitation_causal_fraction,
        mean_snowfall_mm,
        persistent_snow_area_fraction: climate.metrics.persistent_snow_area_fraction,
        sea_ice_area_fraction: climate.metrics.sea_ice_area_fraction,
        annual_mean_ocean_heat_tendency_rms_index,
        latitude_bands: latitude_bands(topology, terrain, climate),
    })
}
