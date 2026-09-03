from pathlib import Path

p = Path('rust/interlink-worldgen-cli/examples/climate_calibration.rs')
s = p.read_text()
needle = '''    println!(\n        "hydrology_mm_year precip_mean={:.3} precip_p95={:.3} pet_mean={:.3} p_over_e={:.6}",\n        report.mean_annual_precipitation_mm,\n        report.p95_annual_precipitation_mm,\n        report.mean_potential_evaporation_mm,\n        report.precipitation_to_evaporation_ratio\n    );\n'''
replacement = needle + '''    println!(\n        "water_budget_kg evaporation={:.6e} precipitation={:.6e} relative_error={:.6e}",\n        climate.metrics.global_evaporation_kg,\n        climate.metrics.global_precipitation_kg,\n        climate.metrics.moisture_budget_relative_error\n    );\n'''
if needle not in s:
    raise SystemExit('hydrology output anchor missing')
s = s.replace(needle, replacement, 1)
p.write_text(s)
