from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"marker not found in {path}: {old[:180]!r}")
    target.write_text(text.replace(old, new, 1))


# Register the WG-6B core module and public contract.
replace_once(
    "rust/interlink-worldgen/src/lib.rs",
    "mod random;\nmod refinement;",
    "mod random;\nmod runoff;\nmod refinement;",
)
replace_once(
    "rust/interlink-worldgen/src/lib.rs",
    "pub use random::derive_stage_seed;\n",
    "pub use random::derive_stage_seed;\npub use runoff::{\n    generate_runoff_discharge, RunoffMetrics, RunoffParameters, RunoffRequest, RunoffState,\n    RUNOFF_STAGE_ID, RUNOFF_STAGE_VERSION,\n};\n",
)

# Keep accumulation numerically conservative in f64, then publish compact f32 fields.
replace_once(
    "rust/interlink-worldgen/src/runoff.rs",
    "    let mut potential_discharge_m3_s = vec![0.0_f32; count];",
    "    let mut potential_discharge_accum_m3_s = vec![0.0_f64; count];",
)
replace_once(
    "rust/interlink-worldgen/src/runoff.rs",
    "        potential_discharge_m3_s[i] = local_m3_s as f32;",
    "        potential_discharge_accum_m3_s[i] = local_m3_s;",
)
replace_once(
    "rust/interlink-worldgen/src/runoff.rs",
    "        let accumulated = f64::from(potential_discharge_m3_s[ri])\n            + f64::from(potential_discharge_m3_s[i]);\n        if !accumulated.is_finite() || accumulated > f32::MAX as f64 {\n            return Err(\"runoff accumulated discharge exceeds representable range\");\n        }\n        potential_discharge_m3_s[ri] = accumulated as f32;",
    "        let accumulated = potential_discharge_accum_m3_s[ri]\n            + potential_discharge_accum_m3_s[i];\n        if !accumulated.is_finite() || accumulated > f32::MAX as f64 {\n            return Err(\"runoff accumulated discharge exceeds representable range\");\n        }\n        potential_discharge_accum_m3_s[ri] = accumulated;",
)
replace_once(
    "rust/interlink-worldgen/src/runoff.rs",
    "        let discharge = f64::from(potential_discharge_m3_s[i]);",
    "        let discharge = potential_discharge_accum_m3_s[i];",
)
replace_once(
    "rust/interlink-worldgen/src/runoff.rs",
    "    Ok(RunoffCore {\n        actual_evapotranspiration_mm,",
    "    let potential_discharge_m3_s = potential_discharge_accum_m3_s\n        .into_iter()\n        .map(|value| value as f32)\n        .collect();\n\n    Ok(RunoffCore {\n        actual_evapotranspiration_mm,",
)

