use crate::lakes::generate_lakes_closed_basins_from_surface;
use crate::runoff::rebind_runoff_to_drainage;
use crate::seasonal::generate_seasonal_hydrology_from_surface;
use crate::{
    derive_stage_seed, ClimateGenerationDiagnostics, ClimateState, DrainageState, GeodesicTopology,
    LakeParameters, LakeRequest, LakeState, PlanetPhysicalParameters, PlanetTopology,
    RunoffParameters, RunoffState, SeasonalHydrologyParameters, SeasonalHydrologyRequest,
    SeasonalHydrologyState, StageIdentity, TerrainEvolutionState, TopographyState, WorldgenError,
};

pub const POST_EROSION_HYDROLOGY_STAGE_ID: &str = "hydrology:post-erosion-reconciliation";
pub const POST_EROSION_HYDROLOGY_STAGE_VERSION: u32 = 1;
const POST_EROSION_HYDROLOGY_NAMESPACE: &str = "hydrology:post-erosion-reconciliation:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PostErosionHydrologyParameters {
    pub runoff: RunoffParameters,
    pub lakes: LakeParameters,
    pub seasonal: SeasonalHydrologyParameters,
}

impl Default for PostErosionHydrologyParameters {
    fn default() -> Self {
        Self {
            runoff: RunoffParameters::default(),
            lakes: LakeParameters::default(),
            seasonal: SeasonalHydrologyParameters::default(),
        }
    }
}

impl PostErosionHydrologyParameters {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.runoff.validate()?;
        self.lakes.validate()?;
        self.seasonal.validate()?;
        Ok(())
    }

    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        hash = fnv_update(hash, &self.runoff.parameter_hash().to_le_bytes());
        hash = fnv_update(hash, &self.lakes.parameter_hash().to_le_bytes());
        fnv_update(hash, &self.seasonal.parameter_hash().to_le_bytes())
    }

    pub fn parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.parameter_hash())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PostErosionHydrologyRequest {
    pub seed: String,
    pub parameters: PostErosionHydrologyParameters,
}

impl PostErosionHydrologyRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            parameters: PostErosionHydrologyParameters::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PostErosionHydrologyMetrics {
    pub sample_count: u32,
    pub pre_erosion_lake_count: u32,
    pub post_erosion_lake_count: u32,
    pub lake_kind_changed_sample_count: u32,
    pub lake_added_sample_count: u32,
    pub lake_removed_sample_count: u32,
    pub flow_regime_changed_sample_count: u32,
    pub maximum_absolute_lake_depth_change_m: f64,
    pub maximum_absolute_annual_realized_discharge_change_m3_s: f64,
    pub maximum_absolute_flow_presence_change: f64,
    pub reconciled_runoff_conservation_relative_error: f64,
    pub reconciled_lake_water_balance_relative_error: f64,
    pub reconciled_seasonal_routing_relative_error: f64,
    pub reconciled_seasonal_water_balance_relative_error: f64,
    pub reconciliation_parameter_hash: u64,
    pub topography_hash: u64,
    pub climate_hash: u64,
    pub pre_erosion_drainage_hash: u64,
    pub pre_erosion_runoff_hash: u64,
    pub pre_erosion_lake_hash: u64,
    pub pre_erosion_seasonal_hash: u64,
    pub terrain_evolution_hash: u64,
    pub evolved_surface_hash: u64,
    pub post_erosion_drainage_hash: u64,
    pub reconciled_runoff_hash: u64,
    pub reconciled_lake_hash: u64,
    pub reconciled_seasonal_hash: u64,
    pub post_erosion_hydrology_hash: u64,
}

