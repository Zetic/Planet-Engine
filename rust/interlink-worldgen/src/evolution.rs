use crate::drainage::generate_drainage_from_surface;
use crate::{
    derive_stage_seed, DrainageRequest, DrainageState, FluvialErosionState, GeodesicTopology,
    LakeState, PlanetPhysicalParameters, PlanetTopology, RunoffState, StageIdentity, TopographyState,
    WorldgenError, INVALID_SAMPLE_ID,
};

pub const TERRAIN_EVOLUTION_STAGE_ID: &str = "geomorphology:bounded-terrain-evolution";
pub const TERRAIN_EVOLUTION_STAGE_VERSION: u32 = 1;
const TERRAIN_EVOLUTION_NAMESPACE: &str = "geomorphology:bounded-terrain-evolution:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainEvolutionParameters {
    /// Upper bound on the single direct geomorphic horizon. WG-7B does not year-step to this value.
    pub maximum_geomorphic_years: f64,
    /// Global adaptive cap on either resolved erosion or resolved land deposition in one solve.
    pub maximum_resolved_elevation_change_m: f64,
    /// Expands WG-7A hydraulic width into a coarse-grid valley footprint.
    pub valley_width_multiplier: f64,
    pub minimum_valley_width_m: f64,
    pub maximum_valley_width_m: f64,
    /// Existing land is kept above the fixed WG-4 sea level by at least this amount.
    pub minimum_land_clearance_m: f64,
    /// Prevents a source cell from being incised below its accepted downstream base level.
    pub minimum_receiver_relief_m: f64,
    /// First-model effective source/deposit densities. Keeping them equal makes applied volume
    /// conservation directly inspectable while the sediment mass ledger remains authoritative.
    pub eroded_material_density_kg_m3: f64,
    pub deposited_sediment_density_kg_m3: f64,
}

impl Default for TerrainEvolutionParameters {
    fn default() -> Self {
        Self {
            maximum_geomorphic_years: 50_000.0,
            maximum_resolved_elevation_change_m: 120.0,
            valley_width_multiplier: 3.0,
            minimum_valley_width_m: 100.0,
            maximum_valley_width_m: 20_000.0,
            minimum_land_clearance_m: 1.0,
            minimum_receiver_relief_m: 0.1,
            eroded_material_density_kg_m3: 1_800.0,
            deposited_sediment_density_kg_m3: 1_800.0,
        }
    }
}

