from pathlib import Path

p = Path('rust/interlink-worldgen/src/climate_calibration.rs')
s = p.read_text()
s = s.replace(
    'request.parameters.moisture_transport_substeps',
    'request.parameters.moisture_transport_minimum_substeps',
)
p.write_text(s)
