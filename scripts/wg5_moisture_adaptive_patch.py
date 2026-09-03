from pathlib import Path

p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()

s = s.replace('pub moisture_transport_substeps: u8,', 'pub moisture_transport_minimum_substeps: u8,\n    pub moisture_transport_maximum_substeps: u8,')
s = s.replace('moisture_transport_substeps: 24,', 'moisture_transport_minimum_substeps: 4,\n            moisture_transport_maximum_substeps: 32,')
s = s.replace('moisture_transport_substeps: 12,', 'moisture_transport_minimum_substeps: 4,\n            moisture_transport_maximum_substeps: 32,')
s = s.replace('moisture_transport_substeps: 8,', 'moisture_transport_minimum_substeps: 4,\n            moisture_transport_maximum_substeps: 32,')
s = s.replace('moisture_transport_substeps: 4,', 'moisture_transport_minimum_substeps: 4,\n            moisture_transport_maximum_substeps: 32,')

s = s.replace('''        if self.moisture_transport_substeps == 0 || self.moisture_transport_substeps > 32 {
            return Err("moisture transport substeps must be from 1 through 32");
        }''', '''        if self.moisture_transport_minimum_substeps == 0
            || self.moisture_transport_maximum_substeps < self.moisture_transport_minimum_substeps
            || self.moisture_transport_maximum_substeps > 32
        {
            return Err("moisture transport substep bounds must be within 1 through 32");
        }''')

s = s.replace('hash = fnv_update(hash, &[self.moisture_transport_substeps]);', 'hash = fnv_update(hash, &[self.moisture_transport_minimum_substeps]);\n        hash = fnv_update(hash, &[self.moisture_transport_maximum_substeps]);')

old = '''fn advect_moisture_substep(
    edges: &[AtmosphericMoistureEdge],
    moisture_mass: &mut [f64],
    cell_area_m2: &[f64],
    wind_east: &[f64],
    wind_north: &[f64],
    substep_seconds: f64,
    cfl_limit: f64,
) -> Vec<f64> {'''
new = '''fn moisture_transport_substeps_for_phase(
    edges: &[AtmosphericMoistureEdge],
    cell_area_m2: &[f64],
    wind_east: &[f64],
    wind_north: &[f64],
    phase_seconds: f64,
    parameters: ClimateParameters,
) -> u8 {
    let mut outgoing_rate = vec![0.0; cell_area_m2.len()];
    for edge in edges {
        let outward_a = wind_east[edge.a] * edge.a_east + wind_north[edge.a] * edge.a_north;
        let outward_b = wind_east[edge.b] * edge.b_east + wind_north[edge.b] * edge.b_north;
        let normal_speed = 0.5 * (outward_a - outward_b);
        if normal_speed > 0.0 {
            outgoing_rate[edge.a] += normal_speed * edge.interface_length_m;
        } else if normal_speed < 0.0 {
            outgoing_rate[edge.b] += -normal_speed * edge.interface_length_m;
        }
    }
    let maximum_phase_courant = outgoing_rate
        .iter()
        .enumerate()
        .map(|(i, rate)| rate * phase_seconds / cell_area_m2[i].max(1.0))
        .fold(0.0_f64, f64::max);
    let required = (maximum_phase_courant / parameters.moisture_transport_cfl_limit)
        .ceil()
        .max(1.0) as u32;
    required
        .clamp(
            u32::from(parameters.moisture_transport_minimum_substeps),
            u32::from(parameters.moisture_transport_maximum_substeps),
        ) as u8
}

fn advect_moisture_substep(
    edges: &[AtmosphericMoistureEdge],
    moisture_mass: &mut [f64],
    cell_area_m2: &[f64],
    wind_east: &[f64],
    wind_north: &[f64],
    substep_seconds: f64,
    cfl_limit: f64,
) -> (Vec<f64>, usize, usize) {'''
if old not in s:
    raise SystemExit('advect signature anchor missing')
s = s.replace(old, new, 1)

