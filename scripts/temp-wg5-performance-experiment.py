from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''                for i in 0..sample_count {
                    air_mass[i] = pressure[i] / planet.surface_gravity_m_s2 * cell_area_m2[i];
                    moisture_mass[i] = humidity[i] * air_mass[i];
                }
''',
    '''                for i in 0..sample_count {
                    air_mass[i] = pressure[i] / planet.surface_gravity_m_s2 * cell_area_m2[i];
                    moisture_mass[i] = humidity[i] * air_mass[i];
                    phase_saturation_air[i] =
                        saturation_specific_humidity(temperature[i], pressure[i]);
                }
''',
    'precompute phase air saturation with air mass',
)

replace_once(
    '''                    let surface_temperature = if ocean[i] {
                        sea_surface_temperature[i]
                    } else {
                        temperature[i]
                    };
                    let saturation_surface =
                        saturation_specific_humidity(surface_temperature, pressure[i]);
''',
    '''                    let saturation_surface = if ocean[i] {
                        saturation_specific_humidity(sea_surface_temperature[i], pressure[i])
                    } else {
                        phase_saturation_air[i]
                    };
''',
    'reuse air saturation for land evaporation',
)

replace_once(
    '''                // Temperature and pressure remain fixed throughout all moisture
                // substeps in this orbital phase. Saturation humidity therefore
                // only needs to be evaluated once per cell per phase.
                for i in 0..sample_count {
                    phase_saturation_air[i] =
                        saturation_specific_humidity(temperature[i], pressure[i]);
                }
''',
    '''                // Temperature and pressure remain fixed throughout moisture
                // transport and condensation, so the air-saturation field computed
                // above is reused for the entire phase.
''',
    'remove duplicate saturation fill',
)

replace_once(
    '''                    let saturation_air = saturation_specific_humidity(temperature[i], pressure[i]);
''',
    '''                    let saturation_air = phase_saturation_air[i];
''',
    'reuse air saturation for condensation',
)

path.write_text(text)
