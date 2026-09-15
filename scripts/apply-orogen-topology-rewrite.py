from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing replacement marker in {path}: {old[:120]!r}")
    if text.count(old) != 1:
        raise SystemExit(f"replacement marker not unique in {path}: {text.count(old)} matches")
    p.write_text(text.replace(old, new, 1))


# --- Orogen province geometry: widen collision structure, make high relief conditional,
#     and stop manufacturing narrow mandatory troughs next to every range.
orogen = "rust/interlink-worldgen/src/orogen_provinces.rs"
replace_once(
    orogen,
    'pub const OROGEN_PROVINCE_STAGE_VERSION: u32 = 2;\nconst OROGEN_PROVINCE_NAMESPACE: &str = "worldgen:geology:tectonic-orogen-provinces:v2";',
    'pub const OROGEN_PROVINCE_STAGE_VERSION: u32 = 3;\nconst OROGEN_PROVINCE_NAMESPACE: &str = "worldgen:geology:tectonic-orogen-provinces:v3";',
)
replace_once(
    orogen,
    '''    match kind {\n        OrogenProvinceKind::ContinentalCollision => 170.0 + maturity * 190.0 + weakness * 90.0,\n        OrogenProvinceKind::CollisionalPlateau => 300.0 + maturity * 310.0 + weakness * 130.0,\n        OrogenProvinceKind::CordilleranArc => 120.0 + maturity * 110.0 + weakness * 55.0,\n        OrogenProvinceKind::IslandArc => 90.0 + maturity * 95.0 + weakness * 45.0,\n        OrogenProvinceKind::TerraneAccretion => 135.0 + maturity * 145.0 + weakness * 75.0,\n        OrogenProvinceKind::TranspressionalOrogen => 85.0 + maturity * 100.0 + weakness * 45.0,\n    }''',
    '''    match kind {\n        // Collision systems need room for a range + hinterland + fold/thrust transition.\n        // The v2 widths were narrow enough that the range itself became a coloured boundary line.\n        OrogenProvinceKind::ContinentalCollision => 220.0 + maturity * 260.0 + weakness * 110.0,\n        OrogenProvinceKind::CollisionalPlateau => 380.0 + maturity * 360.0 + weakness * 150.0,\n        OrogenProvinceKind::CordilleranArc => 135.0 + maturity * 125.0 + weakness * 60.0,\n        OrogenProvinceKind::IslandArc => 100.0 + maturity * 105.0 + weakness * 50.0,\n        OrogenProvinceKind::TerraneAccretion => 175.0 + maturity * 180.0 + weakness * 90.0,\n        OrogenProvinceKind::TranspressionalOrogen => 115.0 + maturity * 125.0 + weakness * 55.0,\n    }''',
)
replace_once(
    orogen,
    '''    let (minimum, maximum) = match kind {\n        OrogenProvinceKind::CollisionalPlateau => (220.0, 900.0),\n        OrogenProvinceKind::ContinentalCollision => (120.0, 560.0),\n        OrogenProvinceKind::TerraneAccretion => (100.0, 480.0),\n        OrogenProvinceKind::CordilleranArc => (85.0, 380.0),\n        OrogenProvinceKind::IslandArc => (65.0, 300.0),\n        OrogenProvinceKind::TranspressionalOrogen => (55.0, 260.0),\n    };''',
    '''    let (minimum, maximum) = match kind {\n        OrogenProvinceKind::CollisionalPlateau => (260.0, 1_100.0),\n        OrogenProvinceKind::ContinentalCollision => (150.0, 720.0),\n        OrogenProvinceKind::TerraneAccretion => (125.0, 620.0),\n        OrogenProvinceKind::CordilleranArc => (90.0, 420.0),\n        OrogenProvinceKind::IslandArc => (70.0, 340.0),\n        OrogenProvinceKind::TranspressionalOrogen => (70.0, 340.0),\n    };''',
)
replace_once(
    orogen,
    '''    let orthogonality = 1.0 - clamp01(traits.obliquity_deg / 90.0);\n    let core_strength = clamp01(\n        (0.22 + maturity * 0.58 + shortening * 0.34)\n            * (0.70 + orthogonality * 0.30)\n            * (0.34 + taper * 0.66),\n    );''',
    '''    let orthogonality = 1.0 - clamp01(traits.obliquity_deg / 90.0);\n    // High mountains are not guaranteed merely because a boundary is convergent.  Require\n    // accumulated shortening, then let inherited weakness/discontinuities and curvature focus\n    // the load along strike.  This keeps convergent margins mountain-prone without turning every\n    // edge of the plate graph into an equally tall ribbon.\n    let shortening_gate = smoothstep(shortening / 0.55);\n    let tectonic_focus = clamp01(\n        0.28\n            + traits.weakness * 0.18\n            + traits.age_discontinuity * 0.20\n            + traits.province_boundary * 0.10\n            + clamp01(traits.curvature_deg / 90.0) * 0.14\n            + traits.fragment_contact * 0.10,\n    );\n    let core_strength = clamp01(\n        shortening_gate\n            * (0.32 + maturity * 0.68)\n            * (0.76 + orthogonality * 0.24)\n            * (0.42 + taper * 0.58)\n            * (0.82 + tectonic_focus * 0.18),\n    );''',
)
replace_once(
    orogen,
    'let arc_sigma_km = 55.0 + 30.0 * (1.0 - resistance_value);',
    'let arc_sigma_km = 80.0 + 45.0 * (1.0 - resistance_value);',
)
replace_once(
    orogen,
    '''                let backarc_value = clamp01(\n                    source.maturity * gaussian(distance_km, arc_center_km + 220.0, 110.0) * 0.66,\n                );''',
    '''                let backarc_value = clamp01(\n                    source.maturity * gaussian(distance_km, arc_center_km + 260.0, 180.0) * 0.42,\n                );''',
)
replace_once(
    orogen,
    '''            let mountain = clamp01(\n                source.core_strength\n                    * gaussian(x, 0.10, 0.20)\n                    * side_amplitude\n                    * (0.82 + 0.18 * transmission)\n                    * reach_envelope,\n            );\n            let root_value = clamp01(\n                source.core_strength\n                    * gaussian(x, 0.18, 0.28)\n                    * if hinterland { 1.0 } else { 0.78 }\n                    * transmission,\n            );''',
    '''            let mountain = clamp01(\n                source.core_strength.powf(1.08)\n                    * gaussian(x, 0.18, 0.32)\n                    * side_amplitude\n                    * (0.82 + 0.18 * transmission)\n                    * reach_envelope,\n            );\n            let root_value = clamp01(\n                source.core_strength\n                    * gaussian(x, 0.30, 0.44)\n                    * if hinterland { 1.0 } else { 0.78 }\n                    * transmission,\n            );''',
)
replace_once(orogen, 'source.plateau_eligibility\n                        * gaussian(x, 0.50, 0.28)', 'source.plateau_eligibility\n                        * gaussian(x, 0.62, 0.40)')
replace_once(orogen, '* gaussian(x, 0.64, 0.20)\n                    * if foreland_side', '* gaussian(x, 0.86, 0.30)\n                    * if foreland_side')
replace_once(
    orogen,
    'source.maturity * source.shortening * gaussian(x, 1.04, 0.17) * reach_envelope,',
    'source.maturity * source.shortening * gaussian(x, 1.22, 0.34) * reach_envelope * 0.62,',
)
replace_once(orogen, '* gaussian(x, 0.10, 0.16),', '* gaussian(x, 0.22, 0.28),')

