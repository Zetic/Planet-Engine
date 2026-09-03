from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()
text = text.replace(
    '        let mut squared_change = 0.0;\n',
    '        let mut squared_change = 0.0;\n        let mut squared_temperature_change = 0.0;\n        let mut squared_sst_change = 0.0;\n',
    1,
)
needle = '            squared_change += delta_temperature * delta_temperature + 0.5 * delta_sst * delta_sst;\n'
replacement = (
    '            squared_temperature_change += delta_temperature * delta_temperature;\n'
    '            squared_sst_change += delta_sst * delta_sst;\n'
    + needle
)
assert needle in text
text = text.replace(needle, replacement, 1)
needle = '        spinup_years = year + 1;\n'
replacement = (
    '        let temperature_rms = (squared_temperature_change / sample_count as f64).sqrt();\n'
    '        let sst_rms = (squared_sst_change / sample_count as f64).sqrt();\n'
    '        eprintln!("WG5_SPINUP year={} combined_rms={:.9} temperature_rms={:.9} sst_rms={:.9}", year + 1, final_temperature_rms_change, temperature_rms, sst_rms);\n'
    + needle
)
assert needle in text
text = text.replace(needle, replacement, 1)
path.write_text(text)
