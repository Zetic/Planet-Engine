from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected exactly one match in {path}, found {count}: {old!r}")
    file.write_text(text.replace(old, new, 1))


# WG-4 @4: keep the calibrated 1200 m / 450 km kernel, but stop applying
# nearly full direct uplift to every mature oceanic spreading boundary. Young
# oceanic-crust thermal relief remains independent and continues to express ridges.
replace_once(
    "rust/interlink-worldgen/src/topography.rs",
    'pub const TOPOGRAPHY_STAGE_VERSION: u32 = 3;',
    'pub const TOPOGRAPHY_STAGE_VERSION: u32 = 4;',
)
replace_once(
    "rust/interlink-worldgen/src/topography.rs",
    'const STRUCTURE_RIFT: u8 = 2;\n',
    'const STRUCTURE_RIFT: u8 = 2;\nconst OCEANIC_RIDGE_DIRECT_RESPONSE_SCALE: f64 = 0.50;\nconst OCEANIC_RIDGE_BASE_RESPONSE: f64 = 0.10;\n',
)
replace_once(
    "rust/interlink-worldgen/src/topography.rs",
    '''            GeologicalBoundaryRegime::OceanicRidge
            | GeologicalBoundaryRegime::TransitionalDivergence => {
                ridge[a] = ridge[a].max(0.35 + 0.65 * divergence);
                ridge[b] = ridge[b].max(0.35 + 0.65 * divergence);
            }
''',
    '''            GeologicalBoundaryRegime::OceanicRidge => {
                let strength = OCEANIC_RIDGE_DIRECT_RESPONSE_SCALE
                    * (OCEANIC_RIDGE_BASE_RESPONSE
                        + (1.0 - OCEANIC_RIDGE_BASE_RESPONSE) * divergence);
                ridge[a] = ridge[a].max(strength);
                ridge[b] = ridge[b].max(strength);
            }
            GeologicalBoundaryRegime::TransitionalDivergence => {
                ridge[a] = ridge[a].max(0.35 + 0.65 * divergence);
                ridge[b] = ridge[b].max(0.35 + 0.65 * divergence);
            }
''',
)

replace_once(
    "rust/interlink-worldgen-wasm/tests/topography_bridge.rs",
    '    assert_eq!(output.stage_version(), 3);',
    '    assert_eq!(output.stage_version(), 4);',
)

replace_once(
    "docs/worldgen-rewrite/TOPOGRAPHY.md",
    'The v2 terrain state keeps forcing components separately inspectable:',
    'The terrain state keeps forcing components separately inspectable:',
)
replace_once(
    "docs/worldgen-rewrite/TOPOGRAPHY.md",
    'The calibrated defaults reduce broad crustal/isostatic support and old/broad uplift while preserving signed tectonic morphology: `isostatic_scale = 0.55`, inherited orogeny `1200 m`, collision uplift `2400 m` over a `600 km` kernel, and mantle-dynamic relief `650 m`. WG-4 `@3` additionally narrows and lowers oceanic spreading-ridge relief from `2000 m / 600 km` to `1200 m / 450 km`. The L5→L7 five-seed calibration reduced aggregate oceanic-ridge endpoint emergence from about `40%` to `19%` and continuous both-land ridge edges from about `31%` to `10%`, while preserving a roughly `26%` mean land fraction, ~`1.4 km` mean land elevation, ~`3.6 km` mean ocean depth, and >`90%` continental-collision endpoint emergence. Rift, trench, arc, thermal-subsidence, water-inventory and mechanical-filter parameters remain unchanged.',
    'The calibrated defaults reduce broad crustal/isostatic support and old/broad uplift while preserving signed tectonic morphology: `isostatic_scale = 0.55`, inherited orogeny `1200 m`, collision uplift `2400 m` over a `600 km` kernel, and mantle-dynamic relief `650 m`. WG-4 `@3` narrowed and lowered the shared ridge kernel from `2000 m / 600 km` to `1200 m / 450 km`. WG-4 `@4` separates mature `OceanicRidge` source response from `TransitionalDivergence`: pure oceanic spreading now uses `0.50 × (0.10 + 0.90 × normalized_divergence)` before the existing inherited ridge-history factor, while transitional divergence keeps the prior `0.35 + 0.65 × normalized_divergence` response. This avoids double-counting young-ocean thermal relief as a second near-full direct uplift without flattening rifted/transitional margins. Across a six-seed L5→L7 / 16-plate ensemble, aggregate oceanic-ridge endpoint emergence falls from about `20%` to `12%`, both-land ridge edges from about `11%` to `5%`, and submerged ridge endpoints shallower than `500 m` from about `30%` to `10%`; median submerged ridge depth rises from ~`0.86 km` to ~`1.23 km`. Mean land fraction remains ~`27%`, mean land elevation ~`1.41 km`, mean ocean depth ~`3.68 km`, and continental-collision endpoint emergence remains >`92%`. Rift, trench, arc, thermal-subsidence, water-inventory and mechanical-filter parameters remain unchanged.',
)
replace_once(
    "docs/worldgen-rewrite/TOPOGRAPHY.md",
    'Stage identity is `terrain:initial-topography@3` with namespace `terrain:structure:v1`.',
    'Stage identity is `terrain:initial-topography@4` with namespace `terrain:structure:v1`.',
)

