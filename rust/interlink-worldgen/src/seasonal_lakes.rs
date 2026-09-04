use crate::{
    ClimateGenerationDiagnostics, ClimateState, DrainageState, GeodesicTopology, LakeState,
    PlanetPhysicalParameters, PlanetTopology, TopographyState, INVALID_SAMPLE_ID,
};
use std::collections::BTreeSet;

const MM_TO_M: f64 = 1.0e-3;
const FLOW_EPSILON_M3_S: f64 = 1.0e-12;
const LAKE_CYCLE_CONVERGENCE_RELATIVE: f64 = 1.0e-6;
const MINIMUM_LAKE_SPINUP_YEARS: u8 = 2;
const MAXIMUM_LAKE_SPINUP_YEARS: u8 = 12;
const EVAPORATION_WEIGHT_FLOOR_K: f64 = 250.0;

#[derive(Debug)]
pub(crate) struct SeasonalLakeRoutingResult {
    pub phase_realized_discharge_m3_s: Vec<f32>,
    /// Phase-major by active WG-6C lake record.
    pub phase_lake_surface_elevation_m: Vec<f32>,
    /// Phase-major by active WG-6C lake record.
    pub phase_lake_area_m2: Vec<f64>,
    /// Phase-major by active WG-6C lake record.
    pub phase_lake_volume_m3: Vec<f64>,
    pub annual_mean_terminal_realized_discharge_m3_s: f64,
    pub maximum_phase_realized_discharge_m3_s: f64,
    pub annual_mean_lake_precipitation_m3_s: f64,
    pub annual_mean_lake_evaporation_m3_s: f64,
    pub annual_mean_unreleased_terminal_storage_m3_s: f64,
    pub water_balance_relative_error: f64,
    pub lake_spinup_years: u8,
    pub final_lake_cycle_relative_change: f64,
    pub maximum_seasonal_lake_level_range_m: f64,
}

#[derive(Debug)]
struct ActiveLakeGeometry {
    members: Vec<usize>,
    sorted_members: Vec<usize>,
    spill_elevation_m: f64,
    spill_receiver: u32,
    maximum_volume_m3: f64,
    evaporation_scale: f64,
}

fn reconstructed_temperature(mean: f32, cosine: f32, sine: f32, angle: f64) -> f64 {
    f64::from(mean) + f64::from(cosine) * angle.cos() + f64::from(sine) * angle.sin()
}

fn evaporation_weight(temperature_k: f64) -> f64 {
    (temperature_k - EVAPORATION_WEIGHT_FLOOR_K).max(1.0)
}

fn geometry_at_surface(
    lake: &ActiveLakeGeometry,
    elevation_m: &[f32],
    area_m2: &[f64],
    surface_elevation_m: f64,
    mut fractions: Option<&mut [f64]>,
) -> (f64, f64) {
    let surface = surface_elevation_m.min(lake.spill_elevation_m);
    let mut area = 0.0_f64;
    let mut volume = 0.0_f64;
    for (rank, &sample) in lake.sorted_members.iter().enumerate() {
        let lower = f64::from(elevation_m[sample]);
        let upper = lake
            .sorted_members
            .get(rank + 1)
            .map(|next| f64::from(elevation_m[*next]))
            .unwrap_or(lake.spill_elevation_m)
            .max(lower)
            .min(lake.spill_elevation_m);
        let fraction = if surface <= lower {
            0.0
        } else if upper <= lower || surface >= upper {
            1.0
        } else {
            ((surface - lower) / (upper - lower)).clamp(0.0, 1.0)
        };
        let depth = (surface - lower).max(0.0);
        if let Some(output) = fractions.as_deref_mut() {
            output[sample] = fraction;
        }
        area += area_m2[sample] * fraction;
        volume += area_m2[sample] * fraction * depth;
    }
    (area, volume)
}

