use crate::{derive_stage_seed, tangent_basis, GeodesicTopology, PlanetPhysicalParameters, StageIdentity, TopographyState, WorldgenError};

const CLIMATE_NAMESPACE: &str = "climate:v1";
pub const CLIMATE_STAGE_ID: &str = "climate:coupled-surface";
pub const CLIMATE_STAGE_VERSION: u32 = 1;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;
const UNIVERSAL_GAS_CONSTANT: f64 = 8.314_462_618;
const TWO_PI: f64 = std::f64::consts::PI * 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClimatePhysicalParameters {
    pub orbital_eccentricity: f64,
    pub longitude_of_periapsis_rad: f64,
    pub atmospheric_mean_molar_mass_kg_per_mol: f64,
    pub atmospheric_specific_heat_j_per_kg_k: f64,
    pub atmospheric_longwave_optical_depth: f64,
}

impl ClimatePhysicalParameters {
    pub const fn earthlike_reference() -> Self {
        Self {
            orbital_eccentricity: 0.0167,
            longitude_of_periapsis_rad: 1.796_767_421_176_181_3,
            atmospheric_mean_molar_mass_kg_per_mol: 0.028_964_7,
            atmospheric_specific_heat_j_per_kg_k: 1_004.0,
            atmospheric_longwave_optical_depth: 0.90,
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.orbital_eccentricity.is_finite()
            || !(0.0..0.95).contains(&self.orbital_eccentricity)
        {
            return Err("climate orbital eccentricity must be finite and within [0, 0.95)");
        }
        if !self.longitude_of_periapsis_rad.is_finite() {
            return Err("climate longitude of periapsis must be finite");
        }
        if !self.atmospheric_mean_molar_mass_kg_per_mol.is_finite()
            || self.atmospheric_mean_molar_mass_kg_per_mol <= 0.0
        {
            return Err("atmospheric mean molar mass must be finite and positive");
        }
        if !self.atmospheric_specific_heat_j_per_kg_k.is_finite()
            || self.atmospheric_specific_heat_j_per_kg_k <= 0.0
        {
            return Err("atmospheric specific heat must be finite and positive");
        }
        if !self.atmospheric_longwave_optical_depth.is_finite()
            || self.atmospheric_longwave_optical_depth < 0.0
        {
            return Err("atmospheric longwave optical depth must be finite and non-negative");
        }
        Ok(())
    }

    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        for value in [
            self.orbital_eccentricity,
            self.longitude_of_periapsis_rad,
            self.atmospheric_mean_molar_mass_kg_per_mol,
            self.atmospheric_specific_heat_j_per_kg_k,
            self.atmospheric_longwave_optical_depth,
        ] {
            hash = fnv_update(hash, &value.to_bits().to_le_bytes());
        }
        hash
    }

    pub fn parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.parameter_hash())
    }
}

