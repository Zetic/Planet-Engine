from pathlib import Path

path = Path('rust/interlink-worldgen/src/climate.rs')
text = path.read_text()
replacements = [
    ('    pub ocean_current_smoothing: f64,\n', ''),
    ('            ocean_current_smoothing: 0.18,\n', ''),
    ('            self.ocean_current_smoothing,\n', ''),
]
for old, new in replacements:
    count = text.count(old)
    assert count >= 1, f'missing expected climate parameter fragment: {old!r}'
    text = text.replace(old, new)
assert 'ocean_current_smoothing' not in text
path.write_text(text)