# --- WG-4 relief: range/core no longer implies a kilometre-deep parallel moat.
causal = "rust/interlink-worldgen/src/causal_pipeline.rs"
replace_once(
    causal,
    'pub const TECTONIC_TOPOGRAPHY_STAGE_VERSION: u32 = 13;\nconst TECTONIC_TOPOGRAPHY_NAMESPACE: &str = "terrain:boundary-localized-orogen-topography:v2";',
    'pub const TECTONIC_TOPOGRAPHY_STAGE_VERSION: u32 = 14;\nconst TECTONIC_TOPOGRAPHY_NAMESPACE: &str = "terrain:orogen-topology-and-connected-ocean:v3";',
)
replace_once(
    causal,
    '''    if subduction {\n        // Subduction topography is one-sided and arc-centred. The collision component is zero so\n        // an oceanic/continental margin cannot accidentally receive both a collision mountain and\n        // a volcanic arc at the same location.\n        let arc_relief = 1_850.0 * mountain_core\n            + volcanic_arc * (2_650.0 + 900.0 * maturity)\n            + 420.0 * fold\n            + 160.0 * intensity\n            - 900.0 * backarc\n            - 120.0 * suture;\n        return (0.0, arc_relief);\n    }''',
    '''    if subduction {\n        // Back-arc extension is a conditional broad subsidence tendency, not a mandatory marine\n        // trench behind every volcanic arc.  Tie its modest deflection to the actual arc load.\n        let arc_load = (0.55 * mountain_core + 0.45 * volcanic_arc).clamp(0.0, 1.0);\n        let backarc_deflection = 260.0 * backarc * (0.25 + 0.75 * arc_load);\n        let arc_relief = 1_500.0 * mountain_core.powf(1.08)\n            + volcanic_arc * (2_450.0 + 800.0 * maturity)\n            + 360.0 * fold\n            + 120.0 * intensity\n            - backarc_deflection;\n        return (0.0, arc_relief);\n    }''',
)
replace_once(
    causal,
    '''    let collision_relief = crust_scale\n        * tectonic_gain\n        * (5_600.0 * mountain_core\n            + 2_450.0 * root * broad_transmission\n            + 1_650.0 * plateau * broad_transmission\n            + 1_900.0 * fold\n            + 2_500.0 * transpression\n            + 260.0 * intensity\n            - 1_350.0 * foreland\n            - 180.0 * suture);''',
    '''    // Foreland subsidence is flexural response to an actual mountain load.  v13 treated the\n    // foreland index itself as a -1.35 km topographic command, creating a continuous below-sea\n    // moat beside almost every range.  Keep the basin broad and shallow unless a substantial load\n    // exists, while moving more collision relief into the crustal root/hinterland.\n    let mountain_load = (0.58 * mountain_core + 0.27 * root + 0.15 * fold).clamp(0.0, 1.0);\n    let foreland_deflection = 320.0 * foreland * mountain_load.powf(1.20);\n    let collision_relief = crust_scale\n        * tectonic_gain\n        * (4_300.0 * mountain_core.powf(1.10)\n            + 3_050.0 * root * broad_transmission\n            + 1_350.0 * plateau * broad_transmission\n            + 1_150.0 * fold\n            + 2_000.0 * transpression\n            + 180.0 * intensity\n            - foreland_deflection\n            - 70.0 * suture);''',
)
replace_once(
    causal,
    '''    let solid_f32 = solid.iter().map(|value| *value as f32).collect::<Vec<_>>();\n    let water = crate::solve_hydrostatic_surface_water(topology, &solid_f32, planet)?;''',
    '''    // Global sea level may only inundate terrain that is reached from oceanic crust through\n    // a below-water path.  Closed continental depressions can remain below the global datum\n    // without becoming magic inland ocean; WG-6 is responsible for their lake hydrology.\n    let ocean_seed_mask = inherited\n        .crust_kind\n        .iter()\n        .map(|kind| u8::from(*kind == CRUST_OCEANIC))\n        .collect::<Vec<_>>();\n    let water = crate::surface_water::solve_hydrostatic_surface_water_connected_f64(\n        topology,\n        &solid,\n        planet,\n        &ocean_seed_mask,\n    )?;''',
)

