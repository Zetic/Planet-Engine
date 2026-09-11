# L8 canonical physical world

Planet Engine treats **L8** as the authoritative final physical topology. The accepted coarse-to-fine production path is **L5 -> L8**: coarse tectonic/geological/lithosphere structure is inherited and refined onto 655,362 final samples before WG-4 through WG-7D complete the physical state.

Lower levels remain supported for smoke tests, diagnostics, convergence work, and internal/coarse solvers. They are not the calibration target for resolution-sensitive claims such as coastline morphology, drainage topology, depressions, lake-size distributions, or final geomorphology.

## Dual-cell gameplay view

Every L8 sample owns its spherical Voronoi/dual cell. On the closed icosphere this yields **655,362 physical cells: 655,350 hexagons and the 12 topologically required pentagons**. The orthographic viewer renders those cells as the canonical GPU surface for every diagnostic. `Physical cell boundaries` is an overlay, not a separate surface diagnostic, so the same hexagon/pentagon topology can be inspected over elevation, climate, hydrology, geology, or any other surface coloring.

Cell perimeters are kept in a GPU index buffer and fade in when cells become screen-resolved. Clicking the globe selects the nearest L8 cell; the selected cell is filled and outlined regardless of the active diagnostic or boundary-overlay state. This is a visualization of the existing physical topology only; it does not add a separate gameplay grid or alter physical state.

## Calibration

Future population calibration and main-vs-candidate ensemble comparisons should use L8 as the authoritative fine level. Reduced-resolution runs may be used to catch obvious regressions quickly, but they are not sufficient evidence for tuning resolution-sensitive physical distributions.

## Measured L8 feasibility

A full native L5 -> L8 WG-7D calibration probe completed successfully at 655,362 samples. On a GitHub-hosted Ubuntu runner the generated binary took about 15.5 seconds after compilation and the complete command peaked at roughly 1.52 GB RSS.

A packaged WASM probe completed the same full L8 physical pipeline in about 28.7 seconds. Reproducing the browser worker's complete typed-array packaging retained 137 arrays totaling about 655 MiB and peaked near 2.27 GiB RSS in Node. These are development-runner measurements, not a browser memory guarantee. Because the canonical packet is materially heavier than the old L6 default, the Pages lab now waits for an explicit **Generate Planet** action instead of allocating L8 automatically on page load.
