mod climate_bridge;
pub use climate_bridge::WasmWorldgenClimate;
mod inheritance_bridge;
pub use inheritance_bridge::WasmWorldgenInheritance;
mod topography_bridge;
pub use topography_bridge::WasmWorldgenTopography;
mod lithosphere_bridge;
pub use lithosphere_bridge::WasmWorldgenLithosphere;

use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_synthetic, generate_tectonics,
    CrustalModel, GeodesicTopology, GeologyRequest, PlanetPhysicalParameters, SyntheticDiagnostic,
    SyntheticRequest, TectonicModel, TectonicsRequest, WORLDGEN_ENGINE_VERSION,
};
use wasm_bindgen::prelude::*;

pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 10;
#[wasm_bindgen]
pub fn worldgen_protocol_version() -> u32 {
    WORLDGEN_WASM_PROTOCOL_VERSION
}
#[wasm_bindgen]
pub fn worldgen_engine_version() -> u32 {
    WORLDGEN_ENGINE_VERSION
}

#[wasm_bindgen]
pub struct WasmWorldgenDiagnostic {
    inner: SyntheticDiagnostic,
}
#[wasm_bindgen]
impl WasmWorldgenDiagnostic {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: String, width: u32, height: u32) -> Result<WasmWorldgenDiagnostic, JsValue> {
        let inner = generate_synthetic(&SyntheticRequest::new(seed, width, height))
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        Ok(Self { inner })
    }
    pub fn generator_version(&self) -> u32 {
        self.inner.generator_version
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
    pub fn width(&self) -> u32 {
        self.inner.field.width()
    }
    pub fn height(&self) -> u32 {
        self.inner.field.height()
    }
    pub fn sample_count(&self) -> u64 {
        self.inner.statistics.sample_count
    }
    pub fn minimum(&self) -> u16 {
        self.inner.statistics.minimum
    }
    pub fn maximum(&self) -> u16 {
        self.inner.statistics.maximum
    }
    pub fn mean(&self) -> f64 {
        self.inner.statistics.mean
    }
    pub fn field_hash_hex(&self) -> String {
        self.inner.statistics.hash_hex()
    }
    pub fn values(&self) -> Vec<u16> {
        self.inner.field.values().to_vec()
    }
}