# --- Connected-ocean hydrostatics.  Keep the existing generic solver for callers that genuinely
#     want a pure global-elevation fill, and add a topology-aware variant for WG-4.
surface = "rust/interlink-worldgen/src/surface_water.rs"
replace_once(
    surface,
    'use crate::{GeodesicTopology, PlanetPhysicalParameters, WorldgenError};',
    'use crate::{GeodesicTopology, PlanetPhysicalParameters, WorldgenError};\nuse std::collections::VecDeque;',
)
marker = '''fn surface_hash(solid_elevation_m: &[f64]) -> u64 {'''
insert = r'''fn water_volume_at_level_masked(
    elevation_m: &[f64],
    areas_sr: &[f64],
    radius_m: f64,
    sea_level_m: f64,
    active_mask: &[u8],
) -> f64 {
    elevation_m
        .iter()
        .zip(areas_sr.iter())
        .zip(active_mask.iter())
        .map(|((elevation, area_sr), active)| {
            if *active == 0 {
                0.0
            } else {
                (sea_level_m - *elevation).max(0.0) * *area_sr * radius_m * radius_m
            }
        })
        .sum()
}

fn solve_sea_level_masked(
    elevation_m: &[f64],
    areas_sr: &[f64],
    planet: PlanetPhysicalParameters,
    active_mask: &[u8],
) -> (Option<f64>, f64, f64) {
    let target = planet.surface_water_volume_m3();
    if target == 0.0 {
        return (None, 0.0, 0.0);
    }
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    for (index, elevation) in elevation_m.iter().enumerate() {
        if active_mask[index] != 0 {
            minimum = minimum.min(*elevation);
            maximum = maximum.max(*elevation);
        }
    }
    if !minimum.is_finite() {
        return solve_sea_level(elevation_m, areas_sr, planet);
    }
    let mut low = minimum - 1.0;
    let mut high = maximum + planet.equivalent_global_water_depth_m() + 1.0;
    while water_volume_at_level_masked(elevation_m, areas_sr, planet.radius_m, high, active_mask)
        < target
    {
        high += (high - low).max(1_000.0);
    }
    for _ in 0..96 {
        let middle = (low + high) * 0.5;
        let volume = water_volume_at_level_masked(
            elevation_m,
            areas_sr,
            planet.radius_m,
            middle,
            active_mask,
        );
        if volume < target {
            low = middle;
        } else {
            high = middle;
        }
    }
    let sea_level = (low + high) * 0.5;
    let solved = water_volume_at_level_masked(
        elevation_m,
        areas_sr,
        planet.radius_m,
        sea_level,
        active_mask,
    );
    let error = ((solved - target) / target).abs();
    (Some(sea_level), solved, error)
}

fn connected_ocean_mask_at_level(
    topology: &GeodesicTopology,
    elevation_m: &[f64],
    ocean_seed_mask: &[u8],
    sea_level_m: f64,
) -> Vec<u8> {
    let count = elevation_m.len();
    let mut connected = vec![0_u8; count];
    let mut queue = VecDeque::new();
    for sample in 0..count {
        if ocean_seed_mask[sample] != 0 && elevation_m[sample] < sea_level_m {
            connected[sample] = 1;
            queue.push_back(sample as u32);
        }
    }
    while let Some(sample) = queue.pop_front() {
        for neighbor in topology.neighbors(sample) {
            let index = *neighbor as usize;
            if connected[index] == 0 && elevation_m[index] < sea_level_m {
                connected[index] = 1;
                queue.push_back(*neighbor);
            }
        }
    }
    connected
}

fn solve_connected_sea_level(
    topology: &GeodesicTopology,
    elevation_m: &[f64],
    planet: PlanetPhysicalParameters,
    ocean_seed_mask: &[u8],
) -> (Option<f64>, f64, f64, Vec<u8>) {
    let count = elevation_m.len();
    if planet.surface_water_volume_m3() == 0.0 {
        return (None, 0.0, 0.0, vec![0; count]);
    }
    if ocean_seed_mask.iter().all(|value| *value == 0) {
        let (level, solved, error) =
            solve_sea_level(elevation_m, topology.dual_area_steradians(), planet);
        let mut active = vec![0_u8; count];
        if let Some(level) = level {
            for sample in 0..count {
                active[sample] = u8::from(elevation_m[sample] < level);
            }
        }
        return (level, solved, error, active);
    }

    // Start from the geologically oceanic reservoir.  Each iteration solves the exact water
    // inventory over the currently reached domain, then admits any additional cells connected by
    // a below-water path.  Reached basins stay active if the redistributed ocean subsequently
    // lowers below their sill; this models a basin that was actually flooded rather than
    // teleporting water into every low continental depression on the planet.
    let mut active = ocean_seed_mask.to_vec();
    for _ in 0..32 {
        let (level, _, _) = solve_sea_level_masked(
            elevation_m,
            topology.dual_area_steradians(),
            planet,
            &active,
        );
        let Some(level) = level else {
            return (None, 0.0, 0.0, vec![0; count]);
        };
        let connected = connected_ocean_mask_at_level(topology, elevation_m, ocean_seed_mask, level);
        let mut grew = false;
        for sample in 0..count {
            if connected[sample] != 0 && active[sample] == 0 {
                active[sample] = 1;
                grew = true;
            }
        }
        if !grew {
            let (level, solved, error) = solve_sea_level_masked(
                elevation_m,
                topology.dual_area_steradians(),
                planet,
                &active,
            );
            return (level, solved, error, active);
        }
    }
    let (level, solved, error) = solve_sea_level_masked(
        elevation_m,
        topology.dual_area_steradians(),
        planet,
        &active,
    );
    (level, solved, error, active)
}

'''
replace_once(surface, marker, insert + marker)