impl PostErosionHydrologyMetrics {
    pub fn reconciliation_parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.reconciliation_parameter_hash)
    }
    pub fn reconciled_runoff_hash_hex(&self) -> String {
        format!("{:016x}", self.reconciled_runoff_hash)
    }
    pub fn reconciled_lake_hash_hex(&self) -> String {
        format!("{:016x}", self.reconciled_lake_hash)
    }
    pub fn reconciled_seasonal_hash_hex(&self) -> String {
        format!("{:016x}", self.reconciled_seasonal_hash)
    }
    pub fn post_erosion_hydrology_hash_hex(&self) -> String {
        format!("{:016x}", self.post_erosion_hydrology_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PostErosionHydrologyState {
    pub stage: StageIdentity,
    pub metrics: PostErosionHydrologyMetrics,
    pub reconciled_runoff: RunoffState,
    pub reconciled_lakes: LakeState,
    pub reconciled_seasonal: SeasonalHydrologyState,
    pub lake_kind_changed_mask: Vec<u8>,
    pub lake_depth_delta_m: Vec<f32>,
    pub annual_realized_discharge_delta_m3_s: Vec<f32>,
    pub flow_regime_changed_mask: Vec<u8>,
    pub flow_presence_delta: Vec<f32>,
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

fn hash_u8_slice(mut hash: u64, values: &[u8]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    fnv_update(hash, values)
}

#[allow(clippy::too_many_arguments)]
pub fn generate_post_erosion_hydrology(
    topology: &GeodesicTopology,
    topography: &TopographyState,
    climate: &ClimateState,
    climate_diagnostics: &ClimateGenerationDiagnostics,
    pre_erosion_drainage: &DrainageState,
    pre_erosion_runoff: &RunoffState,
    pre_erosion_lakes: &LakeState,
    pre_erosion_seasonal: &SeasonalHydrologyState,
    evolution: &TerrainEvolutionState,
    planet: PlanetPhysicalParameters,
    request: &PostErosionHydrologyRequest,
) -> Result<PostErosionHydrologyState, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    request
        .parameters
        .validate()
        .map_err(WorldgenError::InvalidHydrology)?;

    let count = topology.sample_count() as usize;
    if topography.metrics.sample_count as usize != count
        || climate.metrics.sample_count as usize != count
        || pre_erosion_drainage.metrics.sample_count as usize != count
        || pre_erosion_runoff.metrics.sample_count as usize != count
        || pre_erosion_lakes.metrics.sample_count as usize != count
        || pre_erosion_seasonal.metrics.sample_count as usize != count
        || evolution.metrics.sample_count as usize != count
        || evolution.evolved_solid_elevation_m.len() != count
        || evolution.post_erosion_drainage.metrics.sample_count as usize != count
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-7C inputs must align on the canonical fine topology",
        ));
    }
    if evolution.metrics.topography_hash != topography.metrics.topography_hash
        || evolution.metrics.drainage_hash != pre_erosion_drainage.metrics.drainage_hash
        || evolution.metrics.runoff_hash != pre_erosion_runoff.metrics.runoff_hash
        || evolution.metrics.lake_hash != pre_erosion_lakes.metrics.lake_hash
        || evolution.metrics.evolved_surface_hash == 0
        || evolution.metrics.post_erosion_drainage_hash
            != evolution.post_erosion_drainage.metrics.drainage_hash
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-7C requires exact accepted WG-4/WG-6/WG-7B ancestry",
        ));
    }
    if pre_erosion_runoff.metrics.climate_hash != climate.metrics.climate_hash
        || pre_erosion_runoff.metrics.drainage_hash != pre_erosion_drainage.metrics.drainage_hash
        || pre_erosion_lakes.metrics.climate_hash != climate.metrics.climate_hash
        || pre_erosion_lakes.metrics.drainage_hash != pre_erosion_drainage.metrics.drainage_hash
        || pre_erosion_lakes.metrics.runoff_hash != pre_erosion_runoff.metrics.runoff_hash
        || pre_erosion_seasonal.metrics.climate_hash != climate.metrics.climate_hash
        || pre_erosion_seasonal.metrics.drainage_hash != pre_erosion_drainage.metrics.drainage_hash
        || pre_erosion_seasonal.metrics.runoff_hash != pre_erosion_runoff.metrics.runoff_hash
        || pre_erosion_seasonal.metrics.lake_hash != pre_erosion_lakes.metrics.lake_hash
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-7C pre-erosion hydrology must share one accepted ancestry",
        ));
    }
    if request.parameters.runoff.parameter_hash()
        != pre_erosion_runoff.metrics.runoff_parameter_hash
        || request.parameters.lakes.parameter_hash()
            != pre_erosion_lakes.metrics.lake_parameter_hash
        || request.parameters.seasonal.parameter_hash()
            != pre_erosion_seasonal.metrics.seasonal_parameter_hash
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-7C parameters must match the accepted pre-erosion hydrology",
        ));
    }

    let reconciled_runoff = rebind_runoff_to_drainage(
        pre_erosion_runoff,
        &topography.submerged_mask,
        &evolution.post_erosion_drainage,
        planet,
    )?;
    if reconciled_runoff.local_runoff_m3_s != pre_erosion_runoff.local_runoff_m3_s {
        return Err(WorldgenError::InvalidHydrology(
            "WG-7C must preserve accepted local runoff exactly",
        ));
    }
    if reconciled_runoff.potential_discharge_m3_s != evolution.post_erosion_potential_discharge_m3_s
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-7C runoff must match WG-7B post-erosion routing exactly",
        ));
    }

    let lake_request = LakeRequest {
        seed: request.seed.clone(),
        parameters: request.parameters.lakes,
    };
    let reconciled_lakes = generate_lakes_closed_basins_from_surface(
        topology,
        &evolution.evolved_solid_elevation_m,
        &topography.submerged_mask,
        climate,
        &evolution.post_erosion_drainage,
        &reconciled_runoff,
        planet,
        &lake_request,
    )?;
    if reconciled_lakes.stage.derived_seed != pre_erosion_lakes.stage.derived_seed {
        return Err(WorldgenError::InvalidHydrology(
            "WG-7C lake reconciliation seed must match accepted ancestry",
        ));
    }

    let seasonal_request = SeasonalHydrologyRequest {
        seed: request.seed.clone(),
        parameters: request.parameters.seasonal,
    };
    let reconciled_seasonal = generate_seasonal_hydrology_from_surface(
        topology,
        &evolution.evolved_solid_elevation_m,
        &topography.submerged_mask,
        climate,
        climate_diagnostics,
        &evolution.post_erosion_drainage,
        &reconciled_runoff,
        &reconciled_lakes,
        planet,
        &seasonal_request,
    )?;
    if reconciled_seasonal.stage.derived_seed != pre_erosion_seasonal.stage.derived_seed {
        return Err(WorldgenError::InvalidHydrology(
            "WG-7C seasonal reconciliation seed must match accepted ancestry",
        ));
    }

    let mut lake_kind_changed_mask = vec![0_u8; count];
    let mut lake_depth_delta_m = vec![0.0_f32; count];
    let mut annual_realized_discharge_delta_m3_s = vec![0.0_f32; count];
    let mut flow_regime_changed_mask = vec![0_u8; count];
    let mut flow_presence_delta = vec![0.0_f32; count];
    let mut lake_kind_changed_sample_count = 0_u32;
    let mut lake_added_sample_count = 0_u32;
    let mut lake_removed_sample_count = 0_u32;
    let mut flow_regime_changed_sample_count = 0_u32;
    let mut maximum_absolute_lake_depth_change_m = 0.0_f64;
    let mut maximum_absolute_annual_realized_discharge_change_m3_s = 0.0_f64;
    let mut maximum_absolute_flow_presence_change = 0.0_f64;

    for i in 0..count {
        let before_kind = pre_erosion_lakes.lake_kind[i];
        let after_kind = reconciled_lakes.lake_kind[i];
        if before_kind != after_kind {
            lake_kind_changed_mask[i] = 1;
            lake_kind_changed_sample_count += 1;
            if before_kind == 0 && after_kind != 0 {
                lake_added_sample_count += 1;
            } else if before_kind != 0 && after_kind == 0 {
                lake_removed_sample_count += 1;
            }
        }

        let depth_delta = reconciled_lakes.lake_depth_m[i] - pre_erosion_lakes.lake_depth_m[i];
        lake_depth_delta_m[i] = depth_delta;
        maximum_absolute_lake_depth_change_m =
            maximum_absolute_lake_depth_change_m.max(f64::from(depth_delta).abs());

        let realized_delta = reconciled_lakes.realized_discharge_m3_s[i]
            - pre_erosion_lakes.realized_discharge_m3_s[i];
        annual_realized_discharge_delta_m3_s[i] = realized_delta;
        maximum_absolute_annual_realized_discharge_change_m3_s =
            maximum_absolute_annual_realized_discharge_change_m3_s
                .max(f64::from(realized_delta).abs());

        if pre_erosion_seasonal.flow_regime[i] != reconciled_seasonal.flow_regime[i] {
            flow_regime_changed_mask[i] = 1;
            flow_regime_changed_sample_count += 1;
        }
        let presence_delta = reconciled_seasonal.flow_presence_fraction[i]
            - pre_erosion_seasonal.flow_presence_fraction[i];
        flow_presence_delta[i] = presence_delta;
        maximum_absolute_flow_presence_change =
            maximum_absolute_flow_presence_change.max(f64::from(presence_delta).abs());
    }

    let stage_seed = derive_stage_seed(&request.seed, POST_EROSION_HYDROLOGY_NAMESPACE);
    let reconciliation_parameter_hash = request.parameters.parameter_hash();
    let mut post_erosion_hydrology_hash = FNV_OFFSET_BASIS;
    post_erosion_hydrology_hash = fnv_update(
        post_erosion_hydrology_hash,
        POST_EROSION_HYDROLOGY_STAGE_ID.as_bytes(),
    );
    post_erosion_hydrology_hash = fnv_update(
        post_erosion_hydrology_hash,
        &POST_EROSION_HYDROLOGY_STAGE_VERSION.to_le_bytes(),
    );
    post_erosion_hydrology_hash =
        fnv_update(post_erosion_hydrology_hash, &stage_seed.to_le_bytes());
    post_erosion_hydrology_hash = fnv_update(
        post_erosion_hydrology_hash,
        &planet.parameter_hash().to_le_bytes(),
    );
    post_erosion_hydrology_hash = fnv_update(
        post_erosion_hydrology_hash,
        &reconciliation_parameter_hash.to_le_bytes(),
    );
    for identity in [
        topography.metrics.topography_hash,
        climate.metrics.climate_hash,
        pre_erosion_drainage.metrics.drainage_hash,
        pre_erosion_runoff.metrics.runoff_hash,
        pre_erosion_lakes.metrics.lake_hash,
        pre_erosion_seasonal.metrics.seasonal_hydrology_hash,
        evolution.metrics.terrain_evolution_hash,
        evolution.metrics.evolved_surface_hash,
        evolution.post_erosion_drainage.metrics.drainage_hash,
        reconciled_runoff.metrics.runoff_hash,
        reconciled_lakes.metrics.lake_hash,
        reconciled_seasonal.metrics.seasonal_hydrology_hash,
    ] {
        post_erosion_hydrology_hash =
            fnv_update(post_erosion_hydrology_hash, &identity.to_le_bytes());
    }
    post_erosion_hydrology_hash =
        hash_u8_slice(post_erosion_hydrology_hash, &lake_kind_changed_mask);
    post_erosion_hydrology_hash = hash_f32_slice(post_erosion_hydrology_hash, &lake_depth_delta_m);
    post_erosion_hydrology_hash = hash_f32_slice(
        post_erosion_hydrology_hash,
        &annual_realized_discharge_delta_m3_s,
    );
    post_erosion_hydrology_hash =
        hash_u8_slice(post_erosion_hydrology_hash, &flow_regime_changed_mask);
    post_erosion_hydrology_hash = hash_f32_slice(post_erosion_hydrology_hash, &flow_presence_delta);

    let metrics = PostErosionHydrologyMetrics {
        sample_count: count as u32,
        pre_erosion_lake_count: pre_erosion_lakes.metrics.lake_count,
        post_erosion_lake_count: reconciled_lakes.metrics.lake_count,
        lake_kind_changed_sample_count,
        lake_added_sample_count,
        lake_removed_sample_count,
        flow_regime_changed_sample_count,
        maximum_absolute_lake_depth_change_m,
        maximum_absolute_annual_realized_discharge_change_m3_s,
        maximum_absolute_flow_presence_change,
        reconciled_runoff_conservation_relative_error: reconciled_runoff
            .metrics
            .discharge_conservation_relative_error,
        reconciled_lake_water_balance_relative_error: reconciled_lakes
            .metrics
            .water_balance_relative_error,
        reconciled_seasonal_routing_relative_error: reconciled_seasonal
            .metrics
            .seasonal_routing_conservation_relative_error,
        reconciled_seasonal_water_balance_relative_error: reconciled_seasonal
            .metrics
            .seasonal_water_balance_relative_error,
        reconciliation_parameter_hash,
        topography_hash: topography.metrics.topography_hash,
        climate_hash: climate.metrics.climate_hash,
        pre_erosion_drainage_hash: pre_erosion_drainage.metrics.drainage_hash,
        pre_erosion_runoff_hash: pre_erosion_runoff.metrics.runoff_hash,
        pre_erosion_lake_hash: pre_erosion_lakes.metrics.lake_hash,
        pre_erosion_seasonal_hash: pre_erosion_seasonal.metrics.seasonal_hydrology_hash,
        terrain_evolution_hash: evolution.metrics.terrain_evolution_hash,
        evolved_surface_hash: evolution.metrics.evolved_surface_hash,
        post_erosion_drainage_hash: evolution.post_erosion_drainage.metrics.drainage_hash,
        reconciled_runoff_hash: reconciled_runoff.metrics.runoff_hash,
        reconciled_lake_hash: reconciled_lakes.metrics.lake_hash,
        reconciled_seasonal_hash: reconciled_seasonal.metrics.seasonal_hydrology_hash,
        post_erosion_hydrology_hash,
    };

    Ok(PostErosionHydrologyState {
        stage: StageIdentity {
            id: POST_EROSION_HYDROLOGY_STAGE_ID,
            version: POST_EROSION_HYDROLOGY_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics,
        reconciled_runoff,
        reconciled_lakes,
        reconciled_seasonal,
        lake_kind_changed_mask,
        lake_depth_delta_m,
        annual_realized_discharge_delta_m3_s,
        flow_regime_changed_mask,
        flow_presence_delta,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_hash_tracks_nested_hydrology_parameters() {
        let base = PostErosionHydrologyParameters::default();
        let mut changed = base;
        changed.lakes.open_water_evaporation_scale = 1.1;
        assert_ne!(base.parameter_hash(), changed.parameter_hash());
        changed = base;
        changed.runoff.budyko_omega = 2.8;
        assert_ne!(base.parameter_hash(), changed.parameter_hash());
        changed = base;
        changed.seasonal.degree_day_melt_mm_per_k_day = 4.0;
        assert_ne!(base.parameter_hash(), changed.parameter_hash());
    }
}
