# L8 viewer usability

The canonical physical world remains **L5 -> L8** and the camera never changes generation resolution. This viewer layer exists only to make the 655,362-cell result interactive.

## Rendering contract

Orthographic globe interaction uses a WebGL2 point surface as the low-cost camera preview. L8 positions are uploaded once as a static GPU buffer; diagnostic colors are rebuilt only when the diagnostic/season changes. Camera rotation and zoom are shader uniforms, so dragging no longer reprojects all 655,362 samples in JavaScript on every pointer event. At settled high zoom, the final physical world and tile diagnostic stop using point sprites and render the actual contiguous L8 dual polygons across the viewport. The equirectangular map retains the CPU/Canvas path.

During active drag or wheel zoom the viewer intentionally renders only the GPU surface and a compact camera indicator. Once interaction settles, the CPU projection is rebuilt once and cached for overlays. This keeps expensive topology/river/contour work out of the frame-by-frame camera loop.

## Zoom and cells

Globe zoom is continuous from 1x to 24x and does not regenerate the planet. At 4.5x and above, settled `Final physical world` and `Physical dual-cell tiles` views render exact spherical Voronoi/dual polygons for every cell whose projected center intersects the viewport plus a small safety margin. The final physical view hides cell borders for a continuous terrain surface; tile mode draws the same selectable cells with hex/pent boundaries. Geometry is cached with a bounded per-world cache, and the WebGL point cloud remains only the active drag/zoom preview. The 12 required pentagons remain highlighted in tile mode. Clicking the globe resolves the nearest L8 sample by searching the inherited L5 seed set and then hill-climbing the L8 neighbor graph, and reports the stable cell/sample id plus physical context.

WebGL2 is an optimization rather than a correctness dependency: if unavailable, the existing Canvas renderer remains the fallback and still honors camera zoom.
