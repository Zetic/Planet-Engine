# L8 viewer usability

The canonical physical world remains **L5 -> L8** and the camera never changes generation resolution. This viewer layer exists only to make the 655,362-cell result interactive.

## Rendering contract

Orthographic globe interaction uses the actual L8 dual polygons as one persistent WebGL2 surface from 1x through 24x. Each physical sample owns one triangulated dual cell, the polygon mesh includes a center-to-perimeter interpolation weight, and diagnostic colors live in a compact per-cell texture. Camera rotation and zoom are shader uniforms. The viewer does not switch to a second CPU terrain renderer at a zoom threshold.

The front hemisphere uses a compressed clip-space depth range so valid unit-sphere geometry never sits directly on the WebGL near plane. This prevents camera-centered clear-color holes while preserving front-to-back depth ordering. Physical cell boundaries are an overlay on the same GPU cell polygons rather than a separate diagnostic mode, so borders can be combined with any surface diagnostic and remain present while the camera is rotating or zooming.

Enabled diagnostic overlays stay on the Canvas overlay layer during interaction. CPU sample projection is skipped entirely for the common no-overlay globe path; it is refreshed only when an active overlay or diagnostic line layer needs projected sample coordinates. The equirectangular map is inverse-rasterized: each output pixel is mapped back to a spherical direction and resolved to the nearest L8 physical sample by neighbor-graph hill climbing. The map therefore renders a continuous field instead of flattening equal-area sample dots into polar fans.

## Zoom and cells

View zoom is continuous from 1x to 24x and does not regenerate the planet or change renderers. The WebGL globe is contiguous at every zoom because it renders triangulated dual cells rather than point sprites. The equirectangular map uses the same zoom control, cursor-anchored wheel zoom, drag-to-pan interaction, and click-to-inspect workflow; its inverse raster is rebuilt at reduced resolution during active dragging and at full viewport resolution once interaction settles. The optional globe cell-boundary overlay fades in as cells become screen-resolved instead of appearing through a hard 4.5x renderer transition. Clicking either projection resolves the nearest L8 sample and the inspector reports the stable cell/sample id plus physical context.

The old lower-right five-ring dual-cell lens has been removed. Direct zoom, cell borders, clicking, and the cell inspector are the supported inspection path.

The lab starts in **Custom** with `Physical elevation / bathymetry` and no overlays. Choosing Custom explicitly restores that clean state; manually changing a diagnostic or overlay keeps the preset labeled Custom without erasing the user's selections.

WebGL2 is an optimization rather than a correctness dependency: if unavailable, the Canvas renderer remains the fallback and still honors camera zoom.

## Projection switching

Only one projection surface is visible at a time. The WebGL surface canvas is explicitly hidden when the equirectangular map is active, including an author-level `[hidden]` CSS rule so the viewport's generic canvas display rule cannot resurrect the globe underneath the map. Switching back to the globe restores the WebGL surface and leaves the Canvas layer available for overlays.
