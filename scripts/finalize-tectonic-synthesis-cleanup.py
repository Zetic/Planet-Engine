from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    assert count == 1, f"{label}: expected 1 occurrence, found {count}"
    return text.replace(old, new, 1)


# Oblique subduction is still subduction: preserve arc morphology and let obliquity
# contribute transpressional structure without reclassifying the whole source as a
# collision-style transpressional orogen.
orogen = Path('rust/interlink-worldgen/src/orogen_provinces.rs')
text = orogen.read_text()
text = replace_once(
    text,
    '    if mean_obliquity >= 62.0 {\n        return OrogenProvinceKind::TranspressionalOrogen;\n    }',
    '    if mean_obliquity >= 62.0 && first.class == BoundaryClass::Collision {\n        return OrogenProvinceKind::TranspressionalOrogen;\n    }',
    'preserve subduction arc class under high obliquity',
)
# WG-3.6 is rasterized on the coarse tectonic sphere. A physically narrow volcanic-arc
# center can otherwise fall between coarse samples and disappear entirely before fine
# refinement. Keep the center offset, but antialias the diagnostic/magmatic envelope
# broadly enough that every real subduction source survives the coarse representation.
text = replace_once(
    text,
    'gaussian(physical_distance_km, arc_center_km, arc_sigma_km * 0.78)',
    'gaussian(physical_distance_km, arc_center_km, arc_sigma_km * 1.35)',
    'coarse-grid volcanic arc antialiasing',
)
orogen.write_text(text)

# Physical output and the cumulative browser packet both change in this PR.
ci = Path('.github/workflows/ci.yml')
text = ci.read_text()
text = replace_once(
    text,
    "assert p['run']['engine_version']==16",
    "assert p['run']['engine_version']==17",
    'downstream engine gate',
)
lineage = '''      - name: Verify historical lithosphere lineage and projection\n        run: cargo run --release -p interlink-worldgen-cli --example historical_lithosphere_acceptance\n'''
lineage_with_synthesis = lineage + '''      - name: Verify modern plate topology and continental assembly\n        run: cargo test -p interlink-worldgen historical_lithosphere\n'''
text = replace_once(text, lineage, lineage_with_synthesis, 'tectonic synthesis blocking gate')
ci.write_text(text)
