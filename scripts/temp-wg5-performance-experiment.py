from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

replace_once(
    '''    let mut rho = residual
        .iter()
        .zip(preconditioned.iter())
        .map(|(r, z)| r * z)
        .sum::<f64>();

    for _ in 0..usize::from(parameters.atmospheric_heat_solver_iterations) {
''',
    '''    let mut rho = residual
        .iter()
        .zip(preconditioned.iter())
        .map(|(r, z)| r * z)
        .sum::<f64>();
    let initial_rho = rho;
    let mut trace_rho = rho;
    let mut iterations_used = 0usize;
    let mut threshold_hits = [usize::MAX; 4];
    const TRACE_THRESHOLDS: [f64; 4] = [1.0e-2, 1.0e-4, 1.0e-6, 1.0e-8];

    for _ in 0..usize::from(parameters.atmospheric_heat_solver_iterations) {
''',
    'instrument atmospheric solver setup',
)

replace_once(
    '''        let next_rho = residual
            .iter()
            .zip(preconditioned.iter())
            .map(|(r, z)| r * z)
            .sum::<f64>();
        if !next_rho.is_finite() || next_rho <= 1.0e-18 {
            break;
        }
''',
    '''        let next_rho = residual
            .iter()
            .zip(preconditioned.iter())
            .map(|(r, z)| r * z)
            .sum::<f64>();
        iterations_used += 1;
        trace_rho = next_rho;
        if initial_rho.is_finite() && initial_rho > 0.0 && next_rho.is_finite() {
            let ratio = next_rho / initial_rho;
            for (slot, threshold) in threshold_hits.iter_mut().zip(TRACE_THRESHOLDS) {
                if *slot == usize::MAX && ratio <= threshold {
                    *slot = iterations_used;
                }
            }
        }
        if !next_rho.is_finite() || next_rho <= 1.0e-18 {
            break;
        }
''',
    'instrument atmospheric solver iterations',
)

replace_once(
    '''    for i in 0..temperature.len() {
        temperature[i] = x[i].clamp(120.0, 355.0);
    }
}

fn exchange_air_sea_heat(
''',
    '''    if std::env::var_os("WG5_SOLVER_TRACE").is_some() {
        let final_ratio = if initial_rho.is_finite() && initial_rho > 0.0 {
            trace_rho / initial_rho
        } else {
            f64::NAN
        };
        let hit = |value: usize| if value == usize::MAX { 0 } else { value };
        eprintln!(
            "wg5_atmos_solver iterations={} ratio={:.9e} hit_1e2={} hit_1e4={} hit_1e6={} hit_1e8={}",
            iterations_used,
            final_ratio,
            hit(threshold_hits[0]),
            hit(threshold_hits[1]),
            hit(threshold_hits[2]),
            hit(threshold_hits[3]),
        );
    }
    for i in 0..temperature.len() {
        temperature[i] = x[i].clamp(120.0, 355.0);
    }
}

fn exchange_air_sea_heat(
''',
    'emit atmospheric solver trace',
)

replace_once(
    '''    let mut rho = residual
        .iter()
        .zip(preconditioned.iter())
        .map(|(r, z)| r * z)
        .sum::<f64>();
    for _ in 0..usize::from(parameters.ocean_current_correction_iterations) {
''',
    '''    let mut rho = residual
        .iter()
        .zip(preconditioned.iter())
        .map(|(r, z)| r * z)
        .sum::<f64>();
    let initial_rho = rho;
    let mut trace_rho = rho;
    let mut iterations_used = 0usize;
    let mut threshold_hits = [usize::MAX; 4];
    const TRACE_THRESHOLDS: [f64; 4] = [1.0e-2, 1.0e-4, 1.0e-6, 1.0e-8];
    for _ in 0..usize::from(parameters.ocean_current_correction_iterations) {
''',
    'instrument ocean solver setup',
)

replace_once(
    '''        let next_rho = residual
            .iter()
            .zip(preconditioned.iter())
            .map(|(r, z)| r * z)
            .sum::<f64>();
        if !next_rho.is_finite() || next_rho <= 1.0e-24 {
            break;
        }
''',
    '''        let next_rho = residual
            .iter()
            .zip(preconditioned.iter())
            .map(|(r, z)| r * z)
            .sum::<f64>();
        iterations_used += 1;
        trace_rho = next_rho;
        if initial_rho.is_finite() && initial_rho > 0.0 && next_rho.is_finite() {
            let ratio = next_rho / initial_rho;
            for (slot, threshold) in threshold_hits.iter_mut().zip(TRACE_THRESHOLDS) {
                if *slot == usize::MAX && ratio <= threshold {
                    *slot = iterations_used;
                }
            }
        }
        if !next_rho.is_finite() || next_rho <= 1.0e-24 {
            break;
        }
''',
    'instrument ocean solver iterations',
)

replace_once(
    '''    projected_divergence.fill(0.0);
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
''',
    '''    if std::env::var_os("WG5_SOLVER_TRACE").is_some() {
        let final_ratio = if initial_rho.is_finite() && initial_rho > 0.0 {
            trace_rho / initial_rho
        } else {
            f64::NAN
        };
        let hit = |value: usize| if value == usize::MAX { 0 } else { value };
        eprintln!(
            "wg5_ocean_solver iterations={} ratio={:.9e} hit_1e2={} hit_1e4={} hit_1e6={} hit_1e8={}",
            iterations_used,
            final_ratio,
            hit(threshold_hits[0]),
            hit(threshold_hits[1]),
            hit(threshold_hits[2]),
            hit(threshold_hits[3]),
        );
    }
    projected_divergence.fill(0.0);
    for (edge_index, edge) in geometry.edges.iter().enumerate() {
''',
    'emit ocean solver trace',
)

path.write_text(text)
