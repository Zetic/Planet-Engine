# WG-4.5 — Lithology and substrate

WG-4.5 converts the accepted historical lithosphere into persistent material properties for later surface-process stages. It is deliberately downstream of tectonic/crust history and initial topography, and upstream of the later lithology-aware erosion, weathering, glacial, soil, and resource systems.

Stage identity: `geology:lithology-substrate@2`.

## Causal contract

WG-4.5 consumes the fine inherited physical state plus fine historical identity for alignment/diagnostics. Bedrock physics is derived from crust kind, separated oceanic/reworking clocks, crustal mechanics, tectonic history, inherited structures, active/fossil orogenic state, subsidence/basin state, and continuous stability. Persistent `origin_plate_id` / `fragment_id` values are genealogy only: relabeling those categorical IDs while holding the physical state fixed must not change bedrock class or any continuous substrate property. The stage must not regenerate geology from terrain shape or introduce per-cell decorative noise.

The first-pass bedrock classes are:

- oceanic basalt;
- oceanic sediment;
- crystalline basement;
- orogenic metamorphic rock;
- arc volcanic rock;
- rift volcanic rock;
- clastic sedimentary rock;
- carbonate platform;
- accreted terrane material.

The dense normalized material fields are:

- `rock_strength_index`;
- `erodibility_index`;
- `permeability_index`;
- `weathering_susceptibility`;
- `fines_fraction`;
- `carbonate_fraction`.

Compositional variation is carried by continuous geological state rather than fragment-keyed random bias. Accreted terrane, carbonate, volcanic, sedimentary, and metamorphic classes require physical deformation, basin, thermal, stability, or structural evidence. Quiet ancestry contacts therefore do not become substrate seams merely because their IDs differ.

## Placement in the physical pipeline

The cumulative browser pipeline runs WG-4.5 after WG-4 has solved initial topography and connected-ocean hydrostatics, but before inherited WG-3.75 scratch fields are released. This ordering allows the stage to consume the historical mechanical fields without extending their lifetime through climate and hydrology.

WG-4.5 does **not** modify topography in this PR. Existing WG-7A/B erosion and terrain evolution also do **not** consume these fields yet. The following PR can make erosion substrate-aware against this accepted material contract without combining material generation and erosion tuning in one change.

## Acceptance

`lithology_substrate_acceptance` runs `interlink-wg7c`, seeds `1` and `2`, plus a holdout. It requires:

- complete finite normalized fields on the fine topology;
- multiple materially distinct bedrock classes;
- oceanic/non-oceanic class consistency;
- hard crystalline/metamorphic substrate to be substantially stronger and less erodible than sedimentary substrate;
- carbonate platforms to retain strong carbonate identity when present;
- hard/soft and carbonate contrasts to remain physically legible without categorical ancestry forcing;
- a deterministic ancestry-ID intervention to leave the complete WG-4.5 state bit-identical.

The cumulative Planet Engine Lab exposes categorical bedrock and all six continuous substrate fields so visual review can compare lithology directly with crust, provenance, tectonic structures, and physical terrain.

## Memory discipline

WG-4.5 retains one `u8` bedrock class and six `f32` material fields on the fine topology. The upstream topography scratch release remains immediately after WG-4.5, so the new persistent substrate replaces no solver-lifetime guarantees from the accepted L8 memory architecture.
