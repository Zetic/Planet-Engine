use interlink_worldgen::{
    build_icosphere, generate_bounded_terrain_evolution, generate_coupled_climate_with_diagnostics,
    generate_crust_and_history, generate_drainage_topology, generate_fluvial_erosion_sediment,
    generate_initial_topography, generate_lakes_closed_basins, generate_lithosphere,
    generate_post_erosion_hydrology, generate_runoff_discharge, generate_seasonal_hydrology,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    ClimatePhysicalParameters, ClimateRequest, ClimateState, DrainageRequest,
    FluvialErosionRequest, FluvialErosionState, GeodesicTopology, GeologyRequest,
    InheritedBoundarySet, InheritedPhysicalState, LakeRequest, LithosphereRequest,
    PlanetPhysicalParameters, PostErosionHydrologyRequest, PostErosionHydrologyState,
    RunoffRequest, SeasonalHydrologyRequest, TectonicsRequest, TerrainEvolutionRequest,
    TerrainEvolutionState, TopographyRequest, TopographyState, WORLDGEN_ENGINE_VERSION,
};
use wasm_bindgen::prelude::*;

const GENERATION_STAGE_COUNT: u32 = 17;

fn report_generation_progress(
    callback: Option<&js_sys::Function>,
    stage: &str,
    stage_index: u32,
    completed: u32,
    total: u32,
) {
    let Some(callback) = callback else {
        return;
    };
    let _ = callback.call5(
        &JsValue::NULL,
        &JsValue::from_str(stage),
        &JsValue::from_f64(stage_index as f64),
        &JsValue::from_f64(GENERATION_STAGE_COUNT as f64),
        &JsValue::from_f64(completed as f64),
        &JsValue::from_f64(total.max(1) as f64),
    );
}

