from pathlib import Path
import os

cap = os.environ.get('MOISTURE_SPEED_CAP', '2.0')
p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()

s = s.replace(
    'pub moisture_transport_cfl_limit: f64,',
    'pub moisture_transport_cfl_limit: f64,\n    pub maximum_climatological_moisture_transport_speed_m_s: f64,',
    1,
)
s = s.replace(
    'moisture_transport_cfl_limit: 0.90,',
    f'moisture_transport_cfl_limit: 0.90,\n            maximum_climatological_moisture_transport_speed_m_s: {cap},',
    1,
)
# Base patch may emit a different exact CFL literal in ad-hoc sweeps.
if 'maximum_climatological_moisture_transport_speed_m_s' not in s.split('impl Default for ClimateParameters', 1)[1].split('}', 1)[0]:
    s = s.replace(
        'moisture_transport_cfl_limit: 0.9,',
        f'moisture_transport_cfl_limit: 0.9,\n            maximum_climatological_moisture_transport_speed_m_s: {cap},',
        1,
    )

# Ensure the speed cap is a strictly-positive model parameter and participates in identity.
s = s.replace(
    'self.moisture_transport_cfl_limit,\n            self.orographic_precipitation_strength,',
    'self.moisture_transport_cfl_limit,\n            self.maximum_climatological_moisture_transport_speed_m_s,\n            self.orographic_precipitation_strength,',
    1,
)
s = s.replace(
    'self.moisture_transport_cfl_limit,\n            self.convergence_precipitation_relative_humidity,',
    'self.moisture_transport_cfl_limit,\n            self.maximum_climatological_moisture_transport_speed_m_s,\n            self.convergence_precipitation_relative_humidity,',
    1,
)

old = '''fn advect_moisture_substep(
    edges: &[AtmosphericMoistureEdge],
    moisture_mass: &mut [f64],
    cell_area_m2: &[f64],
    wind_east: &[f64],
    wind_north: &[f64],
    substep_seconds: f64,
    cfl_limit: f64,
) -> (Vec<f64>, usize, usize) {'''
new = '''fn advect_moisture_substep(
    edges: &[AtmosphericMoistureEdge],
    moisture_mass: &mut [f64],
    cell_area_m2: &[f64],
    wind_east: &[f64],
    wind_north: &[f64],
    substep_seconds: f64,
    cfl_limit: f64,
    maximum_speed_m_s: f64,
) -> (Vec<f64>, usize, usize) {'''
if old not in s:
    raise SystemExit('advect signature anchor missing for speed cap')
s = s.replace(old, new, 1)

# There are two transport normal-speed calculations: adaptive demand and actual flux.
needle = 'let normal_speed = 0.5 * (outward_a - outward_b);'
replacement_phase = '''let normal_speed = (0.5 * (outward_a - outward_b)).clamp(
            -parameters.maximum_climatological_moisture_transport_speed_m_s,
            parameters.maximum_climatological_moisture_transport_speed_m_s,
        );'''
if s.count(needle) < 2:
    raise SystemExit('expected two moisture normal-speed anchors')
s = s.replace(needle, replacement_phase, 1)
replacement_advect = '''let normal_speed = (0.5 * (outward_a - outward_b))
            .clamp(-maximum_speed_m_s, maximum_speed_m_s);'''
s = s.replace(needle, replacement_advect, 1)

s = s.replace(
    '''                        parameters.moisture_transport_cfl_limit,
                    );''',
    '''                        parameters.moisture_transport_cfl_limit,
                        parameters.maximum_climatological_moisture_transport_speed_m_s,
                    );''',
    1,
)
p.write_text(s)
