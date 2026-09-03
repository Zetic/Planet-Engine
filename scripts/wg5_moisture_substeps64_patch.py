from pathlib import Path

p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()
s = s.replace('moisture_transport_maximum_substeps: 32,', 'moisture_transport_maximum_substeps: 64,')
s = s.replace('self.moisture_transport_maximum_substeps > 32', 'self.moisture_transport_maximum_substeps > 64')
s = s.replace('moisture transport substep bounds must be within 1 through 32', 'moisture transport substep bounds must be within 1 through 64')
p.write_text(s)
