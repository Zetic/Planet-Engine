from pathlib import Path

path = Path("rust/interlink-worldgen/src/orogen_provinces.rs")
s = path.read_text()

s = s.replace(
    "pub const OROGEN_PROVINCE_STAGE_VERSION: u32 = 4;",
    "pub const OROGEN_PROVINCE_STAGE_VERSION: u32 = 5;",
)
s = s.replace(
    "worldgen:geology:tectonic-orogen-provinces:v4",
    "worldgen:geology:tectonic-orogen-provinces:v5",
)
s = s.replace("interlink-orogen-provinces:v4\\0", "interlink-orogen-provinces:v5\\0")
s = s.replace(
    "In v4 this is no longer a direct boundary-normal ridge:",
    "In v5 collision relief is redistributed onto persistent structural strands:",
)

old_axis = '''fn collision_axis_center(signals: StructureSignals, source: BoundarySource) -> f64 {
    let inherited_shift = 0.17 * signals.weak_corridor
        + 0.08 * signals.transfer
        + 0.06 * signals.paleo_suture
        + 0.04 * signals.rift;
    let maturity_shift = 0.05 * source.maturity * source.shortening;
    (0.16 + inherited_shift + maturity_shift).clamp(0.14, 0.52)
}'''
new_axis = '''fn collision_axis_center(signals: StructureSignals, source: BoundarySource) -> f64 {
    // v5 treats the source profile as a broad deformation envelope. The eventual mountain
    // skeleton is extracted later as a two-dimensional structural-strand field, but moving the
    // envelope itself inland prevents the literal suture from remaining the default relief axis.
    let inherited_shift = 0.36 * signals.weak_corridor
        + 0.20 * signals.transfer
        + 0.14 * signals.paleo_suture
        + 0.08 * signals.rift;
    let maturity_shift = 0.12 * source.maturity * source.shortening;
    (0.30 + inherited_shift + maturity_shift).clamp(0.26, 0.92)
}'''
assert old_axis in s
s = s.replace(old_axis, new_axis)

old_profile = '''    let axis_center = collision_axis_center(signals, source);
    let primary_sigma = 0.24 + 0.07 * signals.transfer + 0.04 * (1.0 - resistance);
    let primary = gaussian(x, axis_center, primary_sigma);
    let secondary_eligibility = clamp01(
        smoothstep((source.maturity - 0.25) / 0.60)
            * smoothstep((source.shortening - 0.20) / 0.65)
            * (0.30 + 0.70 * signals.inherited_belt),
    );
    let secondary_center = (axis_center
        + 0.42
        + 0.16 * (1.0 - resistance)
        + 0.10 * signals.transfer)
        .clamp(0.48, 1.16);
    let secondary = gaussian(x, secondary_center, 0.24 + 0.08 * signals.transfer)
        * secondary_eligibility;
    let tertiary = gaussian(x, secondary_center + 0.32, 0.20)
        * secondary_eligibility
        * signals.transfer
        * 0.42;
    let boundary_core_suppression = 0.58 + 0.42 * smoothstep(x / 0.10);'''
new_profile = '''    let axis_center = collision_axis_center(signals, source);
    let primary_sigma = 0.18 + 0.05 * signals.transfer + 0.03 * (1.0 - resistance);
    let primary = gaussian(x, axis_center, primary_sigma);
    let secondary_eligibility = clamp01(
        smoothstep((source.maturity - 0.22) / 0.58)
            * smoothstep((source.shortening - 0.18) / 0.62)
            * (0.22 + 0.78 * signals.inherited_belt.max(signals.weak_corridor)),
    );
    let secondary_center = (axis_center
        + 0.48
        + 0.20 * (1.0 - resistance)
        + 0.16 * signals.transfer)
        .clamp(0.72, 1.48);
    let secondary = gaussian(x, secondary_center, 0.18 + 0.06 * signals.transfer)
        * secondary_eligibility;
    let tertiary = gaussian(x, secondary_center + 0.40, 0.16)
        * secondary_eligibility
        * signals.transfer
        * 0.55;
    let boundary_core_suppression = 0.18 + 0.82 * smoothstep(x / 0.22);'''
assert old_profile in s
s = s.replace(old_profile, new_profile)

