use crate::{
    derive_stage_seed, ClimateState, DrainageDepression, DrainageState, GeodesicTopology,
    PlanetPhysicalParameters, PlanetTopology, RunoffState, StageIdentity, TopographyState,
    WorldgenError, INVALID_SAMPLE_ID,
};
use std::collections::BTreeSet;

pub const LAKE_STAGE_ID: &str = "hydrology:lakes-closed-basins";
pub const LAKE_STAGE_VERSION: u32 = 1;
const LAKE_NAMESPACE: &str = "hydrology:lakes-closed-basins:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const MM_TO_M: f64 = 1.0e-3;
const FLOW_EPSILON_M3_S: f64 = 1.0e-12;

pub const LAKE_KIND_NONE: u8 = 0;
pub const LAKE_KIND_ENDORHEIC: u8 = 1;
pub const LAKE_KIND_OVERFLOWING: u8 = 2;
pub const LAKE_KIND_TERMINAL_STORAGE: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LakeParameters {
    /// Multiplier applied to accepted WG-5 PET over standing water.
    pub open_water_evaporation_scale: f64,
}

impl Default for LakeParameters {
    fn default() -> Self {
        Self {
            open_water_evaporation_scale: 1.0,
        }
    }
}

impl LakeParameters {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.open_water_evaporation_scale.is_finite()
            || self.open_water_evaporation_scale <= 0.0
            || self.open_water_evaporation_scale > 4.0
        {
            return Err("lake open-water evaporation scale must be finite and within (0, 4]");
        }
        Ok(())
    }

    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        hash = fnv_update(
            hash,
            &self.open_water_evaporation_scale.to_bits().to_le_bytes(),
        );
        hash
    }

    pub fn parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.parameter_hash())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LakeRequest {
    pub seed: String,
    pub parameters: LakeParameters,
}

