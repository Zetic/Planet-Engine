from pathlib import Path

# The main rewrite updates current-protocol assertions. These two forms are also current
# protocol pins but have different syntax; update only those exact contracts.
replacements = {
    "tests/wg6Drainage.test.ts": [
        ("assert.equal(command.protocolVersion, 19);", "assert.equal(command.protocolVersion, 20);"),
    ],
    "tests/worldgenCompositeViews.test.ts": [
        ("assert.match(protocol, /WORLDGEN_PROTOCOL_VERSION = 19/);", "assert.match(protocol, /WORLDGEN_PROTOCOL_VERSION = 20/);"),
    ],
}

for path, pairs in replacements.items():
    p = Path(path)
    text = p.read_text()
    for old, new in pairs:
        if old not in text:
            raise SystemExit(f"expected current protocol pin not found in {path}: {old}")
        text = text.replace(old, new, 1)
    p.write_text(text)

print("remaining v20 protocol test pins updated")
