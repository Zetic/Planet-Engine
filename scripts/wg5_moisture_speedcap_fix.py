from pathlib import Path

p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()
pre, post = s.split('    pub fn parameter_hash(&self) -> u64 {', 1)
pre = pre.replace(
    '            self.moisture_transport_cfl_limit,\n            self.maximum_climatological_moisture_transport_speed_m_s,\n            self.convergence_precipitation_relative_humidity,',
    '            self.moisture_transport_cfl_limit,\n            self.convergence_precipitation_relative_humidity,',
    1,
)
if 'self.maximum_climatological_moisture_transport_speed_m_s' not in post:
    post = post.replace(
        '            self.moisture_transport_cfl_limit,\n            self.convergence_precipitation_relative_humidity,',
        '            self.moisture_transport_cfl_limit,\n            self.maximum_climatological_moisture_transport_speed_m_s,\n            self.convergence_precipitation_relative_humidity,',
        1,
    )
s = pre + '    pub fn parameter_hash(&self) -> u64 {' + post
p.write_text(s)
