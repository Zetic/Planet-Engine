# Boundary-localized orogens (WG-3.6 v2 / WG-4 v13)

This rewrite makes convergent boundaries the organizing spine of major relief while treating broad continental plateaus as an exceptional collision outcome.

## Mechanical changes

- Province width is a deformation-reach scale, not a direct mountain footprint.
- Collision propagation is a weighted graph solve. Strong, thick pre-orogenic lithosphere raises travel cost; inherited weak fabric, rifts, shear zones, and crustal discontinuities lower the effective resistance upstream.
- A separate narrow `mountain_core_index` represents the high-relief range. Crustal root, fold-thrust, foreland, plateau, suture, arc, and back-arc fields remain distinct.
- Plateau classification now requires mature, strongly shortened, mechanically permissive continental collision. Plateau relief is lower-amplitude than the mountain core and predominantly hinterland-side.
- Subduction relief is explicitly one-sided: trench/boundary -> inland arc mountain core -> back-arc. Arc offset is expressed in physical kilometres rather than as a fraction of an arbitrarily broad province.
- Fine-grid structural fields are still scratch state and are released immediately after WG-4, preserving the L8 memory architecture.

## Geometric changes

- Nominal collision reaches are hundreds rather than thousands of kilometres; only exceptional plateau systems can approach ~1000 km source scale.
- Active deformation is cut off before source footprints can spread across an entire continent.
- Source/Voronoi seams are locally blended only within the same connected province and plate, so the numerical ownership tessellation does not directly appear as triangular relief.
- End tapers remain causal and connected to boundary-system topology.

## Acceptance invariants

The blocking tests require that stronger continental interiors cost more to deform and that samples belonging to the high-relief mountain core remain within 800 km of their connected convergent boundary. These are structural invariants, not legacy morphology calibration targets.
