from pathlib import Path

path = Path('rust/interlink-worldgen/src/lakes.rs')
text = path.read_text()

old_lakes = """    let lakes = lake_records_by_depression
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
"""
new_lakes = """    let depression_has_lake = lake_records_by_depression
        .iter()
        .map(Option::is_some)
        .collect::<Vec<_>>();
    let lakes = lake_records_by_depression
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
"""
if old_lakes not in text:
    raise SystemExit('expected lake-record collection block not found')
text = text.replace(old_lakes, new_lakes, 1)

old_route = """    for &sample in drainage_order {
        let i = sample as usize;
        if lake_fraction_exact[i] > 0.0 {
            continue;
        }
        let r = receiver[i];
"""
new_route = """    for &sample in drainage_order {
        let i = sample as usize;
        let depression = depression_id[i];
        if depression != INVALID_SAMPLE_ID && depression_has_lake[depression as usize] {
            continue;
        }
        let r = receiver[i];
"""
if old_route not in text:
    raise SystemExit('expected realized-routing block not found')
text = text.replace(old_route, new_route, 1)

old_terminal = """        if submerged_mask[i] != 0
            || (submerged_mask[i] == 0
                && receiver[i] == INVALID_SAMPLE_ID
                && lake_fraction_exact[i] <= 0.0)
        {
            terminal_realized_discharge_m3_s += realized_accum_m3_s[i];
        }
"""
new_terminal = """        let depression = depression_id[i];
        let active_lake_depression =
            depression != INVALID_SAMPLE_ID && depression_has_lake[depression as usize];
        if submerged_mask[i] != 0
            || (submerged_mask[i] == 0
                && receiver[i] == INVALID_SAMPLE_ID
                && !active_lake_depression)
        {
            terminal_realized_discharge_m3_s += realized_accum_m3_s[i];
        }
"""
if old_terminal not in text:
    raise SystemExit('expected terminal-discharge block not found')
text = text.replace(old_terminal, new_terminal, 1)

marker = """    #[test]
    fn wet_basin_reaches_spill_and_routes_only_residual_outflow() {
"""
if marker not in text:
    raise SystemExit('expected WG-6C test marker not found')

new_test = r'''
    #[test]
    fn active_depression_blocks_escape_topology_outside_the_wet_lake_surface() {
        let topology = TestTopology::chain(5);
        let core = solve_lakes_core(
            &topology,
            &[100.0, 0.0, 20.0, 10.0, -10.0],
            &[0, 0, 0, 0, 1],
            &[0.0; 5],
            &[0.0, 1000.0, 0.0, 0.0, 0.0],
            &[2, 2, 3, 4, INVALID_SAMPLE_ID],
            &[0, 1, 2, 3],
            &[
                INVALID_SAMPLE_ID,
                0,
                0,
                INVALID_SAMPLE_ID,
                INVALID_SAMPLE_ID,
            ],
            &[100.0, 30.0, 30.0, 10.0, 0.0],
            &[depression()],
            &[0.1, 0.0, 0.0, 0.0, 0.0],
            1.0,
            1.0,
            LakeParameters::default(),
        )
        .unwrap();
        assert_eq!(core.lakes.len(), 1);
        assert_eq!(core.lakes[0].kind, LAKE_KIND_ENDORHEIC);
        assert!(core.lake_fraction[1] > 0.0);
        assert_eq!(core.lake_fraction[2], 0.0);
        assert!(core.terminal_realized_discharge_m3_s < 1.0e-9);
        assert!(core.water_balance_relative_error < 1.0e-9);
    }

'''
insert_at = text.rfind('\n}')
if insert_at < 0:
    raise SystemExit('module terminator not found')
text = text[:insert_at] + '\n' + new_test + text[insert_at:]

path.write_text(text)
print('patched WG-6C active-depression routing and regression')