s = s.replace('''    let mut donor_scale = vec![1.0; moisture_mass.len()];
    for i in 0..moisture_mass.len() {
        let allowed = moisture_mass[i] * cfl_limit;
        if requested_outflow[i] > allowed && requested_outflow[i] > 0.0 {
            donor_scale[i] = allowed / requested_outflow[i];
        }
    }''', '''    let mut donor_scale = vec![1.0; moisture_mass.len()];
    let mut active_donors = 0usize;
    let mut limited_donors = 0usize;
    for i in 0..moisture_mass.len() {
        if requested_outflow[i] <= 0.0 {
            continue;
        }
        active_donors += 1;
        let allowed = moisture_mass[i] * cfl_limit;
        if requested_outflow[i] > allowed {
            donor_scale[i] = if requested_outflow[i] > 0.0 {
                allowed / requested_outflow[i]
            } else {
                1.0
            };
            limited_donors += 1;
        }
    }''', 1)
s = s.replace('''    delta
}''', '''    (delta, limited_donors, active_donors)
}''', 1)

# Public diagnostics for the actual adaptive solve.
s = s.replace('''    pub moisture_budget_relative_error: f64,
    pub persistent_snow_area_fraction: f64,''', '''    pub moisture_budget_relative_error: f64,
    pub moisture_transport_limiter_fraction: f64,
    pub maximum_moisture_transport_substeps: u8,
    pub persistent_snow_area_fraction: f64,''', 1)

s = s.replace('''    let mut moisture_budget_error_year = 0.0;
    let mut final_temperature_rms_change''', '''    let mut moisture_budget_error_year = 0.0;
    let mut moisture_transport_limited_donor_steps = 0usize;
    let mut moisture_transport_active_donor_steps = 0usize;
    let mut maximum_moisture_transport_substeps_used = 0u8;
    let mut final_temperature_rms_change''', 1)
s = s.replace('''        moisture_budget_error_year = 0.0;
        maximum_ocean_divergence_residual = 0.0;''', '''        moisture_budget_error_year = 0.0;
        moisture_transport_limited_donor_steps = 0;
        moisture_transport_active_donor_steps = 0;
        maximum_moisture_transport_substeps_used = 0;
        maximum_ocean_divergence_residual = 0.0;''', 1)

old = '''                let substep_seconds =
                    phase_seconds / f64::from(parameters.moisture_transport_substeps);
                for _ in 0..usize::from(parameters.moisture_transport_substeps) {
                    let transport_delta = advect_moisture_substep('''
new = '''                let moisture_substeps = moisture_transport_substeps_for_phase(
                    &atmospheric_moisture_edges,
                    &cell_area_m2,
                    &wind_east,
                    &wind_north,
                    phase_seconds,
                    parameters,
                );
                maximum_moisture_transport_substeps_used =
                    maximum_moisture_transport_substeps_used.max(moisture_substeps);
                let substep_seconds = phase_seconds / f64::from(moisture_substeps);
                for _ in 0..usize::from(moisture_substeps) {
                    let (transport_delta, limited_donors, active_donors) = advect_moisture_substep('''
if old not in s:
    raise SystemExit('substep loop anchor missing')
s = s.replace(old, new, 1)

needle = '''                        parameters.moisture_transport_cfl_limit,
                    );
                    for i in 0..sample_count {'''
replace = '''                        parameters.moisture_transport_cfl_limit,
                    );
                    moisture_transport_limited_donor_steps += limited_donors;
                    moisture_transport_active_donor_steps += active_donors;
                    for i in 0..sample_count {'''
if needle not in s:
    raise SystemExit('substep result anchor missing')
s = s.replace(needle, replace, 1)

# Aggregate precipitation mechanisms per cell/phase so phase maxima and snowfall
# accounting describe total precipitation, not whichever mechanism was largest.
s = s.replace('''                let mut phase_evaporation = 0.0;
                let mut phase_precipitation = 0.0;''', '''                let mut phase_evaporation = 0.0;
                let mut phase_precipitation = 0.0;
                let mut precipitation_mass_phase = vec![0.0; sample_count];''', 1)
