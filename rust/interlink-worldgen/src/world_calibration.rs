use crate::{
    ClimateState, CrustKind, FluvialErosionState, GeodesicTopology, InheritedPhysicalState,
    LakeSedimentInfillState, PlanetPhysicalParameters, TerrainEvolutionState, TopographyState,
    WorldgenError, WORLDGEN_ENGINE_VERSION,
};
use std::collections::{BTreeSet, VecDeque};
use std::f64::consts::PI;
use std::fmt::Write as _;

pub const WORLD_CALIBRATION_SCHEMA_ID: &str = "planet-engine-calibration";
pub const WORLD_CALIBRATION_SCHEMA_VERSION: u32 = 1;
pub const WORLD_CALIBRATION_RANKED_LIMIT: usize = 8;
const SIGNIFICANT_CONTINENT_AREA_FRACTION: f64 = 0.0025;

#[derive(Clone, Debug, PartialEq)]
pub struct WorldCalibrationRun {
    pub seed: String,
    pub engine_version: u32,
    pub coarse_level: u8,
    pub fine_level: u8,
    pub sample_count: u32,
    pub plate_count: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldCalibrationHashes {
    pub coarse_topology_hash: String,
    pub fine_topology_hash: String,
    pub tectonic_hash: String,
    pub geology_hash: String,
    pub lithosphere_hash: String,
    pub inheritance_hash: String,
    pub topography_hash: String,
    pub climate_hash: String,
    pub final_drainage_hash: String,
    pub final_runoff_hash: String,
    pub final_lake_hash: String,
    pub final_seasonal_hash: String,
    pub erosion_hash: String,
    pub evolution_hash: String,
    pub post_erosion_hydrology_hash: String,
    pub infill_hash: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContinentalComponentSummary {
    pub anchor_sample: u32,
    pub sample_count: u32,
    pub area_km2: f64,
    pub perimeter_km: f64,
    pub diameter_km: f64,
    pub plate_count: u32,
    pub elongation: f64,
    pub compactness: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContinentalAssemblySummary {
    pub significant_component_count: u32,
    pub component_area_coefficient_of_variation: f64,
    pub largest_to_median_area_ratio: f64,
    pub maximum_elongation: f64,
    pub maximum_compactness: f64,
    pub largest_component_plate_count: u32,
    pub has_major_multiplate_component: bool,
    pub ranked_components: Vec<ContinentalComponentSummary>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopographyCalibrationSummary {
    pub minimum_solid_elevation_m: f64,
    pub p05_solid_elevation_m: f64,
    pub median_solid_elevation_m: f64,
    pub p95_solid_elevation_m: f64,
    pub maximum_solid_elevation_m: f64,
    pub sea_level_m: Option<f64>,
    pub land_area_fraction: f64,
    pub ocean_area_fraction: f64,
    pub mean_land_elevation_m: f64,
    pub mean_water_depth_m: f64,
    pub maximum_water_depth_m: f64,
    pub water_volume_relative_error: f64,
    pub clamped_sample_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClimateCalibrationSummary {
    pub global_solver_level: u8,
    pub global_solver_sample_count: u32,
    pub orbital_phase_count: u8,
    pub spinup_years: u8,
    pub minimum_temperature_k: f64,
    pub mean_temperature_k: f64,
    pub maximum_temperature_k: f64,
    pub mean_land_temperature_k: f64,
    pub mean_ocean_temperature_k: f64,
    pub mean_wind_speed_m_s: f64,
    pub maximum_wind_speed_m_s: f64,
    pub mean_surface_current_m_s: f64,
    pub maximum_surface_current_m_s: f64,
    pub mean_sea_surface_temperature_k: f64,
    pub mean_annual_precipitation_mm: f64,
    pub p95_annual_precipitation_mm: f64,
    pub precipitation_p95_to_mean_ratio: f64,
    pub moisture_budget_relative_error: f64,
    pub moisture_transport_limiter_fraction: f64,
    pub maximum_moisture_transport_substeps: u8,
    pub persistent_snow_area_fraction: f64,
    pub sea_ice_area_fraction: f64,
    pub final_temperature_rms_change_k: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RankedBasinSummary {
    pub basin_id: u32,
    pub outlet_sample: u32,
    pub outlet_kind: u8,
    pub sample_count: u32,
    pub area_km2: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RankedDepressionSummary {
    pub depression_id: u32,
    pub floor_sample: u32,
    pub sample_count: u32,
    pub area_km2: f64,
    pub maximum_depth_m: f64,
    pub floor_elevation_m: f64,
    pub spill_elevation_m: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RankedLakeSummary {
    pub lake_id: u32,
    pub depression_id: u32,
    pub kind: u8,
    pub area_km2: f64,
    pub volume_km3: f64,
    pub maximum_depth_m: f64,
    pub surface_elevation_m: f64,
    pub gross_land_inflow_m3_s: f64,
    pub lake_evaporation_m3_s: f64,
    pub outflow_m3_s: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HydrologyCalibrationSummary {
    pub basin_count: u32,
    pub depression_count: u32,
    pub largest_contributing_area_km2: f64,
    pub largest_basin_area_fraction_of_land: f64,
    pub maximum_depression_depth_m: f64,
    pub drainage_area_conservation_relative_error: f64,
    pub mean_land_precipitation_mm: f64,
    pub mean_land_actual_evapotranspiration_mm: f64,
    pub mean_land_runoff_mm: f64,
    pub land_runoff_fraction: f64,
    pub maximum_potential_discharge_m3_s: f64,
    pub runoff_discharge_conservation_relative_error: f64,
    pub lake_count: u32,
    pub endorheic_lake_count: u32,
    pub overflowing_lake_count: u32,
    pub terminal_storage_lake_count: u32,
    pub total_lake_area_km2: f64,
    pub total_lake_volume_km3: f64,
    pub largest_lake_area_km2: f64,
    pub largest_lake_area_fraction_of_land: f64,
    pub maximum_lake_depth_m: f64,
    pub lake_water_balance_relative_error: f64,
    pub dry_flow_sample_count: u32,
    pub intermittent_flow_sample_count: u32,
    pub perennial_flow_sample_count: u32,
    pub snowmelt_runoff_fraction: f64,
    pub maximum_phase_realized_discharge_m3_s: f64,
    pub seasonal_routing_conservation_relative_error: f64,
    pub seasonal_water_balance_relative_error: f64,
    pub lake_spinup_years: u8,
    pub final_lake_surface_cycle_change_m: f64,
    pub maximum_seasonal_lake_level_range_m: f64,
    pub largest_basins: Vec<RankedBasinSummary>,
    pub largest_depressions: Vec<RankedDepressionSummary>,
    pub largest_lakes: Vec<RankedLakeSummary>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeomorphologyCalibrationSummary {
    pub erosive_sample_count: u32,
    pub active_lake_trap_count: u32,
    pub maximum_effective_discharge_m3_s: f64,
    pub maximum_channel_slope: f64,
    pub maximum_channel_width_m: f64,
    pub maximum_incision_potential_m_per_year: f64,
    pub total_sediment_generated_kg_s: f64,
    pub total_land_deposition_kg_s: f64,
    pub total_lake_deposition_kg_s: f64,
    pub total_terminal_ocean_deposition_kg_s: f64,
    pub erosion_sediment_conservation_relative_error: f64,
    pub geomorphic_duration_years: f64,
    pub eroded_sample_count: u32,
    pub depositional_sample_count: u32,
    pub receiver_changed_sample_count: u32,
    pub receiver_changed_fraction: f64,
    pub maximum_applied_erosion_m: f64,
    pub maximum_applied_deposition_m: f64,
    pub mean_land_absolute_terrain_change_m: f64,
    pub evolution_sediment_conservation_relative_error: f64,
    pub post_erosion_runoff_conservation_relative_error: f64,
    pub filled_depression_count: u32,
    pub filled_sample_count: u32,
    pub capacity_limited_depression_count: u32,
    pub maximum_lake_fill_depth_m: f64,
    pub total_historical_lake_delivery_kg_s: f64,
    pub total_applied_lake_fill_equivalent_kg_s: f64,
    pub total_unapplied_lake_sediment_kg_s: f64,
    pub infill_sediment_conservation_relative_error: f64,
    pub pre_infill_lake_count: u32,
    pub post_infill_lake_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldCalibrationReport {
    pub run: WorldCalibrationRun,
    pub hashes: WorldCalibrationHashes,
    pub continents: ContinentalAssemblySummary,
    pub topography: TopographyCalibrationSummary,
    pub climate: ClimateCalibrationSummary,
    pub hydrology: HydrologyCalibrationSummary,
    pub geomorphology: GeomorphologyCalibrationSummary,
}

#[derive(Clone, Debug)]
struct RawContinentalComponent {
    anchor_sample: u32,
    sample_count: u32,
    area_sr: f64,
    perimeter_rad: f64,
    diameter_rad: f64,
    plate_count: u32,
}

fn great_circle_arc(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2])
        .clamp(-1.0, 1.0)
        .acos()
}

fn continental_components(
    topology: &GeodesicTopology,
    crust_kind: &[u8],
    plate_ids: &[u16],
) -> Vec<RawContinentalComponent> {
    let mask = crust_kind
        .iter()
        .map(|kind| *kind == CrustKind::Continental as u8)
        .collect::<Vec<_>>();
    let mut visited = vec![false; mask.len()];
    let mut components = Vec::new();
    for start in 0..mask.len() {
        if !mask[start] || visited[start] {
            continue;
        }
        visited[start] = true;
        let mut queue = VecDeque::from([start]);
        let mut samples = Vec::new();
        while let Some(sample) = queue.pop_front() {
            samples.push(sample);
            for neighbor in topology.neighbors_of(sample as u32) {
                let index = *neighbor as usize;
                if mask[index] && !visited[index] {
                    visited[index] = true;
                    queue.push_back(index);
                }
            }
        }
        let mut area_sr = 0.0;
        let mut perimeter_rad = 0.0;
        let mut plates = BTreeSet::new();
        for sample in &samples {
            area_sr += topology.dual_area_steradians()[*sample];
            plates.insert(plate_ids[*sample]);
            for (neighbor, arc_length) in topology
                .neighbors_of(*sample as u32)
                .iter()
                .zip(topology.neighbor_arc_lengths_of(*sample as u32).iter())
            {
                if !mask[*neighbor as usize] {
                    perimeter_rad += *arc_length;
                }
            }
        }
        let positions = topology.positions();
        let first = samples[0];
        let farthest_from_first = samples
            .iter()
            .copied()
            .max_by(|a, b| {
                great_circle_arc(positions[first], positions[*a])
                    .total_cmp(&great_circle_arc(positions[first], positions[*b]))
            })
            .unwrap_or(first);
        let farthest = samples
            .iter()
            .copied()
            .max_by(|a, b| {
                great_circle_arc(positions[farthest_from_first], positions[*a]).total_cmp(
                    &great_circle_arc(positions[farthest_from_first], positions[*b]),
                )
            })
            .unwrap_or(farthest_from_first);
        components.push(RawContinentalComponent {
            anchor_sample: start as u32,
            sample_count: samples.len() as u32,
            area_sr,
            perimeter_rad,
            diameter_rad: great_circle_arc(positions[farthest_from_first], positions[farthest]),
            plate_count: plates.len() as u32,
        });
    }
    components
}

fn summarize_continents(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    planet: PlanetPhysicalParameters,
) -> ContinentalAssemblySummary {
    let total_area_sr = topology.metrics().total_area_steradians.max(1.0e-18);
    let mut components =
        continental_components(topology, &inherited.crust_kind, &inherited.plate_ids)
            .into_iter()
            .filter(|component| {
                component.area_sr >= total_area_sr * SIGNIFICANT_CONTINENT_AREA_FRACTION
            })
            .collect::<Vec<_>>();
    components.sort_by(|a, b| b.area_sr.total_cmp(&a.area_sr));
    let areas = components
        .iter()
        .map(|component| component.area_sr)
        .collect::<Vec<_>>();
    let mean_area = if areas.is_empty() {
        0.0
    } else {
        areas.iter().sum::<f64>() / areas.len() as f64
    };
    let variance = if areas.is_empty() {
        0.0
    } else {
        areas
            .iter()
            .map(|area| (area - mean_area).powi(2))
            .sum::<f64>()
            / areas.len() as f64
    };
    let cv = if mean_area > 0.0 {
        variance.sqrt() / mean_area
    } else {
        0.0
    };
    let median = if areas.is_empty() {
        0.0
    } else {
        areas[areas.len() / 2]
    };
    let hierarchy = areas
        .first()
        .copied()
        .map(|largest| largest / median.max(1.0e-18))
        .unwrap_or(0.0);
    let radius_km = planet.radius_m / 1_000.0;
    let area_scale_km2 = radius_km * radius_km;
    let mut ranked = Vec::new();
    let mut max_elongation = 0.0_f64;
    let mut max_compactness = 0.0_f64;
    let mut major_multiplate = false;
    for component in &components {
        let equivalent_radius = (component.area_sr / PI).sqrt().max(1.0e-9);
        let elongation = component.diameter_rad / (2.0 * equivalent_radius);
        let compactness =
            component.perimeter_rad.powi(2) / (4.0 * PI * component.area_sr.max(1.0e-18));
        max_elongation = max_elongation.max(elongation);
        max_compactness = max_compactness.max(compactness);
        if component.area_sr >= total_area_sr * 0.015 && component.plate_count >= 2 {
            major_multiplate = true;
        }
        if ranked.len() < WORLD_CALIBRATION_RANKED_LIMIT {
            ranked.push(ContinentalComponentSummary {
                anchor_sample: component.anchor_sample,
                sample_count: component.sample_count,
                area_km2: component.area_sr * area_scale_km2,
                perimeter_km: component.perimeter_rad * radius_km,
                diameter_km: component.diameter_rad * radius_km,
                plate_count: component.plate_count,
                elongation,
                compactness,
            });
        }
    }
    ContinentalAssemblySummary {
        significant_component_count: components.len() as u32,
        component_area_coefficient_of_variation: cv,
        largest_to_median_area_ratio: hierarchy,
        maximum_elongation: max_elongation,
        maximum_compactness: max_compactness,
        largest_component_plate_count: components
            .first()
            .map(|component| component.plate_count)
            .unwrap_or(0),
        has_major_multiplate_component: major_multiplate,
        ranked_components: ranked,
    }
}

fn ranked_basins(infill: &LakeSedimentInfillState) -> Vec<RankedBasinSummary> {
    let mut basins = infill
        .post_infill_drainage
        .basins
        .iter()
        .collect::<Vec<_>>();
    basins.sort_by(|a, b| b.area_m2.total_cmp(&a.area_m2));
    basins
        .into_iter()
        .take(WORLD_CALIBRATION_RANKED_LIMIT)
        .map(|basin| RankedBasinSummary {
            basin_id: basin.id,
            outlet_sample: basin.outlet_sample,
            outlet_kind: basin.outlet_kind,
            sample_count: basin.sample_count,
            area_km2: basin.area_m2 / 1.0e6,
        })
        .collect()
}

fn ranked_depressions(infill: &LakeSedimentInfillState) -> Vec<RankedDepressionSummary> {
    let mut depressions = infill
        .post_infill_drainage
        .depressions
        .iter()
        .collect::<Vec<_>>();
    depressions.sort_by(|a, b| b.area_m2.total_cmp(&a.area_m2));
    depressions
        .into_iter()
        .take(WORLD_CALIBRATION_RANKED_LIMIT)
        .map(|depression| RankedDepressionSummary {
            depression_id: depression.id,
            floor_sample: depression.floor_sample,
            sample_count: depression.sample_count,
            area_km2: depression.area_m2 / 1.0e6,
            maximum_depth_m: depression.maximum_depth_m,
            floor_elevation_m: depression.floor_elevation_m,
            spill_elevation_m: depression.spill_elevation_m,
        })
        .collect()
}

fn ranked_lakes(infill: &LakeSedimentInfillState) -> Vec<RankedLakeSummary> {
    let mut lakes = infill.reconciled_lakes.lakes.iter().collect::<Vec<_>>();
    lakes.sort_by(|a, b| b.area_m2.total_cmp(&a.area_m2));
    lakes
        .into_iter()
        .take(WORLD_CALIBRATION_RANKED_LIMIT)
        .map(|lake| RankedLakeSummary {
            lake_id: lake.id,
            depression_id: lake.depression_id,
            kind: lake.kind,
            area_km2: lake.area_m2 / 1.0e6,
            volume_km3: lake.volume_m3 / 1.0e9,
            maximum_depth_m: lake.maximum_depth_m,
            surface_elevation_m: lake.surface_elevation_m,
            gross_land_inflow_m3_s: lake.gross_land_inflow_m3_s,
            lake_evaporation_m3_s: lake.lake_evaporation_m3_s,
            outflow_m3_s: lake.outflow_m3_s,
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub fn build_world_calibration_report(
    seed: &str,
    plate_count: u16,
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    terrain: &TopographyState,
    climate: &ClimateState,
    erosion: &FluvialErosionState,
    evolution: &TerrainEvolutionState,
    infill: &LakeSedimentInfillState,
    planet: PlanetPhysicalParameters,
    coarse_topology_hash: &str,
    tectonic_hash: &str,
    geology_hash: &str,
    lithosphere_hash: &str,
) -> Result<WorldCalibrationReport, WorldgenError> {
    let count = topology.metrics().sample_count as usize;
    if inherited.crust_kind.len() != count
        || inherited.plate_ids.len() != count
        || terrain.solid_elevation_m.len() != count
        || climate.temperature_mean_k.len() != count
        || infill.post_infill_solid_elevation_m.len() != count
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "world calibration inputs must align with final topology sample count",
        ));
    }

    let drainage = &infill.post_infill_drainage.metrics;
    let runoff = &infill.reconciled_runoff.metrics;
    let lakes = &infill.reconciled_lakes.metrics;
    let seasonal = &infill.reconciled_seasonal.metrics;
    let erosion_metrics = &erosion.metrics;
    let evolution_metrics = &evolution.metrics;
    let infill_metrics = &infill.metrics;
    let land_area_km2 = (drainage.land_area_m2 / 1.0e6).max(1.0e-12);
    let largest_basins = ranked_basins(infill);
    let largest_depressions = ranked_depressions(infill);
    let largest_lakes = ranked_lakes(infill);
    let largest_basin_area_km2 = largest_basins
        .first()
        .map(|basin| basin.area_km2)
        .unwrap_or(0.0);

    Ok(WorldCalibrationReport {
        run: WorldCalibrationRun {
            seed: seed.to_owned(),
            engine_version: WORLDGEN_ENGINE_VERSION,
            coarse_level: inherited.map.metrics.coarse_level,
            fine_level: inherited.map.metrics.fine_level,
            sample_count: topology.metrics().sample_count,
            plate_count,
        },
        hashes: WorldCalibrationHashes {
            coarse_topology_hash: coarse_topology_hash.to_owned(),
            fine_topology_hash: topology.metrics().topology_hash_hex(),
            tectonic_hash: tectonic_hash.to_owned(),
            geology_hash: geology_hash.to_owned(),
            lithosphere_hash: lithosphere_hash.to_owned(),
            inheritance_hash: inherited.inheritance_hash_hex(),
            topography_hash: terrain.metrics.topography_hash_hex(),
            climate_hash: climate.metrics.climate_hash_hex(),
            final_drainage_hash: drainage.drainage_hash_hex(),
            final_runoff_hash: runoff.runoff_hash_hex(),
            final_lake_hash: lakes.lake_hash_hex(),
            final_seasonal_hash: seasonal.seasonal_hydrology_hash_hex(),
            erosion_hash: format!("{:016x}", erosion_metrics.fluvial_erosion_hash),
            evolution_hash: format!("{:016x}", evolution_metrics.terrain_evolution_hash),
            post_erosion_hydrology_hash: format!(
                "{:016x}",
                infill_metrics.post_erosion_hydrology_hash
            ),
            infill_hash: infill_metrics.lake_sediment_infill_hash_hex(),
        },
        continents: summarize_continents(topology, inherited, planet),
        topography: TopographyCalibrationSummary {
            minimum_solid_elevation_m: terrain.metrics.minimum_solid_elevation_m,
            p05_solid_elevation_m: terrain.metrics.p05_solid_elevation_m,
            median_solid_elevation_m: terrain.metrics.median_solid_elevation_m,
            p95_solid_elevation_m: terrain.metrics.p95_solid_elevation_m,
            maximum_solid_elevation_m: terrain.metrics.maximum_solid_elevation_m,
            sea_level_m: terrain.metrics.sea_level_m,
            land_area_fraction: terrain.metrics.land_area_fraction,
            ocean_area_fraction: terrain.metrics.ocean_area_fraction,
            mean_land_elevation_m: terrain.metrics.mean_land_elevation_m,
            mean_water_depth_m: terrain.metrics.mean_water_depth_m,
            maximum_water_depth_m: terrain.metrics.maximum_water_depth_m,
            water_volume_relative_error: terrain.metrics.water_volume_relative_error,
            clamped_sample_count: terrain.metrics.clamped_sample_count,
        },
        climate: ClimateCalibrationSummary {
            global_solver_level: climate.metrics.global_solver_level,
            global_solver_sample_count: climate.metrics.global_solver_sample_count,
            orbital_phase_count: climate.metrics.orbital_phase_count,
            spinup_years: climate.metrics.spinup_years,
            minimum_temperature_k: climate.metrics.minimum_temperature_k,
            mean_temperature_k: climate.metrics.mean_temperature_k,
            maximum_temperature_k: climate.metrics.maximum_temperature_k,
            mean_land_temperature_k: climate.metrics.mean_land_temperature_k,
            mean_ocean_temperature_k: climate.metrics.mean_ocean_temperature_k,
            mean_wind_speed_m_s: climate.metrics.mean_wind_speed_m_s,
            maximum_wind_speed_m_s: climate.metrics.maximum_wind_speed_m_s,
            mean_surface_current_m_s: climate.metrics.mean_surface_current_m_s,
            maximum_surface_current_m_s: climate.metrics.maximum_surface_current_m_s,
            mean_sea_surface_temperature_k: climate.metrics.mean_sea_surface_temperature_k,
            mean_annual_precipitation_mm: climate.metrics.mean_annual_precipitation_mm,
            p95_annual_precipitation_mm: climate.metrics.p95_annual_precipitation_mm,
            precipitation_p95_to_mean_ratio: climate.metrics.p95_annual_precipitation_mm
                / climate.metrics.mean_annual_precipitation_mm.max(1.0e-12),
            moisture_budget_relative_error: climate.metrics.moisture_budget_relative_error,
            moisture_transport_limiter_fraction: climate
                .metrics
                .moisture_transport_limiter_fraction,
            maximum_moisture_transport_substeps: climate
                .metrics
                .maximum_moisture_transport_substeps,
            persistent_snow_area_fraction: climate.metrics.persistent_snow_area_fraction,
            sea_ice_area_fraction: climate.metrics.sea_ice_area_fraction,
            final_temperature_rms_change_k: climate.metrics.final_temperature_rms_change_k,
        },
        hydrology: HydrologyCalibrationSummary {
            basin_count: drainage.basin_count,
            depression_count: drainage.depression_count,
            largest_contributing_area_km2: drainage.maximum_contributing_area_m2 / 1.0e6,
            largest_basin_area_fraction_of_land: largest_basin_area_km2 / land_area_km2,
            maximum_depression_depth_m: drainage.maximum_depression_depth_m,
            drainage_area_conservation_relative_error: drainage.area_conservation_relative_error,
            mean_land_precipitation_mm: runoff.mean_land_precipitation_mm,
            mean_land_actual_evapotranspiration_mm: runoff.mean_land_actual_evapotranspiration_mm,
            mean_land_runoff_mm: runoff.mean_land_runoff_mm,
            land_runoff_fraction: runoff.land_runoff_fraction,
            maximum_potential_discharge_m3_s: runoff.maximum_potential_discharge_m3_s,
            runoff_discharge_conservation_relative_error: runoff
                .discharge_conservation_relative_error,
            lake_count: lakes.lake_count,
            endorheic_lake_count: lakes.endorheic_lake_count,
            overflowing_lake_count: lakes.overflowing_lake_count,
            terminal_storage_lake_count: lakes.terminal_storage_lake_count,
            total_lake_area_km2: lakes.total_lake_area_m2 / 1.0e6,
            total_lake_volume_km3: lakes.total_lake_volume_m3 / 1.0e9,
            largest_lake_area_km2: lakes.maximum_lake_area_m2 / 1.0e6,
            largest_lake_area_fraction_of_land: (lakes.maximum_lake_area_m2 / 1.0e6)
                / land_area_km2,
            maximum_lake_depth_m: lakes.maximum_lake_depth_m,
            lake_water_balance_relative_error: lakes.water_balance_relative_error,
            dry_flow_sample_count: seasonal.dry_flow_sample_count,
            intermittent_flow_sample_count: seasonal.intermittent_flow_sample_count,
            perennial_flow_sample_count: seasonal.perennial_flow_sample_count,
            snowmelt_runoff_fraction: seasonal.snowmelt_runoff_fraction,
            maximum_phase_realized_discharge_m3_s: seasonal.maximum_phase_realized_discharge_m3_s,
            seasonal_routing_conservation_relative_error: seasonal
                .seasonal_routing_conservation_relative_error,
            seasonal_water_balance_relative_error: seasonal.seasonal_water_balance_relative_error,
            lake_spinup_years: seasonal.lake_spinup_years,
            final_lake_surface_cycle_change_m: seasonal.final_lake_surface_cycle_change_m,
            maximum_seasonal_lake_level_range_m: seasonal.maximum_seasonal_lake_level_range_m,
            largest_basins,
            largest_depressions,
            largest_lakes,
        },
        geomorphology: GeomorphologyCalibrationSummary {
            erosive_sample_count: erosion_metrics.erosive_sample_count,
            active_lake_trap_count: erosion_metrics.active_lake_trap_count,
            maximum_effective_discharge_m3_s: erosion_metrics.maximum_effective_discharge_m3_s,
            maximum_channel_slope: erosion_metrics.maximum_channel_slope,
            maximum_channel_width_m: erosion_metrics.maximum_channel_width_m,
            maximum_incision_potential_m_per_year: erosion_metrics
                .maximum_incision_potential_m_per_year,
            total_sediment_generated_kg_s: erosion_metrics.total_sediment_generated_kg_s,
            total_land_deposition_kg_s: erosion_metrics.total_land_deposition_kg_s,
            total_lake_deposition_kg_s: erosion_metrics.total_lake_deposition_kg_s,
            total_terminal_ocean_deposition_kg_s: erosion_metrics
                .total_terminal_ocean_deposition_kg_s,
            erosion_sediment_conservation_relative_error: erosion_metrics
                .sediment_conservation_relative_error,
            geomorphic_duration_years: evolution_metrics.geomorphic_duration_years,
            eroded_sample_count: evolution_metrics.eroded_sample_count,
            depositional_sample_count: evolution_metrics.depositional_sample_count,
            receiver_changed_sample_count: evolution_metrics.receiver_changed_sample_count,
            receiver_changed_fraction: evolution_metrics.receiver_changed_fraction,
            maximum_applied_erosion_m: evolution_metrics.maximum_applied_erosion_m,
            maximum_applied_deposition_m: evolution_metrics.maximum_applied_deposition_m,
            mean_land_absolute_terrain_change_m: evolution_metrics
                .mean_land_absolute_terrain_change_m,
            evolution_sediment_conservation_relative_error: evolution_metrics
                .sediment_conservation_relative_error,
            post_erosion_runoff_conservation_relative_error: evolution_metrics
                .post_erosion_runoff_conservation_relative_error,
            filled_depression_count: infill_metrics.filled_depression_count,
            filled_sample_count: infill_metrics.filled_sample_count,
            capacity_limited_depression_count: infill_metrics.capacity_limited_depression_count,
            maximum_lake_fill_depth_m: infill_metrics.maximum_fill_depth_m,
            total_historical_lake_delivery_kg_s: infill_metrics.total_historical_lake_delivery_kg_s,
            total_applied_lake_fill_equivalent_kg_s: infill_metrics
                .total_applied_lake_fill_equivalent_kg_s,
            total_unapplied_lake_sediment_kg_s: infill_metrics.total_unapplied_lake_sediment_kg_s,
            infill_sediment_conservation_relative_error: infill_metrics
                .sediment_conservation_relative_error,
            pre_infill_lake_count: infill_metrics.pre_infill_lake_count,
            post_infill_lake_count: infill_metrics.post_infill_lake_count,
        },
    })
}

fn json_escape(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 8);
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => {
                let _ = write!(output, "\\u{:04x}", ch as u32);
            }
            ch => output.push(ch),
        }
    }
    output
}

fn json_number(value: f64) -> String {
    if value.is_finite() {
        value.to_string()
    } else {
        "null".to_owned()
    }
}

fn json_optional_number(value: Option<f64>) -> String {
    value.map(json_number).unwrap_or_else(|| "null".to_owned())
}

impl WorldCalibrationReport {
    pub fn to_json_pretty(&self) -> String {
        let mut out = String::new();
        let q = |value: &str| format!("\"{}\"", json_escape(value));
        let n = json_number;
        let _ = writeln!(out, "{{");
        let _ = writeln!(
            out,
            "  \"schema\": \"{}@{}\",",
            WORLD_CALIBRATION_SCHEMA_ID, WORLD_CALIBRATION_SCHEMA_VERSION
        );
        let _ = writeln!(out, "  \"run\": {{");
        let _ = writeln!(out, "    \"seed\": {},", q(&self.run.seed));
        let _ = writeln!(out, "    \"engine_version\": {},", self.run.engine_version);
        let _ = writeln!(out, "    \"coarse_level\": {},", self.run.coarse_level);
        let _ = writeln!(out, "    \"fine_level\": {},", self.run.fine_level);
        let _ = writeln!(out, "    \"sample_count\": {},", self.run.sample_count);
        let _ = writeln!(out, "    \"plate_count\": {}", self.run.plate_count);
        let _ = writeln!(out, "  }},");
        let _ = writeln!(out, "  \"hashes\": {{");
        let hash_fields = [
            ("coarse_topology", &self.hashes.coarse_topology_hash),
            ("fine_topology", &self.hashes.fine_topology_hash),
            ("tectonic", &self.hashes.tectonic_hash),
            ("geology", &self.hashes.geology_hash),
            ("lithosphere", &self.hashes.lithosphere_hash),
            ("inheritance", &self.hashes.inheritance_hash),
            ("topography", &self.hashes.topography_hash),
            ("climate", &self.hashes.climate_hash),
            ("final_drainage", &self.hashes.final_drainage_hash),
            ("final_runoff", &self.hashes.final_runoff_hash),
            ("final_lake", &self.hashes.final_lake_hash),
            ("final_seasonal", &self.hashes.final_seasonal_hash),
            ("erosion", &self.hashes.erosion_hash),
            ("evolution", &self.hashes.evolution_hash),
            (
                "post_erosion_hydrology",
                &self.hashes.post_erosion_hydrology_hash,
            ),
            ("infill", &self.hashes.infill_hash),
        ];
        for (index, (name, value)) in hash_fields.iter().enumerate() {
            let comma = if index + 1 == hash_fields.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(out, "    \"{name}\": {}{comma}", q(value));
        }
        let _ = writeln!(out, "  }},");
        let c = &self.continents;
        let _ = writeln!(out, "  \"continents\": {{");
        let _ = writeln!(
            out,
            "    \"significant_component_count\": {},",
            c.significant_component_count
        );
        let _ = writeln!(
            out,
            "    \"component_area_cv\": {},",
            n(c.component_area_coefficient_of_variation)
        );
        let _ = writeln!(
            out,
            "    \"largest_to_median_area_ratio\": {},",
            n(c.largest_to_median_area_ratio)
        );
        let _ = writeln!(
            out,
            "    \"maximum_elongation\": {},",
            n(c.maximum_elongation)
        );
        let _ = writeln!(
            out,
            "    \"maximum_compactness\": {},",
            n(c.maximum_compactness)
        );
        let _ = writeln!(
            out,
            "    \"largest_component_plate_count\": {},",
            c.largest_component_plate_count
        );
        let _ = writeln!(
            out,
            "    \"has_major_multiplate_component\": {},",
            c.has_major_multiplate_component
        );
        let _ = writeln!(out, "    \"ranked_components\": [");
        for (index, component) in c.ranked_components.iter().enumerate() {
            let comma = if index + 1 == c.ranked_components.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(out, "      {{\"anchor_sample\":{},\"sample_count\":{},\"area_km2\":{},\"perimeter_km\":{},\"diameter_km\":{},\"plate_count\":{},\"elongation\":{},\"compactness\":{}}}{comma}", component.anchor_sample, component.sample_count, n(component.area_km2), n(component.perimeter_km), n(component.diameter_km), component.plate_count, n(component.elongation), n(component.compactness));
        }
        let _ = writeln!(out, "    ]");
        let _ = writeln!(out, "  }},");
        let t = &self.topography;
        let _ = writeln!(out, "  \"topography\": {{\"minimum_solid_elevation_m\":{},\"p05_solid_elevation_m\":{},\"median_solid_elevation_m\":{},\"p95_solid_elevation_m\":{},\"maximum_solid_elevation_m\":{},\"sea_level_m\":{},\"land_area_fraction\":{},\"ocean_area_fraction\":{},\"mean_land_elevation_m\":{},\"mean_water_depth_m\":{},\"maximum_water_depth_m\":{},\"water_volume_relative_error\":{},\"clamped_sample_count\":{}}},", n(t.minimum_solid_elevation_m), n(t.p05_solid_elevation_m), n(t.median_solid_elevation_m), n(t.p95_solid_elevation_m), n(t.maximum_solid_elevation_m), json_optional_number(t.sea_level_m), n(t.land_area_fraction), n(t.ocean_area_fraction), n(t.mean_land_elevation_m), n(t.mean_water_depth_m), n(t.maximum_water_depth_m), n(t.water_volume_relative_error), t.clamped_sample_count);
        let c = &self.climate;
        let _ = writeln!(out, "  \"climate\": {{\"global_solver_level\":{},\"global_solver_sample_count\":{},\"orbital_phase_count\":{},\"spinup_years\":{},\"minimum_temperature_k\":{},\"mean_temperature_k\":{},\"maximum_temperature_k\":{},\"mean_land_temperature_k\":{},\"mean_ocean_temperature_k\":{},\"mean_wind_speed_m_s\":{},\"maximum_wind_speed_m_s\":{},\"mean_surface_current_m_s\":{},\"maximum_surface_current_m_s\":{},\"mean_sst_k\":{},\"mean_annual_precipitation_mm\":{},\"p95_annual_precipitation_mm\":{},\"precipitation_p95_to_mean_ratio\":{},\"moisture_budget_relative_error\":{},\"moisture_transport_limiter_fraction\":{},\"maximum_moisture_transport_substeps\":{},\"persistent_snow_area_fraction\":{},\"sea_ice_area_fraction\":{},\"final_temperature_rms_change_k\":{}}},", c.global_solver_level, c.global_solver_sample_count, c.orbital_phase_count, c.spinup_years, n(c.minimum_temperature_k), n(c.mean_temperature_k), n(c.maximum_temperature_k), n(c.mean_land_temperature_k), n(c.mean_ocean_temperature_k), n(c.mean_wind_speed_m_s), n(c.maximum_wind_speed_m_s), n(c.mean_surface_current_m_s), n(c.maximum_surface_current_m_s), n(c.mean_sea_surface_temperature_k), n(c.mean_annual_precipitation_mm), n(c.p95_annual_precipitation_mm), n(c.precipitation_p95_to_mean_ratio), n(c.moisture_budget_relative_error), n(c.moisture_transport_limiter_fraction), c.maximum_moisture_transport_substeps, n(c.persistent_snow_area_fraction), n(c.sea_ice_area_fraction), n(c.final_temperature_rms_change_k));
        let h = &self.hydrology;
        let _ = writeln!(out, "  \"hydrology\": {{");
        let _ = writeln!(out, "    \"basin_count\":{}, \"depression_count\":{}, \"largest_contributing_area_km2\":{}, \"largest_basin_area_fraction_of_land\":{}, \"maximum_depression_depth_m\":{}, \"drainage_area_conservation_relative_error\":{},", h.basin_count, h.depression_count, n(h.largest_contributing_area_km2), n(h.largest_basin_area_fraction_of_land), n(h.maximum_depression_depth_m), n(h.drainage_area_conservation_relative_error));
        let _ = writeln!(out, "    \"mean_land_precipitation_mm\":{}, \"mean_land_actual_evapotranspiration_mm\":{}, \"mean_land_runoff_mm\":{}, \"land_runoff_fraction\":{}, \"maximum_potential_discharge_m3_s\":{}, \"runoff_discharge_conservation_relative_error\":{},", n(h.mean_land_precipitation_mm), n(h.mean_land_actual_evapotranspiration_mm), n(h.mean_land_runoff_mm), n(h.land_runoff_fraction), n(h.maximum_potential_discharge_m3_s), n(h.runoff_discharge_conservation_relative_error));
        let _ = writeln!(out, "    \"lake_count\":{}, \"endorheic_lake_count\":{}, \"overflowing_lake_count\":{}, \"terminal_storage_lake_count\":{}, \"total_lake_area_km2\":{}, \"total_lake_volume_km3\":{}, \"largest_lake_area_km2\":{}, \"largest_lake_area_fraction_of_land\":{}, \"maximum_lake_depth_m\":{}, \"lake_water_balance_relative_error\":{},", h.lake_count, h.endorheic_lake_count, h.overflowing_lake_count, h.terminal_storage_lake_count, n(h.total_lake_area_km2), n(h.total_lake_volume_km3), n(h.largest_lake_area_km2), n(h.largest_lake_area_fraction_of_land), n(h.maximum_lake_depth_m), n(h.lake_water_balance_relative_error));
        let _ = writeln!(out, "    \"dry_flow_sample_count\":{}, \"intermittent_flow_sample_count\":{}, \"perennial_flow_sample_count\":{}, \"snowmelt_runoff_fraction\":{}, \"maximum_phase_realized_discharge_m3_s\":{}, \"seasonal_routing_conservation_relative_error\":{}, \"seasonal_water_balance_relative_error\":{}, \"lake_spinup_years\":{}, \"final_lake_surface_cycle_change_m\":{}, \"maximum_seasonal_lake_level_range_m\":{},", h.dry_flow_sample_count, h.intermittent_flow_sample_count, h.perennial_flow_sample_count, n(h.snowmelt_runoff_fraction), n(h.maximum_phase_realized_discharge_m3_s), n(h.seasonal_routing_conservation_relative_error), n(h.seasonal_water_balance_relative_error), h.lake_spinup_years, n(h.final_lake_surface_cycle_change_m), n(h.maximum_seasonal_lake_level_range_m));
        let _ = writeln!(out, "    \"largest_basins\": [");
        for (index, basin) in h.largest_basins.iter().enumerate() {
            let comma = if index + 1 == h.largest_basins.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(out, "      {{\"basin_id\":{},\"outlet_sample\":{},\"outlet_kind\":{},\"sample_count\":{},\"area_km2\":{}}}{comma}", basin.basin_id, basin.outlet_sample, basin.outlet_kind, basin.sample_count, n(basin.area_km2));
        }
        let _ = writeln!(out, "    ],");
        let _ = writeln!(out, "    \"largest_depressions\": [");
        for (index, depression) in h.largest_depressions.iter().enumerate() {
            let comma = if index + 1 == h.largest_depressions.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(out, "      {{\"depression_id\":{},\"floor_sample\":{},\"sample_count\":{},\"area_km2\":{},\"maximum_depth_m\":{},\"floor_elevation_m\":{},\"spill_elevation_m\":{}}}{comma}", depression.depression_id, depression.floor_sample, depression.sample_count, n(depression.area_km2), n(depression.maximum_depth_m), n(depression.floor_elevation_m), n(depression.spill_elevation_m));
        }
        let _ = writeln!(out, "    ],");
        let _ = writeln!(out, "    \"largest_lakes\": [");
        for (index, lake) in h.largest_lakes.iter().enumerate() {
            let comma = if index + 1 == h.largest_lakes.len() {
                ""
            } else {
                ","
            };
            let _ = writeln!(out, "      {{\"lake_id\":{},\"depression_id\":{},\"kind\":{},\"area_km2\":{},\"volume_km3\":{},\"maximum_depth_m\":{},\"surface_elevation_m\":{},\"gross_land_inflow_m3_s\":{},\"lake_evaporation_m3_s\":{},\"outflow_m3_s\":{}}}{comma}", lake.lake_id, lake.depression_id, lake.kind, n(lake.area_km2), n(lake.volume_km3), n(lake.maximum_depth_m), n(lake.surface_elevation_m), n(lake.gross_land_inflow_m3_s), n(lake.lake_evaporation_m3_s), n(lake.outflow_m3_s));
        }
        let _ = writeln!(out, "    ]");
        let _ = writeln!(out, "  }},");
        let g = &self.geomorphology;
        let _ = writeln!(out, "  \"geomorphology\": {{\"erosive_sample_count\":{},\"active_lake_trap_count\":{},\"maximum_effective_discharge_m3_s\":{},\"maximum_channel_slope\":{},\"maximum_channel_width_m\":{},\"maximum_incision_potential_m_per_year\":{},\"total_sediment_generated_kg_s\":{},\"total_land_deposition_kg_s\":{},\"total_lake_deposition_kg_s\":{},\"total_terminal_ocean_deposition_kg_s\":{},\"erosion_sediment_conservation_relative_error\":{},\"geomorphic_duration_years\":{},\"eroded_sample_count\":{},\"depositional_sample_count\":{},\"receiver_changed_sample_count\":{},\"receiver_changed_fraction\":{},\"maximum_applied_erosion_m\":{},\"maximum_applied_deposition_m\":{},\"mean_land_absolute_terrain_change_m\":{},\"evolution_sediment_conservation_relative_error\":{},\"post_erosion_runoff_conservation_relative_error\":{},\"filled_depression_count\":{},\"filled_sample_count\":{},\"capacity_limited_depression_count\":{},\"maximum_lake_fill_depth_m\":{},\"total_historical_lake_delivery_kg_s\":{},\"total_applied_lake_fill_equivalent_kg_s\":{},\"total_unapplied_lake_sediment_kg_s\":{},\"infill_sediment_conservation_relative_error\":{},\"pre_infill_lake_count\":{},\"post_infill_lake_count\":{}}}", g.erosive_sample_count, g.active_lake_trap_count, n(g.maximum_effective_discharge_m3_s), n(g.maximum_channel_slope), n(g.maximum_channel_width_m), n(g.maximum_incision_potential_m_per_year), n(g.total_sediment_generated_kg_s), n(g.total_land_deposition_kg_s), n(g.total_lake_deposition_kg_s), n(g.total_terminal_ocean_deposition_kg_s), n(g.erosion_sediment_conservation_relative_error), n(g.geomorphic_duration_years), g.eroded_sample_count, g.depositional_sample_count, g.receiver_changed_sample_count, n(g.receiver_changed_fraction), n(g.maximum_applied_erosion_m), n(g.maximum_applied_deposition_m), n(g.mean_land_absolute_terrain_change_m), n(g.evolution_sediment_conservation_relative_error), n(g.post_erosion_runoff_conservation_relative_error), g.filled_depression_count, g.filled_sample_count, g.capacity_limited_depression_count, n(g.maximum_lake_fill_depth_m), n(g.total_historical_lake_delivery_kg_s), n(g.total_applied_lake_fill_equivalent_kg_s), n(g.total_unapplied_lake_sediment_kg_s), n(g.infill_sediment_conservation_relative_error), g.pre_infill_lake_count, g.post_infill_lake_count);
        let _ = writeln!(out, "}}");
        out
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# Planet Engine calibration report");
        let _ = writeln!(out, "");
        let _ = writeln!(
            out,
            "Schema: `{}@{}`",
            WORLD_CALIBRATION_SCHEMA_ID, WORLD_CALIBRATION_SCHEMA_VERSION
        );
        let _ = writeln!(
            out,
            "Seed: `{}` · L{} → L{} · {} plates · {} samples · engine v{}",
            self.run.seed,
            self.run.coarse_level,
            self.run.fine_level,
            self.run.plate_count,
            self.run.sample_count,
            self.run.engine_version
        );
        let _ = writeln!(out, "");
        let _ = writeln!(out, "## Continental assembly");
        let _ = writeln!(out, "Significant components: {} · area CV {:.3} · largest/median {:.3}× · max elongation {:.3} · max compactness {:.3} · major multi-plate component: {}", self.continents.significant_component_count, self.continents.component_area_coefficient_of_variation, self.continents.largest_to_median_area_ratio, self.continents.maximum_elongation, self.continents.maximum_compactness, self.continents.has_major_multiplate_component);
        for (index, component) in self.continents.ranked_components.iter().enumerate() {
            let _ = writeln!(out, "{}. {:.3}M km² · diameter {:.0} km · {} plates · elongation {:.3} · compactness {:.3}", index + 1, component.area_km2 / 1.0e6, component.diameter_km, component.plate_count, component.elongation, component.compactness);
        }
        let t = &self.topography;
        let _ = writeln!(out, "\n## Topography");
        let _ = writeln!(out, "Land/ocean: {:.1}% / {:.1}% · mean land elevation {:.0} m · mean/max water depth {:.0}/{:.0} m", t.land_area_fraction * 100.0, t.ocean_area_fraction * 100.0, t.mean_land_elevation_m, t.mean_water_depth_m, t.maximum_water_depth_m);
        let _ = writeln!(out, "Solid elevation P05/P50/P95: {:.0}/{:.0}/{:.0} m · range {:.0} → {:.0} m · water closure {:.3e}", t.p05_solid_elevation_m, t.median_solid_elevation_m, t.p95_solid_elevation_m, t.minimum_solid_elevation_m, t.maximum_solid_elevation_m, t.water_volume_relative_error);
        let c = &self.climate;
        let _ = writeln!(out, "\n## Climate");
        let _ = writeln!(
            out,
            "Temperature {:.1} → {:.1} K · mean {:.1} K · land/ocean {:.1}/{:.1} K · SST {:.1} K",
            c.minimum_temperature_k,
            c.maximum_temperature_k,
            c.mean_temperature_k,
            c.mean_land_temperature_k,
            c.mean_ocean_temperature_k,
            c.mean_sea_surface_temperature_k
        );
        let _ = writeln!(
            out,
            "Precipitation mean/P95 {:.1}/{:.1} mm/yr · P95/mean {:.2}× · moisture closure {:.3e}",
            c.mean_annual_precipitation_mm,
            c.p95_annual_precipitation_mm,
            c.precipitation_p95_to_mean_ratio,
            c.moisture_budget_relative_error
        );
        let h = &self.hydrology;
        let _ = writeln!(out, "\n## Hydrology");
        let _ = writeln!(out, "Basins/depressions: {} / {} · largest contributing area {:.3}M km² · deepest depression {:.1} m · area closure {:.3e}", h.basin_count, h.depression_count, h.largest_contributing_area_km2 / 1.0e6, h.maximum_depression_depth_m, h.drainage_area_conservation_relative_error);
        let _ = writeln!(out, "Land water balance: P {:.1} · AET {:.1} · runoff {:.1} mm/yr · runoff fraction {:.1}% · discharge closure {:.3e}", h.mean_land_precipitation_mm, h.mean_land_actual_evapotranspiration_mm, h.mean_land_runoff_mm, h.land_runoff_fraction * 100.0, h.runoff_discharge_conservation_relative_error);
        let _ = writeln!(out, "Lakes: {} total ({} endorheic / {} overflowing / {} terminal) · area {:.3}M km² · largest {:.0}k km² ({:.2}% of land) · max depth {:.1} m · lake closure {:.3e}", h.lake_count, h.endorheic_lake_count, h.overflowing_lake_count, h.terminal_storage_lake_count, h.total_lake_area_km2 / 1.0e6, h.largest_lake_area_km2 / 1.0e3, h.largest_lake_area_fraction_of_land * 100.0, h.maximum_lake_depth_m, h.lake_water_balance_relative_error);
        let _ = writeln!(out, "Flow regimes: {} dry / {} intermittent / {} perennial · snowmelt {:.2}% · seasonal closure {:.3e}", h.dry_flow_sample_count, h.intermittent_flow_sample_count, h.perennial_flow_sample_count, h.snowmelt_runoff_fraction * 100.0, h.seasonal_water_balance_relative_error);
        if !h.largest_lakes.is_empty() {
            let _ = writeln!(out, "\n### Largest lakes");
            for lake in &h.largest_lakes {
                let _ = writeln!(out, "- lake {} / depression {}: {:.0}k km² · {:.1}k km³ · depth {:.1} m · inflow {:.0} m³/s · evaporation {:.0} m³/s · outflow {:.0} m³/s", lake.lake_id, lake.depression_id, lake.area_km2 / 1.0e3, lake.volume_km3 / 1.0e3, lake.maximum_depth_m, lake.gross_land_inflow_m3_s, lake.lake_evaporation_m3_s, lake.outflow_m3_s);
            }
        }
        if !h.largest_depressions.is_empty() {
            let _ = writeln!(out, "\n### Largest depressions");
            for depression in &h.largest_depressions {
                let _ = writeln!(
                    out,
                    "- depression {}: {:.0}k km² · depth {:.1} m · floor/spill {:.1}/{:.1} m",
                    depression.depression_id,
                    depression.area_km2 / 1.0e3,
                    depression.maximum_depth_m,
                    depression.floor_elevation_m,
                    depression.spill_elevation_m
                );
            }
        }
        let g = &self.geomorphology;
        let _ = writeln!(out, "\n## Geomorphology");
        let _ = writeln!(out, "WG-7A: {} erosive samples · max Q {:.0} m³/s · max slope {:.5} · max width {:.1} m · sediment {:.1} kg/s · closure {:.3e}", g.erosive_sample_count, g.maximum_effective_discharge_m3_s, g.maximum_channel_slope, g.maximum_channel_width_m, g.total_sediment_generated_kg_s, g.erosion_sediment_conservation_relative_error);
        let _ = writeln!(out, "WG-7B: {:.0} years · {} eroded / {} depositional samples · receiver changes {} ({:.3}%) · max erosion/deposition {:.1}/{:.1} m", g.geomorphic_duration_years, g.eroded_sample_count, g.depositional_sample_count, g.receiver_changed_sample_count, g.receiver_changed_fraction * 100.0, g.maximum_applied_erosion_m, g.maximum_applied_deposition_m);
        let _ = writeln!(out, "WG-7D: {} filled depressions / {} samples · max fill {:.1} m · applied/unapplied sediment {:.1}/{:.1} kg/s · lakes {} → {}", g.filled_depression_count, g.filled_sample_count, g.maximum_lake_fill_depth_m, g.total_applied_lake_fill_equivalent_kg_s, g.total_unapplied_lake_sediment_kg_s, g.pre_infill_lake_count, g.post_infill_lake_count);
        let _ = writeln!(out, "\n## Causal identity");
        let _ = writeln!(
            out,
            "`tectonic {}` → `geology {}` → `lithosphere {}` → `topography {}` → `climate {}`",
            self.hashes.tectonic_hash,
            self.hashes.geology_hash,
            self.hashes.lithosphere_hash,
            self.hashes.topography_hash,
            self.hashes.climate_hash
        );
        let _ = writeln!(out, "`drainage {}` → `runoff {}` → `lakes {}` → `seasonal {}` → `erosion {}` → `evolution {}` → `infill {}`", self.hashes.final_drainage_hash, self.hashes.final_runoff_hash, self.hashes.final_lake_hash, self.hashes.final_seasonal_hash, self.hashes.erosion_hash, self.hashes.evolution_hash, self.hashes.infill_hash);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{json_escape, json_number};

    #[test]
    fn calibration_json_helpers_are_stable() {
        assert_eq!(json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\n");
        assert_eq!(json_number(12.5), "12.5");
        assert_eq!(json_number(f64::NAN), "null");
    }
}
