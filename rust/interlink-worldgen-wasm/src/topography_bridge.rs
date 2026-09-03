use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state, GeodesicTopology,
    GeologyRequest, InheritedBoundarySet, InheritedPhysicalState, LithosphereRequest,
    PlanetPhysicalParameters, TectonicsRequest, TopographyRequest, TopographyState,
    WORLDGEN_ENGINE_VERSION,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmWorldgenTopography {
    fine_topology: GeodesicTopology,
    inherited: InheritedPhysicalState,
    boundaries: InheritedBoundarySet,
    inner: TopographyState,
    parameters: PlanetPhysicalParameters,
    coarse_topology_hash: String,
    tectonic_hash: String,
    geology_hash: String,
    lithosphere_hash: String,
    plate_count: u16,
}

#[wasm_bindgen]
impl WasmWorldgenTopography {
    #[wasm_bindgen(constructor)]
    pub fn new(
        seed: String,
        coarse_level: u8,
        fine_level: u8,
        plate_count: u16,
    ) -> Result<WasmWorldgenTopography, JsValue> {
        if coarse_level > fine_level {
            return Err(JsValue::from_str(
                "coarse topology level cannot exceed fine topology level",
            ));
        }
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let coarse_topology =
            build_icosphere(coarse_level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let fine_topology =
            build_icosphere(fine_level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let tectonics = generate_tectonics(
            &coarse_topology,
            &TectonicsRequest::new(seed.as_str(), plate_count),
            parameters,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let geology = generate_crust_and_history(
            &coarse_topology,
            &tectonics,
            &GeologyRequest::new(seed.as_str()),
            parameters,
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
            parameters,
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
        let inner = generate_initial_topography(
            &fine_topology,
            &inherited,
            &boundaries,
            parameters,
            &TopographyRequest::new(seed),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

        Ok(Self {
            fine_topology,
            inherited,
            boundaries,
            inner,
            parameters,
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
        self.inner.stage.id.to_owned()
    }
    pub fn stage_version(&self) -> u32 {
        self.inner.stage.version
    }
    pub fn stage_seed_hex(&self) -> String {
        format!("{:016x}", self.inner.stage.derived_seed)
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

    pub fn topography_hash_hex(&self) -> String {
        self.inner.metrics.topography_hash_hex()
    }
    pub fn topography_parameter_hash_hex(&self) -> String {
        self.inner.metrics.parameter_hash_hex()
    }
    pub fn inheritance_hash_hex(&self) -> String {
        self.inherited.inheritance_hash_hex()
    }
    pub fn boundary_hash_hex(&self) -> String {
        self.boundaries.boundary_hash_hex()
    }
    pub fn planet_parameter_hash_hex(&self) -> String {
        self.parameters.parameter_hash_hex()
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

    pub fn minimum_solid_elevation_m(&self) -> f64 {
        self.inner.metrics.minimum_solid_elevation_m
    }
    pub fn maximum_solid_elevation_m(&self) -> f64 {
        self.inner.metrics.maximum_solid_elevation_m
    }
    pub fn mean_solid_elevation_m(&self) -> f64 {
        self.inner.metrics.mean_solid_elevation_m
    }
    pub fn p05_solid_elevation_m(&self) -> f64 {
        self.inner.metrics.p05_solid_elevation_m
    }
    pub fn median_solid_elevation_m(&self) -> f64 {
        self.inner.metrics.median_solid_elevation_m
    }
    pub fn p95_solid_elevation_m(&self) -> f64 {
        self.inner.metrics.p95_solid_elevation_m
    }
    pub fn has_sea_level(&self) -> bool {
        self.inner.metrics.sea_level_m.is_some()
    }
    pub fn sea_level_m(&self) -> f64 {
        self.inner.metrics.sea_level_m.unwrap_or(0.0)
    }
    pub fn land_area_fraction(&self) -> f64 {
        self.inner.metrics.land_area_fraction
    }
    pub fn ocean_area_fraction(&self) -> f64 {
        self.inner.metrics.ocean_area_fraction
    }
    pub fn mean_land_elevation_m(&self) -> f64 {
        self.inner.metrics.mean_land_elevation_m
    }
    pub fn mean_water_depth_m(&self) -> f64 {
        self.inner.metrics.mean_water_depth_m
    }
    pub fn maximum_water_depth_m(&self) -> f64 {
        self.inner.metrics.maximum_water_depth_m
    }
    pub fn target_water_volume_m3(&self) -> f64 {
        self.inner.metrics.target_water_volume_m3
    }
    pub fn solved_water_volume_m3(&self) -> f64 {
        self.inner.metrics.solved_water_volume_m3
    }
    pub fn water_volume_relative_error(&self) -> f64 {
        self.inner.metrics.water_volume_relative_error
    }
    pub fn clamped_sample_count(&self) -> u32 {
        self.inner.metrics.clamped_sample_count
    }

    pub fn radius_m(&self) -> f64 {
        self.parameters.radius_m
    }
    pub fn surface_gravity_m_s2(&self) -> f64 {
        self.parameters.surface_gravity_m_s2
    }
    pub fn surface_water_mass_kg(&self) -> f64 {
        self.parameters.surface_water_mass_kg
    }
    pub fn equivalent_global_water_depth_m(&self) -> f64 {
        self.parameters.equivalent_global_water_depth_m()
    }
    pub fn ocean_water_density_kg_per_m3(&self) -> f64 {
        self.parameters.ocean_water_density_kg_per_m3
    }
    pub fn isostatic_mantle_density_kg_per_m3(&self) -> f64 {
        self.parameters.isostatic_mantle_density_kg_per_m3
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
    pub fn boundary_samples(&self) -> Vec<u32> {
        self.boundaries.flattened_samples()
    }
    pub fn geological_boundary_regimes(&self) -> Vec<u8> {
        self.boundaries.geological_regimes()
    }

    pub fn isostatic_elevation_m(&self) -> Vec<f32> {
        self.inner.isostatic_elevation_m.clone()
    }
    pub fn thermal_elevation_m(&self) -> Vec<f32> {
        self.inner.thermal_elevation_m.clone()
    }
    pub fn orogenic_elevation_m(&self) -> Vec<f32> {
        self.inner.orogenic_elevation_m.clone()
    }
    pub fn ridge_elevation_m(&self) -> Vec<f32> {
        self.inner.ridge_elevation_m.clone()
    }
    pub fn rift_basin_elevation_m(&self) -> Vec<f32> {
        self.inner.rift_basin_elevation_m.clone()
    }
    pub fn trench_elevation_m(&self) -> Vec<f32> {
        self.inner.trench_elevation_m.clone()
    }
    pub fn arc_elevation_m(&self) -> Vec<f32> {
        self.inner.arc_elevation_m.clone()
    }
    pub fn mantle_dynamic_elevation_m(&self) -> Vec<f32> {
        self.inner.mantle_dynamic_elevation_m.clone()
    }
    pub fn solid_elevation_m(&self) -> Vec<f32> {
        self.inner.solid_elevation_m.clone()
    }
    pub fn elevation_above_sea_level_m(&self) -> Vec<f32> {
        self.inner.elevation_above_sea_level_m.clone()
    }
    pub fn water_depth_m(&self) -> Vec<f32> {
        self.inner.water_depth_m.clone()
    }
    pub fn submerged_mask(&self) -> Vec<u8> {
        self.inner.submerged_mask.clone()
    }
}
