#!/usr/bin/env python3
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one guarded match, found {count}: {old[:100]!r}")
    p.write_text(text.replace(old, new, 1))


topography = 'rust/interlink-worldgen/src/topography.rs'
replace_once(topography, 'pub const TOPOGRAPHY_STAGE_VERSION: u32 = 4;', 'pub const TOPOGRAPHY_STAGE_VERSION: u32 = 5;')
replace_once(
    topography,
    'const OCEANIC_RIDGE_BASE_RESPONSE: f64 = 0.10;\n',
    '''const OCEANIC_RIDGE_BASE_RESPONSE: f64 = 0.10;\nconst CONTINENTAL_RIFT_DIRECT_RESPONSE_SCALE: f64 = 0.10;\nconst CONTINENTAL_RIFT_BASE_RESPONSE: f64 = 0.05;\nconst CONTINENTAL_RIFT_SOURCE_HISTORY_FLOOR: f64 = 0.05;\nconst CONTINENTAL_RIFT_CONTINENTAL_HISTORY_RELIEF_WEIGHT: f64 = 0.05;\nconst TRANSITIONAL_RIFT_HISTORY_RELIEF_WEIGHT: f64 = 0.55;\n''',
)
replace_once(
    topography,
    '''            GeologicalBoundaryRegime::ContinentalRift => {\n                rift[a] = rift[a].max(0.35 + 0.65 * divergence);\n                rift[b] = rift[b].max(0.35 + 0.65 * divergence);\n            }''',
    '''            GeologicalBoundaryRegime::ContinentalRift => {\n                let strength = CONTINENTAL_RIFT_DIRECT_RESPONSE_SCALE\n                    * (CONTINENTAL_RIFT_BASE_RESPONSE\n                        + (1.0 - CONTINENTAL_RIFT_BASE_RESPONSE) * divergence);\n                rift[a] = rift[a].max(strength);\n                rift[b] = rift[b].max(strength);\n            }''',
)
replace_once(
    topography,
    '        rift[i] *= 0.65 + 0.35 * f64::from(inherited.rift_history[i]);',
    '''        rift[i] *= CONTINENTAL_RIFT_SOURCE_HISTORY_FLOOR\n            + (1.0 - CONTINENTAL_RIFT_SOURCE_HISTORY_FLOOR)\n                * f64::from(inherited.rift_history[i]);''',
)
replace_once(
    topography,
    '''        rift_basin[i] = -(p.rift_subsidence_scale_m\n            * (rift_kernel * rift_focus + 0.55 * f64::from(inherited.rift_history[i]))\n            + p.basin_subsidence_scale_m''',
    '''        let inherited_rift_relief_weight = match inherited.crust_kind[i] {\n            CRUST_TRANSITIONAL => TRANSITIONAL_RIFT_HISTORY_RELIEF_WEIGHT,\n            CRUST_OCEANIC => 0.0,\n            _ => CONTINENTAL_RIFT_CONTINENTAL_HISTORY_RELIEF_WEIGHT,\n        };\n        rift_basin[i] = -(p.rift_subsidence_scale_m\n            * (rift_kernel * rift_focus\n                + inherited_rift_relief_weight * f64::from(inherited.rift_history[i]))\n            + p.basin_subsidence_scale_m''',
)

bridge = 'rust/interlink-worldgen-wasm/tests/topography_bridge.rs'
replace_once(bridge, '    assert_eq!(output.stage_version(), 4);', '    assert_eq!(output.stage_version(), 5);')

docs = 'docs/worldgen-rewrite/TOPOGRAPHY.md'
replace_once(
    docs,
    'Rift, trench, arc, thermal-subsidence, water-inventory and mechanical-filter parameters remain unchanged.',
    'Trench, arc, thermal-subsidence, water-inventory and mechanical-filter parameters remain unchanged in `@4`.',
)
marker = 'A five-seed L4 Earth-like ensemble now occupies a deliberately broad **pre-erosion** envelope:'
insert = '''WG-4 `@5` separates an active **continental rift valley** from a **transitional breakup seaway**. WG-3 already expresses extension through continental crust thinning, rift-derived subsidence, and basin potential; the older WG-4 response then stacked a strong guaranteed `ContinentalRift` trough plus another full inherited-rift depression on top of those upstream signals. At an active rift source, `rift_history` is intentionally near one, so that combination made mature-looking marine corridors nearly automatic. `@5` reduces only the extra direct continental-rift response to `0.10 × (0.05 + 0.95 × normalized_divergence)` and limits duplicated inherited-rift relief on continental crust to `0.05`. Transitional crust retains the prior `0.55` inherited-rift relief weight so advanced breakup can still form Red-Sea-style seaways, while basin/subsidence history and crustal-isostatic thinning remain authoritative. In the six-seed L5→L7 / 16-plate acceptance ensemble, aggregate `ContinentalRift` endpoint flooding falls from about `65%` to `43%`, endpoints deeper than `500 m` from about `61%` to `28%`, and endpoints deeper than `1 km` from about `55%` to `17%`. For the visible `interlink-wg7c` calibration seed, continental-rift flooding falls from about `70%` to `32%` and no continental-rift endpoint remains deeper than `500 m`. `TransitionalDivergence` remains strongly marine at about `82%` flooded, preserving genuine breakup/young-sea morphology.\n\n'''
p = Path(docs)
text = p.read_text()
if text.count(marker) != 1:
    raise SystemExit(f'{docs}: acceptance paragraph marker drifted')
p.write_text(text.replace(marker, insert + marker, 1).replace('terrain:initial-topography@4', 'terrain:initial-topography@5', 1))
