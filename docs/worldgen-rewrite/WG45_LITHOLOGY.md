# WG-4.5 — Lithology and substrate

WG-4.5 converts the accepted historical lithosphere into persistent material properties for later surface-process stages. It is deliberately downstream of tectonic/crust history and initial topography, and upstream of the later lithology-aware erosion, weathering, glacial, soil, and resource systems.

Stage identity: `geology:lithology-substrate@1`.

## Causal contract

WG-4.5 consumes the fine inherited physical state plus fine historical material identity. Bedrock is therefore derived from crust kind/age/thickness, tectonic history, inherited structures, active/fossil orogenic state, and persistent `origin_plate_id` / `fragment_id` provenance. It must not regenerate geology from terrain shape or introduce per-cell decorative noise.

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

Fragment-scale compositional variation is deterministic and keyed by historical origin/fragment identity. This preserves coherent material domains instead of painting independent sample noise over the planet.

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
- persistent fragment provenance to remain materially legible;
- material contrast not to collapse at historical fragment boundaries.

The cumulative Planet Engine Lab exposes categorical bedrock and all six continuous substrate fields so visual review can compare lithology directly with crust, provenance, tectonic structures, and physical terrain.

## Memory discipline

WG-4.5 retains one `u8` bedrock class and six `f32` material fields on the fine topology. The upstream topography scratch release remains immediately after WG-4.5, so the new persistent substrate replaces no solver-lifetime guarantees from the accepted L8 memory architecture.
