use crate::{
    derive_stage_seed, tangent_basis, GeodesicTopology, PlanetPhysicalParameters, StageIdentity,
    TopographyState, WorldgenError,
};

const CLIMATE_NAMESPACE: &str = "climate:v1";
pub const CLIMATE_STAGE_ID: &str = "climate:coupled-surface";
pub const CLIMATE_STAGE_VERSION: u32 = 4;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;
const LATENT_HEAT_VAPORIZATION_J_PER_KG: f64 = 2_450_000.0;
const UNIVERSAL_GAS_CONSTANT: f64 = 8.314_462_618;
const TWO_PI: f64 = std::f64::consts::PI * 2.0;
const EARTH_REFERENCE_ROTATION_PERIOD_S: f64 = 86_164.0905;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClimatePhysicalParameters {
    pub orbital_eccentricity: f64,
    pub longitude_of_periapsis_rad: f64,
    pub atmospheric_mean_molar_mass_kg_per_mol: f64,
    pub atmospheric_specific_heat_j_per_kg_k: f64,
    pub atmospheric_shortwave_reflectivity: f64,
    pub atmospheric_longwave_optical_depth: f64,
}

impl ClimatePhysicalParameters {
    pub const fn earthlike_reference() -> Self {
        Self {
            orbital_eccentricity: 0.0167,
            longitude_of_periapsis_rad: 1.796_767_421_176_181_3,
            atmospheric_mean_molar_mass_kg_per_mol: 0.028_964_7,
            atmospheric_specific_heat_j_per_kg_k: 1_004.0,
            atmospheric_shortwave_reflectivity: 0.25,
            atmospheric_longwave_optical_depth: 1.20,
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
        if !self.atmospheric_shortwave_reflectivity.is_finite()
            || !(0.0..1.0).contains(&self.atmospheric_shortwave_reflectivity)
        {
            return Err("atmospheric shortwave reflectivity must be finite and within [0, 1)");
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
            self.atmospheric_shortwave_reflectivity,
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
    pub surface_albedo_shortwave_coupling: f64,
    pub snow_ice_albedo: f64,
    pub snow_albedo_feedback: f64,
    pub lapse_rate_k_per_m: f64,
    pub land_thermal_relaxation: f64,
    pub ocean_thermal_relaxation: f64,
    pub atmospheric_heat_diffusivity_m2_s: f64,
    pub atmospheric_heat_solver_iterations: u8,
    pub air_sea_exchange_coefficient_w_m2_k: f64,
    pub ocean_mixed_layer_depth_m: f64,
    pub wind_thermal_gradient_scale: f64,
    pub topographic_wind_drag: f64,
    pub maximum_wind_speed_m_s: f64,
    pub ocean_wind_coupling: f64,
    pub ocean_coriolis_deflection: f64,
    pub ocean_bathymetric_drag_depth_m: f64,
    pub ocean_current_correction_iterations: u8,
    pub maximum_surface_current_m_s: f64,
    pub ocean_temperature_diffusion: f64,
    pub ocean_advection_relaxation: f64,
    pub ocean_advection_cfl_limit: f64,
    pub evaporation_bulk_transfer_coefficient: f64,
    pub evaporation_energy_fraction: f64,
    pub moisture_transport_minimum_substeps: u8,
    pub moisture_transport_maximum_substeps: u8,
    pub moisture_transport_cfl_limit: f64,
    pub maximum_climatological_moisture_transport_speed_m_s: f64,
    pub convergence_precipitation_relative_humidity: f64,
    pub convergence_precipitation_efficiency: f64,
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
            surface_albedo_shortwave_coupling: 0.25,
            snow_ice_albedo: 0.62,
            snow_albedo_feedback: 0.32,
            lapse_rate_k_per_m: 0.0065,
            land_thermal_relaxation: 0.38,
            ocean_thermal_relaxation: 0.12,
            atmospheric_heat_diffusivity_m2_s: 2000000.0,
            atmospheric_heat_solver_iterations: 20,
            air_sea_exchange_coefficient_w_m2_k: 8.0,
            ocean_mixed_layer_depth_m: 14.0,
            wind_thermal_gradient_scale: 0.72,
            topographic_wind_drag: 42.0,
            maximum_wind_speed_m_s: 65.0,
            ocean_wind_coupling: 0.035,
            ocean_coriolis_deflection: 0.55,
            ocean_bathymetric_drag_depth_m: 700.0,
            ocean_current_correction_iterations: 6,
            maximum_surface_current_m_s: 2.8,
            ocean_temperature_diffusion: 0.08,
            ocean_advection_relaxation: 0.010,
            ocean_advection_cfl_limit: 0.45,
            evaporation_bulk_transfer_coefficient: 0.0015,
            evaporation_energy_fraction: 0.45,
            moisture_transport_minimum_substeps: 4,
            moisture_transport_maximum_substeps: 64,
            moisture_transport_cfl_limit: 0.90,
            maximum_climatological_moisture_transport_speed_m_s: 1.0,
            convergence_precipitation_relative_humidity: 0.60,
            convergence_precipitation_efficiency: 0.35,
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
            self.atmospheric_heat_diffusivity_m2_s,
            self.air_sea_exchange_coefficient_w_m2_k,
            self.ocean_mixed_layer_depth_m,
            self.ocean_wind_coupling,
            self.ocean_bathymetric_drag_depth_m,
            self.maximum_surface_current_m_s,
            self.ocean_advection_cfl_limit,
            self.evaporation_bulk_transfer_coefficient,
            self.evaporation_energy_fraction,
            self.moisture_transport_cfl_limit,
            self.maximum_climatological_moisture_transport_speed_m_s,
            self.orographic_precipitation_strength,
            self.snow_temperature_k,
            self.sea_ice_temperature_k,
        ];
        if positive
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err("climate positive model parameters must be finite and positive");
        }
        let unit_interval = [
            self.land_albedo,
            self.ocean_albedo,
            self.surface_albedo_shortwave_coupling,
            self.snow_ice_albedo,
            self.snow_albedo_feedback,
            self.land_thermal_relaxation,
            self.ocean_thermal_relaxation,
            self.ocean_coriolis_deflection,
            self.ocean_temperature_diffusion,
            self.ocean_advection_relaxation,
            self.ocean_advection_cfl_limit,
            self.moisture_transport_cfl_limit,
            self.convergence_precipitation_relative_humidity,
            self.convergence_precipitation_efficiency,
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
        if self.atmospheric_heat_solver_iterations == 0
            || self.atmospheric_heat_solver_iterations > 32
        {
            return Err("atmospheric heat solver iterations must be from 1 through 32");
        }
        if self.moisture_transport_minimum_substeps == 0
            || self.moisture_transport_maximum_substeps < self.moisture_transport_minimum_substeps
            || self.moisture_transport_maximum_substeps > 64
        {
            return Err("moisture transport substep bounds must be within 1 through 64");
        }
        if !self.evaporation_energy_fraction.is_finite()
            || self.evaporation_energy_fraction <= 0.0
            || self.evaporation_energy_fraction > 1.0
        {
            return Err("evaporation energy fraction must be finite and within (0, 1]");
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
        hash = fnv_update(hash, &[self.atmospheric_heat_solver_iterations]);
        hash = fnv_update(hash, &[self.moisture_transport_minimum_substeps]);
        hash = fnv_update(hash, &[self.moisture_transport_maximum_substeps]);
        for value in [
            self.convergence_temperature_rms_k,
            self.land_albedo,
            self.ocean_albedo,
            self.surface_albedo_shortwave_coupling,
            self.snow_ice_albedo,
            self.snow_albedo_feedback,
            self.lapse_rate_k_per_m,
            self.land_thermal_relaxation,
            self.ocean_thermal_relaxation,
            self.atmospheric_heat_diffusivity_m2_s,
            self.air_sea_exchange_coefficient_w_m2_k,
            self.ocean_mixed_layer_depth_m,
            self.wind_thermal_gradient_scale,
            self.topographic_wind_drag,
            self.maximum_wind_speed_m_s,
            self.ocean_wind_coupling,
            self.ocean_coriolis_deflection,
            self.ocean_bathymetric_drag_depth_m,
            self.maximum_surface_current_m_s,
            self.ocean_temperature_diffusion,
            self.ocean_advection_relaxation,
            self.ocean_advection_cfl_limit,
            self.evaporation_bulk_transfer_coefficient,
            self.moisture_transport_cfl_limit,
            self.maximum_climatological_moisture_transport_speed_m_s,
            self.convergence_precipitation_relative_humidity,
            self.convergence_precipitation_efficiency,
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
pub struct ClimateGenerationDiagnostics {
    /// Final spin-up-year precipitation rate for each retained orbital phase.
    /// Layout is phase-major: phase * sample_count + sample. Values are
    /// annualized mm/year-equivalent rates for direct phase-to-phase comparison.
    pub precipitation_phase_rate_mm_year: Vec<f32>,
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
    pub moisture_transport_limiter_fraction: f64,
    pub maximum_moisture_transport_substeps: u8,
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
    pub sea_surface_temperature_annual_cos_k: Vec<f32>,
    pub sea_surface_temperature_annual_sin_k: Vec<f32>,
    pub current_east_mean_m_s: Vec<f32>,
    pub current_north_mean_m_s: Vec<f32>,
    pub current_east_annual_cos_m_s: Vec<f32>,
    pub current_east_annual_sin_m_s: Vec<f32>,
    pub current_north_annual_cos_m_s: Vec<f32>,
    pub current_north_annual_sin_m_s: Vec<f32>,
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
    if area > 0.0 {
        weighted / area
    } else {
        0.0
    }
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
    if area > 0.0 {
        weighted / area
    } else {
        0.0
    }
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

fn mean_ocean_neighbor(
    topology: &GeodesicTopology,
    ocean: &[bool],
    values: &[f64],
    sample: usize,
) -> f64 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for neighbor in topology.neighbors_of(sample as u32) {
        let index = *neighbor as usize;
        if ocean[index] {
            sum += values[index];
            count += 1;
        }
    }
    if count > 0 {
        sum / count as f64
    } else {
        values[sample]
    }
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
    let value = stellar_flux / std::f64::consts::PI
        * (hour_angle * latitude.sin() * declination.sin()
            + latitude.cos() * declination.cos() * hour_angle.sin());
    value.max(0.0)
}

pub(crate) fn effective_shortwave_albedo(
    atmospheric_reflectivity: f64,
    surface_coupling: f64,
    surface_albedo: f64,
) -> f64 {
    (atmospheric_reflectivity
        + surface_coupling * (1.0 - atmospheric_reflectivity) * surface_albedo)
        .clamp(0.0, 0.95)
}

fn atmospheric_surface_height_m(submerged: bool, elevation_above_sea_level_m: f64) -> f64 {
    if submerged {
        0.0
    } else {
        elevation_above_sea_level_m.max(0.0)
    }
}

fn solve_orbital_forcing(
    mean_longitude_rad: f64,
    eccentricity: f64,
    longitude_of_periapsis_rad: f64,
) -> (f64, f64) {
    let mean_anomaly = (mean_longitude_rad - longitude_of_periapsis_rad).rem_euclid(TWO_PI);
    let mut eccentric_anomaly = if eccentricity < 0.8 {
        mean_anomaly
    } else {
        std::f64::consts::PI
    };
    for _ in 0..16 {
        let residual = eccentric_anomaly - eccentricity * eccentric_anomaly.sin() - mean_anomaly;
        let derivative = 1.0 - eccentricity * eccentric_anomaly.cos();
        let step = residual / derivative.max(1.0e-12);
        eccentric_anomaly -= step;
        if step.abs() <= 1.0e-13 {
            break;
        }
    }
    let half_e = 0.5 * eccentric_anomaly;
    let true_anomaly = 2.0
        * ((1.0 + eccentricity).sqrt() * half_e.sin())
            .atan2((1.0 - eccentricity).sqrt() * half_e.cos());
    let solar_longitude = true_anomaly + longitude_of_periapsis_rad;
    let radius_over_a = (1.0 - eccentricity * eccentric_anomaly.cos()).max(1.0e-6);
    (solar_longitude, 1.0 / (radius_over_a * radius_over_a))
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

fn rotation_response(rotation_period_s: f64) -> f64 {
    (EARTH_REFERENCE_ROTATION_PERIOD_S / rotation_period_s).clamp(0.05, 8.0)
}

fn circulation_cell_edges(rotation_ratio: f64) -> (f64, f64) {
    let hadley_edge_deg = (30.0 / rotation_ratio.sqrt()).clamp(15.0, 60.0);
    let polar_edge_deg = hadley_edge_deg + 0.5 * (90.0 - hadley_edge_deg);
    (hadley_edge_deg, polar_edge_deg)
}

fn baseline_zonal_wind(latitude_rad: f64, rotation_ratio: f64) -> f64 {
    let degrees = latitude_rad.abs().to_degrees();
    let (hadley_edge_deg, polar_edge_deg) = circulation_cell_edges(rotation_ratio);
    let zonal_strength = rotation_ratio.sqrt().clamp(0.35, 2.5);
    if degrees < hadley_edge_deg {
        -8.0 * zonal_strength * (1.0 - 0.35 * degrees / hadley_edge_deg)
    } else if degrees < polar_edge_deg {
        let phase = (degrees - hadley_edge_deg) / (polar_edge_deg - hadley_edge_deg);
        10.0 * zonal_strength * (phase * std::f64::consts::PI).sin()
    } else {
        let phase = ((degrees - polar_edge_deg) / (90.0 - polar_edge_deg)).clamp(0.0, 1.0);
        -4.0 * zonal_strength * (phase * std::f64::consts::FRAC_PI_2).sin().abs()
    }
}

fn coriolis_deflection_factor(latitude_rad: f64, omega: f64, configured_strength: f64) -> f64 {
    let coriolis = 2.0 * omega * latitude_rad.sin();
    if coriolis.abs() <= 1.0e-15 {
        return 0.0;
    }
    let earth_omega = TWO_PI / EARTH_REFERENCE_ROTATION_PERIOD_S;
    let normalized = (coriolis.abs() / (2.0 * earth_omega)).clamp(0.0, 2.5);
    coriolis.signum() * configured_strength * normalized
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

#[derive(Clone, Copy, Debug)]
struct AtmosphericHeatEdge {
    a: usize,
    b: usize,
    geometric_conductance: f64,
}

#[derive(Clone, Debug)]
struct AtmosphericHeatGeometry {
    edges: Vec<AtmosphericHeatEdge>,
    diagonal_geometry: Vec<f64>,
}

fn build_atmospheric_heat_geometry(
    topology: &GeodesicTopology,
    radius_m: f64,
) -> AtmosphericHeatGeometry {
    let count = topology.metrics().sample_count as usize;
    let mut edges = Vec::new();
    let mut diagonal_geometry = vec![0.0; count];
    for a in 0..count {
        for ((neighbor, arc), interface_arc) in topology
            .neighbors_of(a as u32)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(a as u32).iter())
            .zip(topology.neighbor_interface_arc_lengths_of(a as u32).iter())
        {
            let b = *neighbor as usize;
            if b <= a {
                continue;
            }
            let distance_m = (*arc * radius_m).max(1.0);
            let interface_length_m = (*interface_arc * radius_m).max(1.0);
            let geometric_conductance = (interface_length_m / distance_m).max(1.0e-12);
            edges.push(AtmosphericHeatEdge {
                a,
                b,
                geometric_conductance,
            });
            diagonal_geometry[a] += geometric_conductance;
            diagonal_geometry[b] += geometric_conductance;
        }
    }
    AtmosphericHeatGeometry {
        edges,
        diagonal_geometry,
    }
}

fn apply_atmospheric_heat_matrix(
    geometry: &AtmosphericHeatGeometry,
    thermal_capacity_j_k: &[f64],
    diffusion_scale_j_k: f64,
    values: &[f64],
    output: &mut [f64],
) {
    for i in 0..values.len() {
        output[i] = thermal_capacity_j_k[i] * values[i];
    }
    for edge in &geometry.edges {
        let contribution =
            diffusion_scale_j_k * edge.geometric_conductance * (values[edge.a] - values[edge.b]);
        output[edge.a] += contribution;
        output[edge.b] -= contribution;
    }
}

fn diffuse_atmospheric_heat(
    geometry: &AtmosphericHeatGeometry,
    temperature: &mut [f64],
    pressure_pa: &[f64],
    cell_area_m2: &[f64],
    planet: PlanetPhysicalParameters,
    physical: ClimatePhysicalParameters,
    parameters: ClimateParameters,
    phase_seconds: f64,
) {
    if planet.reference_surface_pressure_pa <= 0.0
        || parameters.atmospheric_heat_diffusivity_m2_s <= 0.0
        || geometry.edges.is_empty()
    {
        return;
    }

    let reference_column_capacity_j_m2_k = planet.reference_surface_pressure_pa
        / planet.surface_gravity_m_s2
        * physical.atmospheric_specific_heat_j_per_kg_k;
    let diffusion_scale_j_k = phase_seconds
        * parameters.atmospheric_heat_diffusivity_m2_s
        * reference_column_capacity_j_m2_k;
    let mut capacity = vec![0.0; temperature.len()];
    let mut rhs = vec![0.0; temperature.len()];
    let mut diagonal = vec![0.0; temperature.len()];
    for i in 0..temperature.len() {
        let column_capacity = (pressure_pa[i] / planet.surface_gravity_m_s2
            * physical.atmospheric_specific_heat_j_per_kg_k)
            .max(reference_column_capacity_j_m2_k * 0.02);
        capacity[i] = column_capacity * cell_area_m2[i];
        rhs[i] = capacity[i] * temperature[i];
        diagonal[i] = capacity[i] + diffusion_scale_j_k * geometry.diagonal_geometry[i];
    }

    let mut x = temperature.to_vec();
    let mut matrix_x = vec![0.0; x.len()];
    apply_atmospheric_heat_matrix(geometry, &capacity, diffusion_scale_j_k, &x, &mut matrix_x);
    let mut residual = rhs
        .iter()
        .zip(matrix_x.iter())
        .map(|(b, ax)| b - ax)
        .collect::<Vec<_>>();
    let mut preconditioned = residual
        .iter()
        .enumerate()
        .map(|(i, r)| r / diagonal[i].max(1.0e-18))
        .collect::<Vec<_>>();
    let mut direction = preconditioned.clone();
    let mut matrix_direction = vec![0.0; x.len()];
    let mut rho = residual
        .iter()
        .zip(preconditioned.iter())
        .map(|(r, z)| r * z)
        .sum::<f64>();

    for _ in 0..usize::from(parameters.atmospheric_heat_solver_iterations) {
        if !rho.is_finite() || rho <= 1.0e-18 {
            break;
        }
        apply_atmospheric_heat_matrix(
            geometry,
            &capacity,
            diffusion_scale_j_k,
            &direction,
            &mut matrix_direction,
        );
        let denominator = direction
            .iter()
            .zip(matrix_direction.iter())
            .map(|(d, ad)| d * ad)
            .sum::<f64>();
        if !denominator.is_finite() || denominator <= 1.0e-18 {
            break;
        }
        let alpha = rho / denominator;
        for i in 0..x.len() {
            x[i] += alpha * direction[i];
            residual[i] -= alpha * matrix_direction[i];
            preconditioned[i] = residual[i] / diagonal[i].max(1.0e-18);
        }
        let next_rho = residual
            .iter()
            .zip(preconditioned.iter())
            .map(|(r, z)| r * z)
            .sum::<f64>();
        if !next_rho.is_finite() || next_rho <= 1.0e-18 {
            break;
        }
        let beta = next_rho / rho;
        for i in 0..direction.len() {
            direction[i] = preconditioned[i] + beta * direction[i];
        }
        rho = next_rho;
    }

    for i in 0..temperature.len() {
        temperature[i] = x[i].clamp(120.0, 355.0);
    }
}

fn exchange_air_sea_heat(
    air_temperature_k: &mut f64,
    sea_surface_temperature_k: &mut f64,
    pressure_pa: f64,
    planet: PlanetPhysicalParameters,
    physical: ClimatePhysicalParameters,
    parameters: ClimateParameters,
    phase_seconds: f64,
) {
    if pressure_pa <= 0.0 || parameters.air_sea_exchange_coefficient_w_m2_k <= 0.0 {
        return;
    }
    const WATER_DENSITY_KG_M3: f64 = 1_000.0;
    const WATER_SPECIFIC_HEAT_J_KG_K: f64 = 3_990.0;
    let air_capacity = (pressure_pa / planet.surface_gravity_m_s2
        * physical.atmospheric_specific_heat_j_per_kg_k)
        .max(1.0);
    let ocean_capacity =
        (parameters.ocean_mixed_layer_depth_m * WATER_DENSITY_KG_M3 * WATER_SPECIFIC_HEAT_J_KG_K)
            .max(1.0);
    let total_capacity = air_capacity + ocean_capacity;
    let equilibrium = (air_capacity * *air_temperature_k
        + ocean_capacity * *sea_surface_temperature_k)
        / total_capacity;
    let difference = *air_temperature_k - *sea_surface_temperature_k;
    let decay_rate = parameters.air_sea_exchange_coefficient_w_m2_k
        * (1.0 / air_capacity + 1.0 / ocean_capacity);
    let remaining_difference = difference * (-decay_rate * phase_seconds).exp();
    *air_temperature_k =
        (equilibrium + ocean_capacity / total_capacity * remaining_difference).clamp(120.0, 355.0);
    *sea_surface_temperature_k =
        (equilibrium - air_capacity / total_capacity * remaining_difference).clamp(250.0, 330.0);
}

#[derive(Clone, Copy, Debug)]
struct OceanProjectionEdge {
    a: usize,
    b: usize,
    a_east: f64,
    a_north: f64,
    b_east: f64,
    b_north: f64,
    interface_length_m: f64,
    conductance: f64,
}

#[derive(Clone, Debug)]
struct OceanProjectionGeometry {
    edges: Vec<OceanProjectionEdge>,
    diagonal: Vec<f64>,
}

fn edge_direction_components(
    topology: &GeodesicTopology,
    from: usize,
    to: usize,
    east: [f64; 3],
    north: [f64; 3],
) -> Option<(f64, f64)> {
    let origin = topology.positions()[from];
    let target = topology.positions()[to];
    let radial = dot(target, origin);
    let tangent = [
        target[0] - origin[0] * radial,
        target[1] - origin[1] * radial,
        target[2] - origin[2] * radial,
    ];
    let magnitude = dot(tangent, tangent).sqrt();
    if magnitude <= 1.0e-15 {
        return None;
    }
    let direction = [
        tangent[0] / magnitude,
        tangent[1] / magnitude,
        tangent[2] / magnitude,
    ];
    Some((dot(direction, east), dot(direction, north)))
}

fn symmetric_edge_normal_wind_m_s(
    topology: &GeodesicTopology,
    a: usize,
    b: usize,
    east_bases: &[[f64; 3]],
    north_bases: &[[f64; 3]],
    wind_east: &[f64],
    wind_north: &[f64],
) -> Option<f64> {
    let (a_east, a_north) =
        edge_direction_components(topology, a, b, east_bases[a], north_bases[a])?;
    let (b_east, b_north) =
        edge_direction_components(topology, b, a, east_bases[b], north_bases[b])?;
    let outward_a = wind_east[a] * a_east + wind_north[a] * a_north;
    let outward_b = wind_east[b] * b_east + wind_north[b] * b_north;
    Some(0.5 * (outward_a - outward_b))
}

#[derive(Clone, Copy, Debug)]
struct AtmosphericMoistureEdge {
    a: usize,
    b: usize,
    a_east: f64,
    a_north: f64,
    b_east: f64,
    b_north: f64,
    interface_length_m: f64,
}

fn build_atmospheric_moisture_edges(
    topology: &GeodesicTopology,
    east_bases: &[[f64; 3]],
    north_bases: &[[f64; 3]],
    radius_m: f64,
) -> Vec<AtmosphericMoistureEdge> {
    let mut edges = Vec::new();
    for a in 0..topology.metrics().sample_count as usize {
        for (neighbor, interface_arc) in topology
            .neighbors_of(a as u32)
            .iter()
            .zip(topology.neighbor_interface_arc_lengths_of(a as u32).iter())
        {
            let b = *neighbor as usize;
            if b <= a {
                continue;
            }
            let Some((a_east, a_north)) =
                edge_direction_components(topology, a, b, east_bases[a], north_bases[a])
            else {
                continue;
            };
            let Some((b_east, b_north)) =
                edge_direction_components(topology, b, a, east_bases[b], north_bases[b])
            else {
                continue;
            };
            edges.push(AtmosphericMoistureEdge {
                a,
                b,
                a_east,
                a_north,
                b_east,
                b_north,
                interface_length_m: (*interface_arc * radius_m).max(1.0),
            });
        }
    }
    edges
}

fn moisture_transport_substeps_for_phase(
    edges: &[AtmosphericMoistureEdge],
    cell_area_m2: &[f64],
    wind_east: &[f64],
    wind_north: &[f64],
    phase_seconds: f64,
    parameters: ClimateParameters,
) -> u8 {
    let mut outgoing_rate = vec![0.0; cell_area_m2.len()];
    for edge in edges {
        let outward_a = wind_east[edge.a] * edge.a_east + wind_north[edge.a] * edge.a_north;
        let outward_b = wind_east[edge.b] * edge.b_east + wind_north[edge.b] * edge.b_north;
        let normal_speed = (0.5 * (outward_a - outward_b)).clamp(
            -parameters.maximum_climatological_moisture_transport_speed_m_s,
            parameters.maximum_climatological_moisture_transport_speed_m_s,
        );
        if normal_speed > 0.0 {
            outgoing_rate[edge.a] += normal_speed * edge.interface_length_m;
        } else if normal_speed < 0.0 {
            outgoing_rate[edge.b] += -normal_speed * edge.interface_length_m;
        }
    }
    let maximum_phase_courant = outgoing_rate
        .iter()
        .enumerate()
        .map(|(i, rate)| rate * phase_seconds / cell_area_m2[i].max(1.0))
        .fold(0.0_f64, f64::max);
    let required = (maximum_phase_courant / parameters.moisture_transport_cfl_limit)
        .ceil()
        .max(1.0) as u32;
    required.clamp(
        u32::from(parameters.moisture_transport_minimum_substeps),
        u32::from(parameters.moisture_transport_maximum_substeps),
    ) as u8
}

fn advect_moisture_substep(
    edges: &[AtmosphericMoistureEdge],
    moisture_mass: &mut [f64],
    cell_area_m2: &[f64],
    wind_east: &[f64],
    wind_north: &[f64],
    substep_seconds: f64,
    cfl_limit: f64,
    maximum_speed_m_s: f64,
) -> (Vec<f64>, usize, usize) {
    let mut requested = Vec::<(usize, usize, f64)>::with_capacity(edges.len());
    let mut requested_outflow = vec![0.0; moisture_mass.len()];
    for edge in edges {
        let outward_a = wind_east[edge.a] * edge.a_east + wind_north[edge.a] * edge.a_north;
        let outward_b = wind_east[edge.b] * edge.b_east + wind_north[edge.b] * edge.b_north;
        let normal_speed =
            (0.5 * (outward_a - outward_b)).clamp(-maximum_speed_m_s, maximum_speed_m_s);
        if normal_speed.abs() <= 1.0e-12 {
            continue;
        }
        let (donor, receiver) = if normal_speed >= 0.0 {
            (edge.a, edge.b)
        } else {
            (edge.b, edge.a)
        };
        let donor_column_moisture = moisture_mass[donor] / cell_area_m2[donor].max(1.0);
        let mass =
            donor_column_moisture * normal_speed.abs() * edge.interface_length_m * substep_seconds;
        if mass > 0.0 {
            requested.push((donor, receiver, mass));
            requested_outflow[donor] += mass;
        }
    }
    let mut donor_scale = vec![1.0; moisture_mass.len()];
    let mut active_donors = 0usize;
    let mut limited_donors = 0usize;
    for i in 0..moisture_mass.len() {
        if requested_outflow[i] <= 0.0 {
            continue;
        }
        active_donors += 1;
        let allowed = moisture_mass[i] * cfl_limit;
        if requested_outflow[i] > allowed {
            donor_scale[i] = if requested_outflow[i] > 0.0 {
                allowed / requested_outflow[i]
            } else {
                1.0
            };
            limited_donors += 1;
        }
    }
    let mut delta = vec![0.0; moisture_mass.len()];
    for (donor, receiver, mass) in requested {
        let transfer = mass * donor_scale[donor];
        delta[donor] -= transfer;
        delta[receiver] += transfer;
    }
    for i in 0..moisture_mass.len() {
        moisture_mass[i] = (moisture_mass[i] + delta[i]).max(0.0);
    }
    (delta, limited_donors, active_donors)
}

fn build_ocean_projection_geometry(
    topology: &GeodesicTopology,
    ocean: &[bool],
    east_bases: &[[f64; 3]],
    north_bases: &[[f64; 3]],
    radius_m: f64,
) -> OceanProjectionGeometry {
    let mut edges = Vec::new();
    let mut diagonal = vec![0.0; ocean.len()];
    for a in 0..ocean.len() {
        if !ocean[a] {
            continue;
        }
        for ((neighbor, arc), interface_arc) in topology
            .neighbors_of(a as u32)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(a as u32).iter())
            .zip(topology.neighbor_interface_arc_lengths_of(a as u32).iter())
        {
            let b = *neighbor as usize;
            if b <= a || !ocean[b] {
                continue;
            }
            let Some((a_east, a_north)) =
                edge_direction_components(topology, a, b, east_bases[a], north_bases[a])
            else {
                continue;
            };
            let Some((b_east, b_north)) =
                edge_direction_components(topology, b, a, east_bases[b], north_bases[b])
            else {
                continue;
            };
            let distance_m = (*arc * radius_m).max(1.0);
            let interface_length_m = (*interface_arc * radius_m).max(1.0);
            let conductance = (interface_length_m / distance_m).max(1.0e-12);
            edges.push(OceanProjectionEdge {
                a,
                b,
                a_east,
                a_north,
                b_east,
                b_north,
                interface_length_m,
                conductance,
            });
            diagonal[a] += conductance;
            diagonal[b] += conductance;
        }
    }
    OceanProjectionGeometry { edges, diagonal }
}

fn apply_ocean_laplacian(geometry: &OceanProjectionGeometry, values: &[f64], output: &mut [f64]) {
    output.fill(0.0);
    for edge in &geometry.edges {
        let contribution = edge.conductance * (values[edge.a] - values[edge.b]);
        output[edge.a] += contribution;
        output[edge.b] -= contribution;
    }
}

fn correct_ocean_currents(
    ocean: &[bool],
    geometry: &OceanProjectionGeometry,
    current_east: &mut [f64],
    current_north: &mut [f64],
    projected_edge_transport_m2_s: &mut [f64],
    parameters: &ClimateParameters,
) -> f64 {
    debug_assert_eq!(projected_edge_transport_m2_s.len(), geometry.edges.len());
    projected_edge_transport_m2_s.fill(0.0);
    if geometry.edges.is_empty() {
        current_east.fill(0.0);
        current_north.fill(0.0);
        return 0.0;
    }

    // Convert endpoint ENU vectors into one antisymmetric transport value per
    // ocean-ocean interface. Land interfaces never enter this graph.
    let mut divergence = vec![0.0; ocean.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let outward_a = current_east[edge.a] * edge.a_east + current_north[edge.a] * edge.a_north;
        let outward_b = current_east[edge.b] * edge.b_east + current_north[edge.b] * edge.b_north;
        let normal_speed = 0.5 * (outward_a - outward_b);
        let flux = normal_speed * edge.interface_length_m;
        projected_edge_transport_m2_s[edge_index] = flux;
        divergence[edge.a] += flux;
        divergence[edge.b] -= flux;
    }

    // Solve L p = div(q) with a diagonally preconditioned conjugate-gradient
    // projection. Because edge flux is antisymmetric, each connected ocean
    // component has zero net right-hand side and the constant pressure null
    // mode does not affect the corrected transport.
    let mut pressure = vec![0.0; ocean.len()];
    let mut residual = divergence.clone();
    let mut preconditioned = vec![0.0; ocean.len()];
    let mut direction = vec![0.0; ocean.len()];
    let mut laplacian_direction = vec![0.0; ocean.len()];
    for i in 0..ocean.len() {
        if ocean[i] && geometry.diagonal[i] > 0.0 {
            preconditioned[i] = residual[i] / geometry.diagonal[i];
            direction[i] = preconditioned[i];
        }
    }
    let mut rho = residual
        .iter()
        .zip(preconditioned.iter())
        .map(|(r, z)| r * z)
        .sum::<f64>();
    for _ in 0..usize::from(parameters.ocean_current_correction_iterations) {
        if !rho.is_finite() || rho <= 1.0e-24 {
            break;
        }
        apply_ocean_laplacian(geometry, &direction, &mut laplacian_direction);
        let denominator = direction
            .iter()
            .zip(laplacian_direction.iter())
            .map(|(d, q)| d * q)
            .sum::<f64>();
        if !denominator.is_finite() || denominator <= 1.0e-24 {
            break;
        }
        let alpha = rho / denominator;
        for i in 0..ocean.len() {
            pressure[i] += alpha * direction[i];
            residual[i] -= alpha * laplacian_direction[i];
            preconditioned[i] = if ocean[i] && geometry.diagonal[i] > 0.0 {
                residual[i] / geometry.diagonal[i]
            } else {
                0.0
            };
        }
        let next_rho = residual
            .iter()
            .zip(preconditioned.iter())
            .map(|(r, z)| r * z)
            .sum::<f64>();
        if !next_rho.is_finite() || next_rho <= 1.0e-24 {
            break;
        }
        let beta = next_rho / rho;
        for i in 0..ocean.len() {
            direction[i] = preconditioned[i] + beta * direction[i];
        }
        rho = next_rho;
    }

    let mut projected_divergence = vec![0.0; ocean.len()];
    let mut perimeter = vec![0.0; ocean.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let correction = edge.conductance * (pressure[edge.a] - pressure[edge.b]);
        projected_edge_transport_m2_s[edge_index] -= correction;
        projected_divergence[edge.a] += projected_edge_transport_m2_s[edge_index];
        projected_divergence[edge.b] -= projected_edge_transport_m2_s[edge_index];
        perimeter[edge.a] += edge.interface_length_m;
        perimeter[edge.b] += edge.interface_length_m;
    }

    // Reconstruct the best-fit local ENU display/diagnostic vector from the
    // conservative edge-normal transports. This also naturally turns flow
    // along coastlines because blocked land edges are absent from the solve.
    let mut matrix_ee = vec![0.0; ocean.len()];
    let mut matrix_en = vec![0.0; ocean.len()];
    let mut matrix_nn = vec![0.0; ocean.len()];
    let mut rhs_e = vec![0.0; ocean.len()];
    let mut rhs_n = vec![0.0; ocean.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let normal_speed = projected_edge_transport_m2_s[edge_index] / edge.interface_length_m;
        for (sample, east, north, speed) in [
            (edge.a, edge.a_east, edge.a_north, normal_speed),
            (edge.b, edge.b_east, edge.b_north, -normal_speed),
        ] {
            let weight = edge.interface_length_m;
            matrix_ee[sample] += weight * east * east;
            matrix_en[sample] += weight * east * north;
            matrix_nn[sample] += weight * north * north;
            rhs_e[sample] += weight * speed * east;
            rhs_n[sample] += weight * speed * north;
        }
    }
    current_east.fill(0.0);
    current_north.fill(0.0);
    for i in 0..ocean.len() {
        if !ocean[i] || perimeter[i] <= 0.0 {
            continue;
        }
        let trace = matrix_ee[i] + matrix_nn[i];
        let regularization = (trace * 1.0e-10).max(1.0e-12);
        let a = matrix_ee[i] + regularization;
        let b = matrix_en[i];
        let d = matrix_nn[i] + regularization;
        let determinant = a * d - b * b;
        if determinant.abs() <= 1.0e-18 {
            continue;
        }
        let east = (rhs_e[i] * d - rhs_n[i] * b) / determinant;
        let north = (rhs_n[i] * a - rhs_e[i] * b) / determinant;
        (current_east[i], current_north[i]) =
            clamp_vector(east, north, parameters.maximum_surface_current_m_s);
    }

    let mut residual_speed = 0.0;
    let mut residual_samples = 0.0;
    for i in 0..ocean.len() {
        if ocean[i] && perimeter[i] > 0.0 {
            residual_speed += projected_divergence[i].abs() / perimeter[i];
            residual_samples += 1.0;
        }
    }
    if residual_samples > 0.0 {
        residual_speed / residual_samples
    } else {
        0.0
    }
}

fn conservative_ocean_heat_tendency(
    geometry: &OceanProjectionGeometry,
    edge_transport_m2_s: &[f64],
    temperature_k: &[f64],
    cell_area_m2: &[f64],
    phase_seconds: f64,
    advection_relaxation: f64,
    cfl_limit: f64,
    output_k_s: &mut [f64],
) {
    debug_assert_eq!(edge_transport_m2_s.len(), geometry.edges.len());
    output_k_s.fill(0.0);
    if advection_relaxation <= 0.0 || phase_seconds <= 0.0 {
        return;
    }

    // The projected edge transport is conservative, but one climatology phase
    // spans many physical advection times at L7. Limit aggregate donor outflow
    // rather than clamping cell tendencies independently so the explicit
    // donor-cell heat step remains both stable and conservative.
    let mut outgoing_transport_m2_s = vec![0.0; temperature_k.len()];
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let transport = edge_transport_m2_s[edge_index];
        if transport > 0.0 {
            outgoing_transport_m2_s[edge.a] += transport;
        } else if transport < 0.0 {
            outgoing_transport_m2_s[edge.b] += -transport;
        }
    }
    let mut donor_scale = vec![1.0; temperature_k.len()];
    for sample in 0..temperature_k.len() {
        let outgoing = outgoing_transport_m2_s[sample];
        if outgoing <= 0.0 {
            continue;
        }
        let requested_fraction =
            outgoing * phase_seconds * advection_relaxation / cell_area_m2[sample].max(1.0);
        if requested_fraction > cfl_limit {
            donor_scale[sample] = cfl_limit / requested_fraction;
        }
    }

    for (edge_index, edge) in geometry.edges.iter().enumerate() {
        let transport = edge_transport_m2_s[edge_index];
        if transport.abs() <= 1.0e-18 {
            continue;
        }
        let upstream = if transport >= 0.0 { edge.a } else { edge.b };
        let effective_transport = transport * advection_relaxation * donor_scale[upstream];
        let advected_anomaly_k = temperature_k[upstream] - 273.15;
        let heat_transport = effective_transport * advected_anomaly_k;
        output_k_s[edge.a] -= heat_transport / cell_area_m2[edge.a].max(1.0);
        output_k_s[edge.b] += heat_transport / cell_area_m2[edge.b].max(1.0);
    }
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
        return Err(WorldgenError::InvalidClimate(
            "climate seed must not be empty",
        ));
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
    let mut progress = |_completed_years: u8, _maximum_years: u8| {};
    let (climate, _) = generate_coupled_climate_internal(
        topology,
        terrain,
        planet,
        request,
        false,
        &mut progress,
    )?;
    Ok(climate)
}

pub fn generate_coupled_climate_with_diagnostics(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &ClimateRequest,
    progress: &mut dyn FnMut(u8, u8),
) -> Result<(ClimateState, ClimateGenerationDiagnostics), WorldgenError> {
    generate_coupled_climate_internal(topology, terrain, planet, request, true, progress)
}

fn generate_coupled_climate_internal(
    topology: &GeodesicTopology,
    terrain: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &ClimateRequest,
    capture_precipitation_phases: bool,
    progress: &mut dyn FnMut(u8, u8),
) -> Result<(ClimateState, ClimateGenerationDiagnostics), WorldgenError> {
    validate_inputs(topology, terrain, planet, request)?;
    let parameters = request.parameters;
    let physical = request.physical;
    let sample_count = topology.metrics().sample_count as usize;
    let phase_count = usize::from(parameters.orbital_phase_count);
    let mut precipitation_phase_rate_mm_year = if capture_precipitation_phases {
        vec![0.0_f32; phase_count * sample_count]
    } else {
        Vec::new()
    };
    let stage_seed = derive_stage_seed(request.seed.as_str(), CLIMATE_NAMESPACE);
    let omega = if planet.rotation_period_s > 0.0 {
        TWO_PI / planet.rotation_period_s
    } else {
        0.0
    };
    let rotation_ratio = rotation_response(planet.rotation_period_s);
    let (hadley_edge_deg, _) = circulation_cell_edges(rotation_ratio);
    let rotational_strength = rotation_ratio.sqrt().clamp(0.35, 2.5);
    let overturning_strength = (1.0 / rotation_ratio.sqrt()).clamp(0.4, 2.5);
    let rotational_transition_start_deg = (4.0 / rotational_strength).clamp(2.0, 12.0);
    let rotational_transition_width_deg = (18.0 / rotational_strength).clamp(8.0, 36.0);
    let atmosphere_exists = planet.reference_surface_pressure_pa > 0.0;
    let specific_gas_constant =
        UNIVERSAL_GAS_CONSTANT / physical.atmospheric_mean_molar_mass_kg_per_mol;
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
        terrain_height_m[i] = atmospheric_surface_height_m(
            ocean[i],
            f64::from(terrain.elevation_above_sea_level_m[i]),
        );
        water_depth_m[i] = f64::from(terrain.water_depth_m[i]).max(0.0);
    }

    let terrain_values = terrain_height_m.clone();
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

    let atmospheric_heat_geometry = build_atmospheric_heat_geometry(topology, planet.radius_m);
    let atmospheric_moisture_edges =
        build_atmospheric_moisture_edges(topology, &east_bases, &north_bases, planet.radius_m);
    let ocean_projection_geometry = build_ocean_projection_geometry(
        topology,
        &ocean,
        &east_bases,
        &north_bases,
        planet.radius_m,
    );
    let mut ocean_edge_transport_m2_s = vec![0.0; ocean_projection_geometry.edges.len()];
    let mut ocean_heat_tendency_k_s = vec![0.0; sample_count];

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
        temperature[i] =
            (289.0 - 48.0 * lat_factor - parameters.lapse_rate_k_per_m * terrain_height_m[i])
                .clamp(170.0, 335.0);
        sea_surface_temperature[i] = if ocean[i] {
            (290.0 - 42.0 * lat_factor).clamp(268.0, 307.0)
        } else {
            temperature[i]
        };
        if atmosphere_exists {
            let scale_height =
                (specific_gas_constant * temperature[i] / planet.surface_gravity_m_s2).max(1.0);
            pressure[i] =
                planet.reference_surface_pressure_pa * (-terrain_height_m[i] / scale_height).exp();
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
    let mut wind_speed_sum = vec![0.0; sample_count];
    let mut maximum_wind_speed_over_phases = 0.0_f64;
    let mut sst_sum = vec![0.0; sample_count];
    let mut sst_cos = vec![0.0; sample_count];
    let mut sst_sin = vec![0.0; sample_count];
    let mut current_east_sum = vec![0.0; sample_count];
    let mut current_north_sum = vec![0.0; sample_count];
    let mut current_east_cos = vec![0.0; sample_count];
    let mut current_east_sin = vec![0.0; sample_count];
    let mut current_north_cos = vec![0.0; sample_count];
    let mut current_north_sin = vec![0.0; sample_count];
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
    let mut moisture_transport_limited_donor_steps = 0usize;
    let mut moisture_transport_active_donor_steps = 0usize;
    let mut maximum_moisture_transport_substeps_used = 0u8;
    let mut final_temperature_rms_change = f64::INFINITY;
    let mut spinup_years = parameters.maximum_spinup_years;
    let mut maximum_ocean_divergence_residual = 0.0_f64;

    for year in 0..parameters.maximum_spinup_years {
        progress(year, parameters.maximum_spinup_years);
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
        wind_speed_sum.fill(0.0);
        maximum_wind_speed_over_phases = 0.0;
        sst_sum.fill(0.0);
        sst_cos.fill(0.0);
        sst_sin.fill(0.0);
        current_east_sum.fill(0.0);
        current_north_sum.fill(0.0);
        current_east_cos.fill(0.0);
        current_east_sin.fill(0.0);
        current_north_cos.fill(0.0);
        current_north_sin.fill(0.0);
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
        moisture_transport_limited_donor_steps = 0;
        moisture_transport_active_donor_steps = 0;
        maximum_moisture_transport_substeps_used = 0;
        maximum_ocean_divergence_residual = 0.0;

        for phase in 0..phase_count {
            let mean_longitude = TWO_PI * phase as f64 / phase_count as f64;
            let (solar_longitude, distance_factor) = solve_orbital_forcing(
                mean_longitude,
                physical.orbital_eccentricity,
                physical.longitude_of_periapsis_rad,
            );
            let declination = (planet.axial_tilt_rad.sin() * solar_longitude.sin()).asin();
            let phase_angle = mean_longitude;
            let phase_cos = phase_angle.cos();
            let phase_sin = phase_angle.sin();

            let mut insolation = vec![0.0; sample_count];
            let mut absorbed_surface_energy_w_m2 = vec![0.0; sample_count];
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
                let effective_albedo = if atmosphere_exists {
                    effective_shortwave_albedo(
                        physical.atmospheric_shortwave_reflectivity,
                        parameters.surface_albedo_shortwave_coupling,
                        albedo,
                    )
                } else {
                    // With no atmosphere there is no unresolved atmospheric/cloud
                    // shortwave masking: the exposed surface albedo is the TOA albedo.
                    albedo
                };
                let absorbed = (solar * (1.0 - effective_albedo)
                    + planet.internal_heat_flux_w_per_m2)
                    .max(0.0);
                absorbed_surface_energy_w_m2[i] = absorbed;
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
                let relaxation = if ocean[i] {
                    parameters.ocean_thermal_relaxation
                } else {
                    parameters.land_thermal_relaxation
                };
                temperature[i] = (previous_temperature[i]
                    + relaxation * (radiative_target[i] - previous_temperature[i]))
                    .clamp(120.0, 355.0);
            }
            diffuse_atmospheric_heat(
                &atmospheric_heat_geometry,
                &mut temperature,
                &pressure,
                &cell_area_m2,
                planet,
                physical,
                parameters,
                phase_seconds,
            );
            for i in 0..sample_count {
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
                    let rotational_blend = ((latitude_abs_deg - rotational_transition_start_deg)
                        / rotational_transition_width_deg)
                        .clamp(0.0, 1.0);
                    let geostrophic_east = -gradient_north
                        * 1_000_000.0
                        * parameters.wind_thermal_gradient_scale
                        * rotational_strength;
                    let geostrophic_north = gradient_east
                        * 1_000_000.0
                        * parameters.wind_thermal_gradient_scale
                        * rotational_strength;
                    let zonal = baseline_zonal_wind(latitude[i], rotation_ratio);
                    let meridional = if latitude_abs_deg < hadley_edge_deg {
                        -latitude[i].signum()
                            * 2.6
                            * overturning_strength
                            * (1.0 - latitude_abs_deg / hadley_edge_deg)
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
            ocean_edge_transport_m2_s.fill(0.0);
            ocean_heat_tendency_k_s.fill(0.0);
            if planet.surface_water_mass_kg > 0.0 {
                for i in 0..sample_count {
                    if !ocean[i] {
                        continue;
                    }
                    let deflection = coriolis_deflection_factor(
                        latitude[i],
                        omega,
                        parameters.ocean_coriolis_deflection,
                    );
                    let mobility = (water_depth_m[i] / parameters.ocean_bathymetric_drag_depth_m)
                        .clamp(0.08, 1.0)
                        .sqrt();
                    let east = parameters.ocean_wind_coupling
                        * mobility
                        * (wind_east[i] + deflection * wind_north[i]);
                    let north = parameters.ocean_wind_coupling
                        * mobility
                        * (wind_north[i] - deflection * wind_east[i]);
                    (current_east[i], current_north[i]) =
                        clamp_vector(east, north, parameters.maximum_surface_current_m_s);
                }
                let phase_divergence_residual = correct_ocean_currents(
                    &ocean,
                    &ocean_projection_geometry,
                    &mut current_east,
                    &mut current_north,
                    &mut ocean_edge_transport_m2_s,
                    &parameters,
                );
                maximum_ocean_divergence_residual =
                    maximum_ocean_divergence_residual.max(phase_divergence_residual);
            }

            let previous_sst = sea_surface_temperature.clone();
            conservative_ocean_heat_tendency(
                &ocean_projection_geometry,
                &ocean_edge_transport_m2_s,
                &previous_sst,
                &cell_area_m2,
                phase_seconds,
                parameters.ocean_advection_relaxation,
                parameters.ocean_advection_cfl_limit,
                &mut ocean_heat_tendency_k_s,
            );
            let mut next_sst = previous_sst.clone();
            for i in 0..sample_count {
                if !ocean[i] {
                    next_sst[i] = temperature[i];
                    continue;
                }
                let advection_delta = (ocean_heat_tendency_k_s[i] * phase_seconds).clamp(-4.0, 4.0);
                let neighbor_sst = mean_ocean_neighbor(topology, &ocean, &previous_sst, i);
                next_sst[i] = (previous_sst[i]
                    + advection_delta
                    + parameters.ocean_temperature_diffusion * (neighbor_sst - previous_sst[i]))
                    .clamp(250.0, 330.0);
            }
            sea_surface_temperature = next_sst;
            for i in 0..sample_count {
                if !ocean[i] {
                    continue;
                }
                if atmosphere_exists {
                    exchange_air_sea_heat(
                        &mut temperature[i],
                        &mut sea_surface_temperature[i],
                        pressure[i],
                        planet,
                        physical,
                        parameters,
                        phase_seconds,
                    );
                } else {
                    // The temperature field is the exposed radiative surface state
                    // on an airless body, so there is no distinct air/SST reservoir.
                    sea_surface_temperature[i] = temperature[i];
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
                let mut phase_evaporation = 0.0;
                let mut phase_precipitation = 0.0;
                let mut precipitation_mass_phase = vec![0.0; sample_count];

                // Bulk-aerodynamic evaporation is expressed as a surface mass flux
                // rather than a per-phase humidity relaxation, making the source
                // independent of mesh resolution and orbital phase count.
                let mut requested_ocean_evaporation_mass = vec![0.0; sample_count];
                let mut requested_ocean_evaporation_total = 0.0;
                let mut ocean_absorbed_power_w = 0.0;
                for i in 0..sample_count {
                    if air_mass[i] <= 0.0 {
                        humidity[i] = 0.0;
                        continue;
                    }
                    let q = moisture_mass[i] / air_mass[i];
                    let wind_speed = norm2(wind_east[i], wind_north[i]).max(1.0);
                    let surface_temperature = if ocean[i] {
                        sea_surface_temperature[i]
                    } else {
                        temperature[i]
                    };
                    let saturation_surface =
                        saturation_specific_humidity(surface_temperature, pressure[i]);
                    let density = pressure[i] / (specific_gas_constant * temperature[i].max(120.0));
                    let evaporation_flux = density
                        * parameters.evaporation_bulk_transfer_coefficient
                        * wind_speed
                        * (saturation_surface - q).max(0.0);
                    let potential_mass = evaporation_flux * cell_area_m2[i] * phase_seconds;
                    potential_evaporation_mass_year[i] += potential_mass;
                    if ocean[i] {
                        requested_ocean_evaporation_mass[i] = potential_mass;
                        requested_ocean_evaporation_total += potential_mass;
                        ocean_absorbed_power_w += absorbed_surface_energy_w_m2[i] * cell_area_m2[i];
                    }
                }
                let energy_limited_ocean_evaporation_mass =
                    parameters.evaporation_energy_fraction * ocean_absorbed_power_w * phase_seconds
                        / LATENT_HEAT_VAPORIZATION_J_PER_KG;
                let evaporation_energy_scale = if requested_ocean_evaporation_total > 0.0 {
                    (energy_limited_ocean_evaporation_mass / requested_ocean_evaporation_total)
                        .clamp(0.0, 1.0)
                } else {
                    1.0
                };
                for i in 0..sample_count {
                    if !ocean[i] {
                        continue;
                    }
                    let evaporation_mass =
                        requested_ocean_evaporation_mass[i] * evaporation_energy_scale;
                    moisture_mass[i] += evaporation_mass;
                    phase_evaporation += evaporation_mass;
                }

                // Resolve one seasonal wind state through multiple conservative
                // finite-volume advection substeps. Moisture can therefore cross
                // multiple cells during a phase without an index-order dependency.
                let moisture_substeps = moisture_transport_substeps_for_phase(
                    &atmospheric_moisture_edges,
                    &cell_area_m2,
                    &wind_east,
                    &wind_north,
                    phase_seconds,
                    parameters,
                );
                maximum_moisture_transport_substeps_used =
                    maximum_moisture_transport_substeps_used.max(moisture_substeps);
                let substep_seconds = phase_seconds / f64::from(moisture_substeps);
                for _ in 0..usize::from(moisture_substeps) {
                    let (transport_delta, limited_donors, active_donors) = advect_moisture_substep(
                        &atmospheric_moisture_edges,
                        &mut moisture_mass,
                        &cell_area_m2,
                        &wind_east,
                        &wind_north,
                        substep_seconds,
                        parameters.moisture_transport_cfl_limit,
                        parameters.maximum_climatological_moisture_transport_speed_m_s,
                    );
                    moisture_transport_limited_donor_steps += limited_donors;
                    moisture_transport_active_donor_steps += active_donors;
                    for i in 0..sample_count {
                        if transport_delta[i] <= 0.0 || air_mass[i] <= 0.0 {
                            continue;
                        }
                        let saturation_air =
                            saturation_specific_humidity(temperature[i], pressure[i]);
                        if saturation_air <= 1.0e-12 {
                            continue;
                        }
                        let relative_humidity =
                            (moisture_mass[i] / air_mass[i] / saturation_air).max(0.0);
                        let threshold = parameters.convergence_precipitation_relative_humidity;
                        let activation = if threshold < 1.0 {
                            ((relative_humidity - threshold) / (1.0 - threshold)).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        let convergence_mass = transport_delta[i]
                            * parameters.convergence_precipitation_efficiency
                            * activation;
                        let precipitation_mass = convergence_mass.min(moisture_mass[i]);
                        moisture_mass[i] -= precipitation_mass;
                        precipitation_mass_phase[i] += precipitation_mass;
                    }
                }

                for i in 0..sample_count {
                    if air_mass[i] <= 0.0 {
                        humidity[i] = 0.0;
                        continue;
                    }
                    let wind_speed = norm2(wind_east[i], wind_north[i]);
                    let current_q = moisture_mass[i] / air_mass[i];
                    let saturation_air = saturation_specific_humidity(temperature[i], pressure[i]);
                    let threshold = saturation_air * parameters.condensation_relative_humidity;
                    let excess_q = (current_q - threshold).max(0.0);
                    let condensation_mass =
                        excess_q * air_mass[i] * parameters.condensation_efficiency;
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
                    precipitation_mass_phase[i] += precipitation_mass;
                    let phase_cell_precipitation = precipitation_mass_phase[i];
                    precipitation_mass_year[i] += phase_cell_precipitation;
                    precipitation_phase_max[i] =
                        precipitation_phase_max[i].max(phase_cell_precipitation);
                    if temperature[i] <= parameters.snow_temperature_k
                        && phase_cell_precipitation > 0.0
                    {
                        cold_precipitation_mass_year[i] += phase_cell_precipitation;
                        snow_phase_count[i] += 1.0;
                    }
                    if ocean[i] && sea_surface_temperature[i] <= parameters.sea_ice_temperature_k {
                        sea_ice_phase_count[i] += 1.0;
                    }
                    phase_precipitation += phase_cell_precipitation;
                    humidity[i] = (moisture_mass[i] / air_mass[i]).clamp(0.0, 0.2);
                }
                if capture_precipitation_phases {
                    let offset = phase * sample_count;
                    let annualization = phase_count as f64;
                    for i in 0..sample_count {
                        precipitation_phase_rate_mm_year[offset + i] = (precipitation_mass_phase[i]
                            / cell_area_m2[i].max(1.0)
                            * annualization)
                            as f32;
                    }
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
                let phase_wind_speed = norm2(wind_east[i], wind_north[i]);
                wind_speed_sum[i] += phase_wind_speed;
                maximum_wind_speed_over_phases =
                    maximum_wind_speed_over_phases.max(phase_wind_speed);
                sst_sum[i] += sea_surface_temperature[i];
                sst_cos[i] += sea_surface_temperature[i] * phase_cos;
                sst_sin[i] += sea_surface_temperature[i] * phase_sin;
                current_east_sum[i] += current_east[i];
                current_north_sum[i] += current_north[i];
                current_east_cos[i] += current_east[i] * phase_cos;
                current_east_sin[i] += current_east[i] * phase_sin;
                current_north_cos[i] += current_north[i] * phase_cos;
                current_north_sin[i] += current_north[i] * phase_sin;
                current_speed_sum[i] += norm2(current_east[i], current_north[i]);
                ocean_heat_transport_sum[i] += if ocean[i] {
                    ocean_heat_tendency_k_s[i] * 1_000_000.0
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
        progress(spinup_years, parameters.maximum_spinup_years);
        if spinup_years >= parameters.minimum_spinup_years
            && final_temperature_rms_change <= parameters.convergence_temperature_rms_k
        {
            break;
        }
    }

    if final_temperature_rms_change > parameters.convergence_temperature_rms_k {
        return Err(WorldgenError::InvalidClimate(
            "WG-5 climate did not converge within the configured spin-up bound",
        ));
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
    let mut sst_cos_out = vec![0.0; sample_count];
    let mut sst_sin_out = vec![0.0; sample_count];
    let mut current_east_mean = vec![0.0; sample_count];
    let mut current_north_mean = vec![0.0; sample_count];
    let mut current_east_cos_out = vec![0.0; sample_count];
    let mut current_east_sin_out = vec![0.0; sample_count];
    let mut current_north_cos_out = vec![0.0; sample_count];
    let mut current_north_sin_out = vec![0.0; sample_count];
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
        sst_cos_out[i] = (sst_cos[i] * harmonic_scale) as f32;
        sst_sin_out[i] = (sst_sin[i] * harmonic_scale) as f32;
        current_east_mean[i] = (current_east_sum[i] / phase_count_f64) as f32;
        current_north_mean[i] = (current_north_sum[i] / phase_count_f64) as f32;
        current_east_cos_out[i] = (current_east_cos[i] * harmonic_scale) as f32;
        current_east_sin_out[i] = (current_east_sin[i] * harmonic_scale) as f32;
        current_north_cos_out[i] = (current_north_cos[i] * harmonic_scale) as f32;
        current_north_sin_out[i] = (current_north_sin[i] * harmonic_scale) as f32;
        current_speed_mean[i] = (current_speed_sum[i] / phase_count_f64) as f32;
        ocean_heat_transport[i] = (ocean_heat_transport_sum[i] / phase_count_f64) as f32;
        humidity_mean[i] = (humidity_sum[i] / phase_count_f64) as f32;
        let area = cell_area_m2[i].max(1.0);
        annual_precipitation_mm[i] = (precipitation_mass_year[i] / area) as f32;
        potential_evaporation_mm[i] = (potential_evaporation_mass_year[i] / area) as f32;
        moisture_balance_mm[i] = annual_precipitation_mm[i] - potential_evaporation_mm[i];
        aridity_index[i] = if potential_evaporation_mm[i] > 1.0 {
            (annual_precipitation_mm[i] / potential_evaporation_mm[i]).clamp(0.0, 4.0)
        } else {
            4.0
        };
        let mean_phase_precipitation = precipitation_mass_year[i] / phase_count_f64;
        precipitation_seasonality[i] = if mean_phase_precipitation > 0.0 {
            ((precipitation_phase_max[i] / mean_phase_precipitation) - 1.0).clamp(0.0, 8.0) as f32
        } else {
            0.0
        };
        snowfall_fraction[i] = if precipitation_mass_year[i] > 0.0 {
            (cold_precipitation_mass_year[i] / precipitation_mass_year[i]).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        persistent_snow_potential[i] = ((snow_phase_count[i] / phase_count_f64)
            * f64::from(snowfall_fraction[i]))
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
    let wind_speed_mean = wind_speed_sum
        .iter()
        .map(|value| (value / phase_count_f64) as f32)
        .collect::<Vec<_>>();
    let mean_wind_speed = area_weighted_mean(topology, &wind_speed_mean);
    let maximum_wind_speed = maximum_wind_speed_over_phases;
    let mean_current_speed = subset_area_weighted_mean(topology, &current_speed_mean, |i| ocean[i]);
    let maximum_current_speed = current_speed_mean.iter().copied().fold(0.0_f32, f32::max) as f64;
    let mean_precipitation = area_weighted_mean(topology, &annual_precipitation_mm);
    let p95_precipitation = percentile(&annual_precipitation_mm, 0.95);
    let total_budget_scale = global_evaporation_year
        .abs()
        .max(global_precipitation_year.abs())
        .max(1.0);
    let moisture_budget_relative_error = moisture_budget_error_year / total_budget_scale;
    if moisture_budget_relative_error > 1.0e-8 {
        return Err(WorldgenError::InvalidClimate(
            "WG-5 atmospheric moisture budget did not close within tolerance",
        ));
    }
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
    climate_hash = fnv_update(climate_hash, CLIMATE_STAGE_ID.as_bytes());
    climate_hash = fnv_update(climate_hash, &CLIMATE_STAGE_VERSION.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &stage_seed.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &terrain.metrics.topography_hash.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &planet.parameter_hash().to_le_bytes());
    climate_hash = fnv_update(climate_hash, &physical_hash.to_le_bytes());
    climate_hash = fnv_update(climate_hash, &model_hash.to_le_bytes());
    for values in [
        &annual_mean_insolation_f32,
        &seasonal_insolation_amplitude,
        &temperature_mean,
        &temperature_annual_cos,
        &temperature_annual_sin,
        &temperature_min_f32,
        &temperature_max_f32,
        &pressure_f32,
        &wind_east_mean,
        &wind_north_mean,
        &wind_east_cos_out,
        &wind_east_sin_out,
        &wind_north_cos_out,
        &wind_north_sin_out,
        &sst_mean,
        &sst_cos_out,
        &sst_sin_out,
        &current_east_mean,
        &current_north_mean,
        &current_east_cos_out,
        &current_east_sin_out,
        &current_north_cos_out,
        &current_north_sin_out,
        &current_speed_mean,
        &ocean_heat_transport,
        &humidity_mean,
        &annual_precipitation_mm,
        &precipitation_seasonality,
        &potential_evaporation_mm,
        &moisture_balance_mm,
        &aridity_index,
        &snowfall_fraction,
        &persistent_snow_potential,
        &sea_ice_potential,
    ] {
        climate_hash = hash_f32_slice(climate_hash, values);
    }

    let moisture_transport_limiter_fraction = if moisture_transport_active_donor_steps > 0 {
        moisture_transport_limited_donor_steps as f64 / moisture_transport_active_donor_steps as f64
    } else {
        0.0
    };

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
        ocean_divergence_residual_m_s: maximum_ocean_divergence_residual,
        mean_sea_surface_temperature_k: mean_sst,
        mean_annual_precipitation_mm: mean_precipitation,
        p95_annual_precipitation_mm: p95_precipitation,
        global_evaporation_kg: global_evaporation_year,
        global_precipitation_kg: global_precipitation_year,
        moisture_budget_relative_error,
        moisture_transport_limiter_fraction,
        maximum_moisture_transport_substeps: maximum_moisture_transport_substeps_used,
        persistent_snow_area_fraction,
        sea_ice_area_fraction,
        final_temperature_rms_change_k: final_temperature_rms_change,
        climate_physical_parameter_hash: physical_hash,
        climate_model_parameter_hash: model_hash,
        climate_hash,
    };

    let climate_state = ClimateState {
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
        sea_surface_temperature_annual_cos_k: sst_cos_out,
        sea_surface_temperature_annual_sin_k: sst_sin_out,
        current_east_mean_m_s: current_east_mean,
        current_north_mean_m_s: current_north_mean,
        current_east_annual_cos_m_s: current_east_cos_out,
        current_east_annual_sin_m_s: current_east_sin_out,
        current_north_annual_cos_m_s: current_north_cos_out,
        current_north_annual_sin_m_s: current_north_sin_out,
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
    };

    Ok((
        climate_state,
        ClimateGenerationDiagnostics {
            precipitation_phase_rate_mm_year,
        },
    ))
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
        assert_eq!(
            physical.parameter_hash(),
            ClimatePhysicalParameters::earthlike_reference().parameter_hash()
        );
        assert_eq!(
            model.parameter_hash(),
            ClimateParameters::default().parameter_hash()
        );
        assert_eq!(physical.parameter_hash_hex().len(), 16);
        assert_eq!(model.parameter_hash_hex().len(), 16);
    }

    #[test]
    fn seasonal_solar_geometry_handles_polar_night_and_equatorial_daylight() {
        let equator = daily_mean_insolation(0.0, 0.0, 1361.0);
        let north_pole_winter =
            daily_mean_insolation(std::f64::consts::FRAC_PI_2, -23.4_f64.to_radians(), 1361.0);
        assert!(equator > 400.0);
        assert_eq!(north_pole_winter, 0.0);
    }

    #[test]
    fn conservative_edge_heat_transport_preserves_area_weighted_heat_anomaly() {
        let geometry = OceanProjectionGeometry {
            edges: vec![OceanProjectionEdge {
                a: 0,
                b: 1,
                a_east: 1.0,
                a_north: 0.0,
                b_east: -1.0,
                b_north: 0.0,
                interface_length_m: 10.0,
                conductance: 1.0,
            }],
            diagonal: vec![1.0, 1.0],
        };
        let mut tendency = vec![0.0; 2];
        conservative_ocean_heat_tendency(
            &geometry,
            &[20.0],
            &[300.0, 280.0],
            &[100.0, 200.0],
            1.0,
            1.0,
            1.0,
            &mut tendency,
        );
        let weighted = tendency[0] * 100.0 + tendency[1] * 200.0;
        assert!(weighted.abs() < 1.0e-12);
        assert!(tendency[0] < 0.0);
        assert!(tendency[1] > 0.0);
    }

    #[test]
    fn rotation_response_broadens_slow_hadley_cells_and_zeroes_equatorial_coriolis() {
        let earth_ratio = rotation_response(EARTH_REFERENCE_ROTATION_PERIOD_S);
        let slow_ratio = rotation_response(EARTH_REFERENCE_ROTATION_PERIOD_S * 4.0);
        let fast_ratio = rotation_response(EARTH_REFERENCE_ROTATION_PERIOD_S * 0.5);
        let (earth_hadley, _) = circulation_cell_edges(earth_ratio);
        let (slow_hadley, _) = circulation_cell_edges(slow_ratio);
        let (fast_hadley, _) = circulation_cell_edges(fast_ratio);
        assert!(slow_hadley > earth_hadley);
        assert!(fast_hadley < earth_hadley);
        let earth_omega = TWO_PI / EARTH_REFERENCE_ROTATION_PERIOD_S;
        assert_eq!(coriolis_deflection_factor(0.0, earth_omega, 0.55), 0.0);
        assert!(coriolis_deflection_factor(45_f64.to_radians(), earth_omega, 0.55) > 0.0);
        assert!(coriolis_deflection_factor(-45_f64.to_radians(), earth_omega, 0.55) < 0.0);
        assert!(
            coriolis_deflection_factor(45_f64.to_radians(), earth_omega * 2.0, 0.55).abs()
                > coriolis_deflection_factor(45_f64.to_radians(), earth_omega, 0.55).abs()
        );
    }

    #[test]
    fn ocean_heat_advection_cfl_limiter_is_conservative_and_bounds_donor_exchange() {
        let geometry = OceanProjectionGeometry {
            edges: vec![OceanProjectionEdge {
                a: 0,
                b: 1,
                a_east: 1.0,
                a_north: 0.0,
                b_east: -1.0,
                b_north: 0.0,
                interface_length_m: 1.0,
                conductance: 1.0,
            }],
            diagonal: vec![1.0, 1.0],
        };
        let temperature = [300.0, 280.0];
        let area = [100.0, 100.0];
        let mut tendency = [0.0, 0.0];
        conservative_ocean_heat_tendency(
            &geometry,
            &[100.0],
            &temperature,
            &area,
            10.0,
            1.0,
            0.45,
            &mut tendency,
        );
        let donor_fraction = -tendency[0] * 10.0 / (temperature[0] - 273.15);
        assert!((donor_fraction - 0.45).abs() < 1.0e-12);
        let area_weighted_tendency = tendency[0] * area[0] + tendency[1] * area[1];
        assert!(area_weighted_tendency.abs() < 1.0e-10);

        conservative_ocean_heat_tendency(
            &geometry,
            &[100.0],
            &temperature,
            &area,
            10.0,
            0.0,
            0.45,
            &mut tendency,
        );
        assert_eq!(tendency, [0.0, 0.0]);
    }

    #[test]
    fn effective_shortwave_albedo_preserves_atmospheric_and_surface_causality() {
        let ocean = effective_shortwave_albedo(0.25, 0.25, 0.07);
        let land = effective_shortwave_albedo(0.25, 0.25, 0.24);
        let snow = effective_shortwave_albedo(0.25, 0.25, 0.62);
        assert!((ocean - 0.263_125).abs() < 1.0e-12);
        assert!((land - 0.295).abs() < 1.0e-12);
        assert!((snow - 0.366_25).abs() < 1.0e-12);
        assert!(ocean < land && land < snow);
        assert_eq!(effective_shortwave_albedo(0.25, 0.0, 0.62), 0.25);
    }

    #[test]
    fn thermal_parameters_participate_in_state_identity() {
        let mut physical = ClimatePhysicalParameters::default();
        let physical_hash = physical.parameter_hash();
        physical.atmospheric_shortwave_reflectivity += 0.01;
        assert_ne!(physical.parameter_hash(), physical_hash);

        let mut model = ClimateParameters::default();
        let model_hash = model.parameter_hash();
        model.surface_albedo_shortwave_coupling += 0.01;
        assert_ne!(model.parameter_hash(), model_hash);
    }

    #[test]
    fn air_sea_exchange_conserves_combined_column_heat() {
        const WATER_DENSITY_KG_M3: f64 = 1_000.0;
        const WATER_SPECIFIC_HEAT_J_KG_K: f64 = 3_990.0;
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let physical = ClimatePhysicalParameters::default();
        let parameters = ClimateParameters::default();
        let pressure = planet.reference_surface_pressure_pa;
        let air_capacity =
            pressure / planet.surface_gravity_m_s2 * physical.atmospheric_specific_heat_j_per_kg_k;
        let ocean_capacity =
            parameters.ocean_mixed_layer_depth_m * WATER_DENSITY_KG_M3 * WATER_SPECIFIC_HEAT_J_KG_K;
        let mut air = 280.0;
        let mut sea = 300.0;
        let before = air_capacity * air + ocean_capacity * sea;
        exchange_air_sea_heat(
            &mut air, &mut sea, pressure, planet, physical, parameters, 21_600.0,
        );
        let after = air_capacity * air + ocean_capacity * sea;
        assert!((after - before).abs() / before.abs() < 1.0e-12);
        assert!(air > 280.0);
        assert!(sea < 300.0);
    }

    #[test]
    fn implicit_atmospheric_diffusion_conserves_capacity_weighted_heat() {
        let geometry = AtmosphericHeatGeometry {
            edges: vec![AtmosphericHeatEdge {
                a: 0,
                b: 1,
                geometric_conductance: 1.0,
            }],
            diagonal_geometry: vec![1.0, 1.0],
        };
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let physical = ClimatePhysicalParameters::default();
        let parameters = ClimateParameters::default();
        let pressure = [planet.reference_surface_pressure_pa; 2];
        let area = [1.0e12, 1.0e12];
        let column_capacity = pressure[0] / planet.surface_gravity_m_s2
            * physical.atmospheric_specific_heat_j_per_kg_k;
        let mut temperature = [300.0, 280.0];
        let before =
            column_capacity * area[0] * temperature[0] + column_capacity * area[1] * temperature[1];
        diffuse_atmospheric_heat(
            &geometry,
            &mut temperature,
            &pressure,
            &area,
            planet,
            physical,
            parameters,
            86_400.0,
        );
        let after =
            column_capacity * area[0] * temperature[0] + column_capacity * area[1] * temperature[1];
        assert!((after - before).abs() / before.abs() < 1.0e-10);
        assert!(temperature[0] < 300.0);
        assert!(temperature[1] > 280.0);
    }

    #[test]
    fn atmospheric_surface_ignores_submerged_relief() {
        assert_eq!(atmospheric_surface_height_m(true, -8_000.0), 0.0);
        assert_eq!(atmospheric_surface_height_m(true, -50.0), 0.0);
        assert_eq!(atmospheric_surface_height_m(false, 1_250.0), 1_250.0);
        assert_eq!(atmospheric_surface_height_m(false, -2.0), 0.0);
    }

    #[test]
    fn eccentric_orbit_uses_equal_time_kepler_geometry() {
        let e = 0.7;
        let periapsis = 0.9;
        let (_, peri_flux) = solve_orbital_forcing(periapsis, e, periapsis);
        let (_, apo_flux) = solve_orbital_forcing(periapsis + std::f64::consts::PI, e, periapsis);
        assert!((peri_flux - 1.0 / (1.0 - e).powi(2)).abs() < 1.0e-10);
        assert!((apo_flux - 1.0 / (1.0 + e).powi(2)).abs() < 1.0e-10);
        assert!(peri_flux > apo_flux);
        let (longitude, flux) = solve_orbital_forcing(2.1, 0.94, periapsis);
        assert!(longitude.is_finite());
        assert!(flux.is_finite() && flux > 0.0);
    }

    #[test]
    fn atmospheric_face_velocity_is_orientation_invariant() {
        let topology = crate::build_icosphere(1).unwrap();
        let count = topology.positions().len();
        let mut east_bases = Vec::with_capacity(count);
        let mut north_bases = Vec::with_capacity(count);
        for position in topology.positions() {
            let basis = crate::tangent_basis(*position).unwrap();
            east_bases.push(basis.east);
            north_bases.push(basis.north);
        }
        let a = 0usize;
        let b = topology.neighbors_of(a as u32)[0] as usize;
        let mut east = vec![0.0; count];
        let mut north = vec![0.0; count];
        east[a] = 7.0;
        north[a] = -2.0;
        east[b] = -4.0;
        north[b] = 3.5;
        let forward = symmetric_edge_normal_wind_m_s(
            &topology,
            a,
            b,
            &east_bases,
            &north_bases,
            &east,
            &north,
        )
        .unwrap();
        let reverse = symmetric_edge_normal_wind_m_s(
            &topology,
            b,
            a,
            &east_bases,
            &north_bases,
            &east,
            &north,
        )
        .unwrap();
        assert!((forward + reverse).abs() < 1.0e-12);
    }
}
