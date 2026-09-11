# L8 viewer usability

The canonical physical world remains **L5 -> L8** and the camera never changes generation resolution. This viewer layer exists only to make the 655,362-cell result interactive.

## Rendering contract

Orthographic globe interaction uses the actual L8 dual polygons as one persistent WebGL2 surface from 1x through 24x. Each physical sample owns one triangulated dual cell, the polygon mesh and its perimeter-edge index buffer are uploaded once for the generated world, and diagnostic colors live in a compact per-cell texture. Camera rotation and zoom are shader uniforms. The viewer does not switch to a second CPU terrain renderer at a zoom threshold.

The front hemisphere uses a compressed clip-space depth range so valid unit-sphere geometry never sits directly on the WebGL near plane. This prevents camera-centered clear-color holes while preserving front-to-back depth ordering. Tile mode draws the same GPU cell polygons plus GPU perimeter lines, so borders remain present while the camera is rotating or zooming.

Enabled diagnostic overlays stay on the Canvas overlay layer during interaction. CPU sample projection is skipped entirely for the common no-overlay globe path; it is refreshed only when an active overlay or diagnostic line layer needs projected sample coordinates. The equirectangular map retains the CPU/Canvas fallback path.

## Zoom and cells

Globe zoom is continuous from 1x to 24x and does not regenerate the planet or change renderers. The WebGL globe is contiguous at every zoom because it renders triangulated dual cells rather than point sprites. Tile borders fade in as cells become screen-resolved instead of appearing through a hard 4.5x renderer transition. Clicking the globe resolves the nearest L8 sample by searching the inherited L5 seed set and then hill-climbing the L8 neighbor graph, and reports the stable cell/sample id plus physical context.

The old lower-right five-ring dual-cell lens has been removed. Direct zoom, cell borders, clicking, and the cell inspector are the supported inspection path.

The lab starts in **Custom** with `Physical elevation / bathymetry` and no overlays. Choosing Custom explicitly restores that clean state; manually changing a diagnostic or overlay keeps the preset labeled Custom without erasing the user's selections.

WebGL2 is an optimization rather than a correctness dependency: if unavailable, the Canvas renderer remains the fallback and still honors camera zoom.
