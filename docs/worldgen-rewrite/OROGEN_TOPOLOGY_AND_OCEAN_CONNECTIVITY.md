# Orogen topology and connected ocean (WG-3.6 v3 / WG-4 v14)

This pass fixes the failure exposed by `interlink-wg7c` after boundary localization: ranges had become narrow, nearly uniform boundary ribbons and the model created a broad negative lobe beside almost every range. The global hydrostatic threshold then interpreted each below-datum continental trough as ocean, producing repeated inland mountain-and-moat geometry.

## Orogen topology

- Major collision relief still originates from connected convergent systems, but high mountains now require accumulated shortening rather than convergence alone.
- Inherited weakness, age/province discontinuities, fragment contacts, and boundary curvature focus the load along strike. This preserves mountain-prone convergent margins without making every boundary edge equally tall.
- Collision deformation reach is widened enough to support range + hinterland + fold/thrust structure while remaining far below the old continent-scale province widths.
- The mountain core, crustal root, plateau, fold/thrust belt, and transpressional response overlap more broadly, so the visible range is not a one-cell-style boundary ribbon.
- Foreland and back-arc fields remain causal diagnostics, but their negative topographic response is now conditional on actual mountain/arc load and is hundreds rather than thousands of metres by default. Negative relief is no longer an acceptance requirement.

## Connected ocean

WG-4 no longer treats `solid_elevation < global_sea_level` as sufficient to create ocean water. The final initial-ocean solve starts from oceanic-crust reservoir cells and admits additional terrain only when a below-water graph path reaches it. Water is redistributed over the reached domain while preserving the configured inventory.

This means a continental basin may legitimately sit below the global ocean datum and remain dry. If marine water physically overtops a sill, the basin can join the reached water domain. Closed-basin lake state remains the responsibility of WG-6.

## Blocking invariants

- connected-ocean unit coverage proves that a below-datum lowland separated from the ocean seed by high terrain is not flooded;
- the same-seed WG-4 causal smoke no longer requires negative orogenic relief;
- fewer than 5% of continental foreland samples may be initially submerged on the canonical smoke world;
- fewer than 10% of all continental orogenic-province samples may be initially submerged;
- existing mountain-core boundary-localization and finite/bounded-state invariants remain blocking.