marker = '''/// Solves a hydrostatic surface-water state for a stored solid-elevation field.'''
connected_impl = r'''fn solve_hydrostatic_surface_water_connected_impl(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f64],
    planet: PlanetPhysicalParameters,
    ocean_seed_mask: &[u8],
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    validate_inputs(topology, solid_elevation_m, planet)?;
    let count = topology.metrics().sample_count as usize;
    if ocean_seed_mask.len() != count {
        return Err(WorldgenError::InvalidTopography(
            "connected-ocean seed mask is not aligned to topology",
        ));
    }

    let target_water_volume_m3 = planet.surface_water_volume_m3();
    let (sea_level_m, solved_water_volume_m3, water_volume_relative_error, active_mask) =
        solve_connected_sea_level(topology, solid_elevation_m, planet, ocean_seed_mask);

    let mut elevation_above_sea_level_m = vec![0.0_f32; count];
    let mut water_depth_m = vec![0.0_f32; count];
    let mut submerged_mask = vec![0_u8; count];
    let mut submerged_sample_count = 0_usize;
    for sample in 0..count {
        if let Some(level) = sea_level_m {
            let relative = solid_elevation_m[sample] - level;
            elevation_above_sea_level_m[sample] = relative as f32;
            if active_mask[sample] != 0 && relative < 0.0 {
                water_depth_m[sample] = (-relative) as f32;
                submerged_mask[sample] = 1;
                submerged_sample_count += 1;
            }
        } else {
            elevation_above_sea_level_m[sample] = solid_elevation_m[sample] as f32;
        }
    }

    let surface_hash = surface_hash(solid_elevation_m);
    let water_state_hash = water_state_hash(
        surface_hash,
        planet,
        sea_level_m,
        target_water_volume_m3,
        solved_water_volume_m3,
        water_volume_relative_error,
        &elevation_above_sea_level_m,
        &water_depth_m,
        &submerged_mask,
    );
    Ok(HydrostaticSurfaceWaterState {
        metrics: HydrostaticSurfaceWaterMetrics {
            sea_level_m,
            target_water_volume_m3,
            solved_water_volume_m3,
            water_volume_relative_error,
            submerged_sample_count,
            surface_hash,
            water_state_hash,
        },
        elevation_above_sea_level_m,
        water_depth_m,
        submerged_mask,
    })
}

'''
replace_once(surface, marker, connected_impl + marker)