s = s.replace('''                        precipitation_mass_year[i] += precipitation_mass;
                        precipitation_phase_max[i] =
                            precipitation_phase_max[i].max(precipitation_mass);
                        if temperature[i] <= parameters.snow_temperature_k {
                            cold_precipitation_mass_year[i] += precipitation_mass;
                        }
                        phase_precipitation += precipitation_mass;''', '''                        precipitation_mass_phase[i] += precipitation_mass;''', 1)
s = s.replace('''                    precipitation_mass_year[i] += precipitation_mass;
                    precipitation_phase_max[i] = precipitation_phase_max[i].max(precipitation_mass);
                    if temperature[i] <= parameters.snow_temperature_k {
                        cold_precipitation_mass_year[i] += precipitation_mass;
                        snow_phase_count[i] += 1.0;
                    }
                    if ocean[i] && sea_surface_temperature[i] <= parameters.sea_ice_temperature_k {
                        sea_ice_phase_count[i] += 1.0;
                    }
                    phase_precipitation += precipitation_mass;
                    humidity[i] = (moisture_mass[i] / air_mass[i]).clamp(0.0, 0.2);''', '''                    precipitation_mass_phase[i] += precipitation_mass;
                    let phase_cell_precipitation = precipitation_mass_phase[i];
                    precipitation_mass_year[i] += phase_cell_precipitation;
                    precipitation_phase_max[i] =
                        precipitation_phase_max[i].max(phase_cell_precipitation);
                    if temperature[i] <= parameters.snow_temperature_k
                        && phase_cell_precipitation > 0.0
                    {
                        cold_precipitation_mass_year[i] += phase_cell_precipitation;
                        snow_phase_count[i] += 1.0;
                    }
                    if ocean[i] && sea_surface_temperature[i] <= parameters.sea_ice_temperature_k {
                        sea_ice_phase_count[i] += 1.0;
                    }
                    phase_precipitation += phase_cell_precipitation;
                    humidity[i] = (moisture_mass[i] / air_mass[i]).clamp(0.0, 0.2);''', 1)

# Final diagnostics.
anchor = '''    let metrics = ClimateMetrics {'''
calc = '''    let moisture_transport_limiter_fraction = if moisture_transport_active_donor_steps > 0 {
        moisture_transport_limited_donor_steps as f64
            / moisture_transport_active_donor_steps as f64
    } else {
        0.0
    };

'''
if anchor not in s:
    raise SystemExit('metrics anchor missing')
s = s.replace(anchor, calc + anchor, 1)
s = s.replace('''        moisture_budget_relative_error,
        persistent_snow_area_fraction,''', '''        moisture_budget_relative_error,
        moisture_transport_limiter_fraction,
        maximum_moisture_transport_substeps: maximum_moisture_transport_substeps_used,
        persistent_snow_area_fraction,''', 1)

p.write_text(s)

# Calibration: use runtime limiter/substep diagnostics rather than reconstructing
# the removed edge-fraction cap.
p = Path('rust/interlink-worldgen/src/climate_calibration.rs')
s = p.read_text()
s = s.replace('pub reconstructed_moisture_edge_cap_fraction: f64,', 'pub moisture_transport_limiter_fraction: f64,\n    pub maximum_moisture_transport_substeps: u8,')
# Replace any report initializer field assignment.
import re
s = re.sub(r'reconstructed_moisture_edge_cap_fraction:\s*[^,]+,', 'moisture_transport_limiter_fraction: climate.metrics.moisture_transport_limiter_fraction,\n        maximum_moisture_transport_substeps: climate.metrics.maximum_moisture_transport_substeps,', s)
p.write_text(s)

# Calibration example output.
p = Path('rust/interlink-worldgen-cli/examples/climate_calibration.rs')
s = p.read_text()
s = s.replace('''"transport_caps reconstructed_wind={:.6} reconstructed_moisture_edge={:.6}",
        report.reconstructed_wind_cap_fraction, report.reconstructed_moisture_edge_cap_fraction''', '''"transport wind_cap={:.6} moisture_limiter={:.6} moisture_max_substeps={}",
        report.reconstructed_wind_cap_fraction,
        report.moisture_transport_limiter_fraction,
        report.maximum_moisture_transport_substeps''')
p.write_text(s)
