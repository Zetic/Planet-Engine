# Tectonic structural strands (WG-3.6 v5)

WG-3.6 v5 is the visual follow-up to continental tectonic morphology v4. The v4 architecture removed hard single-source ownership, but visual acceptance showed that major collision ranges still read as direct copies of the plate-boundary graph.

## Representation change

Collision source profiles now define broad deformation envelopes rather than final mountain geometry. Inside each active collision province, v5 scores candidate structural corridors from inherited mobile belts, weak crust, shear/province transfer zones, paleo-sutures and rift memory. Per-province upper-quantile candidates seed primary and secondary structural strands. Those strands then propagate over the spherical adjacency graph on the same plate, with reduced but nonzero transfer across neighboring collision provinces.

The final mountain core is redistributed onto those graph strands. The old boundary-normal ridge is retained only as a weak fallback where no coherent strand exists. This allows range axes to step, branch, migrate hundreds of kilometres inland, and cross source/province ownership seams without turning tectonics into arbitrary noise.

## Continental interiors

Shield/root support and fossil mobile-belt relief are strengthened so that stable continents retain visible tectonic grain at map scale. Active orogens remain much stronger than fossil structure.

## Acceptance target

Same-seed physical-elevation maps should visibly differ from v4 even though coastlines and plate assembly remain unchanged. Mature continental collisions should contain separated or offset belts, literal convergent edges should often lie beside rather than under the highest range axis, junctions should spread into transfer complexes, and large continental interiors should show broad shield/mobile-belt structure rather than featureless procedural mottling. Subduction cordilleras may remain tightly boundary-correlated.

Hydrology, erosion and climate are intentionally unchanged by this PR.

## Verification

The v5 invariants require off-boundary mountain cores while rejecting runaway multi-thousand-kilometre propagation. The WG-4 cutover smoke applies the same structural-strand acceptance after coarse-to-fine inheritance. Workspace compilation, causal-cut tests, WG-3.6 invariants, tectonic smoke tests, browser regressions, browser-bridge compilation, and committed WASM parity are the automated gates before same-seed visual acceptance.