old_interior = '''    root[sample] = root[sample].max((0.14 * shield + 0.15 * mobile) as f32);
    plateau[sample] = plateau[sample].max((0.085 * shield + 0.045 * mobile) as f32);
    mountain[sample] = mountain[sample].max((0.075 * mobile) as f32);
    fold[sample] = fold[sample].max((0.080 * mobile * (0.55 + 0.45 * signals.transfer)) as f32);
    suture[sample] = suture[sample]
        .max((0.15 * signals.paleo_suture + 0.055 * signals.transfer) as f32);
    transpression[sample] = transpression[sample].max((0.060 * mobile * signals.transfer) as f32);
    orogenic[sample] = orogenic[sample].max((0.10 * mobile + 0.035 * shield) as f32);'''
new_interior = '''    // Interior morphology must survive WG-4 at continental map scale. These values remain far
    // below active-orogen amplitudes, but are deliberately strong enough for shields, fossil
    // mobile belts, sutures and failed-rift grain to organize broad continental relief.
    root[sample] = root[sample].max((0.24 * shield + 0.24 * mobile) as f32);
    plateau[sample] = plateau[sample].max((0.13 * shield + 0.07 * mobile) as f32);
    mountain[sample] = mountain[sample].max((0.15 * mobile) as f32);
    fold[sample] = fold[sample].max((0.14 * mobile * (0.50 + 0.50 * signals.transfer)) as f32);
    suture[sample] = suture[sample]
        .max((0.19 * signals.paleo_suture + 0.08 * signals.transfer) as f32);
    transpression[sample] = transpression[sample].max((0.11 * mobile * signals.transfer) as f32);
    orogenic[sample] = orogenic[sample].max((0.18 * mobile + 0.07 * shield) as f32);'''
assert old_interior in s
s = s.replace(old_interior, new_interior)

