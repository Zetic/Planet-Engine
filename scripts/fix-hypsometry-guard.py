from pathlib import Path

p = Path('rust/interlink-worldgen/src/historical_topography.rs')
s = p.read_text()
s = s.replace(
    '// The causal WG-4 solve already identified the connected global ocean. Reuse only submerged\n    // oceanic-crust cells as seeds after the passive-margin deflection, so newly lowered shelves can\n',
    '// The causal WG-4 solve already identified the connected global ocean. Reuse only submerged\n    // oceanic-crust cells as seeds after the historical hypsometry adjustment, so newly lowered shelves can\n',
    1,
)
s = s.replace(
    '/// Persistent rifting already produces `ContinentalMargin` structure in WG-3.5. Materialize that\n/// inherited state as a bounded shelf/basin deflection after the accepted tectonic-province WG-4\n/// solve. The operation is in-place: no second `InheritedPhysicalState` is retained at L8.\n',
    '/// Persistent material state controls the final continental freeboard adjustment after the\n/// accepted tectonic-province WG-4 solve. Stable thick continental interiors receive bounded\n/// buoyancy support, while inherited passive margins retain their shelf/basin deflection. The\n/// operation is in-place: no second `InheritedPhysicalState` is retained at L8.\n',
    1,
)
s = s.replace(
    '"historical passive-margin inputs are not aligned to WG-4 topology",',
    '"historical hypsometry inputs are not aligned to WG-4 topology",',
    1,
)
old = '''    let mut state = causal_pipeline::generate_initial_topography(
        topology, inherited, boundaries, planet, request,
    )?;
    if !has_margin {
        finalize_historical_stage(&mut state, request);
        return Ok(state);
    }
'''
new = '''    let has_continental_support = (0..count)
        .any(|sample| stable_continental_buoyancy_support_m(inherited, sample) > 0.0);
    let mut state = causal_pipeline::generate_initial_topography(
        topology, inherited, boundaries, planet, request,
    )?;
    if !has_margin && !has_continental_support {
        finalize_historical_stage(&mut state, request);
        return Ok(state);
    }
'''
if old not in s:
    raise SystemExit('hypsometry early-return anchor not found')
p.write_text(s.replace(old, new, 1))
