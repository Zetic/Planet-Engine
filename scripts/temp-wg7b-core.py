from pathlib import Path
import re

# Generalize WG-6A's existing drainage packager so WG-7B can rebuild the exact same
# drainage semantics on a distinct evolved surface without fabricating a WG-4 state.
path = Path("rust/interlink-worldgen/src/drainage.rs")
text = path.read_text()
pattern = re.compile(r"pub fn generate_drainage_topology\(.*?\n\}\n\n#\[cfg\(test\)\]", re.S)
match = pattern.search(text)
if not match:
    raise SystemExit("could not locate generate_drainage_topology block")
replacement = r'''pub(crate) fn generate_drainage_from_surface(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f32],
    submerged_mask: &[u8],
    sea_level_m: Option<f64>,
    source_surface_hash: u64,
    planet: PlanetPhysicalParameters,
    request: &DrainageRequest,
) -> Result<DrainageState, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    if solid_elevation_m.len() != topology.sample_count() as usize
        || submerged_mask.len() != topology.sample_count() as usize
    {
        return Err(WorldgenError::InvalidHydrology(
            "drainage surface must align with canonical topology",
        ));
    }
    let core = solve_core(
        topology,
        solid_elevation_m,
        submerged_mask,
        planet.radius_m,
        sea_level_m,
    )
    .map_err(WorldgenError::InvalidHydrology)?;

    let stage_seed = derive_stage_seed(&request.seed, DRAINAGE_NAMESPACE);
    let mut drainage_hash = FNV_OFFSET_BASIS;
    drainage_hash = fnv_update(drainage_hash, DRAINAGE_STAGE_ID.as_bytes());
    drainage_hash = fnv_update(drainage_hash, &DRAINAGE_STAGE_VERSION.to_le_bytes());
    drainage_hash = fnv_update(drainage_hash, &stage_seed.to_le_bytes());
    drainage_hash = fnv_update(drainage_hash, &planet.parameter_hash().to_le_bytes());
    drainage_hash = fnv_update(drainage_hash, &source_surface_hash.to_le_bytes());
    for &value in &core.receiver {
        drainage_hash = fnv_update(drainage_hash, &value.to_le_bytes());
    }
    for &value in &core.outlet_sample {
        drainage_hash = fnv_update(drainage_hash, &value.to_le_bytes());
    }
    for &value in &core.basin_id {
        drainage_hash = fnv_update(drainage_hash, &value.to_le_bytes());
    }
    for &value in &core.depression_id {
        drainage_hash = fnv_update(drainage_hash, &value.to_le_bytes());
    }
    for &value in &core.escape_elevation_m {
        drainage_hash = fnv_update(drainage_hash, &value.to_bits().to_le_bytes());
    }
    for &value in &core.contributing_area_m2 {
        drainage_hash = fnv_update(drainage_hash, &value.to_bits().to_le_bytes());
    }

    let depression_sample_count = core
        .depression_id
        .iter()
        .filter(|value| **value != INVALID_SAMPLE_ID)
        .count() as u32;
    let maximum_depression_depth_m = core
        .depression_depth_m
        .iter()
        .copied()
        .fold(0.0_f64, f64::max);
    let land_sample_count = submerged_mask.iter().filter(|value| **value == 0).count() as u32;
    let ocean_sample_count = topology.sample_count() - land_sample_count;

    Ok(DrainageState {
        stage: StageIdentity {
            id: DRAINAGE_STAGE_ID,
            version: DRAINAGE_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics: DrainageMetrics {
            sample_count: topology.sample_count(),
            land_sample_count,
            ocean_sample_count,
            basin_count: core.basins.len() as u32,
            depression_count: core.depressions.len() as u32,
            depression_sample_count,
            land_area_m2: core.land_area_m2,
            terminal_contributing_area_m2: core.terminal_contributing_area_m2,
            area_conservation_relative_error: core.area_conservation_relative_error,
            maximum_contributing_area_m2: core.maximum_contributing_area_m2,
            maximum_depression_depth_m,
            drainage_hash,
        },
        receiver: core.receiver,
        outlet_sample: core.outlet_sample,
        outlet_kind: core.outlet_kind,
        basin_id: core.basin_id,
        depression_id: core.depression_id,
        hydrologic_escape_elevation_m: core
            .escape_elevation_m
            .into_iter()
            .map(|value| value as f32)
            .collect(),
        depression_depth_m: core
            .depression_depth_m
            .into_iter()
            .map(|value| value as f32)
            .collect(),
        contributing_area_m2: core.contributing_area_m2,
        drainage_order: core.drainage_order,
        basins: core.basins,
        depressions: core.depressions,
    })
}

pub fn generate_drainage_topology(
    topology: &GeodesicTopology,
    topography: &TopographyState,
    planet: PlanetPhysicalParameters,
    request: &DrainageRequest,
) -> Result<DrainageState, WorldgenError> {
    if topography.metrics.sample_count != topology.sample_count() {
        return Err(WorldgenError::InvalidHydrology(
            "drainage topography must align with canonical topology",
        ));
    }
    generate_drainage_from_surface(
        topology,
        &topography.solid_elevation_m,
        &topography.submerged_mask,
        topography.metrics.sea_level_m,
        topography.metrics.topography_hash,
        planet,
        request,
    )
}

#[cfg(test)]'''
text = text[:match.start()] + replacement + text[match.end():]
path.write_text(text)

# Register the WG-7B public API without changing the existing WG-7A contract.
path = Path("rust/interlink-worldgen/src/lib.rs")
text = path.read_text()
old = "mod erosion;\n"
new = "mod erosion;\nmod evolution;\n"
if text.count(old) != 1:
    raise SystemExit(f"lib module anchor count {text.count(old)}")
text = text.replace(old, new, 1)
anchor = '''pub use erosion::{
    generate_fluvial_erosion_sediment, FluvialErosionMetrics, FluvialErosionParameters,
    FluvialErosionRequest, FluvialErosionState, FLUVIAL_EROSION_STAGE_ID,
    FLUVIAL_EROSION_STAGE_VERSION,
};
'''
addition = anchor + '''pub use evolution::{
    generate_bounded_terrain_evolution, TerrainEvolutionMetrics, TerrainEvolutionParameters,
    TerrainEvolutionRequest, TerrainEvolutionState, TERRAIN_EVOLUTION_STAGE_ID,
    TERRAIN_EVOLUTION_STAGE_VERSION,
};
'''
if text.count(anchor) != 1:
    raise SystemExit(f"lib export anchor count {text.count(anchor)}")
text = text.replace(anchor, addition, 1)
if "pub const WORLDGEN_ENGINE_VERSION: u32 = 9;" not in text:
    raise SystemExit("expected WG-7A engine version 9")
text = text.replace(
    "pub const WORLDGEN_ENGINE_VERSION: u32 = 9;",
    "pub const WORLDGEN_ENGINE_VERSION: u32 = 10;",
    1,
)
path.write_text(text)
