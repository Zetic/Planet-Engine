from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"expected text not found in {path}: {old[:120]!r}")
    text = text.replace(old, new, 1)
    p.write_text(text)


# WG-5 behavior and explicit diagnostic status.
replace_once(
    "rust/interlink-worldgen/src/climate.rs",
    'pub const CLIMATE_STAGE_VERSION: u32 = 7;',
    'pub const CLIMATE_STAGE_VERSION: u32 = 8;',
)
replace_once(
    "rust/interlink-worldgen/src/climate.rs",
    '    pub spinup_years: u8,\n    pub mean_temperature_k: f64,',
    '    pub spinup_years: u8,\n    pub spinup_converged: bool,\n    pub convergence_temperature_rms_k: f64,\n    pub mean_temperature_k: f64,',
)
replace_once(
    "rust/interlink-worldgen/src/climate.rs",
    '''    if final_temperature_rms_change > parameters.convergence_temperature_rms_k {\n        return Err(WorldgenError::InvalidClimate(\n            "WG-5 climate did not converge within the configured spin-up bound",\n        ));\n    }\n''',
    '''    // Reaching the deterministic spin-up bound without satisfying the configured\n    // temperature RMS target is a quality diagnostic, not an invalid physical state.\n    // Preserve the final bounded year and let downstream stages continue; hard failures\n    // remain reserved for invalid/non-finite state and conservation violations.\n    let spinup_converged =\n        final_temperature_rms_change <= parameters.convergence_temperature_rms_k;\n''',
)
replace_once(
    "rust/interlink-worldgen/src/climate.rs",
    '        spinup_years,\n        mean_temperature_k: mean_temperature,',
    '        spinup_years,\n        spinup_converged,\n        convergence_temperature_rms_k: parameters.convergence_temperature_rms_k,\n        mean_temperature_k: mean_temperature,',
)

# The explicit regression forces a one-year / impossible-tolerance case and now requires
# a finite returned state instead of an error.
replace_once(
    "rust/interlink-worldgen/tests/climate_ensemble.rs",
    '''#[test]\nfn core_rejects_unconverged_climate_instead_of_returning_a_state() {\n    let planet = PlanetPhysicalParameters::earthlike_reference();\n    let (topology, terrain) = generated_surface("wg5-nonconverged", planet);\n    let mut request = ClimateRequest::new("wg5-nonconverged");\n    request.parameters.minimum_spinup_years = 1;\n    request.parameters.maximum_spinup_years = 1;\n    request.parameters.convergence_temperature_rms_k = 1.0e-12;\n    let error = generate_coupled_climate(&topology, &terrain, planet, &request).unwrap_err();\n    assert!(error.to_string().contains("did not converge"));\n}\n''',
    '''#[test]\nfn core_returns_bounded_state_when_spinup_does_not_converge() {\n    let planet = PlanetPhysicalParameters::earthlike_reference();\n    let (topology, terrain) = generated_surface("wg5-nonconverged", planet);\n    let mut request = ClimateRequest::new("wg5-nonconverged");\n    request.parameters.minimum_spinup_years = 1;\n    request.parameters.maximum_spinup_years = 1;\n    request.parameters.convergence_temperature_rms_k = 1.0e-12;\n    let climate = generate_coupled_climate(&topology, &terrain, planet, &request).unwrap();\n    assert_eq!(climate.metrics.spinup_years, 1);\n    assert!(!climate.metrics.spinup_converged);\n    assert_eq!(\n        climate.metrics.convergence_temperature_rms_k,\n        request.parameters.convergence_temperature_rms_k\n    );\n    assert!(\n        climate.metrics.final_temperature_rms_change_k\n            > climate.metrics.convergence_temperature_rms_k\n    );\n    assert!(climate\n        .temperature_mean_k\n        .iter()\n        .all(|value| value.is_finite()));\n    assert!(climate\n        .annual_precipitation_mm\n        .iter()\n        .all(|value| value.is_finite() && *value >= 0.0));\n}\n''',
)
replace_once(
    "rust/interlink-worldgen/tests/hydroclimate_closure.rs",
    '    assert_eq!(CLIMATE_STAGE_VERSION, 7);',
    '    assert_eq!(CLIMATE_STAGE_VERSION, 8);',
)

