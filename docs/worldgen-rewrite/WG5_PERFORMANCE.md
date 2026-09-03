# WG-5 performance hardening

This work optimizes the accepted `climate:coupled-surface@5` implementation without reopening WG-5 physical calibration.

## Baseline and target

The Planet Lab L4→L6 reference case currently spends essentially all generation time in climate spin-up (about 19.3 s of 19.5 s in the measured browser run). The performance branch therefore treats WG-5 as the sole optimization target until climate ceases to dominate total generation time.

Use `scripts/benchmark-wg5-performance.sh` for repeatable native release benchmarks:

- `l4`: coarse L3 → fine L4, 12 plates;
- `l6`: coarse L4 → fine L6, 16 plates;
- `l7`: coarse L5 → fine L7, 24 plates;
- `suite`: all three.

Set `RUNS=N` to repeat the climate solve against the same already-generated terrain and report mean/median climate-only runtime.

## Acceptance hierarchy

1. Allocation, copy, and cached-geometry changes should preserve the Stage-5 climate hash exactly where practical.
2. Numerical early-exit changes may change floating-point details only after Stage-5 hydroclimate, convergence, conservation, determinism, and WASM parity acceptance remain satisfied.
3. Reduced phase count or multiresolution climate is experimental and must not become the default unless it preserves accepted thermal, precipitation, PET/aridity, snow/ice, wind/current, and seasonal-climate behavior.

## Planned order

1. benchmark and operation instrumentation;
2. reusable climate scratch workspace;
3. eliminate repeated phase/substep allocation and copies;
4. cache per-phase moisture edge transport coefficients;
5. optimize the conservative moisture transport passes;
6. reuse atmospheric/ocean iterative-solver scratch buffers;
7. evaluate physically safe solver convergence exits;
8. only if still necessary, investigate temporal and multiresolution reductions.

WG-6 hydrology remains out of scope for this PR.
