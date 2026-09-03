from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''fn daily_mean_insolation(latitude: f64, declination: f64, stellar_flux: f64) -> f64 {
    let x = -latitude.tan() * declination.tan();
    let hour_angle = if x >= 1.0 {
        0.0
    } else if x <= -1.0 {
        std::f64::consts::PI
    } else {
        x.acos()
    };
    let value = stellar_flux / std::f64::consts::PI
        * (hour_angle * latitude.sin() * declination.sin()
            + latitude.cos() * declination.cos() * hour_angle.sin());
    value.max(0.0)
}
''',
    '''fn daily_mean_insolation(latitude: f64, declination: f64, stellar_flux: f64) -> f64 {
    let x = -latitude.tan() * declination.tan();
    let hour_angle = if x >= 1.0 {
        0.0
    } else if x <= -1.0 {
        std::f64::consts::PI
    } else {
        x.acos()
    };
    let value = stellar_flux / std::f64::consts::PI
        * (hour_angle * latitude.sin() * declination.sin()
            + latitude.cos() * declination.cos() * hour_angle.sin());
    value.max(0.0)
}

#[derive(Clone, Copy, Debug)]
struct InsolationPhaseGeometry {
    declination_sin: f64,
    declination_cos: f64,
    declination_tan: f64,
    stellar_flux_w_m2: f64,
    phase_cos: f64,
    phase_sin: f64,
}

fn daily_mean_insolation_cached(
    latitude_sin: f64,
    latitude_cos: f64,
    latitude_tan: f64,
    phase: InsolationPhaseGeometry,
) -> f64 {
    let x = -latitude_tan * phase.declination_tan;
    let hour_angle = if x >= 1.0 {
        0.0
    } else if x <= -1.0 {
        std::f64::consts::PI
    } else {
        x.acos()
    };
    let value = phase.stellar_flux_w_m2 / std::f64::consts::PI
        * (hour_angle * latitude_sin * phase.declination_sin
            + latitude_cos * phase.declination_cos * hour_angle.sin());
    value.max(0.0)
}
''',
    'add cached insolation helper',
)

replace_once(
    '''    let mut latitude = vec![0.0; sample_count];
    let mut east_bases = vec![[0.0; 3]; sample_count];
''',
    '''    let mut latitude = vec![0.0; sample_count];
    let mut latitude_sin = vec![0.0; sample_count];
    let mut latitude_cos = vec![0.0; sample_count];
    let mut latitude_tan = vec![0.0; sample_count];
    let mut east_bases = vec![[0.0; 3]; sample_count];
''',
    'allocate latitude trig cache',
)

replace_once(
    '''        latitude[i] = position[2].clamp(-1.0, 1.0).asin();
        let basis = tangent_basis(position)?;
''',
    '''        latitude[i] = position[2].clamp(-1.0, 1.0).asin();
        latitude_sin[i] = latitude[i].sin();
        latitude_cos[i] = latitude[i].cos();
        latitude_tan[i] = latitude[i].tan();
        let basis = tangent_basis(position)?;
''',
    'populate latitude trig cache',
)

replace_once(
    '''    let mut temperature = vec![0.0; sample_count];
''',
    '''    let axial_tilt_sin = planet.axial_tilt_rad.sin();
    let mut insolation_phases = Vec::with_capacity(phase_count);
    for phase in 0..phase_count {
        let mean_longitude = TWO_PI * phase as f64 / phase_count as f64;
        let (solar_longitude, distance_factor) = solve_orbital_forcing(
            mean_longitude,
            physical.orbital_eccentricity,
            physical.longitude_of_periapsis_rad,
        );
        let declination = (axial_tilt_sin * solar_longitude.sin()).asin();
        insolation_phases.push(InsolationPhaseGeometry {
            declination_sin: declination.sin(),
            declination_cos: declination.cos(),
            declination_tan: declination.tan(),
            stellar_flux_w_m2: planet.stellar_flux_w_m2 * distance_factor,
            phase_cos: mean_longitude.cos(),
            phase_sin: mean_longitude.sin(),
        });
    }

    let mut temperature = vec![0.0; sample_count];
''',
    'precompute orbital phase geometry',
)

replace_once(
    '''        let lat_factor = latitude[i].sin().abs().powf(1.45);
''',
    '''        let lat_factor = latitude_sin[i].abs().powf(1.45);
''',
    'reuse latitude sine for initialization',
)

replace_once(
    '''        for phase in 0..phase_count {
            let mean_longitude = TWO_PI * phase as f64 / phase_count as f64;
            let (solar_longitude, distance_factor) = solve_orbital_forcing(
                mean_longitude,
                physical.orbital_eccentricity,
                physical.longitude_of_periapsis_rad,
            );
            let declination = (planet.axial_tilt_rad.sin() * solar_longitude.sin()).asin();
            let phase_angle = mean_longitude;
            let phase_cos = phase_angle.cos();
            let phase_sin = phase_angle.sin();

            for i in 0..sample_count {
                let solar = daily_mean_insolation(
                    latitude[i],
                    declination,
                    planet.stellar_flux_w_m2 * distance_factor,
                );
''',
    '''        for phase in 0..phase_count {
            let insolation_phase = insolation_phases[phase];
            let phase_cos = insolation_phase.phase_cos;
            let phase_sin = insolation_phase.phase_sin;

            for i in 0..sample_count {
                let solar = daily_mean_insolation_cached(
                    latitude_sin[i],
                    latitude_cos[i],
                    latitude_tan[i],
                    insolation_phase,
                );
''',
    'reuse cached insolation geometry in spinup',
)

path.write_text(text)
