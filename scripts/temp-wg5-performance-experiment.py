from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''                precipitation_mass_phase.fill(0.0);

                // Bulk-aerodynamic evaporation is expressed as a surface mass flux
''',
    '''                precipitation_mass_phase.fill(0.0);

                // Temperature and pressure remain fixed throughout the moisture
                // calculations in this orbital phase. Cache atmospheric saturation
                // humidity once and reuse it for land evaporation, convergence
                // precipitation, and final condensation.
                for i in 0..sample_count {
                    phase_saturation_air[i] =
                        saturation_specific_humidity(temperature[i], pressure[i]);
                }

                // Bulk-aerodynamic evaporation is expressed as a surface mass flux
''',
    'move phase saturation cache before evaporation',
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
    'reuse atmospheric saturation for land evaporation',
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
    '''''',
    'remove old phase saturation cache location',
)

replace_once(
    '''                    let saturation_air = saturation_specific_humidity(temperature[i], pressure[i]);
                    let threshold = saturation_air * parameters.condensation_relative_humidity;
''',
    '''                    let saturation_air = phase_saturation_air[i];
                    let threshold = saturation_air * parameters.condensation_relative_humidity;
''',
    'reuse atmospheric saturation for final condensation',
)

path.write_text(text)