impl Default for ClimatePhysicalParameters {
    fn default() -> Self {
        Self::earthlike_reference()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClimateParameters {
    pub orbital_phase_count: u8,
    pub minimum_spinup_years: u8,
    pub maximum_spinup_years: u8,
    pub convergence_temperature_rms_k: f64,
    pub land_albedo: f64,
    pub ocean_albedo: f64,
    pub snow_ice_albedo: f64,
    pub snow_albedo_feedback: f64,
    pub lapse_rate_k_per_m: f64,
    pub land_thermal_relaxation: f64,
    pub ocean_thermal_relaxation: f64,
    pub atmospheric_heat_relaxation: f64,
    pub air_sea_exchange_relaxation: f64,
    pub wind_thermal_gradient_scale: f64,
    pub topographic_wind_drag: f64,
    pub maximum_wind_speed_m_s: f64,
    pub ocean_wind_coupling: f64,
    pub ocean_coriolis_deflection: f64,
    pub ocean_bathymetric_drag_depth_m: f64,
    pub ocean_current_smoothing: f64,
    pub ocean_current_correction_iterations: u8,
    pub maximum_surface_current_m_s: f64,
    pub ocean_temperature_diffusion: f64,
    pub ocean_advection_relaxation: f64,
    pub evaporation_relaxation: f64,
    pub moisture_transport_cfl: f64,
    pub condensation_relative_humidity: f64,
    pub condensation_efficiency: f64,
    pub orographic_precipitation_strength: f64,
    pub maximum_orographic_fraction: f64,
    pub snow_temperature_k: f64,
    pub sea_ice_temperature_k: f64,
}

impl Default for ClimateParameters {
    fn default() -> Self {
        Self {
            orbital_phase_count: 24,
            minimum_spinup_years: 4,
            maximum_spinup_years: 10,
            convergence_temperature_rms_k: 0.08,
            land_albedo: 0.24,
            ocean_albedo: 0.07,
            snow_ice_albedo: 0.62,
            snow_albedo_feedback: 0.32,
            lapse_rate_k_per_m: 0.0065,
            land_thermal_relaxation: 0.38,
            ocean_thermal_relaxation: 0.12,
            atmospheric_heat_relaxation: 0.16,
            air_sea_exchange_relaxation: 0.14,
            wind_thermal_gradient_scale: 0.72,
            topographic_wind_drag: 42.0,
            maximum_wind_speed_m_s: 65.0,
            ocean_wind_coupling: 0.035,
            ocean_coriolis_deflection: 0.55,
            ocean_bathymetric_drag_depth_m: 700.0,
            ocean_current_smoothing: 0.18,
            ocean_current_correction_iterations: 4,
            maximum_surface_current_m_s: 2.8,
            ocean_temperature_diffusion: 0.08,
            ocean_advection_relaxation: 0.025,
            evaporation_relaxation: 0.055,
            moisture_transport_cfl: 0.025,
            condensation_relative_humidity: 0.80,
            condensation_efficiency: 0.72,
            orographic_precipitation_strength: 13.0,
            maximum_orographic_fraction: 0.32,
            snow_temperature_k: 273.15,
            sea_ice_temperature_k: 271.35,
        }
    }
}

impl ClimateParameters {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(4..=48).contains(&self.orbital_phase_count) {
            return Err("climate orbital phase count must be from 4 through 48");
        }
        if self.minimum_spinup_years == 0
            || self.maximum_spinup_years < self.minimum_spinup_years
            || self.maximum_spinup_years > 32
        {
            return Err("climate spinup year bounds are invalid");
        }
        let positive = [
            self.convergence_temperature_rms_k,
            self.lapse_rate_k_per_m,
            self.wind_thermal_gradient_scale,
            self.maximum_wind_speed_m_s,
            self.ocean_wind_coupling,
            self.ocean_bathymetric_drag_depth_m,
            self.maximum_surface_current_m_s,
            self.evaporation_relaxation,
            self.moisture_transport_cfl,
            self.orographic_precipitation_strength,
            self.snow_temperature_k,
            self.sea_ice_temperature_k,
        ];
        if positive.iter().any(|value| !value.is_finite() || *value <= 0.0) {
            return Err("climate positive model parameters must be finite and positive");
        }
        let unit_interval = [
            self.land_albedo,
            self.ocean_albedo,
            self.snow_ice_albedo,
            self.snow_albedo_feedback,
            self.land_thermal_relaxation,
            self.ocean_thermal_relaxation,
            self.atmospheric_heat_relaxation,
            self.air_sea_exchange_relaxation,
            self.ocean_coriolis_deflection,
            self.ocean_current_smoothing,
            self.ocean_temperature_diffusion,
            self.ocean_advection_relaxation,
            self.condensation_relative_humidity,
            self.condensation_efficiency,
            self.maximum_orographic_fraction,
        ];
        if unit_interval
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err("climate bounded model parameters must be finite and within [0, 1]");
        }
        if !self.topographic_wind_drag.is_finite() || self.topographic_wind_drag < 0.0 {
            return Err("climate topographic wind drag must be finite and non-negative");
        }
        if self.ocean_current_correction_iterations > 24 {
            return Err("ocean current correction iterations exceed supported bound");
        }
        Ok(())
    }

    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        hash = fnv_update(hash, &[self.orbital_phase_count]);
        hash = fnv_update(hash, &[self.minimum_spinup_years]);
        hash = fnv_update(hash, &[self.maximum_spinup_years]);
        hash = fnv_update(hash, &[self.ocean_current_correction_iterations]);
        for value in [
            self.convergence_temperature_rms_k,
            self.land_albedo,
            self.ocean_albedo,
            self.snow_ice_albedo,
            self.snow_albedo_feedback,
            self.lapse_rate_k_per_m,
            self.land_thermal_relaxation,
            self.ocean_thermal_relaxation,
            self.atmospheric_heat_relaxation,
            self.air_sea_exchange_relaxation,
            self.wind_thermal_gradient_scale,
            self.topographic_wind_drag,
            self.maximum_wind_speed_m_s,
            self.ocean_wind_coupling,
            self.ocean_coriolis_deflection,
            self.ocean_bathymetric_drag_depth_m,
            self.ocean_current_smoothing,
            self.maximum_surface_current_m_s,
            self.ocean_temperature_diffusion,
            self.ocean_advection_relaxation,
            self.evaporation_relaxation,
            self.moisture_transport_cfl,
            self.condensation_relative_humidity,
            self.condensation_efficiency,
            self.orographic_precipitation_strength,
            self.maximum_orographic_fraction,
            self.snow_temperature_k,
            self.sea_ice_temperature_k,
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
pub struct ClimateRequest {
    pub seed: String,
    pub physical: ClimatePhysicalParameters,
    pub parameters: ClimateParameters,
}

impl ClimateRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            physical: ClimatePhysicalParameters::default(),
            parameters: ClimateParameters::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClimateMetrics {
    pub sample_count: u32,
    pub orbital_phase_count: u8,
    pub spinup_years: u8,
    pub mean_temperature_k: f64,
    pub minimum_temperature_k: f64,
    pub maximum_temperature_k: f64,
    pub mean_land_temperature_k: f64,
    pub mean_ocean_temperature_k: f64,
    pub mean_wind_speed_m_s: f64,
    pub maximum_wind_speed_m_s: f64,
    pub mean_surface_current_m_s: f64,
    pub maximum_surface_current_m_s: f64,
    pub ocean_divergence_residual_m_s: f64,
    pub mean_sea_surface_temperature_k: f64,
    pub mean_annual_precipitation_mm: f64,
    pub p95_annual_precipitation_mm: f64,
    pub global_evaporation_kg: f64,
    pub global_precipitation_kg: f64,
    pub moisture_budget_relative_error: f64,
    pub persistent_snow_area_fraction: f64,
    pub sea_ice_area_fraction: f64,
    pub final_temperature_rms_change_k: f64,
    pub climate_physical_parameter_hash: u64,
    pub climate_model_parameter_hash: u64,
    pub climate_hash: u64,
}

impl ClimateMetrics {
    pub fn climate_physical_parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.climate_physical_parameter_hash)
    }
    pub fn climate_model_parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.climate_model_parameter_hash)
    }
    pub fn climate_hash_hex(&self) -> String {
        format!("{:016x}", self.climate_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClimateState {
    pub stage: StageIdentity,
    pub metrics: ClimateMetrics,
    pub annual_mean_insolation_w_m2: Vec<f32>,
    pub seasonal_insolation_amplitude_w_m2: Vec<f32>,
    pub temperature_mean_k: Vec<f32>,
    pub temperature_annual_cos_k: Vec<f32>,
    pub temperature_annual_sin_k: Vec<f32>,
    pub temperature_min_k: Vec<f32>,
    pub temperature_max_k: Vec<f32>,
    pub local_pressure_pa: Vec<f32>,
    pub wind_east_mean_m_s: Vec<f32>,
    pub wind_north_mean_m_s: Vec<f32>,
    pub wind_east_annual_cos_m_s: Vec<f32>,
    pub wind_east_annual_sin_m_s: Vec<f32>,
    pub wind_north_annual_cos_m_s: Vec<f32>,
    pub wind_north_annual_sin_m_s: Vec<f32>,
    pub sea_surface_temperature_mean_k: Vec<f32>,
    pub current_east_mean_m_s: Vec<f32>,
    pub current_north_mean_m_s: Vec<f32>,
    pub current_speed_mean_m_s: Vec<f32>,
    pub ocean_heat_transport_index: Vec<f32>,
    pub specific_humidity_mean: Vec<f32>,
    pub annual_precipitation_mm: Vec<f32>,
    pub precipitation_seasonality: Vec<f32>,
    pub potential_evaporation_mm: Vec<f32>,
    pub moisture_balance_mm: Vec<f32>,
    pub aridity_index: Vec<f32>,
    pub snowfall_fraction: Vec<f32>,
    pub persistent_snow_potential: Vec<f32>,
    pub sea_ice_potential: Vec<f32>,
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

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm2(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

fn percentile(values: &[f32], fraction: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.iter().copied().map(f64::from).collect::<Vec<_>>();
    sorted.sort_by(f64::total_cmp);
    let index = ((sorted.len() - 1) as f64 * fraction).round() as usize;
    sorted[index]
}

fn area_weighted_mean(topology: &GeodesicTopology, values: &[f32]) -> f64 {
    let mut weighted = 0.0;
    let mut area = 0.0;
    for (index, value) in values.iter().enumerate() {
        let sample_area = topology.dual_area_steradians()[index];
        weighted += sample_area * f64::from(*value);
        area += sample_area;
    }
    if area > 0.0 { weighted / area } else { 0.0 }
}

fn subset_area_weighted_mean(
    topology: &GeodesicTopology,
    values: &[f32],
    predicate: impl Fn(usize) -> bool,
) -> f64 {
    let mut weighted = 0.0;
    let mut area = 0.0;
    for (index, value) in values.iter().enumerate() {
        if !predicate(index) {
            continue;
        }
        let sample_area = topology.dual_area_steradians()[index];
        weighted += sample_area * f64::from(*value);
        area += sample_area;
    }
    if area > 0.0 { weighted / area } else { 0.0 }
}

fn scalar_gradient(
    topology: &GeodesicTopology,
    values: &[f64],
    radius_m: f64,
    sample: usize,
    east: [f64; 3],
    north: [f64; 3],
) -> (f64, f64) {
    let origin = topology.positions()[sample];
    let neighbors = topology.neighbors_of(sample as u32);
    let lengths = topology.neighbor_arc_lengths_of(sample as u32);
    let mut east_gradient = 0.0;
    let mut north_gradient = 0.0;
    let mut weight_sum = 0.0;
    for (neighbor, arc) in neighbors.iter().zip(lengths.iter()) {
        let neighbor_index = *neighbor as usize;
        let neighbor_position = topology.positions()[neighbor_index];
        let radial = dot(neighbor_position, origin);
        let tangent = [
            neighbor_position[0] - origin[0] * radial,
            neighbor_position[1] - origin[1] * radial,
            neighbor_position[2] - origin[2] * radial,
        ];
        let tangent_norm = dot(tangent, tangent).sqrt();
        if tangent_norm <= 1.0e-15 {
            continue;
        }
        let direction = [
            tangent[0] / tangent_norm,
            tangent[1] / tangent_norm,
            tangent[2] / tangent_norm,
        ];
        let distance = (*arc * radius_m).max(1.0);
        let derivative = (values[neighbor_index] - values[sample]) / distance;
        let weight = 1.0 / distance;
        east_gradient += derivative * dot(direction, east) * weight;
        north_gradient += derivative * dot(direction, north) * weight;
        weight_sum += weight;
    }
    if weight_sum > 0.0 {
        (east_gradient / weight_sum, north_gradient / weight_sum)
    } else {
        (0.0, 0.0)
    }
}

fn mean_neighbor(topology: &GeodesicTopology, values: &[f64], sample: usize) -> f64 {
    let neighbors = topology.neighbors_of(sample as u32);
    if neighbors.is_empty() {
        return values[sample];
    }
    neighbors
        .iter()
        .map(|neighbor| values[*neighbor as usize])
        .sum::<f64>()
        / neighbors.len() as f64
}

fn daily_mean_insolation(latitude: f64, declination: f64, stellar_flux: f64) -> f64 {
    let x = -latitude.tan() * declination.tan();
    let hour_angle = if x >= 1.0 {
        0.0
    } else if x <= -1.0 {
        std::f64::consts::PI
    } else {
        x.acos()
    };
    let value = stellar_flux
        / std::f64::consts::PI
        * (hour_angle * latitude.sin() * declination.sin()
            + latitude.cos() * declination.cos() * hour_angle.sin());
    value.max(0.0)
}

fn saturation_specific_humidity(temperature_k: f64, pressure_pa: f64) -> f64 {
    if pressure_pa <= 1.0 || temperature_k <= 100.0 {
        return 0.0;
    }
    let celsius = temperature_k - 273.15;
    let exponent = (17.625 * celsius / (celsius + 243.04)).clamp(-60.0, 60.0);
    let vapor_pressure = (610.94 * exponent.exp()).min(pressure_pa * 0.95);
    let denominator = pressure_pa - 0.378 * vapor_pressure;
    if denominator <= 1.0 {
        0.2
    } else {
        (0.622 * vapor_pressure / denominator).clamp(0.0, 0.2)
    }
}

fn baseline_zonal_wind(latitude_rad: f64) -> f64 {
    let degrees = latitude_rad.abs().to_degrees();
    if degrees < 30.0 {
        -8.0 * (1.0 - 0.35 * degrees / 30.0)
    } else if degrees < 60.0 {
        10.0 * ((degrees - 30.0) / 30.0 * std::f64::consts::PI).sin()
    } else {
        -4.0 * ((degrees - 60.0) / 30.0 * std::f64::consts::FRAC_PI_2)
            .sin()
            .abs()
    }
}

fn clamp_vector(east: f64, north: f64, maximum: f64) -> (f64, f64) {
    let speed = norm2(east, north);
    if speed <= maximum || speed <= f64::EPSILON {
        (east, north)
    } else {
        let scale = maximum / speed;
        (east * scale, north * scale)
    }
}

fn current_divergence(
    topology: &GeodesicTopology,
    ocean: &[bool],
    east_bases: &[[f64; 3]],
    north_bases: &[[f64; 3]],
    current_east: &[f64],
    current_north: &[f64],
) -> Vec<f64> {
    let mut divergence = vec![0.0; ocean.len()];
    for i in 0..ocean.len() {
        if !ocean[i] {
            continue;
        }
        let origin = topology.positions()[i];
        let mut total = 0.0;
        let mut count = 0.0;
        for neighbor in topology.neighbors_of(i as u32) {
            let j = *neighbor as usize;
            if !ocean[j] {
                continue;
            }
            let position = topology.positions()[j];
            let radial = dot(position, origin);
            let tangent = [
                position[0] - origin[0] * radial,
                position[1] - origin[1] * radial,
                position[2] - origin[2] * radial,
            ];
            let magnitude = dot(tangent, tangent).sqrt();
            if magnitude <= 1.0e-15 {
                continue;
            }
            let direction = [
                tangent[0] / magnitude,
                tangent[1] / magnitude,
                tangent[2] / magnitude,
            ];
            let outward = current_east[i] * dot(direction, east_bases[i])
                + current_north[i] * dot(direction, north_bases[i]);
            total += outward;
            count += 1.0;
        }
        if count > 0.0 {
            divergence[i] = total / count;
        }
    }
    divergence
}

fn correct_ocean_currents(
    topology: &GeodesicTopology,
    ocean: &[bool],
    east_bases: &[[f64; 3]],
    north_bases: &[[f64; 3]],
    current_east: &mut [f64],
    current_north: &mut [f64],
    parameters: &ClimateParameters,
) -> f64 {
    for _ in 0..parameters.ocean_current_correction_iterations {
        let divergence = current_divergence(
            topology,
            ocean,
            east_bases,
            north_bases,
            current_east,
            current_north,
        );
        let mut next_east = current_east.to_vec();
        let mut next_north = current_north.to_vec();
        for i in 0..ocean.len() {
            if !ocean[i] {
                continue;
            }
            let neighbors = topology.neighbors_of(i as u32);
            let mut neighbor_east = 0.0;
            let mut neighbor_north = 0.0;
            let mut ocean_neighbors = 0.0;
            for neighbor in neighbors {
                let j = *neighbor as usize;
                if ocean[j] {
                    neighbor_east += current_east[j];
                    neighbor_north += current_north[j];
                    ocean_neighbors += 1.0;
                }
            }
            if ocean_neighbors > 0.0 {
                neighbor_east /= ocean_neighbors;
                neighbor_north /= ocean_neighbors;
                next_east[i] = current_east[i]
                    + parameters.ocean_current_smoothing * (neighbor_east - current_east[i]);
                next_north[i] = current_north[i]
                    + parameters.ocean_current_smoothing * (neighbor_north - current_north[i]);
            }
            let damping = (1.0 - 0.30 * divergence[i].abs().min(1.0)).clamp(0.55, 1.0);
            next_east[i] *= damping;
            next_north[i] *= damping;
            (next_east[i], next_north[i]) = clamp_vector(
                next_east[i],
                next_north[i],
                parameters.maximum_surface_current_m_s,
            );
        }
        current_east.copy_from_slice(&next_east);
        current_north.copy_from_slice(&next_north);
    }
    let residual = current_divergence(
        topology,
        ocean,
        east_bases,
        north_bases,
        current_east,
        current_north,
    );
    let mut total = 0.0;
    let mut count = 0.0;
    for (index, value) in residual.iter().enumerate() {
        if ocean[index] {
            total += value.abs();
            count += 1.0;
        }
    }
    if count > 0.0 { total / count } else { 0.0 }
}

fn validate_inputs(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &ClimateRequest,
) -> Result<(), WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    request
        .physical
        .validate()
        .map_err(WorldgenError::InvalidClimate)?;
    request
        .parameters
        .validate()
        .map_err(WorldgenError::InvalidClimate)?;
    if request.seed.trim().is_empty() {
        return Err(WorldgenError::InvalidClimate("climate seed must not be empty"));
    }
    let sample_count = topology.metrics().sample_count as usize;
    for len in [
        terrain.solid_elevation_m.len(),
        terrain.elevation_above_sea_level_m.len(),
        terrain.water_depth_m.len(),
        terrain.submerged_mask.len(),
    ] {
        if len != sample_count {
            return Err(WorldgenError::InvalidClimate(
                "WG-5 terrain fields must match the climate topology sample count",
            ));
        }
    }
    if terrain
        .elevation_above_sea_level_m
        .iter()
        .chain(terrain.water_depth_m.iter())
        .any(|value| !value.is_finite())
    {
        return Err(WorldgenError::InvalidClimate(
            "WG-5 terrain input contains non-finite values",
        ));
    }
    Ok(())
}

pub fn generate_coupled_climate(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &ClimateRequest,
) -> Result<ClimateState, WorldgenError> {
    validate_inputs(topology, terrain, planet, request)?;
    let parameters = request.parameters;
    let physical = request.physical;
    let sample_count = topology.metrics().sample_count as usize;
    let phase_count = usize::from(parameters.orbital_phase_count);
    let stage_seed = derive_stage_seed(request.seed.as_str(), CLIMATE_NAMESPACE);
    let omega = if planet.rotation_period_s > 0.0 {
        TWO_PI / planet.rotation_period_s
    } else {
        0.0
    };
    let atmosphere_exists = planet.reference_surface_pressure_pa > 0.0;
    let specific_gas_constant = UNIVERSAL_GAS_CONSTANT / physical.atmospheric_mean_molar_mass_kg_per_mol;
    let phase_seconds = planet.orbital_period_s / phase_count as f64;

    let mut latitude = vec![0.0; sample_count];
    let mut east_bases = vec![[0.0; 3]; sample_count];
    let mut north_bases = vec![[0.0; 3]; sample_count];
    let mut cell_area_m2 = vec![0.0; sample_count];
    let mut ocean = vec![false; sample_count];
    let mut terrain_height_m = vec![0.0; sample_count];
    let mut water_depth_m = vec![0.0; sample_count];
    for i in 0..sample_count {
        let position = topology.positions()[i];
        latitude[i] = position[2].clamp(-1.0, 1.0).asin();
        let basis = tangent_basis(position)?;
        east_bases[i] = basis.east;
        north_bases[i] = basis.north;
        cell_area_m2[i] = topology.dual_area_steradians()[i] * planet.radius_m * planet.radius_m;
        ocean[i] = terrain.submerged_mask[i] != 0;
        terrain_height_m[i] = if ocean[i] {
            0.0
        } else {
            f64::from(terrain.elevation_above_sea_level_m[i]).max(0.0)
        };
        water_depth_m[i] = f64::from(terrain.water_depth_m[i]).max(0.0);
    }

    let terrain_values = terrain
        .elevation_above_sea_level_m
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    let mut terrain_gradient_east = vec![0.0; sample_count];
    let mut terrain_gradient_north = vec![0.0; sample_count];
    for i in 0..sample_count {
        let (east, north) = scalar_gradient(
            topology,
            &terrain_values,
            planet.radius_m,
            i,
            east_bases[i],
            north_bases[i],
        );
        terrain_gradient_east[i] = east;
        terrain_gradient_north[i] = north;
    }

    let mut temperature = vec![0.0; sample_count];
    let mut sea_surface_temperature = vec![0.0; sample_count];
    let mut pressure = vec![0.0; sample_count];
    let mut humidity = vec![0.0; sample_count];
    let mut wind_east = vec![0.0; sample_count];
    let mut wind_north = vec![0.0; sample_count];
    let mut current_east = vec![0.0; sample_count];
    let mut current_north = vec![0.0; sample_count];
    for i in 0..sample_count {
        let lat_factor = latitude[i].sin().abs().powf(1.45);
        temperature[i] = (289.0 - 48.0 * lat_factor
            - parameters.lapse_rate_k_per_m * terrain_height_m[i])
            .clamp(170.0, 335.0);
        sea_surface_temperature[i] = if ocean[i] {
            (290.0 - 42.0 * lat_factor).clamp(268.0, 307.0)
        } else {
            temperature[i]
        };
        if atmosphere_exists {
            let scale_height = (specific_gas_constant * temperature[i]
                / planet.surface_gravity_m_s2)
                .max(1.0);
            pressure[i] = planet.reference_surface_pressure_pa
                * (-terrain_height_m[i] / scale_height).exp();
            humidity[i] = if ocean[i] { 0.010 } else { 0.005 };
        }
    }

    let mut annual_mean_insolation = vec![0.0; sample_count];
    let mut insolation_min = vec![f64::INFINITY; sample_count];
    let mut insolation_max = vec![f64::NEG_INFINITY; sample_count];
    let mut temperature_sum = vec![0.0; sample_count];
    let mut temperature_cos = vec![0.0; sample_count];
    let mut temperature_sin = vec![0.0; sample_count];
    let mut temperature_min = vec![f64::INFINITY; sample_count];
    let mut temperature_max = vec![f64::NEG_INFINITY; sample_count];
    let mut wind_east_sum = vec![0.0; sample_count];
    let mut wind_north_sum = vec![0.0; sample_count];
    let mut wind_east_cos = vec![0.0; sample_count];
    let mut wind_east_sin = vec![0.0; sample_count];
    let mut wind_north_cos = vec![0.0; sample_count];
    let mut wind_north_sin = vec![0.0; sample_count];
    let mut sst_sum = vec![0.0; sample_count];
    let mut current_east_sum = vec![0.0; sample_count];
    let mut current_north_sum = vec![0.0; sample_count];
    let mut current_speed_sum = vec![0.0; sample_count];
    let mut ocean_heat_transport_sum = vec![0.0; sample_count];
    let mut humidity_sum = vec![0.0; sample_count];
    let mut precipitation_mass_year = vec![0.0; sample_count];
    let mut precipitation_phase_max = vec![0.0_f64; sample_count];
    let mut potential_evaporation_mass_year = vec![0.0; sample_count];
    let mut cold_precipitation_mass_year = vec![0.0; sample_count];
    let mut snow_phase_count = vec![0.0; sample_count];
    let mut sea_ice_phase_count = vec![0.0; sample_count];
    let mut global_evaporation_year = 0.0;
    let mut global_precipitation_year = 0.0;
    let mut moisture_budget_error_year = 0.0;
    let mut final_temperature_rms_change = f64::INFINITY;
    let mut spinup_years = parameters.maximum_spinup_years;
    let mut final_divergence_residual = 0.0;

    for year in 0..parameters.maximum_spinup_years {
        let start_temperature = temperature.clone();
        let start_sst = sea_surface_temperature.clone();

        annual_mean_insolation.fill(0.0);
        insolation_min.fill(f64::INFINITY);
        insolation_max.fill(f64::NEG_INFINITY);
        temperature_sum.fill(0.0);
        temperature_cos.fill(0.0);
        temperature_sin.fill(0.0);
        temperature_min.fill(f64::INFINITY);
        temperature_max.fill(f64::NEG_INFINITY);
        wind_east_sum.fill(0.0);
        wind_north_sum.fill(0.0);
        wind_east_cos.fill(0.0);
        wind_east_sin.fill(0.0);
        wind_north_cos.fill(0.0);
        wind_north_sin.fill(0.0);
        sst_sum.fill(0.0);
        current_east_sum.fill(0.0);
        current_north_sum.fill(0.0);
        current_speed_sum.fill(0.0);
        ocean_heat_transport_sum.fill(0.0);
        humidity_sum.fill(0.0);
        precipitation_mass_year.fill(0.0);
        precipitation_phase_max.fill(0.0);
        potential_evaporation_mass_year.fill(0.0);
        cold_precipitation_mass_year.fill(0.0);
        snow_phase_count.fill(0.0);
        sea_ice_phase_count.fill(0.0);
        global_evaporation_year = 0.0;
        global_precipitation_year = 0.0;
        moisture_budget_error_year = 0.0;

        for phase in 0..phase_count {
            let orbital_angle = TWO_PI * phase as f64 / phase_count as f64;
            let eccentricity = physical.orbital_eccentricity;
            let distance_factor = ((1.0 + eccentricity
                * (orbital_angle - physical.longitude_of_periapsis_rad).cos())
                / (1.0 - eccentricity * eccentricity))
                .powi(2);
            let declination = (planet.axial_tilt_rad.sin() * orbital_angle.sin()).asin();
            let phase_angle = orbital_angle;
            let phase_cos = phase_angle.cos();
            let phase_sin = phase_angle.sin();

            let mut insolation = vec![0.0; sample_count];
            let mut radiative_target = vec![0.0; sample_count];
            for i in 0..sample_count {
                let solar = daily_mean_insolation(
                    latitude[i],
                    declination,
                    planet.stellar_flux_w_m2 * distance_factor,
                );
                insolation[i] = solar;
                annual_mean_insolation[i] += solar / phase_count as f64;
                insolation_min[i] = insolation_min[i].min(solar);
                insolation_max[i] = insolation_max[i].max(solar);
                let base_albedo = if ocean[i] {
                    parameters.ocean_albedo
                } else {
                    parameters.land_albedo
                };
                let cold_state = if ocean[i] {
                    sea_surface_temperature[i] < parameters.sea_ice_temperature_k
                } else {
                    temperature[i] < parameters.snow_temperature_k
                };
                let albedo = if cold_state {
                    base_albedo
                        + parameters.snow_albedo_feedback
                            * (parameters.snow_ice_albedo - base_albedo)
                } else {
                    base_albedo
                }
                .clamp(0.0, 0.95);
                let absorbed = (solar * (1.0 - albedo) + planet.internal_heat_flux_w_per_m2)
                    .max(0.0);
                let effective_temperature = if absorbed > 0.0 {
                    (absorbed / STEFAN_BOLTZMANN).powf(0.25)
                } else {
                    120.0
                };
                let pressure_ratio = if planet.reference_surface_pressure_pa > 0.0 {
                    (pressure[i] / planet.reference_surface_pressure_pa).clamp(0.0, 2.0)
                } else {
                    0.0
                };
                let greenhouse = (1.0
                    + 0.75 * physical.atmospheric_longwave_optical_depth * pressure_ratio)
                    .powf(0.25);
                radiative_target[i] = (effective_temperature * greenhouse
                    - parameters.lapse_rate_k_per_m * terrain_height_m[i])
                    .clamp(120.0, 355.0);
            }

            let previous_temperature = temperature.clone();
            for i in 0..sample_count {
                let neighbor_temperature = mean_neighbor(topology, &previous_temperature, i);
                let transported_target = radiative_target[i]
                    + parameters.atmospheric_heat_relaxation
                        * (neighbor_temperature - previous_temperature[i]);
                let relaxation = if ocean[i] {
                    parameters.ocean_thermal_relaxation
                } else {
                    parameters.land_thermal_relaxation
                };
                temperature[i] = (previous_temperature[i]
                    + relaxation * (transported_target - previous_temperature[i]))
                    .clamp(120.0, 355.0);
                if atmosphere_exists {
                    let scale_height = (specific_gas_constant * temperature[i]
                        / planet.surface_gravity_m_s2)
                        .max(1.0);
                    pressure[i] = planet.reference_surface_pressure_pa
                        * (-terrain_height_m[i] / scale_height).exp();
                } else {
                    pressure[i] = 0.0;
                }
            }

            if atmosphere_exists {
                let temperature_for_gradient = temperature.clone();
                for i in 0..sample_count {
                    let (gradient_east, gradient_north) = scalar_gradient(
                        topology,
                        &temperature_for_gradient,
                        planet.radius_m,
                        i,
                        east_bases[i],
                        north_bases[i],
                    );
                    let latitude_abs_deg = latitude[i].abs().to_degrees();
                    let rotational_blend = ((latitude_abs_deg - 4.0) / 18.0).clamp(0.0, 1.0);
                    let rotation_sign = if omega >= 0.0 { 1.0 } else { -1.0 };
                    let geostrophic_east = -rotation_sign
                        * gradient_north
                        * 1_000_000.0
                        * parameters.wind_thermal_gradient_scale;
                    let geostrophic_north = rotation_sign
                        * gradient_east
                        * 1_000_000.0
                        * parameters.wind_thermal_gradient_scale;
                    let zonal = baseline_zonal_wind(latitude[i]) * rotation_sign;
                    let meridional = if latitude_abs_deg < 30.0 {
                        -latitude[i].signum() * 2.6 * (1.0 - latitude_abs_deg / 30.0)
                    } else {
                        0.0
                    };
                    let slope = norm2(terrain_gradient_east[i], terrain_gradient_north[i]);
                    let drag = 1.0 / (1.0 + parameters.topographic_wind_drag * slope);
                    let east = (zonal + rotational_blend * geostrophic_east) * drag;
                    let north = (meridional + rotational_blend * geostrophic_north) * drag;
                    (wind_east[i], wind_north[i]) =
                        clamp_vector(east, north, parameters.maximum_wind_speed_m_s);
                }
            } else {
                wind_east.fill(0.0);
                wind_north.fill(0.0);
            }

            current_east.fill(0.0);
            current_north.fill(0.0);
            if planet.surface_water_mass_kg > 0.0 {
                for i in 0..sample_count {
                    if !ocean[i] {
                        continue;
                    }
                    let coriolis = 2.0 * omega * latitude[i].sin();
                    let sign = if coriolis >= 0.0 { 1.0 } else { -1.0 };
                    let mobility = (water_depth_m[i] / parameters.ocean_bathymetric_drag_depth_m)
                        .clamp(0.08, 1.0)
                        .sqrt();
                    let east = parameters.ocean_wind_coupling
                        * mobility
                        * (wind_east[i]
                            + sign * parameters.ocean_coriolis_deflection * wind_north[i]);
                    let north = parameters.ocean_wind_coupling
                        * mobility
                        * (wind_north[i]
                            - sign * parameters.ocean_coriolis_deflection * wind_east[i]);
                    (current_east[i], current_north[i]) =
                        clamp_vector(east, north, parameters.maximum_surface_current_m_s);
                }
                final_divergence_residual = correct_ocean_currents(
                    topology,
                    &ocean,
                    &east_bases,
                    &north_bases,
                    &mut current_east,
                    &mut current_north,
                    &parameters,
                );
            }

            let previous_sst = sea_surface_temperature.clone();
            let mut next_sst = previous_sst.clone();
            for i in 0..sample_count {
                if !ocean[i] {
                    next_sst[i] = temperature[i];
                    continue;
                }
                let (sst_gradient_east, sst_gradient_north) = scalar_gradient(
                    topology,
                    &previous_sst,
                    planet.radius_m,
                    i,
                    east_bases[i],
                    north_bases[i],
                );
                let advection_k_s = -(current_east[i] * sst_gradient_east
                    + current_north[i] * sst_gradient_north);
                let advection_delta = (advection_k_s
                    * phase_seconds
                    * parameters.ocean_advection_relaxation)
                    .clamp(-4.0, 4.0);
                let neighbor_sst = mean_neighbor(topology, &previous_sst, i);
                next_sst[i] = (previous_sst[i]
                    + advection_delta
                    + parameters.ocean_temperature_diffusion * (neighbor_sst - previous_sst[i])
                    + parameters.air_sea_exchange_relaxation
                        * (temperature[i] - previous_sst[i]))
                    .clamp(260.0, 325.0);
            }
            sea_surface_temperature = next_sst;
            for i in 0..sample_count {
                if ocean[i] {
                    temperature[i] += parameters.air_sea_exchange_relaxation
                        * (sea_surface_temperature[i] - temperature[i]);
                }
            }

            if atmosphere_exists {
                let mut air_mass = vec![0.0; sample_count];
                let mut moisture_mass = vec![0.0; sample_count];
                for i in 0..sample_count {
                    air_mass[i] = pressure[i] / planet.surface_gravity_m_s2 * cell_area_m2[i];
                    moisture_mass[i] = humidity[i] * air_mass[i];
                }
                let moisture_before = moisture_mass.iter().sum::<f64>();
                let mut transport_delta = vec![0.0; sample_count];
                for i in 0..sample_count {
                    let origin = topology.positions()[i];
                    for (neighbor_index, arc) in topology
                        .neighbors_of(i as u32)
                        .iter()
                        .zip(topology.neighbor_arc_lengths_of(i as u32).iter())
                    {
                        let j = *neighbor_index as usize;
                        if j <= i {
                            continue;
                        }
                        let position = topology.positions()[j];
                        let radial = dot(position, origin);
                        let tangent = [
                            position[0] - origin[0] * radial,
                            position[1] - origin[1] * radial,
                            position[2] - origin[2] * radial,
                        ];
                        let tangent_norm = dot(tangent, tangent).sqrt();
                        if tangent_norm <= 1.0e-15 {
                            continue;
                        }
                        let direction = [
                            tangent[0] / tangent_norm,
                            tangent[1] / tangent_norm,
                            tangent[2] / tangent_norm,
                        ];
                        let projected = wind_east[i] * dot(direction, east_bases[i])
                            + wind_north[i] * dot(direction, north_bases[i]);
                        let distance = (*arc * planet.radius_m).max(1.0);
                        let fraction = (projected.abs() * phase_seconds / distance
                            * parameters.moisture_transport_cfl)
                            .clamp(0.0, 0.22);
                        if projected >= 0.0 {
                            let transfer = moisture_mass[i] * fraction;
                            transport_delta[i] -= transfer;
                            transport_delta[j] += transfer;
                        } else {
                            let transfer = moisture_mass[j] * fraction;
                            transport_delta[j] -= transfer;
                            transport_delta[i] += transfer;
                        }
                    }
                }
                for i in 0..sample_count {
                    moisture_mass[i] = (moisture_mass[i] + transport_delta[i]).max(0.0);
                }

                let mut phase_evaporation = 0.0;
                let mut phase_precipitation = 0.0;
                for i in 0..sample_count {
                    if air_mass[i] <= 0.0 {
                        humidity[i] = 0.0;
                        continue;
                    }
                    let q = moisture_mass[i] / air_mass[i];
                    let wind_speed = norm2(wind_east[i], wind_north[i]);
                    let saturation_surface = saturation_specific_humidity(
                        if ocean[i] {
                            sea_surface_temperature[i]
                        } else {
                            temperature[i]
                        },
                        pressure[i],
                    );
                    let potential_fraction = ((saturation_surface - q).max(0.0)
                        * parameters.evaporation_relaxation
                        * (0.65 + (wind_speed / 12.0).clamp(0.0, 1.4)))
                        .max(0.0);
                    let potential_mass = potential_fraction * air_mass[i];
                    potential_evaporation_mass_year[i] += potential_mass;
                    if ocean[i] {
                        moisture_mass[i] += potential_mass;
                        phase_evaporation += potential_mass;
                    }

                    let current_q = moisture_mass[i] / air_mass[i];
                    let saturation_air = saturation_specific_humidity(temperature[i], pressure[i]);
                    let threshold = saturation_air * parameters.condensation_relative_humidity;
                    let excess_q = (current_q - threshold).max(0.0);
                    let condensation_mass = excess_q
                        * air_mass[i]
                        * parameters.condensation_efficiency;
                    let along_slope = if wind_speed > 0.25 {
                        (wind_east[i] * terrain_gradient_east[i]
                            + wind_north[i] * terrain_gradient_north[i])
                            / wind_speed
                    } else {
                        0.0
                    };
                    let orographic_fraction = (along_slope.max(0.0)
                        * parameters.orographic_precipitation_strength)
                        .clamp(0.0, parameters.maximum_orographic_fraction);
                    let after_condensation = (moisture_mass[i] - condensation_mass).max(0.0);
                    let orographic_mass = after_condensation * orographic_fraction;
                    let precipitation_mass = condensation_mass + orographic_mass;
                    moisture_mass[i] = (after_condensation - orographic_mass).max(0.0);
                    precipitation_mass_year[i] += precipitation_mass;
                    precipitation_phase_max[i] =
                        precipitation_phase_max[i].max(precipitation_mass);
                    if temperature[i] <= parameters.snow_temperature_k {
                        cold_precipitation_mass_year[i] += precipitation_mass;
                        snow_phase_count[i] += 1.0;
                    }
                    if ocean[i]
                        && sea_surface_temperature[i] <= parameters.sea_ice_temperature_k
                    {
                        sea_ice_phase_count[i] += 1.0;
                    }
                    phase_precipitation += precipitation_mass;
                    humidity[i] = (moisture_mass[i] / air_mass[i]).clamp(0.0, 0.2);
                }
                let moisture_after = moisture_mass.iter().sum::<f64>();
                let expected_change = phase_evaporation - phase_precipitation;
                moisture_budget_error_year +=
                    ((moisture_after - moisture_before) - expected_change).abs();
                global_evaporation_year += phase_evaporation;
                global_precipitation_year += phase_precipitation;
            } else {
                humidity.fill(0.0);
            }

            for i in 0..sample_count {
                temperature_sum[i] += temperature[i];
                temperature_cos[i] += temperature[i] * phase_cos;
                temperature_sin[i] += temperature[i] * phase_sin;
                temperature_min[i] = temperature_min[i].min(temperature[i]);
                temperature_max[i] = temperature_max[i].max(temperature[i]);
                wind_east_sum[i] += wind_east[i];
                wind_north_sum[i] += wind_north[i];
                wind_east_cos[i] += wind_east[i] * phase_cos;
                wind_east_sin[i] += wind_east[i] * phase_sin;
                wind_north_cos[i] += wind_north[i] * phase_cos;
                wind_north_sin[i] += wind_north[i] * phase_sin;
                sst_sum[i] += sea_surface_temperature[i];
                current_east_sum[i] += current_east[i];
                current_north_sum[i] += current_north[i];
                current_speed_sum[i] += norm2(current_east[i], current_north[i]);
                let (sst_gradient_east, sst_gradient_north) = scalar_gradient(
                    topology,
                    &sea_surface_temperature,
                    planet.radius_m,
                    i,
                    east_bases[i],
                    north_bases[i],
                );
                ocean_heat_transport_sum[i] += if ocean[i] {
                    -(current_east[i] * sst_gradient_east
                        + current_north[i] * sst_gradient_north)
                        * 1_000_000.0
                } else {
                    0.0
                };
                humidity_sum[i] += humidity[i];
            }
        }

        let mut squared_change = 0.0;
        for i in 0..sample_count {
            let delta_temperature = temperature[i] - start_temperature[i];
            let delta_sst = sea_surface_temperature[i] - start_sst[i];
            squared_change += delta_temperature * delta_temperature + 0.5 * delta_sst * delta_sst;
        }
        final_temperature_rms_change = (squared_change / (sample_count as f64 * 1.5)).sqrt();
        spinup_years = year + 1;
        if spinup_years >= parameters.minimum_spinup_years
            && final_temperature_rms_change <= parameters.convergence_temperature_rms_k
        {
            break;
        }
    }

    let phase_count_f64 = phase_count as f64;
    let harmonic_scale = 2.0 / phase_count_f64;
    let mut annual_mean_insolation_f32 = vec![0.0; sample_count];
    let mut seasonal_insolation_amplitude = vec![0.0; sample_count];
    let mut temperature_mean = vec![0.0; sample_count];
    let mut temperature_annual_cos = vec![0.0; sample_count];
    let mut temperature_annual_sin = vec![0.0; sample_count];
    let mut temperature_min_f32 = vec![0.0; sample_count];
    let mut temperature_max_f32 = vec![0.0; sample_count];
    let mut pressure_f32 = vec![0.0; sample_count];
    let mut wind_east_mean = vec![0.0; sample_count];
    let mut wind_north_mean = vec![0.0; sample_count];
    let mut wind_east_cos_out = vec![0.0; sample_count];
    let mut wind_east_sin_out = vec![0.0; sample_count];
    let mut wind_north_cos_out = vec![0.0; sample_count];
    let mut wind_north_sin_out = vec![0.0; sample_count];
    let mut sst_mean = vec![0.0; sample_count];
    let mut current_east_mean = vec![0.0; sample_count];
    let mut current_north_mean = vec![0.0; sample_count];
    let mut current_speed_mean = vec![0.0; sample_count];
    let mut ocean_heat_transport = vec![0.0; sample_count];
    let mut humidity_mean = vec![0.0; sample_count];
    let mut annual_precipitation_mm = vec![0.0; sample_count];
    let mut precipitation_seasonality = vec![0.0; sample_count];
    let mut potential_evaporation_mm = vec![0.0; sample_count];
    let mut moisture_balance_mm = vec![0.0; sample_count];
    let mut aridity_index = vec![0.0; sample_count];
    let mut snowfall_fraction = vec![0.0; sample_count];
    let mut persistent_snow_potential = vec![0.0; sample_count];
    let mut sea_ice_potential = vec![0.0; sample_count];

    for i in 0..sample_count {
        annual_mean_insolation_f32[i] = annual_mean_insolation[i] as f32;
        seasonal_insolation_amplitude[i] =
            (0.5 * (insolation_max[i] - insolation_min[i])).max(0.0) as f32;
        temperature_mean[i] = (temperature_sum[i] / phase_count_f64) as f32;
        temperature_annual_cos[i] = (temperature_cos[i] * harmonic_scale) as f32;
        temperature_annual_sin[i] = (temperature_sin[i] * harmonic_scale) as f32;
        temperature_min_f32[i] = temperature_min[i] as f32;
        temperature_max_f32[i] = temperature_max[i] as f32;
        pressure_f32[i] = pressure[i] as f32;
        wind_east_mean[i] = (wind_east_sum[i] / phase_count_f64) as f32;
        wind_north_mean[i] = (wind_north_sum[i] / phase_count_f64) as f32;
        wind_east_cos_out[i] = (wind_east_cos[i] * harmonic_scale) as f32;
        wind_east_sin_out[i] = (wind_east_sin[i] * harmonic_scale) as f32;
        wind_north_cos_out[i] = (wind_north_cos[i] * harmonic_scale) as f32;
        wind_north_sin_out[i] = (wind_north_sin[i] * harmonic_scale) as f32;
        sst_mean[i] = (sst_sum[i] / phase_count_f64) as f32;
        current_east_mean[i] = (current_east_sum[i] / phase_count_f64) as f32;
        current_north_mean[i] = (current_north_sum[i] / phase_count_f64) as f32;
        current_speed_mean[i] = (current_speed_sum[i] / phase_count_f64) as f32;
        ocean_heat_transport[i] = (ocean_heat_transport_sum[i] / phase_count_f64) as f32;
        humidity_mean[i] = (humidity_sum[i] / phase_count_f64) as f32;
        let area = cell_area_m2[i].max(1.0);
        annual_precipitation_mm[i] = (precipitation_mass_year[i] / area) as f32;
        potential_evaporation_mm[i] = (potential_evaporation_mass_year[i] / area) as f32;
        moisture_balance_mm[i] =
            annual_precipitation_mm[i] - potential_evaporation_mm[i];
        aridity_index[i] = if potential_evaporation_mm[i] > 1.0 {
            (annual_precipitation_mm[i] / potential_evaporation_mm[i]).clamp(0.0, 4.0)
        } else {
            4.0
        };
        let mean_phase_precipitation = precipitation_mass_year[i] / phase_count_f64;
        precipitation_seasonality[i] = if mean_phase_precipitation > 0.0 {
            ((precipitation_phase_max[i] / mean_phase_precipitation) - 1.0)
                .clamp(0.0, 8.0) as f32
        } else {
            0.0
        };
        snowfall_fraction[i] = if precipitation_mass_year[i] > 0.0 {
            (cold_precipitation_mass_year[i] / precipitation_mass_year[i]).clamp(0.0, 1.0)
                as f32
        } else {
            0.0
        };
        persistent_snow_potential[i] =
            ((snow_phase_count[i] / phase_count_f64) * f64::from(snowfall_fraction[i]))
                .clamp(0.0, 1.0) as f32;
        sea_ice_potential[i] = if ocean[i] {
            (sea_ice_phase_count[i] / phase_count_f64).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
    }

    let mean_temperature = area_weighted_mean(topology, &temperature_mean);
    let mean_land_temperature =
        subset_area_weighted_mean(topology, &temperature_mean, |i| !ocean[i]);
    let mean_ocean_temperature =
        subset_area_weighted_mean(topology, &temperature_mean, |i| ocean[i]);
    let mean_sst = subset_area_weighted_mean(topology, &sst_mean, |i| ocean[i]);
    let mut wind_speed_values = vec![0.0_f32; sample_count];
    for i in 0..sample_count {
        wind_speed_values[i] = norm2(
            f64::from(wind_east_mean[i]),
            f64::from(wind_north_mean[i]),
        ) as f32;
    }
    let mean_wind_speed = area_weighted_mean(topology, &wind_speed_values);
    let maximum_wind_speed = wind_speed_values
        .iter()
        .copied()
        .fold(0.0_f32, f32::max) as f64;
    let mean_current_speed = subset_area_weighted_mean(topology, &current_speed_mean, |i| ocean[i]);
    let maximum_current_speed = current_speed_mean
        .iter()
        .copied()
        .fold(0.0_f32, f32::max) as f64;
    let mean_precipitation = area_weighted_mean(topology, &annual_precipitation_mm);
    let p95_precipitation = percentile(&annual_precipitation_mm, 0.95);
    let total_budget_scale = global_evaporation_year
        .abs()
        .max(global_precipitation_year.abs())
        .max(1.0);
    let moisture_budget_relative_error = moisture_budget_error_year / total_budget_scale;
    let total_area = topology.metrics().total_area_steradians.max(1.0e-12);
    let persistent_snow_area_fraction = persistent_snow_potential
        .iter()
        .enumerate()
        .map(|(i, value)| topology.dual_area_steradians()[i] * f64::from(*value))
        .sum::<f64>()
        / total_area;
    let sea_ice_area_fraction = sea_ice_potential
        .iter()
        .enumerate()
        .map(|(i, value)| topology.dual_area_steradians()[i] * f64::from(*value))
        .sum::<f64>()
        / total_area;

    let physical_hash = physical.parameter_hash();
    let model_hash = parameters.parameter_hash();
    let mut climate_hash = FNV_OFFSET_BASIS;
    climate_hash = fnv_update(climate_hash, &stage_seed.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &terrain.metrics.topography_hash.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &planet.parameter_hash().to_le_bytes());
    climate_hash = fnv_update(climate_hash, &physical_hash.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &model_hash.to_le_bytes());
    climate_hash = hash_f32_slice(climate_hash, &temperature_mean);
    climate_hash = hash_f32_slice(climate_hash, &wind_east_mean);
    climate_hash = hash_f32_slice(climate_hash, &wind_north_mean);
    climate_hash = hash_f32_slice(climate_hash, &sst_mean);
    climate_hash = hash_f32_slice(climate_hash, &current_east_mean);
    climate_hash = hash_f32_slice(climate_hash, &current_north_mean);
    climate_hash = hash_f32_slice(climate_hash, &annual_precipitation_mm);
    climate_hash = hash_f32_slice(climate_hash, &aridity_index);

    let metrics = ClimateMetrics {
        sample_count: sample_count as u32,
        orbital_phase_count: parameters.orbital_phase_count,
        spinup_years,
        mean_temperature_k: mean_temperature,
        minimum_temperature_k: temperature_min_f32
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min) as f64,
        maximum_temperature_k: temperature_max_f32
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max) as f64,
        mean_land_temperature_k: mean_land_temperature,
        mean_ocean_temperature_k: mean_ocean_temperature,
        mean_wind_speed_m_s: mean_wind_speed,
        maximum_wind_speed_m_s: maximum_wind_speed,
        mean_surface_current_m_s: mean_current_speed,
        maximum_surface_current_m_s: maximum_current_speed,
        ocean_divergence_residual_m_s: final_divergence_residual,
        mean_sea_surface_temperature_k: mean_sst,
        mean_annual_precipitation_mm: mean_precipitation,
        p95_annual_precipitation_mm: p95_precipitation,
        global_evaporation_kg: global_evaporation_year,
        global_precipitation_kg: global_precipitation_year,
        moisture_budget_relative_error,
        persistent_snow_area_fraction,
        sea_ice_area_fraction,
        final_temperature_rms_change_k: final_temperature_rms_change,
        climate_physical_parameter_hash: physical_hash,
        climate_model_parameter_hash: model_hash,
        climate_hash,
    };

    Ok(ClimateState {
        stage: StageIdentity {
            id: CLIMATE_STAGE_ID,
            version: CLIMATE_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics,
        annual_mean_insolation_w_m2: annual_mean_insolation_f32,
        seasonal_insolation_amplitude_w_m2: seasonal_insolation_amplitude,
        temperature_mean_k: temperature_mean,
        temperature_annual_cos_k: temperature_annual_cos,
        temperature_annual_sin_k: temperature_annual_sin,
        temperature_min_k: temperature_min_f32,
        temperature_max_k: temperature_max_f32,
        local_pressure_pa: pressure_f32,
        wind_east_mean_m_s: wind_east_mean,
        wind_north_mean_m_s: wind_north_mean,
        wind_east_annual_cos_m_s: wind_east_cos_out,
        wind_east_annual_sin_m_s: wind_east_sin_out,
        wind_north_annual_cos_m_s: wind_north_cos_out,
        wind_north_annual_sin_m_s: wind_north_sin_out,
        sea_surface_temperature_mean_k: sst_mean,
        current_east_mean_m_s: current_east_mean,
        current_north_mean_m_s: current_north_mean,
        current_speed_mean_m_s: current_speed_mean,
        ocean_heat_transport_index: ocean_heat_transport,
        specific_humidity_mean: humidity_mean,
        annual_precipitation_mm,
        precipitation_seasonality,
        potential_evaporation_mm,
        moisture_balance_mm,
        aridity_index,
        snowfall_fraction,
        persistent_snow_potential,
        sea_ice_potential,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_climate_parameters_are_valid_and_stable() {
        let physical = ClimatePhysicalParameters::default();
        let model = ClimateParameters::default();
        physical.validate().unwrap();
        model.validate().unwrap();
        assert_eq!(physical.parameter_hash(), ClimatePhysicalParameters::earthlike_reference().parameter_hash());
        assert_eq!(model.parameter_hash(), ClimateParameters::default().parameter_hash());
        assert_eq!(physical.parameter_hash_hex().len(), 16);
        assert_eq!(model.parameter_hash_hex().len(), 16);
    }

    #[test]
    fn seasonal_solar_geometry_handles_polar_night_and_equatorial_daylight() {
        let equator = daily_mean_insolation(0.0, 0.0, 1361.0);
        let north_pole_winter = daily_mean_insolation(
            std::f64::consts::FRAC_PI_2,
            -23.4_f64.to_radians(),
            1361.0,
        );
        assert!(equator > 400.0);
        assert_eq!(north_pole_winter, 0.0);
    }
}