marker = '''/// Internal f64 entry point used by WG-4 while its solid surface is still in'''
public_connected = r'''/// Solves the global ocean from geologic seed cells and expands it only through terrain that
/// is actually reachable below the solved water surface.  Closed below-datum continental basins
/// remain dry until a marine connection is physically overtopped.
pub fn solve_hydrostatic_surface_water_connected(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f32],
    planet: PlanetPhysicalParameters,
    ocean_seed_mask: &[u8],
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    let solid_elevation_m = solid_elevation_m
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    solve_hydrostatic_surface_water_connected_impl(
        topology,
        &solid_elevation_m,
        planet,
        ocean_seed_mask,
    )
}

pub(crate) fn solve_hydrostatic_surface_water_connected_f64(
    topology: &GeodesicTopology,
    solid_elevation_m: &[f64],
    planet: PlanetPhysicalParameters,
    ocean_seed_mask: &[u8],
) -> Result<HydrostaticSurfaceWaterState, WorldgenError> {
    solve_hydrostatic_surface_water_connected_impl(topology, solid_elevation_m, planet, ocean_seed_mask)
}

'''
replace_once(surface, marker, public_connected + marker)

# Add regression before the test module's final brace.
p = Path(surface)
text = p.read_text()
needle = '''    #[test]\n    fn analytical_single_cell_basin_solves_known_level() {'''
if needle not in text:
    raise SystemExit('surface water test insertion marker missing')
