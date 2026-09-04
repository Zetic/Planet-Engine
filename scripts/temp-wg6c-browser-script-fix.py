from pathlib import Path

path = Path('scripts/temp-wg6c-browser.py')
text = path.read_text()
old = "  metric(metrics, 'WG-6C lake hash', result.lakeMetrics.lakeHash);\n}\"\ntext = text.replace(metrics_marker, metrics_add, 1)"
new = "  metric(metrics, 'WG-6C lake hash', result.lakeMetrics.lakeHash);\n}\"\"\"\ntext = text.replace(metrics_marker, metrics_add, 1)"
if old not in text:
    raise SystemExit('WG-6C browser transformer syntax marker missing')
path.write_text(text.replace(old, new, 1))
