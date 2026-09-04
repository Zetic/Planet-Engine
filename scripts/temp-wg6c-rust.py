from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"marker not found in {path}: {old[:180]!r}")
    target.write_text(text.replace(old, new, 1))


replace_once(
    "rust/interlink-worldgen/src/lib.rs",
    "mod hydroclimate;\nmod lithosphere;",
    "mod hydroclimate;\nmod lakes;\nmod lithosphere;",
)
replace_once(
    "rust/interlink-worldgen/src/lib.rs",
    "pub use lithosphere::{\n",
    "pub use lakes::{\n"
    "    generate_lakes_closed_basins, LakeMetrics, LakeParameters, LakeRecord, LakeRequest,\n"
    "    LakeState, LAKE_KIND_ENDORHEIC, LAKE_KIND_NONE, LAKE_KIND_OVERFLOWING,\n"
    "    LAKE_KIND_TERMINAL_STORAGE, LAKE_STAGE_ID, LAKE_STAGE_VERSION,\n"
    "};\n"
    "pub use lithosphere::{\n",
)

replace_once(
    "rust/interlink-worldgen-wasm/src/lib.rs",
    "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 12;",
    "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 13;",
)

replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "    generate_drainage_topology, generate_initial_topography, generate_lithosphere,\n"
    "    generate_runoff_discharge, generate_tectonics, inherit_boundary_interfaces,\n",
    "    generate_drainage_topology, generate_initial_topography, generate_lakes_closed_basins,\n"
    "    generate_lithosphere, generate_runoff_discharge, generate_tectonics,\n"
    "    inherit_boundary_interfaces,\n",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "    InheritedPhysicalState, LithosphereRequest, PlanetPhysicalParameters, RunoffRequest,\n"
    "    RunoffState, TectonicsRequest, TopographyRequest, TopographyState, WORLDGEN_ENGINE_VERSION,\n",
    "    InheritedPhysicalState, LakeRequest, LakeState, LithosphereRequest,\n"
    "    PlanetPhysicalParameters, RunoffRequest, RunoffState, TectonicsRequest,\n"
    "    TopographyRequest, TopographyState, WORLDGEN_ENGINE_VERSION,\n",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "const GENERATION_STAGE_COUNT: u32 = 12;",
    "const GENERATION_STAGE_COUNT: u32 = 13;",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "    drainage: DrainageState,\n    runoff: RunoffState,\n    planet: PlanetPhysicalParameters,",
    "    drainage: DrainageState,\n    runoff: RunoffState,\n    lakes: LakeState,\n    planet: PlanetPhysicalParameters,",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "            &RunoffRequest::new(seed),\n        )\n        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"runoff-discharge\", 10, 1, 1);\n\n        Ok(Self {",
    "            &RunoffRequest::new(seed.as_str()),\n        )\n        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"runoff-discharge\", 10, 1, 1);\n\n"
    "        report_generation_progress(progress, \"lake-equilibrium\", 11, 0, 1);\n"
    "        let lakes = generate_lakes_closed_basins(\n"
    "            &fine_topology,\n"
    "            &terrain,\n"
    "            &climate,\n"
    "            &drainage,\n"
    "            &runoff,\n"
    "            planet,\n"
    "            &LakeRequest::new(seed.as_str()),\n"
    "        )\n"
    "        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n"
    "        report_generation_progress(progress, \"lake-equilibrium\", 11, 1, 1);\n\n"
    "        Ok(Self {",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "            drainage,\n            runoff,\n            planet,",
    "            drainage,\n            runoff,\n            lakes,\n            planet,",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "    pub fn potential_discharge_m3_s(&self) -> Vec<f32> {\n"
    "        self.runoff.potential_discharge_m3_s.clone()\n"
    "    }\n}\\n" if False else "    pub fn potential_discharge_m3_s(&self) -> Vec<f32> {\n        self.runoff.potential_discharge_m3_s.clone()\n    }\n}\n",
    "    pub fn potential_discharge_m3_s(&self) -> Vec<f32> {\n"
    "        self.runoff.potential_discharge_m3_s.clone()\n"
    "    }\n\n"
    "    pub fn lake_stage_id(&self) -> String { self.lakes.stage.id.to_owned() }\n"
    "    pub fn lake_stage_version(&self) -> u32 { self.lakes.stage.version }\n"
    "    pub fn lake_stage_seed_hex(&self) -> String { format!(\"{:016x}\", self.lakes.stage.derived_seed) }\n"
    "    pub fn lake_hash_hex(&self) -> String { self.lakes.metrics.lake_hash_hex() }\n"
    "    pub fn lake_parameter_hash_hex(&self) -> String { self.lakes.metrics.lake_parameter_hash_hex() }\n"
    "    pub fn lake_climate_hash_hex(&self) -> String { self.lakes.metrics.climate_hash_hex() }\n"
    "    pub fn lake_drainage_hash_hex(&self) -> String { self.lakes.metrics.drainage_hash_hex() }\n"
    "    pub fn lake_runoff_hash_hex(&self) -> String { self.lakes.metrics.runoff_hash_hex() }\n"
    "    pub fn lake_count(&self) -> u32 { self.lakes.metrics.lake_count }\n"
    "    pub fn endorheic_lake_count(&self) -> u32 { self.lakes.metrics.endorheic_lake_count }\n"
    "    pub fn overflowing_lake_count(&self) -> u32 { self.lakes.metrics.overflowing_lake_count }\n"
    "    pub fn terminal_storage_lake_count(&self) -> u32 { self.lakes.metrics.terminal_storage_lake_count }\n"
    "    pub fn lake_sample_count(&self) -> u32 { self.lakes.metrics.lake_sample_count }\n"
    "    pub fn total_lake_area_m2(&self) -> f64 { self.lakes.metrics.total_lake_area_m2 }\n"
    "    pub fn total_lake_volume_m3(&self) -> f64 { self.lakes.metrics.total_lake_volume_m3 }\n"
    "    pub fn maximum_lake_area_m2(&self) -> f64 { self.lakes.metrics.maximum_lake_area_m2 }\n"
    "    pub fn maximum_lake_depth_m(&self) -> f64 { self.lakes.metrics.maximum_lake_depth_m }\n"
    "    pub fn total_lake_precipitation_m3_s(&self) -> f64 { self.lakes.metrics.total_lake_precipitation_m3_s }\n"
    "    pub fn total_lake_evaporation_m3_s(&self) -> f64 { self.lakes.metrics.total_lake_evaporation_m3_s }\n"
    "    pub fn terminal_realized_discharge_m3_s(&self) -> f64 { self.lakes.metrics.terminal_realized_discharge_m3_s }\n"
    "    pub fn maximum_realized_discharge_m3_s(&self) -> f64 { self.lakes.metrics.maximum_realized_discharge_m3_s }\n"
    "    pub fn unreleased_storage_m3_s(&self) -> f64 { self.lakes.metrics.unreleased_storage_m3_s }\n"
    "    pub fn lake_water_balance_relative_error(&self) -> f64 { self.lakes.metrics.water_balance_relative_error }\n"
    "    pub fn lake_id(&self) -> Vec<u32> { self.lakes.lake_id.clone() }\n"
    "    pub fn lake_kind(&self) -> Vec<u8> { self.lakes.lake_kind.clone() }\n"
    "    pub fn lake_fraction(&self) -> Vec<f32> { self.lakes.lake_fraction.clone() }\n"
    "    pub fn lake_depth_m(&self) -> Vec<f32> { self.lakes.lake_depth_m.clone() }\n"
    "    pub fn realized_discharge_m3_s(&self) -> Vec<f32> { self.lakes.realized_discharge_m3_s.clone() }\n"
    "    pub fn lake_depression_ids(&self) -> Vec<u32> { self.lakes.lakes.iter().map(|lake| lake.depression_id).collect() }\n"
    "    pub fn lake_kinds(&self) -> Vec<u8> { self.lakes.lakes.iter().map(|lake| lake.kind).collect() }\n"
    "    pub fn lake_surface_elevations_m(&self) -> Vec<f64> { self.lakes.lakes.iter().map(|lake| lake.surface_elevation_m).collect() }\n"
    "    pub fn lake_areas_m2(&self) -> Vec<f64> { self.lakes.lakes.iter().map(|lake| lake.area_m2).collect() }\n"
    "    pub fn lake_volumes_m3(&self) -> Vec<f64> { self.lakes.lakes.iter().map(|lake| lake.volume_m3).collect() }\n"
    "    pub fn lake_outflows_m3_s(&self) -> Vec<f64> { self.lakes.lakes.iter().map(|lake| lake.outflow_m3_s).collect() }\n"
    "    pub fn lake_spill_samples(&self) -> Vec<u32> { self.lakes.lakes.iter().map(|lake| lake.spill_sample).collect() }\n"
    "}\n",
)
