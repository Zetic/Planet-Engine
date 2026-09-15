from pathlib import Path
import subprocess


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing marker in {path}: {old[:100]!r}")
    if text.count(old) != 1:
        raise SystemExit(f"non-unique marker in {path}: {text.count(old)}")
    p.write_text(text.replace(old, new, 1))


surface = "rust/interlink-worldgen/src/surface_water.rs"
replace_once(
    surface,
    'use crate::{GeodesicTopology, PlanetPhysicalParameters, WorldgenError};\nuse std::collections::VecDeque;',
    'use crate::{GeodesicTopology, PlanetPhysicalParameters, PlanetTopology, WorldgenError};\nuse std::collections::VecDeque;',
)

causal = "rust/interlink-worldgen/src/causal_pipeline.rs"
replace_once(
    causal,
    'use std::ops::{Deref, DerefMut};',
    'use std::collections::VecDeque;\nuse std::ops::{Deref, DerefMut};',
)
marker = 'fn province_relief(inherited: &InheritedPhysicalState, index: usize) -> (f64, f64) {'
helper = r'''fn major_ocean_reservoir_seed_mask(
    topology: &GeodesicTopology,
    crust_kind: &[u8],
    provisional_submerged: &[u8],
) -> Vec<u8> {
    let count = topology.metrics().sample_count as usize;
    let mut visited = vec![false; count];
    let mut components: Vec<(f64, Vec<usize>)> = Vec::new();
    let mut candidate_area = 0.0_f64;
    let surface_area = topology.dual_area_steradians().iter().sum::<f64>();

    for sample in 0..count {
        if visited[sample]
            || crust_kind[sample] != CRUST_OCEANIC
            || provisional_submerged[sample] == 0
        {
            continue;
        }
        visited[sample] = true;
        let mut queue = VecDeque::from([sample as u32]);
        let mut members = Vec::new();
        let mut area = 0.0_f64;
        while let Some(current) = queue.pop_front() {
            let index = current as usize;
            members.push(index);
            area += topology.dual_area_steradians()[index];
            for neighbor in topology.neighbors(current) {
                let neighbor = *neighbor as usize;
                if !visited[neighbor]
                    && crust_kind[neighbor] == CRUST_OCEANIC
                    && provisional_submerged[neighbor] != 0
                {
                    visited[neighbor] = true;
                    queue.push_back(neighbor as u32);
                }
            }
        }
        candidate_area += area;
        components.push((area, members));
    }

    components.sort_by(|left, right| right.0.total_cmp(&left.0));
    // Global ocean reservoirs must be broad components, not tiny trapped oceanic slivers.
    // Marginal seas do not need to be seeds: they join automatically if a marine path reaches them.
    let minimum_reservoir_area = (surface_area * 0.0025).max(candidate_area * 0.015);
    let mut seeds = vec![0_u8; count];
    let mut kept = 0usize;
    for (area, members) in &components {
        if *area + 1.0e-15 < minimum_reservoir_area {
            continue;
        }
        kept += 1;
        for sample in members {
            seeds[*sample] = 1;
        }
    }
    if kept == 0 {
        if let Some((_, members)) = components.first() {
            for sample in members {
                seeds[*sample] = 1;
            }
        }
    }
    seeds
}

'''
replace_once(causal, marker, helper + marker)
replace_once(
    causal,
    '''        // Remove most legacy collision-thickening isostatic imprint in areas where the old
        // radial history was strong. New crustal-root/plateau fields now own that relief.
        let legacy_orogen = f64::from(inherited.legacy.orogenic_history[i]).clamp(0.0, 1.0);
        let debiased_isostasy =
            f64::from(baseline.isostatic_elevation_m[i]) * (1.0 - 0.58 * legacy_orogen);

        raw[i] = debiased_isostasy
''',
    '''        // Preserve the actual crustal-isostatic state.  The causal cut already discards the
        // legacy *orogenic elevation* field; attenuating all isostatic support wherever legacy
        // orogenic history was strong carved an artificial low corridor around the replacement
        // range.  Thick continental crust remains buoyant regardless of which relief model owns
        // the active mountain load.
        raw[i] = f64::from(baseline.isostatic_elevation_m[i])
''',
)
replace_once(
    causal,
    '''    let ocean_seed_mask = inherited
        .crust_kind
        .iter()
        .map(|kind| u8::from(*kind == CRUST_OCEANIC))
        .collect::<Vec<_>>();
    let water = crate::surface_water::solve_hydrostatic_surface_water_connected_f64(
''',
    '''    let solid_f32 = solid.iter().map(|value| *value as f32).collect::<Vec<_>>();
    // Use the old threshold solve only to discover broad submerged oceanic reservoirs.  It does
    // not define final water state.  This prevents every tiny oceanic crust remnant from becoming
    // an independent marine-water source inside a collision zone.
    let provisional = crate::surface_water::solve_hydrostatic_surface_water_f64(
        topology,
        &solid,
        planet,
    )?;
    let ocean_seed_mask = major_ocean_reservoir_seed_mask(
        topology,
        &inherited.crust_kind,
        &provisional.submerged_mask,
    );
    let water = crate::surface_water::solve_hydrostatic_surface_water_connected_f64(
''',
)

orogen = "rust/interlink-worldgen/src/orogen_provinces.rs"
replace_once(
    orogen,
    'OrogenProvinceKind::CollisionalPlateau => (260.0, 1_100.0),',
    'OrogenProvinceKind::CollisionalPlateau => (260.0, 980.0),',
)

smoke = "rust/interlink-worldgen-cli/examples/tectonic_topography_cutover_smoke.rs"
replace_once(
    smoke,
    'CrustKind, LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,',
    'CrustKind, LithosphereRequest, OrogenProvinceKind, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,',
)
replace_once(
    smoke,
    '''        if continental && inherited.province_ids[sample] != 0 {
            continental_orogen += 1;
            if terrain.submerged_mask[sample] != 0 {
                flooded_continental_orogen += 1;
            }
        }
''',
    '''        let kind = inherited.province_kind[sample];
        let collision_orogen = kind == OrogenProvinceKind::ContinentalCollision as u8
            || kind == OrogenProvinceKind::CollisionalPlateau as u8
            || kind == OrogenProvinceKind::TerraneAccretion as u8
            || kind == OrogenProvinceKind::TranspressionalOrogen as u8;
        if continental && collision_orogen {
            continental_orogen += 1;
            if terrain.submerged_mask[sample] != 0 {
                flooded_continental_orogen += 1;
            }
        }
''',
)

# The temporary helper is allowed to push source changes but the GitHub App token cannot update a
# workflow file.  Keep the first helper's CI edit out of this generated commit; CI is patched
# directly after the source commit lands.
subprocess.run(["git", "checkout", "HEAD", "--", ".github/workflows/ci.yml"], check=True)

print('ocean reservoir and isostasy follow-up applied')
