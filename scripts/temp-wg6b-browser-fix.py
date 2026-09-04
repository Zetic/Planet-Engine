from pathlib import Path

path = Path("src/worldgen/diagnostics/worldgenClimateLabStandalone.ts")
text = path.read_text()

replacements = {
    "result.metrics.maximumDepressionDepthM": "result.drainageMetrics.maximumDepressionDepthM",
    "result.metrics.maximumContributingAreaM2": "result.drainageMetrics.maximumContributingAreaM2",
    "result.metrics.landSampleCount": "result.drainageMetrics.landSampleCount",
    "const count = result.metrics.sampleCount;": "const count = result.drainageMetrics.sampleCount;",
    "for (let sample = 0; sample < result.metrics.sampleCount; sample += 1)": "for (let sample = 0; sample < result.drainageMetrics.sampleCount; sample += 1)",
    "`v${drainage.engineVersion} · ${result.drainageStage.id}@${result.drainageStage.version}`": "`v${result.engineVersion} · ${result.drainageStage.id}@${result.drainageStage.version}`",
}

for old, new in replacements.items():
    if old not in text:
        raise SystemExit(f"WG-6B browser fix marker not found: {old}")
    text = text.replace(old, new)

path.write_text(text)
