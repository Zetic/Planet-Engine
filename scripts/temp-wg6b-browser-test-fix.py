from pathlib import Path


def replace_required(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"marker not found in {path}: {old[:180]!r}")
    target.write_text(text.replace(old, new, 1))


def replace_if_present(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old in text:
        target.write_text(text.replace(old, new))


# All command fixtures follow protocol v12. Different historical tests encode
# that expectation with either object literals or assert.equal arguments.
for path in [
    "tests/wg4Topography.test.ts",
    "tests/wg5Climate.test.ts",
    "tests/wg6Drainage.test.ts",
]:
    replace_if_present(path, "protocolVersion: 11", "protocolVersion: 12")
    replace_if_present(path, "protocolVersion, 11", "protocolVersion, 12")

# WG-4 remains cumulatively inspectable, but the public frontier is WG-6B.
replace_required(
    "tests/wg4Topography.test.ts",
    "Planet Engine Lab keeps every WG-3.75 and WG-4 view cumulatively through WG-6A",
    "Planet Engine Lab keeps every WG-3.75 and WG-4 view cumulatively through WG-6B",
)
replace_required(
    "tests/wg4Topography.test.ts",
    "assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, and WG-6A drainage topology/i);",
    r"assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, and WG-6B annual runoff\/discharge/i);",
)

# The primary Lab consumes WG-6A and WG-6B from the same cumulative climate
# transport. Keep the dedicated drainage command available for non-primary
# clients, but reject any second drainage generation from the main Lab.
replace_required(
    "tests/wg6Drainage.test.ts",
    r"  assert.match(source, /client\.generateDrainage\(request\)/);",
    r"  assert.doesNotMatch(source, /client\.generateDrainage\(/);\n  assert.match(source, /client\.generateClimate\(request, handleGenerationProgress\)/);\n  assert.match(source, /drainageMetrics/);",
)
replace_required(
    "tests/wg6Drainage.test.ts",
    r"  assert.match(source, /drainage\.topographyHash !== loaded\.metrics\.topographyHash/);",
    r"  assert.match(source, /loaded\.runoffMetrics\.climateHash !== loaded\.metrics\.climateHash/);\n  assert.match(source, /loaded\.runoffMetrics\.drainageHash !== loaded\.drainageMetrics\.drainageHash/);",
)

# The standalone drainage page/controller is intentionally gone. Keep the
# engine-independence assertion focused on the canonical cumulative Lab and
# include both permanent hydrology core modules in the existence contract.
replace_required(
    "tests/worldgenRewrite.test.ts",
    "Planet Engine source stays independent from legacy gameplay world objects through WG-6A",
    "Planet Engine source stays independent from legacy gameplay world objects through WG-6B",
)
replace_required(
    "tests/worldgenRewrite.test.ts",
    "    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',\n    'src/worldgen/diagnostics/worldgenDrainageLabStandalone.ts',",
    "    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',",
)
replace_required(
    "tests/worldgenRewrite.test.ts",
    "    'rust/interlink-worldgen/src/climate.rs',\n    'rust/interlink-worldgen/tests/climate_ensemble.rs',",
    "    'rust/interlink-worldgen/src/climate.rs',\n    'rust/interlink-worldgen/src/drainage.rs',\n    'rust/interlink-worldgen/src/runoff.rs',\n    'rust/interlink-worldgen/tests/climate_ensemble.rs',",
)
