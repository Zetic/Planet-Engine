from pathlib import Path


topo = Path('rust/interlink-worldgen/src/historical_topography.rs')
text = topo.read_text()
old = (
    "    if inherited.structural_zone_kind[sample] != InheritedStructureKind::ContinentalMargin as u8\n"
    "        || inherited.crust_kind[sample] == CrustKind::Oceanic as u8\n"
    "    {\n"
    "        return 0.0;\n"
    "    }\n"
)
new = (
    "    if inherited.structural_zone_kind[sample] != InheritedStructureKind::ContinentalMargin as u8\n"
    "        || inherited.crust_kind[sample] == CrustKind::Oceanic as u8\n"
    "        || inherited.province_kind[sample] != 0\n"
    "    {\n"
    "        return 0.0;\n"
    "    }\n"
)
assert text.count(old) == 1, 'expected passive-margin guard exactly once'
topo.write_text(text.replace(old, new, 1))

causal = Path('rust/interlink-worldgen/src/causal_pipeline.rs')
text = causal.read_text()
old = '            + 180.0 * intensity\n'
new = '            + 360.0 * intensity\n'
assert text.count(old) == 1, 'expected active-collision support term exactly once'
causal.write_text(text.replace(old, new, 1))

smoke = Path('rust/interlink-worldgen-cli/examples/tectonic_topography_cutover_smoke.rs')
text = smoke.read_text()
marker = '    if terrain.stage.version != TOPOGRAPHY_STAGE_VERSION || terrain.stage.version != 15 {\n'
diag = '''    let mut flooded = Vec::new();
    let mut kind_counts = [0usize; 7];
    let mut kind_flooded = [0usize; 7];
    for sample in 0..terrain.solid_elevation_m.len() {
        let continental = inherited.crust_kind[sample] == CrustKind::Continental as u8;
        let kind = inherited.province_kind[sample] as usize;
        let collision_orogen = kind == OrogenProvinceKind::ContinentalCollision as usize
            || kind == OrogenProvinceKind::CollisionalPlateau as usize
            || kind == OrogenProvinceKind::TerraneAccretion as usize
            || kind == OrogenProvinceKind::TranspressionalOrogen as usize;
        if continental && collision_orogen {
            if kind < kind_counts.len() { kind_counts[kind] += 1; }
            if terrain.submerged_mask[sample] != 0 {
                if kind < kind_flooded.len() { kind_flooded[kind] += 1; }
                flooded.push((
                    terrain.water_depth_m[sample],
                    inherited.orogenic_history[sample],
                    inherited.mountain_core_index[sample],
                    inherited.crustal_root_index[sample],
                    inherited.plateau_index[sample],
                    inherited.fold_thrust_index[sample],
                    inherited.suture_index[sample],
                    inherited.transpression_index[sample],
                    inherited.boundary_distance_km[sample],
                    terrain.solid_elevation_m[sample],
                    terrain.orogenic_elevation_m[sample],
                ));
            }
        }
    }
    flooded.sort_by(|a, b| a.0.total_cmp(&b.0));
    println!("OROGEN_FLOOD_DIAG kinds={:?} flooded={:?}", kind_counts, kind_flooded);
    for q in [0.0_f64, 0.25, 0.50, 0.75, 0.90, 0.95, 0.99, 1.0] {
        if !flooded.is_empty() {
            let idx = (((flooded.len() - 1) as f64) * q).round() as usize;
            let v = flooded[idx];
            println!("OROGEN_FLOOD_Q q={:.2} depth={:.1} intensity={:.3} core={:.3} root={:.3} plateau={:.3} fold={:.3} suture={:.3} trans={:.3} distance={:.1} solid={:.1} orogen={:.1}", q, v.0, v.1, v.2, v.3, v.4, v.5, v.6, v.7, v.8, v.9, v.10);
        }
    }
'''
assert text.count(marker) == 1, 'expected smoke gate marker exactly once'
smoke.write_text(text.replace(marker, diag + marker, 1))
