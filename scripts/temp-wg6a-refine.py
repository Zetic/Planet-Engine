from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"marker not found in {path}: {old[:120]!r}")
    target.write_text(text.replace(old, new, 1))


# Keep older browser command fixtures aligned with the protocol bump.
for test_path in ["tests/wg4Topography.test.ts", "tests/wg5Climate.test.ts"]:
    target = Path(test_path)
    text = target.read_text().replace("protocolVersion: 10", "protocolVersion: 11")
    target.write_text(text)

# Ocean outlets are water surfaces, not seafloor cells. Priority-Flood therefore
# seeds ocean cells at the solved sea level so bathymetric depth cannot steer a
# coastal land cell toward an arbitrary deeper ocean neighbor.
replace_once(
    "rust/interlink-worldgen/src/drainage.rs",
    "fn priority_flood<T: PlanetTopology>(\n    topology: &T,\n    elevation_m: &[f32],\n    submerged_mask: &[u8],\n) -> (Vec<f64>, Vec<u32>, Vec<u32>, Vec<u8>) {",
    "fn priority_flood<T: PlanetTopology>(\n    topology: &T,\n    elevation_m: &[f32],\n    submerged_mask: &[u8],\n    sea_level_m: Option<f64>,\n) -> (Vec<f64>, Vec<u32>, Vec<u32>, Vec<u8>) {",
)
replace_once(
    "rust/interlink-worldgen/src/drainage.rs",
    "            escape[i] = f64::from(elevation_m[i]);\n            outlet_kind[i] = DRAINAGE_OUTLET_OCEAN;",
    "            escape[i] = sea_level_m.unwrap_or_else(|| f64::from(elevation_m[i]));\n            outlet_kind[i] = DRAINAGE_OUTLET_OCEAN;",
)
replace_once(
    "rust/interlink-worldgen/src/drainage.rs",
    "fn solve_core<T: PlanetTopology>(\n    topology: &T,\n    elevation_m: &[f32],\n    submerged_mask: &[u8],\n    radius_m: f64,\n) -> Result<DrainageCore, &'static str> {\n    validate_inputs(topology, elevation_m, submerged_mask, radius_m)?;\n    let count = topology.sample_count() as usize;\n    let (escape, flood_parent, flood_rank, mut outlet_kind) =\n        priority_flood(topology, elevation_m, submerged_mask);",
    "fn solve_core<T: PlanetTopology>(\n    topology: &T,\n    elevation_m: &[f32],\n    submerged_mask: &[u8],\n    radius_m: f64,\n    sea_level_m: Option<f64>,\n) -> Result<DrainageCore, &'static str> {\n    validate_inputs(topology, elevation_m, submerged_mask, radius_m)?;\n    let count = topology.sample_count() as usize;\n    if submerged_mask.iter().any(|value| *value != 0)\n        && sea_level_m.is_none_or(|value| !value.is_finite())\n    {\n        return Err(\"drainage ocean routing requires a finite solved sea level\");\n    }\n    let (escape, flood_parent, flood_rank, mut outlet_kind) =\n        priority_flood(topology, elevation_m, submerged_mask, sea_level_m);",
)
replace_once(
    "rust/interlink-worldgen/src/drainage.rs",
    "        planet.radius_m,\n    )\n    .map_err(WorldgenError::InvalidHydrology)?;",
    "        planet.radius_m,\n        topography.metrics.sea_level_m,\n    )\n    .map_err(WorldgenError::InvalidHydrology)?;",
)

# Propagate the terminal kind to every land sample after resolving the outlet.
replace_once(
    "rust/interlink-worldgen/src/drainage.rs",
    "    let mut unique_outlets = BTreeMap::<u32, u32>::new();\n    for i in 0..count {\n        if submerged_mask[i] == 0 {\n            let outlet = outlet_sample[i];\n            if !unique_outlets.contains_key(&outlet) {\n                let next = unique_outlets.len() as u32;\n                unique_outlets.insert(outlet, next);\n            }\n        }\n    }",
    "    for i in 0..count {\n        if submerged_mask[i] != 0 {\n            continue;\n        }\n        let outlet = outlet_sample[i];\n        let resolved_kind = outlet_kind[outlet as usize];\n        if resolved_kind == DRAINAGE_OUTLET_NONE {\n            return Err(\"drainage outlet kind was not resolved at terminal sample\");\n        }\n        outlet_kind[i] = resolved_kind;\n    }\n\n    let mut sorted_outlets = outlet_sample\n        .iter()\n        .enumerate()\n        .filter(|(sample, _)| submerged_mask[*sample] == 0)\n        .map(|(_, outlet)| *outlet)\n        .collect::<Vec<_>>();\n    sorted_outlets.sort_unstable();\n    sorted_outlets.dedup();\n    let unique_outlets = sorted_outlets\n        .into_iter()\n        .enumerate()\n        .map(|(id, outlet)| (outlet, id as u32))\n        .collect::<BTreeMap<_, _>>();",
)

