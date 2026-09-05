from pathlib import Path

p = Path('rust/interlink-worldgen/src/infill.rs')
s = p.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global s
    if old not in s:
        raise RuntimeError(f'missing anchor: {label}')
    s = s.replace(old, new, 1)

replace_once(
    '''    pub post_erosion_hydrology_hash: u64,
    pub input_evolved_surface_hash: u64,
    pub post_infill_surface_hash: u64,
''',
    '''    pub post_erosion_hydrology_hash: u64,
    pub input_evolved_surface_hash: u64,
    pub pre_infill_drainage_hash: u64,
    pub pre_infill_runoff_hash: u64,
    pub pre_infill_lake_hash: u64,
    pub pre_infill_seasonal_hash: u64,
    pub post_infill_surface_hash: u64,
''',
    'metrics fields',
)
replace_once(
    '''    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        &reconciliation
            .metrics
            .post_erosion_hydrology_hash
            .to_le_bytes(),
    );
    lake_sediment_infill_hash =
        hash_f32_slice(lake_sediment_infill_hash, &post_infill_solid_elevation_m);
''',
    '''    lake_sediment_infill_hash = fnv_update(
        lake_sediment_infill_hash,
        &reconciliation
            .metrics
            .post_erosion_hydrology_hash
            .to_le_bytes(),
    );
    for ancestry_hash in [
        reconciliation.metrics.post_erosion_drainage_hash,
        reconciliation.metrics.reconciled_runoff_hash,
        reconciliation.metrics.reconciled_lake_hash,
        reconciliation.metrics.reconciled_seasonal_hash,
    ] {
        lake_sediment_infill_hash =
            fnv_update(lake_sediment_infill_hash, &ancestry_hash.to_le_bytes());
    }
    lake_sediment_infill_hash =
        hash_f32_slice(lake_sediment_infill_hash, &post_infill_solid_elevation_m);
''',
    'hash ancestry',
)
replace_once(
    '''            post_erosion_hydrology_hash: reconciliation.metrics.post_erosion_hydrology_hash,
            input_evolved_surface_hash: evolution.metrics.evolved_surface_hash,
            post_infill_surface_hash,
''',
    '''            post_erosion_hydrology_hash: reconciliation.metrics.post_erosion_hydrology_hash,
            input_evolved_surface_hash: evolution.metrics.evolved_surface_hash,
            pre_infill_drainage_hash: reconciliation.metrics.post_erosion_drainage_hash,
            pre_infill_runoff_hash: reconciliation.metrics.reconciled_runoff_hash,
            pre_infill_lake_hash: reconciliation.metrics.reconciled_lake_hash,
            pre_infill_seasonal_hash: reconciliation.metrics.reconciled_seasonal_hash,
            post_infill_surface_hash,
''',
    'metrics values',
)
p.write_text(s)