#[wasm_bindgen]
pub struct WasmWorldgenTopology {
    inner: GeodesicTopology,
}
#[wasm_bindgen]
impl WasmWorldgenTopology {
    #[wasm_bindgen(constructor)]
    pub fn new(level: u8) -> Result<WasmWorldgenTopology, JsValue> {
        Ok(Self {
            inner: build_icosphere(level).map_err(|error| JsValue::from_str(&error.to_string()))?,
        })
    }
    pub fn generator_version(&self) -> u32 {
        WORLDGEN_ENGINE_VERSION
    }
    pub fn level(&self) -> u8 {
        self.inner.level()
    }
    pub fn sample_count(&self) -> u32 {
        self.inner.metrics().sample_count
    }
    pub fn edge_count(&self) -> u32 {
        self.inner.metrics().edge_count
    }
    pub fn face_count(&self) -> u32 {
        self.inner.metrics().face_count
    }
    pub fn five_neighbor_count(&self) -> u32 {
        self.inner.metrics().five_neighbor_count
    }
    pub fn six_neighbor_count(&self) -> u32 {
        self.inner.metrics().six_neighbor_count
    }
    pub fn total_area_steradians(&self) -> f64 {
        self.inner.metrics().total_area_steradians
    }
    pub fn minimum_area_steradians(&self) -> f64 {
        self.inner.metrics().minimum_area_steradians
    }
    pub fn maximum_area_steradians(&self) -> f64 {
        self.inner.metrics().maximum_area_steradians
    }
    pub fn mean_area_steradians(&self) -> f64 {
        self.inner.metrics().mean_area_steradians
    }
    pub fn area_coefficient_of_variation(&self) -> f64 {
        self.inner.metrics().area_coefficient_of_variation
    }
    pub fn minimum_edge_arc_radians(&self) -> f64 {
        self.inner.metrics().minimum_edge_arc_radians
    }
    pub fn maximum_edge_arc_radians(&self) -> f64 {
        self.inner.metrics().maximum_edge_arc_radians
    }
    pub fn mean_edge_arc_radians(&self) -> f64 {
        self.inner.metrics().mean_edge_arc_radians
    }
    pub fn edge_coefficient_of_variation(&self) -> f64 {
        self.inner.metrics().edge_coefficient_of_variation
    }
    pub fn minimum_interface_arc_radians(&self) -> f64 {
        self.inner.metrics().minimum_interface_arc_radians
    }
    pub fn maximum_interface_arc_radians(&self) -> f64 {
        self.inner.metrics().maximum_interface_arc_radians
    }
    pub fn mean_interface_arc_radians(&self) -> f64 {
        self.inner.metrics().mean_interface_arc_radians
    }
    pub fn interface_coefficient_of_variation(&self) -> f64 {
        self.inner.metrics().interface_coefficient_of_variation
    }
    pub fn topology_hash_hex(&self) -> String {
        self.inner.metrics().topology_hash_hex()
    }
    pub fn positions(&self) -> Vec<f64> {
        self.inner.flattened_positions()
    }
    pub fn faces(&self) -> Vec<u32> {
        self.inner.flattened_faces()
    }
    pub fn neighbor_offsets(&self) -> Vec<u32> {
        self.inner.neighbor_offsets().to_vec()
    }
    pub fn neighbors(&self) -> Vec<u32> {
        self.inner.neighbor_indices().to_vec()
    }
    pub fn neighbor_arc_lengths_rad(&self) -> Vec<f64> {
        self.inner.neighbor_center_arc_lengths_rad_values().to_vec()
    }
    pub fn neighbor_interface_arc_lengths_rad(&self) -> Vec<f64> {
        self.inner
            .neighbor_interface_arc_lengths_rad_values()
            .to_vec()
    }
    pub fn area_steradians(&self) -> Vec<f64> {
        self.inner.dual_area_steradians().to_vec()
    }
    pub fn birth_levels(&self) -> Vec<u8> {
        self.inner.birth_levels().to_vec()
    }
    pub fn parent_edges(&self) -> Vec<u32> {
        self.inner.flattened_parent_edges()
    }
}