marker = "\nfn blend_profiles(primary: StructuralProfile, secondary: StructuralProfile) -> StructuralProfile {"
assert marker in s
helpers = r'''

fn collision_like(kind: u8) -> bool {
    kind == OrogenProvinceKind::ContinentalCollision as u8
        || kind == OrogenProvinceKind::CollisionalPlateau as u8
        || kind == OrogenProvinceKind::TerraneAccretion as u8
        || kind == OrogenProvinceKind::TranspressionalOrogen as u8
}

fn structural_strand_affinity(signals: StructureSignals) -> f64 {
    clamp01(
        0.38 * signals.inherited_belt
            + 0.28 * signals.weak_corridor
            + 0.18 * signals.transfer
            + 0.10 * signals.paleo_suture
            + 0.06 * signals.rift,
    )
}

fn spread_structural_strands<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    province_ids: &[u16],
    province_kind: &[u8],
    affinity: &[f32],
    seeds: &[f32],
    passes: usize,
    decay: f64,
) -> Vec<f32> {
    let mut current = seeds.to_vec();
    let mut next = current.clone();
    for _ in 0..passes {
        next.copy_from_slice(&current);
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            if !collision_like(province_kind[index]) {
                continue;
            }
            let plate = tectonics.plate_ids[index];
            let province = province_ids[index];
            for neighbor in topology.neighbors(sample) {
                let neighbor = *neighbor as usize;
                if tectonics.plate_ids[neighbor] != plate || !collision_like(province_kind[neighbor]) {
                    continue;
                }
                let transfer = if province_ids[neighbor] == province { 1.0 } else { 0.68 };
                let inherited = 0.72 + 0.28 * f64::from(affinity[index]);
                let candidate = f64::from(current[neighbor]) * decay * transfer * inherited;
                next[index] = next[index].max(candidate as f32);
            }
        }
        std::mem::swap(&mut current, &mut next);
    }
    current
}

#[allow(clippy::too_many_arguments)]
fn redistribute_collision_strands<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    pre: &PreOrogenicLithosphereModel,
    province_ids: &[u16],
    province_kind: &[u8],
    boundary_distance: &[f32],
    local_width: &[f32],
    maturity: &[f32],
    shortening: &[f32],
    mountain: &mut [f32],
    root: &mut [f32],
    fold: &mut [f32],
    transpression: &mut [f32],
    orogenic: &mut [f32],
) {
    let count = topology.sample_count() as usize;
    let max_province = province_ids.iter().copied().max().unwrap_or(0) as usize;
    if max_province == 0 {
        return;
    }

    let mut affinity = vec![0.0_f32; count];
    let mut primary_score = vec![0.0_f32; count];
    let mut secondary_score = vec![0.0_f32; count];
    let mut primary_values = vec![Vec::<f32>::new(); max_province + 1];
    let mut secondary_values = vec![Vec::<f32>::new(); max_province + 1];

    for sample in 0..count {
        if !collision_like(province_kind[sample]) || province_ids[sample] == 0 {
            continue;
        }
        let signals = structure_signals(pre, sample);
        let inherited = structural_strand_affinity(signals);
        affinity[sample] = inherited as f32;
        let width = f64::from(local_width[sample]).max(120.0);
        let distance = f64::from(boundary_distance[sample]);
        let x = distance / width;
        let off_boundary = smoothstep((distance - 65.0) / 240.0);
        let tectonic = f64::from(maturity[sample]) * f64::from(shortening[sample]);
        let deformation = f64::from(mountain[sample])
            .max(f64::from(root[sample]) * 0.90)
            .max(f64::from(fold[sample]) * 0.78)
            .max(f64::from(transpression[sample]) * 0.85);

        let primary_center = 0.58 + 0.25 * signals.weak_corridor + 0.10 * signals.transfer;
        let secondary_center = 1.08 + 0.22 * signals.transfer + 0.14 * signals.inherited_belt;
        let p = clamp01(
            tectonic
                * (0.22 + 0.78 * inherited)
                * gaussian(x, primary_center, 0.38)
                * off_boundary
                * (0.48 + 0.52 * deformation),
        );
        let secondary_control = signals.inherited_belt.max(signals.transfer);
        let q = clamp01(
            tectonic
                * (0.10 + 0.90 * secondary_control)
                * gaussian(x, secondary_center, 0.30)
                * off_boundary
                * (0.44 + 0.56 * deformation),
        );
        primary_score[sample] = p as f32;
        secondary_score[sample] = q as f32;
        if p > 0.0 {
            primary_values[province_ids[sample] as usize].push(p as f32);
        }
        if q > 0.0 {
            secondary_values[province_ids[sample] as usize].push(q as f32);
        }
    }

    let quantiles = |mut values: Vec<Vec<f32>>, q: f64| -> Vec<f32> {
        values
            .iter_mut()
            .map(|items| {
                if items.is_empty() {
                    return 1.0;
                }
                items.sort_by(|a, b| a.total_cmp(b));
                let index = (((items.len() - 1) as f64) * q).floor() as usize;
                items[index]
            })
            .collect()
    };
    let primary_threshold = quantiles(primary_values, 0.74);
    let secondary_threshold = quantiles(secondary_values, 0.82);
    let mut primary_seed = vec![0.0_f32; count];
    let mut secondary_seed = vec![0.0_f32; count];
    for sample in 0..count {
        let province = province_ids[sample] as usize;
        if province == 0 || !collision_like(province_kind[sample]) {
            continue;
        }
        if primary_score[sample] >= primary_threshold[province] && primary_score[sample] > 0.025 {
            primary_seed[sample] = (f64::from(primary_score[sample]) * 1.65).min(1.0) as f32;
        }
        if secondary_score[sample] >= secondary_threshold[province] && secondary_score[sample] > 0.020 {
            secondary_seed[sample] = (f64::from(secondary_score[sample]) * 1.85).min(1.0) as f32;
        }
    }

    let primary = spread_structural_strands(
        topology,
        tectonics,
        province_ids,
        province_kind,
        &affinity,
        &primary_seed,
        4,
        0.86,
    );
    let secondary = spread_structural_strands(
        topology,
        tectonics,
        province_ids,
        province_kind,
        &affinity,
        &secondary_seed,
        3,
        0.84,
    );

    for sample in 0..count {
        if !collision_like(province_kind[sample]) {
            continue;
        }
        let primary = f64::from(primary[sample]);
        let secondary = f64::from(secondary[sample]);
        let strand = clamp01(primary.max(secondary * 0.86));
        let legacy = f64::from(mountain[sample]);
        let retained = if strand > 0.08 { legacy * 0.30 } else { legacy * 0.62 };
        mountain[sample] = clamp01(retained.max(strand)) as f32;
        root[sample] = f64::from(root[sample])
            .max(primary * 0.70)
            .max(secondary * 0.52)
            .min(1.0) as f32;
        fold[sample] = f64::from(fold[sample])
            .max(secondary * 0.72)
            .max(primary * 0.26)
            .min(1.0) as f32;
        let signals = structure_signals(pre, sample);
        transpression[sample] = f64::from(transpression[sample])
            .max(strand * signals.transfer * 0.48)
            .min(1.0) as f32;
        orogenic[sample] = f64::from(orogenic[sample]).max(strand * 0.88).min(1.0) as f32;
    }
}
'''
s = s.replace(marker, helpers + marker)