# append the new regression after the existing analytical test by replacing the final module brace.
new_test = r'''

    #[test]
    fn connected_ocean_does_not_teleport_into_closed_lowland() {
        let topology = build_icosphere(1).unwrap();
        let count = topology.metrics().sample_count as usize;
        let seed = 0usize;
        let seed_neighbors = topology.neighbors(seed as u32);
        let closed = (1..count)
            .find(|sample| !seed_neighbors.contains(&(*sample as u32)))
            .expect("level-1 sphere must have a sample outside the seed neighborhood");
        let mut surface = vec![2_000.0_f32; count];
        surface[seed] = -1_000.0;
        surface[closed] = -1_200.0;
        let mut ocean_seed_mask = vec![0_u8; count];
        ocean_seed_mask[seed] = 1;
        let mut planet = PlanetPhysicalParameters::earthlike_reference();
        let physical_cell_area_m2 =
            topology.dual_area_steradians()[seed] * planet.radius_m * planet.radius_m;
        let target_volume_m3 = 100.0 * physical_cell_area_m2;
        planet.surface_water_mass_kg = target_volume_m3 * planet.ocean_water_density_kg_per_m3;

        let state = solve_hydrostatic_surface_water_connected(
            &topology,
            &surface,
            planet,
            &ocean_seed_mask,
        )
        .unwrap();
        assert_eq!(state.submerged_mask[seed], 1);
        assert_eq!(state.submerged_mask[closed], 0);
        assert_eq!(state.water_depth_m[closed], 0.0);
        assert!(state.elevation_above_sea_level_m[closed] < 0.0);
        assert!(state.metrics.water_volume_relative_error < 1.0e-10);
    }
'''
if not text.endswith('\n}\n'):
    raise SystemExit('unexpected surface_water.rs ending')
p.write_text(text[:-3] + new_test + '\n}\n')

# --- Public API / engine identity.
lib = "rust/interlink-worldgen/src/lib.rs"
replace_once(
    lib,
    '''pub use surface_water::{\n    solve_hydrostatic_surface_water, HydrostaticSurfaceWaterMetrics, HydrostaticSurfaceWaterState,\n};''',
    '''pub use surface_water::{\n    solve_hydrostatic_surface_water, solve_hydrostatic_surface_water_connected,\n    HydrostaticSurfaceWaterMetrics, HydrostaticSurfaceWaterState,\n};''',
)
replace_once(lib, 'pub const WORLDGEN_ENGINE_VERSION: u32 = 13;', 'pub const WORLDGEN_ENGINE_VERSION: u32 = 14;')

# --- Turn the causal smoke into an acceptance test for the exact failure mode.
smoke = "rust/interlink-worldgen-cli/examples/tectonic_topography_cutover_smoke.rs"
replace_once(
    smoke,
    '    LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,\n    TOPOGRAPHY_STAGE_VERSION,',
    '    CrustKind, LithosphereRequest, PlanetPhysicalParameters, TectonicsRequest, TopographyRequest,\n    TOPOGRAPHY_STAGE_VERSION,',
)
replace_once(smoke, 'terrain.stage.version != 13', 'terrain.stage.version != 14')
replace_once(
    smoke,
    '''    let negative_orogen = terrain\n        .orogenic_elevation_m\n        .iter()\n        .filter(|value| **value < -100.0)\n        .count();''',
    '''    let negative_orogen = terrain\n        .orogenic_elevation_m\n        .iter()\n        .filter(|value| **value < -100.0)\n        .count();\n    let mut continental_foreland = 0usize;\n    let mut flooded_continental_foreland = 0usize;\n    let mut continental_orogen = 0usize;\n    let mut flooded_continental_orogen = 0usize;\n    for sample in 0..terrain.solid_elevation_m.len() {\n        let continental = inherited.crust_kind[sample] == CrustKind::Continental as u8;\n        if continental && inherited.foreland_basin_index[sample] > 0.20 {\n            continental_foreland += 1;\n            if terrain.submerged_mask[sample] != 0 {\n                flooded_continental_foreland += 1;\n            }\n        }\n        if continental && inherited.province_ids[sample] != 0 {\n            continental_orogen += 1;\n            if terrain.submerged_mask[sample] != 0 {\n                flooded_continental_orogen += 1;\n            }\n        }\n    }''',
)
replace_once(
    smoke,
    '''        "WG-4 boundary-localized topography: stage=v{} provinces={} active_samples={} core={} far_core={} relief(+/-)={}/{} solid={:.0}..{:.0}m clamped={} land={:.1}% province_hash={} topo_hash={}",''',
    '''        "WG-4 orogen topology: stage=v{} provinces={} active_samples={} core={} far_core={} relief(+/-)={}/{} foreland_flood={}/{} orogen_flood={}/{} solid={:.0}..{:.0}m clamped={} land={:.1}% province_hash={} topo_hash={}",''',
)
replace_once(
    smoke,
    '''        negative_orogen,\n        terrain.metrics.minimum_solid_elevation_m,''',
    '''        negative_orogen,\n        flooded_continental_foreland,\n        continental_foreland,\n        flooded_continental_orogen,\n        continental_orogen,\n        terrain.metrics.minimum_solid_elevation_m,''',
)
replace_once(
    smoke,
    '''        || far_mountain_core_samples != 0\n        || positive_orogen == 0\n        || negative_orogen == 0''',
    '''        || far_mountain_core_samples != 0\n        || positive_orogen == 0''',
)
replace_once(
    smoke,
    '''    if terrain\n        .solid_elevation_m''',
    '''    if continental_foreland > 0 && flooded_continental_foreland * 20 > continental_foreland {\n        return Err(format!(\n            "continental foreland flooding is still systematic: {}/{}",\n            flooded_continental_foreland, continental_foreland\n        ));\n    }\n    if continental_orogen > 0 && flooded_continental_orogen * 10 > continental_orogen {\n        return Err(format!(\n            "continental orogen flooding is still excessive: {}/{}",\n            flooded_continental_orogen, continental_orogen\n        ));\n    }\n    if terrain\n        .solid_elevation_m''',
)

