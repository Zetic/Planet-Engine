from pathlib import Path

for path in ["index.html", "worldgen-lab.html"]:
    p = Path(path)
    s = p.read_text()
    replacements = [
        ('data-label="Prevailing winds"> Prevailing winds', 'data-label="Seasonal prevailing winds"> Seasonal prevailing winds'),
        ('data-label="Surface currents"> Surface currents', 'data-label="Seasonal surface ocean currents"> Seasonal surface ocean currents'),
        ('data-label="Tectonic boundaries"> Tectonic boundaries', 'data-label="Fine tectonic boundaries"> Fine tectonic boundaries'),
        ('data-label="Geological boundaries"> Geological boundaries', 'data-label="Fine geological regimes"> Fine geological regimes'),
    ]
    for old, new in replacements:
        if old not in s:
            raise SystemExit(f"missing overlay label anchor in {path}: {old}")
        s = s.replace(old, new, 1)
    p.write_text(s)

p = Path("tests/wg4Topography.test.ts")
s = p.read_text()
s = s.replace("protocol v8", "protocol v9")
s = s.replace("assert.equal(WORLDGEN_PROTOCOL_VERSION, 8);", "assert.equal(WORLDGEN_PROTOCOL_VERSION, 9);", 1)
s = s.replace("{ protocolVersion: 8, requestId: 77", "{ protocolVersion: 9, requestId: 77", 1)
p.write_text(s)

p = Path("tests/worldgenRewrite.test.ts")
s = p.read_text()
s = s.replace("const PROTOCOL = 8;", "const PROTOCOL = 9;", 1)
s = s.replace("browser protocol v8", "browser protocol v9", 1)
p.write_text(s)

print("Updated intentional protocol-v9 regressions and preserved legacy overlay labels")