# Update direct core tests for the explicit sea-surface input.
target = Path("rust/interlink-worldgen/src/drainage.rs")
text = target.read_text()
text = text.replace(
    "solve_core(&topology, &[3.0, 2.0, 1.0, -1.0], &[0, 0, 0, 1], 1.0).unwrap()",
    "solve_core(&topology, &[3.0, 2.0, 1.0, -1.0], &[0, 0, 0, 1], 1.0, Some(0.0)).unwrap()",
)
text = text.replace(
    "solve_core(&topology, &elevation, &[0, 0, 0, 1], 1.0).unwrap()",
    "solve_core(&topology, &elevation, &[0, 0, 0, 1], 1.0, Some(0.0)).unwrap()",
)
text = text.replace(
    "solve_core(&topology, &[2.0, 2.0, 2.0, -1.0], &[0, 0, 0, 1], 1.0).unwrap()",
    "solve_core(&topology, &[2.0, 2.0, 2.0, -1.0], &[0, 0, 0, 1], 1.0, Some(0.0)).unwrap()",
)
text = text.replace(
    "solve_core(&topology, &[4.0, 3.0, 1.0, 2.0], &[0, 0, 0, 0], 1.0).unwrap()",
    "solve_core(&topology, &[4.0, 3.0, 1.0, 2.0], &[0, 0, 0, 0], 1.0, None).unwrap()",
)
target.write_text(text)

replace_once(
    "rust/interlink-worldgen/src/drainage.rs",
    "        assert_eq!(result.receiver, vec![1, 2, 3, INVALID_SAMPLE_ID]);\n        assert_eq!(result.outlet_sample[0], 3);\n        assert_eq!(result.basins.len(), 1);",
    "        assert_eq!(result.receiver, vec![1, 2, 3, INVALID_SAMPLE_ID]);\n        assert_eq!(result.outlet_sample[0], 3);\n        assert_eq!(result.outlet_kind[0], DRAINAGE_OUTLET_OCEAN);\n        assert_eq!(result.outlet_kind[1], DRAINAGE_OUTLET_OCEAN);\n        assert_eq!(result.outlet_kind[2], DRAINAGE_OUTLET_OCEAN);\n        assert_eq!(result.basins.len(), 1);",
)
replace_once(
    "rust/interlink-worldgen/src/drainage.rs",
    "        assert_eq!(result.outlet_kind[2], DRAINAGE_OUTLET_INTERNAL);\n        assert!(result.outlet_sample.iter().all(|value| *value == 2));",
    "        assert_eq!(result.outlet_kind[2], DRAINAGE_OUTLET_INTERNAL);\n        assert!(result\n            .outlet_kind\n            .iter()\n            .all(|value| *value == DRAINAGE_OUTLET_INTERNAL));\n        assert!(result.outlet_sample.iter().all(|value| *value == 2));",
)

# Regression: different seafloor depths must not choose the coastal outlet.
insert_test = r'''

    #[test]
    fn ocean_bathymetry_does_not_steer_coastal_land_routing() {
        let topology = TestTopology {
            neighbors: vec![vec![1, 2], vec![0], vec![0]],
            distances: vec![vec![1.0e-3, 1.0e-3], vec![1.0e-3], vec![1.0e-3]],
            areas: vec![1.0; 3],
        };
        let result = solve_core(
            &topology,
            &[10.0, -100.0, -5_000.0],
            &[0, 1, 1],
            1.0,
            Some(0.0),
        )
        .unwrap();
        assert_eq!(result.escape_elevation_m[1], 0.0);
        assert_eq!(result.escape_elevation_m[2], 0.0);
        assert_eq!(result.receiver[0], 1);
        assert_eq!(result.outlet_sample[0], 1);
        assert_eq!(result.outlet_kind[0], DRAINAGE_OUTLET_OCEAN);
    }
'''
replace_once(
    "rust/interlink-worldgen/src/drainage.rs",
    "\n    #[test]\n    fn dry_planet_uses_global_minimum_as_internal_terminal() {",
    insert_test + "\n    #[test]\n    fn dry_planet_uses_global_minimum_as_internal_terminal() {",
)

# Generated-planet acceptance also requires every land cell's resolved kind to
# agree with its terminal outlet.
replace_once(
    "rust/interlink-worldgen/tests/drainage_topology.rs",
    "        assert_ne!(drainage_a.outlet_sample[sample], INVALID_SAMPLE_ID);\n        let receiver = drainage_a.receiver[sample];",
    "        assert_ne!(drainage_a.outlet_sample[sample], INVALID_SAMPLE_ID);\n        let outlet = drainage_a.outlet_sample[sample] as usize;\n        assert_eq!(drainage_a.outlet_kind[sample], drainage_a.outlet_kind[outlet]);\n        let receiver = drainage_a.receiver[sample];",
)

# Document the two semantic guarantees explicitly.
replace_once(
    "docs/worldgen-rewrite/WG6_HYDROLOGY.md",
    "Ocean cells are terminal outlets. The graph is flooded outward from terminal outlets in increasing escape elevation.",
    "Ocean cells are terminal outlets. They enter Priority-Flood at the solved sea-surface elevation, not at bathymetric seafloor elevation, so deep offshore relief cannot steer coastal drainage. The graph is flooded outward from terminal outlets in increasing escape elevation.",
)
replace_once(
    "docs/worldgen-rewrite/WG6_HYDROLOGY.md",
    "- `outlet_sample` and `outlet_kind`;\n- `basin_id`;",
    "- `outlet_sample` and resolved `outlet_kind` for every land sample;\n- canonical `basin_id`, assigned by ascending terminal outlet sample;",
)