fn surface_for_volume(
    lake: &ActiveLakeGeometry,
    elevation_m: &[f32],
    area_m2: &[f64],
    volume_m3: f64,
) -> f64 {
    if lake.sorted_members.is_empty() || volume_m3 <= 0.0 {
        return lake
            .sorted_members
            .first()
            .map(|sample| f64::from(elevation_m[*sample]))
            .unwrap_or(lake.spill_elevation_m);
    }
    if volume_m3 >= lake.maximum_volume_m3 {
        return lake.spill_elevation_m;
    }
    let mut lower = f64::from(elevation_m[lake.sorted_members[0]]);
    let mut upper = lake.spill_elevation_m;
    for _ in 0..48 {
        let middle = 0.5 * (lower + upper);
        let (_, volume) = geometry_at_surface(lake, elevation_m, area_m2, middle, None);
        if volume < volume_m3 {
            lower = middle;
        } else {
            upper = middle;
        }
    }
    0.5 * (lower + upper)
}

fn fill_lake_fractions(
    lakes: &[ActiveLakeGeometry],
    elevation_m: &[f32],
    area_m2: &[f64],
    volumes_m3: &[f64],
    fractions: &mut [f64],
) {
    fractions.fill(0.0);
    for (lake_index, lake) in lakes.iter().enumerate() {
        let surface = surface_for_volume(lake, elevation_m, area_m2, volumes_m3[lake_index]);
        geometry_at_surface(lake, elevation_m, area_m2, surface, Some(fractions));
    }
}

fn find_downstream_active_lake(
    start: u32,
    active_lake_for_sample: &[u32],
    submerged_mask: &[u8],
    receiver: &[u32],
) -> Result<u32, &'static str> {
    if start == INVALID_SAMPLE_ID {
        return Ok(INVALID_SAMPLE_ID);
    }
    let count = receiver.len();
    let mut sample = start;
    for _ in 0..=count {
        let i = sample as usize;
        if i >= count {
            return Err("seasonal lake spill path leaves the canonical topology");
        }
        let active = active_lake_for_sample[i];
        if active != INVALID_SAMPLE_ID {
            return Ok(active);
        }
        if submerged_mask[i] != 0 || receiver[i] == INVALID_SAMPLE_ID {
            return Ok(INVALID_SAMPLE_ID);
        }
        sample = receiver[i];
    }
    Err("seasonal lake spill path must remain acyclic")
}