# --- Make the connected-ocean regression blocking in normal PR CI.
ci = ".github/workflows/ci.yml"
replace_once(
    ci,
    '''      - name: Test tectonic orogen province invariants\n        run: cargo test -p interlink-worldgen orogen_provinces\n      - name: Smoke-test tectonic causal foundations''',
    '''      - name: Test tectonic orogen province invariants\n        run: cargo test -p interlink-worldgen orogen_provinces\n      - name: Test connected-ocean topology\n        run: cargo test -p interlink-worldgen connected_ocean\n      - name: Smoke-test tectonic causal foundations''',
)

Path("docs/worldgen-rewrite/OROGEN_TOPOLOGY_AND_OCEAN_CONNECTIVITY.md").write_text('''# Orogen topology and connected ocean (WG-3.6 v3 / WG-4 v14)\n\nThis pass fixes the failure exposed by `interlink-wg7c` after boundary localization: ranges had become narrow, nearly uniform boundary ribbons and the model created a broad negative lobe beside almost every range. The global hydrostatic threshold then interpreted each below-datum continental trough as ocean, producing repeated inland mountain-and-moat geometry.\n\n## Orogen topology\n\n- Major collision relief still originates from connected convergent systems, but high mountains now require accumulated shortening rather than convergence alone.\n- Inherited weakness, age/province discontinuities, fragment contacts, and boundary curvature focus the load along strike. This preserves mountain-prone convergent margins without making every boundary edge equally tall.\n- Collision deformation reach is widened enough to support range + hinterland + fold/thrust structure while remaining far below the old continent-scale province widths.\n- The mountain core, crustal root, plateau, fold/thrust belt, and transpressional response overlap more broadly, so the visible range is not a one-cell-style boundary ribbon.\n- Foreland and back-arc fields remain causal diagnostics, but their negative topographic response is now conditional on actual mountain/arc load and is hundreds rather than thousands of metres by default. Negative relief is no longer an acceptance requirement.\n\n## Connected ocean\n\nWG-4 no longer treats `solid_elevation < global_sea_level` as sufficient to create ocean water. The final initial-ocean solve starts from oceanic-crust reservoir cells and admits additional terrain only when a below-water graph path reaches it. Water is redistributed over the reached domain while preserving the configured inventory.\n\nThis means a continental basin may legitimately sit below the global ocean datum and remain dry. If marine water physically overtops a sill, the basin can join the reached water domain. Closed-basin lake state remains the responsibility of WG-6.\n\n## Blocking invariants\n\n- connected-ocean unit coverage proves that a below-datum lowland separated from the ocean seed by high terrain is not flooded;\n- the same-seed WG-4 causal smoke no longer requires negative orogenic relief;\n- fewer than 5% of continental foreland samples may be initially submerged on the canonical smoke world;\n- fewer than 10% of all continental orogenic-province samples may be initially submerged;\n- existing mountain-core boundary-localization and finite/bounded-state invariants remain blocking.\n''')

print('orogen topology rewrite applied')
