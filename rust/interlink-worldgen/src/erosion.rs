use crate::{
    derive_stage_seed, DrainageState, GeodesicTopology, InheritedPhysicalState, LakeState,
    PlanetPhysicalParameters, PlanetTopology, SeasonalHydrologyState, StageIdentity,
    TopographyState, WorldgenError, INVALID_SAMPLE_ID,
};

pub const FLUVIAL_EROSION_STAGE_ID: &str = "geomorphology:fluvial-erosion-sediment";
pub const FLUVIAL_EROSION_STAGE_VERSION: u32 = 1;
const FLUVIAL_EROSION_NAMESPACE: &str = "geomorphology:fluvial-erosion-sediment:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const MINIMUM_EDGE_DISTANCE_M: f64 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FluvialErosionParameters {
    /// Power mean used to derive a peak-sensitive erosive discharge from WG-6D phases.
    pub effective_discharge_power: f64,
    /// Reference discharge used only to nondimensionalize stream-power forcing.
    pub reference_discharge_m3_s: f64,
    /// Reference channel slope used only to nondimensionalize stream-power forcing.
    pub reference_slope: f64,
    pub discharge_exponent: f64,
    pub slope_exponent: f64,
    /// Upper bound for diagnostic incision potential before terrain mutation is introduced.
    pub maximum_incision_m_per_year: f64,
    /// Fraction of a dual cell treated as actively channelized for sediment-production accounting.
    pub channelized_area_fraction: f64,
    pub sediment_bulk_density_kg_m3: f64,
    /// Reference transport capacity at the reference discharge and slope.
    pub reference_transport_capacity_kg_s: f64,
    pub transport_discharge_exponent: f64,
    pub transport_slope_exponent: f64,
}

impl Default for FluvialErosionParameters {
    fn default() -> Self {
        Self {
            effective_discharge_power: 2.0,
            reference_discharge_m3_s: 100.0,
            reference_slope: 0.01,
            discharge_exponent: 0.5,
            slope_exponent: 1.0,
            maximum_incision_m_per_year: 0.01,
            channelized_area_fraction: 2.5e-4,
            sediment_bulk_density_kg_m3: 1_800.0,
            reference_transport_capacity_kg_s: 50.0,
            transport_discharge_exponent: 1.2,
            transport_slope_exponent: 1.0,
        }
    }
}

