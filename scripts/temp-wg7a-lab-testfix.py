from pathlib import Path

for filename in ["tests/wg4Topography.test.ts", "tests/wg6Drainage.test.ts"]:
    path = Path(filename)
    text = path.read_text()
    text = text.replace("through WG-6D", "through WG-7A")
    text = text.replace("THROUGH WG-6D", "THROUGH WG-7A")
    path.write_text(text)
