use interlink_worldgen::{
    build_icosphere, generate_coupled_climate, generate_crust_and_history,
    generate_initial_topography, generate_lithosphere, generate_tectonics,
    inherit_boundary_interfaces, inherit_physical_state, ClimatePhysicalParameters, ClimateRequest,
    ClimateState, GeodesicTopology, GeologyRequest, InheritedBoundarySet, InheritedPhysicalState,
    LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,
    TopographyState, WORLDGEN_ENGINE_VERSION,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmWorldgenClimate {
    fine_topology: GeodesicTopology,
    inherited: InheritedPhysicalState,
    boundaries: InheritedBoundarySet,
    terrain: TopographyState,
    climate: ClimateState,
    planet: PlanetPhysicalParameters,
    climate_physical: ClimatePhysicalParameters,
    coarse_topology_hash: String,
    tectonic_hash: String,
    geology_hash: String,
    lithosphere_hash: String,
    plate_count: u16,
}

#[wasm_bindgen]
impl WasmWorldgenClimate {
    #[wasm_bindgen(constructor)]
    pub fn new(
        seed: String,
        coarse_level: u8,
        fine_level: u8,
        plate_count: u16,
    ) -> Result<WasmWorldgenClimate, JsValue> {
        if coarse_level > fine_level {
            return Err(JsValue::from_str(
                "coarse topology level cannot exceed fine topology level",
            ));
        }
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let coarse_topology =
            build_icosphere(coarse_level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let fine_topology =
            build_icosphere(fine_level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let tectonics = generate_tectonics(
            &coarse_topology,
            &TectonicsRequest::new(seed.as_str(), plate_count),
            planet,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let geology = generate_crust_and_history(
            &coarse_topology,
            &tectonics,
            &GeologyRequest::new(seed.as_str()),
            planet,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let lithosphere = generate_lithosphere(
            &coarse_topology,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let inherited = inherit_physical_state(
            &fine_topology,
            coarse_level,
            &tectonics,
            &geology,
            &lithosphere,
            planet,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let boundaries = inherit_boundary_interfaces(
            &coarse_topology,
            &fine_topology,
            &tectonics,
            &geology,
            &inherited.plate_ids,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let terrain = generate_initial_topography(
            &fine_topology,
            &inherited,
            &boundaries,
            planet,
            &TopographyRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let climate_request = ClimateRequest::new(seed);
        let climate_physical = climate_request.physical;
        let climate = generate_coupled_climate(&fine_topology, &terrain, planet, &climate_request)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

        Ok(Self {
            fine_topology,
            inherited,
            boundaries,
            terrain,
            climate,
            planet,
            climate_physical,
            coarse_topology_hash: coarse_topology.metrics().topology_hash_hex(),
            tectonic_hash: tectonics.metrics.tectonic_hash_hex(),
            geology_hash: geology.metrics.geology_hash_hex(),
            lithosphere_hash: lithosphere.metrics.lithosphere_hash_hex(),
            plate_count,
        })
    }

    pub fn generator_version(&self) -> u32 {
        WORLDGEN_ENGINE_VERSION
    }
    pub fn stage_id(&self) -> String {
        self.climate.stage.id.to_owned()
    }
    pub fn stage_version(&self) -> u32 {
        self.climate.stage.version
    }
    pub fn stage_seed_hex(&self) -> String {
        format!("{:016x}", self.climate.stage.derived_seed)
    }
    pub fn coarse_level(&self) -> u8 {
        self.inherited.map.metrics.coarse_level
    }
    pub fn fine_level(&self) -> u8 {
        self.inherited.map.metrics.fine_level
    }
    pub fn coarse_sample_count(&self) -> u32 {
        self.inherited.map.metrics.coarse_sample_count
    }
    pub fn fine_sample_count(&self) -> u32 {
        self.inherited.map.metrics.fine_sample_count
    }
    pub fn plate_count(&self) -> u16 {
        self.plate_count
    }
    pub fn fine_boundary_edge_count(&self) -> u32 {
        self.boundaries.boundaries.len() as u32
    }

    pub fn climate_hash_hex(&self) -> String {
        self.climate.metrics.climate_hash_hex()
    }
    pub fn climate_physical_parameter_hash_hex(&self) -> String {
        self.climate.metrics.climate_physical_parameter_hash_hex()
    }
    pub fn climate_model_parameter_hash_hex(&self) -> String {
        self.climate.metrics.climate_model_parameter_hash_hex()
    }
    pub fn topography_hash_hex(&self) -> String {
        self.terrain.metrics.topography_hash_hex()
    }
    pub fn inheritance_hash_hex(&self) -> String {
        self.inherited.inheritance_hash_hex()
    }
    pub fn boundary_hash_hex(&self) -> String {
        self.boundaries.boundary_hash_hex()
    }
    pub fn planet_parameter_hash_hex(&self) -> String {
        self.planet.parameter_hash_hex()
    }
    pub fn coarse_topology_hash_hex(&self) -> String {
        self.coarse_topology_hash.clone()
    }
    pub fn fine_topology_hash_hex(&self) -> String {
        self.fine_topology.metrics().topology_hash_hex()
    }
    pub fn tectonic_hash_hex(&self) -> String {
        self.tectonic_hash.clone()
    }
    pub fn geology_hash_hex(&self) -> String {
        self.geology_hash.clone()
    }
    pub fn lithosphere_hash_hex(&self) -> String {
        self.lithosphere_hash.clone()
    }

    pub fn orbital_phase_count(&self) -> u8 {
        self.climate.metrics.orbital_phase_count
    }
    pub fn spinup_years(&self) -> u8 {
        self.climate.metrics.spinup_years
    }
    pub fn mean_temperature_k(&self) -> f64 {
        self.climate.metrics.mean_temperature_k
    }
    pub fn minimum_temperature_k(&self) -> f64 {
        self.climate.metrics.minimum_temperature_k
    }
    pub fn maximum_temperature_k(&self) -> f64 {
        self.climate.metrics.maximum_temperature_k
    }
    pub fn mean_land_temperature_k(&self) -> f64 {
        self.climate.metrics.mean_land_temperature_k
    }
    pub fn mean_ocean_temperature_k(&self) -> f64 {
        self.climate.metrics.mean_ocean_temperature_k
    }
    pub fn mean_wind_speed_m_s(&self) -> f64 {
        self.climate.metrics.mean_wind_speed_m_s
    }
    pub fn maximum_wind_speed_m_s(&self) -> f64 {
        self.climate.metrics.maximum_wind_speed_m_s
    }
    pub fn mean_surface_current_m_s(&self) -> f64 {
        self.climate.metrics.mean_surface_current_m_s
    }
    pub fn maximum_surface_current_m_s(&self) -> f64 {
        self.climate.metrics.maximum_surface_current_m_s
    }
    pub fn ocean_divergence_residual_m_s(&self) -> f64 {
        self.climate.metrics.ocean_divergence_residual_m_s
    }
    pub fn mean_sea_surface_temperature_k(&self) -> f64 {
        self.climate.metrics.mean_sea_surface_temperature_k
    }
    pub fn mean_annual_precipitation_mm(&self) -> f64 {
        self.climate.metrics.mean_annual_precipitation_mm
    }
    pub fn p95_annual_precipitation_mm(&self) -> f64 {
        self.climate.metrics.p95_annual_precipitation_mm
    }
    pub fn global_evaporation_kg(&self) -> f64 {
        self.climate.metrics.global_evaporation_kg
    }
    pub fn global_precipitation_kg(&self) -> f64 {
        self.climate.metrics.global_precipitation_kg
    }
    pub fn moisture_budget_relative_error(&self) -> f64 {
        self.climate.metrics.moisture_budget_relative_error
    }
    pub fn persistent_snow_area_fraction(&self) -> f64 {
        self.climate.metrics.persistent_snow_area_fraction
    }
    pub fn sea_ice_area_fraction(&self) -> f64 {
        self.climate.metrics.sea_ice_area_fraction
    }
    pub fn final_temperature_rms_change_k(&self) -> f64 {
        self.climate.metrics.final_temperature_rms_change_k
    }

    pub fn sea_level_m(&self) -> f64 {
        self.terrain.metrics.sea_level_m.unwrap_or(0.0)
    }
    pub fn has_sea_level(&self) -> bool {
        self.terrain.metrics.sea_level_m.is_some()
    }
    pub fn land_area_fraction(&self) -> f64 {
        self.terrain.metrics.land_area_fraction
    }
    pub fn ocean_area_fraction(&self) -> f64 {
        self.terrain.metrics.ocean_area_fraction
    }
    pub fn minimum_solid_elevation_m(&self) -> f64 {
        self.terrain.metrics.minimum_solid_elevation_m
    }
    pub fn maximum_solid_elevation_m(&self) -> f64 {
        self.terrain.metrics.maximum_solid_elevation_m
    }

    pub fn radius_m(&self) -> f64 {
        self.planet.radius_m
    }
    pub fn surface_gravity_m_s2(&self) -> f64 {
        self.planet.surface_gravity_m_s2
    }
    pub fn rotation_period_s(&self) -> f64 {
        self.planet.rotation_period_s
    }
    pub fn axial_tilt_rad(&self) -> f64 {
        self.planet.axial_tilt_rad
    }
    pub fn orbital_period_s(&self) -> f64 {
        self.planet.orbital_period_s
    }
    pub fn stellar_flux_w_m2(&self) -> f64 {
        self.planet.stellar_flux_w_m2
    }
    pub fn reference_surface_pressure_pa(&self) -> f64 {
        self.planet.reference_surface_pressure_pa
    }
    pub fn surface_water_mass_kg(&self) -> f64 {
        self.planet.surface_water_mass_kg
    }
    pub fn equivalent_global_water_depth_m(&self) -> f64 {
        self.planet.equivalent_global_water_depth_m()
    }
    pub fn internal_heat_flux_w_per_m2(&self) -> f64 {
        self.planet.internal_heat_flux_w_per_m2
    }

    pub fn orbital_eccentricity(&self) -> f64 {
        self.climate_physical.orbital_eccentricity
    }
    pub fn longitude_of_periapsis_rad(&self) -> f64 {
        self.climate_physical.longitude_of_periapsis_rad
    }
    pub fn atmospheric_mean_molar_mass_kg_per_mol(&self) -> f64 {
        self.climate_physical.atmospheric_mean_molar_mass_kg_per_mol
    }
    pub fn atmospheric_specific_heat_j_per_kg_k(&self) -> f64 {
        self.climate_physical.atmospheric_specific_heat_j_per_kg_k
    }
    pub fn atmospheric_shortwave_reflectivity(&self) -> f64 {
        self.climate_physical.atmospheric_shortwave_reflectivity
    }
    pub fn atmospheric_longwave_optical_depth(&self) -> f64 {
        self.climate_physical.atmospheric_longwave_optical_depth
    }

    pub fn positions(&self) -> Vec<f64> {
        self.fine_topology.flattened_positions()
    }
    pub fn faces(&self) -> Vec<u32> {
        self.fine_topology.flattened_faces()
    }
    pub fn neighbor_offsets(&self) -> Vec<u32> {
        self.fine_topology.neighbor_offsets().to_vec()
    }
    pub fn neighbors(&self) -> Vec<u32> {
        self.fine_topology.neighbor_indices().to_vec()
    }

    pub fn plate_ids(&self) -> Vec<u16> {
        self.inherited.plate_ids.clone()
    }
    pub fn crust_kind(&self) -> Vec<u8> {
        self.inherited.crust_kind.clone()
    }
    pub fn nearest_coarse_source(&self) -> Vec<u32> {
        self.inherited.map.nearest_coarse_source.clone()
    }
    pub fn inherited_sample_mask(&self) -> Vec<u8> {
        self.inherited.map.inherited_sample_mask.clone()
    }
    pub fn crust_age_myr(&self) -> Vec<f32> {
        self.inherited.crust_age_myr.clone()
    }
    pub fn crust_thickness_km(&self) -> Vec<f32> {
        self.inherited.crust_thickness_km.clone()
    }
    pub fn orogenic_history(&self) -> Vec<f32> {
        self.inherited.orogenic_history.clone()
    }
    pub fn ridge_history(&self) -> Vec<f32> {
        self.inherited.ridge_history.clone()
    }
    pub fn trench_history(&self) -> Vec<f32> {
        self.inherited.trench_history.clone()
    }
    pub fn strength_index(&self) -> Vec<f32> {
        self.inherited.strength_index.clone()
    }
    pub fn weakness_index(&self) -> Vec<f32> {
        self.inherited.weakness_index.clone()
    }
    pub fn mantle_dynamic_support_index(&self) -> Vec<f32> {
        self.inherited.mantle_dynamic_support_index.clone()
    }
    pub fn structural_zone_kind(&self) -> Vec<u8> {
        self.inherited.structural_zone_kind.clone()
    }
    pub fn fragmentation_propensity(&self) -> Vec<f32> {
        self.inherited.fragmentation_propensity.clone()
    }
    pub fn kinematic_domain_ids(&self) -> Vec<u16> {
        self.inherited.kinematic_domain_ids.clone()
    }
    pub fn boundary_samples(&self) -> Vec<u32> {
        self.boundaries.flattened_samples()
    }
    pub fn boundary_kinds(&self) -> Vec<u8> {
        self.boundaries.tectonic_kinds()
    }
    pub fn geological_boundary_regimes(&self) -> Vec<u8> {
        self.boundaries.geological_regimes()
    }
    pub fn boundary_coarse_source_indices(&self) -> Vec<u32> {
        self.boundaries.coarse_boundary_indices()
    }

    pub fn isostatic_elevation_m(&self) -> Vec<f32> {
        self.terrain.isostatic_elevation_m.clone()
    }
    pub fn thermal_elevation_m(&self) -> Vec<f32> {
        self.terrain.thermal_elevation_m.clone()
    }
    pub fn orogenic_elevation_m(&self) -> Vec<f32> {
        self.terrain.orogenic_elevation_m.clone()
    }
    pub fn ridge_elevation_m(&self) -> Vec<f32> {
        self.terrain.ridge_elevation_m.clone()
    }
    pub fn rift_basin_elevation_m(&self) -> Vec<f32> {
        self.terrain.rift_basin_elevation_m.clone()
    }
    pub fn trench_elevation_m(&self) -> Vec<f32> {
        self.terrain.trench_elevation_m.clone()
    }
    pub fn arc_elevation_m(&self) -> Vec<f32> {
        self.terrain.arc_elevation_m.clone()
    }
    pub fn mantle_dynamic_elevation_m(&self) -> Vec<f32> {
        self.terrain.mantle_dynamic_elevation_m.clone()
    }
    pub fn solid_elevation_m(&self) -> Vec<f32> {
        self.terrain.solid_elevation_m.clone()
    }
    pub fn elevation_above_sea_level_m(&self) -> Vec<f32> {
        self.terrain.elevation_above_sea_level_m.clone()
    }
    pub fn water_depth_m(&self) -> Vec<f32> {
        self.terrain.water_depth_m.clone()
    }
    pub fn submerged_mask(&self) -> Vec<u8> {
        self.terrain.submerged_mask.clone()
    }

    pub fn annual_mean_insolation_w_m2(&self) -> Vec<f32> {
        self.climate.annual_mean_insolation_w_m2.clone()
    }
    pub fn seasonal_insolation_amplitude_w_m2(&self) -> Vec<f32> {
        self.climate.seasonal_insolation_amplitude_w_m2.clone()
    }
    pub fn temperature_mean_k(&self) -> Vec<f32> {
        self.climate.temperature_mean_k.clone()
    }
    pub fn temperature_annual_cos_k(&self) -> Vec<f32> {
        self.climate.temperature_annual_cos_k.clone()
    }
    pub fn temperature_annual_sin_k(&self) -> Vec<f32> {
        self.climate.temperature_annual_sin_k.clone()
    }
    pub fn temperature_min_k(&self) -> Vec<f32> {
        self.climate.temperature_min_k.clone()
    }
    pub fn temperature_max_k(&self) -> Vec<f32> {
        self.climate.temperature_max_k.clone()
    }
    pub fn local_pressure_pa(&self) -> Vec<f32> {
        self.climate.local_pressure_pa.clone()
    }
    pub fn wind_east_mean_m_s(&self) -> Vec<f32> {
        self.climate.wind_east_mean_m_s.clone()
    }
    pub fn wind_north_mean_m_s(&self) -> Vec<f32> {
        self.climate.wind_north_mean_m_s.clone()
    }
    pub fn wind_east_annual_cos_m_s(&self) -> Vec<f32> {
        self.climate.wind_east_annual_cos_m_s.clone()
    }
    pub fn wind_east_annual_sin_m_s(&self) -> Vec<f32> {
        self.climate.wind_east_annual_sin_m_s.clone()
    }
    pub fn wind_north_annual_cos_m_s(&self) -> Vec<f32> {
        self.climate.wind_north_annual_cos_m_s.clone()
    }
    pub fn wind_north_annual_sin_m_s(&self) -> Vec<f32> {
        self.climate.wind_north_annual_sin_m_s.clone()
    }
    pub fn sea_surface_temperature_mean_k(&self) -> Vec<f32> {
        self.climate.sea_surface_temperature_mean_k.clone()
    }
    pub fn sea_surface_temperature_annual_cos_k(&self) -> Vec<f32> {
        self.climate.sea_surface_temperature_annual_cos_k.clone()
    }
    pub fn sea_surface_temperature_annual_sin_k(&self) -> Vec<f32> {
        self.climate.sea_surface_temperature_annual_sin_k.clone()
    }
    pub fn current_east_mean_m_s(&self) -> Vec<f32> {
        self.climate.current_east_mean_m_s.clone()
    }
    pub fn current_north_mean_m_s(&self) -> Vec<f32> {
        self.climate.current_north_mean_m_s.clone()
    }
    pub fn current_east_annual_cos_m_s(&self) -> Vec<f32> {
        self.climate.current_east_annual_cos_m_s.clone()
    }
    pub fn current_east_annual_sin_m_s(&self) -> Vec<f32> {
        self.climate.current_east_annual_sin_m_s.clone()
    }
    pub fn current_north_annual_cos_m_s(&self) -> Vec<f32> {
        self.climate.current_north_annual_cos_m_s.clone()
    }
    pub fn current_north_annual_sin_m_s(&self) -> Vec<f32> {
        self.climate.current_north_annual_sin_m_s.clone()
    }
    pub fn current_speed_mean_m_s(&self) -> Vec<f32> {
        self.climate.current_speed_mean_m_s.clone()
    }
    pub fn ocean_heat_transport_index(&self) -> Vec<f32> {
        self.climate.ocean_heat_transport_index.clone()
    }
    pub fn specific_humidity_mean(&self) -> Vec<f32> {
        self.climate.specific_humidity_mean.clone()
    }
    pub fn annual_precipitation_mm(&self) -> Vec<f32> {
        self.climate.annual_precipitation_mm.clone()
    }
    pub fn precipitation_seasonality(&self) -> Vec<f32> {
        self.climate.precipitation_seasonality.clone()
    }
    pub fn potential_evaporation_mm(&self) -> Vec<f32> {
        self.climate.potential_evaporation_mm.clone()
    }
    pub fn moisture_balance_mm(&self) -> Vec<f32> {
        self.climate.moisture_balance_mm.clone()
    }
    pub fn aridity_index(&self) -> Vec<f32> {
        self.climate.aridity_index.clone()
    }
    pub fn snowfall_fraction(&self) -> Vec<f32> {
        self.climate.snowfall_fraction.clone()
    }
    pub fn persistent_snow_potential(&self) -> Vec<f32> {
        self.climate.persistent_snow_potential.clone()
    }
    pub fn sea_ice_potential(&self) -> Vec<f32> {
        self.climate.sea_ice_potential.clone()
    }
}
