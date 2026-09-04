from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)

path = Path('docs/worldgen-rewrite/WG7_EROSION.md')
text = path.read_text()
anchor = '### Deferred beyond WG-7B\n'
section = r'''### Final WG-7B benchmark matrix

The final benchmark matrix was run on GitHub Actions Ubuntu 24.04 with Rust `1.98.1`, optimized release builds, and three WG-7B-only timed runs after upstream state construction. Run `33923547403` benchmarked commit `2099497ede3272ed6ed71495c12d61f1bf60769c`. The WG-7B parameter hash is `79ebfd14fdef843c` in every case.

Fixed release seed `ci-wg7b-evolution`:

| fine level | coarse level | plates | samples | runtime mean / median (ms) | eroded / depositional | receiver changes | max erosion / deposition (m) | mean land |Δz| (m) | sediment closure | drainage area closure | runoff closure |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| L4 | L3 | 12 | 2,562 | 0.630 / 0.631 | 487 / 124 | 1 | 6.526032 / 1.146689 | 0.431530 | `3.657e-16` | `1.541e-16` | `0` |
| L6 | L4 | 16 | 40,962 | 8.547 / 8.611 | 8,938 / 3,141 | 30 | 38.269152 / 31.045940 | 0.800932 | `1.774e-16` | `3.264e-15` | `1.942e-16` |
| L7 | L5 | 24 | 163,842 | 43.938 / 44.813 | 30,026 / 14,497 | 70 | 35.117153 / 30.102793 | 1.128892 | `1.988e-15` | `4.269e-15` | `1.059e-14` |

All three fixed-release cases selected the full `250,000 year` bounded horizon. Their applied-sediment and post-erosion identities are:

| fine level | generated / land / lake / terminal-ocean sediment (kg/s) | max post-erosion Q (m³/s) | evolution hash | evolved-surface hash | rebuilt-drainage hash | WG-7A erosion hash |
|---:|---:|---:|---|---|---|---|
| L4 | 9,948.766885 / 161.149188 / 235.152654 / 9,552.465043 | 82,860.638672 | `71838b2cc6b700f7` | `910e5346a04431f7` | `017602cbf617a7b5` | `60c2ee7716d00fd2` |
| L6 | 20,504.592989 / 1,205.766005 / 2,793.767165 / 16,505.059818 | 493,990.306288 | `a29a936533c787c4` | `a9da933d095ecf17` | `afa34e4e3e4e05c7` | `975860e59fbbaf1f` |
| L7 | 25,620.296763 / 1,150.176735 / 1,027.475446 / 23,442.644583 | 144,990.635417 | `606bd14884243c3d` | `0a974d1f538c4f8c` | `f370f70d1392efeb` | `d290e5f38b0114a7` |

Fixed accepted WG-5 ancestry seed `ci-wg5-l7`, coarse L5, 24 plates:

| fine level | samples | runtime mean / median (ms) | eroded / depositional | receiver changes | max erosion / deposition / |Δz| (m) | mean land |Δz| (m) | sediment closure | drainage area closure | runoff closure |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| L6 | 40,962 | 8.730 / 8.815 | 9,139 / 3,179 | 13 | 27.247991 / 36.617950 / 36.420056 | 0.631664 | `0` | `3.115e-15` | `1.335e-15` |
| L7 | 163,842 | 46.904 / 46.339 | 36,961 / 16,520 | 134 | 57.346347 / 102.291346 / 101.656348 | 0.784727 | `5.295e-16` | `1.438e-14` | `2.574e-15` |

Both accepted-ancestry cases also selected `250,000 years`. L6 generated `16,507.432963 kg/s` of applied sediment, closing into `728.049710` land, `1,152.293564` lake, and `14,627.089689 kg/s` terminal/ocean deposition; its evolution/surface/drainage hashes are `6f6e08c69167ea2a`, `40ae5f0105a3e36c`, and `5e2f369357847826`. L7 generated `20,610.611447 kg/s`, closing into `1,629.925343` land, `2,229.895484` lake, and `16,750.790621 kg/s` terminal/ocean deposition; its evolution/surface/drainage hashes are `a60c68094eda5373`, `12c720550eb3fa63`, and `e4aa3b27b0d61992`.

The incremental stage is comfortably below its performance policy: fixed-release L7 is `43.938 ms` mean and accepted-ancestry L7 is `46.904 ms` mean, versus the `150 ms` preferred target and `250 ms` profiling trigger. The benchmark also demonstrates that terrain mutation can change basin/depression counts on accepted ancestry (L6: 1,916→1,915 basins and 50→49 depressions; L7: 4,395→4,394 basins and 110→107 depressions) while preserving drainage-area and rerouted-runoff conservation.

'''
if '### Final WG-7B benchmark matrix' in text:
    raise SystemExit('WG-7B benchmark section already exists')
text = replace_once(text, anchor, section + anchor, 'WG-7B benchmark insertion')
path.write_text(text)

path = Path('docs/worldgen-rewrite/VALIDATION.md')
text = path.read_text()
text = replace_once(
    text,
    '`bash scripts/check-wg7b-evolution.sh` is the permanent fixed L4 acceptance path. Final L4/L6/L7 and fixed-ancestry L6/L7 benchmark values are recorded in `WG7_EROSION.md` after the exact-head benchmark matrix.',
    '`bash scripts/check-wg7b-evolution.sh` is the permanent fixed L4 acceptance path. Final L4/L6/L7 and fixed-ancestry L6/L7 benchmark values from benchmark run `33923547403` are recorded in `WG7_EROSION.md`.',
    'WG-7B validation benchmark wording',
)
path.write_text(text)
