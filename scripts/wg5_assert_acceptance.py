import re
import sys
from pathlib import Path

text = Path(sys.argv[1]).read_text()
rms = float(re.search(r'final_rms_change=([0-9.eE+-]+)', text).group(1))
moisture = float(re.search(r'relative_error=([0-9.eE+-]+)', text).group(1))
print(f'acceptance final_rms_change={rms:.9g} moisture_relative_error={moisture:.9g}')
if rms > 0.08:
    raise SystemExit(f'climate did not converge: RMS {rms} > 0.08 K')
if moisture > 1.0e-8:
    raise SystemExit(f'moisture budget did not close: {moisture} > 1e-8')
