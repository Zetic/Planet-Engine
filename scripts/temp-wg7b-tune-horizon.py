from pathlib import Path

path = Path("rust/interlink-worldgen/src/evolution.rs")
text = path.read_text()
old = "            maximum_geomorphic_years: 50_000.0,"
new = "            maximum_geomorphic_years: 250_000.0,"
if text.count(old) != 1:
    raise SystemExit(f"expected one default horizon target, found {text.count(old)}")
text = text.replace(old, new, 1)

old_test = '''        let p = TerrainEvolutionParameters {
            maximum_geomorphic_years: 50_000.0,
            maximum_resolved_elevation_change_m: 100.0,
            ..TerrainEvolutionParameters::default()
        };
        assert_eq!(adaptive_duration_years(0.0, p), 0.0);
        assert!((adaptive_duration_years(0.001, p) - 50_000.0).abs() < 1.0e-9);
        assert!((adaptive_duration_years(0.01, p) - 10_000.0).abs() < 1.0e-9);'''
new_test = '''        let p = TerrainEvolutionParameters {
            maximum_geomorphic_years: 250_000.0,
            maximum_resolved_elevation_change_m: 100.0,
            ..TerrainEvolutionParameters::default()
        };
        assert_eq!(adaptive_duration_years(0.0, p), 0.0);
        assert!((adaptive_duration_years(0.001, p) - 100_000.0).abs() < 1.0e-9);
        assert!((adaptive_duration_years(0.0001, p) - 250_000.0).abs() < 1.0e-9);
        assert!((adaptive_duration_years(0.01, p) - 10_000.0).abs() < 1.0e-9);'''
if text.count(old_test) != 1:
    raise SystemExit("could not locate adaptive-horizon unit test")
text = text.replace(old_test, new_test, 1)
path.write_text(text)
