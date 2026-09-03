from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

# The caller immediately scans every cell again for convergence precipitation.
# Keep the exact per-cell moisture update expression, but perform it at the
# start of that caller pass so one full-grid traversal disappears per moisture
# substep.
replace_once(
    '''    for i in 0..moisture_mass.len() {
        moisture_mass[i] = (moisture_mass[i] + delta[i]).max(0.0);
    }
    (limited_donors, active_donors)
}
''',
    '''    (limited_donors, active_donors)
}
''',
    'remove standalone moisture delta application pass',
)

replace_once(
    '''                    moisture_transport_limited_donor_steps += limited_donors;
                    moisture_transport_active_donor_steps += active_donors;
                    for i in 0..sample_count {
                        if moisture_transport_delta[i] <= 0.0 || air_mass[i] <= 0.0 {
''',
    '''                    moisture_transport_limited_donor_steps += limited_donors;
                    moisture_transport_active_donor_steps += active_donors;
                    for i in 0..sample_count {
                        moisture_mass[i] =
                            (moisture_mass[i] + moisture_transport_delta[i]).max(0.0);
                        if moisture_transport_delta[i] <= 0.0 || air_mass[i] <= 0.0 {
''',
    'fuse moisture delta application with convergence precipitation pass',
)

path.write_text(text)