# Extend the cumulative climate bridge so the primary Lab runs WG-5, WG-6A and
# WG-6B once, without a second climate solve or a second WG-4 construction.
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "    build_icosphere, generate_coupled_climate_with_diagnostics, generate_crust_and_history,\n    generate_initial_topography, generate_lithosphere, generate_tectonics,\n    inherit_boundary_interfaces, inherit_physical_state, ClimatePhysicalParameters, ClimateRequest,\n    ClimateState, GeodesicTopology, GeologyRequest, InheritedBoundarySet, InheritedPhysicalState,\n    LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,\n    TopographyState, WORLDGEN_ENGINE_VERSION,",
    "    build_icosphere, generate_coupled_climate_with_diagnostics, generate_crust_and_history,\n    generate_drainage_topology, generate_initial_topography, generate_lithosphere,\n    generate_runoff_discharge, generate_tectonics, inherit_boundary_interfaces,\n    inherit_physical_state, ClimatePhysicalParameters, ClimateRequest, ClimateState,\n    DrainageRequest, DrainageState, GeodesicTopology, GeologyRequest, InheritedBoundarySet,\n    InheritedPhysicalState, LithosphereRequest, PlanetPhysicalParameters, RunoffRequest,\n    RunoffState, TectonicsRequest, TopographyRequest, TopographyState, WORLDGEN_ENGINE_VERSION,",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "const GENERATION_STAGE_COUNT: u32 = 10;",
    "const GENERATION_STAGE_COUNT: u32 = 12;",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "    terrain: TopographyState,\n    climate: ClimateState,",
    "    terrain: TopographyState,\n    climate: ClimateState,\n    drainage: DrainageState,\n    runoff: RunoffState,",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "        let climate_request = ClimateRequest::new(seed);",
    "        let climate_request = ClimateRequest::new(seed.as_str());",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;\n\n        Ok(Self {",
    "        let precipitation_phase_rate_mm_year = diagnostics.precipitation_phase_rate_mm_year;\n\n        report_generation_progress(progress, \"drainage-topology\", 9, 0, 1);\n        let drainage = generate_drainage_topology(\n            &fine_topology,\n            &terrain,\n            planet,\n            &DrainageRequest::new(seed.as_str()),\n        )\n        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"drainage-topology\", 9, 1, 1);\n\n        report_generation_progress(progress, \"runoff-discharge\", 10, 0, 1);\n        let runoff = generate_runoff_discharge(\n            &fine_topology,\n            &terrain,\n            &climate,\n            &drainage,\n            planet,\n            &RunoffRequest::new(seed),\n        )\n        .map_err(|error| JsValue::from_str(&error.to_string()))?;\n        report_generation_progress(progress, \"runoff-discharge\", 10, 1, 1);\n\n        Ok(Self {",
)
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "            terrain,\n            climate,\n            planet,",
    "            terrain,\n            climate,\n            drainage,\n            runoff,\n            planet,",
)

