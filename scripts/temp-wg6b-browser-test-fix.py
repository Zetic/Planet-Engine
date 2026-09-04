from pathlib import Path


def replace_all(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"marker not found in {path}: {old!r}")
    target.write_text(text.replace(old, new))


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"marker not found in {path}: {old[:180]!r}")
    target.write_text(text.replace(old, new, 1))


# All browser command fixtures follow the protocol-v12 boundary after the
# cumulative WG-6B result expands the WASM/browser contract.
for path in [
    "tests/wg4Topography.test.ts",
    "tests/wg5Climate.test.ts",
    "tests/wg6Drainage.test.ts",
]:
    replace_all(path, "protocolVersion: 11", "protocolVersion: 12")

# WG-4 remains cumulatively inspectable, but the public frontier is now WG-6B.
replace_once(
    "tests/wg4Topography.test.ts",
    "Planet Engine Lab keeps every WG-3.75 and WG-4 view cumulatively through WG-6A",
    "Planet Engine Lab keeps every WG-3.75 and WG-4 view cumulatively through WG-6B",
)
replace_once(
    "tests/wg4Topography.test.ts",
    "assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, and WG-6A drainage topology/i);",
    "assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, and WG-6B annual runoff\/discharge/i);",
)

# The primary Lab must no longer issue a separate drainage request. WG-6A is
# carried by the same cumulative result that also exposes WG-6B.
replace_once(
    "tests/wg6Drainage.test.ts",
    "  assert.match(source, /client\\.generateDrainage\\(request\\)/);",
    "  assert.doesNotMatch(source, /client\\.generateDrainage\\(/);\n  assert.match(source, /client\\.generateClimate\\(request, handleGenerationProgress\\)/);\n  assert.match(source, /drainageMetrics/);",
)

# The standalone drainage page/controller is intentionally gone. Keep the
# engine-independence assertion focused on the canonical cumulative Lab.
replace_once(
    "tests/worldgenRewrite.test.ts",
    "Planet Engine source stays independent from legacy gameplay world objects through WG-6A",
    "Planet Engine source stays independent from legacy gameplay world objects through WG-6B",
)
replace_once(
    "tests/worldgenRewrite.test.ts",
    "    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',\n    'src/worldgen/diagnostics/worldgenDrainageLabStandalone.ts',",
    "    'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts',",
)
replace_once(
    "tests/worldgenRewrite.test.ts",
    "    'rust/interlink-worldgen/src/climate.rs',\n    'rust/interlink-worldgen/tests/climate_ensemble.rs',",
    "    'rust/interlink-worldgen/src/climate.rs',\n    'rust/interlink-worldgen/src/drainage.rs',\n    'rust/interlink-worldgen/src/runoff.rs',\n    'rust/interlink-worldgen/tests/climate_ensemble.rs',",
)