# Native CLI follows the same nonfatal policy while retaining a visible warning.
replace_once(
    "rust/interlink-worldgen-cli/src/main.rs",
    '''    if metrics.final_temperature_rms_change_k\n        > climate_request.parameters.convergence_temperature_rms_k\n    {\n        return Err(format!(\n            "WG-5 climate did not converge: final RMS change {:.6} K exceeds target {:.6} K after {} model years",\n            metrics.final_temperature_rms_change_k,\n            climate_request.parameters.convergence_temperature_rms_k,\n            metrics.spinup_years\n        ));\n    }\n''',
    '''    if !metrics.spinup_converged {\n        eprintln!(\n            "warning: WG-5 reached the {}-year spin-up bound with final RMS change {:.6} K above target {:.6} K; accepting the bounded final climate state",\n            metrics.spinup_years,\n            metrics.final_temperature_rms_change_k,\n            metrics.convergence_temperature_rms_k,\n        );\n    }\n''',
)

# Expose the advisory status through WASM and the cumulative browser packet.
replace_once(
    "rust/interlink-worldgen-wasm/src/climate_bridge.rs",
    '''    pub fn spinup_years(&self) -> u8 {\n        self.climate.metrics.spinup_years\n    }\n    pub fn mean_temperature_k(&self) -> f64 {''',
    '''    pub fn spinup_years(&self) -> u8 {\n        self.climate.metrics.spinup_years\n    }\n    pub fn spinup_converged(&self) -> bool {\n        self.climate.metrics.spinup_converged\n    }\n    pub fn convergence_temperature_rms_k(&self) -> f64 {\n        self.climate.metrics.convergence_temperature_rms_k\n    }\n    pub fn mean_temperature_k(&self) -> f64 {''',
)
replace_once(
    "rust/interlink-worldgen-wasm/src/lib.rs",
    'pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 19;',
    'pub const WORLDGEN_WASM_PROTOCOL_VERSION: u32 = 20;',
)
replace_once(
    "src/worldgen/protocol.ts",
    'export const WORLDGEN_PROTOCOL_VERSION = 19;',
    'export const WORLDGEN_PROTOCOL_VERSION = 20;',
)
replace_once(
    "src/worldgen/protocol.ts",
    '  spinupYears: number;\n  meanTemperatureK: number;',
    '  spinupYears: number;\n  spinupConverged: boolean;\n  convergenceTemperatureRmsK: number;\n  meanTemperatureK: number;',
)
replace_once(
    "src/worldgen/worldgenWorker.ts",
    'global_solver_level(): number; global_solver_sample_count(): number; spinup_years(): number; mean_temperature_k(): number;',
    'global_solver_level(): number; global_solver_sample_count(): number; spinup_years(): number; spinup_converged(): boolean; convergence_temperature_rms_k(): number; mean_temperature_k(): number;',
)
replace_once(
    "src/worldgen/worldgenWorker.ts",
    'globalSolverSampleCount: output.global_solver_sample_count(), spinupYears: output.spinup_years(),',
    'globalSolverSampleCount: output.global_solver_sample_count(), spinupYears: output.spinup_years(), spinupConverged: output.spinup_converged(), convergenceTemperatureRmsK: output.convergence_temperature_rms_k(),',
)

# Make fallback status visible instead of silently pretending the diagnostic target was met.
replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    "  metric(metrics, 'Spin-up', `${result.metrics.spinupYears} model years · ΔT ${result.metrics.finalTemperatureRmsChangeK.toFixed(3)} K RMS`);",
    "  metric(metrics, 'Spin-up', `${result.metrics.spinupYears} model years · ΔT ${result.metrics.finalTemperatureRmsChangeK.toFixed(3)} K RMS · ${result.metrics.spinupConverged ? 'converged' : `bounded fallback (target ≤ ${result.metrics.convergenceTemperatureRmsK.toFixed(3)} K)`}`);",
)
replace_once(
    "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts",
    "  generationStep.textContent = `${result.metrics.spinupYears} climate spin-up years`;",
    "  generationStep.textContent = `${result.metrics.spinupYears} climate spin-up years${result.metrics.spinupConverged ? '' : ' · bounded fallback accepted'}`;",
)

