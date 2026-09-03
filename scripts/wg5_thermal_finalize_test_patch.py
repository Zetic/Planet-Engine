from pathlib import Path

p = Path('rust/interlink-worldgen/tests/climate_ensemble.rs')
s = p.read_text()
old = '    no_transport_request.parameters.atmospheric_heat_relaxation = 0.0;\n'
new = '    no_transport_request.parameters.atmospheric_heat_diffusivity_m2_s = 1.0;\n'
if old not in s:
    raise SystemExit('missing atmospheric transport regression anchor')
p.write_text(s.replace(old, new, 1))