#[allow(clippy::too_many_arguments)]
fn build_active_lakes(
    topography: &TopographyState,
    climate: &ClimateState,
    drainage: &DrainageState,
    lake_state: &LakeState,
    area_m2: &[f64],
    year_seconds: f64,
) -> Result<(Vec<ActiveLakeGeometry>, Vec<u32>, Vec<usize>), &'static str> {
    let count = topography.metrics.sample_count as usize;
    let mut lake_by_depression = vec![INVALID_SAMPLE_ID; drainage.depressions.len()];
    for (lake_index, record) in lake_state.lakes.iter().enumerate() {
        let depression = record.depression_id as usize;
        if depression >= drainage.depressions.len() || lake_by_depression[depression] != INVALID_SAMPLE_ID
        {
            return Err("WG-6D requires unique WG-6C lake records on known depressions");
        }
        lake_by_depression[depression] = lake_index as u32;
    }

    let mut active_lake_for_sample = vec![INVALID_SAMPLE_ID; count];
    let mut members = vec![Vec::<usize>::new(); lake_state.lakes.len()];
    for sample in 0..count {
        let depression = drainage.depression_id[sample];
        if depression == INVALID_SAMPLE_ID {
            continue;
        }
        let lake_index = lake_by_depression[depression as usize];
        if lake_index != INVALID_SAMPLE_ID {
            active_lake_for_sample[sample] = lake_index;
            members[lake_index as usize].push(sample);
        }
    }

    let mut active_lakes = Vec::with_capacity(lake_state.lakes.len());
    for (lake_index, record) in lake_state.lakes.iter().enumerate() {
        let depression = &drainage.depressions[record.depression_id as usize];
        let mut sorted_members = members[lake_index].clone();
        sorted_members.sort_by(|a, b| {
            f64::from(topography.solid_elevation_m[*a])
                .total_cmp(&f64::from(topography.solid_elevation_m[*b]))
                .then_with(|| a.cmp(b))
        });
        if sorted_members.is_empty() {
            return Err("WG-6D active lake must retain WG-6A depression membership");
        }
        let mut geometry = ActiveLakeGeometry {
            members: members[lake_index].clone(),
            sorted_members,
            spill_elevation_m: depression.spill_elevation_m,
            spill_receiver: record.spill_receiver,
            maximum_volume_m3: 0.0,
            evaporation_scale: 1.0,
        };
        geometry.maximum_volume_m3 = geometry_at_surface(
            &geometry,
            &topography.solid_elevation_m,
            area_m2,
            geometry.spill_elevation_m,
            None,
        )
        .1;

        let mut unscaled_evaporation_m3_s = 0.0_f64;
        for &sample in &geometry.members {
            let fraction = f64::from(lake_state.lake_fraction[sample]);
            if fraction <= 0.0 {
                continue;
            }
            unscaled_evaporation_m3_s += f64::from(climate.potential_evaporation_mm[sample])
                * MM_TO_M
                * area_m2[sample]
                * fraction
                / year_seconds;
        }
        geometry.evaporation_scale = if unscaled_evaporation_m3_s > FLOW_EPSILON_M3_S {
            (record.lake_evaporation_m3_s / unscaled_evaporation_m3_s).max(0.0)
        } else {
            1.0
        };
        active_lakes.push(geometry);
    }

    let mut downstream_lake = vec![INVALID_SAMPLE_ID; active_lakes.len()];
    for (lake_index, geometry) in active_lakes.iter().enumerate() {
        let target = find_downstream_active_lake(
            geometry.spill_receiver,
            &active_lake_for_sample,
            &topography.submerged_mask,
            &drainage.receiver,
        )?;
        if target == lake_index as u32 {
            return Err("WG-6D active lake spill graph cannot target itself");
        }
        downstream_lake[lake_index] = target;
    }

    let mut indegree = vec![0_u32; active_lakes.len()];
    for &target in &downstream_lake {
        if target != INVALID_SAMPLE_ID {
            indegree[target as usize] += 1;
        }
    }
    let mut ready = BTreeSet::new();
    for (lake_index, &degree) in indegree.iter().enumerate() {
        if degree == 0 {
            ready.insert(lake_index);
        }
    }
    let mut lake_order = Vec::with_capacity(active_lakes.len());
    while let Some(&lake_index) = ready.iter().next() {
        ready.remove(&lake_index);
        lake_order.push(lake_index);
        let target = downstream_lake[lake_index];
        if target != INVALID_SAMPLE_ID {
            let degree = &mut indegree[target as usize];
            *degree -= 1;
            if *degree == 0 {
                ready.insert(target as usize);
            }
        }
    }
    if lake_order.len() != active_lakes.len() {
        return Err("WG-6D active lake spill graph must be acyclic");
    }

    Ok((active_lakes, active_lake_for_sample, lake_order))
}