# Preserve the status in calibration exports too.
replace_once(
    "src/worldgen/calibrationPacket.ts",
    '      final_temperature_rms_change_k: result.metrics.finalTemperatureRmsChangeK,',
    '      final_temperature_rms_change_k: result.metrics.finalTemperatureRmsChangeK,\n      spinup_converged: result.metrics.spinupConverged,\n      convergence_temperature_rms_k: result.metrics.convergenceTemperatureRmsK,',
)

# Browser regression proves the new status is carried end-to-end and the core no longer
# contains the old fatal nonconvergence branch.
wg5_test = Path("tests/wg5Climate.test.ts")
text = wg5_test.read_text()
text = text.replace('assert.equal(WORLDGEN_PROTOCOL_VERSION, 19);', 'assert.equal(WORLDGEN_PROTOCOL_VERSION, 20);', 1)
text = text.replace('    protocolVersion: 19,', '    protocolVersion: 20,', 1)
anchor = "test('cumulative WG-5 Lab exposes climate diagnostics and stored seasonal reconstruction', () => {"
insert = '''test('WG-5 spin-up max bound is advisory and transported to the browser', () => {\n  const climate = fs.readFileSync('rust/interlink-worldgen/src/climate.rs', 'utf8');\n  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');\n  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');\n  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');\n  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');\n  assert.doesNotMatch(climate, /WG-5 climate did not converge within the configured spin-up bound/);\n  assert.match(climate, /spinup_converged/);\n  assert.match(bridge, /spinup_converged/);\n  assert.match(protocol, /spinupConverged/);\n  assert.match(protocol, /convergenceTemperatureRmsK/);\n  assert.match(worker, /spinup_converged\(\)/);\n  assert.match(worker, /convergence_temperature_rms_k\(\)/);\n  assert.match(lab, /bounded fallback/);\n});\n\n'''
if anchor not in text:
    raise SystemExit('WG-5 browser test insertion anchor not found')
text = text.replace(anchor, insert + anchor, 1)
wg5_test.write_text(text)

# Protocol v20 is an additive cumulative metric shape change. Update tests that explicitly
# pin the current protocol constant; stage-count literals and historical test names are untouched.
for p in Path("tests").glob("*.test.ts"):
    text = p.read_text()
    text = text.replace('assert.equal(WORLDGEN_PROTOCOL_VERSION, 19);', 'assert.equal(WORLDGEN_PROTOCOL_VERSION, 20);')
    text = text.replace('const PROTOCOL = 19;', 'const PROTOCOL = 20;')
    text = text.replace('protocolVersion: 19,', 'protocolVersion: 20,')
    p.write_text(text)

# Documentation: distinguish quality convergence from correctness/conservation gates.
replace_once(
    "docs/worldgen-rewrite/WG5_CLIMATE.md",
    'The public WG-5 generator now rejects a climate state if the configured annual temperature convergence tolerance is not reached or if the final atmospheric moisture budget exceeds the conservation tolerance. Native CLI, Rust callers, WASM, and browser generation therefore share the same acceptance contract.',
    'WG-5 temperature spin-up convergence is a quality diagnostic rather than a validity gate. The solver still stops early when the configured annual RMS target is reached, but if the deterministic maximum spin-up bound is exhausted it now returns the final bounded climate state with `spinup_converged = false` and records both the final RMS change and target tolerance. Native CLI, Rust callers, WASM, and browser generation therefore continue into downstream hydrology instead of rejecting an otherwise finite state. Hard failures remain reserved for invalid/non-finite model state and conservation failures such as atmospheric moisture-budget closure.',
)

print('WG-5 nonfatal spin-up rewrite applied')
