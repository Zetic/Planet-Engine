from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    text = target.read_text()
    if old not in text:
        raise SystemExit(f"marker not found in {path}: {old[:140]!r}")
    target.write_text(text.replace(old, new, 1))


# The direct Lab URL is intentionally an exact alias of the GitHub Pages root.
Path("worldgen-lab.html").write_text(Path("index.html").read_text())

replace_once(
    "tests/pages.test.ts",
    "GitHub Pages root serves the cumulative Planet Engine Lab through WG-5",
    "GitHub Pages root serves the cumulative Planet Engine Lab through WG-6A",
)
replace_once(
    "tests/pages.test.ts",
    "assert.match(html, /PLANET ENGINE · THROUGH WG-5/);",
    "assert.match(html, /PLANET ENGINE · THROUGH WG-6A/);",
)

replace_once(
    "tests/wg4Topography.test.ts",
    "Planet Engine Lab keeps every WG-3.75 and WG-4 view cumulatively under WG-5",
    "Planet Engine Lab keeps every WG-3.75 and WG-4 view cumulatively through WG-6A",
)
replace_once(
    "tests/wg4Topography.test.ts",
    "assert.match(html, /PLANET ENGINE · THROUGH WG-5/);",
    "assert.match(html, /PLANET ENGINE · THROUGH WG-6A/);",
)
replace_once(
    "tests/wg4Topography.test.ts",
    "assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, and WG-5 coupled-climate pipeline/i);",
    "assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, and WG-6A drainage topology/i);",
)