#[wasm_bindgen]
pub struct WasmWorldgenTectonics {
    topology: GeodesicTopology,
    inner: TectonicModel,
}
#[wasm_bindgen]
impl WasmWorldgenTectonics {
    #[wasm_bindgen(constructor)]
    pub fn new(
        seed: String,
        level: u8,
        plate_count: u16,
    ) -> Result<WasmWorldgenTectonics, JsValue> {
        let topology =
            build_icosphere(level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let inner = generate_tectonics(
            &topology,
            &TectonicsRequest::new(seed, plate_count),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        Ok(Self { topology, inner })
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
    pub fn level(&self) -> u8 {
        self.topology.level()
    }
    pub fn sample_count(&self) -> u32 {
        self.inner.metrics.sample_count
    }
    pub fn plate_count(&self) -> u16 {
        self.inner.metrics.plate_count
    }
    pub fn boundary_edge_count(&self) -> u32 {
        self.inner.metrics.boundary_edge_count
    }
    pub fn convergent_edge_count(&self) -> u32 {
        self.inner.metrics.convergent_edge_count
    }
    pub fn divergent_edge_count(&self) -> u32 {
        self.inner.metrics.divergent_edge_count
    }
    pub fn transform_edge_count(&self) -> u32 {
        self.inner.metrics.transform_edge_count
    }
    pub fn minimum_plate_area_fraction(&self) -> f64 {
        self.inner.metrics.minimum_plate_area_fraction
    }
    pub fn maximum_plate_area_fraction(&self) -> f64 {
        self.inner.metrics.maximum_plate_area_fraction
    }
    pub fn mean_plate_area_fraction(&self) -> f64 {
        self.inner.metrics.mean_plate_area_fraction
    }
    pub fn minimum_seed_separation_rad(&self) -> f64 {
        self.inner.metrics.minimum_seed_separation_rad
    }
    pub fn mean_reference_speed_mm_per_year(&self) -> f64 {
        self.inner.metrics.mean_reference_speed_mm_per_year
    }
    pub fn tectonic_hash_hex(&self) -> String {
        self.inner.metrics.tectonic_hash_hex()
    }
    pub fn topology_hash_hex(&self) -> String {
        self.topology.metrics().topology_hash_hex()
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
        self.inner.plate_ids.clone()
    }
    pub fn plate_seed_samples(&self) -> Vec<u32> {
        self.inner.plate_seed_samples()
    }
    pub fn plate_euler_poles(&self) -> Vec<f64> {
        self.inner.flattened_euler_poles()
    }
    pub fn plate_angular_velocities_rad_per_myr(&self) -> Vec<f64> {
        self.inner.flattened_angular_velocities_rad_per_myr()
    }
    pub fn plate_area_steradians(&self) -> Vec<f64> {
        self.inner.plate_area_steradians()
    }
    pub fn boundary_samples(&self) -> Vec<u32> {
        self.inner.flattened_boundary_samples()
    }
    pub fn boundary_plate_ids(&self) -> Vec<u16> {
        self.inner.flattened_boundary_plate_ids()
    }
    pub fn boundary_kinds(&self) -> Vec<u8> {
        self.inner.boundary_kinds()
    }
    pub fn boundary_normal_rates_m_per_year(&self) -> Vec<f64> {
        self.inner.boundary_normal_rates_m_per_year()
    }
    pub fn boundary_shear_rates_m_per_year(&self) -> Vec<f64> {
        self.inner.boundary_shear_rates_m_per_year()
    }
}

#[wasm_bindgen]
pub struct WasmWorldgenGeology {
    topology: GeodesicTopology,
    tectonics: TectonicModel,
    inner: CrustalModel,
}

#[wasm_bindgen]
impl WasmWorldgenGeology {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: String, level: u8, plate_count: u16) -> Result<WasmWorldgenGeology, JsValue> {
        let topology =
            build_icosphere(level).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let parameters = PlanetPhysicalParameters::earthlike_reference();
        let tectonics = generate_tectonics(
            &topology,
            &TectonicsRequest::new(seed.as_str(), plate_count),
            parameters,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let inner = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new(seed),
            parameters,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        Ok(Self {
            topology,
            tectonics,
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
    pub fn province_seed_hex(&self) -> String {
        format!("{:016x}", self.inner.province_seed)
    }
    pub fn property_seed_hex(&self) -> String {
        format!("{:016x}", self.inner.property_seed)
    }
    pub fn history_seed_hex(&self) -> String {
        format!("{:016x}", self.inner.history_seed)
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
    pub fn geology_hash_hex(&self) -> String {
        self.inner.metrics.geology_hash_hex()
    }
    pub fn tectonic_hash_hex(&self) -> String {
        self.tectonics.metrics.tectonic_hash_hex()
    }
    pub fn topology_hash_hex(&self) -> String {
        self.topology.metrics().topology_hash_hex()
    }

    pub fn continental_area_fraction(&self) -> f64 {
        self.inner.metrics.continental_area_fraction
    }
    pub fn transitional_area_fraction(&self) -> f64 {
        self.inner.metrics.transitional_area_fraction
    }
    pub fn oceanic_area_fraction(&self) -> f64 {
        self.inner.metrics.oceanic_area_fraction
    }
    pub fn mean_continental_age_myr(&self) -> f64 {
        self.inner.metrics.mean_continental_age_myr
    }
    pub fn mean_oceanic_age_myr(&self) -> f64 {
        self.inner.metrics.mean_oceanic_age_myr
    }
    pub fn mean_continental_thickness_km(&self) -> f64 {
        self.inner.metrics.mean_continental_thickness_km
    }
    pub fn mean_oceanic_thickness_km(&self) -> f64 {
        self.inner.metrics.mean_oceanic_thickness_km
    }

    pub fn oceanic_subduction_edges(&self) -> u32 {
        self.inner.metrics.oceanic_subduction_edges
    }
    pub fn ocean_continent_subduction_edges(&self) -> u32 {
        self.inner.metrics.ocean_continent_subduction_edges
    }
    pub fn continental_collision_edges(&self) -> u32 {
        self.inner.metrics.continental_collision_edges
    }
    pub fn oceanic_ridge_edges(&self) -> u32 {
        self.inner.metrics.oceanic_ridge_edges
    }
    pub fn continental_rift_edges(&self) -> u32 {
        self.inner.metrics.continental_rift_edges
    }
    pub fn transitional_divergence_edges(&self) -> u32 {
        self.inner.metrics.transitional_divergence_edges
    }
    pub fn transform_edges(&self) -> u32 {
        self.inner.metrics.transform_edges
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
    pub fn boundary_plate_ids(&self) -> Vec<u16> {
        self.tectonics.flattened_boundary_plate_ids()
    }
    pub fn boundary_kinds(&self) -> Vec<u8> {
        self.tectonics.boundary_kinds()
    }

    pub fn crust_kind(&self) -> Vec<u8> {
        self.inner.crust_kind.clone()
    }
    pub fn crust_province_id(&self) -> Vec<u16> {
        self.inner.crust_province_id.clone()
    }
    pub fn crust_age_myr(&self) -> Vec<f32> {
        self.inner.crust_age_myr.clone()
    }
    pub fn crust_thickness_km(&self) -> Vec<f32> {
        self.inner.crust_thickness_km.clone()
    }
    pub fn crust_density_kg_per_m3(&self) -> Vec<f32> {
        self.inner.crust_density_kg_per_m3.clone()
    }
    pub fn buoyancy_index(&self) -> Vec<f32> {
        self.inner.buoyancy_index.clone()
    }
    pub fn orogenic_history(&self) -> Vec<f32> {
        self.inner.orogenic_history.clone()
    }
    pub fn rift_history(&self) -> Vec<f32> {
        self.inner.rift_history.clone()
    }
    pub fn ridge_history(&self) -> Vec<f32> {
        self.inner.ridge_history.clone()
    }
    pub fn subduction_history(&self) -> Vec<f32> {
        self.inner.subduction_history.clone()
    }
    pub fn trench_history(&self) -> Vec<f32> {
        self.inner.trench_history.clone()
    }
    pub fn volcanic_arc_history(&self) -> Vec<f32> {
        self.inner.volcanic_arc_history.clone()
    }
    pub fn transform_history(&self) -> Vec<f32> {
        self.inner.transform_history.clone()
    }
    pub fn subsidence_history(&self) -> Vec<f32> {
        self.inner.subsidence_history.clone()
    }
    pub fn basin_potential(&self) -> Vec<f32> {
        self.inner.basin_potential.clone()
    }
    pub fn crustal_strain(&self) -> Vec<f32> {
        self.inner.crustal_strain.clone()
    }

    pub fn geological_boundary_regimes(&self) -> Vec<u8> {
        self.inner.boundary_regimes()
    }
    pub fn subduction_polarities(&self) -> Vec<u8> {
        self.inner.subduction_polarities()
    }
    pub fn plate_scale_classes(&self) -> Vec<u8> {
        self.inner.plate_scale_classes()
    }
    pub fn plate_continental_fractions(&self) -> Vec<f64> {
        self.inner
            .plate_summaries
            .iter()
            .map(|plate| plate.continental_fraction)
            .collect()
    }
    pub fn plate_transitional_fractions(&self) -> Vec<f64> {
        self.inner
            .plate_summaries
            .iter()
            .map(|plate| plate.transitional_fraction)
            .collect()
    }
    pub fn plate_oceanic_fractions(&self) -> Vec<f64> {
        self.inner
            .plate_summaries
            .iter()
            .map(|plate| plate.oceanic_fraction)
            .collect()
    }
    pub fn plate_mean_crust_age_myr(&self) -> Vec<f64> {
        self.inner
            .plate_summaries
            .iter()
            .map(|plate| plate.mean_crust_age_myr)
            .collect()
    }
    pub fn plate_mean_crust_thickness_km(&self) -> Vec<f64> {
        self.inner
            .plate_summaries
            .iter()
            .map(|plate| plate.mean_crust_thickness_km)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wasm_protocol_and_engine_versions_are_explicit() {
        assert_eq!(worldgen_protocol_version(), WORLDGEN_WASM_PROTOCOL_VERSION);
        assert_eq!(worldgen_engine_version(), WORLDGEN_ENGINE_VERSION);
    }
    #[test]
    fn topology_bridge_exposes_canonical_counts_and_flux_geometry() {
        let topology = WasmWorldgenTopology::new(2).unwrap();
        assert_eq!(topology.sample_count(), 162);
        assert_eq!(topology.five_neighbor_count(), 12);
        assert_eq!(topology.faces().len(), 320 * 3);
        assert_eq!(
            topology.neighbors().len(),
            topology.neighbor_arc_lengths_rad().len()
        );
        assert_eq!(
            topology.neighbors().len(),
            topology.neighbor_interface_arc_lengths_rad().len()
        );
    }
    #[test]
    fn tectonics_bridge_exposes_complete_partition_and_boundaries() {
        let tectonics = WasmWorldgenTectonics::new("wasm-wg2".to_owned(), 3, 12).unwrap();
        assert_eq!(
            tectonics.plate_ids().len(),
            tectonics.sample_count() as usize
        );
        assert_eq!(tectonics.plate_seed_samples().len(), 12);
        assert_eq!(
            tectonics.boundary_samples().len(),
            tectonics.boundary_edge_count() as usize * 2
        );
        assert_eq!(
            tectonics.boundary_kinds().len(),
            tectonics.boundary_edge_count() as usize
        );
    }
    #[test]
    fn geology_bridge_exposes_complete_sample_fields_and_boundary_interpretation() {
        let geology = WasmWorldgenGeology::new("wasm-wg3".to_owned(), 3, 12).unwrap();
        let samples = geology.sample_count() as usize;
        assert_eq!(geology.crust_kind().len(), samples);
        assert_eq!(geology.crust_age_myr().len(), samples);
        assert_eq!(geology.crust_thickness_km().len(), samples);
        assert_eq!(geology.orogenic_history().len(), samples);
        assert_eq!(
            geology.geological_boundary_regimes().len(),
            geology.boundary_edge_count() as usize
        );
        assert_eq!(
            geology.subduction_polarities().len(),
            geology.boundary_edge_count() as usize
        );
        assert_eq!(
            geology.plate_scale_classes().len(),
            geology.plate_count() as usize
        );
    }
    #[test]
    fn lithosphere_bridge_exposes_mechanics_and_refinement_fields() {
        let lithosphere = WasmWorldgenLithosphere::new("wasm-wg3-5".to_owned(), 3, 12).unwrap();
        let samples = lithosphere.sample_count() as usize;
        assert_eq!(lithosphere.strength_index().len(), samples);
        assert_eq!(lithosphere.effective_elastic_thickness_km().len(), samples);
        assert_eq!(lithosphere.structural_zone_kind().len(), samples);
        assert_eq!(lithosphere.fragment_ids().len(), samples);
        assert_eq!(lithosphere.kinematic_domain_ids().len(), samples);
        assert_eq!(
            lithosphere.fragment_kinds().len(),
            lithosphere.tectonic_fragment_count() as usize
        );
    }
    #[test]
    fn inheritance_bridge_exposes_fine_physics_and_boundaries() {
        let inherited = WasmWorldgenInheritance::new("wasm-wg3-75".to_owned(), 2, 3, 10).unwrap();
        let samples = inherited.fine_sample_count() as usize;
        assert_eq!(inherited.plate_ids().len(), samples);
        assert_eq!(inherited.crust_kind().len(), samples);
        assert_eq!(inherited.strength_index().len(), samples);
        assert_eq!(inherited.nearest_coarse_source().len(), samples);
        assert_eq!(inherited.inherited_sample_mask().len(), samples);
        assert_eq!(
            inherited.boundary_samples().len(),
            inherited.fine_boundary_edge_count() as usize * 2
        );
        assert_eq!(
            inherited.boundary_kinds().len(),
            inherited.fine_boundary_edge_count() as usize
        );
        assert_eq!(
            inherited.geological_boundary_regimes().len(),
            inherited.fine_boundary_edge_count() as usize
        );
        assert_eq!(
            inherited.boundary_coarse_source_indices().len(),
            inherited.fine_boundary_edge_count() as usize
        );
        assert!(inherited.equivalent_global_water_depth_m() > 0.0);
    }
}