impl TerrainEvolutionParameters {
    pub fn validate(&self) -> Result<(), &'static str> {
        let positive = [
            self.maximum_geomorphic_years,
            self.maximum_resolved_elevation_change_m,
            self.valley_width_multiplier,
            self.minimum_valley_width_m,
            self.maximum_valley_width_m,
            self.minimum_land_clearance_m,
            self.minimum_receiver_relief_m,
            self.eroded_material_density_kg_m3,
            self.deposited_sediment_density_kg_m3,
        ];
        if positive
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err("WG-7B terrain-evolution parameters must be finite and positive");
        }
        if self.minimum_valley_width_m > self.maximum_valley_width_m {
            return Err("WG-7B minimum valley width must not exceed maximum valley width");
        }
        if self.maximum_geomorphic_years > 10_000_000.0 {
            return Err("WG-7B direct geomorphic horizon exceeds supported bound");
        }
        if self.maximum_resolved_elevation_change_m > 5_000.0 {
            return Err("WG-7B resolved elevation-change cap exceeds supported bound");
        }
        Ok(())
    }

    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        for value in [
            self.maximum_geomorphic_years,
            self.maximum_resolved_elevation_change_m,
            self.valley_width_multiplier,
            self.minimum_valley_width_m,
            self.maximum_valley_width_m,
            self.minimum_land_clearance_m,
            self.minimum_receiver_relief_m,
            self.eroded_material_density_kg_m3,
            self.deposited_sediment_density_kg_m3,
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
pub struct TerrainEvolutionRequest {
    pub seed: String,
    pub parameters: TerrainEvolutionParameters,
}

impl TerrainEvolutionRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            parameters: TerrainEvolutionParameters::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TerrainEvolutionMetrics {
    pub sample_count: u32,
    pub geomorphic_duration_years: f64,
    pub eroded_sample_count: u32,
    pub depositional_sample_count: u32,
    pub receiver_changed_sample_count: u32,
    pub receiver_changed_fraction: f64,
    pub maximum_applied_erosion_m: f64,
    pub maximum_applied_deposition_m: f64,
    pub maximum_absolute_terrain_change_m: f64,
    pub mean_land_absolute_terrain_change_m: f64,
    pub total_applied_sediment_generated_kg_s: f64,
    pub total_land_deposition_kg_s: f64,
    pub total_lake_sink_kg_s: f64,
    pub total_terminal_ocean_sink_kg_s: f64,
    pub sediment_conservation_relative_error: f64,
    pub maximum_post_erosion_potential_discharge_m3_s: f64,
    pub post_erosion_runoff_conservation_relative_error: f64,
    pub evolution_parameter_hash: u64,
    pub topography_hash: u64,
    pub drainage_hash: u64,
    pub runoff_hash: u64,
    pub lake_hash: u64,
    pub fluvial_erosion_hash: u64,
    pub evolved_surface_hash: u64,
    pub post_erosion_drainage_hash: u64,
    pub terrain_evolution_hash: u64,
}

impl TerrainEvolutionMetrics {
    pub fn evolution_parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution_parameter_hash)
    }
    pub fn evolved_surface_hash_hex(&self) -> String {
        format!("{:016x}", self.evolved_surface_hash)
    }
    pub fn post_erosion_drainage_hash_hex(&self) -> String {
        format!("{:016x}", self.post_erosion_drainage_hash)
    }
    pub fn terrain_evolution_hash_hex(&self) -> String {
        format!("{:016x}", self.terrain_evolution_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TerrainEvolutionState {
    pub stage: StageIdentity,
    pub metrics: TerrainEvolutionMetrics,
    /// Final WG-7B solid surface. WG-4 remains immutable upstream truth.
    pub evolved_solid_elevation_m: Vec<f32>,
    pub terrain_delta_m: Vec<f32>,
    pub applied_erosion_m: Vec<f32>,
    pub applied_deposition_m: Vec<f32>,
    /// Average applied source mass rate over the chosen direct geomorphic horizon.
    pub applied_sediment_supply_kg_s: Vec<f32>,
    pub applied_sediment_load_kg_s: Vec<f32>,
    /// Ordinary land deposition only; lake and terminal/ocean sinks do not alter WG-7B terrain.
    pub applied_land_deposition_kg_s: Vec<f32>,
    /// 1 where the rebuilt WG-6A receiver differs from the accepted pre-erosion receiver.
    pub receiver_changed_mask: Vec<u8>,
    /// Rebuilt drainage on the evolved land surface and fixed WG-4 ocean mask.
    pub post_erosion_drainage: DrainageState,
    /// Accepted WG-6B local runoff rerouted once over the rebuilt drainage DAG.
    pub post_erosion_potential_discharge_m3_s: Vec<f32>,
}

#[derive(Debug)]
struct AppliedSedimentRouting {
    load_kg_s: Vec<f32>,
    land_deposition_kg_s: Vec<f32>,
    total_land_deposition_kg_s: f64,
    total_lake_sink_kg_s: f64,
    total_terminal_ocean_sink_kg_s: f64,
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

fn adaptive_duration_years(maximum_resolved_rate_m_per_year: f64, p: TerrainEvolutionParameters) -> f64 {
    if maximum_resolved_rate_m_per_year <= 0.0 {
        return 0.0;
    }
    p.maximum_geomorphic_years.min(
        p.maximum_resolved_elevation_change_m / maximum_resolved_rate_m_per_year,
    )
}

#[allow(clippy::too_many_arguments)]
fn route_applied_sediment(
    source_kg_s: &[f32],
    transport_capacity_kg_s: &[f32],
    submerged_mask: &[u8],
    receiver: &[u32],
    drainage_order: &[u32],
    depression_id: &[u32],
    active_lake_depression: &[bool],
) -> Result<AppliedSedimentRouting, &'static str> {
    let count = source_kg_s.len();
    if transport_capacity_kg_s.len() != count
        || submerged_mask.len() != count
        || receiver.len() != count
        || depression_id.len() != count
    {
        return Err("WG-7B applied sediment fields must align with topology dimensions");
    }
    let mut incoming = vec![0.0_f64; count];
    let mut load_kg_s = vec![0.0_f32; count];
    let mut land_deposition_kg_s = vec![0.0_f32; count];
    let mut total_land_deposition_kg_s = 0.0_f64;
    let mut total_lake_sink_kg_s = 0.0_f64;
    let mut total_terminal_ocean_sink_kg_s = 0.0_f64;

    for &sample in drainage_order {
        let i = sample as usize;
        if i >= count || submerged_mask[i] != 0 {
            return Err("WG-7B drainage order contains an invalid land sample");
        }
        let source = f64::from(source_kg_s[i]);
        let capacity = f64::from(transport_capacity_kg_s[i]);
        if !source.is_finite() || source < 0.0 || !capacity.is_finite() || capacity < 0.0 {
            return Err("WG-7B sediment source/capacity must be finite and non-negative");
        }
        let available = incoming[i] + source;
        if !available.is_finite() {
            return Err("WG-7B routed sediment exceeded finite range");
        }

        let depression = depression_id[i];
        let active_lake = depression != INVALID_SAMPLE_ID
            && (depression as usize) < active_lake_depression.len()
            && active_lake_depression[depression as usize];
        if active_lake {
            total_lake_sink_kg_s += available;
            continue;
        }

        let downstream = receiver[i];
        if downstream == INVALID_SAMPLE_ID {
            total_terminal_ocean_sink_kg_s += available;
            continue;
        }
        let downstream_index = downstream as usize;
        if downstream_index >= count {
            return Err("WG-7B receiver points outside topology");
        }

        let carried = available.min(capacity);
        let deposited = available - carried;
        land_deposition_kg_s[i] = deposited as f32;
        total_land_deposition_kg_s += deposited;
        if submerged_mask[downstream_index] != 0 {
            total_terminal_ocean_sink_kg_s += carried;
        } else {
            load_kg_s[i] = carried as f32;
            incoming[downstream_index] += carried;
        }
    }

    Ok(AppliedSedimentRouting {
        load_kg_s,
        land_deposition_kg_s,
        total_land_deposition_kg_s,
        total_lake_sink_kg_s,
        total_terminal_ocean_sink_kg_s,
    })
}

fn reroute_local_runoff(
    local_runoff_m3_s: &[f32],
    submerged_mask: &[u8],
    drainage: &DrainageState,
) -> Result<(Vec<f32>, f64, f64), &'static str> {
    let count = local_runoff_m3_s.len();
    if submerged_mask.len() != count || drainage.receiver.len() != count {
        return Err("WG-7B runoff reroute fields must align with topology dimensions");
    }
    let mut accumulated = vec![0.0_f64; count];
    let mut total_local = 0.0_f64;
    for i in 0..count {
        if submerged_mask[i] == 0 {
            let value = f64::from(local_runoff_m3_s[i]);
            if !value.is_finite() || value < 0.0 {
                return Err("WG-7B local runoff must be finite and non-negative");
            }
            accumulated[i] = value;
            total_local += value;
        }
    }
    for &sample in &drainage.drainage_order {
        let i = sample as usize;
        let downstream = drainage.receiver[i];
        if downstream != INVALID_SAMPLE_ID {
            accumulated[downstream as usize] += accumulated[i];
        }
    }
    let mut terminal = 0.0_f64;
    let mut maximum = 0.0_f64;
    for i in 0..count {
        maximum = maximum.max(accumulated[i]);
        if submerged_mask[i] != 0 || drainage.receiver[i] == INVALID_SAMPLE_ID {
            terminal += accumulated[i];
        }
    }
    let relative_error = if total_local > 0.0 {
        (terminal - total_local).abs() / total_local
    } else {
        terminal.abs()
    };
    Ok((
        accumulated.into_iter().map(|value| value as f32).collect(),
        maximum,
        relative_error,
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn generate_bounded_terrain_evolution(
    topology: &GeodesicTopology,
    topography: &TopographyState,
    drainage: &DrainageState,
    runoff: &RunoffState,
    lakes: &LakeState,
    erosion: &FluvialErosionState,
    planet: PlanetPhysicalParameters,
    request: &TerrainEvolutionRequest,
) -> Result<TerrainEvolutionState, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    request
        .parameters
        .validate()
        .map_err(WorldgenError::InvalidGeomorphology)?;

    let count = topology.sample_count() as usize;
    if topography.metrics.sample_count as usize != count
        || drainage.metrics.sample_count as usize != count
        || runoff.metrics.sample_count as usize != count
        || lakes.metrics.sample_count as usize != count
        || erosion.metrics.sample_count as usize != count
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7B inputs must align on the canonical fine topology",
        ));
    }
    if erosion.metrics.topography_hash != topography.metrics.topography_hash
        || erosion.metrics.drainage_hash != drainage.metrics.drainage_hash
        || erosion.metrics.lake_hash != lakes.metrics.lake_hash
        || runoff.metrics.drainage_hash != drainage.metrics.drainage_hash
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7B requires exact accepted WG-4/WG-6/WG-7A ancestry",
        ));
    }
    let required_lengths = [
        topography.solid_elevation_m.len(),
        topography.submerged_mask.len(),
        drainage.receiver.len(),
        drainage.depression_id.len(),
        runoff.local_runoff_m3_s.len(),
        erosion.channel_width_m.len(),
        erosion.incision_potential_m_per_year.len(),
        erosion.sediment_transport_capacity_kg_s.len(),
    ];
    if required_lengths.iter().any(|length| *length != count) {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7B requires complete upstream terrain/hydrology/erosion fields",
        ));
    }

    let p = request.parameters;
    let year_seconds = planet.orbital_period_s;
    let sea_level = topography.metrics.sea_level_m;
    let mut active_lake_depression = vec![false; drainage.depressions.len()];
    for lake in &lakes.lakes {
        let depression = lake.depression_id as usize;
        if depression >= active_lake_depression.len() {
            return Err(WorldgenError::InvalidGeomorphology(
                "WG-7B lake references an unknown accepted depression",
            ));
        }
        active_lake_depression[depression] = true;
    }

    let mut raw_erosion_rate_m_year = vec![0.0_f64; count];
    let mut provisional_source_kg_s = vec![0.0_f32; count];
    for i in 0..count {
        if topography.submerged_mask[i] != 0 {
            continue;
        }
        let incision = f64::from(erosion.incision_potential_m_per_year[i]).max(0.0);
        let channel_width = f64::from(erosion.channel_width_m[i]).max(0.0);
        let downstream = drainage.receiver[i];
        if incision <= 0.0 || channel_width <= 0.0 || downstream == INVALID_SAMPLE_ID {
            continue;
        }
        let neighbors = topology.neighbors_of(i as u32);
        let arcs = topology.neighbor_arc_lengths_of(i as u32);
        let edge = neighbors
            .iter()
            .position(|sample| *sample == downstream)
            .ok_or(WorldgenError::InvalidGeomorphology(
                "WG-7B accepted receiver is not adjacent on the canonical topology",
            ))?;
        let segment_length_m = arcs[edge] * planet.radius_m;
        let cell_area_m2 = topology.area_steradians(i as u32) * planet.radius_m * planet.radius_m;
        let valley_width_m = (channel_width * p.valley_width_multiplier)
            .clamp(p.minimum_valley_width_m, p.maximum_valley_width_m);
        let valley_area_m2 = (segment_length_m * valley_width_m).min(cell_area_m2);
        let resolved_rate = incision * valley_area_m2 / cell_area_m2;
        raw_erosion_rate_m_year[i] = resolved_rate;
        let source = resolved_rate * cell_area_m2 * p.eroded_material_density_kg_m3 / year_seconds;
        if !source.is_finite() || source < 0.0 || source > f32::MAX as f64 {
            return Err(WorldgenError::InvalidGeomorphology(
                "WG-7B provisional sediment source exceeds representable range",
            ));
        }
        provisional_source_kg_s[i] = source as f32;
    }

    let provisional_routing = route_applied_sediment(
        &provisional_source_kg_s,
        &erosion.sediment_transport_capacity_kg_s,
        &topography.submerged_mask,
        &drainage.receiver,
        &drainage.drainage_order,
        &drainage.depression_id,
        &active_lake_depression,
    )
    .map_err(WorldgenError::InvalidGeomorphology)?;

    let mut maximum_resolved_rate_m_year = raw_erosion_rate_m_year
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    for i in 0..count {
        if topography.submerged_mask[i] != 0 {
            continue;
        }
        let area_m2 = topology.area_steradians(i as u32) * planet.radius_m * planet.radius_m;
        let deposition_rate = f64::from(provisional_routing.land_deposition_kg_s[i])
            * year_seconds
            / (area_m2 * p.deposited_sediment_density_kg_m3);
        maximum_resolved_rate_m_year = maximum_resolved_rate_m_year.max(deposition_rate);
    }
    let geomorphic_duration_years = adaptive_duration_years(maximum_resolved_rate_m_year, p);
    let duration_seconds = geomorphic_duration_years * year_seconds;

    let mut applied_erosion_m = vec![0.0_f32; count];
    let mut applied_sediment_supply_kg_s = vec![0.0_f32; count];
    let mut eroded_sample_count = 0_u32;
    let mut maximum_applied_erosion_m = 0.0_f64;
    let mut total_applied_sediment_generated_kg_s = 0.0_f64;
    if geomorphic_duration_years > 0.0 {
        for i in 0..count {
            if topography.submerged_mask[i] != 0 || raw_erosion_rate_m_year[i] <= 0.0 {
                continue;
            }
            let current = f64::from(topography.solid_elevation_m[i]);
            let sea_floor = sea_level
                .map(|level| level + p.minimum_land_clearance_m)
                .unwrap_or(f64::NEG_INFINITY);
            let downstream = drainage.receiver[i];
            let receiver_floor = if downstream == INVALID_SAMPLE_ID {
                f64::NEG_INFINITY
            } else if topography.submerged_mask[downstream as usize] != 0 {
                sea_floor
            } else {
                f64::from(topography.solid_elevation_m[downstream as usize])
                    + p.minimum_receiver_relief_m
            };
            let floor = sea_floor.max(receiver_floor);
            let available_relief_m = (current - floor).max(0.0);
            let requested = raw_erosion_rate_m_year[i] * geomorphic_duration_years;
            let erosion_depth = requested.min(available_relief_m);
            if erosion_depth <= 0.0 {
                continue;
            }
            applied_erosion_m[i] = erosion_depth as f32;
            eroded_sample_count += 1;
            maximum_applied_erosion_m = maximum_applied_erosion_m.max(erosion_depth);
            let area_m2 = topology.area_steradians(i as u32) * planet.radius_m * planet.radius_m;
            let source = erosion_depth * area_m2 * p.eroded_material_density_kg_m3
                / duration_seconds;
            applied_sediment_supply_kg_s[i] = source as f32;
            total_applied_sediment_generated_kg_s += f64::from(applied_sediment_supply_kg_s[i]);
        }
    }

    let routing = route_applied_sediment(
        &applied_sediment_supply_kg_s,
        &erosion.sediment_transport_capacity_kg_s,
        &topography.submerged_mask,
        &drainage.receiver,
        &drainage.drainage_order,
        &drainage.depression_id,
        &active_lake_depression,
    )
    .map_err(WorldgenError::InvalidGeomorphology)?;

    let total_sink = routing.total_land_deposition_kg_s
        + routing.total_lake_sink_kg_s
        + routing.total_terminal_ocean_sink_kg_s;
    let sediment_conservation_relative_error = if total_applied_sediment_generated_kg_s > 0.0 {
        (total_sink - total_applied_sediment_generated_kg_s).abs()
            / total_applied_sediment_generated_kg_s
    } else {
        total_sink.abs()
    };

    let mut applied_deposition_m = vec![0.0_f32; count];
    let mut evolved_solid_elevation_m = topography.solid_elevation_m.clone();
    let mut terrain_delta_m = vec![0.0_f32; count];
    let mut depositional_sample_count = 0_u32;
    let mut maximum_applied_deposition_m = 0.0_f64;
    let mut maximum_absolute_terrain_change_m = 0.0_f64;
    let mut land_area_m2 = 0.0_f64;
    let mut absolute_change_area_sum = 0.0_f64;

    for i in 0..count {
        if topography.submerged_mask[i] != 0 {
            continue;
        }
        let area_m2 = topology.area_steradians(i as u32) * planet.radius_m * planet.radius_m;
        let deposition_depth = if duration_seconds > 0.0 {
            f64::from(routing.land_deposition_kg_s[i]) * duration_seconds
                / (area_m2 * p.deposited_sediment_density_kg_m3)
        } else {
            0.0
        };
        if deposition_depth > 0.0 {
            depositional_sample_count += 1;
            maximum_applied_deposition_m = maximum_applied_deposition_m.max(deposition_depth);
        }
        applied_deposition_m[i] = deposition_depth as f32;
        let delta = deposition_depth - f64::from(applied_erosion_m[i]);
        let evolved = f64::from(topography.solid_elevation_m[i]) + delta;
        evolved_solid_elevation_m[i] = evolved as f32;
        terrain_delta_m[i] = delta as f32;
        maximum_absolute_terrain_change_m = maximum_absolute_terrain_change_m.max(delta.abs());
        land_area_m2 += area_m2;
        absolute_change_area_sum += delta.abs() * area_m2;
    }
    let mean_land_absolute_terrain_change_m = if land_area_m2 > 0.0 {
        absolute_change_area_sum / land_area_m2
    } else {
        0.0
    };

    let stage_seed = derive_stage_seed(&request.seed, TERRAIN_EVOLUTION_NAMESPACE);
    let evolution_parameter_hash = p.parameter_hash();
    let mut evolved_surface_hash = FNV_OFFSET_BASIS;
    evolved_surface_hash = fnv_update(evolved_surface_hash, TERRAIN_EVOLUTION_STAGE_ID.as_bytes());
    evolved_surface_hash = fnv_update(
        evolved_surface_hash,
        &TERRAIN_EVOLUTION_STAGE_VERSION.to_le_bytes(),
    );
    evolved_surface_hash = fnv_update(evolved_surface_hash, &stage_seed.to_le_bytes());
    evolved_surface_hash = fnv_update(evolved_surface_hash, &evolution_parameter_hash.to_le_bytes());
    evolved_surface_hash = fnv_update(evolved_surface_hash, &topography.metrics.topography_hash.to_le_bytes());
    evolved_surface_hash = fnv_update(evolved_surface_hash, &drainage.metrics.drainage_hash.to_le_bytes());
    evolved_surface_hash = fnv_update(evolved_surface_hash, &erosion.metrics.fluvial_erosion_hash.to_le_bytes());
    evolved_surface_hash = hash_f32_slice(evolved_surface_hash, &evolved_solid_elevation_m);

    let post_erosion_drainage = generate_drainage_from_surface(
        topology,
        &evolved_solid_elevation_m,
        &topography.submerged_mask,
        topography.metrics.sea_level_m,
        evolved_surface_hash,
        planet,
        &DrainageRequest::new(request.seed.as_str()),
    )?;

    let mut receiver_changed_mask = vec![0_u8; count];
    let mut receiver_changed_sample_count = 0_u32;
    for i in 0..count {
        if topography.submerged_mask[i] == 0
            && drainage.receiver[i] != post_erosion_drainage.receiver[i]
        {
            receiver_changed_mask[i] = 1;
            receiver_changed_sample_count += 1;
        }
    }
    let receiver_changed_fraction = if drainage.metrics.land_sample_count > 0 {
        f64::from(receiver_changed_sample_count) / f64::from(drainage.metrics.land_sample_count)
    } else {
        0.0
    };

    let (
        post_erosion_potential_discharge_m3_s,
        maximum_post_erosion_potential_discharge_m3_s,
        post_erosion_runoff_conservation_relative_error,
    ) = reroute_local_runoff(
        &runoff.local_runoff_m3_s,
        &topography.submerged_mask,
        &post_erosion_drainage,
    )
    .map_err(WorldgenError::InvalidGeomorphology)?;

    let mut terrain_evolution_hash = evolved_surface_hash;
    terrain_evolution_hash = fnv_update(
        terrain_evolution_hash,
        &post_erosion_drainage.metrics.drainage_hash.to_le_bytes(),
    );
    terrain_evolution_hash = hash_f32_slice(terrain_evolution_hash, &applied_erosion_m);
    terrain_evolution_hash = hash_f32_slice(terrain_evolution_hash, &applied_deposition_m);
    terrain_evolution_hash = hash_f32_slice(terrain_evolution_hash, &applied_sediment_supply_kg_s);
    terrain_evolution_hash = hash_f32_slice(terrain_evolution_hash, &routing.load_kg_s);
    terrain_evolution_hash = hash_f32_slice(terrain_evolution_hash, &routing.land_deposition_kg_s);
    terrain_evolution_hash = hash_u8_slice(terrain_evolution_hash, &receiver_changed_mask);
    terrain_evolution_hash = hash_f32_slice(
        terrain_evolution_hash,
        &post_erosion_potential_discharge_m3_s,
    );

    Ok(TerrainEvolutionState {
        stage: StageIdentity {
            id: TERRAIN_EVOLUTION_STAGE_ID,
            version: TERRAIN_EVOLUTION_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics: TerrainEvolutionMetrics {
            sample_count: count as u32,
            geomorphic_duration_years,
            eroded_sample_count,
            depositional_sample_count,
            receiver_changed_sample_count,
            receiver_changed_fraction,
            maximum_applied_erosion_m,
            maximum_applied_deposition_m,
            maximum_absolute_terrain_change_m,
            mean_land_absolute_terrain_change_m,
            total_applied_sediment_generated_kg_s,
            total_land_deposition_kg_s: routing.total_land_deposition_kg_s,
            total_lake_sink_kg_s: routing.total_lake_sink_kg_s,
            total_terminal_ocean_sink_kg_s: routing.total_terminal_ocean_sink_kg_s,
            sediment_conservation_relative_error,
            maximum_post_erosion_potential_discharge_m3_s,
            post_erosion_runoff_conservation_relative_error,
            evolution_parameter_hash,
            topography_hash: topography.metrics.topography_hash,
            drainage_hash: drainage.metrics.drainage_hash,
            runoff_hash: runoff.metrics.runoff_hash,
            lake_hash: lakes.metrics.lake_hash,
            fluvial_erosion_hash: erosion.metrics.fluvial_erosion_hash,
            evolved_surface_hash,
            post_erosion_drainage_hash: post_erosion_drainage.metrics.drainage_hash,
            terrain_evolution_hash,
        },
        evolved_solid_elevation_m,
        terrain_delta_m,
        applied_erosion_m,
        applied_deposition_m,
        applied_sediment_supply_kg_s,
        applied_sediment_load_kg_s: routing.load_kg_s,
        applied_land_deposition_kg_s: routing.land_deposition_kg_s,
        receiver_changed_mask,
        post_erosion_drainage,
        post_erosion_potential_discharge_m3_s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_horizon_caps_resolved_change_without_time_stepping() {
        let p = TerrainEvolutionParameters {
            maximum_geomorphic_years: 50_000.0,
            maximum_resolved_elevation_change_m: 100.0,
            ..TerrainEvolutionParameters::default()
        };
        assert_eq!(adaptive_duration_years(0.0, p), 0.0);
        assert!((adaptive_duration_years(0.001, p) - 50_000.0).abs() < 1.0e-9);
        assert!((adaptive_duration_years(0.01, p) - 10_000.0).abs() < 1.0e-9);
    }

    #[test]
    fn applied_sediment_routes_capacity_limited_land_deposition_and_ocean_sink() {
        let routed = route_applied_sediment(
            &[10.0, 0.0, 0.0],
            &[6.0, 4.0, 0.0],
            &[0, 0, 1],
            &[1, 2, INVALID_SAMPLE_ID],
            &[0, 1],
            &[INVALID_SAMPLE_ID; 3],
            &[],
        )
        .unwrap();
        assert!((routed.total_land_deposition_kg_s - 6.0).abs() < 1.0e-9);
        assert!((routed.total_terminal_ocean_sink_kg_s - 4.0).abs() < 1.0e-9);
        assert_eq!(routed.load_kg_s[0], 6.0);
        assert_eq!(routed.load_kg_s[1], 0.0);
    }

    #[test]
    fn active_lake_depression_is_a_complete_applied_sediment_sink() {
        let routed = route_applied_sediment(
            &[5.0, 0.0, 0.0],
            &[10.0, 10.0, 0.0],
            &[0, 0, 1],
            &[1, 2, INVALID_SAMPLE_ID],
            &[0, 1],
            &[INVALID_SAMPLE_ID, 0, INVALID_SAMPLE_ID],
            &[true],
        )
        .unwrap();
        assert!((routed.total_lake_sink_kg_s - 5.0).abs() < 1.0e-9);
        assert_eq!(routed.land_deposition_kg_s, vec![0.0, 0.0, 0.0]);
    }
}
