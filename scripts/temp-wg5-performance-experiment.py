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

    for _ in 0..usize::from(parameters.atmospheric_heat_solver_iterations) {
''',
    'capture atmospheric initial residual',
)

replace_once(
    '''        if !next_rho.is_finite() || next_rho <= 1.0e-18 {
            break;
        }
        let beta = next_rho / rho;
''',
    '''        if !next_rho.is_finite() || next_rho <= 1.0e-18 {
            break;
        }
        if initial_rho.is_finite()
            && initial_rho > 0.0
            && next_rho / initial_rho <= 1.0e-3
        {
            break;
        }
        let beta = next_rho / rho;
''',
    'add conservative atmospheric relative residual exit',
)

path.write_text(text)
