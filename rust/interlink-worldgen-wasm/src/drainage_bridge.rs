use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_drainage_topology,
    generate_initial_topography, generate_lithosphere, generate_tectonics,
    inherit_boundary_interfaces, inherit_physical_state, DrainageRequest, DrainageState,
    GeodesicTopology, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters,
    TectonicsRequest, TopographyRequest, TopographyState, WORLDGEN_ENGINE_VERSION,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmWorldgenDrainage {
    topology: GeodesicTopology,
    terrain: TopographyState,
    drainage: DrainageState,
    planet: PlanetPhysicalParameters,
    coarse_level: u8,
    plate_count: u16,
}

#[wasm_bindgen]
impl WasmWorldgenDrainage {
    #[wasm_bindgen(constructor)]
    pub fn new(
        seed: String,
        coarse_level: u8,
        fine_level: u8,
        plate_count: u16,
    ) -> Result<WasmWorldgenDrainage, JsValue> {
        if coarse_level > fine_level {
            return Err(JsValue::from_str(
                "coarse topology level cannot exceed fine topology level",
            ));
        }
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let coarse =
            build_icosphere(coarse_level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let fine =
            build_icosphere(fine_level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let tectonics = generate_tectonics(
            &coarse,
            &TectonicsRequest::new(seed.as_str(), plate_count),
            planet,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let geology = generate_crust_and_history(
            &coarse,
            &tectonics,
            &GeologyRequest::new(seed.as_str()),
            planet,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let lithosphere = generate_lithosphere(
            &coarse,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let inherited = inherit_physical_state(
            &fine,
            coarse_level,
            &tectonics,
            &geology,
            &lithosphere,
            planet,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let boundaries = inherit_boundary_interfaces(
            &coarse,
            &fine,
            &tectonics,
            &geology,
            &inherited.plate_ids,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let terrain = generate_initial_topography(
            &fine,
            &inherited,
            &boundaries,
            planet,
            &TopographyRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let drainage = generate_drainage_topology(
            &fine,
            &terrain,
            planet,
            &DrainageRequest::new(seed),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

        Ok(Self {
            topology: fine,
            terrain,
            drainage,
            planet,
            coarse_level,
            plate_count,
        })
    }

    pub fn generator_version(&self) -> u32 {
        WORLDGEN_ENGINE_VERSION
    }
    pub fn stage_id(&self) -> String {
        self.drainage.stage.id.to_owned()
    }
    pub fn stage_version(&self) -> u32 {
        self.drainage.stage.version
    }
    pub fn stage_seed_hex(&self) -> String {
        format!("{:016x}", self.drainage.stage.derived_seed)
    }
    pub fn coarse_level(&self) -> u8 {
        self.coarse_level
    }
    pub fn fine_level(&self) -> u8 {
        self.topology.level()
    }
    pub fn sample_count(&self) -> u32 {
        self.drainage.metrics.sample_count
    }
    pub fn plate_count(&self) -> u16 {
        self.plate_count
    }
    pub fn drainage_hash_hex(&self) -> String {
        self.drainage.metrics.drainage_hash_hex()
    }
    pub fn topography_hash_hex(&self) -> String {
        self.terrain.metrics.topography_hash_hex()
    }
    pub fn topology_hash_hex(&self) -> String {
        self.topology.metrics().topology_hash_hex()
    }
    pub fn planet_parameter_hash_hex(&self) -> String {
        self.planet.parameter_hash_hex()
    }

    pub fn land_sample_count(&self) -> u32 {
        self.drainage.metrics.land_sample_count
    }
    pub fn ocean_sample_count(&self) -> u32 {
        self.drainage.metrics.ocean_sample_count
    }
    pub fn basin_count(&self) -> u32 {
        self.drainage.metrics.basin_count
    }
    pub fn depression_count(&self) -> u32 {
        self.drainage.metrics.depression_count
    }
    pub fn depression_sample_count(&self) -> u32 {
        self.drainage.metrics.depression_sample_count
    }
    pub fn land_area_m2(&self) -> f64 {
        self.drainage.metrics.land_area_m2
    }
    pub fn terminal_contributing_area_m2(&self) -> f64 {
        self.drainage.metrics.terminal_contributing_area_m2
    }
    pub fn area_conservation_relative_error(&self) -> f64 {
        self.drainage.metrics.area_conservation_relative_error
    }
    pub fn maximum_contributing_area_m2(&self) -> f64 {
        self.drainage.metrics.maximum_contributing_area_m2
    }
    pub fn maximum_depression_depth_m(&self) -> f64 {
        self.drainage.metrics.maximum_depression_depth_m
    }

    pub fn positions(&self) -> Vec<f64> {
        self.topology.flattened_positions()
    }
    pub fn faces(&self) -> Vec<u32> {
        self.topology.flattened_faces()
    }
    pub fn neighbor_offsets(&self) -> Vec<u32> {
        self.topology.neighbor_offsets().to_vec()
    }
    pub fn neighbors(&self) -> Vec<u32> {
        self.topology.neighbor_indices().to_vec()
    }
    pub fn solid_elevation_m(&self) -> Vec<f32> {
        self.terrain.solid_elevation_m.clone()
    }
    pub fn elevation_above_sea_level_m(&self) -> Vec<f32> {
        self.terrain.elevation_above_sea_level_m.clone()
    }
    pub fn submerged_mask(&self) -> Vec<u8> {
        self.terrain.submerged_mask.clone()
    }

    pub fn receiver(&self) -> Vec<u32> {
        self.drainage.receiver.clone()
    }
    pub fn outlet_sample(&self) -> Vec<u32> {
        self.drainage.outlet_sample.clone()
    }
    pub fn outlet_kind(&self) -> Vec<u8> {
        self.drainage.outlet_kind.clone()
    }
    pub fn basin_id(&self) -> Vec<u32> {
        self.drainage.basin_id.clone()
    }
    pub fn depression_id(&self) -> Vec<u32> {
        self.drainage.depression_id.clone()
    }
    pub fn hydrologic_escape_elevation_m(&self) -> Vec<f32> {
        self.drainage.hydrologic_escape_elevation_m.clone()
    }
    pub fn depression_depth_m(&self) -> Vec<f32> {
        self.drainage.depression_depth_m.clone()
    }
    pub fn contributing_area_m2(&self) -> Vec<f64> {
        self.drainage.contributing_area_m2.clone()
    }
    pub fn drainage_order(&self) -> Vec<u32> {
        self.drainage.drainage_order.clone()
    }

    pub fn basin_outlet_samples(&self) -> Vec<u32> {
        self.drainage
            .basins
            .iter()
            .map(|basin| basin.outlet_sample)
            .collect()
    }
    pub fn basin_outlet_kinds(&self) -> Vec<u8> {
        self.drainage
            .basins
            .iter()
            .map(|basin| basin.outlet_kind)
            .collect()
    }
    pub fn basin_areas_m2(&self) -> Vec<f64> {
        self.drainage.basins.iter().map(|basin| basin.area_m2).collect()
    }
    pub fn depression_floor_samples(&self) -> Vec<u32> {
        self.drainage
            .depressions
            .iter()
            .map(|depression| depression.floor_sample)
            .collect()
    }
    pub fn depression_floor_elevations_m(&self) -> Vec<f64> {
        self.drainage
            .depressions
            .iter()
            .map(|depression| depression.floor_elevation_m)
            .collect()
    }
    pub fn depression_spill_elevations_m(&self) -> Vec<f64> {
        self.drainage
            .depressions
            .iter()
            .map(|depression| depression.spill_elevation_m)
            .collect()
    }
    pub fn depression_areas_m2(&self) -> Vec<f64> {
        self.drainage
            .depressions
            .iter()
            .map(|depression| depression.area_m2)
            .collect()
    }
}
