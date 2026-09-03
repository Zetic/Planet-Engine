from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '    let terrain_values = terrain_height_m.clone();\n',
    '',
    'terrain clone',
)
replace_once(
    '            &terrain_values,\n',
    '            &terrain_height_m,\n',
    'terrain gradient source',
)

scratch_anchor = '''    let mut maximum_ocean_divergence_residual = 0.0_f64;

    for year in 0..parameters.maximum_spinup_years {
'''
scratch_insert = '''    let mut maximum_ocean_divergence_residual = 0.0_f64;

    // Phase/year workspaces are allocated once and reused. WG-5 executes these
    // paths hundreds of times during spin-up, so per-phase Vec allocation and
    // cloning are pure overhead rather than part of the physical model.
    let mut start_temperature = vec![0.0; sample_count];
    let mut start_sst = vec![0.0; sample_count];
    let mut previous_temperature = vec![0.0; sample_count];
    let mut previous_sst = vec![0.0; sample_count];
    let mut next_sst = vec![0.0; sample_count];
    let mut absorbed_surface_energy_w_m2 = vec![0.0; sample_count];
    let mut radiative_target = vec![0.0; sample_count];
    let mut air_mass = vec![0.0; sample_count];
    let mut moisture_mass = vec![0.0; sample_count];
    let mut precipitation_mass_phase = vec![0.0; sample_count];
    let mut requested_ocean_evaporation_mass = vec![0.0; sample_count];

    for year in 0..parameters.maximum_spinup_years {
'''
replace_once(scratch_anchor, scratch_insert, 'workspace insertion')

replace_once(
    '''        let start_temperature = temperature.clone();
        let start_sst = sea_surface_temperature.clone();
''',
    '''        start_temperature.copy_from_slice(&temperature);
        start_sst.copy_from_slice(&sea_surface_temperature);
''',
    'year start clones',
)

replace_once(
    '''            let mut insolation = vec![0.0; sample_count];
            let mut absorbed_surface_energy_w_m2 = vec![0.0; sample_count];
            let mut radiative_target = vec![0.0; sample_count];
''',
    '',
    'phase radiation allocations',
)
replace_once(
    '                insolation[i] = solar;\n',
    '',
    'write-only insolation buffer',
)
replace_once(
    '            let previous_temperature = temperature.clone();\n',
    '            previous_temperature.copy_from_slice(&temperature);\n',
    'temperature clone',
)
replace_once(
    '''            if atmosphere_exists {
                let temperature_for_gradient = temperature.clone();
                for i in 0..sample_count {
                    let (gradient_east, gradient_north) = scalar_gradient(
                        topology,
                        &temperature_for_gradient,
''',
    '''            if atmosphere_exists {
                for i in 0..sample_count {
                    let (gradient_east, gradient_north) = scalar_gradient(
                        topology,
                        &temperature,
''',
    'temperature gradient clone',
)

replace_once(
    '            let previous_sst = sea_surface_temperature.clone();\n',
    '            previous_sst.copy_from_slice(&sea_surface_temperature);\n',
    'sst previous clone',
)
replace_once(
    '            let mut next_sst = previous_sst.clone();\n',
    '            next_sst.copy_from_slice(&previous_sst);\n',
    'sst next clone',
)
replace_once(
    '            sea_surface_temperature = next_sst;\n',
    '            sea_surface_temperature.copy_from_slice(&next_sst);\n',
    'sst replacement copy',
)

replace_once(
    '''            if atmosphere_exists {
                let mut air_mass = vec![0.0; sample_count];
                let mut moisture_mass = vec![0.0; sample_count];
                for i in 0..sample_count {
''',
    '''            if atmosphere_exists {
                for i in 0..sample_count {
''',
    'air moisture allocations',
)
replace_once(
    '''                let mut phase_evaporation = 0.0;
                let mut phase_precipitation = 0.0;
                let mut precipitation_mass_phase = vec![0.0; sample_count];

                // Bulk-aerodynamic evaporation is expressed as a surface mass flux
''',
    '''                let mut phase_evaporation = 0.0;
                let mut phase_precipitation = 0.0;
                precipitation_mass_phase.fill(0.0);

                // Bulk-aerodynamic evaporation is expressed as a surface mass flux
''',
    'phase precipitation allocation',
)
replace_once(
    '''                let mut requested_ocean_evaporation_mass = vec![0.0; sample_count];
                let mut requested_ocean_evaporation_total = 0.0;
''',
    '''                requested_ocean_evaporation_mass.fill(0.0);
                let mut requested_ocean_evaporation_total = 0.0;
''',
    'evaporation allocation',
)

path.write_text(text)
