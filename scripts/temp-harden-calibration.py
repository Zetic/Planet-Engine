from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"missing expected text in {path}: {old[:120]!r}")
    target.write_text(text.replace(old, new, 1))


replace(
    "src/worldgen/calibrationPacket.ts",
    "    schema: WORLD_CALIBRATION_SCHEMA,\n    run: {",
    "    schema: WORLD_CALIBRATION_SCHEMA,\n"
    "    fidelity: {\n"
    "      source: 'github-pages',\n"
    "      canonical_dual_cell_area: false,\n"
    "      complete_internal_lake_budget: false,\n"
    "      approximation_notes: [\n"
    "        'continental component area uses equal-sample area because protocol v18 does not transport dual-cell area',\n"
    "        'derived topography percentiles and means are unweighted sample summaries',\n"
    "        'per-lake gross inflow and evaporation are unavailable in protocol v18 and remain null',\n"
    "      ],\n"
    "    },\n"
    "    run: {",
)
replace(
    "src/worldgen/calibrationPacket.ts",
    "    `Schema: \\`${packet.schema}\\``,\n    `Seed: \\`${seed}\\` · L${result.coarseLevel}",
    "    `Schema: \\`${packet.schema}\\``,\n"
    "    'Fidelity: GitHub Pages packet · equal-sample continental area / unweighted derived topography summaries · partial per-lake budget',\n"
    "    `Seed: \\`${seed}\\` · L${result.coarseLevel}",
)

rust = Path("rust/interlink-worldgen/src/world_calibration.rs")
text = rust.read_text()
json_anchor = """        let _ = writeln!(
            out,
            \"  \\\"schema\\\": \\\"{}@{}\\\",\",
            WORLD_CALIBRATION_SCHEMA_ID, WORLD_CALIBRATION_SCHEMA_VERSION
        );
        let _ = writeln!(out, \"  \\\"run\\\": {{\");
"""
json_replacement = """        let _ = writeln!(
            out,
            \"  \\\"schema\\\": \\\"{}@{}\\\",\",
            WORLD_CALIBRATION_SCHEMA_ID, WORLD_CALIBRATION_SCHEMA_VERSION
        );
        let _ = writeln!(out, \"  \\\"fidelity\\\": {{\\\"source\\\":\\\"native\\\",\\\"canonical_dual_cell_area\\\":true,\\\"complete_internal_lake_budget\\\":true,\\\"approximation_notes\\\":[]}},\");
        let _ = writeln!(out, \"  \\\"run\\\": {{\");
"""
if json_anchor not in text:
    raise SystemExit("native JSON schema anchor not found")
text = text.replace(json_anchor, json_replacement, 1)
md_anchor = """        let _ = writeln!(
            out,
            \"Schema: `{}@{}`\",
            WORLD_CALIBRATION_SCHEMA_ID, WORLD_CALIBRATION_SCHEMA_VERSION
        );
        let _ = writeln!(
            out,
            \"Seed: `{}` · L{} → L{} · {} plates · {} samples · engine v{}\",
"""
md_replacement = """        let _ = writeln!(
            out,
            \"Schema: `{}@{}`\",
            WORLD_CALIBRATION_SCHEMA_ID, WORLD_CALIBRATION_SCHEMA_VERSION
        );
        let _ = writeln!(out, \"Fidelity: native canonical dual-cell areas + complete internal lake budget\");
        let _ = writeln!(
            out,
            \"Seed: `{}` · L{} → L{} · {} plates · {} samples · engine v{}\",
"""
if md_anchor not in text:
    raise SystemExit("native Markdown schema anchor not found")
rust.write_text(text.replace(md_anchor, md_replacement, 1))

controller = Path("src/worldgen/diagnostics/worldgenClimateLabStandalone.ts")
text = controller.read_text()
marker = "let current: WorldgenClimateResult | null = null;"
if marker not in text:
    raise SystemExit("current result declaration not found")
text = text.replace(
    marker,
    marker + "\nlet currentCalibrationRequest: { seed: string; plateCount: number } | null = null;",
    1,
)
text = text.replace(
    "function calibrationFileStem(): string {\n  const normalized = seed.value.trim()",
    "function calibrationFileStem(value: string): string {\n  const normalized = value.trim()",
    1,
)
text = text.replace(
    "  if (!current) return;\n  try {\n    await navigator.clipboard.writeText(worldCalibrationMarkdown(current, seed.value, Number(plates.value)));",
    "  if (!current || !currentCalibrationRequest) return;\n  try {\n    await navigator.clipboard.writeText(worldCalibrationMarkdown(current, currentCalibrationRequest.seed, currentCalibrationRequest.plateCount));",
    1,
)
text = text.replace(
    "  if (!current) return;\n  const blob = new Blob([worldCalibrationJson(current, seed.value, Number(plates.value))]",
    "  if (!current || !currentCalibrationRequest) return;\n  const blob = new Blob([worldCalibrationJson(current, currentCalibrationRequest.seed, currentCalibrationRequest.plateCount)]",
    1,
)
text = text.replace(
    "anchor.download = `planet-calibration-${calibrationFileStem()}.json`;",
    "anchor.download = `planet-calibration-${calibrationFileStem(currentCalibrationRequest.seed)}.json`;",
    1,
)
text = text.replace(
    "    current = loaded;\n    copyCalibration.disabled = false;",
    "    current = loaded;\n    currentCalibrationRequest = { seed: request.seed, plateCount: request.plateCount };\n    copyCalibration.disabled = false;",
    1,
)
controller.write_text(text)

test = Path("tests/calibrationPacket.test.ts")
text = test.read_text()
text = text.replace(
    "    assert.match(packet, /RANKED_LIMIT = 8/);",
    "    assert.match(packet, /RANKED_LIMIT = 8/);\n"
    "    assert.match(packet, /canonical_dual_cell_area: false/);\n"
    "    assert.match(packet, /approximation_notes/);",
    1,
)
text = text.replace(
    "    assert.match(controller, /worldCalibrationJson/);",
    "    assert.match(controller, /worldCalibrationJson/);\n"
    "    assert.match(controller, /currentCalibrationRequest/);\n"
    "    assert.match(controller, /seed: request.seed, plateCount: request.plateCount/);",
    1,
)
test.write_text(text)

docs = Path("docs/worldgen-rewrite/CALIBRATION_OBSERVABILITY.md")
text = docs.read_text()
old = "The browser does not export raw per-cell arrays. It aggregates them locally first. Because protocol v18 does not transport dual-cell area or every internal per-lake water-budget term, the Pages packet estimates continental component area from equal sample area and leaves unavailable per-lake terms null."
new = "The browser does not export raw per-cell arrays. It aggregates them locally first. Because protocol v18 does not transport dual-cell area or every internal per-lake water-budget term, the Pages packet estimates continental component area from equal sample area, uses unweighted sample summaries for derived topography percentiles/means, and leaves unavailable per-lake terms null. These limitations are recorded in the packet itself under `fidelity`, so a detached JSON/Markdown export remains self-describing."
if old not in text:
    raise SystemExit("docs fidelity paragraph not found")
docs.write_text(text.replace(old, new, 1))
