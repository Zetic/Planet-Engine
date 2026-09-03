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

Mean specific humidity is divided by saturation specific humidity evaluated at annual-mean temperature and pressure. Because WG-5 does not currently store seasonal humidity harmonics, this is a mean-state diagnostic rather than the annual mean of instantaneous relative humidity. Values above one are therefore diagnostic evidence of the compression/mean-state approximation and must not be interpreted as instantaneous supersaturation.

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

## Measured baseline

The initial quality ensemble was measured against the unchanged `climate:coupled-surface@2` model. The calibration layer did not change the climate hashes or physical defaults.

### L6 reference ensemble

| Seed | Land mean elevation | Land p95 elevation | Land >2 km | Land / ocean temp | SST | Energy-imbalance proxy | Moisture edge cap | Mean precip |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `interlink-wg5` | 4068 m | 9256 m | 78.0% | 231.3 / 310.6 K | 310.9 K | +51.0 W/m² | 90.0% | 186.0 mm/yr |
| `wg5-cal-a` | 3545 m | 8787 m | 63.1% | 246.6 / 302.6 K | 304.7 K | +52.7 W/m² | 89.2% | 176.2 mm/yr |
| `wg5-cal-b` | 3934 m | 9725 m | 71.2% | 238.9 / 305.3 K | 306.7 K | +50.9 W/m² | 90.1% | 190.4 mm/yr |
| `wg5-cal-c` | 3591 m | 9107 m | 64.0% | 243.3 / 304.6 K | 306.5 K | +52.0 W/m² | 89.6% | 174.1 mm/yr |

The L7 quality case (`ci-wg5-l7`) remains numerically accepted while showing the same calibration pattern: mean land elevation `3205 m`, land p95 `7770 m`, land/ocean temperature `246.9 / 304.0 K`, mean SST `306.0 K`, energy-imbalance proxy `+45.4 W/m²`, moisture-edge cap occupancy `94.8%`, and mean precipitation `182.5 mm/yr`.

### Baseline findings

1. **WG-4 hypsometry is a real upstream calibration issue.** Across the four L6 reference worlds, mean land elevation is about `3.5–4.1 km`, p95 land elevation is about `8.8–9.7 km`, and `63–78%` of land area lies above `2 km`. The climate layer should not compensate for this by adding an arbitrary land warming term; WG-4 relief amplitudes/distributions require a separate audit.

2. **The reduced thermal budget has a systematic warm-ocean / cold-land bias.** L6 mean SST is `304.7–310.9 K`, land/ocean annual-mean temperature contrast is `56–79 K`, and the current reduced energy-imbalance proxy remains about `+51 W/m²` across all four L6 seeds. The L7 quality case remains at `+45 W/m²`. This is consistent across seeds rather than being a single-world anomaly.

3. **Atmospheric moisture transport is strongly resolution-cap dominated.** Reconstructed edge-cap occupancy is about `63–65%` at L4, `89–90%` at L6, and `94.8%` at L7. This is the strongest numerical evidence that the current single graph-transfer sweep per orbital phase is not resolution-independent and should be redesigned before precipitation is tuned.

4. **The wind-speed ceiling is not driving the moisture problem.** Reconstructed wind-cap occupancy is `0%` in the L4 ensemble and only about `1.5–1.8%` in the L6/L7 cases.

5. **Low precipitation is not a moisture-conservation leak.** Global precipitation/evaporation ratio remains about `0.998–0.999`, consistent with the already accepted moisture-budget closure. The weak hydrological cycle is therefore a transport/source-sink calibration problem rather than lost water mass.

6. **Global precipitation is not dominated by the explicit orographic sink.** Disabling `maximum_orographic_fraction` on the same accepted WG-4 surface changes global mean precipitation by only about `0.10–0.22%` in the measured ensemble. Thin or terrain-aligned precipitation features may still be locally important, but the low global precipitation cannot be explained as excessive global orographic removal.

7. **The humidity proxy shows severe dry tails, especially at higher resolution.** L6 area-weighted p05 relative-humidity proxy falls to roughly `0.01` or lower, while median values stay near `0.58–0.60`. Values above one in the upper tail are expected from the documented annual-mean proxy and reinforce the need for direct seasonal humidity diagnostics before treating that tail as physical supersaturation.

8. **The exported annual-mean ocean-heat tendency retains more spatial amplitude at higher resolution.** Its ocean-only RMS index is roughly `0.03–0.04` at L4, `0.09–0.12` at L6, and `0.15` in the L7 quality case. This should be tracked when the thermal budget and heat-transport diagnostics are revised.

For the user-reference `interlink-wg5` L6 world, the latitude-band report also exposes the extreme thermal structure directly: the `0–15°` band averages about `310.3 K`, while the southern `75–90°` band averages about `168.3 K`; tropical SST is about `324 K`. This confirms that the unusual screenshot is a model-state signal, not only a palette or projection artifact.

## Baseline interpretation

The measured baseline changes the priority order for subsequent tuning work:

1. Audit WG-4 hypsometry so WG-5 is not asked to compensate for systematically elevated continents.
2. Rebuild the reduced shortwave/longwave and atmospheric heat-redistribution budget to remove the systematic hot-ocean / cold-land regime.
3. Replace the cap-dominated single-sweep atmospheric moisture transport with a resolution-aware climatological transport solve.
4. Only after the thermal and moisture transport regimes are corrected, tune precipitation mechanisms and diagnostics.
5. Re-evaluate PET, aridity, snow, and sea-ice potentials against the corrected thermal/hydrological state.

No WG-5 default should be tuned merely to make a screenshot look better; subsequent changes should be evaluated against these fixed measurements and the existing determinism, conservation, convergence, and L7 acceptance contracts.
