use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_lithosphere, generate_tectonics,
    inherit_boundary_interfaces, inherit_physical_state, GeodesicTopology, GeologyRequest,
    InheritedBoundarySet, InheritedPhysicalState, LithosphereRequest, PlanetPhysicalParameters,
    TectonicsRequest, MULTIRES_STAGE_ID, MULTIRES_STAGE_VERSION, WORLDGEN_ENGINE_VERSION,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmWorldgenInheritance {
    fine_topology: GeodesicTopology,
    inner: InheritedPhysicalState,
    boundaries: InheritedBoundarySet,
    parameters: PlanetPhysicalParameters,
    coarse_topology_hash: String,
    tectonic_hash: String,
    geology_hash: String,
    lithosphere_hash: String,
    plate_count: u16,
}

#[wasm_bindgen]
impl WasmWorldgenInheritance {
    #[wasm_bindgen(constructor)]
    pub fn new(
        seed: String,
        coarse_level: u8,
        fine_level: u8,
        plate_count: u16,
    ) -> Result<WasmWorldgenInheritance, JsValue> {
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
            &LithosphereRequest::new(seed),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let inner = inherit_physical_state(
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
            &inner.plate_ids,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

        Ok(Self {
            fine_topology,
            coarse_topology_hash: coarse_topology.metrics().topology_hash_hex(),
            tectonic_hash: tectonics.metrics.tectonic_hash_hex(),
            geology_hash: geology.metrics.geology_hash_hex(),
            lithosphere_hash: lithosphere.metrics.lithosphere_hash_hex(),
            plate_count,
            inner,
            boundaries,
            parameters,
        })
    }

    pub fn generator_version(&self) -> u32 { WORLDGEN_ENGINE_VERSION }
    pub fn stage_id(&self) -> String { MULTIRES_STAGE_ID.to_owned() }
    pub fn stage_version(&self) -> u32 { MULTIRES_STAGE_VERSION }
    pub fn coarse_level(&self) -> u8 { self.inner.map.metrics.coarse_level }
    pub fn fine_level(&self) -> u8 { self.inner.map.metrics.fine_level }
    pub fn coarse_sample_count(&self) -> u32 { self.inner.map.metrics.coarse_sample_count }
    pub fn fine_sample_count(&self) -> u32 { self.inner.map.metrics.fine_sample_count }
    pub fn added_sample_count(&self) -> u32 { self.inner.map.metrics.added_sample_count }
    pub fn plate_count(&self) -> u16 { self.plate_count }
    pub fn fine_boundary_edge_count(&self) -> u32 { self.boundaries.boundaries.len() as u32 }

    pub fn provenance_hash_hex(&self) -> String { self.inner.map.metrics.provenance_hash_hex() }
    pub fn parameter_hash_hex(&self) -> String { self.inner.parameter_hash_hex() }
    pub fn inheritance_hash_hex(&self) -> String { self.inner.inheritance_hash_hex() }
    pub fn boundary_hash_hex(&self) -> String { self.boundaries.boundary_hash_hex() }
    pub fn coarse_topology_hash_hex(&self) -> String { self.coarse_topology_hash.clone() }
    pub fn fine_topology_hash_hex(&self) -> String { self.fine_topology.metrics().topology_hash_hex() }
    pub fn tectonic_hash_hex(&self) -> String { self.tectonic_hash.clone() }
    pub fn geology_hash_hex(&self) -> String { self.geology_hash.clone() }
    pub fn lithosphere_hash_hex(&self) -> String { self.lithosphere_hash.clone() }

    pub fn radius_m(&self) -> f64 { self.parameters.radius_m }
    pub fn surface_gravity_m_s2(&self) -> f64 { self.parameters.surface_gravity_m_s2 }
    pub fn surface_water_mass_kg(&self) -> f64 { self.parameters.surface_water_mass_kg }
    pub fn equivalent_global_water_depth_m(&self) -> f64 { self.parameters.equivalent_global_water_depth_m() }
    pub fn ocean_water_density_kg_per_m3(&self) -> f64 { self.parameters.ocean_water_density_kg_per_m3 }
    pub fn isostatic_mantle_density_kg_per_m3(&self) -> f64 { self.parameters.isostatic_mantle_density_kg_per_m3 }
    pub fn internal_heat_flux_w_per_m2(&self) -> f64 { self.parameters.internal_heat_flux_w_per_m2 }
    pub fn mantle_thermal_expansivity_per_k(&self) -> f64 { self.parameters.mantle_thermal_expansivity_per_k }

    pub fn positions(&self) -> Vec<f64> { self.fine_topology.flattened_positions() }
    pub fn faces(&self) -> Vec<u32> { self.fine_topology.flattened_faces() }
    pub fn neighbor_offsets(&self) -> Vec<u32> { self.fine_topology.neighbor_offsets().to_vec() }
    pub fn neighbors(&self) -> Vec<u32> { self.fine_topology.neighbor_indices().to_vec() }
    pub fn nearest_coarse_source(&self) -> Vec<u32> { self.inner.map.nearest_coarse_source.clone() }
    pub fn inherited_sample_mask(&self) -> Vec<u8> { self.inner.map.inherited_sample_mask.clone() }

    pub fn plate_ids(&self) -> Vec<u16> { self.inner.plate_ids.clone() }
    pub fn crust_kind(&self) -> Vec<u8> { self.inner.crust_kind.clone() }
    pub fn crust_province_id(&self) -> Vec<u16> { self.inner.crust_province_id.clone() }
    pub fn crust_age_myr(&self) -> Vec<f32> { self.inner.crust_age_myr.clone() }
    pub fn crust_thickness_km(&self) -> Vec<f32> { self.inner.crust_thickness_km.clone() }
    pub fn crust_density_kg_per_m3(&self) -> Vec<f32> { self.inner.crust_density_kg_per_m3.clone() }
    pub fn buoyancy_index(&self) -> Vec<f32> { self.inner.buoyancy_index.clone() }
    pub fn orogenic_history(&self) -> Vec<f32> { self.inner.orogenic_history.clone() }
    pub fn rift_history(&self) -> Vec<f32> { self.inner.rift_history.clone() }
    pub fn ridge_history(&self) -> Vec<f32> { self.inner.ridge_history.clone() }
    pub fn subduction_history(&self) -> Vec<f32> { self.inner.subduction_history.clone() }
    pub fn trench_history(&self) -> Vec<f32> { self.inner.trench_history.clone() }
    pub fn volcanic_arc_history(&self) -> Vec<f32> { self.inner.volcanic_arc_history.clone() }
    pub fn transform_history(&self) -> Vec<f32> { self.inner.transform_history.clone() }
    pub fn subsidence_history(&self) -> Vec<f32> { self.inner.subsidence_history.clone() }
    pub fn basin_potential(&self) -> Vec<f32> { self.inner.basin_potential.clone() }
    pub fn crustal_strain(&self) -> Vec<f32> { self.inner.crustal_strain.clone() }

    pub fn strength_index(&self) -> Vec<f32> { self.inner.strength_index.clone() }
    pub fn weakness_index(&self) -> Vec<f32> { self.inner.weakness_index.clone() }
    pub fn effective_elastic_thickness_km(&self) -> Vec<f32> { self.inner.effective_elastic_thickness_km.clone() }
    pub fn thermal_anomaly_index(&self) -> Vec<f32> { self.inner.thermal_anomaly_index.clone() }
    pub fn mantle_upwelling_index(&self) -> Vec<f32> { self.inner.mantle_upwelling_index.clone() }
    pub fn mantle_dynamic_support_index(&self) -> Vec<f32> { self.inner.mantle_dynamic_support_index.clone() }
    pub fn compensated_buoyancy_index(&self) -> Vec<f32> { self.inner.compensated_buoyancy_index.clone() }
    pub fn structural_fabric_strength(&self) -> Vec<f32> { self.inner.structural_fabric_strength.clone() }
    pub fn structural_zone_kind(&self) -> Vec<u8> { self.inner.structural_zone_kind.clone() }
    pub fn fragmentation_propensity(&self) -> Vec<f32> { self.inner.fragmentation_propensity.clone() }
    pub fn fragment_ids(&self) -> Vec<u16> { self.inner.fragment_ids.clone() }
    pub fn kinematic_domain_ids(&self) -> Vec<u16> { self.inner.kinematic_domain_ids.clone() }

    pub fn boundary_samples(&self) -> Vec<u32> { self.boundaries.flattened_samples() }
    pub fn boundary_kinds(&self) -> Vec<u8> { self.boundaries.tectonic_kinds() }
    pub fn geological_boundary_regimes(&self) -> Vec<u8> { self.boundaries.geological_regimes() }
    pub fn subduction_polarities(&self) -> Vec<u8> { self.boundaries.subduction_polarities() }
    pub fn boundary_normal_rates_m_per_year(&self) -> Vec<f64> { self.boundaries.normal_rates_m_per_year() }
    pub fn boundary_shear_rates_m_per_year(&self) -> Vec<f64> { self.boundaries.shear_rates_m_per_year() }
    pub fn boundary_coarse_source_indices(&self) -> Vec<u32> { self.boundaries.coarse_boundary_indices() }
}
