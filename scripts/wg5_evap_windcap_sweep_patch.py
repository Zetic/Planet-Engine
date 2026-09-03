from pathlib import Path
import os

cap = os.environ.get('EVAP_WIND_CAP', '8.0')
p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()
needle = 'let wind_speed = norm2(wind_east[i], wind_north[i]).max(1.0);'
replacement = f'let wind_speed = norm2(wind_east[i], wind_north[i]).max(1.0).min({cap});'
count = s.count(needle)
if count != 1:
    raise SystemExit(f'expected one evaporation wind-speed anchor, found {count}')
s = s.replace(needle, replacement, 1)
p.write_text(s)
