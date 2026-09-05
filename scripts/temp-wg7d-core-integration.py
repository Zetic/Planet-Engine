from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old not in text:
        raise RuntimeError(f"missing transform anchor: {label}")
    return text.replace(old, new, 1)

# Wire the new public module.
lib_path = Path('rust/interlink-worldgen/src/lib.rs')
lib = lib_path.read_text()
lib = replace_once(lib, 'mod hydroclimate;\nmod lakes;\n', 'mod hydroclimate;\nmod infill;\nmod lakes;\n', 'lib module')
lib = replace_once(
    lib,
    '''pub use hydroclimate::{
    build_hydroclimate_closure_report, HydroclimateClosureReport, HydroclimateLatitudeBand,
};
''',
    '''pub use hydroclimate::{
    build_hydroclimate_closure_report, HydroclimateClosureReport, HydroclimateLatitudeBand,
};
pub use infill::{
    generate_lake_sediment_infill, LakeSedimentInfillMetrics, LakeSedimentInfillParameters,
    LakeSedimentInfillRequest, LakeSedimentInfillState, LAKE_SEDIMENT_INFILL_STAGE_ID,
    LAKE_SEDIMENT_INFILL_STAGE_VERSION,
};
''',
    'lib export',
)
lib_path.write_text(lib)

# Retain compact per-depression lake delivery inside the private WG-7B routing result and expose
# a crate-private deterministic reconstruction helper for WG-7D. This does not change WG-7B's
# public state or hash.
evo_path = Path('rust/interlink-worldgen/src/evolution.rs')
evo = evo_path.read_text()
evo = replace_once(
    evo,
    '''struct AppliedSedimentRouting {
    load_kg_s: Vec<f32>,
    land_deposition_kg_s: Vec<f32>,
    total_land_deposition_kg_s: f64,
    total_lake_sink_kg_s: f64,
    total_terminal_ocean_sink_kg_s: f64,
}
''',
    '''struct AppliedSedimentRouting {
    load_kg_s: Vec<f32>,
    land_deposition_kg_s: Vec<f32>,
    lake_sink_kg_s_by_depression: Vec<f64>,
    total_land_deposition_kg_s: f64,
    total_lake_sink_kg_s: f64,
    total_terminal_ocean_sink_kg_s: f64,
}
''',
    'routing struct',
)
evo = replace_once(
    evo,
    '''    let mut land_deposition_kg_s = vec![0.0_f32; count];
    let mut total_land_deposition_kg_s = 0.0_f64;
''',
    '''    let mut land_deposition_kg_s = vec![0.0_f32; count];
    let mut lake_sink_kg_s_by_depression = vec![0.0_f64; active_lake_depression.len()];
    let mut total_land_deposition_kg_s = 0.0_f64;
''',
    'routing storage',
)
evo = replace_once(
    evo,
    '''        if active_lake {
            total_lake_sink_kg_s += available;
            continue;
        }
''',
    '''        if active_lake {
            total_lake_sink_kg_s += available;
            lake_sink_kg_s_by_depression[depression as usize] += available;
            continue;
        }
''',
    'lake routing ledger',
)
evo = replace_once(
    evo,
    '''    Ok(AppliedSedimentRouting {
        load_kg_s,
        land_deposition_kg_s,
        total_land_deposition_kg_s,
''',
    '''    Ok(AppliedSedimentRouting {
        load_kg_s,
        land_deposition_kg_s,
        lake_sink_kg_s_by_depression,
        total_land_deposition_kg_s,
''',
    'routing return',
)
route_end = '''    Ok(AppliedSedimentRouting {
        load_kg_s,
        land_deposition_kg_s,
        lake_sink_kg_s_by_depression,
        total_land_deposition_kg_s,
        total_lake_sink_kg_s,
        total_terminal_ocean_sink_kg_s,
    })
}

'''
helper = route_end + '''pub(crate) fn reconstruct_applied_lake_sediment_delivery_kg_s(
    topography: &TopographyState,
    drainage: &DrainageState,
    lakes: &LakeState,
    erosion: &FluvialErosionState,
    evolution: &TerrainEvolutionState,
) -> Result<Vec<f64>, WorldgenError> {
    let count = topography.solid_elevation_m.len();
    if topography.submerged_mask.len() != count
        || drainage.receiver.len() != count
        || drainage.depression_id.len() != count
        || erosion.sediment_transport_capacity_kg_s.len() != count
        || evolution.applied_sediment_supply_kg_s.len() != count
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7D lake-delivery reconstruction fields must align",
        ));
    }
    if evolution.metrics.drainage_hash != drainage.metrics.drainage_hash
        || evolution.metrics.lake_hash != lakes.metrics.lake_hash
        || evolution.metrics.fluvial_erosion_hash != erosion.metrics.fluvial_erosion_hash
    {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7D lake-delivery reconstruction requires exact WG-7B ancestry",
        ));
    }

    let mut active_lake_depression = vec![false; drainage.depressions.len()];
    for lake in &lakes.lakes {
        let depression = lake.depression_id as usize;
        if depression >= active_lake_depression.len() {
            return Err(WorldgenError::InvalidGeomorphology(
                "WG-7D historical lake references an unknown accepted depression",
            ));
        }
        active_lake_depression[depression] = true;
    }
    let routing = route_applied_sediment(
        &evolution.applied_sediment_supply_kg_s,
        &erosion.sediment_transport_capacity_kg_s,
        &topography.submerged_mask,
        &drainage.receiver,
        &drainage.drainage_order,
        &drainage.depression_id,
        &active_lake_depression,
    )
    .map_err(WorldgenError::InvalidGeomorphology)?;
    let expected = evolution.metrics.total_lake_sink_kg_s;
    let tolerance = 1.0e-9_f64.max(expected.abs() * 1.0e-12);
    if (routing.total_lake_sink_kg_s - expected).abs() > tolerance {
        return Err(WorldgenError::InvalidGeomorphology(
            "WG-7D reconstructed lake sediment does not match accepted WG-7B sink ledger",
        ));
    }
    Ok(routing.lake_sink_kg_s_by_depression)
}

'''
evo = replace_once(evo, route_end, helper, 'reconstruction helper')
evo_path.write_text(evo)

