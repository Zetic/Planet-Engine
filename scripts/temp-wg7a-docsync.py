from pathlib import Path


def once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)


path = Path("docs/worldgen-rewrite/WG6_HYDROLOGY.md")
text = path.read_text()
text = once(
    text,
    "WG-6B originally entered the cumulative browser contract at protocol version `12`; the current cumulative Lab contract is protocol version `14` through WG-6D.",
    "WG-6B originally entered the cumulative browser contract at protocol version `12`; the current cumulative Lab contract is protocol version `15` through WG-7A.",
    "WG-6 protocol note",
)
old_browser = """`index.html` is the single public Planet Engine Lab entrypoint through WG-6D. The former `drainage.html` and `worldgen-lab.html` entrypoints and the standalone drainage-page controller remain removed. Browser/WASM protocol version `14` carries WG-6D in the same cumulative result as WG-5, WG-6A, WG-6B, and WG-6C, so the primary Lab still performs one matched physical generation rather than issuing a redundant hydrology request.

The cumulative Lab retains all prior WG-6 diagnostics and adds:
"""
new_browser = """`index.html` is the single public Planet Engine Lab entrypoint through WG-7A. The former `drainage.html` and `worldgen-lab.html` entrypoints and the standalone drainage-page controller remain removed. Browser/WASM protocol version `15` carries WG-7A after WG-6D in the same cumulative result as WG-5, WG-6A, WG-6B, and WG-6C, so the primary Lab still performs one matched physical generation rather than issuing a redundant hydrology or erosion request.

The cumulative Lab retains all prior WG-6 diagnostics and exposes WG-7A erosion/sediment diagnostics beside them. WG-6D adds:
"""
text = once(text, old_browser, new_browser, "WG-6 browser frontier")
old_deferred = """WG-6D completes the planned WG-6 generation-time surface-hydrology stack through phase runoff/discharge timing, snow accumulation/melt timing, dynamic seasonal lake storage, and dry/intermittent/perennial realized-flow classification. It does not model individual storms or flood peaks, groundwater, permanent snow/glacier mass balance, river width/depth or hydraulic geometry, floodplains/wetlands, channel migration, erosion, sediment transport, deltas, biomes, resources, or gameplay geography. Terrain response, incision, and sediment transport remain WG-7 or later stages."""
new_deferred = """WG-6D completes the planned WG-6 generation-time surface-hydrology stack through phase runoff/discharge timing, snow accumulation/melt timing, dynamic seasonal lake storage, and dry/intermittent/perennial realized-flow classification. It does not model individual storms or flood peaks, groundwater, permanent snow/glacier mass balance, floodplains/wetlands, channel migration, deltas, biomes, resources, or gameplay geography. WG-7A now consumes this accepted hydrology to derive diagnostic hydraulic channel width, erosive forcing, incision potential, and conservative sediment transport/deposition. Applied terrain response, valley development, sedimentary fill, and drainage recalculation remain WG-7B or later stages."""
text = once(text, old_deferred, new_deferred, "WG-6 deferred frontier")
path.write_text(text)

path = Path("docs/worldgen-rewrite/VALIDATION.md")
text = path.read_text()
wg7 = """

## WG-7A fluvial-erosion and sediment gates

WG-7A is accepted as a deterministic, conservative forcing/routing stage before terrain mutation:

- all WG-7A fields align on the canonical fine topology;
- the supplied WG-6D state retains the exact accepted WG-6A drainage and WG-6C lake ancestry;
- the complete phase-major WG-6D realized-discharge field is present and finite;
- effective erosive discharge is peak-sensitive and deterministic across the retained seasonal hydrograph;
- receiver slope and channel length come only from the accepted WG-6A receiver edge and immutable WG-4 solid terrain;
- hydraulic channel width remains finite, positive for positive effective discharge, and inside the parameterized bounds;
- inherited erodibility remains in `[0.05, 1]` and responds monotonically to the documented strength/weakness/fragmentation/fabric terms;
- diagnostic incision is finite, nonnegative, bounded by the parameterized ceiling, and does not mutate WG-4 terrain;
- sediment production uses receiver-segment length and discharge-derived channel width rather than dual-cell area;
- sediment transport follows the accepted WG-6A upstream-to-downstream order;
- every member of an active WG-6C depression acts as a complete first-pass sediment trap, preventing dry depression members from bypassing the lake control volume;
- generated sediment closes into land, lake, and terminal/ocean deposition within `1e-10` in permanent CI;
- the fixed-ancestry L6/L7 diagnostic is interpreted as a resolution-sensitivity check rather than per-cell equality, with accepted generated-sediment drift around 11.9% for seed `ci-wg5-l7`;
- every public WG-7A vector and upstream ancestry identity participates in deterministic hashing;
- browser/WASM protocol v15 carries WG-7A in the same cumulative planet result, and the primary Lab exposes the accepted WG-7A diagnostic fields without a second generation request;
- WG-4 terrain, WG-6A receiver/depression topology, and all accepted WG-6 hydrology remain unchanged by WG-7A.

`bash scripts/check-wg7a-erosion.sh` is the permanent fixed L4 acceptance path. It checks canonical dimensions, nonempty erosive behavior, finite positive forcing, deterministic hash formatting, and global sediment closure. Native unit regressions additionally cover peak-sensitive effective discharge, inherited erodibility, bounded incision, hydraulic-width scaling, complete lake-depression trapping, ocean export, and mass conservation.
"""
if "## WG-7A fluvial-erosion and sediment gates" in text:
    raise SystemExit("WG-7A validation section already exists")
path.write_text(text.rstrip() + wg7 + "\n")