#[wasm_bindgen]
pub struct WasmWorldgenClimate {
    fine_topology: GeodesicTopology,
    inherited: InheritedPhysicalState,
    boundaries: InheritedBoundarySet,
    terrain: TopographyState,
    climate: ClimateState,
    erosion: FluvialErosionState,
    evolution: TerrainEvolutionState,
    reconciliation: PostErosionHydrologyState,
    planet: PlanetPhysicalParameters,
    climate_physical: ClimatePhysicalParameters,
    precipitation_phase_rate_mm_year: Vec<f32>,
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
        progress: Option<js_sys::Function>,
    ) -> Result<WasmWorldgenClimate, JsValue> {
        if coarse_level > fine_level {
            return Err(JsValue::from_str(
                "coarse topology level cannot exceed fine topology level",
            ));
        }
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let progress = progress.as_ref();
        report_generation_progress(progress, "coarse-topology", 0, 0, 1);
        let coarse_topology =
            build_icosphere(coarse_level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "coarse-topology", 0, 1, 1);
        report_generation_progress(progress, "fine-topology", 1, 0, 1);
        let fine_topology =
            build_icosphere(fine_level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "fine-topology", 1, 1, 1);
        report_generation_progress(progress, "tectonics", 2, 0, 1);
        let tectonics = generate_tectonics(
            &coarse_topology,
            &TectonicsRequest::new(seed.as_str(), plate_count),
            planet,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "tectonics", 2, 1, 1);
        report_generation_progress(progress, "geology", 3, 0, 1);
        let geology = generate_crust_and_history(
            &coarse_topology,
            &tectonics,
            &GeologyRequest::new(seed.as_str()),
            planet,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "geology", 3, 1, 1);
        report_generation_progress(progress, "lithosphere", 4, 0, 1);
        let lithosphere = generate_lithosphere(
            &coarse_topology,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "lithosphere", 4, 1, 1);
        report_generation_progress(progress, "inheritance", 5, 0, 1);
        let inherited = inherit_physical_state(
            &fine_topology,
            coarse_level,
            &tectonics,
            &geology,
            &lithosphere,
            planet,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "inheritance", 5, 1, 1);
        report_generation_progress(progress, "boundary-refinement", 6, 0, 1);
        let boundaries = inherit_boundary_interfaces(
            &coarse_topology,
            &fine_topology,
            &tectonics,
            &geology,
            &inherited.plate_ids,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "boundary-refinement", 6, 1, 1);
        report_generation_progress(progress, "topography", 7, 0, 1);
        let terrain = generate_initial_topography(
            &fine_topology,
            &inherited,
            &boundaries,
            planet,
            &TopographyRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "topography", 7, 1, 1);
        let climate_request = ClimateRequest::new(seed.as_str());
        let climate_physical = climate_request.physical;
        report_generation_progress(
            progress,
            "climate-spinup",
            8,
            0,
            climate_request.parameters.maximum_spinup_years as u32,
        );
        let mut climate_progress = |completed_years: u8, maximum_years: u8| {
            report_generation_progress(
                progress,
                "climate-spinup",
                8,
                completed_years as u32,
                maximum_years as u32,
            );
        };
        let (climate, diagnostics) = generate_coupled_climate_with_diagnostics(
            &fine_topology,
            &terrain,
            planet,
            &climate_request,
            &mut climate_progress,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

        report_generation_progress(progress, "drainage-topology", 9, 0, 1);
        let drainage = generate_drainage_topology(
            &fine_topology,
            &terrain,
            planet,
            &DrainageRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "drainage-topology", 9, 1, 1);

        report_generation_progress(progress, "runoff-discharge", 10, 0, 1);
        let runoff = generate_runoff_discharge(
            &fine_topology,
            &terrain,
            &climate,
            &drainage,
            planet,
            &RunoffRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "runoff-discharge", 10, 1, 1);

        report_generation_progress(progress, "lake-equilibrium", 11, 0, 1);
        let lakes = generate_lakes_closed_basins(
            &fine_topology,
            &terrain,
            &climate,
            &drainage,
            &runoff,
            planet,
            &LakeRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "lake-equilibrium", 11, 1, 1);

        report_generation_progress(progress, "seasonal-hydrology", 12, 0, 1);
        let seasonal = generate_seasonal_hydrology(
            &fine_topology,
            &terrain,
            &climate,
            &diagnostics,
            &drainage,
            &runoff,
            &lakes,
            planet,
            &SeasonalHydrologyRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "seasonal-hydrology", 12, 1, 1);

        report_generation_progress(progress, "fluvial-erosion-sediment", 13, 0, 1);
        let erosion = generate_fluvial_erosion_sediment(
            &fine_topology,
            &inherited,
            &terrain,
            &drainage,
            &lakes,
            &seasonal,
            planet,
            &FluvialErosionRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "fluvial-erosion-sediment", 13, 1, 1);

        report_generation_progress(progress, "bounded-terrain-evolution", 14, 0, 1);
        let evolution = generate_bounded_terrain_evolution(
            &fine_topology,
            &terrain,
            &drainage,
            &runoff,
            &lakes,
            &erosion,
            planet,
            &TerrainEvolutionRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "bounded-terrain-evolution", 14, 1, 1);

        report_generation_progress(progress, "post-erosion-hydrology", 15, 0, 1);
        let reconciliation = generate_post_erosion_hydrology(
            &fine_topology,
            &terrain,
            &climate,
            &diagnostics,
            &drainage,
            &runoff,
            &lakes,
            &seasonal,
            &evolution,
            planet,
            &PostErosionHydrologyRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "post-erosion-hydrology", 15, 1, 1);
        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;

        Ok(Self {
            fine_topology,
            inherited,
            boundaries,
            terrain,
            climate,
            erosion,
            evolution,
            reconciliation,
            planet,
            climate_physical,
            precipitation_phase_rate_mm_year,
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
    pub fn global_solver_level(&self) -> u8 {
        self.climate.metrics.global_solver_level
    }
    pub fn global_solver_sample_count(&self) -> u32 {
        self.climate.metrics.global_solver_sample_count
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
    pub fn moisture_transport_limiter_fraction(&self) -> f64 {
        self.climate.metrics.moisture_transport_limiter_fraction
    }
    pub fn maximum_moisture_transport_substeps(&self) -> u8 {
        self.climate.metrics.maximum_moisture_transport_substeps
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
    pub fn precipitation_phase_rate_mm_year(&self) -> Vec<f32> {
        self.precipitation_phase_rate_mm_year.clone()
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

    pub fn drainage_stage_id(&self) -> String {
        self.evolution.post_erosion_drainage.stage.id.to_owned()
    }
    pub fn drainage_stage_version(&self) -> u32 {
        self.evolution.post_erosion_drainage.stage.version
    }
    pub fn drainage_stage_seed_hex(&self) -> String {
        format!(
            "{:016x}",
            self.evolution.post_erosion_drainage.stage.derived_seed
        )
    }
    pub fn drainage_hash_hex(&self) -> String {
        self.evolution
            .post_erosion_drainage
            .metrics
            .drainage_hash_hex()
    }
    pub fn drainage_land_sample_count(&self) -> u32 {
        self.evolution
            .post_erosion_drainage
            .metrics
            .land_sample_count
    }
    pub fn drainage_ocean_sample_count(&self) -> u32 {
        self.evolution
            .post_erosion_drainage
            .metrics
            .ocean_sample_count
    }
    pub fn drainage_basin_count(&self) -> u32 {
        self.evolution.post_erosion_drainage.metrics.basin_count
    }
    pub fn drainage_depression_count(&self) -> u32 {
        self.evolution
            .post_erosion_drainage
            .metrics
            .depression_count
    }
    pub fn drainage_depression_sample_count(&self) -> u32 {
        self.evolution
            .post_erosion_drainage
            .metrics
            .depression_sample_count
    }
    pub fn drainage_land_area_m2(&self) -> f64 {
        self.evolution.post_erosion_drainage.metrics.land_area_m2
    }
    pub fn terminal_contributing_area_m2(&self) -> f64 {
        self.evolution
            .post_erosion_drainage
            .metrics
            .terminal_contributing_area_m2
    }
    pub fn drainage_area_conservation_relative_error(&self) -> f64 {
        self.evolution
            .post_erosion_drainage
            .metrics
            .area_conservation_relative_error
    }
    pub fn maximum_contributing_area_m2(&self) -> f64 {
        self.evolution
            .post_erosion_drainage
            .metrics
            .maximum_contributing_area_m2
    }
    pub fn maximum_depression_depth_m(&self) -> f64 {
        self.evolution
            .post_erosion_drainage
            .metrics
            .maximum_depression_depth_m
    }
    pub fn receiver(&self) -> Vec<u32> {
        self.evolution.post_erosion_drainage.receiver.clone()
    }
    pub fn outlet_sample(&self) -> Vec<u32> {
        self.evolution.post_erosion_drainage.outlet_sample.clone()
    }
    pub fn outlet_kind(&self) -> Vec<u8> {
        self.evolution.post_erosion_drainage.outlet_kind.clone()
    }
    pub fn basin_id(&self) -> Vec<u32> {
        self.evolution.post_erosion_drainage.basin_id.clone()
    }
    pub fn depression_id(&self) -> Vec<u32> {
        self.evolution.post_erosion_drainage.depression_id.clone()
    }
    pub fn hydrologic_escape_elevation_m(&self) -> Vec<f32> {
        self.evolution
            .post_erosion_drainage
            .hydrologic_escape_elevation_m
            .clone()
    }
    pub fn depression_depth_m(&self) -> Vec<f32> {
        self.evolution
            .post_erosion_drainage
            .depression_depth_m
            .clone()
    }
    pub fn contributing_area_m2(&self) -> Vec<f64> {
        self.evolution
            .post_erosion_drainage
            .contributing_area_m2
            .clone()
    }
    pub fn drainage_order(&self) -> Vec<u32> {
        self.evolution.post_erosion_drainage.drainage_order.clone()
    }
    pub fn basin_outlet_samples(&self) -> Vec<u32> {
        self.evolution
            .post_erosion_drainage
            .basins
            .iter()
            .map(|basin| basin.outlet_sample)
            .collect()
    }
    pub fn basin_outlet_kinds(&self) -> Vec<u8> {
        self.evolution
            .post_erosion_drainage
            .basins
            .iter()
            .map(|basin| basin.outlet_kind)
            .collect()
    }
    pub fn basin_areas_m2(&self) -> Vec<f64> {
        self.evolution
            .post_erosion_drainage
            .basins
            .iter()
            .map(|basin| basin.area_m2)
            .collect()
    }
    pub fn depression_floor_samples(&self) -> Vec<u32> {
        self.evolution
            .post_erosion_drainage
            .depressions
            .iter()
            .map(|depression| depression.floor_sample)
            .collect()
    }
    pub fn depression_floor_elevations_m(&self) -> Vec<f64> {
        self.evolution
            .post_erosion_drainage
            .depressions
            .iter()
            .map(|depression| depression.floor_elevation_m)
            .collect()
    }
    pub fn depression_spill_elevations_m(&self) -> Vec<f64> {
        self.evolution
            .post_erosion_drainage
            .depressions
            .iter()
            .map(|depression| depression.spill_elevation_m)
            .collect()
    }
    pub fn depression_areas_m2(&self) -> Vec<f64> {
        self.evolution
            .post_erosion_drainage
            .depressions
            .iter()
            .map(|depression| depression.area_m2)
            .collect()
    }

    pub fn runoff_stage_id(&self) -> String {
        self.reconciliation.reconciled_runoff.stage.id.to_owned()
    }
    pub fn runoff_stage_version(&self) -> u32 {
        self.reconciliation.reconciled_runoff.stage.version
    }
    pub fn runoff_stage_seed_hex(&self) -> String {
        format!(
            "{:016x}",
            self.reconciliation.reconciled_runoff.stage.derived_seed
        )
    }
    pub fn runoff_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .runoff_hash_hex()
    }
    pub fn runoff_parameter_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .runoff_parameter_hash_hex()
    }
    pub fn runoff_climate_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .climate_hash_hex()
    }
    pub fn runoff_drainage_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .drainage_hash_hex()
    }
    pub fn mean_land_runoff_precipitation_mm(&self) -> f64 {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .mean_land_precipitation_mm
    }
    pub fn mean_land_actual_evapotranspiration_mm(&self) -> f64 {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .mean_land_actual_evapotranspiration_mm
    }
    pub fn mean_land_runoff_mm(&self) -> f64 {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .mean_land_runoff_mm
    }
    pub fn maximum_land_runoff_mm(&self) -> f64 {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .maximum_land_runoff_mm
    }
    pub fn land_runoff_fraction(&self) -> f64 {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .land_runoff_fraction
    }
    pub fn total_local_runoff_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .total_local_runoff_m3_s
    }
    pub fn terminal_discharge_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .terminal_discharge_m3_s
    }
    pub fn discharge_conservation_relative_error(&self) -> f64 {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .discharge_conservation_relative_error
    }
    pub fn maximum_potential_discharge_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_runoff
            .metrics
            .maximum_potential_discharge_m3_s
    }
    pub fn actual_evapotranspiration_mm(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_runoff
            .actual_evapotranspiration_mm
            .clone()
    }
    pub fn local_runoff_mm(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_runoff
            .local_runoff_mm
            .clone()
    }
    pub fn runoff_fraction(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_runoff
            .runoff_fraction
            .clone()
    }
    pub fn local_runoff_m3_s(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_runoff
            .local_runoff_m3_s
            .clone()
    }
    pub fn potential_discharge_m3_s(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_runoff
            .potential_discharge_m3_s
            .clone()
    }

    pub fn lake_stage_id(&self) -> String {
        self.reconciliation.reconciled_lakes.stage.id.to_owned()
    }
    pub fn lake_stage_version(&self) -> u32 {
        self.reconciliation.reconciled_lakes.stage.version
    }
    pub fn lake_stage_seed_hex(&self) -> String {
        format!(
            "{:016x}",
            self.reconciliation.reconciled_lakes.stage.derived_seed
        )
    }
    pub fn lake_hash_hex(&self) -> String {
        self.reconciliation.reconciled_lakes.metrics.lake_hash_hex()
    }
    pub fn lake_parameter_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .lake_parameter_hash_hex()
    }
    pub fn lake_climate_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .climate_hash_hex()
    }
    pub fn lake_drainage_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .drainage_hash_hex()
    }
    pub fn lake_runoff_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .runoff_hash_hex()
    }
    pub fn lake_count(&self) -> u32 {
        self.reconciliation.reconciled_lakes.metrics.lake_count
    }
    pub fn endorheic_lake_count(&self) -> u32 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .endorheic_lake_count
    }
    pub fn overflowing_lake_count(&self) -> u32 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .overflowing_lake_count
    }
    pub fn terminal_storage_lake_count(&self) -> u32 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .terminal_storage_lake_count
    }
    pub fn lake_sample_count(&self) -> u32 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .lake_sample_count
    }
    pub fn total_lake_area_m2(&self) -> f64 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .total_lake_area_m2
    }
    pub fn total_lake_volume_m3(&self) -> f64 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .total_lake_volume_m3
    }
    pub fn maximum_lake_area_m2(&self) -> f64 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .maximum_lake_area_m2
    }
    pub fn maximum_lake_depth_m(&self) -> f64 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .maximum_lake_depth_m
    }
    pub fn total_lake_precipitation_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .total_lake_precipitation_m3_s
    }
    pub fn total_lake_evaporation_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .total_lake_evaporation_m3_s
    }
    pub fn terminal_realized_discharge_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .terminal_realized_discharge_m3_s
    }
    pub fn maximum_realized_discharge_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .maximum_realized_discharge_m3_s
    }
    pub fn unreleased_storage_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .unreleased_storage_m3_s
    }
    pub fn lake_water_balance_relative_error(&self) -> f64 {
        self.reconciliation
            .reconciled_lakes
            .metrics
            .water_balance_relative_error
    }
    pub fn lake_id(&self) -> Vec<u32> {
        self.reconciliation.reconciled_lakes.lake_id.clone()
    }
    pub fn lake_kind(&self) -> Vec<u8> {
        self.reconciliation.reconciled_lakes.lake_kind.clone()
    }
    pub fn lake_fraction(&self) -> Vec<f32> {
        self.reconciliation.reconciled_lakes.lake_fraction.clone()
    }
    pub fn lake_depth_m(&self) -> Vec<f32> {
        self.reconciliation.reconciled_lakes.lake_depth_m.clone()
    }
    pub fn realized_discharge_m3_s(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_lakes
            .realized_discharge_m3_s
            .clone()
    }
    pub fn lake_depression_ids(&self) -> Vec<u32> {
        self.reconciliation
            .reconciled_lakes
            .lakes
            .iter()
            .map(|lake| lake.depression_id)
            .collect()
    }
    pub fn lake_kinds(&self) -> Vec<u8> {
        self.reconciliation
            .reconciled_lakes
            .lakes
            .iter()
            .map(|lake| lake.kind)
            .collect()
    }
    pub fn lake_surface_elevations_m(&self) -> Vec<f64> {
        self.reconciliation
            .reconciled_lakes
            .lakes
            .iter()
            .map(|lake| lake.surface_elevation_m)
            .collect()
    }
    pub fn lake_areas_m2(&self) -> Vec<f64> {
        self.reconciliation
            .reconciled_lakes
            .lakes
            .iter()
            .map(|lake| lake.area_m2)
            .collect()
    }
    pub fn lake_volumes_m3(&self) -> Vec<f64> {
        self.reconciliation
            .reconciled_lakes
            .lakes
            .iter()
            .map(|lake| lake.volume_m3)
            .collect()
    }
    pub fn lake_outflows_m3_s(&self) -> Vec<f64> {
        self.reconciliation
            .reconciled_lakes
            .lakes
            .iter()
            .map(|lake| lake.outflow_m3_s)
            .collect()
    }
    pub fn lake_spill_samples(&self) -> Vec<u32> {
        self.reconciliation
            .reconciled_lakes
            .lakes
            .iter()
            .map(|lake| lake.spill_sample)
            .collect()
    }

    pub fn seasonal_stage_id(&self) -> String {
        self.reconciliation.reconciled_seasonal.stage.id.to_owned()
    }
    pub fn seasonal_stage_version(&self) -> u32 {
        self.reconciliation.reconciled_seasonal.stage.version
    }
    pub fn seasonal_stage_seed_hex(&self) -> String {
        format!(
            "{:016x}",
            self.reconciliation.reconciled_seasonal.stage.derived_seed
        )
    }
    pub fn seasonal_hydrology_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .seasonal_hydrology_hash_hex()
    }
    pub fn seasonal_parameter_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .seasonal_parameter_hash_hex()
    }
    pub fn seasonal_climate_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .climate_hash_hex()
    }
    pub fn seasonal_drainage_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .drainage_hash_hex()
    }
    pub fn seasonal_runoff_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .runoff_hash_hex()
    }
    pub fn seasonal_lake_hash_hex(&self) -> String {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .lake_hash_hex()
    }
    pub fn seasonal_dry_flow_sample_count(&self) -> u32 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .dry_flow_sample_count
    }
    pub fn seasonal_intermittent_flow_sample_count(&self) -> u32 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .intermittent_flow_sample_count
    }
    pub fn seasonal_perennial_flow_sample_count(&self) -> u32 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .perennial_flow_sample_count
    }
    pub fn maximum_phase_local_runoff_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .maximum_phase_local_runoff_m3_s
    }
    pub fn maximum_phase_potential_discharge_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .maximum_phase_potential_discharge_m3_s
    }
    pub fn maximum_phase_realized_discharge_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .maximum_phase_realized_discharge_m3_s
    }
    pub fn snowmelt_runoff_fraction(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .snowmelt_runoff_fraction
    }
    pub fn annual_mean_seasonal_local_runoff_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .annual_mean_local_runoff_m3_s
    }
    pub fn annual_local_runoff_closure_relative_error(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .annual_local_runoff_closure_relative_error
    }
    pub fn annual_mean_terminal_potential_discharge_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .annual_mean_terminal_potential_discharge_m3_s
    }
    pub fn seasonal_routing_conservation_relative_error(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .seasonal_routing_conservation_relative_error
    }
    pub fn annual_mean_terminal_seasonal_realized_discharge_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .annual_mean_terminal_realized_discharge_m3_s
    }
    pub fn annual_mean_seasonal_lake_precipitation_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .annual_mean_lake_precipitation_m3_s
    }
    pub fn annual_mean_seasonal_lake_evaporation_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .annual_mean_lake_evaporation_m3_s
    }
    pub fn annual_mean_seasonal_unreleased_terminal_storage_m3_s(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .annual_mean_unreleased_terminal_storage_m3_s
    }
    pub fn seasonal_water_balance_relative_error(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .seasonal_water_balance_relative_error
    }
    pub fn seasonal_lake_spinup_years(&self) -> u8 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .lake_spinup_years
    }
    pub fn final_lake_cycle_relative_change(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .final_lake_cycle_relative_change
    }
    pub fn final_lake_surface_cycle_change_m(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .final_lake_surface_cycle_change_m
    }
    pub fn maximum_seasonal_lake_level_range_m(&self) -> f64 {
        self.reconciliation
            .reconciled_seasonal
            .metrics
            .maximum_seasonal_lake_level_range_m
    }
    pub fn seasonal_phase_local_runoff_m3_s(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_seasonal
            .phase_local_runoff_m3_s
            .clone()
    }
    pub fn seasonal_phase_snowmelt_runoff_m3_s(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_seasonal
            .phase_snowmelt_runoff_m3_s
            .clone()
    }
    pub fn seasonal_phase_snow_storage_mm(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_seasonal
            .phase_snow_storage_mm
            .clone()
    }
    pub fn seasonal_phase_potential_discharge_m3_s(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_seasonal
            .phase_potential_discharge_m3_s
            .clone()
    }
    pub fn seasonal_phase_realized_discharge_m3_s(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_seasonal
            .phase_realized_discharge_m3_s
            .clone()
    }
    pub fn seasonal_flow_presence_fraction(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_seasonal
            .flow_presence_fraction
            .clone()
    }
    pub fn seasonal_flow_regime(&self) -> Vec<u8> {
        self.reconciliation.reconciled_seasonal.flow_regime.clone()
    }
    pub fn seasonal_phase_lake_surface_elevation_m(&self) -> Vec<f32> {
        self.reconciliation
            .reconciled_seasonal
            .phase_lake_surface_elevation_m
            .clone()
    }
    pub fn seasonal_phase_lake_area_m2(&self) -> Vec<f64> {
        self.reconciliation
            .reconciled_seasonal
            .phase_lake_area_m2
            .clone()
    }
    pub fn seasonal_phase_lake_volume_m3(&self) -> Vec<f64> {
        self.reconciliation
            .reconciled_seasonal
            .phase_lake_volume_m3
            .clone()
    }

    pub fn erosion_stage_id(&self) -> String {
        self.erosion.stage.id.to_owned()
    }
    pub fn erosion_stage_version(&self) -> u32 {
        self.erosion.stage.version
    }
    pub fn erosion_stage_seed_hex(&self) -> String {
        format!("{:016x}", self.erosion.stage.derived_seed)
    }
    pub fn fluvial_erosion_hash_hex(&self) -> String {
        self.erosion.metrics.fluvial_erosion_hash_hex()
    }
    pub fn erosion_parameter_hash_hex(&self) -> String {
        self.erosion.metrics.erosion_parameter_hash_hex()
    }
    pub fn erosion_inheritance_hash_hex(&self) -> String {
        self.erosion.metrics.inheritance_hash_hex()
    }
    pub fn erosion_topography_hash_hex(&self) -> String {
        self.erosion.metrics.topography_hash_hex()
    }
    pub fn erosion_drainage_hash_hex(&self) -> String {
        self.erosion.metrics.drainage_hash_hex()
    }
    pub fn erosion_lake_hash_hex(&self) -> String {
        self.erosion.metrics.lake_hash_hex()
    }
    pub fn erosion_seasonal_hydrology_hash_hex(&self) -> String {
        self.erosion.metrics.seasonal_hydrology_hash_hex()
    }
    pub fn erosive_sample_count(&self) -> u32 {
        self.erosion.metrics.erosive_sample_count
    }
    pub fn active_lake_trap_count(&self) -> u32 {
        self.erosion.metrics.active_lake_trap_count
    }
    pub fn maximum_effective_discharge_m3_s(&self) -> f64 {
        self.erosion.metrics.maximum_effective_discharge_m3_s
    }
    pub fn maximum_channel_slope(&self) -> f64 {
        self.erosion.metrics.maximum_channel_slope
    }
    pub fn maximum_channel_width_m(&self) -> f64 {
        self.erosion.metrics.maximum_channel_width_m
    }
    pub fn maximum_incision_potential_m_per_year(&self) -> f64 {
        self.erosion.metrics.maximum_incision_potential_m_per_year
    }
    pub fn total_sediment_generated_kg_s(&self) -> f64 {
        self.erosion.metrics.total_sediment_generated_kg_s
    }
    pub fn total_land_deposition_kg_s(&self) -> f64 {
        self.erosion.metrics.total_land_deposition_kg_s
    }
    pub fn total_lake_deposition_kg_s(&self) -> f64 {
        self.erosion.metrics.total_lake_deposition_kg_s
    }
    pub fn total_terminal_ocean_deposition_kg_s(&self) -> f64 {
        self.erosion.metrics.total_terminal_ocean_deposition_kg_s
    }
    pub fn maximum_sediment_load_kg_s(&self) -> f64 {
        self.erosion.metrics.maximum_sediment_load_kg_s
    }
    pub fn sediment_conservation_relative_error(&self) -> f64 {
        self.erosion.metrics.sediment_conservation_relative_error
    }
    pub fn effective_discharge_m3_s(&self) -> Vec<f32> {
        self.erosion.effective_discharge_m3_s.clone()
    }
    pub fn channel_slope(&self) -> Vec<f32> {
        self.erosion.channel_slope.clone()
    }
    pub fn channel_width_m(&self) -> Vec<f32> {
        self.erosion.channel_width_m.clone()
    }
    pub fn erodibility_index(&self) -> Vec<f32> {
        self.erosion.erodibility_index.clone()
    }
    pub fn stream_power_index(&self) -> Vec<f32> {
        self.erosion.stream_power_index.clone()
    }
    pub fn incision_potential_m_per_year(&self) -> Vec<f32> {
        self.erosion.incision_potential_m_per_year.clone()
    }
    pub fn local_sediment_supply_kg_s(&self) -> Vec<f32> {
        self.erosion.local_sediment_supply_kg_s.clone()
    }
    pub fn sediment_transport_capacity_kg_s(&self) -> Vec<f32> {
        self.erosion.sediment_transport_capacity_kg_s.clone()
    }
    pub fn sediment_load_kg_s(&self) -> Vec<f32> {
        self.erosion.sediment_load_kg_s.clone()
    }
    pub fn sediment_deposition_kg_s(&self) -> Vec<f32> {
        self.erosion.sediment_deposition_kg_s.clone()
    }

    pub fn reconciliation_stage_id(&self) -> String {
        self.reconciliation.stage.id.to_owned()
    }
    pub fn reconciliation_stage_version(&self) -> u32 {
        self.reconciliation.stage.version
    }
    pub fn reconciliation_stage_seed_hex(&self) -> String {
        format!("{:016x}", self.reconciliation.stage.derived_seed)
    }
    pub fn post_erosion_hydrology_hash_hex(&self) -> String {
        self.reconciliation
            .metrics
            .post_erosion_hydrology_hash_hex()
    }
    pub fn reconciliation_parameter_hash_hex(&self) -> String {
        self.reconciliation
            .metrics
            .reconciliation_parameter_hash_hex()
    }
    pub fn reconciliation_topography_hash_hex(&self) -> String {
        format!("{:016x}", self.reconciliation.metrics.topography_hash)
    }
    pub fn reconciliation_climate_hash_hex(&self) -> String {
        format!("{:016x}", self.reconciliation.metrics.climate_hash)
    }
    pub fn reconciliation_pre_erosion_drainage_hash_hex(&self) -> String {
        format!(
            "{:016x}",
            self.reconciliation.metrics.pre_erosion_drainage_hash
        )
    }
    pub fn reconciliation_pre_erosion_runoff_hash_hex(&self) -> String {
        format!(
            "{:016x}",
            self.reconciliation.metrics.pre_erosion_runoff_hash
        )
    }
    pub fn reconciliation_pre_erosion_lake_hash_hex(&self) -> String {
        format!("{:016x}", self.reconciliation.metrics.pre_erosion_lake_hash)
    }
    pub fn reconciliation_pre_erosion_seasonal_hash_hex(&self) -> String {
        format!(
            "{:016x}",
            self.reconciliation.metrics.pre_erosion_seasonal_hash
        )
    }
    pub fn reconciliation_terrain_evolution_hash_hex(&self) -> String {
        format!(
            "{:016x}",
            self.reconciliation.metrics.terrain_evolution_hash
        )
    }
    pub fn reconciliation_evolved_surface_hash_hex(&self) -> String {
        format!("{:016x}", self.reconciliation.metrics.evolved_surface_hash)
    }
    pub fn reconciliation_post_erosion_drainage_hash_hex(&self) -> String {
        format!(
            "{:016x}",
            self.reconciliation.metrics.post_erosion_drainage_hash
        )
    }
    pub fn reconciliation_reconciled_runoff_hash_hex(&self) -> String {
        self.reconciliation.metrics.reconciled_runoff_hash_hex()
    }
    pub fn reconciliation_reconciled_lake_hash_hex(&self) -> String {
        self.reconciliation.metrics.reconciled_lake_hash_hex()
    }
    pub fn reconciliation_reconciled_seasonal_hash_hex(&self) -> String {
        self.reconciliation.metrics.reconciled_seasonal_hash_hex()
    }
    pub fn pre_erosion_lake_count(&self) -> u32 {
        self.reconciliation.metrics.pre_erosion_lake_count
    }
    pub fn post_erosion_lake_count(&self) -> u32 {
        self.reconciliation.metrics.post_erosion_lake_count
    }
    pub fn lake_kind_changed_sample_count(&self) -> u32 {
        self.reconciliation.metrics.lake_kind_changed_sample_count
    }
    pub fn lake_added_sample_count(&self) -> u32 {
        self.reconciliation.metrics.lake_added_sample_count
    }
    pub fn lake_removed_sample_count(&self) -> u32 {
        self.reconciliation.metrics.lake_removed_sample_count
    }
    pub fn flow_regime_changed_sample_count(&self) -> u32 {
        self.reconciliation.metrics.flow_regime_changed_sample_count
    }
    pub fn maximum_absolute_lake_depth_change_m(&self) -> f64 {
        self.reconciliation
            .metrics
            .maximum_absolute_lake_depth_change_m
    }
    pub fn maximum_absolute_annual_realized_discharge_change_m3_s(&self) -> f64 {
        self.reconciliation
            .metrics
            .maximum_absolute_annual_realized_discharge_change_m3_s
    }
    pub fn maximum_absolute_flow_presence_change(&self) -> f64 {
        self.reconciliation
            .metrics
            .maximum_absolute_flow_presence_change
    }
    pub fn reconciled_runoff_conservation_relative_error(&self) -> f64 {
        self.reconciliation
            .metrics
            .reconciled_runoff_conservation_relative_error
    }
    pub fn reconciled_lake_water_balance_relative_error(&self) -> f64 {
        self.reconciliation
            .metrics
            .reconciled_lake_water_balance_relative_error
    }
    pub fn reconciled_seasonal_routing_relative_error(&self) -> f64 {
        self.reconciliation
            .metrics
            .reconciled_seasonal_routing_relative_error
    }
    pub fn reconciled_seasonal_water_balance_relative_error(&self) -> f64 {
        self.reconciliation
            .metrics
            .reconciled_seasonal_water_balance_relative_error
    }
    pub fn reconciliation_lake_kind_changed_mask(&self) -> Vec<u8> {
        self.reconciliation.lake_kind_changed_mask.clone()
    }
    pub fn reconciliation_lake_depth_delta_m(&self) -> Vec<f32> {
        self.reconciliation.lake_depth_delta_m.clone()
    }
    pub fn reconciliation_annual_realized_discharge_delta_m3_s(&self) -> Vec<f32> {
        self.reconciliation
            .annual_realized_discharge_delta_m3_s
            .clone()
    }
    pub fn reconciliation_flow_regime_changed_mask(&self) -> Vec<u8> {
        self.reconciliation.flow_regime_changed_mask.clone()
    }
    pub fn reconciliation_flow_presence_delta(&self) -> Vec<f32> {
        self.reconciliation.flow_presence_delta.clone()
    }

    pub fn evolution_stage_id(&self) -> String {
        self.evolution.stage.id.to_owned()
    }
    pub fn evolution_stage_version(&self) -> u32 {
        self.evolution.stage.version
    }
    pub fn evolution_stage_seed_hex(&self) -> String {
        format!("{:016x}", self.evolution.stage.derived_seed)
    }
    pub fn terrain_evolution_hash_hex(&self) -> String {
        self.evolution.metrics.terrain_evolution_hash_hex()
    }
    pub fn evolution_parameter_hash_hex(&self) -> String {
        self.evolution.metrics.evolution_parameter_hash_hex()
    }
    pub fn evolved_surface_hash_hex(&self) -> String {
        self.evolution.metrics.evolved_surface_hash_hex()
    }
    pub fn post_erosion_drainage_hash_hex(&self) -> String {
        self.evolution.metrics.post_erosion_drainage_hash_hex()
    }
    pub fn evolution_topography_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution.metrics.topography_hash)
    }
    pub fn evolution_drainage_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution.metrics.drainage_hash)
    }
    pub fn evolution_runoff_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution.metrics.runoff_hash)
    }
    pub fn evolution_lake_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution.metrics.lake_hash)
    }
    pub fn evolution_fluvial_erosion_hash_hex(&self) -> String {
        format!("{:016x}", self.evolution.metrics.fluvial_erosion_hash)
    }
    pub fn geomorphic_duration_years(&self) -> f64 {
        self.evolution.metrics.geomorphic_duration_years
    }
    pub fn evolved_eroded_sample_count(&self) -> u32 {
        self.evolution.metrics.eroded_sample_count
    }
    pub fn evolved_depositional_sample_count(&self) -> u32 {
        self.evolution.metrics.depositional_sample_count
    }
    pub fn receiver_changed_sample_count(&self) -> u32 {
        self.evolution.metrics.receiver_changed_sample_count
    }
    pub fn receiver_changed_fraction(&self) -> f64 {
        self.evolution.metrics.receiver_changed_fraction
    }
    pub fn maximum_applied_erosion_m(&self) -> f64 {
        self.evolution.metrics.maximum_applied_erosion_m
    }
    pub fn maximum_applied_deposition_m(&self) -> f64 {
        self.evolution.metrics.maximum_applied_deposition_m
    }
    pub fn maximum_absolute_terrain_change_m(&self) -> f64 {
        self.evolution.metrics.maximum_absolute_terrain_change_m
    }
    pub fn mean_land_absolute_terrain_change_m(&self) -> f64 {
        self.evolution.metrics.mean_land_absolute_terrain_change_m
    }
    pub fn total_applied_sediment_generated_kg_s(&self) -> f64 {
        self.evolution.metrics.total_applied_sediment_generated_kg_s
    }
    pub fn evolution_total_land_deposition_kg_s(&self) -> f64 {
        self.evolution.metrics.total_land_deposition_kg_s
    }
    pub fn total_lake_sink_kg_s(&self) -> f64 {
        self.evolution.metrics.total_lake_sink_kg_s
    }
    pub fn total_terminal_ocean_sink_kg_s(&self) -> f64 {
        self.evolution.metrics.total_terminal_ocean_sink_kg_s
    }
    pub fn evolution_sediment_conservation_relative_error(&self) -> f64 {
        self.evolution.metrics.sediment_conservation_relative_error
    }
    pub fn maximum_post_erosion_potential_discharge_m3_s(&self) -> f64 {
        self.evolution
            .metrics
            .maximum_post_erosion_potential_discharge_m3_s
    }
    pub fn post_erosion_runoff_conservation_relative_error(&self) -> f64 {
        self.evolution
            .metrics
            .post_erosion_runoff_conservation_relative_error
    }
    pub fn evolved_solid_elevation_m(&self) -> Vec<f32> {
        self.evolution.evolved_solid_elevation_m.clone()
    }
    pub fn terrain_delta_m(&self) -> Vec<f32> {
        self.evolution.terrain_delta_m.clone()
    }
    pub fn applied_erosion_m(&self) -> Vec<f32> {
        self.evolution.applied_erosion_m.clone()
    }
    pub fn applied_deposition_m(&self) -> Vec<f32> {
        self.evolution.applied_deposition_m.clone()
    }
    pub fn receiver_changed_mask(&self) -> Vec<u8> {
        self.evolution.receiver_changed_mask.clone()
    }
    pub fn post_erosion_contributing_area_m2(&self) -> Vec<f64> {
        self.evolution
            .post_erosion_drainage
            .contributing_area_m2
            .clone()
    }
    pub fn post_erosion_potential_discharge_m3_s(&self) -> Vec<f32> {
        self.evolution.post_erosion_potential_discharge_m3_s.clone()
    }
}
