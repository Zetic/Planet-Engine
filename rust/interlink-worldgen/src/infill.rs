use crate::drainage::generate_drainage_from_surface;
use crate::evolution::reconstruct_applied_lake_sediment_delivery_kg_s;
use crate::lakes::generate_lakes_closed_basins_from_surface;
use crate::runoff::rebind_runoff_to_drainage;
use crate::seasonal::generate_seasonal_hydrology_from_surface;
use crate::{
    derive_stage_seed, ClimateGenerationDiagnostics, ClimateState, DrainageRequest, DrainageState,
    FluvialErosionState, GeodesicTopology, LakeRequest, LakeState, PlanetPhysicalParameters,
    PlanetTopology, PostErosionHydrologyParameters, PostErosionHydrologyState, RunoffState,
    SeasonalHydrologyRequest, SeasonalHydrologyState, StageIdentity, TerrainEvolutionState,
    TopographyState, WorldgenError, INVALID_SAMPLE_ID,
};

pub const LAKE_SEDIMENT_INFILL_STAGE_ID: &str = "geomorphology:lake-sediment-infill";
pub const LAKE_SEDIMENT_INFILL_STAGE_VERSION: u32 = 1;
const LAKE_SEDIMENT_INFILL_NAMESPACE: &str = "geomorphology:lake-sediment-infill:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const DELIVERY_EPSILON_KG_S: f64 = 1.0e-12;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LakeSedimentInfillParameters {
    /// Maximum amount a historical lake-floor sample may be raised in this direct solve.
    pub maximum_fill_depth_m: f64,
    /// Bulk density used to convert accepted historical lake-sink mass to deposited volume.
    pub deposited_sediment_density_kg_m3: f64,
    /// Hydrology parameters must exactly match the accepted WG-7C reconciliation contract.
    pub hydrology: PostErosionHydrologyParameters,
}

impl Default for LakeSedimentInfillParameters {
    fn default() -> Self {
        Self {
            maximum_fill_depth_m: 120.0,
            deposited_sediment_density_kg_m3: 1_800.0,
            hydrology: PostErosionHydrologyParameters::default(),
        }
    }
}

impl LakeSedimentInfillParameters {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.maximum_fill_depth_m.is_finite()
            || self.maximum_fill_depth_m <= 0.0
            || self.maximum_fill_depth_m > 5_000.0
        {
            return Err("WG-7D maximum lake fill depth must be finite and within (0, 5000]");
        }
        if !self.deposited_sediment_density_kg_m3.is_finite()
            || self.deposited_sediment_density_kg_m3 <= 0.0
            || self.deposited_sediment_density_kg_m3 > 10_000.0
        {
            return Err("WG-7D deposited sediment density must be finite and within (0, 10000]");
        }
        self.hydrology.validate()?;
        Ok(())
    }

    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        hash = fnv_update(hash, &self.maximum_fill_depth_m.to_bits().to_le_bytes());
        hash = fnv_update(
            hash,
            &self
                .deposited_sediment_density_kg_m3
                .to_bits()
                .to_le_bytes(),
        );
        fnv_update(hash, &self.hydrology.parameter_hash().to_le_bytes())
    }

    pub fn parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.parameter_hash())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LakeSedimentInfillRequest {
    pub seed: String,
    pub parameters: LakeSedimentInfillParameters,
}

impl LakeSedimentInfillRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            parameters: LakeSedimentInfillParameters::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LakeSedimentInfillMetrics {
    pub sample_count: u32,
    pub geomorphic_duration_years: f64,
    pub historical_lake_trap_count: u32,
    pub filled_depression_count: u32,
    pub filled_sample_count: u32,
    pub capacity_limited_depression_count: u32,
    pub maximum_fill_depth_m: f64,
    pub total_historical_lake_delivery_kg_s: f64,
    pub total_applied_lake_fill_equivalent_kg_s: f64,
    pub total_unapplied_lake_sediment_kg_s: f64,
    pub total_applied_lake_fill_volume_m3: f64,
    pub sediment_conservation_relative_error: f64,
    pub pre_infill_lake_count: u32,
    pub post_infill_lake_count: u32,
    pub post_infill_runoff_conservation_relative_error: f64,
    pub post_infill_lake_water_balance_relative_error: f64,
    pub post_infill_seasonal_routing_relative_error: f64,
    pub post_infill_seasonal_water_balance_relative_error: f64,
    pub infill_parameter_hash: u64,
    pub topography_hash: u64,
    pub climate_hash: u64,
    pub pre_erosion_drainage_hash: u64,
    pub pre_erosion_lake_hash: u64,
    pub fluvial_erosion_hash: u64,
    pub terrain_evolution_hash: u64,
    pub post_erosion_hydrology_hash: u64,
    pub input_evolved_surface_hash: u64,
    pub post_infill_surface_hash: u64,
    pub post_infill_drainage_hash: u64,
    pub post_infill_runoff_hash: u64,
    pub post_infill_lake_hash: u64,
    pub post_infill_seasonal_hash: u64,
    pub lake_sediment_infill_hash: u64,
}

