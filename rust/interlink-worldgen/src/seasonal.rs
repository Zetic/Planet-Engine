use crate::{
    derive_stage_seed, ClimateGenerationDiagnostics, ClimateState, DrainageState,
    GeodesicTopology, PlanetPhysicalParameters, PlanetTopology, RunoffState, StageIdentity,
    TopographyState, WorldgenError, INVALID_SAMPLE_ID,
};

pub const SEASONAL_HYDROLOGY_STAGE_ID: &str = "hydrology:seasonal-hydrology";
pub const SEASONAL_HYDROLOGY_STAGE_VERSION: u32 = 1;
const SEASONAL_HYDROLOGY_NAMESPACE: &str = "hydrology:seasonal-hydrology:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const SECONDS_PER_DAY: f64 = 86_400.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeasonalHydrologyParameters {
    pub snow_temperature_k: f64,
    pub snow_transition_k: f64,
    pub melt_temperature_k: f64,
    pub degree_day_melt_mm_per_k_day: f64,
}

impl Default for SeasonalHydrologyParameters {
    fn default() -> Self {
        Self {
            snow_temperature_k: 273.15,
            snow_transition_k: 2.0,
            melt_temperature_k: 273.15,
            degree_day_melt_mm_per_k_day: 3.0,
        }
    }
}

impl SeasonalHydrologyParameters {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.snow_temperature_k.is_finite()
            || !(180.0..=330.0).contains(&self.snow_temperature_k)
        {
            return Err("seasonal hydrology snow temperature must be finite and within [180, 330] K");
        }
        if !self.snow_transition_k.is_finite()
            || self.snow_transition_k <= 0.0
            || self.snow_transition_k > 20.0
        {
            return Err("seasonal hydrology snow transition must be finite and within (0, 20] K");
        }
        if !self.melt_temperature_k.is_finite()
            || !(180.0..=330.0).contains(&self.melt_temperature_k)
        {
            return Err("seasonal hydrology melt temperature must be finite and within [180, 330] K");
        }
        if !self.degree_day_melt_mm_per_k_day.is_finite()
            || self.degree_day_melt_mm_per_k_day <= 0.0
            || self.degree_day_melt_mm_per_k_day > 50.0
        {
            return Err("seasonal hydrology degree-day melt factor must be finite and within (0, 50]");
        }
        Ok(())
    }

    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        for value in [
            self.snow_temperature_k,
            self.snow_transition_k,
            self.melt_temperature_k,
            self.degree_day_melt_mm_per_k_day,
        ] {
            hash = fnv_update(hash, &value.to_bits().to_le_bytes());
        }
        hash
    }

    pub fn parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.parameter_hash())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SeasonalHydrologyRequest {
    pub seed: String,
    pub parameters: SeasonalHydrologyParameters,
}

impl SeasonalHydrologyRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            parameters: SeasonalHydrologyParameters::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SeasonalHydrologyMetrics {
    pub sample_count: u32,
    pub orbital_phase_count: u8,
    pub maximum_phase_local_runoff_m3_s: f64,
    pub maximum_phase_potential_discharge_m3_s: f64,
    pub snowmelt_runoff_fraction: f64,
    pub annual_mean_local_runoff_m3_s: f64,
    pub annual_local_runoff_closure_relative_error: f64,
    pub annual_mean_terminal_potential_discharge_m3_s: f64,
    pub seasonal_routing_conservation_relative_error: f64,
    pub seasonal_parameter_hash: u64,
    pub climate_hash: u64,
    pub drainage_hash: u64,
    pub runoff_hash: u64,
    pub seasonal_hydrology_hash: u64,
}