hydrology_methods = r'''

    pub fn drainage_stage_id(&self) -> String { self.drainage.stage.id.to_owned() }
    pub fn drainage_stage_version(&self) -> u32 { self.drainage.stage.version }
    pub fn drainage_stage_seed_hex(&self) -> String { format!("{:016x}", self.drainage.stage.derived_seed) }
    pub fn drainage_hash_hex(&self) -> String { self.drainage.metrics.drainage_hash_hex() }
    pub fn drainage_land_sample_count(&self) -> u32 { self.drainage.metrics.land_sample_count }
    pub fn drainage_ocean_sample_count(&self) -> u32 { self.drainage.metrics.ocean_sample_count }
    pub fn drainage_basin_count(&self) -> u32 { self.drainage.metrics.basin_count }
    pub fn drainage_depression_count(&self) -> u32 { self.drainage.metrics.depression_count }
    pub fn drainage_depression_sample_count(&self) -> u32 { self.drainage.metrics.depression_sample_count }
    pub fn drainage_land_area_m2(&self) -> f64 { self.drainage.metrics.land_area_m2 }
    pub fn terminal_contributing_area_m2(&self) -> f64 { self.drainage.metrics.terminal_contributing_area_m2 }
    pub fn drainage_area_conservation_relative_error(&self) -> f64 { self.drainage.metrics.area_conservation_relative_error }
    pub fn maximum_contributing_area_m2(&self) -> f64 { self.drainage.metrics.maximum_contributing_area_m2 }
    pub fn maximum_depression_depth_m(&self) -> f64 { self.drainage.metrics.maximum_depression_depth_m }
    pub fn receiver(&self) -> Vec<u32> { self.drainage.receiver.clone() }
    pub fn outlet_sample(&self) -> Vec<u32> { self.drainage.outlet_sample.clone() }
    pub fn outlet_kind(&self) -> Vec<u8> { self.drainage.outlet_kind.clone() }
    pub fn basin_id(&self) -> Vec<u32> { self.drainage.basin_id.clone() }
    pub fn depression_id(&self) -> Vec<u32> { self.drainage.depression_id.clone() }
    pub fn hydrologic_escape_elevation_m(&self) -> Vec<f32> { self.drainage.hydrologic_escape_elevation_m.clone() }
    pub fn depression_depth_m(&self) -> Vec<f32> { self.drainage.depression_depth_m.clone() }
    pub fn contributing_area_m2(&self) -> Vec<f64> { self.drainage.contributing_area_m2.clone() }
    pub fn drainage_order(&self) -> Vec<u32> { self.drainage.drainage_order.clone() }
    pub fn basin_outlet_samples(&self) -> Vec<u32> { self.drainage.basins.iter().map(|basin| basin.outlet_sample).collect() }
    pub fn basin_outlet_kinds(&self) -> Vec<u8> { self.drainage.basins.iter().map(|basin| basin.outlet_kind).collect() }
    pub fn basin_areas_m2(&self) -> Vec<f64> { self.drainage.basins.iter().map(|basin| basin.area_m2).collect() }
    pub fn depression_floor_samples(&self) -> Vec<u32> { self.drainage.depressions.iter().map(|depression| depression.floor_sample).collect() }
    pub fn depression_floor_elevations_m(&self) -> Vec<f64> { self.drainage.depressions.iter().map(|depression| depression.floor_elevation_m).collect() }
    pub fn depression_spill_elevations_m(&self) -> Vec<f64> { self.drainage.depressions.iter().map(|depression| depression.spill_elevation_m).collect() }
    pub fn depression_areas_m2(&self) -> Vec<f64> { self.drainage.depressions.iter().map(|depression| depression.area_m2).collect() }

    pub fn runoff_stage_id(&self) -> String { self.runoff.stage.id.to_owned() }
    pub fn runoff_stage_version(&self) -> u32 { self.runoff.stage.version }
    pub fn runoff_stage_seed_hex(&self) -> String { format!("{:016x}", self.runoff.stage.derived_seed) }
    pub fn runoff_hash_hex(&self) -> String { self.runoff.metrics.runoff_hash_hex() }
    pub fn runoff_parameter_hash_hex(&self) -> String { self.runoff.metrics.runoff_parameter_hash_hex() }
    pub fn runoff_climate_hash_hex(&self) -> String { self.runoff.metrics.climate_hash_hex() }
    pub fn runoff_drainage_hash_hex(&self) -> String { self.runoff.metrics.drainage_hash_hex() }
    pub fn mean_land_runoff_precipitation_mm(&self) -> f64 { self.runoff.metrics.mean_land_precipitation_mm }
    pub fn mean_land_actual_evapotranspiration_mm(&self) -> f64 { self.runoff.metrics.mean_land_actual_evapotranspiration_mm }
    pub fn mean_land_runoff_mm(&self) -> f64 { self.runoff.metrics.mean_land_runoff_mm }
    pub fn maximum_land_runoff_mm(&self) -> f64 { self.runoff.metrics.maximum_land_runoff_mm }
    pub fn land_runoff_fraction(&self) -> f64 { self.runoff.metrics.land_runoff_fraction }
    pub fn total_local_runoff_m3_s(&self) -> f64 { self.runoff.metrics.total_local_runoff_m3_s }
    pub fn terminal_discharge_m3_s(&self) -> f64 { self.runoff.metrics.terminal_discharge_m3_s }
    pub fn discharge_conservation_relative_error(&self) -> f64 { self.runoff.metrics.discharge_conservation_relative_error }
    pub fn maximum_potential_discharge_m3_s(&self) -> f64 { self.runoff.metrics.maximum_potential_discharge_m3_s }
    pub fn actual_evapotranspiration_mm(&self) -> Vec<f32> { self.runoff.actual_evapotranspiration_mm.clone() }
    pub fn local_runoff_mm(&self) -> Vec<f32> { self.runoff.local_runoff_mm.clone() }
    pub fn runoff_fraction(&self) -> Vec<f32> { self.runoff.runoff_fraction.clone() }
    pub fn local_runoff_m3_s(&self) -> Vec<f32> { self.runoff.local_runoff_m3_s.clone() }
    pub fn potential_discharge_m3_s(&self) -> Vec<f32> { self.runoff.potential_discharge_m3_s.clone() }
'''
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    "    pub fn sea_ice_potential(&self) -> Vec<f32> {\n        self.climate.sea_ice_potential.clone()\n    }\n}",
    "    pub fn sea_ice_potential(&self) -> Vec<f32> {\n        self.climate.sea_ice_potential.clone()\n    }" + hydrology_methods + "\n}",
)

# Browser/WASM package contract changes with the cumulative result surface.
replace_once(
    "rust/interlink-worldgen-wasm/src/lib.rs",
    "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 11;",
    "pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 12;",
)
