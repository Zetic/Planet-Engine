# WG-5 Climate Calibration Baseline

This document defines the measurement layer used before changing WG-5 climate equations or default parameters. It is intentionally diagnostic-only: the calibration report reads the accepted WG-4/WG-5 state and does not modify generation, hashes, stage identity, or browser protocol output.

## Purpose

WG-5 already has deterministic generation, convergence rejection, moisture conservation, L7 resolution acceptance, and causal regression tests. Those contracts establish numerical validity, but they do not establish that an Earth-like reference world occupies a climatologically plausible regime.

The calibration layer therefore records physical-scale signals that can explain visually suspicious output before tuning begins:

- latitude-band annual temperature, precipitation, humidity, SST, snowfall, and sea-ice potential;
- area-weighted land hypsometry and highland fractions;
- clear-surface absorbed-shortwave proxy using the configured land/ocean albedos;
- outgoing-longwave proxy using the current reduced gray-atmosphere relation;
- resulting top-of-atmosphere energy-imbalance proxy;
- reconstructed occupancy of the wind-speed cap;
- reconstructed occupancy of the atmospheric moisture edge-transfer cap;
- mean-state relative-humidity percentiles;
- annual precipitation, PET, and global precipitation/evaporation ratio;
- a causal no-orography intervention on the same accepted WG-4 planet;
- snowfall-mass proxy and existing persistent-snow / sea-ice area potentials;
- spatial RMS of the exported annual-mean ocean heat-advection tendency index.

These values are diagnostics, not acceptance thresholds in this first calibration PR.

## Important proxy semantics

Several quantities are deliberately named as proxies because the current public `ClimateState` stores compressed climatology rather than every orbital-phase intermediate.

### Clear-surface absorbed shortwave

`annual_mean_insolation * (1 - configured surface albedo)` is area-weighted over the accepted surface. It intentionally excludes the dynamic snow/ice feedback and therefore answers a narrow question: how much shortwave the current land/ocean albedo choice admits before cryosphere feedback.

### Outgoing-longwave proxy

The report applies the inverse of the current reduced gray-atmosphere greenhouse multiplier to the stored annual-mean surface temperature. It is useful for diagnosing the current thermal regime, but it is not a radiative-transfer model or a strict globally closed energy budget.

### Relative-humidity proxy

Mean specific humidity is divided by saturation specific humidity evaluated at annual-mean temperature and pressure. Because WG-5 does not currently store seasonal humidity harmonics, this is a mean-state diagnostic rather than the annual mean of instantaneous relative humidity.

### Reconstructed transport-cap occupancy

Wind and moisture-edge cap occupancy are reconstructed from the stored first annual wind harmonics at the configured orbital phases. They indicate whether the retained climatology lives near numerical caps; they are not a replacement for future direct solver instrumentation if exact phase-level cap accounting becomes necessary.

### Orographic precipitation causal fraction

The calibration example reruns WG-5 on the exact same accepted WG-4 surface with `maximum_orographic_fraction = 0`. The difference in global mean precipitation is reported as a causal sensitivity to the orographic sink. Because the climate is coupled, this is intentionally an intervention result rather than an algebraic decomposition of each precipitation event.

### Ocean heat tendency RMS index

The current public field is a signed annual-mean local SST tendency index. The report records its ocean-only spatial RMS so annual positive/negative values do not disappear into one global signed mean. A later diagnostic PR may replace or supplement this with direct phase-level heat-transport magnitude.

## Repeatable ensembles

The repository provides:

```bash
bash scripts/run-wg5-calibration.sh smoke
bash scripts/run-wg5-calibration.sh standard
bash scripts/run-wg5-calibration.sh quality
```

`smoke` runs one low-resolution `interlink-wg5` case.

`standard` runs fixed three-seed L4 and four-case L6 ensembles, including the visual-reference seed `interlink-wg5`.

`quality` runs the standard ensemble and then the permanent L7 quality seed `ci-wg5-l7`.

The underlying single-case command is:

```bash
cargo run --release -p interlink-worldgen-cli --example climate_calibration -- \
  --seed interlink-wg5 \
  --coarse-level 4 \
  --level 6 \
  --plates 16
```

Use `--skip-orography-intervention` when only the reference climate is needed.

## Baseline interpretation

The first tuning pass should use this report to answer, in order:

1. Is WG-4 hypsometry itself producing unusually high continental surfaces across seeds?
2. Does the current shortwave/longwave proxy explain the hot-ocean / cold-land split?
3. Are wind or moisture transport caps controlling a large fraction of the climatology?
4. Is the atmosphere globally moisture-starved, or is precipitation primarily failing to reach land interiors?
5. How strongly does total precipitation depend on the orographic sink?
6. Are snow/sea-ice potentials primarily consequences of thermal bias rather than independent cryosphere behavior?

No WG-5 default should be tuned merely to make a screenshot look better until these measurements have been collected across the fixed ensemble.
