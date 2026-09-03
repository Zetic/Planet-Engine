from pathlib import Path
import os

fraction = os.environ.get('EVAP_ENERGY_FRACTION', '0.45')
p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()

s = s.replace(
    'const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;',
    'const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;\nconst LATENT_HEAT_VAPORIZATION_J_PER_KG: f64 = 2_450_000.0;',
    1,
)
s = s.replace(
    'pub evaporation_bulk_transfer_coefficient: f64,',
    'pub evaporation_bulk_transfer_coefficient: f64,\n    pub evaporation_energy_fraction: f64,',
    1,
)
s = s.replace(
    'evaporation_bulk_transfer_coefficient: 0.0015,',
    f'evaporation_bulk_transfer_coefficient: 0.0015,\n            evaporation_energy_fraction: {fraction},',
    1,
)

# Explicit bounded validation; this parameter is a fraction, not a transport rate.
anchor = '''        if self.moisture_transport_minimum_substeps == 0
            || self.moisture_transport_maximum_substeps < self.moisture_transport_minimum_substeps
            || self.moisture_transport_maximum_substeps > 64
        {
            return Err("moisture transport substep bounds must be within 1 through 64");
        }'''
if anchor not in s:
    raise SystemExit('adaptive validation anchor missing')
s = s.replace(
    anchor,
    anchor + '''
        if !self.evaporation_energy_fraction.is_finite()
            || self.evaporation_energy_fraction <= 0.0
            || self.evaporation_energy_fraction > 1.0
        {
            return Err("evaporation energy fraction must be finite and within (0, 1]");
        }''',
    1,
)

# Include the closure coefficient in model identity without accidentally adding it
# to any earlier validation arrays.
pre, post = s.split('    pub fn parameter_hash(&self) -> u64 {', 1)
needle = '            self.evaporation_bulk_transfer_coefficient,\n'
if needle not in post:
    raise SystemExit('parameter hash evaporation anchor missing')
post = post.replace(
    needle,
    needle + '            self.evaporation_energy_fraction,\n',
    1,
)
s = pre + '    pub fn parameter_hash(&self) -> u64 {' + post

# Retain the same surface-energy field used by the thermal solve so evaporation
# closes against the thermal model instead of an independent climatology.
s = s.replace(
    '            let mut radiative_target = vec![0.0; sample_count];',
    '            let mut absorbed_surface_energy_w_m2 = vec![0.0; sample_count];\n            let mut radiative_target = vec![0.0; sample_count];',
    1,
)
needle = '''                let absorbed = (solar * (1.0 - effective_albedo)
                    + planet.internal_heat_flux_w_per_m2)
                    .max(0.0);'''
if needle not in s:
    raise SystemExit('absorbed energy anchor missing')
s = s.replace(
    needle,
    needle + '\n                absorbed_surface_energy_w_m2[i] = absorbed;',
    1,
)

old = '''                for i in 0..sample_count {
                    if air_mass[i] <= 0.0 {
                        humidity[i] = 0.0;
                        continue;
                    }
                    let q = moisture_mass[i] / air_mass[i];
                    let wind_speed = norm2(wind_east[i], wind_north[i]).max(1.0);
                    let surface_temperature = if ocean[i] {
                        sea_surface_temperature[i]
                    } else {
                        temperature[i]
                    };
                    let saturation_surface =
                        saturation_specific_humidity(surface_temperature, pressure[i]);
                    let density = pressure[i]
                        / (specific_gas_constant * temperature[i].max(120.0));
                    let evaporation_flux = density
                        * parameters.evaporation_bulk_transfer_coefficient
                        * wind_speed
                        * (saturation_surface - q).max(0.0);
                    let potential_mass =
                        evaporation_flux * cell_area_m2[i] * phase_seconds;
                    potential_evaporation_mass_year[i] += potential_mass;
                    if ocean[i] {
                        moisture_mass[i] += potential_mass;
                        phase_evaporation += potential_mass;
                    }
                }'''
new = '''                let mut requested_ocean_evaporation_mass = vec![0.0; sample_count];
                let mut requested_ocean_evaporation_total = 0.0;
                let mut ocean_absorbed_power_w = 0.0;
                for i in 0..sample_count {
                    if air_mass[i] <= 0.0 {
                        humidity[i] = 0.0;
                        continue;
                    }
                    let q = moisture_mass[i] / air_mass[i];
                    let wind_speed = norm2(wind_east[i], wind_north[i]).max(1.0);
                    let surface_temperature = if ocean[i] {
                        sea_surface_temperature[i]
                    } else {
                        temperature[i]
                    };
                    let saturation_surface =
                        saturation_specific_humidity(surface_temperature, pressure[i]);
                    let density = pressure[i]
                        / (specific_gas_constant * temperature[i].max(120.0));
                    let evaporation_flux = density
                        * parameters.evaporation_bulk_transfer_coefficient
                        * wind_speed
                        * (saturation_surface - q).max(0.0);
                    let potential_mass =
                        evaporation_flux * cell_area_m2[i] * phase_seconds;
                    potential_evaporation_mass_year[i] += potential_mass;
                    if ocean[i] {
                        requested_ocean_evaporation_mass[i] = potential_mass;
                        requested_ocean_evaporation_total += potential_mass;
                        ocean_absorbed_power_w +=
                            absorbed_surface_energy_w_m2[i] * cell_area_m2[i];
                    }
                }
                let energy_limited_ocean_evaporation_mass =
                    parameters.evaporation_energy_fraction
                        * ocean_absorbed_power_w
                        * phase_seconds
                        / LATENT_HEAT_VAPORIZATION_J_PER_KG;
                let evaporation_energy_scale = if requested_ocean_evaporation_total > 0.0 {
                    (energy_limited_ocean_evaporation_mass
                        / requested_ocean_evaporation_total)
                        .clamp(0.0, 1.0)
                } else {
                    1.0
                };
                for i in 0..sample_count {
                    if !ocean[i] {
                        continue;
                    }
                    let evaporation_mass =
                        requested_ocean_evaporation_mass[i] * evaporation_energy_scale;
                    moisture_mass[i] += evaporation_mass;
                    phase_evaporation += evaporation_mass;
                }'''
if old not in s:
    raise SystemExit('bulk evaporation block anchor missing')
s = s.replace(old, new, 1)

p.write_text(s)
