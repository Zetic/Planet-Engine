use crate::{
    derive_stage_seed, ClimateState, DrainageState, GeodesicTopology, PlanetPhysicalParameters,
    PlanetTopology, StageIdentity, TopographyState, WorldgenError, INVALID_SAMPLE_ID,
};

pub const RUNOFF_STAGE_ID: &str = "hydrology:runoff-discharge";
pub const RUNOFF_STAGE_VERSION: u32 = 1;
const RUNOFF_NAMESPACE: &str = "hydrology:runoff-discharge:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const MM_TO_M: f64 = 1.0e-3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RunoffParameters {
    /// Fu/Budyko shape parameter. Values greater than 1 produce a smooth
    /// transition between water-limited and energy-limited annual AET.
    pub budyko_omega: f64,
}

impl Default for RunoffParameters {
    fn default() -> Self {
        Self { budyko_omega: 2.6 }
    }
}

impl RunoffParameters {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.budyko_omega.is_finite() || self.budyko_omega <= 1.0 || self.budyko_omega > 12.0 {
            return Err("runoff Budyko omega must be finite and within (1, 12]");
        }
        Ok(())
    }

    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        hash = fnv_update(hash, &self.budyko_omega.to_bits().to_le_bytes());
        hash
    }

    pub fn parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.parameter_hash())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunoffRequest {
    pub seed: String,
    pub parameters: RunoffParameters,
}

impl RunoffRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            parameters: RunoffParameters::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunoffMetrics {
    pub sample_count: u32,
    pub land_sample_count: u32,
    pub land_area_m2: f64,
    pub mean_land_precipitation_mm: f64,
    pub mean_land_actual_evapotranspiration_mm: f64,
    pub mean_land_runoff_mm: f64,
    pub maximum_land_runoff_mm: f64,
    pub land_runoff_fraction: f64,
    pub total_local_runoff_m3_s: f64,
    pub terminal_discharge_m3_s: f64,
    pub discharge_conservation_relative_error: f64,
    pub maximum_potential_discharge_m3_s: f64,
    pub runoff_parameter_hash: u64,
    pub climate_hash: u64,
    pub drainage_hash: u64,
    pub runoff_hash: u64,
}

impl RunoffMetrics {
    pub fn runoff_parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.runoff_parameter_hash)
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunoffState {
    pub stage: StageIdentity,
    pub metrics: RunoffMetrics,
    pub actual_evapotranspiration_mm: Vec<f32>,
    pub local_runoff_mm: Vec<f32>,
    pub runoff_fraction: Vec<f32>,
    pub local_runoff_m3_s: Vec<f32>,
    /// Potential annualized discharge routed over the WG-6A escape graph.
    /// WG-6C may later retain water in lakes/endorheic basins before spill.
    pub potential_discharge_m3_s: Vec<f32>,
}

#[derive(Debug)]
struct RunoffCore {
    actual_evapotranspiration_mm: Vec<f32>,
    local_runoff_mm: Vec<f32>,
    runoff_fraction: Vec<f32>,
    local_runoff_m3_s: Vec<f32>,
    potential_discharge_m3_s: Vec<f32>,
    land_area_m2: f64,
    precipitation_area_sum_mm_m2: f64,
    aet_area_sum_mm_m2: f64,
    runoff_area_sum_mm_m2: f64,
    maximum_land_runoff_mm: f64,
    total_local_runoff_m3_s: f64,
    terminal_discharge_m3_s: f64,
    discharge_conservation_relative_error: f64,
    maximum_potential_discharge_m3_s: f64,
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

/// Annual actual evapotranspiration from Fu's analytical Budyko curve.
/// Inputs and output use the same depth-per-year units.
fn budyko_actual_evapotranspiration(precipitation: f64, pet: f64, omega: f64) -> f64 {
    if precipitation <= 0.0 {
        return 0.0;
    }
    if pet <= 0.0 {
        return 0.0;
    }
    let aridity = pet / precipitation;
    let aet_fraction =
        1.0 + aridity - (1.0 + aridity.powf(omega)).powf(1.0 / omega);
    (precipitation * aet_fraction).clamp(0.0, precipitation.min(pet))
}

fn validate_core_inputs<T: PlanetTopology>(
    topology: &T,
    submerged_mask: &[u8],
    precipitation_mm: &[f32],
    pet_mm: &[f32],
    receiver: &[u32],
    drainage_order: &[u32],
    radius_m: f64,
    year_seconds: f64,
    parameters: RunoffParameters,
) -> Result<(), &'static str> {
    parameters.validate()?;
    let count = topology.sample_count() as usize;
    if count == 0 {
        return Err("runoff topology requires at least one sample");
    }
    if submerged_mask.len() != count
        || precipitation_mm.len() != count
        || pet_mm.len() != count
        || receiver.len() != count
    {
        return Err("runoff inputs must align with topology sample count");
    }
    if !radius_m.is_finite() || radius_m <= 0.0 {
        return Err("runoff planet radius must be finite and positive");
    }
    if !year_seconds.is_finite() || year_seconds <= 0.0 {
        return Err("runoff orbital year duration must be finite and positive");
    }
    if precipitation_mm.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err("runoff precipitation must be finite and non-negative");
    }
    if pet_mm.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err("runoff potential evaporation must be finite and non-negative");
    }
    if submerged_mask.iter().any(|value| *value > 1) {
        return Err("runoff submerged mask must contain only 0 or 1");
    }
    let land_count = submerged_mask.iter().filter(|value| **value == 0).count();
    if drainage_order.len() != land_count {
        return Err("runoff drainage order must contain every land sample exactly once");
    }
    let mut seen = vec![false; count];
    for &sample in drainage_order {
        let i = sample as usize;
        if i >= count || submerged_mask[i] != 0 || seen[i] {
            return Err("runoff drainage order contains an invalid or duplicate sample");
        }
        seen[i] = true;
        let r = receiver[i];
        if r != INVALID_SAMPLE_ID && r as usize >= count {
            return Err("runoff receiver references a sample outside topology");
        }
    }
    Ok(())
}