impl SeasonalHydrologyMetrics {
    pub fn seasonal_parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.seasonal_parameter_hash)
    }
    pub fn climate_hash_hex(&self) -> String {
        format!("{:016x}", self.climate_hash)
    }
    pub fn drainage_hash_hex(&self) -> String {
        format!("{:016x}", self.drainage_hash)
    }
    pub fn runoff_hash_hex(&self) -> String {
        format!("{:016x}", self.runoff_hash)
    }
    pub fn seasonal_hydrology_hash_hex(&self) -> String {
        format!("{:016x}", self.seasonal_hydrology_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SeasonalHydrologyState {
    pub stage: StageIdentity,
    pub metrics: SeasonalHydrologyMetrics,
    /// Phase-major local runoff rate: `phase * sample_count + sample`.
    pub phase_local_runoff_m3_s: Vec<f32>,
    /// Phase-major runoff whose timing is attributable to snowmelt.
    pub phase_snowmelt_runoff_m3_s: Vec<f32>,
    /// Snow storage after each phase, in water-equivalent millimetres.
    pub phase_snow_storage_mm: Vec<f32>,
    /// Phase-major potential discharge routed over the accepted WG-6A DAG.
    /// This intentionally ignores WG-6C lake retention; seasonal realized
    /// discharge is added in the next WG-6D slice.
    pub phase_potential_discharge_m3_s: Vec<f32>,
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

fn reconstructed_temperature(mean: f32, cosine: f32, sine: f32, angle: f64) -> f64 {
    f64::from(mean) + f64::from(cosine) * angle.cos() + f64::from(sine) * angle.sin()
}

fn snowfall_fraction(temperature_k: f64, parameters: SeasonalHydrologyParameters) -> f64 {
    let half_width = parameters.snow_transition_k * 0.5;
    let cold = parameters.snow_temperature_k - half_width;
    let warm = parameters.snow_temperature_k + half_width;
    if temperature_k <= cold {
        1.0
    } else if temperature_k >= warm {
        0.0
    } else {
        ((warm - temperature_k) / parameters.snow_transition_k).clamp(0.0, 1.0)
    }
}

#[allow(clippy::too_many_arguments)]
fn partition_phases(
    precipitation_rate_mm_year: &[f32],
    temperature_mean_k: f32,
    temperature_annual_cos_k: f32,
    temperature_annual_sin_k: f32,
    has_runoff: bool,
    phase_seconds: f64,
    year_seconds: f64,
    parameters: SeasonalHydrologyParameters,
    runoff_share: &mut [f64],
    snowmelt_share: &mut [f64],
    snow_storage_mm: &mut [f64],
    liquid_available_mm: &mut [f64],
    snowmelt_available_mm: &mut [f64],
) {
    let phase_count = precipitation_rate_mm_year.len();
    runoff_share.fill(0.0);
    snowmelt_share.fill(0.0);
    snow_storage_mm.fill(0.0);
    liquid_available_mm.fill(0.0);
    snowmelt_available_mm.fill(0.0);

    let mut storage = 0.0_f64;
    let phase_days = phase_seconds / SECONDS_PER_DAY;
    let precipitation_depth_scale = phase_seconds / year_seconds;
    for phase in 0..phase_count {
        let angle = std::f64::consts::TAU * phase as f64 / phase_count as f64;
        let temperature_k = reconstructed_temperature(
            temperature_mean_k,
            temperature_annual_cos_k,
            temperature_annual_sin_k,
            angle,
        );
        let precipitation_mm =
            f64::from(precipitation_rate_mm_year[phase]) * precipitation_depth_scale;
        let snow_fraction = snowfall_fraction(temperature_k, parameters);
        let snowfall_mm = precipitation_mm * snow_fraction;
        let rainfall_mm = precipitation_mm - snowfall_mm;
        storage += snowfall_mm;
        let melt_capacity_mm = parameters.degree_day_melt_mm_per_k_day
            * (temperature_k - parameters.melt_temperature_k).max(0.0)
            * phase_days;
        let melt_mm = storage.min(melt_capacity_mm);
        storage -= melt_mm;
        liquid_available_mm[phase] = rainfall_mm + melt_mm;
        snowmelt_available_mm[phase] = melt_mm;
        snow_storage_mm[phase] = storage;
    }

    if !has_runoff {
        return;
    }
    let liquid_total = liquid_available_mm.iter().sum::<f64>();
    if liquid_total > 0.0 {
        for phase in 0..phase_count {
            let share = liquid_available_mm[phase] / liquid_total;
            runoff_share[phase] = share;
            let melt_fraction = if liquid_available_mm[phase] > 0.0 {
                snowmelt_available_mm[phase] / liquid_available_mm[phase]
            } else {
                0.0
            };
            snowmelt_share[phase] = share * melt_fraction.clamp(0.0, 1.0);
        }
        return;
    }

    // WG-6B assumes zero annual snow-storage change. Preserve that accepted
    // annual runoff total for now; persistent-snow retention is a later WG-6D
    // step in this draft PR.
    let precipitation_total = precipitation_rate_mm_year
        .iter()
        .map(|value| f64::from(*value))
        .sum::<f64>();
    if precipitation_total > 0.0 {
        for phase in 0..phase_count {
            runoff_share[phase] =
                f64::from(precipitation_rate_mm_year[phase]) / precipitation_total;
        }
    } else {
        runoff_share.fill(1.0 / phase_count as f64);
    }
}

fn route_phase_discharge(
    phase_local_runoff_m3_s: &[f32],
    submerged_mask: &[u8],
    receiver: &[u32],
    drainage_order: &[u32],
    accumulation_m3_s: &mut [f64],
    output_m3_s: &mut [f32],
) -> Result<(f64, f64, f64), &'static str> {
    let count = phase_local_runoff_m3_s.len();
    if submerged_mask.len() != count
        || receiver.len() != count
        || accumulation_m3_s.len() != count
        || output_m3_s.len() != count
    {
        return Err("seasonal routing fields must align with the topology sample count");
    }

    accumulation_m3_s.fill(0.0);
    for i in 0..count {
        let local = f64::from(phase_local_runoff_m3_s[i]);
        if !local.is_finite() || local < 0.0 {
            return Err("seasonal local runoff must be finite and non-negative");
        }
        accumulation_m3_s[i] = local;
    }

    for &sample in drainage_order {
        let i = sample as usize;
        if i >= count || submerged_mask[i] != 0 {
            return Err("seasonal drainage order contains an invalid land sample");
        }
        let downstream = receiver[i];
        if downstream == INVALID_SAMPLE_ID {
            continue;
        }
        let downstream_index = downstream as usize;
        if downstream_index >= count {
            return Err("seasonal receiver references a sample outside topology");
        }
        let accumulated = accumulation_m3_s[downstream_index] + accumulation_m3_s[i];
        if !accumulated.is_finite() || accumulated > f32::MAX as f64 {
            return Err("seasonal accumulated discharge exceeds representable range");
        }
        accumulation_m3_s[downstream_index] = accumulated;
    }

    let mut total_local_m3_s = 0.0_f64;
    let mut terminal_m3_s = 0.0_f64;
    let mut maximum_m3_s = 0.0_f64;
    for i in 0..count {
        total_local_m3_s += f64::from(phase_local_runoff_m3_s[i]);
        let discharge = accumulation_m3_s[i];
        maximum_m3_s = maximum_m3_s.max(discharge);
        output_m3_s[i] = discharge as f32;
        if submerged_mask[i] != 0 || receiver[i] == INVALID_SAMPLE_ID {
            terminal_m3_s += discharge;
        }
    }

    Ok((total_local_m3_s, terminal_m3_s, maximum_m3_s))
}

pub fn generate_seasonal_hydrology(
    topology: &GeodesicTopology,
    topography: &TopographyState,
    climate: &ClimateState,
    climate_diagnostics: &ClimateGenerationDiagnostics,
    drainage: &DrainageState,
    runoff: &RunoffState,
    planet: PlanetPhysicalParameters,
    request: &SeasonalHydrologyRequest,
) -> Result<SeasonalHydrologyState, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    request
        .parameters
        .validate()
        .map_err(WorldgenError::InvalidHydrology)?;

    let count = topology.sample_count() as usize;
    let phase_count = usize::from(climate.metrics.orbital_phase_count);
    if topography.metrics.sample_count as usize != count
        || climate.metrics.sample_count as usize != count
        || drainage.metrics.sample_count as usize != count
        || runoff.metrics.sample_count as usize != count
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-6D inputs must align on the canonical fine topology",
        ));
    }
    if runoff.metrics.climate_hash != climate.metrics.climate_hash
        || runoff.metrics.drainage_hash != drainage.metrics.drainage_hash
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-6D requires WG-6B state derived from the accepted WG-5/WG-6A identities",
        ));
    }
    if drainage.receiver.len() != count
        || drainage.drainage_order.len() != drainage.metrics.land_sample_count as usize
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-6D requires a complete accepted WG-6A drainage graph",
        ));
    }
    if phase_count == 0
        || climate_diagnostics.precipitation_phase_rate_mm_year.len() != count * phase_count
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-6D requires retained WG-5 phase precipitation diagnostics",
        ));
    }

    let total_phase_samples = count.checked_mul(phase_count).ok_or(
        WorldgenError::InvalidHydrology("WG-6D phase field length overflow"),
    )?;
    let mut phase_local_runoff_m3_s = vec![0.0_f32; total_phase_samples];
    let mut phase_snowmelt_runoff_m3_s = vec![0.0_f32; total_phase_samples];
    let mut phase_snow_storage_mm = vec![0.0_f32; total_phase_samples];
    let mut phase_potential_discharge_m3_s = vec![0.0_f32; total_phase_samples];
    let mut sample_precipitation = vec![0.0_f32; phase_count];
    let mut runoff_share = vec![0.0_f64; phase_count];
    let mut snowmelt_share = vec![0.0_f64; phase_count];
    let mut snow_storage_mm = vec![0.0_f64; phase_count];
    let mut liquid_available_mm = vec![0.0_f64; phase_count];
    let mut snowmelt_available_mm = vec![0.0_f64; phase_count];
    let phase_seconds = planet.orbital_period_s / phase_count as f64;
    let mut local_phase_sum = 0.0_f64;
    let mut snowmelt_phase_sum = 0.0_f64;
    let mut maximum_phase_local_runoff_m3_s = 0.0_f64;

    for sample in 0..count {
        if topography.submerged_mask[sample] != 0 {
            continue;
        }
        for phase in 0..phase_count {
            sample_precipitation[phase] =
                climate_diagnostics.precipitation_phase_rate_mm_year[phase * count + sample];
        }
        partition_phases(
            &sample_precipitation,
            climate.temperature_mean_k[sample],
            climate.temperature_annual_cos_k[sample],
            climate.temperature_annual_sin_k[sample],
            runoff.local_runoff_m3_s[sample] > 0.0,
            phase_seconds,
            planet.orbital_period_s,
            request.parameters,
            &mut runoff_share,
            &mut snowmelt_share,
            &mut snow_storage_mm,
            &mut liquid_available_mm,
            &mut snowmelt_available_mm,
        );
        let annual_local = f64::from(runoff.local_runoff_m3_s[sample]);
        for phase in 0..phase_count {
            let index = phase * count + sample;
            let local = annual_local * phase_count as f64 * runoff_share[phase];
            let snowmelt = annual_local * phase_count as f64 * snowmelt_share[phase];
            phase_local_runoff_m3_s[index] = local as f32;
            phase_snowmelt_runoff_m3_s[index] = snowmelt as f32;
            phase_snow_storage_mm[index] = snow_storage_mm[phase] as f32;
            local_phase_sum += local;
            snowmelt_phase_sum += snowmelt;
            maximum_phase_local_runoff_m3_s = maximum_phase_local_runoff_m3_s.max(local);
        }
    }

    let annual_mean_local_runoff_m3_s = local_phase_sum / phase_count as f64;
    let annual_local_runoff_closure_relative_error = if runoff.metrics.total_local_runoff_m3_s > 0.0
    {
        (annual_mean_local_runoff_m3_s - runoff.metrics.total_local_runoff_m3_s).abs()
            / runoff.metrics.total_local_runoff_m3_s
    } else {
        annual_mean_local_runoff_m3_s.abs()
    };
    let snowmelt_runoff_fraction = if local_phase_sum > 0.0 {
        (snowmelt_phase_sum / local_phase_sum).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut accumulation_m3_s = vec![0.0_f64; count];
    let mut terminal_phase_sum_m3_s = 0.0_f64;
    let mut routed_local_phase_sum_m3_s = 0.0_f64;
    let mut maximum_phase_potential_discharge_m3_s = 0.0_f64;
    for phase in 0..phase_count {
        let start = phase * count;
        let end = start + count;
        let (routed_local, terminal, maximum) = route_phase_discharge(
            &phase_local_runoff_m3_s[start..end],
            &topography.submerged_mask,
            &drainage.receiver,
            &drainage.drainage_order,
            &mut accumulation_m3_s,
            &mut phase_potential_discharge_m3_s[start..end],
        )
        .map_err(WorldgenError::InvalidHydrology)?;
        routed_local_phase_sum_m3_s += routed_local;
        terminal_phase_sum_m3_s += terminal;
        maximum_phase_potential_discharge_m3_s =
            maximum_phase_potential_discharge_m3_s.max(maximum);
    }

    let annual_mean_terminal_potential_discharge_m3_s =
        terminal_phase_sum_m3_s / phase_count as f64;
    let seasonal_routing_conservation_relative_error = if routed_local_phase_sum_m3_s > 0.0 {
        (terminal_phase_sum_m3_s - routed_local_phase_sum_m3_s).abs()
            / routed_local_phase_sum_m3_s
    } else {
        terminal_phase_sum_m3_s.abs()
    };

    let stage_seed = derive_stage_seed(&request.seed, SEASONAL_HYDROLOGY_NAMESPACE);
    let seasonal_parameter_hash = request.parameters.parameter_hash();
    let mut seasonal_hydrology_hash = FNV_OFFSET_BASIS;
    seasonal_hydrology_hash = fnv_update(
        seasonal_hydrology_hash,
        SEASONAL_HYDROLOGY_STAGE_ID.as_bytes(),
    );
    seasonal_hydrology_hash = fnv_update(
        seasonal_hydrology_hash,
        &SEASONAL_HYDROLOGY_STAGE_VERSION.to_le_bytes(),
    );
    seasonal_hydrology_hash = fnv_update(seasonal_hydrology_hash, &stage_seed.to_le_bytes());
    seasonal_hydrology_hash = fnv_update(
        seasonal_hydrology_hash,
        &seasonal_parameter_hash.to_le_bytes(),
    );
    seasonal_hydrology_hash = fnv_update(
        seasonal_hydrology_hash,
        &climate.metrics.climate_hash.to_le_bytes(),
    );
    seasonal_hydrology_hash = fnv_update(
        seasonal_hydrology_hash,
        &drainage.metrics.drainage_hash.to_le_bytes(),
    );
    seasonal_hydrology_hash = fnv_update(
        seasonal_hydrology_hash,
        &runoff.metrics.runoff_hash.to_le_bytes(),
    );
    seasonal_hydrology_hash =
        hash_f32_slice(seasonal_hydrology_hash, &phase_local_runoff_m3_s);
    seasonal_hydrology_hash =
        hash_f32_slice(seasonal_hydrology_hash, &phase_snowmelt_runoff_m3_s);
    seasonal_hydrology_hash =
        hash_f32_slice(seasonal_hydrology_hash, &phase_snow_storage_mm);
    seasonal_hydrology_hash =
        hash_f32_slice(seasonal_hydrology_hash, &phase_potential_discharge_m3_s);

    Ok(SeasonalHydrologyState {
        stage: StageIdentity {
            id: SEASONAL_HYDROLOGY_STAGE_ID,
            version: SEASONAL_HYDROLOGY_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics: SeasonalHydrologyMetrics {
            sample_count: count as u32,
            orbital_phase_count: climate.metrics.orbital_phase_count,
            maximum_phase_local_runoff_m3_s,
            maximum_phase_potential_discharge_m3_s,
            snowmelt_runoff_fraction,
            annual_mean_local_runoff_m3_s,
            annual_local_runoff_closure_relative_error,
            annual_mean_terminal_potential_discharge_m3_s,
            seasonal_routing_conservation_relative_error,
            seasonal_parameter_hash,
            climate_hash: climate.metrics.climate_hash,
            drainage_hash: drainage.metrics.drainage_hash,
            runoff_hash: runoff.metrics.runoff_hash,
            seasonal_hydrology_hash,
        },
        phase_local_runoff_m3_s,
        phase_snowmelt_runoff_m3_s,
        phase_snow_storage_mm,
        phase_potential_discharge_m3_s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snow_is_stored_in_cold_phases_and_released_in_warm_phases() {
        let parameters = SeasonalHydrologyParameters::default();
        let precipitation = [400.0_f32; 4];
        let mut runoff = [0.0_f64; 4];
        let mut snowmelt = [0.0_f64; 4];
        let mut storage = [0.0_f64; 4];
        let mut liquid = [0.0_f64; 4];
        let mut melt_available = [0.0_f64; 4];
        partition_phases(
            &precipitation,
            273.15,
            -12.0,
            0.0,
            true,
            90.0 * SECONDS_PER_DAY,
            360.0 * SECONDS_PER_DAY,
            parameters,
            &mut runoff,
            &mut snowmelt,
            &mut storage,
            &mut liquid,
            &mut melt_available,
        );
        assert!((runoff.iter().sum::<f64>() - 1.0).abs() < 1.0e-12);
        assert!(snowmelt.iter().sum::<f64>() > 0.0);
        assert!(storage.iter().copied().fold(0.0_f64, f64::max) > 0.0);
    }

    #[test]
    fn warm_precipitation_keeps_phase_shares_normalized() {
        let parameters = SeasonalHydrologyParameters::default();
        let precipitation = [1200.0_f32, 600.0, 0.0, 600.0];
        let mut runoff = [0.0_f64; 4];
        let mut snowmelt = [0.0_f64; 4];
        let mut storage = [0.0_f64; 4];
        let mut liquid = [0.0_f64; 4];
        let mut melt_available = [0.0_f64; 4];
        partition_phases(
            &precipitation,
            290.0,
            0.0,
            0.0,
            true,
            90.0 * SECONDS_PER_DAY,
            360.0 * SECONDS_PER_DAY,
            parameters,
            &mut runoff,
            &mut snowmelt,
            &mut storage,
            &mut liquid,
            &mut melt_available,
        );
        assert!((runoff.iter().sum::<f64>() - 1.0).abs() < 1.0e-12);
        assert_eq!(snowmelt.iter().sum::<f64>(), 0.0);
        assert_eq!(storage.iter().sum::<f64>(), 0.0);
        assert!(runoff[0] > runoff[1]);
    }

    #[test]
    fn seasonal_routing_accumulates_each_phase_over_the_accepted_dag() {
        let local = [2.0_f32, 3.0, 0.0];
        let submerged = [0_u8, 0, 1];
        let receiver = [1_u32, 2, INVALID_SAMPLE_ID];
        let drainage_order = [0_u32, 1];
        let mut accumulation = [0.0_f64; 3];
        let mut output = [0.0_f32; 3];
        let (total_local, terminal, maximum) = route_phase_discharge(
            &local,
            &submerged,
            &receiver,
            &drainage_order,
            &mut accumulation,
            &mut output,
        )
        .unwrap();
        assert_eq!(total_local, 5.0);
        assert_eq!(terminal, 5.0);
        assert_eq!(maximum, 5.0);
        assert_eq!(output, [2.0, 5.0, 5.0]);
    }

    #[test]
    fn seasonal_routing_conserves_internal_terminal_flow() {
        let local = [1.25_f32, 2.75, 4.0];
        let submerged = [0_u8, 0, 0];
        let receiver = [1_u32, INVALID_SAMPLE_ID, INVALID_SAMPLE_ID];
        let drainage_order = [0_u32, 2, 1];
        let mut accumulation = [0.0_f64; 3];
        let mut output = [0.0_f32; 3];
        let (total_local, terminal, _) = route_phase_discharge(
            &local,
            &submerged,
            &receiver,
            &drainage_order,
            &mut accumulation,
            &mut output,
        )
        .unwrap();
        assert!((terminal - total_local).abs() < 1.0e-12);
        assert_eq!(output, [1.25, 4.0, 4.0]);
    }
}
