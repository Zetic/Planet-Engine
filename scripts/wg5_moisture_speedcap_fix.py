from pathlib import Path

p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()
head, climate_parameters = s.split('impl ClimateParameters {', 1)
validate, parameter_hash = climate_parameters.split('    pub fn parameter_hash(&self) -> u64 {', 1)
validate = validate.replace(
    '            self.moisture_transport_cfl_limit,\n            self.maximum_climatological_moisture_transport_speed_m_s,\n            self.convergence_precipitation_relative_humidity,',
    '            self.moisture_transport_cfl_limit,\n            self.convergence_precipitation_relative_humidity,',
    1,
)
if 'self.maximum_climatological_moisture_transport_speed_m_s' not in parameter_hash.split('        ] {', 1)[0]:
    parameter_hash = parameter_hash.replace(
        '            self.moisture_transport_cfl_limit,\n            self.convergence_precipitation_relative_humidity,',
        '            self.moisture_transport_cfl_limit,\n            self.maximum_climatological_moisture_transport_speed_m_s,\n            self.convergence_precipitation_relative_humidity,',
        1,
    )
s = (
    head
    + 'impl ClimateParameters {'
    + validate
    + '    pub fn parameter_hash(&self) -> u64 {'
    + parameter_hash
)
p.write_text(s)