# The Lab currently exposes canonical WG-7D hydrology through the top-level
# drainage/runoff/lake/seasonal getters. Label it as final state rather than
# implying these values are the historical pre-erosion WG-6 ancestry.
lab = "src/worldgen/diagnostics/worldgenClimateLabStandalone.ts"
replace_once(lab, "metric(metrics, 'Hydrology / stage',", "metric(metrics, 'Final hydrology / drainage stage',")
replace_once(lab, "metric(metrics, 'Drainage topology',", "metric(metrics, 'Final drainage topology',")
replace_once(lab, "metric(metrics, 'Largest contributing area',", "metric(metrics, 'Final largest contributing area',")
replace_once(lab, "metric(metrics, 'Deepest depression',", "metric(metrics, 'Final deepest depression',")
replace_once(lab, "metric(metrics, 'Drainage area closure',", "metric(metrics, 'Final drainage area closure',")
replace_once(
    lab,
    "  metric(metrics, 'Drainage hash', result.drainageMetrics.drainageHash);\n  metric(metrics, 'Hydrology identity', result.runoffMetrics.climateHash === result.metrics.climateHash && result.runoffMetrics.drainageHash === result.drainageMetrics.drainageHash ? 'WG-5 / WG-6A / WG-6B match' : 'MISMATCH');",
    "  metric(metrics, 'Final drainage hash', result.drainageMetrics.drainageHash);\n  metric(metrics, 'Pre-erosion WG-6A drainage hash', result.reconciliationMetrics.preErosionDrainageHash);\n  metric(metrics, 'Final hydrology identity', result.runoffMetrics.climateHash === result.metrics.climateHash && result.runoffMetrics.drainageHash === result.drainageMetrics.drainageHash ? 'WG-7D canonical drainage / runoff match' : 'MISMATCH');",
)
replace_once(
    lab,
    "throw new Error('WG-6B drainage identity does not match accepted WG-6A topology.');",
    "throw new Error('Final runoff drainage identity does not match canonical WG-7D drainage.');",
)
replace_once(
    lab,
    "throw new Error('WG-6C drainage identity does not match accepted WG-6A topology.');",
    "throw new Error('Final lake drainage identity does not match canonical WG-7D drainage.');",
)
replace_once(
    lab,
    "throw new Error('WG-6C runoff identity does not match accepted WG-6B runoff.');",
    "throw new Error('Final lake runoff identity does not match canonical WG-7D runoff.');",
)
replace_once(
    lab,
    "throw new Error('WG-6D drainage identity does not match accepted WG-6A topology.');",
    "throw new Error('Final seasonal drainage identity does not match canonical WG-7D drainage.');",
)
replace_once(
    lab,
    "throw new Error('WG-6D runoff identity does not match accepted WG-6B runoff.');",
    "throw new Error('Final seasonal runoff identity does not match canonical WG-7D runoff.');",
)
replace_once(
    lab,
    "throw new Error('WG-6D lake identity does not match accepted WG-6C state.');",
    "throw new Error('Final seasonal lake identity does not match canonical WG-7D lake state.');",
)

replace_once("index.html", '<title>Planet Engine · Through WG-7C</title>', '<title>Planet Engine · Through WG-7D</title>')
replace_once("index.html", 'PLANET ENGINE · THROUGH WG-7C', 'PLANET ENGINE · THROUGH WG-7D')
replace_once(
    "index.html",
    'Generate one deterministic physical planet through WG-7C, then inspect topology, tectonics, geology, lithosphere, initial topography, climate, drainage, lakes, seasonal realized flow, erosive forcing, sediment routing, the evolved terrain surface, and rebuilt post-erosion drainage from one matched physical state.',
    'Generate one deterministic physical planet through WG-7D, then inspect topology, tectonics, geology, lithosphere, initial topography, climate, canonical final drainage, lakes, seasonal realized flow, erosive forcing, sediment routing, terrain evolution, post-erosion reconciliation, and lake-sediment infill from one matched physical state.',
)