#[allow(clippy::too_many_arguments)]
fn route_lake_outflow(
    start: u32,
    flow_m3_s: f64,
    active_lake_for_sample: &[u32],
    submerged_mask: &[u8],
    receiver: &[u32],
    realized_accum_m3_s: &mut [f64],
    lake_inflow_m3_s: &mut [f64],
    terminal_m3_s: &mut f64,
) -> Result<(), &'static str> {
    if flow_m3_s <= FLOW_EPSILON_M3_S {
        return Ok(());
    }
    if start == INVALID_SAMPLE_ID {
        return Err("seasonal overflowing lake requires a real spill receiver");
    }
    let count = receiver.len();
    let mut sample = start;
    for _ in 0..=count {
        let i = sample as usize;
        if i >= count {
            return Err("seasonal lake outflow leaves the canonical topology");
        }
        realized_accum_m3_s[i] += flow_m3_s;
        let active = active_lake_for_sample[i];
        if active != INVALID_SAMPLE_ID {
            lake_inflow_m3_s[active as usize] += flow_m3_s;
            return Ok(());
        }
        if submerged_mask[i] != 0 || receiver[i] == INVALID_SAMPLE_ID {
            *terminal_m3_s += flow_m3_s;
            return Ok(());
        }
        sample = receiver[i];
    }
    Err("seasonal lake outflow routing must remain acyclic")
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn solve_seasonal_lake_routing(
    topology: &GeodesicTopology,
    topography: &TopographyState,
    climate: &ClimateState,
    climate_diagnostics: &ClimateGenerationDiagnostics,
    drainage: &DrainageState,
    lake_state: &LakeState,
    planet: PlanetPhysicalParameters,
    phase_local_runoff_m3_s: &[f32],
) -> Result<SeasonalLakeRoutingResult, &'static str> {
    let count = topology.sample_count() as usize;
    let phase_count = usize::from(climate.metrics.orbital_phase_count);
    if phase_count == 0
        || phase_local_runoff_m3_s.len() != count * phase_count
        || climate_diagnostics.precipitation_phase_rate_mm_year.len() != count * phase_count
        || drainage.receiver.len() != count
        || drainage.depression_id.len() != count
        || lake_state.lake_fraction.len() != count
    {
        return Err("WG-6D seasonal lake routing inputs must align with phase/topology dimensions");
    }

    let mut area_m2 = vec![0.0_f64; count];
    for i in 0..count {
        if topography.submerged_mask[i] == 0 {
            area_m2[i] = topology.area_steradians(i as u32) * planet.radius_m * planet.radius_m;
            if !area_m2[i].is_finite() || area_m2[i] <= 0.0 {
                return Err("WG-6D land cell area must be finite and positive");
            }
        }
    }

    let (active_lakes, active_lake_for_sample, lake_order) = build_active_lakes(
        topography,
        climate,
        drainage,
        lake_state,
        &area_m2,
        planet.orbital_period_s,
    )?;
    let lake_count = active_lakes.len();
    let total_phase_samples = count * phase_count;
    let total_phase_lakes = lake_count * phase_count;
    let mut phase_realized_discharge_m3_s = vec![0.0_f32; total_phase_samples];
    let mut phase_lake_surface_elevation_m = vec![0.0_f32; total_phase_lakes];
    let mut phase_lake_area_m2 = vec![0.0_f64; total_phase_lakes];
    let mut phase_lake_volume_m3 = vec![0.0_f64; total_phase_lakes];

    let mut current_volume_m3 = vec![0.0_f64; lake_count];
    for (lake_index, geometry) in active_lakes.iter().enumerate() {
        let annual_surface = lake_state.lakes[lake_index]
            .surface_elevation_m
            .min(geometry.spill_elevation_m);
        current_volume_m3[lake_index] = geometry_at_surface(
            geometry,
            &topography.solid_elevation_m,
            &area_m2,
            annual_surface,
            None,
        )
        .1
        .clamp(0.0, geometry.maximum_volume_m3);
    }

    let mut pet_weight_mean = vec![1.0_f64; count];
    for geometry in &active_lakes {
        for &sample in &geometry.members {
            let mut weight_sum = 0.0_f64;
            for phase in 0..phase_count {
                let angle = std::f64::consts::TAU * phase as f64 / phase_count as f64;
                let temperature = reconstructed_temperature(
                    climate.temperature_mean_k[sample],
                    climate.temperature_annual_cos_k[sample],
                    climate.temperature_annual_sin_k[sample],
                    angle,
                );
                weight_sum += evaporation_weight(temperature);
            }
            pet_weight_mean[sample] = (weight_sum / phase_count as f64).max(1.0e-12);
        }
    }

    let phase_seconds = planet.orbital_period_s / phase_count as f64;
    let mut lake_fraction = vec![0.0_f64; count];
    let mut realized_accum_m3_s = vec![0.0_f64; count];
    let mut lake_inflow_m3_s = vec![0.0_f64; lake_count];

    let maximum_spinup_years = if lake_count == 0 {
        1
    } else {
        MAXIMUM_LAKE_SPINUP_YEARS
    };
    let mut completed_years = 0_u8;
    let mut final_cycle_relative_change = 0.0_f64;
    let mut final_terminal_volume_m3 = 0.0_f64;
    let mut final_lake_precipitation_volume_m3 = 0.0_f64;
    let mut final_lake_evaporation_volume_m3 = 0.0_f64;
    let mut final_terminal_storage_volume_m3 = 0.0_f64;
    let mut final_input_volume_m3 = 0.0_f64;
    let mut final_start_storage_m3 = 0.0_f64;
    let mut final_end_storage_m3 = 0.0_f64;
    let mut final_maximum_realized_m3_s = 0.0_f64;
    let mut final_maximum_lake_level_range_m = 0.0_f64;

    for year in 0..maximum_spinup_years {
        let year_start_volume_m3 = current_volume_m3.clone();
        let mut input_volume_m3 = 0.0_f64;
        let mut terminal_volume_m3 = 0.0_f64;
        let mut lake_precipitation_volume_m3 = 0.0_f64;
        let mut lake_evaporation_volume_m3 = 0.0_f64;
        let mut terminal_storage_volume_m3 = 0.0_f64;
        let mut maximum_realized_m3_s = 0.0_f64;
        let mut minimum_lake_surface_m = vec![f64::INFINITY; lake_count];
        let mut maximum_lake_surface_m = vec![f64::NEG_INFINITY; lake_count];

        for phase in 0..phase_count {
            fill_lake_fractions(
                &active_lakes,
                &topography.solid_elevation_m,
                &area_m2,
                &current_volume_m3,
                &mut lake_fraction,
            );

            realized_accum_m3_s.fill(0.0);
            lake_inflow_m3_s.fill(0.0);
            let phase_start = phase * count;
            let mut dry_local_rate_m3_s = 0.0_f64;
            for i in 0..count {
                if topography.submerged_mask[i] != 0 {
                    continue;
                }
                let local = f64::from(phase_local_runoff_m3_s[phase_start + i]);
                if !local.is_finite() || local < 0.0 {
                    return Err("WG-6D phase local runoff must be finite and non-negative");
                }
                let dry_fraction = (1.0 - lake_fraction[i]).clamp(0.0, 1.0);
                let dry_local = local * dry_fraction;
                realized_accum_m3_s[i] = dry_local;
                dry_local_rate_m3_s += dry_local;
            }
            input_volume_m3 += dry_local_rate_m3_s * phase_seconds;

            for &sample in &drainage.drainage_order {
                let i = sample as usize;
                if active_lake_for_sample[i] != INVALID_SAMPLE_ID {
                    continue;
                }
                let downstream = drainage.receiver[i];
                if downstream != INVALID_SAMPLE_ID {
                    realized_accum_m3_s[downstream as usize] += realized_accum_m3_s[i];
                }
            }

            for i in 0..count {
                let lake_index = active_lake_for_sample[i];
                if lake_index != INVALID_SAMPLE_ID {
                    lake_inflow_m3_s[lake_index as usize] += realized_accum_m3_s[i];
                }
            }

            let mut terminal_rate_m3_s = 0.0_f64;
            for i in 0..count {
                let active_lake = active_lake_for_sample[i] != INVALID_SAMPLE_ID;
                if topography.submerged_mask[i] != 0
                    || (topography.submerged_mask[i] == 0
                        && drainage.receiver[i] == INVALID_SAMPLE_ID
                        && !active_lake)
                {
                    terminal_rate_m3_s += realized_accum_m3_s[i];
                }
            }

            for &lake_index in &lake_order {
                let geometry = &active_lakes[lake_index];
                let mut precipitation_rate_m3_s = 0.0_f64;
                let mut evaporation_rate_m3_s = 0.0_f64;
                let angle = std::f64::consts::TAU * phase as f64 / phase_count as f64;
                for &sample in &geometry.members {
                    let fraction = lake_fraction[sample];
                    if fraction <= 0.0 {
                        continue;
                    }
                    let precipitation_rate_mm_year = f64::from(
                        climate_diagnostics.precipitation_phase_rate_mm_year
                            [phase_start + sample],
                    );
                    precipitation_rate_m3_s += precipitation_rate_mm_year
                        * MM_TO_M
                        * area_m2[sample]
                        * fraction
                        / planet.orbital_period_s;

                    let temperature = reconstructed_temperature(
                        climate.temperature_mean_k[sample],
                        climate.temperature_annual_cos_k[sample],
                        climate.temperature_annual_sin_k[sample],
                        angle,
                    );
                    let pet_scale = evaporation_weight(temperature) / pet_weight_mean[sample];
                    let pet_rate_mm_year = f64::from(climate.potential_evaporation_mm[sample])
                        * pet_scale
                        * geometry.evaporation_scale;
                    evaporation_rate_m3_s += pet_rate_mm_year
                        * MM_TO_M
                        * area_m2[sample]
                        * fraction
                        / planet.orbital_period_s;
                }

                let precipitation_volume = precipitation_rate_m3_s * phase_seconds;
                input_volume_m3 += precipitation_volume;
                lake_precipitation_volume_m3 += precipitation_volume;

                let inflow_volume = lake_inflow_m3_s[lake_index] * phase_seconds;
                let available_volume = current_volume_m3[lake_index]
                    + inflow_volume
                    + precipitation_volume;
                let requested_evaporation_volume = evaporation_rate_m3_s * phase_seconds;
                let actual_evaporation_volume = requested_evaporation_volume.min(available_volume);
                lake_evaporation_volume_m3 += actual_evaporation_volume;
                let mut retained_volume = (available_volume - actual_evaporation_volume).max(0.0);

                if retained_volume > geometry.maximum_volume_m3 {
                    let excess_volume = retained_volume - geometry.maximum_volume_m3;
                    retained_volume = geometry.maximum_volume_m3;
                    if geometry.spill_receiver != INVALID_SAMPLE_ID {
                        let outflow_m3_s = excess_volume / phase_seconds;
                        route_lake_outflow(
                            geometry.spill_receiver,
                            outflow_m3_s,
                            &active_lake_for_sample,
                            &topography.submerged_mask,
                            &drainage.receiver,
                            &mut realized_accum_m3_s,
                            &mut lake_inflow_m3_s,
                            &mut terminal_rate_m3_s,
                        )?;
                    } else {
                        terminal_storage_volume_m3 += excess_volume;
                    }
                }
                current_volume_m3[lake_index] = retained_volume;
            }

            terminal_volume_m3 += terminal_rate_m3_s * phase_seconds;

            for i in 0..count {
                maximum_realized_m3_s = maximum_realized_m3_s.max(realized_accum_m3_s[i]);
                let value = if lake_fraction[i] > 0.0 {
                    0.0
                } else {
                    realized_accum_m3_s[i]
                };
                if value > f32::MAX as f64 || !value.is_finite() {
                    return Err("WG-6D realized seasonal discharge exceeds representable range");
                }
                phase_realized_discharge_m3_s[phase_start + i] = value as f32;
            }

            for (lake_index, geometry) in active_lakes.iter().enumerate() {
                let surface = surface_for_volume(
                    geometry,
                    &topography.solid_elevation_m,
                    &area_m2,
                    current_volume_m3[lake_index],
                );
                let (area, volume) = geometry_at_surface(
                    geometry,
                    &topography.solid_elevation_m,
                    &area_m2,
                    surface,
                    None,
                );
                minimum_lake_surface_m[lake_index] = minimum_lake_surface_m[lake_index].min(surface);
                maximum_lake_surface_m[lake_index] = maximum_lake_surface_m[lake_index].max(surface);
                let index = phase * lake_count + lake_index;
                phase_lake_surface_elevation_m[index] = surface as f32;
                phase_lake_area_m2[index] = area;
                phase_lake_volume_m3[index] = volume;
            }
        }

        let mut cycle_relative_change = 0.0_f64;
        for (lake_index, geometry) in active_lakes.iter().enumerate() {
            let scale = geometry.maximum_volume_m3.max(1.0);
            cycle_relative_change = cycle_relative_change.max(
                (current_volume_m3[lake_index] - year_start_volume_m3[lake_index]).abs() / scale,
            );
        }
        completed_years = year + 1;
        final_cycle_relative_change = cycle_relative_change;
        final_start_storage_m3 = year_start_volume_m3.iter().sum::<f64>();
        final_end_storage_m3 = current_volume_m3.iter().sum::<f64>();
        final_input_volume_m3 = input_volume_m3;
        final_terminal_volume_m3 = terminal_volume_m3;
        final_lake_precipitation_volume_m3 = lake_precipitation_volume_m3;
        final_lake_evaporation_volume_m3 = lake_evaporation_volume_m3;
        final_terminal_storage_volume_m3 = terminal_storage_volume_m3;
        final_maximum_realized_m3_s = maximum_realized_m3_s;
        final_maximum_lake_level_range_m = minimum_lake_surface_m
            .iter()
            .zip(maximum_lake_surface_m.iter())
            .filter(|(minimum, maximum)| minimum.is_finite() && maximum.is_finite())
            .map(|(minimum, maximum)| maximum - minimum)
            .fold(0.0_f64, f64::max);

        if lake_count == 0
            || (completed_years >= MINIMUM_LAKE_SPINUP_YEARS
                && cycle_relative_change <= LAKE_CYCLE_CONVERGENCE_RELATIVE)
        {
            break;
        }
    }

    let water_lhs = final_start_storage_m3 + final_input_volume_m3;
    let water_rhs = final_end_storage_m3
        + final_terminal_volume_m3
        + final_lake_evaporation_volume_m3
        + final_terminal_storage_volume_m3;
    let water_balance_relative_error = if water_lhs > 0.0 {
        (water_lhs - water_rhs).abs() / water_lhs
    } else {
        water_rhs.abs()
    };

    Ok(SeasonalLakeRoutingResult {
        phase_realized_discharge_m3_s,
        phase_lake_surface_elevation_m,
        phase_lake_area_m2,
        phase_lake_volume_m3,
        annual_mean_terminal_realized_discharge_m3_s: final_terminal_volume_m3
            / planet.orbital_period_s,
        maximum_phase_realized_discharge_m3_s: final_maximum_realized_m3_s,
        annual_mean_lake_precipitation_m3_s: final_lake_precipitation_volume_m3
            / planet.orbital_period_s,
        annual_mean_lake_evaporation_m3_s: final_lake_evaporation_volume_m3
            / planet.orbital_period_s,
        annual_mean_unreleased_terminal_storage_m3_s: final_terminal_storage_volume_m3
            / planet.orbital_period_s,
        water_balance_relative_error,
        lake_spinup_years: completed_years,
        final_lake_cycle_relative_change: final_cycle_relative_change,
        maximum_seasonal_lake_level_range_m: final_maximum_lake_level_range_m,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_lake() -> ActiveLakeGeometry {
        ActiveLakeGeometry {
            members: vec![0, 1],
            sorted_members: vec![0, 1],
            spill_elevation_m: 20.0,
            spill_receiver: INVALID_SAMPLE_ID,
            maximum_volume_m3: 0.0,
            evaporation_scale: 1.0,
        }
    }

    #[test]
    fn seasonal_lake_geometry_is_monotone_with_surface_level() {
        let mut lake = simple_lake();
        let elevation = [0.0_f32, 10.0];
        let area = [100.0_f64, 100.0];
        lake.maximum_volume_m3 = geometry_at_surface(&lake, &elevation, &area, 20.0, None).1;
        let (_, low_volume) = geometry_at_surface(&lake, &elevation, &area, 5.0, None);
        let (_, high_volume) = geometry_at_surface(&lake, &elevation, &area, 15.0, None);
        assert!(low_volume > 0.0);
        assert!(high_volume > low_volume);
        assert!(lake.maximum_volume_m3 > high_volume);
    }

    #[test]
    fn seasonal_lake_volume_inversion_recovers_surface_level() {
        let mut lake = simple_lake();
        let elevation = [0.0_f32, 10.0];
        let area = [100.0_f64, 100.0];
        lake.maximum_volume_m3 = geometry_at_surface(&lake, &elevation, &area, 20.0, None).1;
        let target_surface = 13.0;
        let target_volume = geometry_at_surface(&lake, &elevation, &area, target_surface, None).1;
        let recovered = surface_for_volume(&lake, &elevation, &area, target_volume);
        assert!((recovered - target_surface).abs() < 1.0e-8);
    }
}
