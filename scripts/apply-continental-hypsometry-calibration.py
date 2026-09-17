from pathlib import Path

p = Path('rust/interlink-worldgen/src/historical_topography.rs')
s = p.read_text()

s = s.replace(
    'pub const HISTORICAL_TOPOGRAPHY_STAGE_VERSION: u32 = 15;\nconst HISTORICAL_TOPOGRAPHY_NAMESPACE: &str = "terrain:historical-material-morphology:v1";',
    'pub const HISTORICAL_TOPOGRAPHY_STAGE_VERSION: u32 = 16;\nconst HISTORICAL_TOPOGRAPHY_NAMESPACE: &str = "terrain:historical-material-morphology:v2";',
    1,
)

anchor = '''fn finalize_historical_stage(state: &mut TopographyState, request: &TopographyRequest) {'''
insert = '''fn stable_continental_buoyancy_support_m(
    inherited: &InheritedPhysicalState,
    sample: usize,
) -> f64 {
    if inherited.crust_kind[sample] != CrustKind::Continental as u8
        || inherited.province_kind[sample] != 0
        || inherited.structural_zone_kind[sample]
            == InheritedStructureKind::ContinentalMargin as u8
        || inherited.structural_zone_kind[sample]
            == InheritedStructureKind::InheritedRift as u8
    {
        return 0.0;
    }

    // Stable continental interiors should retain a modest freeboard advantage from their thick,
    // buoyant lithospheric columns. This is deliberately a bounded isostatic correction rather
    // than a land-mask command: active rifts, subsiding basins, passive margins, and active
    // orogenic provinces retain their own causal topography and may remain submerged.
    let rift = f64::from(inherited.rift_history[sample]).clamp(0.0, 1.0);
    let subsidence = f64::from(inherited.subsidence_history[sample]).clamp(0.0, 1.0);
    let basin = f64::from(inherited.basin_potential[sample]).clamp(0.0, 1.0);
    let release = clamp01((rift - 0.12) / 0.35)
        .max(clamp01((subsidence - 0.16) / 0.40))
        .max(clamp01((basin - 0.18) / 0.45));
    let stability = 1.0 - release;
    if stability <= 0.0 {
        return 0.0;
    }

    let buoyancy = clamp01(
        (f64::from(inherited.compensated_buoyancy_index[sample]) + 0.25) / 1.25,
    );
    let thickness = clamp01((f64::from(inherited.crust_thickness_km[sample]) - 30.0) / 22.0);
    let strength = f64::from(inherited.strength_index[sample]).clamp(0.0, 1.0);
    let physical_support = (0.76 + 0.24 * buoyancy)
        * (0.84 + 0.16 * thickness)
        * (0.90 + 0.10 * strength);

    480.0 * stability.powf(1.15) * physical_support
}

'''
if anchor not in s:
    raise SystemExit('finalize anchor not found')
s = s.replace(anchor, insert + anchor, 1)

s = s.replace(
    'b"terrain:historical-passive-margin-topography:v1\\0",',
    'b"terrain:historical-continental-hypsometry:v2\\0",',
    1,
)

old_validation = '''        || inherited.weakness_index.len() != count
        || inherited.crust_kind.len() != count
    {'''
new_validation = '''        || inherited.weakness_index.len() != count
        || inherited.strength_index.len() != count
        || inherited.crust_kind.len() != count
        || inherited.crust_thickness_km.len() != count
        || inherited.compensated_buoyancy_index.len() != count
        || inherited.rift_history.len() != count
        || inherited.subsidence_history.len() != count
        || inherited.basin_potential.len() != count
    {'''
if old_validation not in s:
    raise SystemExit('input validation anchor not found')
s = s.replace(old_validation, new_validation, 1)

old_loop = '''    let mut area_weighted_deflection = 0.0_f64;
    for sample in 0..count {
        let deflection = passive_margin_deflection_m(inherited, sample);
        if deflection == 0.0 {
            continue;
        }
        state.rift_basin_elevation_m[sample] += deflection as f32;
        state.solid_elevation_m[sample] += deflection as f32;
        area_weighted_deflection += deflection * areas[sample];
    }

    // Preserve the WG-4 global solid datum after adding the local shelf/basin term.
    let datum_shift = area_weighted_deflection / total_area;'''
new_loop = '''    let mut area_weighted_adjustment = 0.0_f64;
    for sample in 0..count {
        let support = stable_continental_buoyancy_support_m(inherited, sample);
        if support != 0.0 {
            state.isostatic_elevation_m[sample] += support as f32;
            state.solid_elevation_m[sample] += support as f32;
            area_weighted_adjustment += support * areas[sample];
        }

        let deflection = passive_margin_deflection_m(inherited, sample);
        if deflection != 0.0 {
            state.rift_basin_elevation_m[sample] += deflection as f32;
            state.solid_elevation_m[sample] += deflection as f32;
            area_weighted_adjustment += deflection * areas[sample];
        }
    }

    // Preserve the WG-4 global solid datum after adding the local isostatic and shelf terms.
    let datum_shift = area_weighted_adjustment / total_area;'''
if old_loop not in s:
    raise SystemExit('historical adjustment loop anchor not found')
s = s.replace(old_loop, new_loop, 1)

p.write_text(s)

smoke = Path('rust/interlink-worldgen-cli/examples/tectonic_topography_cutover_smoke.rs')
t = smoke.read_text()
old = 'if terrain.stage.version != TOPOGRAPHY_STAGE_VERSION || terrain.stage.version != 15 {'
new = 'if terrain.stage.version != TOPOGRAPHY_STAGE_VERSION || terrain.stage.version != 16 {'
if old not in t:
    raise SystemExit('topography stage assertion anchor not found')
smoke.write_text(t.replace(old, new, 1))