impl LakeSedimentInfillMetrics {
    pub fn infill_parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.infill_parameter_hash)
    }
    pub fn post_infill_surface_hash_hex(&self) -> String {
        format!("{:016x}", self.post_infill_surface_hash)
    }
    pub fn post_infill_drainage_hash_hex(&self) -> String {
        format!("{:016x}", self.post_infill_drainage_hash)
    }
    pub fn lake_sediment_infill_hash_hex(&self) -> String {
        format!("{:016x}", self.lake_sediment_infill_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LakeSedimentInfillState {
    pub stage: StageIdentity,
    pub metrics: LakeSedimentInfillMetrics,
    /// Distinct WG-7D solid surface. WG-4 and WG-7B remain immutable historical states.
    pub post_infill_solid_elevation_m: Vec<f32>,
    pub lake_fill_depth_m: Vec<f32>,
    pub post_infill_drainage: DrainageState,
    pub reconciled_runoff: RunoffState,
    pub reconciled_lakes: LakeState,
    pub reconciled_seasonal: SeasonalHydrologyState,
}

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

fn surface_hash(values: &[f32]) -> u64 {
    hash_f32_slice(FNV_OFFSET_BASIS, values)
}

fn volume_to_level(
    members: &[usize],
    elevation_m: &[f32],
    topology: &GeodesicTopology,
    radius_m: f64,
    level_m: f64,
) -> f64 {
    members
        .iter()
        .map(|&sample| {
            let elevation = f64::from(elevation_m[sample]);
            if elevation >= level_m {
                0.0
            } else {
                topology.area_steradians(sample as u32)
                    * radius_m
                    * radius_m
                    * (level_m - elevation)
            }
        })
        .sum()
}

fn solve_fill_level(
    members: &[usize],
    elevation_m: &[f32],
    topology: &GeodesicTopology,
    radius_m: f64,
    lower_m: f64,
    upper_m: f64,
    target_volume_m3: f64,
) -> f64 {
    if target_volume_m3 <= 0.0 || upper_m <= lower_m {
        return lower_m;
    }
    let capacity = volume_to_level(members, elevation_m, topology, radius_m, upper_m);
    if target_volume_m3 >= capacity {
        return upper_m;
    }
    let mut lower = lower_m;
    let mut upper = upper_m;
    for _ in 0..52 {
        let middle = 0.5 * (lower + upper);
        if volume_to_level(members, elevation_m, topology, radius_m, middle) < target_volume_m3 {
            lower = middle;
        } else {
            upper = middle;
        }
    }
    0.5 * (lower + upper)
}

#[allow(clippy::too_many_arguments)]
pub fn generate_lake_sediment_infill(
    topology: &GeodesicTopology,
    topography: &TopographyState,
    climate: &ClimateState,
    climate_diagnostics: &ClimateGenerationDiagnostics,
    pre_erosion_drainage: &DrainageState,
    pre_erosion_lakes: &LakeState,
    erosion: &FluvialErosionState,
    evolution: &TerrainEvolutionState,
    reconciliation: &PostErosionHydrologyState,
    planet: PlanetPhysicalParameters,
    request: &LakeSedimentInfillRequest,
) -> Result<LakeSedimentInfillState, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    request
        .parameters
        .validate()
        .map_err(WorldgenError::InvalidGeomorphology)?;

    let count = topology.sample_count() as usize;
    if topography.metrics.sample_count as usize != count
        || climate.metrics.sample_count as usize != count
        || pre_erosion_drainage.metrics.sample_count as usize != count
        || pre_erosion_lakes.metrics.sample_count as usize != count
        || erosion.metrics.sample_count as usize != count
        || evolution.metrics.sample_count as usize != count
        || reconciliation.metrics.sample_count as usize != count
        || evolution.evolved_solid_elevation_m.len() != count
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7D inputs must align on the canonical fine topology",
        ));
    }
    if evolution.metrics.topography_hash != topography.metrics.topography_hash
        || evolution.metrics.drainage_hash != pre_erosion_drainage.metrics.drainage_hash
        || evolution.metrics.lake_hash != pre_erosion_lakes.metrics.lake_hash
        || evolution.metrics.fluvial_erosion_hash != erosion.metrics.fluvial_erosion_hash
        || reconciliation.metrics.topography_hash != topography.metrics.topography_hash
        || reconciliation.metrics.climate_hash != climate.metrics.climate_hash
        || reconciliation.metrics.terrain_evolution_hash != evolution.metrics.terrain_evolution_hash
        || reconciliation.metrics.evolved_surface_hash != evolution.metrics.evolved_surface_hash
        || reconciliation.metrics.post_erosion_drainage_hash
            != evolution.post_erosion_drainage.metrics.drainage_hash
        || reconciliation.metrics.reconciled_runoff_hash
            != reconciliation.reconciled_runoff.metrics.runoff_hash
        || reconciliation.metrics.reconciled_lake_hash
            != reconciliation.reconciled_lakes.metrics.lake_hash
        || reconciliation.metrics.reconciled_seasonal_hash
            != reconciliation
                .reconciled_seasonal
                .metrics
                .seasonal_hydrology_hash
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7D requires exact accepted WG-4/WG-5/WG-7A/WG-7B/WG-7C ancestry",
        ));
    }
    if request.parameters.hydrology.parameter_hash()
        != reconciliation.metrics.reconciliation_parameter_hash
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7D hydrology parameters must match accepted WG-7C reconciliation",
        ));
    }

    let delivery_kg_s = reconstruct_applied_lake_sediment_delivery_kg_s(
        topography,
        pre_erosion_drainage,
        pre_erosion_lakes,
        erosion,
        evolution,
    )?;
    if delivery_kg_s.len() != pre_erosion_drainage.depressions.len() {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7D reconstructed lake delivery does not align with accepted depressions",
        ));
    }

    let mut members_by_depression =
        vec![Vec::<usize>::new(); pre_erosion_drainage.depressions.len()];
    for (sample, depression_id) in pre_erosion_drainage
        .depression_id
        .iter()
        .copied()
        .enumerate()
    {
        if depression_id == INVALID_SAMPLE_ID {
            continue;
        }
        let depression = depression_id as usize;
        if depression >= members_by_depression.len() {
            return Err(WorldgenError::InvalidGeomorphology(
                "WG-7D accepted depression membership points outside depression table",
            ));
        }
        members_by_depression[depression].push(sample);
    }

    let p = request.parameters;
    let duration_seconds = evolution.metrics.geomorphic_duration_years * planet.orbital_period_s;
    let density = p.deposited_sediment_density_kg_m3;
    let mut post_infill_solid_elevation_m = evolution.evolved_solid_elevation_m.clone();
    let mut lake_fill_depth_m = vec![0.0_f32; count];
    let mut historical_lake_trap_count = 0_u32;
    let mut filled_depression_count = 0_u32;
    let mut capacity_limited_depression_count = 0_u32;

    for (depression_index, &delivery) in delivery_kg_s.iter().enumerate() {
        if delivery <= DELIVERY_EPSILON_KG_S {
            continue;
        }
        historical_lake_trap_count += 1;
        if duration_seconds <= 0.0 {
            continue;
        }
        let depression = &pre_erosion_drainage.depressions[depression_index];
        let members: Vec<usize> = members_by_depression[depression_index]
            .iter()
            .copied()
            .filter(|&sample| topography.submerged_mask[sample] == 0)
            .collect();
        if members.is_empty() {
            continue;
        }
        let floor = members
            .iter()
            .map(|&sample| f64::from(evolution.evolved_solid_elevation_m[sample]))
            .fold(f64::INFINITY, f64::min);
        if !floor.is_finite() {
            continue;
        }
        let depth_ceiling = floor + p.maximum_fill_depth_m;
        let upper = if depression.spill_elevation_m.is_finite() {
            depression.spill_elevation_m.min(depth_ceiling)
        } else {
            depth_ceiling
        };
        if upper <= floor {
            continue;
        }
        let capacity_m3 = volume_to_level(
            &members,
            &evolution.evolved_solid_elevation_m,
            topology,
            planet.radius_m,
            upper,
        );
        if capacity_m3 <= 0.0 {
            continue;
        }
        let delivered_volume_m3 = delivery * duration_seconds / density;
        if delivered_volume_m3 > capacity_m3 * (1.0 + 1.0e-12) {
            capacity_limited_depression_count += 1;
        }
        let applied_target_m3 = delivered_volume_m3.min(capacity_m3);
        let fill_level = solve_fill_level(
            &members,
            &evolution.evolved_solid_elevation_m,
            topology,
            planet.radius_m,
            floor,
            upper,
            applied_target_m3,
        );
        let mut changed = false;
        for &sample in &members {
            let old = evolution.evolved_solid_elevation_m[sample];
            let old_f64 = f64::from(old);
            if old_f64 >= fill_level {
                continue;
            }
            let new = fill_level as f32;
            if new <= old {
                continue;
            }
            post_infill_solid_elevation_m[sample] = new;
            lake_fill_depth_m[sample] = new - old;
            changed = true;
        }
        if changed {
            filled_depression_count += 1;
        }
    }

    let mut filled_sample_count = 0_u32;
    let mut maximum_fill_depth_m = 0.0_f64;
    let mut total_applied_lake_fill_volume_m3 = 0.0_f64;
    for (sample, &fill) in lake_fill_depth_m.iter().enumerate() {
        if fill <= 0.0 {
            continue;
        }
        filled_sample_count += 1;
        maximum_fill_depth_m = maximum_fill_depth_m.max(f64::from(fill));
        let area_m2 = topology.area_steradians(sample as u32) * planet.radius_m * planet.radius_m;
        total_applied_lake_fill_volume_m3 += area_m2 * f64::from(fill);
    }
    if maximum_fill_depth_m > p.maximum_fill_depth_m + 1.0e-3 {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7D applied fill exceeds configured fill-depth bound",
        ));
    }

    let total_historical_lake_delivery_kg_s: f64 = delivery_kg_s.iter().sum();
    let total_applied_lake_fill_equivalent_kg_s = if duration_seconds > 0.0 {
        total_applied_lake_fill_volume_m3 * density / duration_seconds
    } else {
        0.0
    };
    let total_unapplied_lake_sediment_kg_s =
        (total_historical_lake_delivery_kg_s - total_applied_lake_fill_equivalent_kg_s).max(0.0);
    let sediment_conservation_relative_error = if total_historical_lake_delivery_kg_s > 0.0 {
        ((total_applied_lake_fill_equivalent_kg_s + total_unapplied_lake_sediment_kg_s)
            - total_historical_lake_delivery_kg_s)
            .abs()
            / total_historical_lake_delivery_kg_s
    } else {
        total_applied_lake_fill_equivalent_kg_s.abs()
    };
    if sediment_conservation_relative_error > 1.0e-5 {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7D lake-infill sediment ledger does not conserve accepted lake delivery",
        ));
    }

    let post_infill_surface_hash = surface_hash(&post_infill_solid_elevation_m);
    let drainage_request = DrainageRequest::new(request.seed.clone());
    let post_infill_drainage = generate_drainage_from_surface(
        topology,
        &post_infill_solid_elevation_m,
        &topography.submerged_mask,
        topography.metrics.sea_level_m,
        post_infill_surface_hash,
        planet,
        &drainage_request,
    )?;

    let reconciled_runoff = rebind_runoff_to_drainage(
        &reconciliation.reconciled_runoff,
        &topography.submerged_mask,
        &post_infill_drainage,
        planet,
    )?;
    if reconciled_runoff.local_runoff_m3_s != reconciliation.reconciled_runoff.local_runoff_m3_s {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7D must preserve accepted local runoff exactly",
        ));
    }

    let lake_request = LakeRequest {
        seed: request.seed.clone(),
        parameters: p.hydrology.lakes,
    };
    let reconciled_lakes = generate_lakes_closed_basins_from_surface(
        topology,
        &post_infill_solid_elevation_m,
        &topography.submerged_mask,
        climate,
        &post_infill_drainage,
        &reconciled_runoff,
        planet,
        &lake_request,
    )?;
    let seasonal_request = SeasonalHydrologyRequest {
        seed: request.seed.clone(),
        parameters: p.hydrology.seasonal,
    };
    let reconciled_seasonal = generate_seasonal_hydrology_from_surface(
        topology,
        &post_infill_solid_elevation_m,
        &topography.submerged_mask,
        climate,
        climate_diagnostics,
        &post_infill_drainage,
        &reconciled_runoff,
        &reconciled_lakes,
        planet,
        &seasonal_request,
        p.hydrology.maximum_lake_spinup_years,
    )?;

    let stage_seed = derive_stage_seed(&request.seed, LAKE_SEDIMENT_INFILL_NAMESPACE);
    let infill_parameter_hash = p.parameter_hash();
    let mut lake_sediment_infill_hash = FNV_OFFSET_BASIS;
    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        LAKE_SEDIMENT_INFILL_STAGE_ID.as_bytes(),
    );
    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        &LAKE_SEDIMENT_INFILL_STAGE_VERSION.to_le_bytes(),
    );
    lake_sediment_infill_hash = fnv_update(lake_sediment_infill_hash, &stage_seed.to_le_bytes());
    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        &infill_parameter_hash.to_le_bytes(),
    );
    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        &evolution.metrics.terrain_evolution_hash.to_le_bytes(),
    );
    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        &reconciliation
            .metrics
            .post_erosion_hydrology_hash
            .to_le_bytes(),
    );
    lake_sediment_infill_hash =
        hash_f32_slice(lake_sediment_infill_hash, &post_infill_solid_elevation_m);
    lake_sediment_infill_hash = hash_f32_slice(lake_sediment_infill_hash, &lake_fill_depth_m);
    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        &post_infill_drainage.metrics.drainage_hash.to_le_bytes(),
    );
    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        &reconciled_runoff.metrics.runoff_hash.to_le_bytes(),
    );
    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        &reconciled_lakes.metrics.lake_hash.to_le_bytes(),
    );
    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        &reconciled_seasonal
            .metrics
            .seasonal_hydrology_hash
            .to_le_bytes(),
    );

    Ok(LakeSedimentInfillState {
        stage: StageIdentity {
            id: LAKE_SEDIMENT_INFILL_STAGE_ID,
            version: LAKE_SEDIMENT_INFILL_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics: LakeSedimentInfillMetrics {
            sample_count: topology.sample_count(),
            geomorphic_duration_years: evolution.metrics.geomorphic_duration_years,
            historical_lake_trap_count,
            filled_depression_count,
            filled_sample_count,
            capacity_limited_depression_count,
            maximum_fill_depth_m,
            total_historical_lake_delivery_kg_s,
            total_applied_lake_fill_equivalent_kg_s,
            total_unapplied_lake_sediment_kg_s,
            total_applied_lake_fill_volume_m3,
            sediment_conservation_relative_error,
            pre_infill_lake_count: reconciliation.reconciled_lakes.metrics.lake_count,
            post_infill_lake_count: reconciled_lakes.metrics.lake_count,
            post_infill_runoff_conservation_relative_error: reconciled_runoff
                .metrics
                .discharge_conservation_relative_error,
            post_infill_lake_water_balance_relative_error: reconciled_lakes
                .metrics
                .water_balance_relative_error,
            post_infill_seasonal_routing_relative_error: reconciled_seasonal
                .metrics
                .seasonal_routing_conservation_relative_error,
            post_infill_seasonal_water_balance_relative_error: reconciled_seasonal
                .metrics
                .seasonal_water_balance_relative_error,
            infill_parameter_hash,
            topography_hash: topography.metrics.topography_hash,
            climate_hash: climate.metrics.climate_hash,
            pre_erosion_drainage_hash: pre_erosion_drainage.metrics.drainage_hash,
            pre_erosion_lake_hash: pre_erosion_lakes.metrics.lake_hash,
            fluvial_erosion_hash: erosion.metrics.fluvial_erosion_hash,
            terrain_evolution_hash: evolution.metrics.terrain_evolution_hash,
            post_erosion_hydrology_hash: reconciliation.metrics.post_erosion_hydrology_hash,
            input_evolved_surface_hash: evolution.metrics.evolved_surface_hash,
            post_infill_surface_hash,
            post_infill_drainage_hash: post_infill_drainage.metrics.drainage_hash,
            post_infill_runoff_hash: reconciled_runoff.metrics.runoff_hash,
            post_infill_lake_hash: reconciled_lakes.metrics.lake_hash,
            post_infill_seasonal_hash: reconciled_seasonal.metrics.seasonal_hydrology_hash,
            lake_sediment_infill_hash,
        },
        post_infill_solid_elevation_m,
        lake_fill_depth_m,
        post_infill_drainage,
        reconciled_runoff,
        reconciled_lakes,
        reconciled_seasonal,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_parameter_hash_changes_with_depth_bound() {
        let a = LakeSedimentInfillParameters::default();
        let mut b = a;
        b.maximum_fill_depth_m = 60.0;
        assert_ne!(a.parameter_hash(), b.parameter_hash());
    }

    #[test]
    fn fill_parameters_reject_invalid_depth() {
        let mut p = LakeSedimentInfillParameters::default();
        p.maximum_fill_depth_m = 0.0;
        assert!(p.validate().is_err());
    }
}
