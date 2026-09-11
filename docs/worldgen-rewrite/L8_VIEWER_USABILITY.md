# L8 viewer usability

The canonical physical world remains **L5 -> L8** and the camera never changes generation resolution. This viewer layer exists only to make the 655,362-cell result interactive.

## Rendering contract

Orthographic globe interaction uses the actual L8 dual polygons as a persistent WebGL2 surface. Each physical sample owns one triangulated dual cell, and the cell geometry is uploaded once for the generated world. Diagnostic colors live in a compact per-cell texture and update only when the diagnostic/season changes. Camera rotation and zoom are shader uniforms, so dragging never falls back to circular point sprites and never reprojects all 655,362 samples in JavaScript on every pointer event. The equirectangular map retains the CPU/Canvas path.

During active drag or wheel zoom the viewer renders the same contiguous GPU dual-cell surface used underneath settled frames, plus a compact camera indicator. Once interaction settles, the CPU projection is rebuilt once and cached for overlays. This keeps expensive topology/river/contour work out of the frame-by-frame camera loop without changing the shape of the world while the camera moves.

## Zoom and cells

Globe zoom is continuous from 1x to 24x and does not regenerate the planet. The WebGL globe is contiguous at every zoom because it renders triangulated dual cells rather than point sprites. At 4.5x and above, settled `Final physical world` and `Physical dual-cell tiles` views may additionally use the Canvas exact-cell pass for border and selection detail, but the underlying GPU surface is the same physical cell geometry during movement and at rest. The 12 required pentagons remain highlighted in tile mode. Clicking the globe resolves the nearest L8 sample by searching the inherited L5 seed set and then hill-climbing the L8 neighbor graph, and reports the stable cell/sample id plus physical context.

WebGL2 is an optimization rather than a correctness dependency: if unavailable, the existing Canvas renderer remains the fallback and still honors camera zoom.