impl FluvialErosionParameters {
    pub fn validate(&self) -> Result<(), &'static str> {
        let positive = [
            self.effective_discharge_power,
            self.reference_discharge_m3_s,
            self.reference_slope,
            self.discharge_exponent,
            self.slope_exponent,
            self.maximum_incision_m_per_year,
            self.channelized_area_fraction,
            self.sediment_bulk_density_kg_m3,
            self.reference_transport_capacity_kg_s,
            self.transport_discharge_exponent,
            self.transport_slope_exponent,
        ];
        if positive
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err("WG-7A erosion parameters must be finite and positive");
        }
        if !(1.0..=8.0).contains(&self.effective_discharge_power) {
            return Err("WG-7A effective-discharge power must be within [1, 8]");
        }
        if self.channelized_area_fraction > 0.1 {
            return Err("WG-7A channelized area fraction must not exceed 0.1");
        }
        if self.maximum_incision_m_per_year > 10.0 {
            return Err("WG-7A maximum diagnostic incision must not exceed 10 m/yr");
        }
        Ok(())
    }

    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        for value in [
            self.effective_discharge_power,
            self.reference_discharge_m3_s,
            self.reference_slope,
            self.discharge_exponent,
            self.slope_exponent,
            self.maximum_incision_m_per_year,
            self.channelized_area_fraction,
            self.sediment_bulk_density_kg_m3,
            self.reference_transport_capacity_kg_s,
            self.transport_discharge_exponent,
            self.transport_slope_exponent,
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
pub struct FluvialErosionRequest {
    pub seed: String,
    pub parameters: FluvialErosionParameters,
}

impl FluvialErosionRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            parameters: FluvialErosionParameters::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FluvialErosionMetrics {
    pub sample_count: u32,
    pub orbital_phase_count: u8,
    pub erosive_sample_count: u32,
    pub active_lake_trap_count: u32,
    pub maximum_effective_discharge_m3_s: f64,
    pub maximum_channel_slope: f64,
    pub maximum_incision_potential_m_per_year: f64,
    pub total_sediment_generated_kg_s: f64,
    pub total_land_deposition_kg_s: f64,
    pub total_lake_deposition_kg_s: f64,
    pub total_terminal_ocean_deposition_kg_s: f64,
    pub maximum_sediment_load_kg_s: f64,
    pub sediment_conservation_relative_error: f64,
    pub erosion_parameter_hash: u64,
    pub inheritance_hash: u64,
    pub topography_hash: u64,
    pub drainage_hash: u64,
    pub lake_hash: u64,
    pub seasonal_hydrology_hash: u64,
    pub fluvial_erosion_hash: u64,
}

impl FluvialErosionMetrics {
    pub fn erosion_parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.erosion_parameter_hash)
    }
    pub fn inheritance_hash_hex(&self) -> String {
        format!("{:016x}", self.inheritance_hash)
    }
    pub fn topography_hash_hex(&self) -> String {
        format!("{:016x}", self.topography_hash)
    }
    pub fn drainage_hash_hex(&self) -> String {
        format!("{:016x}", self.drainage_hash)
    }
    pub fn lake_hash_hex(&self) -> String {
        format!("{:016x}", self.lake_hash)
    }
    pub fn seasonal_hydrology_hash_hex(&self) -> String {
        format!("{:016x}", self.seasonal_hydrology_hash)
    }
    pub fn fluvial_erosion_hash_hex(&self) -> String {
        format!("{:016x}", self.fluvial_erosion_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FluvialErosionState {
    pub stage: StageIdentity,
    pub metrics: FluvialErosionMetrics,
    /// Peak-sensitive power mean of WG-6D realized discharge over retained phases.
    pub effective_discharge_m3_s: Vec<f32>,
    /// Positive physical fall toward the accepted WG-6A receiver divided by edge distance.
    pub channel_slope: Vec<f32>,
    /// Dimensionless inherited rock erodibility in [0.05, 1].
    pub erodibility_index: Vec<f32>,
    /// Dimensionless peak-sensitive stream-power forcing before the bounded response transform.
    pub stream_power_index: Vec<f32>,
    /// Diagnostic fluvial incision potential. WG-7A does not apply this to terrain.
    pub incision_potential_m_per_year: Vec<f32>,
    /// Locally generated sediment supply implied by the diagnostic incision potential.
    pub local_sediment_supply_kg_s: Vec<f32>,
    /// Parameterized local carrying capacity on the accepted drainage graph.
    pub sediment_transport_capacity_kg_s: Vec<f32>,
    /// Sediment load leaving each land sample after local deposition/trapping.
    pub sediment_load_kg_s: Vec<f32>,
    /// Sediment deposited locally, in active lake control volumes, or at terminal/ocean sinks.
    pub sediment_deposition_kg_s: Vec<f32>,
}

#[derive(Debug)]
struct SedimentRoutingMetrics {
    total_land_deposition_kg_s: f64,
    total_lake_deposition_kg_s: f64,
    total_terminal_ocean_deposition_kg_s: f64,
    maximum_load_kg_s: f64,
    active_lake_trap_count: u32,
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

fn effective_discharge(phases: &[f32], power: f64) -> Result<f64, &'static str> {
    if phases.is_empty() {
        return Err("WG-7A effective discharge requires at least one orbital phase");
    }
    let mut sum = 0.0_f64;
    for value in phases {
        let q = f64::from(*value);
        if !q.is_finite() || q < 0.0 {
            return Err("WG-7A realized discharge must be finite and non-negative");
        }
        sum += q.powf(power);
    }
    Ok((sum / phases.len() as f64).powf(1.0 / power))
}

fn inherited_erodibility(strength: f32, weakness: f32, fabric: f32, fragmentation: f32) -> f64 {
    let strength = f64::from(strength).clamp(0.0, 1.0);
    let weakness = f64::from(weakness).clamp(0.0, 1.0);
    let fabric = f64::from(fabric).clamp(0.0, 1.0);
    let fragmentation = f64::from(fragmentation).clamp(0.0, 1.0);
    (0.05 + 0.35 * weakness + 0.25 * fragmentation + 0.15 * fabric + 0.20 * (1.0 - strength))
        .clamp(0.05, 1.0)
}

fn bounded_incision_response(
    effective_discharge_m3_s: f64,
    slope: f64,
    erodibility: f64,
    parameters: FluvialErosionParameters,
) -> (f64, f64) {
    if effective_discharge_m3_s <= 0.0 || slope <= 0.0 {
        return (0.0, 0.0);
    }
    let q = (effective_discharge_m3_s / parameters.reference_discharge_m3_s).max(0.0);
    let s = (slope / parameters.reference_slope).max(0.0);
    let forcing = q.powf(parameters.discharge_exponent) * s.powf(parameters.slope_exponent);
    let bounded = forcing / (1.0 + forcing);
    let incision = parameters.maximum_incision_m_per_year * erodibility * bounded;
    (forcing, incision)
}

#[allow(clippy::too_many_arguments)]
fn route_sediment(
    local_supply_kg_s: &[f32],
    transport_capacity_kg_s: &[f32],
    submerged_mask: &[u8],
    receiver: &[u32],
    drainage_order: &[u32],
    active_lake_depression: &[bool],
    depression_id: &[u32],
    sediment_load_kg_s: &mut [f32],
    sediment_deposition_kg_s: &mut [f32],
) -> Result<SedimentRoutingMetrics, &'static str> {
    let count = local_supply_kg_s.len();
    if transport_capacity_kg_s.len() != count
        || submerged_mask.len() != count
        || receiver.len() != count
        || depression_id.len() != count
        || sediment_load_kg_s.len() != count
        || sediment_deposition_kg_s.len() != count
    {
        return Err("WG-7A sediment-routing fields must align with topology dimensions");
    }

    let mut incoming = vec![0.0_f64; count];
    let mut lake_trapped = vec![false; active_lake_depression.len()];
    let mut metrics = SedimentRoutingMetrics {
        total_land_deposition_kg_s: 0.0,
        total_lake_deposition_kg_s: 0.0,
        total_terminal_ocean_deposition_kg_s: 0.0,
        maximum_load_kg_s: 0.0,
        active_lake_trap_count: 0,
    };

    for &sample in drainage_order {
        let i = sample as usize;
        if i >= count || submerged_mask[i] != 0 {
            return Err("WG-7A drainage order contains an invalid land sample");
        }
        let local = f64::from(local_supply_kg_s[i]);
        let capacity = f64::from(transport_capacity_kg_s[i]);
        if !local.is_finite() || local < 0.0 || !capacity.is_finite() || capacity < 0.0 {
            return Err("WG-7A sediment supply/capacity must be finite and non-negative");
        }
        let available = incoming[i] + local;
        if !available.is_finite() {
            return Err("WG-7A sediment load overflowed finite range");
        }

        let depression = depression_id[i];
        let is_active_lake = depression != INVALID_SAMPLE_ID
            && (depression as usize) < active_lake_depression.len()
            && active_lake_depression[depression as usize];
        if is_active_lake {
            sediment_deposition_kg_s[i] = available as f32;
            metrics.total_lake_deposition_kg_s += available;
            if !lake_trapped[depression as usize] && available > 0.0 {
                lake_trapped[depression as usize] = true;
                metrics.active_lake_trap_count += 1;
            }
            continue;
        }

        let carried = available.min(capacity);
        let local_deposition = available - carried;
        sediment_load_kg_s[i] = carried as f32;
        sediment_deposition_kg_s[i] = local_deposition as f32;
        metrics.total_land_deposition_kg_s += local_deposition;
        metrics.maximum_load_kg_s = metrics.maximum_load_kg_s.max(carried);

        let downstream = receiver[i];
        if downstream == INVALID_SAMPLE_ID {
            sediment_load_kg_s[i] = 0.0;
            sediment_deposition_kg_s[i] = (local_deposition + carried) as f32;
            metrics.total_land_deposition_kg_s -= local_deposition;
            metrics.total_terminal_ocean_deposition_kg_s += available;
            continue;
        }
        let downstream_index = downstream as usize;
        if downstream_index >= count {
            return Err("WG-7A drainage receiver points outside topology");
        }
        if submerged_mask[downstream_index] != 0 {
            sediment_load_kg_s[i] = 0.0;
            sediment_deposition_kg_s[i] = local_deposition as f32;
            sediment_deposition_kg_s[downstream_index] += carried as f32;
            metrics.total_terminal_ocean_deposition_kg_s += carried;
            incoming[downstream_index] += 0.0;
        } else {
            incoming[downstream_index] += carried;
        }
    }

    Ok(metrics)
}

#[allow(clippy::too_many_arguments)]
pub fn generate_fluvial_erosion_sediment(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    topography: &TopographyState,
    drainage: &DrainageState,
    lakes: &LakeState,
    seasonal: &SeasonalHydrologyState,
    planet: PlanetPhysicalParameters,
    request: &FluvialErosionRequest,
) -> Result<FluvialErosionState, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    request
        .parameters
        .validate()
        .map_err(WorldgenError::InvalidGeomorphology)?;

    let count = topology.sample_count() as usize;
    let phase_count = usize::from(seasonal.metrics.orbital_phase_count);
    if inherited.map.metrics.fine_sample_count as usize != count
        || topography.metrics.sample_count as usize != count
        || drainage.metrics.sample_count as usize != count
        || lakes.metrics.sample_count as usize != count
        || seasonal.metrics.sample_count as usize != count
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7A inputs must align on the canonical fine topology",
        ));
    }
    if seasonal.metrics.drainage_hash != drainage.metrics.drainage_hash
        || seasonal.metrics.lake_hash != lakes.metrics.lake_hash
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7A requires WG-6D state derived from the accepted WG-6A/WG-6C ancestry",
        ));
    }
    if phase_count == 0 || seasonal.phase_realized_discharge_m3_s.len() != count * phase_count {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7A requires the complete WG-6D phase realized-discharge field",
        ));
    }
    let inherited_lengths = [
        inherited.strength_index.len(),
        inherited.weakness_index.len(),
        inherited.structural_fabric_strength.len(),
        inherited.fragmentation_propensity.len(),
    ];
    if inherited_lengths.iter().any(|length| *length != count)
        || topography.solid_elevation_m.len() != count
        || topography.submerged_mask.len() != count
        || drainage.receiver.len() != count
        || drainage.depression_id.len() != count
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7A physical forcing fields are incomplete",
        ));
    }

    let mut effective_discharge_m3_s = vec![0.0_f32; count];
    let mut channel_slope = vec![0.0_f32; count];
    let mut erodibility_index = vec![0.0_f32; count];
    let mut stream_power_index = vec![0.0_f32; count];
    let mut incision_potential_m_per_year = vec![0.0_f32; count];
    let mut local_sediment_supply_kg_s = vec![0.0_f32; count];
    let mut sediment_transport_capacity_kg_s = vec![0.0_f32; count];
    let mut sediment_load_kg_s = vec![0.0_f32; count];
    let mut sediment_deposition_kg_s = vec![0.0_f32; count];

    let mut erosive_sample_count = 0_u32;
    let mut maximum_effective_discharge_m3_s = 0.0_f64;
    let mut maximum_channel_slope = 0.0_f64;
    let mut maximum_incision_potential_m_per_year = 0.0_f64;
    let mut total_sediment_generated_kg_s = 0.0_f64;

    for i in 0..count {
        if topography.submerged_mask[i] != 0 {
            continue;
        }
        let mut phase_values = Vec::with_capacity(phase_count);
        for phase in 0..phase_count {
            phase_values.push(seasonal.phase_realized_discharge_m3_s[phase * count + i]);
        }
        let q_eff =
            effective_discharge(&phase_values, request.parameters.effective_discharge_power)
                .map_err(WorldgenError::InvalidGeomorphology)?;
        effective_discharge_m3_s[i] = q_eff as f32;
        maximum_effective_discharge_m3_s = maximum_effective_discharge_m3_s.max(q_eff);

        let downstream = drainage.receiver[i];
        let slope = if downstream == INVALID_SAMPLE_ID {
            0.0
        } else {
            let downstream_index = downstream as usize;
            if downstream_index >= count {
                return Err(WorldgenError::InvalidGeomorphology(
                    "WG-7A drainage receiver points outside topology",
                ));
            }
            let neighbors = topology.neighbors_of(i as u32);
            let arcs = topology.neighbor_arc_lengths_of(i as u32);
            let edge = neighbors
                .iter()
                .position(|sample| *sample == downstream)
                .ok_or(WorldgenError::InvalidGeomorphology(
                    "WG-7A receiver is not adjacent on the canonical topology",
                ))?;
            let distance_m = (arcs[edge] * planet.radius_m).max(MINIMUM_EDGE_DISTANCE_M);
            let fall_m = f64::from(topography.solid_elevation_m[i])
                - f64::from(topography.solid_elevation_m[downstream_index]);
            (fall_m / distance_m).max(0.0)
        };
        channel_slope[i] = slope as f32;
        maximum_channel_slope = maximum_channel_slope.max(slope);

        let erodibility = inherited_erodibility(
            inherited.strength_index[i],
            inherited.weakness_index[i],
            inherited.structural_fabric_strength[i],
            inherited.fragmentation_propensity[i],
        );
        erodibility_index[i] = erodibility as f32;

        let (stream_power, incision) =
            bounded_incision_response(q_eff, slope, erodibility, request.parameters);
        stream_power_index[i] = stream_power.min(f32::MAX as f64) as f32;
        incision_potential_m_per_year[i] = incision as f32;
        maximum_incision_potential_m_per_year = maximum_incision_potential_m_per_year.max(incision);
        if incision > 0.0 {
            erosive_sample_count += 1;
        }

        let area_m2 = topology.area_steradians(i as u32) * planet.radius_m * planet.radius_m;
        if !area_m2.is_finite() || area_m2 <= 0.0 {
            return Err(WorldgenError::InvalidGeomorphology(
                "WG-7A dual-cell area must be finite and positive",
            ));
        }
        let incision_m_s = incision / planet.orbital_period_s;
        let supply = incision_m_s
            * area_m2
            * request.parameters.channelized_area_fraction
            * request.parameters.sediment_bulk_density_kg_m3;
        if !supply.is_finite() || supply < 0.0 || supply > f32::MAX as f64 {
            return Err(WorldgenError::InvalidGeomorphology(
                "WG-7A sediment production exceeds representable range",
            ));
        }
        local_sediment_supply_kg_s[i] = supply as f32;
        total_sediment_generated_kg_s += f64::from(local_sediment_supply_kg_s[i]);

        if q_eff > 0.0 && slope > 0.0 {
            let q = (q_eff / request.parameters.reference_discharge_m3_s).max(0.0);
            let s = (slope / request.parameters.reference_slope).max(0.0);
            let capacity = request.parameters.reference_transport_capacity_kg_s
                * q.powf(request.parameters.transport_discharge_exponent)
                * s.powf(request.parameters.transport_slope_exponent);
            if !capacity.is_finite() || capacity < 0.0 || capacity > f32::MAX as f64 {
                return Err(WorldgenError::InvalidGeomorphology(
                    "WG-7A sediment transport capacity exceeds representable range",
                ));
            }
            sediment_transport_capacity_kg_s[i] = capacity as f32;
        }
    }

    let mut active_lake_depression = vec![false; drainage.depressions.len()];
    for lake in &lakes.lakes {
        let depression = lake.depression_id as usize;
        if depression >= active_lake_depression.len() {
            return Err(WorldgenError::InvalidGeomorphology(
                "WG-7A lake references an unknown drainage depression",
            ));
        }
        active_lake_depression[depression] = true;
    }

    let routing = route_sediment(
        &local_sediment_supply_kg_s,
        &sediment_transport_capacity_kg_s,
        &topography.submerged_mask,
        &drainage.receiver,
        &drainage.drainage_order,
        &active_lake_depression,
        &drainage.depression_id,
        &mut sediment_load_kg_s,
        &mut sediment_deposition_kg_s,
    )
    .map_err(WorldgenError::InvalidGeomorphology)?;

    let total_deposition = routing.total_land_deposition_kg_s
        + routing.total_lake_deposition_kg_s
        + routing.total_terminal_ocean_deposition_kg_s;
    let sediment_conservation_relative_error = if total_sediment_generated_kg_s > 0.0 {
        (total_deposition - total_sediment_generated_kg_s).abs() / total_sediment_generated_kg_s
    } else {
        total_deposition.abs()
    };

    let stage_seed = derive_stage_seed(&request.seed, FLUVIAL_EROSION_NAMESPACE);
    let erosion_parameter_hash = request.parameters.parameter_hash();
    let mut fluvial_erosion_hash = FNV_OFFSET_BASIS;
    fluvial_erosion_hash = fnv_update(fluvial_erosion_hash, FLUVIAL_EROSION_STAGE_ID.as_bytes());
    fluvial_erosion_hash = fnv_update(
        fluvial_erosion_hash,
        &FLUVIAL_EROSION_STAGE_VERSION.to_le_bytes(),
    );
    for value in [
        stage_seed,
        erosion_parameter_hash,
        inherited.inheritance_hash,
        topography.metrics.topography_hash,
        drainage.metrics.drainage_hash,
        lakes.metrics.lake_hash,
        seasonal.metrics.seasonal_hydrology_hash,
    ] {
        fluvial_erosion_hash = fnv_update(fluvial_erosion_hash, &value.to_le_bytes());
    }
    fluvial_erosion_hash = hash_f32_slice(fluvial_erosion_hash, &effective_discharge_m3_s);
    fluvial_erosion_hash = hash_f32_slice(fluvial_erosion_hash, &channel_slope);
    fluvial_erosion_hash = hash_f32_slice(fluvial_erosion_hash, &erodibility_index);
    fluvial_erosion_hash = hash_f32_slice(fluvial_erosion_hash, &stream_power_index);
    fluvial_erosion_hash = hash_f32_slice(fluvial_erosion_hash, &incision_potential_m_per_year);
    fluvial_erosion_hash = hash_f32_slice(fluvial_erosion_hash, &local_sediment_supply_kg_s);
    fluvial_erosion_hash = hash_f32_slice(fluvial_erosion_hash, &sediment_transport_capacity_kg_s);
    fluvial_erosion_hash = hash_f32_slice(fluvial_erosion_hash, &sediment_load_kg_s);
    fluvial_erosion_hash = hash_f32_slice(fluvial_erosion_hash, &sediment_deposition_kg_s);

    Ok(FluvialErosionState {
        stage: StageIdentity {
            id: FLUVIAL_EROSION_STAGE_ID,
            version: FLUVIAL_EROSION_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics: FluvialErosionMetrics {
            sample_count: count as u32,
            orbital_phase_count: seasonal.metrics.orbital_phase_count,
            erosive_sample_count,
            active_lake_trap_count: routing.active_lake_trap_count,
            maximum_effective_discharge_m3_s,
            maximum_channel_slope,
            maximum_incision_potential_m_per_year,
            total_sediment_generated_kg_s,
            total_land_deposition_kg_s: routing.total_land_deposition_kg_s,
            total_lake_deposition_kg_s: routing.total_lake_deposition_kg_s,
            total_terminal_ocean_deposition_kg_s: routing.total_terminal_ocean_deposition_kg_s,
            maximum_sediment_load_kg_s: routing.maximum_load_kg_s,
            sediment_conservation_relative_error,
            erosion_parameter_hash,
            inheritance_hash: inherited.inheritance_hash,
            topography_hash: topography.metrics.topography_hash,
            drainage_hash: drainage.metrics.drainage_hash,
            lake_hash: lakes.metrics.lake_hash,
            seasonal_hydrology_hash: seasonal.metrics.seasonal_hydrology_hash,
            fluvial_erosion_hash,
        },
        effective_discharge_m3_s,
        channel_slope,
        erodibility_index,
        stream_power_index,
        incision_potential_m_per_year,
        local_sediment_supply_kg_s,
        sediment_transport_capacity_kg_s,
        sediment_load_kg_s,
        sediment_deposition_kg_s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flashy_hydrograph_has_greater_peak_sensitive_effective_discharge() {
        let steady = effective_discharge(&[10.0_f32, 10.0], 2.0).unwrap();
        let flashy = effective_discharge(&[0.0_f32, 20.0], 2.0).unwrap();
        assert!((steady - 10.0).abs() < 1.0e-12);
        assert!(flashy > steady);
    }

    #[test]
    fn inherited_weakness_and_fragmentation_raise_erodibility() {
        let strong = inherited_erodibility(0.95, 0.05, 0.05, 0.05);
        let weak = inherited_erodibility(0.15, 0.85, 0.75, 0.90);
        assert!(weak > strong);
        assert!((0.05..=1.0).contains(&strong));
        assert!((0.05..=1.0).contains(&weak));
    }

    #[test]
    fn bounded_stream_power_never_exceeds_parameterized_incision_ceiling() {
        let parameters = FluvialErosionParameters::default();
        let (_, incision) = bounded_incision_response(1.0e9, 2.0, 1.0, parameters);
        assert!(incision > 0.0);
        assert!(incision <= parameters.maximum_incision_m_per_year);
    }

    #[test]
    fn sediment_router_conserves_mass_and_traps_active_lake_depression() {
        let local = [10.0_f32, 0.0, 0.0, 0.0];
        let capacity = [10.0_f32, 10.0, 10.0, 0.0];
        let submerged = [0_u8, 0, 0, 1];
        let receiver = [1_u32, 2, 3, INVALID_SAMPLE_ID];
        let order = [0_u32, 1, 2];
        let depression = [INVALID_SAMPLE_ID, 0, 0, INVALID_SAMPLE_ID];
        let active = [true];
        let mut load = [0.0_f32; 4];
        let mut deposition = [0.0_f32; 4];
        let metrics = route_sediment(
            &local,
            &capacity,
            &submerged,
            &receiver,
            &order,
            &active,
            &depression,
            &mut load,
            &mut deposition,
        )
        .unwrap();
        let deposited = deposition
            .iter()
            .map(|value| f64::from(*value))
            .sum::<f64>();
        assert!((deposited - 10.0).abs() < 1.0e-12);
        assert!((metrics.total_lake_deposition_kg_s - 10.0).abs() < 1.0e-12);
        assert_eq!(metrics.active_lake_trap_count, 1);
        assert_eq!(load[1], 0.0);
    }

    #[test]
    fn sediment_router_deposits_export_at_ocean_sink() {
        let local = [6.0_f32, 0.0];
        let capacity = [6.0_f32, 0.0];
        let submerged = [0_u8, 1];
        let receiver = [1_u32, INVALID_SAMPLE_ID];
        let order = [0_u32];
        let depression = [INVALID_SAMPLE_ID, INVALID_SAMPLE_ID];
        let mut load = [0.0_f32; 2];
        let mut deposition = [0.0_f32; 2];
        let metrics = route_sediment(
            &local,
            &capacity,
            &submerged,
            &receiver,
            &order,
            &[],
            &depression,
            &mut load,
            &mut deposition,
        )
        .unwrap();
        assert!((f64::from(deposition[1]) - 6.0).abs() < 1.0e-12);
        assert!((metrics.total_terminal_ocean_deposition_kg_s - 6.0).abs() < 1.0e-12);
    }
}
