/* @ts-self-types="./interlink_worldgen_wasm.d.ts" */

export class WasmWorldgenDiagnostic {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmWorldgenDiagnosticFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmworldgendiagnostic_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    field_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgendiagnostic_field_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    generator_version() {
        const ret = wasm.wasmworldgendiagnostic_generator_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    height() {
        const ret = wasm.wasmworldgendiagnostic_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    maximum() {
        const ret = wasm.wasmworldgendiagnostic_maximum(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean() {
        const ret = wasm.wasmworldgendiagnostic_mean(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    minimum() {
        const ret = wasm.wasmworldgendiagnostic_minimum(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {string} seed
     * @param {number} width
     * @param {number} height
     */
    constructor(seed, width, height) {
        const ptr0 = passStringToWasm0(seed, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmworldgendiagnostic_new(ptr0, len0, width, height);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        WasmWorldgenDiagnosticFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {bigint}
     */
    sample_count() {
        const ret = wasm.wasmworldgendiagnostic_sample_count(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @returns {string}
     */
    stage_id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgendiagnostic_stage_id(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    stage_seed_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgendiagnostic_stage_seed_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    stage_version() {
        const ret = wasm.wasmworldgendiagnostic_stage_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint16Array}
     */
    values() {
        const ret = wasm.wasmworldgendiagnostic_values(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {number}
     */
    width() {
        const ret = wasm.wasmworldgendiagnostic_width(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) WasmWorldgenDiagnostic.prototype[Symbol.dispose] = WasmWorldgenDiagnostic.prototype.free;

export class WasmWorldgenGeology {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmWorldgenGeologyFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmworldgengeology_free(ptr, 0);
    }
    /**
     * @returns {Float32Array}
     */
    basin_potential() {
        const ret = wasm.wasmworldgengeology_basin_potential(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    boundary_edge_count() {
        const ret = wasm.wasmworldgengeology_boundary_edge_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint8Array}
     */
    boundary_kinds() {
        const ret = wasm.wasmworldgengeology_boundary_kinds(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Uint16Array}
     */
    boundary_plate_ids() {
        const ret = wasm.wasmworldgengeology_boundary_plate_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    boundary_samples() {
        const ret = wasm.wasmworldgengeology_boundary_samples(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    buoyancy_index() {
        const ret = wasm.wasmworldgengeology_buoyancy_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    continental_area_fraction() {
        const ret = wasm.wasmworldgengeology_continental_area_fraction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    continental_collision_edges() {
        const ret = wasm.wasmworldgengeology_continental_collision_edges(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    continental_rift_edges() {
        const ret = wasm.wasmworldgengeology_continental_rift_edges(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Float32Array}
     */
    crust_age_myr() {
        const ret = wasm.wasmworldgengeology_crust_age_myr(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    crust_density_kg_per_m3() {
        const ret = wasm.wasmworldgengeology_crust_density_kg_per_m3(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint8Array}
     */
    crust_kind() {
        const ret = wasm.wasmworldgengeology_crust_kind(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Uint16Array}
     */
    crust_province_id() {
        const ret = wasm.wasmworldgengeology_crust_province_id(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    crust_thickness_km() {
        const ret = wasm.wasmworldgengeology_crust_thickness_km(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    crustal_strain() {
        const ret = wasm.wasmworldgengeology_crustal_strain(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    faces() {
        const ret = wasm.wasmworldgengeology_faces(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    generator_version() {
        const ret = wasm.wasmworldgengeology_generator_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint8Array}
     */
    geological_boundary_regimes() {
        const ret = wasm.wasmworldgengeology_geological_boundary_regimes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {string}
     */
    geology_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgengeology_geology_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    history_seed_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgengeology_history_seed_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    level() {
        const ret = wasm.wasmworldgengeology_level(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_continental_age_myr() {
        const ret = wasm.wasmworldgengeology_mean_continental_age_myr(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_continental_thickness_km() {
        const ret = wasm.wasmworldgengeology_mean_continental_thickness_km(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_oceanic_age_myr() {
        const ret = wasm.wasmworldgengeology_mean_oceanic_age_myr(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_oceanic_thickness_km() {
        const ret = wasm.wasmworldgengeology_mean_oceanic_thickness_km(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint32Array}
     */
    neighbor_offsets() {
        const ret = wasm.wasmworldgengeology_neighbor_offsets(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    neighbors() {
        const ret = wasm.wasmworldgengeology_neighbors(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string} seed
     * @param {number} level
     * @param {number} plate_count
     */
    constructor(seed, level, plate_count) {
        const ptr0 = passStringToWasm0(seed, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmworldgengeology_new(ptr0, len0, level, plate_count);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        WasmWorldgenGeologyFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {number}
     */
    ocean_continent_subduction_edges() {
        const ret = wasm.wasmworldgengeology_ocean_continent_subduction_edges(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    oceanic_area_fraction() {
        const ret = wasm.wasmworldgengeology_oceanic_area_fraction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    oceanic_ridge_edges() {
        const ret = wasm.wasmworldgengeology_oceanic_ridge_edges(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    oceanic_subduction_edges() {
        const ret = wasm.wasmworldgengeology_oceanic_subduction_edges(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Float32Array}
     */
    orogenic_history() {
        const ret = wasm.wasmworldgengeology_orogenic_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    plate_continental_fractions() {
        const ret = wasm.wasmworldgengeology_plate_continental_fractions(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {number}
     */
    plate_count() {
        const ret = wasm.wasmworldgengeology_plate_count(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint16Array}
     */
    plate_ids() {
        const ret = wasm.wasmworldgengeology_plate_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    plate_mean_crust_age_myr() {
        const ret = wasm.wasmworldgengeology_plate_mean_crust_age_myr(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    plate_mean_crust_thickness_km() {
        const ret = wasm.wasmworldgengeology_plate_mean_crust_thickness_km(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    plate_oceanic_fractions() {
        const ret = wasm.wasmworldgengeology_plate_oceanic_fractions(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Uint8Array}
     */
    plate_scale_classes() {
        const ret = wasm.wasmworldgengeology_plate_scale_classes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    plate_transitional_fractions() {
        const ret = wasm.wasmworldgengeology_plate_transitional_fractions(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    positions() {
        const ret = wasm.wasmworldgengeology_positions(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {string}
     */
    property_seed_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgengeology_property_seed_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    province_seed_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgengeology_province_seed_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Float32Array}
     */
    ridge_history() {
        const ret = wasm.wasmworldgengeology_ridge_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    rift_history() {
        const ret = wasm.wasmworldgengeology_rift_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    sample_count() {
        const ret = wasm.wasmworldgengeology_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    stage_id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgengeology_stage_id(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    stage_seed_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgengeology_stage_seed_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    stage_version() {
        const ret = wasm.wasmworldgengeology_stage_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Float32Array}
     */
    subduction_history() {
        const ret = wasm.wasmworldgengeology_subduction_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint8Array}
     */
    subduction_polarities() {
        const ret = wasm.wasmworldgengeology_subduction_polarities(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    subsidence_history() {
        const ret = wasm.wasmworldgengeology_subsidence_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string}
     */
    tectonic_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgengeology_tectonic_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    topology_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgengeology_topology_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    transform_edges() {
        const ret = wasm.wasmworldgengeology_transform_edges(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Float32Array}
     */
    transform_history() {
        const ret = wasm.wasmworldgengeology_transform_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    transitional_area_fraction() {
        const ret = wasm.wasmworldgengeology_transitional_area_fraction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    transitional_divergence_edges() {
        const ret = wasm.wasmworldgengeology_transitional_divergence_edges(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Float32Array}
     */
    trench_history() {
        const ret = wasm.wasmworldgengeology_trench_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    volcanic_arc_history() {
        const ret = wasm.wasmworldgengeology_volcanic_arc_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}
if (Symbol.dispose) WasmWorldgenGeology.prototype[Symbol.dispose] = WasmWorldgenGeology.prototype.free;

export class WasmWorldgenInheritance {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmWorldgenInheritanceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmworldgeninheritance_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    added_sample_count() {
        const ret = wasm.wasmworldgeninheritance_added_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Float32Array}
     */
    basin_potential() {
        const ret = wasm.wasmworldgeninheritance_basin_potential(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    boundary_coarse_source_indices() {
        const ret = wasm.wasmworldgeninheritance_boundary_coarse_source_indices(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string}
     */
    boundary_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgeninheritance_boundary_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Uint8Array}
     */
    boundary_kinds() {
        const ret = wasm.wasmworldgeninheritance_boundary_kinds(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    boundary_normal_rates_m_per_year() {
        const ret = wasm.wasmworldgeninheritance_boundary_normal_rates_m_per_year(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    boundary_samples() {
        const ret = wasm.wasmworldgeninheritance_boundary_samples(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    boundary_shear_rates_m_per_year() {
        const ret = wasm.wasmworldgeninheritance_boundary_shear_rates_m_per_year(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    buoyancy_index() {
        const ret = wasm.wasmworldgeninheritance_buoyancy_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    coarse_level() {
        const ret = wasm.wasmworldgeninheritance_coarse_level(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    coarse_sample_count() {
        const ret = wasm.wasmworldgeninheritance_coarse_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    coarse_topology_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgeninheritance_coarse_topology_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Float32Array}
     */
    compensated_buoyancy_index() {
        const ret = wasm.wasmworldgeninheritance_compensated_buoyancy_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    crust_age_myr() {
        const ret = wasm.wasmworldgeninheritance_crust_age_myr(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    crust_density_kg_per_m3() {
        const ret = wasm.wasmworldgeninheritance_crust_density_kg_per_m3(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint8Array}
     */
    crust_kind() {
        const ret = wasm.wasmworldgeninheritance_crust_kind(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Uint16Array}
     */
    crust_province_id() {
        const ret = wasm.wasmworldgeninheritance_crust_province_id(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    crust_thickness_km() {
        const ret = wasm.wasmworldgeninheritance_crust_thickness_km(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    crustal_strain() {
        const ret = wasm.wasmworldgeninheritance_crustal_strain(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    effective_elastic_thickness_km() {
        const ret = wasm.wasmworldgeninheritance_effective_elastic_thickness_km(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    equivalent_global_water_depth_m() {
        const ret = wasm.wasmworldgeninheritance_equivalent_global_water_depth_m(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint32Array}
     */
    faces() {
        const ret = wasm.wasmworldgeninheritance_faces(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    fine_boundary_edge_count() {
        const ret = wasm.wasmworldgeninheritance_fine_boundary_edge_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    fine_level() {
        const ret = wasm.wasmworldgeninheritance_fine_level(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    fine_sample_count() {
        const ret = wasm.wasmworldgeninheritance_fine_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    fine_topology_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgeninheritance_fine_topology_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Uint16Array}
     */
    fragment_ids() {
        const ret = wasm.wasmworldgeninheritance_fragment_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    fragmentation_propensity() {
        const ret = wasm.wasmworldgeninheritance_fragmentation_propensity(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    generator_version() {
        const ret = wasm.wasmworldgeninheritance_generator_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint8Array}
     */
    geological_boundary_regimes() {
        const ret = wasm.wasmworldgeninheritance_geological_boundary_regimes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {string}
     */
    geology_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgeninheritance_geology_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    inheritance_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgeninheritance_inheritance_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Uint8Array}
     */
    inherited_sample_mask() {
        const ret = wasm.wasmworldgeninheritance_inherited_sample_mask(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {number}
     */
    internal_heat_flux_w_per_m2() {
        const ret = wasm.wasmworldgeninheritance_internal_heat_flux_w_per_m2(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    isostatic_mantle_density_kg_per_m3() {
        const ret = wasm.wasmworldgeninheritance_isostatic_mantle_density_kg_per_m3(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint16Array}
     */
    kinematic_domain_ids() {
        const ret = wasm.wasmworldgeninheritance_kinematic_domain_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {string}
     */
    lithosphere_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgeninheritance_lithosphere_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Float32Array}
     */
    mantle_dynamic_support_index() {
        const ret = wasm.wasmworldgeninheritance_mantle_dynamic_support_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    mantle_thermal_expansivity_per_k() {
        const ret = wasm.wasmworldgeninheritance_mantle_thermal_expansivity_per_k(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Float32Array}
     */
    mantle_upwelling_index() {
        const ret = wasm.wasmworldgeninheritance_mantle_upwelling_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    nearest_coarse_source() {
        const ret = wasm.wasmworldgeninheritance_nearest_coarse_source(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    neighbor_offsets() {
        const ret = wasm.wasmworldgeninheritance_neighbor_offsets(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    neighbors() {
        const ret = wasm.wasmworldgeninheritance_neighbors(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string} seed
     * @param {number} coarse_level
     * @param {number} fine_level
     * @param {number} plate_count
     */
    constructor(seed, coarse_level, fine_level, plate_count) {
        const ptr0 = passStringToWasm0(seed, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmworldgeninheritance_new(ptr0, len0, coarse_level, fine_level, plate_count);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        WasmWorldgenInheritanceFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {number}
     */
    ocean_water_density_kg_per_m3() {
        const ret = wasm.wasmworldgeninheritance_ocean_water_density_kg_per_m3(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Float32Array}
     */
    orogenic_history() {
        const ret = wasm.wasmworldgeninheritance_orogenic_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string}
     */
    parameter_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgeninheritance_parameter_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    plate_count() {
        const ret = wasm.wasmworldgeninheritance_plate_count(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint16Array}
     */
    plate_ids() {
        const ret = wasm.wasmworldgeninheritance_plate_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    positions() {
        const ret = wasm.wasmworldgeninheritance_positions(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {string}
     */
    provenance_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgeninheritance_provenance_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    radius_m() {
        const ret = wasm.wasmworldgeninheritance_radius_m(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Float32Array}
     */
    ridge_history() {
        const ret = wasm.wasmworldgeninheritance_ridge_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    rift_history() {
        const ret = wasm.wasmworldgeninheritance_rift_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string}
     */
    stage_id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgeninheritance_stage_id(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    stage_version() {
        const ret = wasm.wasmworldgeninheritance_stage_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Float32Array}
     */
    strength_index() {
        const ret = wasm.wasmworldgeninheritance_strength_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    structural_fabric_strength() {
        const ret = wasm.wasmworldgeninheritance_structural_fabric_strength(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint8Array}
     */
    structural_zone_kind() {
        const ret = wasm.wasmworldgeninheritance_structural_zone_kind(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    subduction_history() {
        const ret = wasm.wasmworldgeninheritance_subduction_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint8Array}
     */
    subduction_polarities() {
        const ret = wasm.wasmworldgeninheritance_subduction_polarities(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    subsidence_history() {
        const ret = wasm.wasmworldgeninheritance_subsidence_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    surface_gravity_m_s2() {
        const ret = wasm.wasmworldgeninheritance_surface_gravity_m_s2(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    surface_water_mass_kg() {
        const ret = wasm.wasmworldgeninheritance_surface_water_mass_kg(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    tectonic_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgeninheritance_tectonic_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Float32Array}
     */
    thermal_anomaly_index() {
        const ret = wasm.wasmworldgeninheritance_thermal_anomaly_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    transform_history() {
        const ret = wasm.wasmworldgeninheritance_transform_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    trench_history() {
        const ret = wasm.wasmworldgeninheritance_trench_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    volcanic_arc_history() {
        const ret = wasm.wasmworldgeninheritance_volcanic_arc_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    weakness_index() {
        const ret = wasm.wasmworldgeninheritance_weakness_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}
if (Symbol.dispose) WasmWorldgenInheritance.prototype[Symbol.dispose] = WasmWorldgenInheritance.prototype.free;

export class WasmWorldgenLithosphere {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmWorldgenLithosphereFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmworldgenlithosphere_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    boundary_edge_count() {
        const ret = wasm.wasmworldgenlithosphere_boundary_edge_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint8Array}
     */
    boundary_kinds() {
        const ret = wasm.wasmworldgenlithosphere_boundary_kinds(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    boundary_samples() {
        const ret = wasm.wasmworldgenlithosphere_boundary_samples(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    compensated_buoyancy_index() {
        const ret = wasm.wasmworldgenlithosphere_compensated_buoyancy_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    continental_margin_sample_count() {
        const ret = wasm.wasmworldgenlithosphere_continental_margin_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint8Array}
     */
    crust_kind() {
        const ret = wasm.wasmworldgenlithosphere_crust_kind(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    crustal_strain() {
        const ret = wasm.wasmworldgenlithosphere_crustal_strain(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    effective_elastic_thickness_km() {
        const ret = wasm.wasmworldgenlithosphere_effective_elastic_thickness_km(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    faces() {
        const ret = wasm.wasmworldgenlithosphere_faces(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    fragment_angular_velocities_rad_per_myr() {
        const ret = wasm.wasmworldgenlithosphere_fragment_angular_velocities_rad_per_myr(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    fragment_area_fractions_of_parent() {
        const ret = wasm.wasmworldgenlithosphere_fragment_area_fractions_of_parent(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    fragment_area_steradians() {
        const ret = wasm.wasmworldgenlithosphere_fragment_area_steradians(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Uint16Array}
     */
    fragment_ids() {
        const ret = wasm.wasmworldgenlithosphere_fragment_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Uint8Array}
     */
    fragment_kinds() {
        const ret = wasm.wasmworldgenlithosphere_fragment_kinds(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    fragment_mean_propensity() {
        const ret = wasm.wasmworldgenlithosphere_fragment_mean_propensity(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    fragment_mean_weakness() {
        const ret = wasm.wasmworldgenlithosphere_fragment_mean_weakness(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Uint16Array}
     */
    fragment_parent_plate_ids() {
        const ret = wasm.wasmworldgenlithosphere_fragment_parent_plate_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    fragment_seed_samples() {
        const ret = wasm.wasmworldgenlithosphere_fragment_seed_samples(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    fragmentation_propensity() {
        const ret = wasm.wasmworldgenlithosphere_fragmentation_propensity(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    fragmented_area_fraction() {
        const ret = wasm.wasmworldgenlithosphere_fragmented_area_fraction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    generator_version() {
        const ret = wasm.wasmworldgenlithosphere_generator_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint8Array}
     */
    geological_boundary_regimes() {
        const ret = wasm.wasmworldgenlithosphere_geological_boundary_regimes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {string}
     */
    geology_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgenlithosphere_geology_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Uint16Array}
     */
    kinematic_domain_ids() {
        const ret = wasm.wasmworldgenlithosphere_kinematic_domain_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {number}
     */
    level() {
        const ret = wasm.wasmworldgenlithosphere_level(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    lithosphere_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgenlithosphere_lithosphere_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Float32Array}
     */
    mantle_dynamic_support_index() {
        const ret = wasm.wasmworldgenlithosphere_mantle_dynamic_support_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string}
     */
    mantle_seed_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgenlithosphere_mantle_seed_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Float32Array}
     */
    mantle_upwelling_index() {
        const ret = wasm.wasmworldgenlithosphere_mantle_upwelling_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    mean_dynamic_support_index() {
        const ret = wasm.wasmworldgenlithosphere_mean_dynamic_support_index(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_effective_elastic_thickness_km() {
        const ret = wasm.wasmworldgenlithosphere_mean_effective_elastic_thickness_km(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_mantle_upwelling_index() {
        const ret = wasm.wasmworldgenlithosphere_mean_mantle_upwelling_index(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_strength_index() {
        const ret = wasm.wasmworldgenlithosphere_mean_strength_index(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_weakness_index() {
        const ret = wasm.wasmworldgenlithosphere_mean_weakness_index(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    mechanical_seed_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgenlithosphere_mechanical_seed_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    microplate_count() {
        const ret = wasm.wasmworldgenlithosphere_microplate_count(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint32Array}
     */
    neighbor_offsets() {
        const ret = wasm.wasmworldgenlithosphere_neighbor_offsets(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    neighbors() {
        const ret = wasm.wasmworldgenlithosphere_neighbors(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string} seed
     * @param {number} level
     * @param {number} plate_count
     */
    constructor(seed, level, plate_count) {
        const ptr0 = passStringToWasm0(seed, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmworldgenlithosphere_new(ptr0, len0, level, plate_count);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        WasmWorldgenLithosphereFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {Float32Array}
     */
    orogenic_history() {
        const ret = wasm.wasmworldgenlithosphere_orogenic_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    plate_count() {
        const ret = wasm.wasmworldgenlithosphere_plate_count(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint16Array}
     */
    plate_ids() {
        const ret = wasm.wasmworldgenlithosphere_plate_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    positions() {
        const ret = wasm.wasmworldgenlithosphere_positions(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {string}
     */
    refinement_seed_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgenlithosphere_refinement_seed_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Float32Array}
     */
    ridge_history() {
        const ret = wasm.wasmworldgenlithosphere_ridge_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    rift_history() {
        const ret = wasm.wasmworldgenlithosphere_rift_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    rift_zone_sample_count() {
        const ret = wasm.wasmworldgenlithosphere_rift_zone_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    sample_count() {
        const ret = wasm.wasmworldgenlithosphere_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    stage_id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgenlithosphere_stage_id(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    stage_seed_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgenlithosphere_stage_seed_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    stage_version() {
        const ret = wasm.wasmworldgenlithosphere_stage_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Float32Array}
     */
    strength_index() {
        const ret = wasm.wasmworldgenlithosphere_strength_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    structural_fabric_strength() {
        const ret = wasm.wasmworldgenlithosphere_structural_fabric_strength(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint8Array}
     */
    structural_zone_kind() {
        const ret = wasm.wasmworldgenlithosphere_structural_zone_kind(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    subduction_history() {
        const ret = wasm.wasmworldgenlithosphere_subduction_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    suture_sample_count() {
        const ret = wasm.wasmworldgenlithosphere_suture_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    tectonic_fragment_count() {
        const ret = wasm.wasmworldgenlithosphere_tectonic_fragment_count(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    tectonic_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgenlithosphere_tectonic_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    terrane_count() {
        const ret = wasm.wasmworldgenlithosphere_terrane_count(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Float32Array}
     */
    thermal_anomaly_index() {
        const ret = wasm.wasmworldgenlithosphere_thermal_anomaly_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string}
     */
    topology_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgenlithosphere_topology_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Float32Array}
     */
    transform_history() {
        const ret = wasm.wasmworldgenlithosphere_transform_history(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    transform_zone_sample_count() {
        const ret = wasm.wasmworldgenlithosphere_transform_zone_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Float32Array}
     */
    weakness_index() {
        const ret = wasm.wasmworldgenlithosphere_weakness_index(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}
if (Symbol.dispose) WasmWorldgenLithosphere.prototype[Symbol.dispose] = WasmWorldgenLithosphere.prototype.free;

export class WasmWorldgenTectonics {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmWorldgenTectonicsFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmworldgentectonics_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    boundary_edge_count() {
        const ret = wasm.wasmworldgentectonics_boundary_edge_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint8Array}
     */
    boundary_kinds() {
        const ret = wasm.wasmworldgentectonics_boundary_kinds(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    boundary_normal_rates_m_per_year() {
        const ret = wasm.wasmworldgentectonics_boundary_normal_rates_m_per_year(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Uint16Array}
     */
    boundary_plate_ids() {
        const ret = wasm.wasmworldgentectonics_boundary_plate_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    boundary_samples() {
        const ret = wasm.wasmworldgentectonics_boundary_samples(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    boundary_shear_rates_m_per_year() {
        const ret = wasm.wasmworldgentectonics_boundary_shear_rates_m_per_year(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {number}
     */
    convergent_edge_count() {
        const ret = wasm.wasmworldgentectonics_convergent_edge_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    divergent_edge_count() {
        const ret = wasm.wasmworldgentectonics_divergent_edge_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint32Array}
     */
    faces() {
        const ret = wasm.wasmworldgentectonics_faces(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    generator_version() {
        const ret = wasm.wasmworldgentectonics_generator_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    level() {
        const ret = wasm.wasmworldgentectonics_level(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    maximum_plate_area_fraction() {
        const ret = wasm.wasmworldgentectonics_maximum_plate_area_fraction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_plate_area_fraction() {
        const ret = wasm.wasmworldgentectonics_mean_plate_area_fraction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_reference_speed_mm_per_year() {
        const ret = wasm.wasmworldgentectonics_mean_reference_speed_mm_per_year(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    minimum_plate_area_fraction() {
        const ret = wasm.wasmworldgentectonics_minimum_plate_area_fraction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    minimum_seed_separation_rad() {
        const ret = wasm.wasmworldgentectonics_minimum_seed_separation_rad(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint32Array}
     */
    neighbor_offsets() {
        const ret = wasm.wasmworldgentectonics_neighbor_offsets(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    neighbors() {
        const ret = wasm.wasmworldgentectonics_neighbors(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string} seed
     * @param {number} level
     * @param {number} plate_count
     */
    constructor(seed, level, plate_count) {
        const ptr0 = passStringToWasm0(seed, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmworldgentectonics_new(ptr0, len0, level, plate_count);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        WasmWorldgenTectonicsFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {Float64Array}
     */
    plate_angular_velocities_rad_per_myr() {
        const ret = wasm.wasmworldgentectonics_plate_angular_velocities_rad_per_myr(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    plate_area_steradians() {
        const ret = wasm.wasmworldgentectonics_plate_area_steradians(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {number}
     */
    plate_count() {
        const ret = wasm.wasmworldgentectonics_plate_count(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Float64Array}
     */
    plate_euler_poles() {
        const ret = wasm.wasmworldgentectonics_plate_euler_poles(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Uint16Array}
     */
    plate_ids() {
        const ret = wasm.wasmworldgentectonics_plate_ids(this.__wbg_ptr);
        var v1 = getArrayU16FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 2, 2);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    plate_seed_samples() {
        const ret = wasm.wasmworldgentectonics_plate_seed_samples(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    positions() {
        const ret = wasm.wasmworldgentectonics_positions(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {number}
     */
    sample_count() {
        const ret = wasm.wasmworldgentectonics_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    stage_id() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgentectonics_stage_id(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    stage_seed_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgentectonics_stage_seed_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    stage_version() {
        const ret = wasm.wasmworldgentectonics_stage_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    tectonic_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgentectonics_tectonic_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    topology_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgentectonics_topology_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    transform_edge_count() {
        const ret = wasm.wasmworldgentectonics_transform_edge_count(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) WasmWorldgenTectonics.prototype[Symbol.dispose] = WasmWorldgenTectonics.prototype.free;

export class WasmWorldgenTopology {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmWorldgenTopologyFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmworldgentopology_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    area_coefficient_of_variation() {
        const ret = wasm.wasmworldgentopology_area_coefficient_of_variation(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Float64Array}
     */
    area_steradians() {
        const ret = wasm.wasmworldgentopology_area_steradians(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Uint8Array}
     */
    birth_levels() {
        const ret = wasm.wasmworldgentopology_birth_levels(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {number}
     */
    edge_coefficient_of_variation() {
        const ret = wasm.wasmworldgentopology_edge_coefficient_of_variation(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    edge_count() {
        const ret = wasm.wasmworldgentopology_edge_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    face_count() {
        const ret = wasm.wasmworldgentopology_face_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Uint32Array}
     */
    faces() {
        const ret = wasm.wasmworldgentopology_faces(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    five_neighbor_count() {
        const ret = wasm.wasmworldgentopology_five_neighbor_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    generator_version() {
        const ret = wasm.wasmworldgentopology_generator_version(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    interface_coefficient_of_variation() {
        const ret = wasm.wasmworldgentopology_interface_coefficient_of_variation(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    level() {
        const ret = wasm.wasmworldgentopology_level(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    maximum_area_steradians() {
        const ret = wasm.wasmworldgentopology_maximum_area_steradians(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    maximum_edge_arc_radians() {
        const ret = wasm.wasmworldgentopology_maximum_edge_arc_radians(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    maximum_interface_arc_radians() {
        const ret = wasm.wasmworldgentopology_maximum_interface_arc_radians(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_area_steradians() {
        const ret = wasm.wasmworldgentopology_mean_area_steradians(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_edge_arc_radians() {
        const ret = wasm.wasmworldgentopology_mean_edge_arc_radians(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_interface_arc_radians() {
        const ret = wasm.wasmworldgentopology_mean_interface_arc_radians(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    minimum_area_steradians() {
        const ret = wasm.wasmworldgentopology_minimum_area_steradians(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    minimum_edge_arc_radians() {
        const ret = wasm.wasmworldgentopology_minimum_edge_arc_radians(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    minimum_interface_arc_radians() {
        const ret = wasm.wasmworldgentopology_minimum_interface_arc_radians(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Float64Array}
     */
    neighbor_arc_lengths_rad() {
        const ret = wasm.wasmworldgentopology_neighbor_arc_lengths_rad(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    neighbor_interface_arc_lengths_rad() {
        const ret = wasm.wasmworldgentopology_neighbor_interface_arc_lengths_rad(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    neighbor_offsets() {
        const ret = wasm.wasmworldgentopology_neighbor_offsets(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Uint32Array}
     */
    neighbors() {
        const ret = wasm.wasmworldgentopology_neighbors(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {number} level
     */
    constructor(level) {
        const ret = wasm.wasmworldgentopology_new(level);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        WasmWorldgenTopologyFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {Uint32Array}
     */
    parent_edges() {
        const ret = wasm.wasmworldgentopology_parent_edges(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float64Array}
     */
    positions() {
        const ret = wasm.wasmworldgentopology_positions(this.__wbg_ptr);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {number}
     */
    sample_count() {
        const ret = wasm.wasmworldgentopology_sample_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    six_neighbor_count() {
        const ret = wasm.wasmworldgentopology_six_neighbor_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    topology_hash_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmworldgentopology_topology_hash_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    total_area_steradians() {
        const ret = wasm.wasmworldgentopology_total_area_steradians(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) WasmWorldgenTopology.prototype[Symbol.dispose] = WasmWorldgenTopology.prototype.free;

/**
 * @returns {number}
 */
export function worldgen_engine_version() {
    const ret = wasm.worldgen_engine_version();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function worldgen_protocol_version() {
    const ret = wasm.worldgen_protocol_version();
    return ret >>> 0;
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_bb96b2010945f0bc: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./interlink_worldgen_wasm_bg.js": import0,
    };
}

const WasmWorldgenDiagnosticFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmworldgendiagnostic_free(ptr, 1));
const WasmWorldgenGeologyFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmworldgengeology_free(ptr, 1));
const WasmWorldgenInheritanceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmworldgeninheritance_free(ptr, 1));
const WasmWorldgenLithosphereFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmworldgenlithosphere_free(ptr, 1));
const WasmWorldgenTectonicsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmworldgentectonics_free(ptr, 1));
const WasmWorldgenTopologyFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmworldgentopology_free(ptr, 1));

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayF64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

function getArrayU16FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint16ArrayMemory0().subarray(ptr / 2, ptr / 2 + len);
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint16ArrayMemory0 = null;
function getUint16ArrayMemory0() {
    if (cachedUint16ArrayMemory0 === null || cachedUint16ArrayMemory0.byteLength === 0) {
        cachedUint16ArrayMemory0 = new Uint16Array(wasm.memory.buffer);
    }
    return cachedUint16ArrayMemory0;
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedFloat32ArrayMemory0 = null;
    cachedFloat64ArrayMemory0 = null;
    cachedUint16ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('interlink_worldgen_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
