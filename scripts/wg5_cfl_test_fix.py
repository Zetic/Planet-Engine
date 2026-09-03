from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()
old = '''        conservative_ocean_heat_tendency(
            &geometry,
            &[20.0],
            &[300.0, 280.0],
            &[100.0, 200.0],
            &mut tendency,
        );
'''
new = '''        conservative_ocean_heat_tendency(
            &geometry,
            &[20.0],
            &[300.0, 280.0],
            &[100.0, 200.0],
            1.0,
            1.0,
            1.0,
            &mut tendency,
        );
'''
assert old in text
path.write_text(text.replace(old, new, 1))