fn solve_runoff_core<T: PlanetTopology>(
    topology: &T,
    submerged_mask: &[u8],
    precipitation_mm: &[f32],
    pet_mm: &[f32],
    receiver: &[u32],
    drainage_order: &[u32],
    radius_m: f64,
    year_seconds: f64,
    parameters: RunoffParameters,
) -> Result<RunoffCore, &'static str> {
    validate_core_inputs(
        topology,
        submerged_mask,
        precipitation_mm,
        pet_mm,
        receiver,
        drainage_order,
        radius_m,
        year_seconds,
        parameters,
    )?;

    let count = topology.sample_count() as usize;
    let mut actual_evapotranspiration_mm = vec![0.0_f32; count];
    let mut local_runoff_mm = vec![0.0_f32; count];
    let mut runoff_fraction = vec![0.0_f32; count];
    let mut local_runoff_m3_s = vec![0.0_f32; count];
    let mut potential_discharge_m3_s = vec![0.0_f32; count];

    let mut land_area_m2 = 0.0;
    let mut precipitation_area_sum_mm_m2 = 0.0;
    let mut aet_area_sum_mm_m2 = 0.0;
    let mut runoff_area_sum_mm_m2 = 0.0;
    let mut maximum_land_runoff_mm = 0.0_f64;
    let mut total_local_runoff_m3_s = 0.0_f64;

    for i in 0..count {
        if submerged_mask[i] != 0 {
            continue;
        }
        let area_m2 = topology.area_steradians(i as u32) * radius_m * radius_m;
        if !area_m2.is_finite() || area_m2 <= 0.0 {
            return Err("runoff cell area must be finite and positive");
        }
        let precipitation = f64::from(precipitation_mm[i]);
        let pet = f64::from(pet_mm[i]);
        let aet = budyko_actual_evapotranspiration(precipitation, pet, parameters.budyko_omega);
        let runoff = (precipitation - aet).max(0.0);
        let fraction = if precipitation > 0.0 {
            (runoff / precipitation).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let local_m3_s = runoff * MM_TO_M * area_m2 / year_seconds;

        actual_evapotranspiration_mm[i] = aet as f32;
        local_runoff_mm[i] = runoff as f32;
        runoff_fraction[i] = fraction as f32;
        local_runoff_m3_s[i] = local_m3_s as f32;
        potential_discharge_m3_s[i] = local_m3_s as f32;

        land_area_m2 += area_m2;
        precipitation_area_sum_mm_m2 += precipitation * area_m2;
        aet_area_sum_mm_m2 += aet * area_m2;
        runoff_area_sum_mm_m2 += runoff * area_m2;
        maximum_land_runoff_mm = maximum_land_runoff_mm.max(runoff);
        total_local_runoff_m3_s += local_m3_s;
    }

    for &sample in drainage_order {
        let i = sample as usize;
        let r = receiver[i];
        if r == INVALID_SAMPLE_ID {
            continue;
        }
        let ri = r as usize;
        let accumulated = f64::from(potential_discharge_m3_s[ri])
            + f64::from(potential_discharge_m3_s[i]);
        if !accumulated.is_finite() || accumulated > f32::MAX as f64 {
            return Err("runoff accumulated discharge exceeds representable range");
        }
        potential_discharge_m3_s[ri] = accumulated as f32;
    }

    let mut terminal_discharge_m3_s = 0.0_f64;
    let mut maximum_potential_discharge_m3_s = 0.0_f64;
    for i in 0..count {
        let discharge = f64::from(potential_discharge_m3_s[i]);
        maximum_potential_discharge_m3_s = maximum_potential_discharge_m3_s.max(discharge);
        if submerged_mask[i] != 0 || receiver[i] == INVALID_SAMPLE_ID {
            terminal_discharge_m3_s += discharge;
        }
    }
    let discharge_conservation_relative_error = if total_local_runoff_m3_s > 0.0 {
        (terminal_discharge_m3_s - total_local_runoff_m3_s).abs() / total_local_runoff_m3_s
    } else {
        terminal_discharge_m3_s.abs()
    };

    Ok(RunoffCore {
        actual_evapotranspiration_mm,
        local_runoff_mm,
        runoff_fraction,
        local_runoff_m3_s,
        potential_discharge_m3_s,
        land_area_m2,
        precipitation_area_sum_mm_m2,
        aet_area_sum_mm_m2,
        runoff_area_sum_mm_m2,
        maximum_land_runoff_mm,
        total_local_runoff_m3_s,
        terminal_discharge_m3_s,
        discharge_conservation_relative_error,
        maximum_potential_discharge_m3_s,
    })
}

