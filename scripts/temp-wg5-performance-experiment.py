from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()

def replace_once(old: str, new: str, label: str) -> None:
    global text
    if old not in text:
        raise SystemExit(f'{label} anchor not found')
    text = text.replace(old, new, 1)

old_fn = '''fn advect_moisture_substep(
    edges: &[PhaseAtmosphericMoistureEdge],
    moisture_mass: &mut [f64],
    cell_area_m2: &[f64],
    substep_seconds: f64,
    cfl_limit: f64,
    requested_outflow: &mut [f64],
    donor_scale: &mut [f64],
    delta: &mut [f64],
) -> (usize, usize) {
    requested_outflow.fill(0.0);
    donor_scale.fill(1.0);
    delta.fill(0.0);

    for edge in edges {
        let donor_column_moisture = moisture_mass[edge.donor] / cell_area_m2[edge.donor].max(1.0);
        let mass = donor_column_moisture
            * edge.normal_speed_abs_m_s
            * edge.interface_length_m
            * substep_seconds;
        if mass > 0.0 {
            requested_outflow[edge.donor] += mass;
        }
    }

    let mut active_donors = 0usize;
    let mut limited_donors = 0usize;
    for i in 0..moisture_mass.len() {
        if requested_outflow[i] <= 0.0 {
            continue;
        }
        active_donors += 1;
        let allowed = moisture_mass[i] * cfl_limit;
        if requested_outflow[i] > allowed {
            donor_scale[i] = allowed / requested_outflow[i];
            limited_donors += 1;
        }
    }

    for edge in edges {
        let donor_column_moisture = moisture_mass[edge.donor] / cell_area_m2[edge.donor].max(1.0);
        let mass = donor_column_moisture
            * edge.normal_speed_abs_m_s
            * edge.interface_length_m
            * substep_seconds;
        let transfer = mass * donor_scale[edge.donor];
        delta[edge.donor] -= transfer;
        delta[edge.receiver] += transfer;
    }
    for i in 0..moisture_mass.len() {
        moisture_mass[i] = (moisture_mass[i] + delta[i]).max(0.0);
    }
    (limited_donors, active_donors)
}
'''
new_fn = '''fn advect_moisture_substep(
    edges: &[PhaseAtmosphericMoistureEdge],
    moisture_mass: &mut [f64],
    cell_area_m2: &[f64],
    substep_seconds: f64,
    cfl_limit: f64,
    column_moisture: &mut [f64],
    requested_edge_mass: &mut [f64],
    requested_outflow: &mut [f64],
    donor_scale: &mut [f64],
    delta: &mut [f64],
) -> (usize, usize) {
    debug_assert!(requested_edge_mass.len() >= edges.len());
    requested_outflow.fill(0.0);
    donor_scale.fill(1.0);
    delta.fill(0.0);

    // Moisture mass is unchanged during the request pass, so compute each
    // donor column density once per cell rather than repeating the same
    // division for every incident transport edge (and then repeating it again
    // during application).
    for i in 0..moisture_mass.len() {
        column_moisture[i] = moisture_mass[i] / cell_area_m2[i].max(1.0);
    }
    for (edge_index, edge) in edges.iter().enumerate() {
        let mass = column_moisture[edge.donor]
            * edge.normal_speed_abs_m_s
            * edge.interface_length_m
            * substep_seconds;
        requested_edge_mass[edge_index] = mass;
        if mass > 0.0 {
            requested_outflow[edge.donor] += mass;
        }
    }

    let mut active_donors = 0usize;
    let mut limited_donors = 0usize;
    for i in 0..moisture_mass.len() {
        if requested_outflow[i] <= 0.0 {
            continue;
        }
        active_donors += 1;
        let allowed = moisture_mass[i] * cfl_limit;
        if requested_outflow[i] > allowed {
            donor_scale[i] = allowed / requested_outflow[i];
            limited_donors += 1;
        }
    }

    // Reuse the exact request mass calculated above. Donor scaling does not
    // mutate moisture until this pass, so recomputing the mass here was pure
    // duplicate floating-point work.
    for (edge_index, edge) in edges.iter().enumerate() {
        let transfer = requested_edge_mass[edge_index] * donor_scale[edge.donor];
        delta[edge.donor] -= transfer;
        delta[edge.receiver] += transfer;
    }
    for i in 0..moisture_mass.len() {
        moisture_mass[i] = (moisture_mass[i] + delta[i]).max(0.0);
    }
    (limited_donors, active_donors)
}
'''
replace_once(old_fn, new_fn, 'moisture substep function')

workspace_old = '''    let mut moisture_requested_outflow = vec![0.0; sample_count];
    let mut moisture_donor_scale = vec![1.0; sample_count];
    let mut moisture_transport_delta = vec![0.0; sample_count];
'''
workspace_new = '''    let mut moisture_column_mass_per_m2 = vec![0.0; sample_count];
    let mut moisture_requested_edge_mass = vec![0.0; atmospheric_moisture_edges.len()];
    let mut moisture_requested_outflow = vec![0.0; sample_count];
    let mut moisture_donor_scale = vec![1.0; sample_count];
    let mut moisture_transport_delta = vec![0.0; sample_count];
    let mut phase_saturation_air = vec![0.0; sample_count];
'''
replace_once(workspace_old, workspace_new, 'moisture workspace')

call_old = '''                        substep_seconds,
                        parameters.moisture_transport_cfl_limit,
                        &mut moisture_requested_outflow,
                        &mut moisture_donor_scale,
                        &mut moisture_transport_delta,
'''
call_new = '''                        substep_seconds,
                        parameters.moisture_transport_cfl_limit,
                        &mut moisture_column_mass_per_m2,
                        &mut moisture_requested_edge_mass,
                        &mut moisture_requested_outflow,
                        &mut moisture_donor_scale,
                        &mut moisture_transport_delta,
'''
replace_once(call_old, call_new, 'moisture substep call')

sat_anchor = '''                let substep_seconds = phase_seconds / f64::from(moisture_substeps);
                for _ in 0..usize::from(moisture_substeps) {
'''
sat_insert = '''                let substep_seconds = phase_seconds / f64::from(moisture_substeps);
                // Temperature and pressure remain fixed throughout all moisture
                // substeps in this orbital phase. Saturation humidity therefore
                // only needs to be evaluated once per cell per phase.
                for i in 0..sample_count {
                    phase_saturation_air[i] =
                        saturation_specific_humidity(temperature[i], pressure[i]);
                }
                for _ in 0..usize::from(moisture_substeps) {
'''
replace_once(sat_anchor, sat_insert, 'phase saturation insertion')

sat_old = '''                        let saturation_air =
                            saturation_specific_humidity(temperature[i], pressure[i]);
                        if saturation_air <= 1.0e-12 {
'''
sat_new = '''                        let saturation_air = phase_saturation_air[i];
                        if saturation_air <= 1.0e-12 {
'''
replace_once(sat_old, sat_new, 'substep saturation lookup')

path.write_text(text)