impl LakeRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            parameters: LakeParameters::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LakeRecord {
    pub id: u32,
    pub depression_id: u32,
    pub kind: u8,
    pub surface_elevation_m: f64,
    pub area_m2: f64,
    pub volume_m3: f64,
    pub maximum_depth_m: f64,
    pub gross_land_inflow_m3_s: f64,
    pub lake_precipitation_m3_s: f64,
    pub lake_evaporation_m3_s: f64,
    pub outflow_m3_s: f64,
    pub unreleased_storage_m3_s: f64,
    pub spill_sample: u32,
    pub spill_receiver: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LakeMetrics {
    pub sample_count: u32,
    pub lake_count: u32,
    pub endorheic_lake_count: u32,
    pub overflowing_lake_count: u32,
    pub terminal_storage_lake_count: u32,
    pub lake_sample_count: u32,
    pub total_lake_area_m2: f64,
    pub total_lake_volume_m3: f64,
    pub maximum_lake_area_m2: f64,
    pub maximum_lake_depth_m: f64,
    pub total_lake_precipitation_m3_s: f64,
    pub total_lake_evaporation_m3_s: f64,
    pub terminal_realized_discharge_m3_s: f64,
    pub maximum_realized_discharge_m3_s: f64,
    pub unreleased_storage_m3_s: f64,
    pub water_balance_relative_error: f64,
    pub lake_parameter_hash: u64,
    pub climate_hash: u64,
    pub drainage_hash: u64,
    pub runoff_hash: u64,
    pub lake_hash: u64,
}

impl LakeMetrics {
    pub fn lake_parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.lake_parameter_hash)
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
    pub fn lake_hash_hex(&self) -> String {
        format!("{:016x}", self.lake_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LakeState {
    pub stage: StageIdentity,
    pub metrics: LakeMetrics,
    pub lake_id: Vec<u32>,
    pub lake_kind: Vec<u8>,
    pub lake_fraction: Vec<f32>,
    pub lake_depth_m: Vec<f32>,
    pub realized_discharge_m3_s: Vec<f32>,
    pub lakes: Vec<LakeRecord>,
}

#[derive(Debug)]
struct LakeCore {
    lake_id: Vec<u32>,
    lake_kind: Vec<u8>,
    lake_fraction: Vec<f32>,
    lake_depth_m: Vec<f32>,
    realized_discharge_m3_s: Vec<f32>,
    lakes: Vec<LakeRecord>,
    total_lake_area_m2: f64,
    total_lake_volume_m3: f64,
    maximum_lake_area_m2: f64,
    maximum_lake_depth_m: f64,
    total_lake_precipitation_m3_s: f64,
    total_lake_evaporation_m3_s: f64,
    terminal_realized_discharge_m3_s: f64,
    maximum_realized_discharge_m3_s: f64,
    unreleased_storage_m3_s: f64,
    water_balance_relative_error: f64,
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn hash_u32_slice(mut hash: u64, values: &[u32]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    for value in values {
        hash = fnv_update(hash, &value.to_le_bytes());
    }
    hash
}

fn hash_u8_slice(mut hash: u64, values: &[u8]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    fnv_update(hash, values)
}

fn hash_f32_slice(mut hash: u64, values: &[f32]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    for value in values {
        hash = fnv_update(hash, &value.to_bits().to_le_bytes());
    }
    hash
}

#[allow(clippy::too_many_arguments)]
fn validate_core_inputs<T: PlanetTopology>(
    topology: &T,
    elevation_m: &[f32],
    submerged_mask: &[u8],
    precipitation_mm: &[f32],
    pet_mm: &[f32],
    receiver: &[u32],
    drainage_order: &[u32],
    depression_id: &[u32],
    hydrologic_escape_elevation_m: &[f32],
    depressions: &[DrainageDepression],
    local_runoff_m3_s: &[f32],
    radius_m: f64,
    year_seconds: f64,
    parameters: LakeParameters,
) -> Result<(), &'static str> {
    parameters.validate()?;
    let count = topology.sample_count() as usize;
    if count == 0 {
        return Err("lake topology requires at least one sample");
    }
    for field in [
        elevation_m.len(),
        submerged_mask.len(),
        precipitation_mm.len(),
        pet_mm.len(),
        receiver.len(),
        depression_id.len(),
        hydrologic_escape_elevation_m.len(),
        local_runoff_m3_s.len(),
    ] {
        if field != count {
            return Err("WG-6C inputs must align with topology sample count");
        }
    }
    if !radius_m.is_finite() || radius_m <= 0.0 {
        return Err("lake planet radius must be finite and positive");
    }
    if !year_seconds.is_finite() || year_seconds <= 0.0 {
        return Err("lake orbital year duration must be finite and positive");
    }
    if precipitation_mm.iter().any(|v| !v.is_finite() || *v < 0.0)
        || pet_mm.iter().any(|v| !v.is_finite() || *v < 0.0)
        || local_runoff_m3_s.iter().any(|v| !v.is_finite() || *v < 0.0)
    {
        return Err("WG-6C forcing must be finite and non-negative");
    }
    let land_count = submerged_mask.iter().filter(|value| **value == 0).count();
    if drainage_order.len() != land_count {
        return Err("WG-6C requires a complete WG-6A drainage order");
    }
    for (index, depression) in depressions.iter().enumerate() {
        if depression.id as usize != index {
            return Err("WG-6C depression records must use canonical contiguous IDs");
        }
        if !depression.spill_elevation_m.is_finite()
            || !depression.floor_elevation_m.is_finite()
            || depression.spill_elevation_m < depression.floor_elevation_m
        {
            return Err("WG-6C depression geometry must be finite and ordered");
        }
    }
    for &id in depression_id {
        if id != INVALID_SAMPLE_ID && id as usize >= depressions.len() {
            return Err("WG-6C depression field references an unknown depression");
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn solve_lakes_core<T: PlanetTopology>(
    topology: &T,
    elevation_m: &[f32],
    submerged_mask: &[u8],
    precipitation_mm: &[f32],
    pet_mm: &[f32],
    receiver: &[u32],
    drainage_order: &[u32],
    depression_id: &[u32],
    hydrologic_escape_elevation_m: &[f32],
    depressions: &[DrainageDepression],
    local_runoff_m3_s: &[f32],
    radius_m: f64,
    year_seconds: f64,
    parameters: LakeParameters,
) -> Result<LakeCore, &'static str> {
    validate_core_inputs(
        topology,
        elevation_m,
        submerged_mask,
        precipitation_mm,
        pet_mm,
        receiver,
        drainage_order,
        depression_id,
        hydrologic_escape_elevation_m,
        depressions,
        local_runoff_m3_s,
        radius_m,
        year_seconds,
        parameters,
    )?;

    let count = topology.sample_count() as usize;
    let depression_count = depressions.len();
    let mut area_m2 = vec![0.0_f64; count];
    for i in 0..count {
        if submerged_mask[i] == 0 {
            area_m2[i] = topology.area_steradians(i as u32) * radius_m * radius_m;
            if !area_m2[i].is_finite() || area_m2[i] <= 0.0 {
                return Err("WG-6C cell area must be finite and positive");
            }
        }
    }

    let mut members = vec![Vec::<usize>::new(); depression_count];
    for (sample, &id) in depression_id.iter().enumerate() {
        if id != INVALID_SAMPLE_ID {
            members[id as usize].push(sample);
        }
    }
    for (id, samples) in members.iter().enumerate() {
        if samples.len() != depressions[id].sample_count as usize {
            return Err("WG-6C depression membership disagrees with WG-6A record");
        }
    }

    let mut spill_sample = vec![INVALID_SAMPLE_ID; depression_count];
    let mut spill_receiver = vec![INVALID_SAMPLE_ID; depression_count];
    let mut spill_escape = vec![f64::NEG_INFINITY; depression_count];
    for i in 0..count {
        let id = depression_id[i];
        if id == INVALID_SAMPLE_ID {
            continue;
        }
        let r = receiver[i];
        let exits_region = r == INVALID_SAMPLE_ID || depression_id[r as usize] != id;
        if !exits_region {
            continue;
        }
        let d = id as usize;
        let escape = f64::from(hydrologic_escape_elevation_m[i]);
        if escape > spill_escape[d]
            || (escape.to_bits() == spill_escape[d].to_bits()
                && (spill_sample[d] == INVALID_SAMPLE_ID || (i as u32) < spill_sample[d]))
        {
            spill_escape[d] = escape;
            spill_sample[d] = i as u32;
            spill_receiver[d] = r;
        }
    }
    if spill_sample
        .iter()
        .any(|sample| *sample == INVALID_SAMPLE_ID)
    {
        return Err("WG-6C could not resolve a deterministic depression spill sample");
    }

    // For each land sample, cache the first depression encountered downstream.
    let mut first_downstream_depression = vec![INVALID_SAMPLE_ID; count];
    for &sample in drainage_order.iter().rev() {
        let i = sample as usize;
        let own = depression_id[i];
        if own != INVALID_SAMPLE_ID {
            first_downstream_depression[i] = own;
            continue;
        }
        let r = receiver[i];
        if r == INVALID_SAMPLE_ID || submerged_mask[r as usize] != 0 {
            continue;
        }
        let downstream_id = depression_id[r as usize];
        first_downstream_depression[i] = if downstream_id != INVALID_SAMPLE_ID {
            downstream_id
        } else {
            first_downstream_depression[r as usize]
        };
    }

    let mut external_inflow_m3_s = vec![0.0_f64; depression_count];
    let mut member_local_runoff_m3_s = vec![0.0_f64; depression_count];
    for i in 0..count {
        if submerged_mask[i] != 0 {
            continue;
        }
        let local = f64::from(local_runoff_m3_s[i]);
        let own = depression_id[i];
        if own != INVALID_SAMPLE_ID {
            member_local_runoff_m3_s[own as usize] += local;
        } else {
            let target = first_downstream_depression[i];
            if target != INVALID_SAMPLE_ID {
                external_inflow_m3_s[target as usize] += local;
            }
        }
    }

    let mut downstream_depression = vec![INVALID_SAMPLE_ID; depression_count];
    for d in 0..depression_count {
        let r = spill_receiver[d];
        if r == INVALID_SAMPLE_ID || submerged_mask[r as usize] != 0 {
            continue;
        }
        let direct = depression_id[r as usize];
        downstream_depression[d] = if direct != INVALID_SAMPLE_ID {
            direct
        } else {
            first_downstream_depression[r as usize]
        };
        if downstream_depression[d] == d as u32 {
            return Err("WG-6C depression spill routing cannot target itself");
        }
    }

    let mut indegree = vec![0_u32; depression_count];
    for &target in &downstream_depression {
        if target != INVALID_SAMPLE_ID {
            indegree[target as usize] += 1;
        }
    }
    let mut ready = BTreeSet::new();
    for (id, &degree) in indegree.iter().enumerate() {
        if degree == 0 {
            ready.insert(id);
        }
    }
    let mut depression_order = Vec::with_capacity(depression_count);
    while let Some(&id) = ready.iter().next() {
        ready.remove(&id);
        depression_order.push(id);
        let target = downstream_depression[id];
        if target != INVALID_SAMPLE_ID {
            let degree = &mut indegree[target as usize];
            *degree -= 1;
            if *degree == 0 {
                ready.insert(target as usize);
            }
        }
    }
    if depression_order.len() != depression_count {
        return Err("WG-6C depression spill graph must be acyclic");
    }

    let mut lake_id = vec![INVALID_SAMPLE_ID; count];
    let mut lake_kind = vec![LAKE_KIND_NONE; count];
    let mut lake_fraction = vec![0.0_f32; count];
    let mut lake_depth_m = vec![0.0_f32; count];
    let mut lake_records_by_depression = vec![None::<LakeRecord>; depression_count];

    for d in depression_order {
        let gross_land_inflow = external_inflow_m3_s[d] + member_local_runoff_m3_s[d];
        if gross_land_inflow <= FLOW_EPSILON_M3_S {
            continue;
        }

        let mut sorted = members[d].clone();
        sorted.sort_by(|a, b| {
            f64::from(elevation_m[*a])
                .total_cmp(&f64::from(elevation_m[*b]))
                .then_with(|| a.cmp(b))
        });

        let mut balance = gross_land_inflow;
        let mut surface_elevation_m = depressions[d].spill_elevation_m;
        let mut solved_below_spill = false;

        for (rank, &sample) in sorted.iter().enumerate() {
            let p_flux =
                f64::from(precipitation_mm[sample]) * MM_TO_M * area_m2[sample] / year_seconds;
            let e_flux = f64::from(pet_mm[sample])
                * parameters.open_water_evaporation_scale
                * MM_TO_M
                * area_m2[sample]
                / year_seconds;
            let adjustment = p_flux - e_flux - f64::from(local_runoff_m3_s[sample]);
            let next_balance = balance + adjustment;

            if balance > FLOW_EPSILON_M3_S && next_balance <= FLOW_EPSILON_M3_S && adjustment < 0.0
            {
                let fraction = (balance / -adjustment).clamp(0.0, 1.0);
                lake_fraction[sample] = fraction as f32;
                let lower = f64::from(elevation_m[sample]);
                let upper = sorted
                    .get(rank + 1)
                    .map(|next| f64::from(elevation_m[*next]))
                    .unwrap_or(depressions[d].spill_elevation_m)
                    .max(lower)
                    .min(depressions[d].spill_elevation_m);
                surface_elevation_m = lower + (upper - lower) * fraction;
                balance = 0.0;
                solved_below_spill = true;
                break;
            }

            lake_fraction[sample] = 1.0;
            balance = next_balance;
            if balance <= FLOW_EPSILON_M3_S {
                surface_elevation_m = sorted
                    .get(rank + 1)
                    .map(|next| f64::from(elevation_m[*next]))
                    .unwrap_or(depressions[d].spill_elevation_m)
                    .min(depressions[d].spill_elevation_m);
                balance = 0.0;
                solved_below_spill = true;
                break;
            }
        }

        let kind;
        let outflow_m3_s;
        let unreleased_storage_m3_s;
        if solved_below_spill {
            kind = LAKE_KIND_ENDORHEIC;
            outflow_m3_s = 0.0;
            unreleased_storage_m3_s = 0.0;
        } else if balance > FLOW_EPSILON_M3_S && spill_receiver[d] != INVALID_SAMPLE_ID {
            kind = LAKE_KIND_OVERFLOWING;
            outflow_m3_s = balance;
            unreleased_storage_m3_s = 0.0;
            surface_elevation_m = depressions[d].spill_elevation_m;
        } else if balance > FLOW_EPSILON_M3_S {
            kind = LAKE_KIND_TERMINAL_STORAGE;
            outflow_m3_s = 0.0;
            unreleased_storage_m3_s = balance;
            surface_elevation_m = depressions[d].spill_elevation_m;
        } else {
            kind = LAKE_KIND_ENDORHEIC;
            outflow_m3_s = 0.0;
            unreleased_storage_m3_s = 0.0;
        }

        let mut area = 0.0_f64;
        let mut volume = 0.0_f64;
        let mut maximum_depth = 0.0_f64;
        let mut lake_precipitation = 0.0_f64;
        let mut lake_evaporation = 0.0_f64;
        for &sample in &members[d] {
            let fraction = f64::from(lake_fraction[sample]);
            if fraction <= 0.0 {
                continue;
            }
            lake_id[sample] = d as u32;
            lake_kind[sample] = kind;
            let depth = (surface_elevation_m - f64::from(elevation_m[sample])).max(0.0);
            lake_depth_m[sample] = depth as f32;
            area += area_m2[sample] * fraction;
            volume += area_m2[sample] * fraction * depth;
            maximum_depth = maximum_depth.max(depth);
            lake_precipitation +=
                f64::from(precipitation_mm[sample]) * MM_TO_M * area_m2[sample] * fraction
                    / year_seconds;
            lake_evaporation += f64::from(pet_mm[sample])
                * parameters.open_water_evaporation_scale
                * MM_TO_M
                * area_m2[sample]
                * fraction
                / year_seconds;
        }

        let id = lake_records_by_depression
            .iter()
            .filter(|record| record.is_some())
            .count() as u32;
        lake_records_by_depression[d] = Some(LakeRecord {
            id,
            depression_id: d as u32,
            kind,
            surface_elevation_m,
            area_m2: area,
            volume_m3: volume,
            maximum_depth_m: maximum_depth,
            gross_land_inflow_m3_s: gross_land_inflow,
            lake_precipitation_m3_s: lake_precipitation,
            lake_evaporation_m3_s: lake_evaporation,
            outflow_m3_s,
            unreleased_storage_m3_s,
            spill_sample: spill_sample[d],
            spill_receiver: spill_receiver[d],
        });

        if outflow_m3_s > 0.0 {
            let target = downstream_depression[d];
            if target != INVALID_SAMPLE_ID {
                external_inflow_m3_s[target as usize] += outflow_m3_s;
            }
        }
    }

    let lakes = lake_records_by_depression
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let mut injection_m3_s = vec![0.0_f64; count];
    for lake in &lakes {
        if lake.kind == LAKE_KIND_OVERFLOWING && lake.spill_receiver != INVALID_SAMPLE_ID {
            injection_m3_s[lake.spill_receiver as usize] += lake.outflow_m3_s;
        }
    }

    let mut realized_accum_m3_s = injection_m3_s;
    for i in 0..count {
        if submerged_mask[i] == 0 {
            realized_accum_m3_s[i] +=
                f64::from(local_runoff_m3_s[i]) * (1.0 - f64::from(lake_fraction[i]));
        }
    }
    for &sample in drainage_order {
        let i = sample as usize;
        if lake_fraction[i] > 0.0 {
            continue;
        }
        let r = receiver[i];
        if r != INVALID_SAMPLE_ID {
            realized_accum_m3_s[r as usize] += realized_accum_m3_s[i];
        }
    }

    let mut terminal_realized_discharge_m3_s = 0.0_f64;
    let mut maximum_realized_discharge_m3_s = 0.0_f64;
    let mut realized_discharge_m3_s = vec![0.0_f32; count];
    for i in 0..count {
        maximum_realized_discharge_m3_s =
            maximum_realized_discharge_m3_s.max(realized_accum_m3_s[i]);
        if lake_fraction[i] <= 0.0 {
            realized_discharge_m3_s[i] = realized_accum_m3_s[i] as f32;
        }
        if submerged_mask[i] != 0
            || (submerged_mask[i] == 0
                && receiver[i] == INVALID_SAMPLE_ID
                && lake_fraction[i] <= 0.0)
        {
            terminal_realized_discharge_m3_s += realized_accum_m3_s[i];
        }
    }

    let total_lake_area_m2 = lakes.iter().map(|lake| lake.area_m2).sum::<f64>();
    let total_lake_volume_m3 = lakes.iter().map(|lake| lake.volume_m3).sum::<f64>();
    let maximum_lake_area_m2 = lakes
        .iter()
        .map(|lake| lake.area_m2)
        .fold(0.0_f64, f64::max);
    let maximum_lake_depth_m = lakes
        .iter()
        .map(|lake| lake.maximum_depth_m)
        .fold(0.0_f64, f64::max);
    let total_lake_precipitation_m3_s = lakes
        .iter()
        .map(|lake| lake.lake_precipitation_m3_s)
        .sum::<f64>();
    let total_lake_evaporation_m3_s = lakes
        .iter()
        .map(|lake| lake.lake_evaporation_m3_s)
        .sum::<f64>();
    let unreleased_storage_m3_s = lakes
        .iter()
        .map(|lake| lake.unreleased_storage_m3_s)
        .sum::<f64>();
    let dry_land_runoff_m3_s = (0..count)
        .filter(|i| submerged_mask[*i] == 0)
        .map(|i| f64::from(local_runoff_m3_s[i]) * (1.0 - f64::from(lake_fraction[i])))
        .sum::<f64>();
    let water_input = dry_land_runoff_m3_s + total_lake_precipitation_m3_s;
    let water_output =
        terminal_realized_discharge_m3_s + total_lake_evaporation_m3_s + unreleased_storage_m3_s;
    let water_balance_relative_error = if water_input > 0.0 {
        (water_input - water_output).abs() / water_input
    } else {
        water_output.abs()
    };

    Ok(LakeCore {
        lake_id,
        lake_kind,
        lake_fraction,
        lake_depth_m,
        realized_discharge_m3_s,
        lakes,
        total_lake_area_m2,
        total_lake_volume_m3,
        maximum_lake_area_m2,
        maximum_lake_depth_m,
        total_lake_precipitation_m3_s,
        total_lake_evaporation_m3_s,
        terminal_realized_discharge_m3_s,
        maximum_realized_discharge_m3_s,
        unreleased_storage_m3_s,
        water_balance_relative_error,
    })
}

pub fn generate_lakes_closed_basins(
    topology: &GeodesicTopology,
    topography: &TopographyState,
    climate: &ClimateState,
    drainage: &DrainageState,
    runoff: &RunoffState,
    planet: PlanetPhysicalParameters,
    request: &LakeRequest,
) -> Result<LakeState, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    request
        .parameters
        .validate()
        .map_err(WorldgenError::InvalidHydrology)?;
    let count = topology.sample_count();
    if topography.metrics.sample_count != count
        || climate.metrics.sample_count != count
        || drainage.metrics.sample_count != count
        || runoff.metrics.sample_count != count
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-6C inputs must align on the canonical fine topology",
        ));
    }
    if runoff.metrics.climate_hash != climate.metrics.climate_hash
        || runoff.metrics.drainage_hash != drainage.metrics.drainage_hash
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-6C requires WG-6B state derived from the accepted WG-5/WG-6A identities",
        ));
    }

    let core = solve_lakes_core(
        topology,
        &topography.solid_elevation_m,
        &topography.submerged_mask,
        &climate.annual_precipitation_mm,
        &climate.potential_evaporation_mm,
        &drainage.receiver,
        &drainage.drainage_order,
        &drainage.depression_id,
        &drainage.hydrologic_escape_elevation_m,
        &drainage.depressions,
        &runoff.local_runoff_m3_s,
        planet.radius_m,
        planet.orbital_period_s,
        request.parameters,
    )
    .map_err(WorldgenError::InvalidHydrology)?;

    let stage_seed = derive_stage_seed(&request.seed, LAKE_NAMESPACE);
    let lake_parameter_hash = request.parameters.parameter_hash();
    let mut lake_hash = FNV_OFFSET_BASIS;
    lake_hash = fnv_update(lake_hash, LAKE_STAGE_ID.as_bytes());
    lake_hash = fnv_update(lake_hash, &LAKE_STAGE_VERSION.to_le_bytes());
    lake_hash = fnv_update(lake_hash, &stage_seed.to_le_bytes());
    lake_hash = fnv_update(lake_hash, &planet.parameter_hash().to_le_bytes());
    lake_hash = fnv_update(lake_hash, &lake_parameter_hash.to_le_bytes());
    lake_hash = fnv_update(lake_hash, &climate.metrics.climate_hash.to_le_bytes());
    lake_hash = fnv_update(lake_hash, &drainage.metrics.drainage_hash.to_le_bytes());
    lake_hash = fnv_update(lake_hash, &runoff.metrics.runoff_hash.to_le_bytes());
    lake_hash = hash_u32_slice(lake_hash, &core.lake_id);
    lake_hash = hash_u8_slice(lake_hash, &core.lake_kind);
    lake_hash = hash_f32_slice(lake_hash, &core.lake_fraction);
    lake_hash = hash_f32_slice(lake_hash, &core.lake_depth_m);
    lake_hash = hash_f32_slice(lake_hash, &core.realized_discharge_m3_s);

    let lake_sample_count = core
        .lake_fraction
        .iter()
        .filter(|fraction| **fraction > 0.0)
        .count() as u32;
    let endorheic_lake_count = core
        .lakes
        .iter()
        .filter(|lake| lake.kind == LAKE_KIND_ENDORHEIC)
        .count() as u32;
    let overflowing_lake_count = core
        .lakes
        .iter()
        .filter(|lake| lake.kind == LAKE_KIND_OVERFLOWING)
        .count() as u32;
    let terminal_storage_lake_count = core
        .lakes
        .iter()
        .filter(|lake| lake.kind == LAKE_KIND_TERMINAL_STORAGE)
        .count() as u32;

    Ok(LakeState {
        stage: StageIdentity {
            id: LAKE_STAGE_ID,
            version: LAKE_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics: LakeMetrics {
            sample_count: count,
            lake_count: core.lakes.len() as u32,
            endorheic_lake_count,
            overflowing_lake_count,
            terminal_storage_lake_count,
            lake_sample_count,
            total_lake_area_m2: core.total_lake_area_m2,
            total_lake_volume_m3: core.total_lake_volume_m3,
            maximum_lake_area_m2: core.maximum_lake_area_m2,
            maximum_lake_depth_m: core.maximum_lake_depth_m,
            total_lake_precipitation_m3_s: core.total_lake_precipitation_m3_s,
            total_lake_evaporation_m3_s: core.total_lake_evaporation_m3_s,
            terminal_realized_discharge_m3_s: core.terminal_realized_discharge_m3_s,
            maximum_realized_discharge_m3_s: core.maximum_realized_discharge_m3_s,
            unreleased_storage_m3_s: core.unreleased_storage_m3_s,
            water_balance_relative_error: core.water_balance_relative_error,
            lake_parameter_hash,
            climate_hash: climate.metrics.climate_hash,
            drainage_hash: drainage.metrics.drainage_hash,
            runoff_hash: runoff.metrics.runoff_hash,
            lake_hash,
        },
        lake_id: core.lake_id,
        lake_kind: core.lake_kind,
        lake_fraction: core.lake_fraction,
        lake_depth_m: core.lake_depth_m,
        realized_discharge_m3_s: core.realized_discharge_m3_s,
        lakes: core.lakes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTopology {
        neighbors: Vec<Vec<u32>>,
        distances: Vec<Vec<f64>>,
        areas: Vec<f64>,
    }

    impl TestTopology {
        fn chain(count: usize) -> Self {
            let mut neighbors = vec![Vec::new(); count];
            let mut distances = vec![Vec::new(); count];
            for i in 0..count - 1 {
                neighbors[i].push((i + 1) as u32);
                distances[i].push(1.0);
                neighbors[i + 1].push(i as u32);
                distances[i + 1].push(1.0);
            }
            Self {
                neighbors,
                distances,
                areas: vec![1.0; count],
            }
        }
    }

    impl PlanetTopology for TestTopology {
        fn sample_count(&self) -> u32 {
            self.neighbors.len() as u32
        }
        fn unit_position(&self, _sample: u32) -> [f64; 3] {
            [1.0, 0.0, 0.0]
        }
        fn area_steradians(&self, sample: u32) -> f64 {
            self.areas[sample as usize]
        }
        fn neighbors(&self, sample: u32) -> &[u32] {
            &self.neighbors[sample as usize]
        }
        fn neighbor_arc_lengths_rad(&self, sample: u32) -> &[f64] {
            &self.distances[sample as usize]
        }
        fn neighbor_interface_arc_lengths_rad(&self, sample: u32) -> &[f64] {
            &self.distances[sample as usize]
        }
    }

    fn depression() -> DrainageDepression {
        DrainageDepression {
            id: 0,
            sample_count: 2,
            area_m2: 2.0,
            floor_sample: 1,
            floor_elevation_m: 0.0,
            spill_elevation_m: 30.0,
            maximum_depth_m: 30.0,
        }
    }

    #[test]
    fn evaporation_can_close_a_basin_below_spill() {
        let topology = TestTopology::chain(5);
        let core = solve_lakes_core(
            &topology,
            &[100.0, 0.0, 20.0, 10.0, -10.0],
            &[0, 0, 0, 0, 1],
            &[0.0; 5],
            &[0.0, 1000.0, 1000.0, 0.0, 0.0],
            &[1, 2, 3, 4, INVALID_SAMPLE_ID],
            &[0, 1, 2, 3],
            &[
                INVALID_SAMPLE_ID,
                0,
                0,
                INVALID_SAMPLE_ID,
                INVALID_SAMPLE_ID,
            ],
            &[100.0, 30.0, 30.0, 10.0, 0.0],
            &[depression()],
            &[0.1, 0.0, 0.0, 0.0, 0.0],
            1.0,
            1.0,
            LakeParameters::default(),
        )
        .unwrap();
        assert_eq!(core.lakes.len(), 1);
        assert_eq!(core.lakes[0].kind, LAKE_KIND_ENDORHEIC);
        assert_eq!(core.lakes[0].outflow_m3_s, 0.0);
        assert!(core.lakes[0].surface_elevation_m < 30.0);
        assert!(core.water_balance_relative_error < 1.0e-9);
        assert!(core.terminal_realized_discharge_m3_s < 1.0e-9);
    }

    #[test]
    fn wet_basin_reaches_spill_and_routes_only_residual_outflow() {
        let topology = TestTopology::chain(5);
        let core = solve_lakes_core(
            &topology,
            &[100.0, 0.0, 20.0, 10.0, -10.0],
            &[0, 0, 0, 0, 1],
            &[0.0; 5],
            &[0.0, 100.0, 100.0, 0.0, 0.0],
            &[1, 2, 3, 4, INVALID_SAMPLE_ID],
            &[0, 1, 2, 3],
            &[
                INVALID_SAMPLE_ID,
                0,
                0,
                INVALID_SAMPLE_ID,
                INVALID_SAMPLE_ID,
            ],
            &[100.0, 30.0, 30.0, 10.0, 0.0],
            &[depression()],
            &[3.0, 0.0, 0.0, 0.0, 0.0],
            1.0,
            1.0,
            LakeParameters::default(),
        )
        .unwrap();
        assert_eq!(core.lakes.len(), 1);
        assert_eq!(core.lakes[0].kind, LAKE_KIND_OVERFLOWING);
        assert!((core.lakes[0].outflow_m3_s - 2.8).abs() < 1.0e-9);
        assert!((core.terminal_realized_discharge_m3_s - 2.8).abs() < 1.0e-6);
        assert!(core.water_balance_relative_error < 1.0e-9);
    }
}
