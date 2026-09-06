from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected exactly one match in {path}, found {count}: {old!r}")
    file.write_text(text.replace(old, new, 1))


replace_once(
    "rust/interlink-worldgen/src/topography.rs",
    'pub const TOPOGRAPHY_STAGE_VERSION: u32 = 2;',
    'pub const TOPOGRAPHY_STAGE_VERSION: u32 = 3;',
)
replace_once(
    "rust/interlink-worldgen/src/topography.rs",
    '            ridge_uplift_scale_m: 2_000.0,\n            ridge_width_m: 600_000.0,',
    '            ridge_uplift_scale_m: 1_200.0,\n            ridge_width_m: 450_000.0,',
)
replace_once(
    "rust/interlink-worldgen-wasm/tests/topography_bridge.rs",
    '    assert_eq!(output.stage_version(), 2);',
    '    assert_eq!(output.stage_version(), 3);',
)
replace_once(
    "docs/worldgen-rewrite/TOPOGRAPHY.md",
    'The calibrated defaults reduce broad crustal/isostatic support and old/broad uplift while preserving signed tectonic morphology: `isostatic_scale = 0.55`, inherited orogeny `1200 m`, collision uplift `2400 m` over a `600 km` kernel, and mantle-dynamic relief `650 m`. Ridge, rift, trench, arc, thermal-subsidence, water-inventory and mechanical-filter parameters are unchanged.',
    'The calibrated defaults reduce broad crustal/isostatic support and old/broad uplift while preserving signed tectonic morphology: `isostatic_scale = 0.55`, inherited orogeny `1200 m`, collision uplift `2400 m` over a `600 km` kernel, and mantle-dynamic relief `650 m`. WG-4 `@3` additionally narrows and lowers oceanic spreading-ridge relief from `2000 m / 600 km` to `1200 m / 450 km`. The L5→L7 five-seed calibration reduced aggregate oceanic-ridge endpoint emergence from about `40%` to `19%` and continuous both-land ridge edges from about `31%` to `10%`, while preserving a roughly `26%` mean land fraction, ~`1.4 km` mean land elevation, ~`3.6 km` mean ocean depth, and >`90%` continental-collision endpoint emergence. Rift, trench, arc, thermal-subsidence, water-inventory and mechanical-filter parameters remain unchanged.',
)
replace_once(
    "docs/worldgen-rewrite/TOPOGRAPHY.md",
    'Stage identity is `terrain:initial-topography@2` with namespace `terrain:structure:v1`.',
    'Stage identity is `terrain:initial-topography@3` with namespace `terrain:structure:v1`.',
)