pub fn generate_runoff_discharge(
    topology: &GeodesicTopology,
    topography: &TopographyState,
    climate: &ClimateState,
    drainage: &DrainageState,
    planet: PlanetPhysicalParameters,
    request: &RunoffRequest,
) -> Result<RunoffState, WorldgenError> {
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
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-6B inputs must align on the canonical fine topology",
        ));
    }
    if drainage.receiver.len() != count as usize
        || drainage.drainage_order.len() != drainage.metrics.land_sample_count as usize
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-6B requires a complete WG-6A drainage graph",
        ));
    }
    if climate.annual_precipitation_mm.len() != count as usize
        || climate.potential_evaporation_mm.len() != count as usize
    {
        return Err(WorldgenError::InvalidHydrology(
            "WG-6B climate forcing fields must align with the canonical fine topology",
        ));
    }

    let core = solve_runoff_core(
        topology,
        &topography.submerged_mask,
        &climate.annual_precipitation_mm,
        &climate.potential_evaporation_mm,
        &drainage.receiver,
        &drainage.drainage_order,
        planet.radius_m,
        planet.orbital_period_s,
        request.parameters,
    )
    .map_err(WorldgenError::InvalidHydrology)?;

    let stage_seed = derive_stage_seed(&request.seed, RUNOFF_NAMESPACE);
    let runoff_parameter_hash = request.parameters.parameter_hash();
    let mut runoff_hash = FNV_OFFSET_BASIS;
    runoff_hash = fnv_update(runoff_hash, RUNOFF_STAGE_ID.as_bytes());
    runoff_hash = fnv_update(runoff_hash, &RUNOFF_STAGE_VERSION.to_le_bytes());
    runoff_hash = fnv_update(runoff_hash, &stage_seed.to_le_bytes());
    runoff_hash = fnv_update(runoff_hash, &planet.parameter_hash().to_le_bytes());
    runoff_hash = fnv_update(runoff_hash, &runoff_parameter_hash.to_le_bytes());
    runoff_hash = fnv_update(runoff_hash, &climate.metrics.climate_hash.to_le_bytes());
    runoff_hash = fnv_update(runoff_hash, &drainage.metrics.drainage_hash.to_le_bytes());
    runoff_hash = hash_f32_slice(runoff_hash, &core.actual_evapotranspiration_mm);
    runoff_hash = hash_f32_slice(runoff_hash, &core.local_runoff_mm);
    runoff_hash = hash_f32_slice(runoff_hash, &core.runoff_fraction);
    runoff_hash = hash_f32_slice(runoff_hash, &core.local_runoff_m3_s);
    runoff_hash = hash_f32_slice(runoff_hash, &core.potential_discharge_m3_s);

    let land_area_m2 = core.land_area_m2;
    let mean_land_precipitation_mm = if land_area_m2 > 0.0 {
        core.precipitation_area_sum_mm_m2 / land_area_m2
    } else {
        0.0
    };
    let mean_land_actual_evapotranspiration_mm = if land_area_m2 > 0.0 {
        core.aet_area_sum_mm_m2 / land_area_m2
    } else {
        0.0
    };
    let mean_land_runoff_mm = if land_area_m2 > 0.0 {
        core.runoff_area_sum_mm_m2 / land_area_m2
    } else {
        0.0
    };
    let land_runoff_fraction = if core.precipitation_area_sum_mm_m2 > 0.0 {
        (core.runoff_area_sum_mm_m2 / core.precipitation_area_sum_mm_m2).clamp(0.0, 1.0)
    } else {
        0.0
    };

    Ok(RunoffState {
        stage: StageIdentity {
            id: RUNOFF_STAGE_ID,
            version: RUNOFF_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics: RunoffMetrics {
            sample_count: count,
            land_sample_count: drainage.metrics.land_sample_count,
            land_area_m2,
            mean_land_precipitation_mm,
            mean_land_actual_evapotranspiration_mm,
            mean_land_runoff_mm,
            maximum_land_runoff_mm: core.maximum_land_runoff_mm,
            land_runoff_fraction,
            total_local_runoff_m3_s: core.total_local_runoff_m3_s,
            terminal_discharge_m3_s: core.terminal_discharge_m3_s,
            discharge_conservation_relative_error: core.discharge_conservation_relative_error,
            maximum_potential_discharge_m3_s: core.maximum_potential_discharge_m3_s,
            runoff_parameter_hash,
            climate_hash: climate.metrics.climate_hash,
            drainage_hash: drainage.metrics.drainage_hash,
            runoff_hash,
        },
        actual_evapotranspiration_mm: core.actual_evapotranspiration_mm,
        local_runoff_mm: core.local_runoff_mm,
        runoff_fraction: core.runoff_fraction,
        local_runoff_m3_s: core.local_runoff_m3_s,
        potential_discharge_m3_s: core.potential_discharge_m3_s,
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

    #[test]
    fn budyko_curve_respects_water_and_energy_limits() {
        let omega = 2.6;
        assert_eq!(budyko_actual_evapotranspiration(0.0, 1000.0, omega), 0.0);
        assert_eq!(budyko_actual_evapotranspiration(1000.0, 0.0, omega), 0.0);
        let arid = budyko_actual_evapotranspiration(100.0, 1000.0, omega);
        assert!(arid > 90.0 && arid <= 100.0);
        let humid = budyko_actual_evapotranspiration(2000.0, 500.0, omega);
        assert!(humid > 0.0 && humid <= 500.0);
    }

    #[test]
    fn annual_runoff_accumulates_downstream_and_conserves_discharge() {
        let topology = TestTopology::chain(4);
        let submerged = [0_u8, 0, 0, 1];
        let precipitation = [1000.0_f32, 1000.0, 1000.0, 0.0];
        let pet = [0.0_f32; 4];
        let receiver = [1_u32, 2, 3, INVALID_SAMPLE_ID];
        let order = [0_u32, 1, 2];
        let core = solve_runoff_core(
            &topology,
            &submerged,
            &precipitation,
            &pet,
            &receiver,
            &order,
            1.0,
            1.0,
            RunoffParameters::default(),
        )
        .unwrap();

        assert!((core.local_runoff_m3_s[0] - 1.0).abs() < 1.0e-6);
        assert!((core.potential_discharge_m3_s[0] - 1.0).abs() < 1.0e-6);
        assert!((core.potential_discharge_m3_s[1] - 2.0).abs() < 1.0e-6);
        assert!((core.potential_discharge_m3_s[2] - 3.0).abs() < 1.0e-6);
        assert!((core.potential_discharge_m3_s[3] - 3.0).abs() < 1.0e-6);
        assert!((core.total_local_runoff_m3_s - 3.0).abs() < 1.0e-6);
        assert!((core.terminal_discharge_m3_s - 3.0).abs() < 1.0e-6);
        assert!(core.discharge_conservation_relative_error < 1.0e-6);
    }

    #[test]
    fn dry_cells_produce_zero_runoff_and_discharge() {
        let topology = TestTopology::chain(3);
        let core = solve_runoff_core(
            &topology,
            &[0, 0, 1],
            &[0.0, 0.0, 0.0],
            &[500.0, 500.0, 0.0],
            &[1, 2, INVALID_SAMPLE_ID],
            &[0, 1],
            1.0,
            1.0,
            RunoffParameters::default(),
        )
        .unwrap();
        assert!(core.local_runoff_m3_s.iter().all(|value| *value == 0.0));
        assert!(core.potential_discharge_m3_s.iter().all(|value| *value == 0.0));
        assert_eq!(core.discharge_conservation_relative_error, 0.0);
    }
}
