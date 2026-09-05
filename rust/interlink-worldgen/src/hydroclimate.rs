use crate::{ClimateState, GeodesicTopology, TopographyState, WorldgenError};

const LATITUDE_BAND_DEGREES: i32 = 15;

#[derive(Clone, Debug, PartialEq)]
pub struct HydroclimateLatitudeBand {
    pub minimum_latitude_deg: f64,
    pub maximum_latitude_deg: f64,
    pub area_fraction: f64,
    pub land_area_fraction: f64,
    pub mean_precipitation_mm: f64,
    pub mean_land_precipitation_mm: Option<f64>,
    pub mean_land_potential_evaporation_mm: Option<f64>,
    pub mean_land_aridity_index: Option<f64>,
    pub mean_land_precipitation_seasonality: Option<f64>,
    pub mean_snowfall_mm: f64,
    pub mean_persistent_snow_potential: f64,
    pub mean_sea_ice_potential: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HydroclimateClosureReport {
    pub sample_count: u32,
    pub mean_land_precipitation_mm: f64,
    pub land_precipitation_p05_mm: f64,
    pub land_precipitation_p50_mm: f64,
    pub land_precipitation_p95_mm: f64,
    pub land_precipitation_spatial_cv: f64,
    pub land_precipitation_seasonality_p50: f64,
    pub land_precipitation_seasonality_p95: f64,
    pub mean_land_potential_evaporation_mm: f64,
    pub land_potential_evaporation_p95_mm: f64,
    pub land_aridity_index_p05: f64,
    pub land_aridity_index_p50: f64,
    pub land_aridity_index_p95: f64,
    pub land_aridity_below_0_2_fraction: f64,
    pub land_aridity_below_0_5_fraction: f64,
    pub land_aridity_at_least_1_fraction: f64,
    pub tropical_land_precipitation_mm: Option<f64>,
    pub subtropical_land_precipitation_mm: Option<f64>,
    pub midlatitude_land_precipitation_mm: Option<f64>,
    pub polar_land_precipitation_mm: Option<f64>,
    pub tropical_to_subtropical_land_precipitation_ratio: Option<f64>,
    pub mean_land_snowfall_mm: f64,
    pub persistent_snow_land_area_fraction: f64,
    pub sea_ice_ocean_area_fraction: f64,
    pub no_orography_land_precipitation_rms_difference_mm: Option<f64>,
    pub no_orography_land_precipitation_rms_fraction_of_mean: Option<f64>,
    pub latitude_bands: Vec<HydroclimateLatitudeBand>,
}

fn latitude_deg(topology: &GeodesicTopology, index: usize) -> f64 {
    topology.positions()[index][2]
        .clamp(-1.0, 1.0)
        .asin()
        .to_degrees()
}

fn weighted_mean(
    topology: &GeodesicTopology,
    values: impl Iterator<Item = (usize, f64)>,
) -> Option<f64> {
    let mut weighted = 0.0;
    let mut area = 0.0;
    for (index, value) in values {
        let sample_area = topology.dual_area_steradians()[index];
        weighted += value * sample_area;
        area += sample_area;
    }
    if area > 0.0 {
        Some(weighted / area)
    } else {
        None
    }
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
    if total > 0.0 {
        matching / total
    } else {
        0.0
    }
}

fn weighted_percentile(mut values: Vec<(f64, f64)>, fraction: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.0.total_cmp(&b.0));
    let total_weight = values
        .iter()
        .map(|(_, weight)| *weight)
        .sum::<f64>()
        .max(1.0e-18);
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

fn weighted_spatial_cv(
    topology: &GeodesicTopology,
    indices: &[usize],
    values: &[f32],
) -> f64 {
    let mean = weighted_mean(
        topology,
        indices
            .iter()
            .copied()
            .map(|index| (index, f64::from(values[index]))),
    )
    .unwrap_or(0.0);
    if mean.abs() <= 1.0e-12 {
        return 0.0;
    }
    let variance = weighted_mean(
        topology,
        indices.iter().copied().map(|index| {
            let delta = f64::from(values[index]) - mean;
            (index, delta * delta)
        }),
    )
    .unwrap_or(0.0);
    variance.max(0.0).sqrt() / mean.abs()
}

fn latitude_band_report(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    climate: &ClimateState,
) -> Vec<HydroclimateLatitudeBand> {
    let mut result = Vec::new();
    let total_area = topology.metrics().total_area_steradians.max(1.0e-18);
    for minimum in (-90..90).step_by(LATITUDE_BAND_DEGREES as usize) {
        let maximum = minimum + LATITUDE_BAND_DEGREES;
        let indices = (0..climate.annual_precipitation_mm.len())
            .filter(|index| {
                let latitude = latitude_deg(topology, *index);
                latitude >= f64::from(minimum)
                    && (latitude < f64::from(maximum) || maximum == 90 && latitude <= 90.0)
            })
            .collect::<Vec<_>>();
        let land_indices = indices
            .iter()
            .copied()
            .filter(|index| terrain.submerged_mask[*index] == 0)
            .collect::<Vec<_>>();
        let band_area = indices
            .iter()
            .map(|index| topology.dual_area_steradians()[*index])
            .sum::<f64>();
        let land_area = land_indices
            .iter()
            .map(|index| topology.dual_area_steradians()[*index])
            .sum::<f64>();
        let mean_precipitation_mm = weighted_mean(
            topology,
            indices
                .iter()
                .copied()
                .map(|index| (index, f64::from(climate.annual_precipitation_mm[index]))),
        )
        .unwrap_or(0.0);
        let mean_land_precipitation_mm = weighted_mean(
            topology,
            land_indices
                .iter()
                .copied()
                .map(|index| (index, f64::from(climate.annual_precipitation_mm[index]))),
        );
        let mean_land_potential_evaporation_mm = weighted_mean(
            topology,
            land_indices
                .iter()
                .copied()
                .map(|index| (index, f64::from(climate.potential_evaporation_mm[index]))),
        );
        let mean_land_aridity_index = weighted_mean(
            topology,
            land_indices
                .iter()
                .copied()
                .map(|index| (index, f64::from(climate.aridity_index[index]))),
        );
        let mean_land_precipitation_seasonality = weighted_mean(
            topology,
            land_indices
                .iter()
                .copied()
                .map(|index| (index, f64::from(climate.precipitation_seasonality[index]))),
        );
        let mean_snowfall_mm = weighted_mean(
            topology,
            indices.iter().copied().map(|index| {
                (
                    index,
                    f64::from(climate.annual_precipitation_mm[index])
                        * f64::from(climate.snowfall_fraction[index]),
                )
            }),
        )
        .unwrap_or(0.0);
        let mean_persistent_snow_potential = weighted_mean(
            topology,
            indices
                .iter()
                .copied()
                .map(|index| (index, f64::from(climate.persistent_snow_potential[index]))),
        )
        .unwrap_or(0.0);
        let mean_sea_ice_potential = weighted_mean(
            topology,
            indices
                .iter()
                .copied()
                .map(|index| (index, f64::from(climate.sea_ice_potential[index]))),
        )
        .unwrap_or(0.0);
        result.push(HydroclimateLatitudeBand {
            minimum_latitude_deg: f64::from(minimum),
            maximum_latitude_deg: f64::from(maximum),
            area_fraction: band_area / total_area,
            land_area_fraction: if band_area > 0.0 {
                land_area / band_area
            } else {
                0.0
            },
            mean_precipitation_mm,
            mean_land_precipitation_mm,
            mean_land_potential_evaporation_mm,
            mean_land_aridity_index,
            mean_land_precipitation_seasonality,
            mean_snowfall_mm,
            mean_persistent_snow_potential,
            mean_sea_ice_potential,
        });
    }
    result
}

pub fn build_hydroclimate_closure_report(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    climate: &ClimateState,
    no_orography_climate: Option<&ClimateState>,
) -> Result<HydroclimateClosureReport, WorldgenError> {
    let count = topology.metrics().sample_count as usize;
    let required_lengths = [
        climate.annual_precipitation_mm.len(),
        climate.precipitation_seasonality.len(),
        climate.potential_evaporation_mm.len(),
        climate.aridity_index.len(),
        climate.snowfall_fraction.len(),
        climate.persistent_snow_potential.len(),
        climate.sea_ice_potential.len(),
        terrain.submerged_mask.len(),
    ];
    if required_lengths.iter().any(|length| *length != count) {
        return Err(WorldgenError::InvalidClimate(
            "WG-5 hydroclimate closure inputs must match topology sample count",
        ));
    }
    if let Some(no_orography) = no_orography_climate {
        if no_orography.annual_precipitation_mm.len() != count {
            return Err(WorldgenError::InvalidClimate(
                "WG-5 no-orography climate must match topology sample count",
            ));
        }
    }

    let land_indices = (0..count)
        .filter(|index| terrain.submerged_mask[*index] == 0)
        .collect::<Vec<_>>();
    let ocean_indices = (0..count)
        .filter(|index| terrain.submerged_mask[*index] != 0)
        .collect::<Vec<_>>();
    if land_indices.is_empty() {
        return Err(WorldgenError::InvalidClimate(
            "WG-5 hydroclimate closure requires at least one land sample",
        ));
    }

    let precipitation_values = land_indices
        .iter()
        .map(|index| {
            (
                f64::from(climate.annual_precipitation_mm[*index]),
                topology.dual_area_steradians()[*index],
            )
        })
        .collect::<Vec<_>>();
    let seasonality_values = land_indices
        .iter()
        .map(|index| {
            (
                f64::from(climate.precipitation_seasonality[*index]),
                topology.dual_area_steradians()[*index],
            )
        })
        .collect::<Vec<_>>();
    let potential_evaporation_values = land_indices
        .iter()
        .map(|index| {
            (
                f64::from(climate.potential_evaporation_mm[*index]),
                topology.dual_area_steradians()[*index],
            )
        })
        .collect::<Vec<_>>();
    let aridity_values = land_indices
        .iter()
        .map(|index| {
            (
                f64::from(climate.aridity_index[*index]),
                topology.dual_area_steradians()[*index],
            )
        })
        .collect::<Vec<_>>();

    let mean_land_precipitation_mm = weighted_mean(
        topology,
        land_indices
            .iter()
            .copied()
            .map(|index| (index, f64::from(climate.annual_precipitation_mm[index]))),
    )
    .unwrap_or(0.0);
    let mean_land_potential_evaporation_mm = weighted_mean(
        topology,
        land_indices
            .iter()
            .copied()
            .map(|index| (index, f64::from(climate.potential_evaporation_mm[index]))),
    )
    .unwrap_or(0.0);
    let mean_land_snowfall_mm = weighted_mean(
        topology,
        land_indices.iter().copied().map(|index| {
            (
                index,
                f64::from(climate.annual_precipitation_mm[index])
                    * f64::from(climate.snowfall_fraction[index]),
            )
        }),
    )
    .unwrap_or(0.0);

    let latitude_mean = |minimum_abs_deg: f64, maximum_abs_deg: f64| {
        weighted_mean(
            topology,
            land_indices.iter().copied().filter_map(|index| {
                let absolute_latitude = latitude_deg(topology, index).abs();
                if absolute_latitude >= minimum_abs_deg && absolute_latitude < maximum_abs_deg {
                    Some((index, f64::from(climate.annual_precipitation_mm[index])))
                } else {
                    None
                }
            }),
        )
    };
    let tropical_land_precipitation_mm = latitude_mean(0.0, 15.0);
    let subtropical_land_precipitation_mm = latitude_mean(15.0, 35.0);
    let midlatitude_land_precipitation_mm = latitude_mean(35.0, 60.0);
    let polar_land_precipitation_mm = latitude_mean(60.0, 90.000_001);
    let tropical_to_subtropical_land_precipitation_ratio =
        match (tropical_land_precipitation_mm, subtropical_land_precipitation_mm) {
            (Some(tropical), Some(subtropical)) if subtropical > 1.0e-12 => {
                Some(tropical / subtropical)
            }
            _ => None,
        };

    let (no_orography_land_precipitation_rms_difference_mm, no_orography_land_precipitation_rms_fraction_of_mean) =
        if let Some(no_orography) = no_orography_climate {
            let rms = weighted_mean(
                topology,
                land_indices.iter().copied().map(|index| {
                    let delta = f64::from(climate.annual_precipitation_mm[index])
                        - f64::from(no_orography.annual_precipitation_mm[index]);
                    (index, delta * delta)
                }),
            )
            .unwrap_or(0.0)
            .max(0.0)
            .sqrt();
            let fraction = if mean_land_precipitation_mm > 1.0e-12 {
                rms / mean_land_precipitation_mm
            } else {
                0.0
            };
            (Some(rms), Some(fraction))
        } else {
            (None, None)
        };

    Ok(HydroclimateClosureReport {
        sample_count: climate.metrics.sample_count,
        mean_land_precipitation_mm,
        land_precipitation_p05_mm: weighted_percentile(precipitation_values.clone(), 0.05),
        land_precipitation_p50_mm: weighted_percentile(precipitation_values.clone(), 0.50),
        land_precipitation_p95_mm: weighted_percentile(precipitation_values, 0.95),
        land_precipitation_spatial_cv: weighted_spatial_cv(
            topology,
            &land_indices,
            &climate.annual_precipitation_mm,
        ),
        land_precipitation_seasonality_p50: weighted_percentile(seasonality_values.clone(), 0.50),
        land_precipitation_seasonality_p95: weighted_percentile(seasonality_values, 0.95),
        mean_land_potential_evaporation_mm,
        land_potential_evaporation_p95_mm: weighted_percentile(potential_evaporation_values, 0.95),
        land_aridity_index_p05: weighted_percentile(aridity_values.clone(), 0.05),
        land_aridity_index_p50: weighted_percentile(aridity_values.clone(), 0.50),
        land_aridity_index_p95: weighted_percentile(aridity_values, 0.95),
        land_aridity_below_0_2_fraction: weighted_fraction(
            topology,
            land_indices.iter().copied(),
            |index| climate.aridity_index[index] < 0.2,
        ),
        land_aridity_below_0_5_fraction: weighted_fraction(
            topology,
            land_indices.iter().copied(),
            |index| climate.aridity_index[index] < 0.5,
        ),
        land_aridity_at_least_1_fraction: weighted_fraction(
            topology,
            land_indices.iter().copied(),
            |index| climate.aridity_index[index] >= 1.0,
        ),
        tropical_land_precipitation_mm,
        subtropical_land_precipitation_mm,
        midlatitude_land_precipitation_mm,
        polar_land_precipitation_mm,
        tropical_to_subtropical_land_precipitation_ratio,
        mean_land_snowfall_mm,
        persistent_snow_land_area_fraction: weighted_fraction(
            topology,
            land_indices.iter().copied(),
            |index| climate.persistent_snow_potential[index] >= 0.5,
        ),
        sea_ice_ocean_area_fraction: weighted_fraction(
            topology,
            ocean_indices.iter().copied(),
            |index| climate.sea_ice_potential[index] >= 0.5,
        ),
        no_orography_land_precipitation_rms_difference_mm,
        no_orography_land_precipitation_rms_fraction_of_mean,
        latitude_bands: latitude_band_report(topology, terrain, climate),
    })
}