# Make lake-fill density explicit WG-7D physics rather than an untracked literal.
infill_path = Path('rust/interlink-worldgen/src/infill.rs')
infill = infill_path.read_text()
infill = replace_once(
    infill,
    '''    /// Hydrology parameters must exactly match the accepted WG-7C reconciliation contract.
    pub hydrology: PostErosionHydrologyParameters,
''',
    '''    /// Bulk density used to convert accepted historical lake-sink mass to deposited volume.
    pub deposited_sediment_density_kg_m3: f64,
    /// Hydrology parameters must exactly match the accepted WG-7C reconciliation contract.
    pub hydrology: PostErosionHydrologyParameters,
''',
    'density field',
)
infill = replace_once(
    infill,
    '''        Self {
            maximum_fill_depth_m: 120.0,
            hydrology: PostErosionHydrologyParameters::default(),
''',
    '''        Self {
            maximum_fill_depth_m: 120.0,
            deposited_sediment_density_kg_m3: 1_800.0,
            hydrology: PostErosionHydrologyParameters::default(),
''',
    'density default',
)
infill = replace_once(
    infill,
    '''        self.hydrology.validate()?;
        Ok(())
''',
    '''        if !self.deposited_sediment_density_kg_m3.is_finite()
            || self.deposited_sediment_density_kg_m3 <= 0.0
            || self.deposited_sediment_density_kg_m3 > 10_000.0
        {
            return Err("WG-7D deposited sediment density must be finite and within (0, 10000]");
        }
        self.hydrology.validate()?;
        Ok(())
''',
    'density validation',
)
infill = replace_once(
    infill,
    '''        hash = fnv_update(hash, &self.maximum_fill_depth_m.to_bits().to_le_bytes());
        fnv_update(hash, &self.hydrology.parameter_hash().to_le_bytes())
''',
    '''        hash = fnv_update(hash, &self.maximum_fill_depth_m.to_bits().to_le_bytes());
        hash = fnv_update(
            hash,
            &self.deposited_sediment_density_kg_m3.to_bits().to_le_bytes(),
        );
        fnv_update(hash, &self.hydrology.parameter_hash().to_le_bytes())
''',
    'density hash',
)
infill = replace_once(
    infill,
    '''    let density = 1_800.0_f64;
''',
    '''    let density = p.deposited_sediment_density_kg_m3;
''',
    'density use',
)
infill_path.write_text(infill)
