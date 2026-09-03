use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_lithosphere, generate_tectonics,
    CrustalModel, GeodesicTopology, GeologyRequest, LithosphereRequest, LithosphericModel,
    PlanetPhysicalParameters, TectonicModel, TectonicsRequest, WORLDGEN_ENGINE_VERSION,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmWorldgenLithosphere {
    topology: GeodesicTopology,
    tectonics: TectonicModel,
    geology: CrustalModel,
    inner: LithosphericModel,
}

#[wasm_bindgen]
impl WasmWorldgenLithosphere {
    #[wasm_bindgen(constructor)]
    pub fn new(
        seed: String,
        level: u8,
        plate_count: u16,
    ) -> Result<WasmWorldgenLithosphere, JsValue> {
        let topology =
            build_icosphere(level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let tectonics = generate_tectonics(
            &topology,
            &TectonicsRequest::new(seed.as_str(), plate_count),
            parameters,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let geology = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new(seed.as_str()),
            parameters,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let inner = generate_lithosphere(
            &topology,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        Ok(Self {
            topology,
            tectonics,
            geology,
            inner,
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
    pub fn mechanical_seed_hex(&self) -> String {
        format!("{:016x}", self.inner.mechanical_seed)
    }
    pub fn mantle_seed_hex(&self) -> String {
        format!("{:016x}", self.inner.mantle_seed)
    }
    pub fn refinement_seed_hex(&self) -> String {
        format!("{:016x}", self.inner.refinement_seed)
    }
    pub fn level(&self) -> u8 {
        self.topology.level()
    }
    pub fn sample_count(&self) -> u32 {
        self.inner.metrics.sample_count
    }
    pub fn plate_count(&self) -> u16 {
        self.tectonics.metrics.plate_count
    }
    pub fn boundary_edge_count(&self) -> u32 {
        self.tectonics.metrics.boundary_edge_count
    }
    pub fn topology_hash_hex(&self) -> String {
        self.topology.metrics().topology_hash_hex()
    }
    pub fn tectonic_hash_hex(&self) -> String {
        self.tectonics.metrics.tectonic_hash_hex()
    }
    pub fn geology_hash_hex(&self) -> String {
        self.geology.metrics.geology_hash_hex()
    }
    pub fn lithosphere_hash_hex(&self) -> String {
        self.inner.metrics.lithosphere_hash_hex()
    }

    pub fn mean_strength_index(&self) -> f64 {
        self.inner.metrics.mean_strength_index
    }
    pub fn mean_weakness_index(&self) -> f64 {
        self.inner.metrics.mean_weakness_index
    }
    pub fn mean_effective_elastic_thickness_km(&self) -> f64 {
        self.inner.metrics.mean_effective_elastic_thickness_km
    }
    pub fn mean_mantle_upwelling_index(&self) -> f64 {
        self.inner.metrics.mean_mantle_upwelling_index
    }
    pub fn mean_dynamic_support_index(&self) -> f64 {
        self.inner.metrics.mean_dynamic_support_index
    }
    pub fn suture_sample_count(&self) -> u32 {
        self.inner.metrics.suture_sample_count
    }
    pub fn rift_zone_sample_count(&self) -> u32 {
        self.inner.metrics.rift_zone_sample_count
    }
    pub fn transform_zone_sample_count(&self) -> u32 {
        self.inner.metrics.transform_zone_sample_count
    }
    pub fn continental_margin_sample_count(&self) -> u32 {
        self.inner.metrics.continental_margin_sample_count
    }
    pub fn tectonic_fragment_count(&self) -> u16 {
        self.inner.metrics.tectonic_fragment_count
    }
    pub fn microplate_count(&self) -> u16 {
        self.inner.metrics.microplate_count
    }
    pub fn terrane_count(&self) -> u16 {
        self.inner.metrics.terrane_count
    }
    pub fn fragmented_area_fraction(&self) -> f64 {
        self.inner.metrics.fragmented_area_fraction
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
    pub fn plate_ids(&self) -> Vec<u16> {
        self.tectonics.plate_ids.clone()
    }
    pub fn boundary_samples(&self) -> Vec<u32> {
        self.tectonics.flattened_boundary_samples()
    }
    pub fn boundary_kinds(&self) -> Vec<u8> {
        self.tectonics.boundary_kinds()
    }
    pub fn crust_kind(&self) -> Vec<u8> {
        self.geology.crust_kind.clone()
    }
    pub fn geological_boundary_regimes(&self) -> Vec<u8> {
        self.geology.boundary_regimes()
    }
    pub fn orogenic_history(&self) -> Vec<f32> {
        self.geology.orogenic_history.clone()
    }
    pub fn rift_history(&self) -> Vec<f32> {
        self.geology.rift_history.clone()
    }
    pub fn ridge_history(&self) -> Vec<f32> {
        self.geology.ridge_history.clone()
    }
    pub fn subduction_history(&self) -> Vec<f32> {
        self.geology.subduction_history.clone()
    }
    pub fn transform_history(&self) -> Vec<f32> {
        self.geology.transform_history.clone()
    }
    pub fn crustal_strain(&self) -> Vec<f32> {
        self.geology.crustal_strain.clone()
    }

    pub fn strength_index(&self) -> Vec<f32> {
        self.inner.strength_index.clone()
    }
    pub fn weakness_index(&self) -> Vec<f32> {
        self.inner.weakness_index.clone()
    }
    pub fn effective_elastic_thickness_km(&self) -> Vec<f32> {
        self.inner.effective_elastic_thickness_km.clone()
    }
    pub fn thermal_anomaly_index(&self) -> Vec<f32> {
        self.inner.thermal_anomaly_index.clone()
    }
    pub fn mantle_upwelling_index(&self) -> Vec<f32> {
        self.inner.mantle_upwelling_index.clone()
    }
    pub fn mantle_dynamic_support_index(&self) -> Vec<f32> {
        self.inner.mantle_dynamic_support_index.clone()
    }
    pub fn compensated_buoyancy_index(&self) -> Vec<f32> {
        self.inner.compensated_buoyancy_index.clone()
    }
    pub fn structural_fabric_strength(&self) -> Vec<f32> {
        self.inner.structural_fabric_strength.clone()
    }
    pub fn structural_zone_kind(&self) -> Vec<u8> {
        self.inner.structural_zone_kind.clone()
    }
    pub fn fragmentation_propensity(&self) -> Vec<f32> {
        self.inner.fragmentation_propensity.clone()
    }
    pub fn fragment_ids(&self) -> Vec<u16> {
        self.inner.fragment_ids.clone()
    }
    pub fn kinematic_domain_ids(&self) -> Vec<u16> {
        self.inner.kinematic_domain_ids.clone()
    }

    pub fn fragment_parent_plate_ids(&self) -> Vec<u16> {
        self.inner
            .fragments
            .iter()
            .map(|fragment| fragment.parent_plate_id)
            .collect()
    }
    pub fn fragment_kinds(&self) -> Vec<u8> {
        self.inner
            .fragments
            .iter()
            .map(|fragment| fragment.kind as u8)
            .collect()
    }
    pub fn fragment_seed_samples(&self) -> Vec<u32> {
        self.inner
            .fragments
            .iter()
            .map(|fragment| fragment.seed_sample)
            .collect()
    }
    pub fn fragment_area_steradians(&self) -> Vec<f64> {
        self.inner
            .fragments
            .iter()
            .map(|fragment| fragment.area_steradians)
            .collect()
    }
    pub fn fragment_area_fractions_of_parent(&self) -> Vec<f64> {
        self.inner
            .fragments
            .iter()
            .map(|fragment| fragment.area_fraction_of_parent)
            .collect()
    }
    pub fn fragment_mean_weakness(&self) -> Vec<f64> {
        self.inner
            .fragments
            .iter()
            .map(|fragment| fragment.mean_weakness)
            .collect()
    }
    pub fn fragment_mean_propensity(&self) -> Vec<f64> {
        self.inner
            .fragments
            .iter()
            .map(|fragment| fragment.mean_fragmentation_propensity)
            .collect()
    }
    pub fn fragment_angular_velocities_rad_per_myr(&self) -> Vec<f64> {
        let mut output = Vec::with_capacity(self.inner.fragments.len() * 3);
        for fragment in &self.inner.fragments {
            output.extend_from_slice(&fragment.angular_velocity_rad_per_myr);
        }
        output
    }
}