before_smooth = "    orogenic = smooth_same_plate(topology, tectonics, &province_ids, &orogenic, 2);"
call = '''    redistribute_collision_strands(
        topology,
        tectonics,
        pre,
        &province_ids,
        &province_kind,
        &boundary_distance,
        &local_width,
        &maturity,
        &shortening,
        &mut mountain_core,
        &mut root,
        &mut fold_thrust,
        &mut transpression,
        &mut orogenic,
    );

    orogenic = smooth_same_plate(topology, tectonics, &province_ids, &orogenic, 2);'''
assert before_smooth in s
s = s.replace(before_smooth, call, 1)

s = s.replace('let seed = "wg36-v4-orogen-determinism";', 'let seed = "wg36-v5-orogen-determinism";')
s = s.replace('let seed = "wg36-v4-causal-cut";', 'let seed = "wg36-v5-causal-cut";')
s = s.replace('let seed = "wg36-v4-interior-structure";', 'let seed = "wg36-v5-interior-structure";')
s = s.replace(
    "assert!(inherited > plain + 0.15);\n        assert!(inherited <= 0.52);",
    "assert!(inherited > plain + 0.30);\n        assert!(inherited <= 0.92);",
)
s = s.replace(
    "let displaced = source_profile(source, 1, 240.0, 240.0, resistance, inherited);",
    "let displaced = source_profile(source, 1, 390.0, 390.0, resistance, inherited);",
)
s = s.replace(
    "assert!(displaced.boundary_distance_km >= 200.0);",
    "assert!(displaced.boundary_distance_km >= 350.0);",
)

path.write_text(s)

doc = Path("docs/worldgen-rewrite/TECTONIC_STRUCTURAL_STRANDS.md")
doc.write_text(
    """# Tectonic structural strands (WG-3.6 v5)

WG-3.6 v5 is the visual follow-up to continental tectonic morphology v4. The v4 architecture removed hard single-source ownership, but visual acceptance showed that major collision ranges still read as direct copies of the plate-boundary graph.

## Representation change

Collision source profiles now define broad deformation envelopes rather than final mountain geometry. Inside each active collision province, v5 scores candidate structural corridors from inherited mobile belts, weak crust, shear/province transfer zones, paleo-sutures and rift memory. Per-province upper-quantile candidates seed primary and secondary structural strands. Those strands then propagate over the spherical adjacency graph on the same plate, with reduced but nonzero transfer across neighboring collision provinces.

The final mountain core is redistributed onto those graph strands. The old boundary-normal ridge is retained only as a weak fallback where no coherent strand exists. This allows range axes to step, branch, migrate hundreds of kilometres inland, and cross source/province ownership seams without turning tectonics into arbitrary noise.

## Continental interiors

Shield/root support and fossil mobile-belt relief are strengthened so that stable continents retain visible tectonic grain at map scale. Active orogens remain much stronger than fossil structure.

## Acceptance target

Same-seed physical-elevation maps should visibly differ from v4 even though coastlines and plate assembly remain unchanged. Mature continental collisions should contain separated or offset belts, literal convergent edges should often lie beside rather than under the highest range axis, junctions should spread into transfer complexes, and large continental interiors should show broad shield/mobile-belt structure rather than featureless procedural mottling. Subduction cordilleras may remain tightly boundary-correlated.

Hydrology, erosion and climate are intentionally unchanged by this PR.
"""
)
