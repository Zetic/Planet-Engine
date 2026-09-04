from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)


# --- Primary Lab HTML -----------------------------------------------------------
path = Path("index.html")
text = path.read_text()
text = replace_once(text, "<title>Planet Engine · Through WG-7A</title>", "<title>Planet Engine · Through WG-7B</title>", "Lab title")
text = replace_once(text, "PLANET ENGINE · THROUGH WG-7A", "PLANET ENGINE · THROUGH WG-7B", "Lab kicker")
text = replace_once(text, "value=\"interlink-wg7a\"", "value=\"interlink-wg7b\"", "Lab default seed")
text = replace_once(
    text,
    "<p>Generate one deterministic physical planet through WG-7A, then inspect topology, tectonics, geology, lithosphere, topography, climate, drainage, lakes, seasonal realized flow, erosive forcing, and sediment routing from one matched physical state.</p>",
    "<p>Generate one deterministic physical planet through WG-7B, then inspect topology, tectonics, geology, lithosphere, initial topography, climate, drainage, lakes, seasonal realized flow, erosive forcing, sediment routing, the evolved terrain surface, and rebuilt post-erosion drainage from one matched physical state.</p>",
    "Lab intro",
)
wg7a_group = '          <optgroup label="Geomorphology · Fluvial erosion / sediment (WG-7A)">'
wg7b_group = '''          <optgroup label="Geomorphology · Evolved terrain / rebuilt drainage (WG-7B)">
            <option value="evolution-solid-elevation">Evolved solid elevation</option>
            <option value="evolution-terrain-delta">Terrain elevation delta</option>
            <option value="evolution-applied-erosion">Applied erosion depth</option>
            <option value="evolution-applied-deposition">Applied land deposition depth</option>
            <option value="evolution-receiver-change">Changed drainage receivers</option>
            <option value="evolution-contributing-area">Post-erosion contributing area</option>
            <option value="evolution-potential-discharge">Post-erosion potential discharge</option>
          </optgroup>
'''
text = replace_once(text, wg7a_group, wg7b_group + wg7a_group, "WG-7B Lab diagnostic group")
old_frontier = '''        <strong>Current physical frontier: WG-7A</strong>
        <p>One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, and WG-7A fluvial erosion/sediment diagnostics in one cumulative Rust/WASM result. WG-7A references the exact accepted inheritance, topography, drainage, lake, and seasonal-hydrology identities.</p>
        <p>WG-7A derives peak-sensitive effective discharge, channel slope and hydraulic width, inherited erodibility, bounded incision potential, sediment production, carrying capacity, downstream sediment load, and deposition. Active WG-6C lake depressions are first-pass complete sediment traps, and global generated sediment is conserved into land, lake, and terminal/ocean deposition.</p>
        <p>WG-7A is deliberately non-mutating: the displayed WG-4 surface and WG-6 drainage/hydrology remain unchanged while erosional forcing is validated. Applied incision, valley development, sedimentary fill, drainage recalculation, glacier flow, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.</p>'''
new_frontier = '''        <strong>Current physical frontier: WG-7B</strong>
        <p>One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, WG-7A fluvial erosion/sediment diagnostics, and WG-7B bounded terrain evolution in one cumulative Rust/WASM result.</p>
        <p>WG-7B converts WG-7A channel incision into cell-average valley erosion, chooses one bounded adaptive geomorphic horizon, conserves the actually applied sediment mass into land deposition plus lake and terminal/ocean sinks, and creates a distinct evolved solid surface without rewriting WG-4 identity.</p>
        <p>The WG-4 ocean mask/coastline remains fixed in WG-7B v1. Drainage is rebuilt exactly once on the evolved surface and accepted WG-6B local runoff is rerouted over that new DAG for post-erosion diagnostics; climate, lake equilibrium, and seasonal hydrology are not iterated. Coastline migration, evolving lake capacity/spill thresholds, delta/coastal construction, hillslope transport, glaciers, weathering/soil/lithology, resources, Regions, Features, and gameplay integration remain downstream.</p>'''
text = replace_once(text, old_frontier, new_frontier, "WG-7B Lab frontier note")
path.write_text(text)


# --- Primary Lab controller -----------------------------------------------------
path = Path("src/worldgen/diagnostics/worldgenClimateLabStandalone.ts")
text = path.read_text()
erosion_modes = "const EROSION_MODES = new Set(['erosion-effective-discharge', 'erosion-channel-slope', 'erosion-channel-width', 'erosion-erodibility', 'erosion-incision-potential', 'erosion-sediment-supply', 'erosion-sediment-load', 'erosion-sediment-deposition']);"
evolution_modes = "const EVOLUTION_MODES = new Set(['evolution-solid-elevation', 'evolution-terrain-delta', 'evolution-applied-erosion', 'evolution-applied-deposition', 'evolution-receiver-change', 'evolution-contributing-area', 'evolution-potential-discharge']);\n"
text = replace_once(text, erosion_modes, evolution_modes + erosion_modes, "WG-7B Lab mode set")
scalar_anchor = "    case 'erosion-effective-discharge': {\n"
scalar_cases = '''    case 'evolution-solid-elevation': return { values: result.evolvedSolidElevationM, minimum: -12_000, maximum: 8_000, lowHue: 225, highHue: 25 };
    case 'evolution-terrain-delta': {
      const bound = Math.max(0.01, result.evolutionMetrics.maximumAbsoluteTerrainChangeM);
      return { values: result.terrainDeltaM, minimum: -bound, maximum: bound, lowHue: 225, highHue: 20 };
    }
    case 'evolution-applied-erosion': return { values: result.appliedErosionM, minimum: 0, maximum: Math.max(0.01, result.evolutionMetrics.maximumAppliedErosionM), lowHue: 55, highHue: 345 };
    case 'evolution-applied-deposition': return { values: result.appliedDepositionM, minimum: 0, maximum: Math.max(0.01, result.evolutionMetrics.maximumAppliedDepositionM), lowHue: 205, highHue: 45 };
    case 'evolution-receiver-change': {
      for (let index = 0; index < scalarScratch.length; index += 1) scalarScratch[index] = result.receiverChangedMask[index]!;
      return { values: scalarScratch, minimum: 0, maximum: 1, lowHue: 210, highHue: 5 };
    }
    case 'evolution-contributing-area': {
      let maximum = 0;
      for (let index = 0; index < scalarScratch.length; index += 1) {
        const value = Math.log1p(Math.max(0, result.postErosionContributingAreaM2[index]!));
        scalarScratch[index] = value;
        maximum = Math.max(maximum, value);
      }
      return { values: scalarScratch, minimum: 0, maximum: Math.max(1e-6, maximum), lowHue: 55, highHue: 205 };
    }
    case 'evolution-potential-discharge': {
      for (let index = 0; index < scalarScratch.length; index += 1) scalarScratch[index] = Math.log1p(Math.max(0, result.postErosionPotentialDischargeM3S[index]!));
      return { values: scalarScratch, minimum: 0, maximum: Math.log1p(Math.max(1e-6, result.evolutionMetrics.maximumPostErosionPotentialDischargeM3S)), lowHue: 205, highHue: 20 };
    }
'''
text = replace_once(text, scalar_anchor, scalar_cases + scalar_anchor, "WG-7B scalar diagnostics")
text = replace_once(
    text,
    "  if (EROSION_MODES.has(mode) && mode !== 'erosion-sediment-deposition' && result.submergedMask[sample]) return '#102c43';",
    "  if (EVOLUTION_MODES.has(mode) && result.submergedMask[sample]) return '#102c43';\n  if (EROSION_MODES.has(mode) && mode !== 'erosion-sediment-deposition' && result.submergedMask[sample]) return '#102c43';",
    "WG-7B ocean diagnostic mask",
)
text = replace_once(
    text,
    "  'fluvial-erosion-sediment': 'Fluvial erosion / sediment',\n  packaging: 'Packaging / transfer',",
    "  'fluvial-erosion-sediment': 'Fluvial erosion / sediment',\n  'bounded-terrain-evolution': 'Bounded terrain evolution',\n  packaging: 'Packaging / transfer',",
    "WG-7B generation-stage label",
)
metrics_anchor = "  metric(metrics, 'WG-7A erosion hash', result.erosionMetrics.fluvialErosionHash);"
metrics_new = metrics_anchor + '''
  metric(metrics, 'WG-7B / stage', `v${result.engineVersion} · ${result.evolutionStage.id}@${result.evolutionStage.version}`);
  metric(metrics, 'WG-7B geomorphic horizon', `${result.evolutionMetrics.geomorphicDurationYears.toFixed(0)} years`);
  metric(metrics, 'WG-7B changed terrain samples', `${result.evolutionMetrics.erodedSampleCount.toLocaleString()} eroded · ${result.evolutionMetrics.depositionalSampleCount.toLocaleString()} depositional`);
  metric(metrics, 'WG-7B receiver changes', `${result.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} · ${(result.evolutionMetrics.receiverChangedFraction * 100).toFixed(3)}% of land`);
  metric(metrics, 'WG-7B terrain change', `${result.evolutionMetrics.maximumAppliedErosionM.toFixed(2)} m max erosion · ${result.evolutionMetrics.maximumAppliedDepositionM.toFixed(2)} m max deposition · ${result.evolutionMetrics.meanLandAbsoluteTerrainChangeM.toFixed(3)} m mean |Δz|`);
  metric(metrics, 'WG-7B sediment generation', `${result.evolutionMetrics.totalAppliedSedimentGeneratedKgS.toFixed(1)} kg/s`);
  metric(metrics, 'WG-7B deposition land / lake / terminal-ocean', `${result.evolutionMetrics.totalLandDepositionKgS.toFixed(1)} / ${result.evolutionMetrics.totalLakeSinkKgS.toFixed(1)} / ${result.evolutionMetrics.totalTerminalOceanSinkKgS.toFixed(1)} kg/s`);
  metric(metrics, 'WG-7B sediment closure', result.evolutionMetrics.sedimentConservationRelativeError.toExponential(2));
  metric(metrics, 'WG-7B max post-erosion potential flow', `${result.evolutionMetrics.maximumPostErosionPotentialDischargeM3S.toFixed(1)} m³/s`);
  metric(metrics, 'WG-7B post-erosion runoff closure', result.evolutionMetrics.postErosionRunoffConservationRelativeError.toExponential(2));
  metric(metrics, 'WG-7B evolved surface / drainage hash', `${result.evolutionMetrics.evolvedSurfaceHash} / ${result.evolutionMetrics.postErosionDrainageHash}`);
  metric(metrics, 'WG-7B evolution hash', result.evolutionMetrics.terrainEvolutionHash);'''
text = replace_once(text, metrics_anchor, metrics_new, "WG-7B Lab metrics")
text = replace_once(
    text,
    "  status.textContent = 'Generating one physical planet through WG-7A fluvial erosion and sediment diagnostics in Rust/WASM…';",
    "  status.textContent = 'Generating one physical planet through WG-7B bounded terrain evolution in Rust/WASM…';",
    "WG-7B generation status",
)
ancestry_anchor = "    if (loaded.erosionMetrics.seasonalHydrologyHash !== loaded.seasonalMetrics.seasonalHydrologyHash) throw new Error('WG-7A seasonal identity does not match accepted WG-6D hydrology.');"
ancestry_new = ancestry_anchor + '''
    if (loaded.evolutionMetrics.topographyHash !== loaded.metrics.topographyHash) throw new Error('WG-7B topography identity does not match accepted WG-4 terrain.');
    if (loaded.evolutionMetrics.drainageHash !== loaded.drainageMetrics.drainageHash) throw new Error('WG-7B drainage identity does not match accepted WG-6A topology.');
    if (loaded.evolutionMetrics.runoffHash !== loaded.runoffMetrics.runoffHash) throw new Error('WG-7B runoff identity does not match accepted WG-6B runoff.');
    if (loaded.evolutionMetrics.lakeHash !== loaded.lakeMetrics.lakeHash) throw new Error('WG-7B lake identity does not match accepted WG-6C state.');
    if (loaded.evolutionMetrics.fluvialErosionHash !== loaded.erosionMetrics.fluvialErosionHash) throw new Error('WG-7B erosion identity does not match accepted WG-7A forcing.');'''
text = replace_once(text, ancestry_anchor, ancestry_new, "WG-7B Lab ancestry checks")
text = replace_once(
    text,
    "    generationStep.textContent = `${loaded.metrics.spinupYears} climate spin-up years · ${loaded.drainageMetrics.basinCount.toLocaleString()} basins · ${loaded.lakeMetrics.lakeCount.toLocaleString()} equilibrium lakes · ${loaded.erosionMetrics.erosiveSampleCount.toLocaleString()} erosive cells`;",
    "    generationStep.textContent = `${loaded.metrics.spinupYears} climate spin-up years · ${loaded.drainageMetrics.basinCount.toLocaleString()} basins · ${loaded.lakeMetrics.lakeCount.toLocaleString()} equilibrium lakes · ${loaded.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} receivers changed after evolution`;",
    "WG-7B generation step summary",
)
text = replace_once(
    text,
    "    status.textContent = `Planet ready through WG-7A: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.erosionMetrics.erosiveSampleCount.toLocaleString()} erosive cells, ${loaded.erosionMetrics.totalSedimentGeneratedKgS.toFixed(1)} kg/s generated sediment, closure ${loaded.erosionMetrics.sedimentConservationRelativeError.toExponential(2)}.`;",
    "    status.textContent = `Planet ready through WG-7B: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.evolutionMetrics.erodedSampleCount.toLocaleString()} evolved erosion cells, ${loaded.evolutionMetrics.receiverChangedSampleCount.toLocaleString()} drainage receivers changed, mean land |Δz| ${loaded.evolutionMetrics.meanLandAbsoluteTerrainChangeM.toFixed(3)} m, sediment closure ${loaded.evolutionMetrics.sedimentConservationRelativeError.toExponential(2)}.`;",
    "WG-7B ready status",
)
path.write_text(text)


# --- Browser regression updates -------------------------------------------------
path = Path("tests/pages.test.ts")
text = path.read_text()
text = text.replace("through WG-7A", "through WG-7B").replace("THROUGH WG-7A", "THROUGH WG-7B")
text = replace_once(text, "  assert.match(html, /Effective erosive discharge/);", "  assert.match(html, /Effective erosive discharge/);\n  assert.match(html, /Applied erosion depth/);\n  assert.match(html, /Post-erosion potential discharge/);", "Pages WG-7B diagnostics")
path.write_text(text)

path = Path("tests/wg4Topography.test.ts")
text = path.read_text()
text = text.replace("through WG-7A", "through WG-7B").replace("THROUGH WG-7A", "THROUGH WG-7B")
old_pipeline = r"one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff\/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, and WG-7A fluvial erosion\/sediment diagnostics"
new_pipeline = r"one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff\/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, WG-7A fluvial erosion\/sediment diagnostics, and WG-7B bounded terrain evolution"
text = replace_once(text, old_pipeline, new_pipeline, "WG-4 cumulative pipeline assertion")
path.write_text(text)

path = Path("tests/wg6Drainage.test.ts")
text = path.read_text().replace("THROUGH WG-7A", "THROUGH WG-7B")
path.write_text(text)

path = Path("tests/wg7Erosion.test.ts")
text = path.read_text()
text = text.replace("Current physical frontier: WG-7A", "Current physical frontier: WG-7B")
path.write_text(text)

path = Path("tests/worldgenRewrite.test.ts")
text = path.read_text().replace("through WG-7A", "through WG-7B")
path.write_text(text)

Path("tests/wg7TerrainEvolution.test.ts").write_text('''import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_PROTOCOL_VERSION } from '../dist/worldgen/protocol.js';

test('WG-7B cumulative browser contract is protocol v16 and single-request', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 16);
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');
  const html = fs.readFileSync('index.html', 'utf8');
  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  for (const field of [
    'evolutionStage', 'evolutionMetrics', 'evolvedSolidElevationM', 'terrainDeltaM',
    'appliedErosionM', 'appliedDepositionM', 'receiverChangedMask',
    'postErosionContributingAreaM2', 'postErosionPotentialDischargeM3S',
  ]) {
    assert.match(protocol, new RegExp(field));
    assert.match(worker, new RegExp(field));
  }
  assert.match(bridge, /generate_bounded_terrain_evolution/);
  assert.match(bridge, /bounded-terrain-evolution/);
  assert.match(worker, /terrain_evolution_hash_hex/);
  assert.match(worker, /post_erosion_runoff_conservation_relative_error/);
  assert.match(worker, /client\.generateClimate|generateClimate/);
  assert.doesNotMatch(worker, /generateEvolution/);
  for (const mode of [
    'evolution-solid-elevation', 'evolution-terrain-delta', 'evolution-applied-erosion',
    'evolution-applied-deposition', 'evolution-receiver-change', 'evolution-contributing-area',
    'evolution-potential-discharge',
  ]) {
    assert.match(html, new RegExp(mode));
    assert.match(lab, new RegExp(mode));
  }
  assert.match(html, /Current physical frontier: WG-7B/);
  assert.match(lab, /WG-7B sediment closure/);
  assert.match(lab, /evolutionMetrics\.fluvialErosionHash/);
  assert.match(lab, /evolutionMetrics\.topographyHash/);
  assert.match(lab, /evolutionMetrics\.drainageHash/);
  assert.match(lab, /evolutionMetrics\.runoffHash/);
  assert.match(lab, /evolutionMetrics\.lakeHash/);
});
''')


# --- Root README ---------------------------------------------------------------
path = Path("README.md")
text = path.read_text()
text = replace_once(
    text,
    "diagnostic fluvial erosion and conservative sediment routing, planetary physical profiles",
    "diagnostic fluvial erosion, conservative sediment routing, bounded terrain evolution with one post-erosion drainage rebuild, planetary physical profiles",
    "README ownership",
)
text = replace_once(
    text,
    "WG-7A fluvial erosion and sediment diagnostics\n```",
    "WG-7A fluvial erosion and sediment diagnostics\n          ↓\nWG-7B bounded terrain evolution + rebuilt drainage\n```",
    "README pipeline",
)
old_wg7a = "WG-7A consumes the accepted WG-6D phase realized-discharge field, immutable WG-4 terrain, WG-6A receiver topology, WG-6C lake control volumes, and inherited lithospheric mechanics to derive peak-sensitive effective discharge, channel slope and hydraulic width, erodibility, bounded incision potential, sediment production, transport capacity, routed load, and deposition. Active lake depressions are complete first-pass sediment traps and generated sediment is conserved into land, lake, and terminal/ocean deposition. WG-7A is deliberately diagnostic and non-mutating: applied incision, valley development, sedimentary fill, and drainage recalculation are deferred to WG-7B."
new_wg7 = old_wg7a.replace(" WG-7A is deliberately diagnostic and non-mutating: applied incision, valley development, sedimentary fill, and drainage recalculation are deferred to WG-7B.", " WG-7A remains the immutable forcing/sediment foundation consumed by WG-7B.") + "\n\nWG-7B applies WG-7A incision over a bounded channel/valley footprint to a distinct evolved terrain state, chooses one adaptive direct geomorphic horizon capped by resolved elevation change, recomputes sediment from the actually applied erosion volume, and conserves that mass into ordinary land deposition plus lake and terminal/ocean sinks. It then rebuilds WG-6A drainage exactly once on the evolved surface and reroutes accepted WG-6B local runoff over that new DAG. WG-4 identity and its ocean mask remain unchanged; WG-5 climate, WG-6C lake equilibrium, and WG-6D seasonal hydrology are not iterated in WG-7B v1."
text = replace_once(text, old_wg7a, new_wg7, "README WG-7A/WG-7B summary")
text = replace_once(text, "bash scripts/check-wg7a-erosion.sh", "bash scripts/check-wg7a-erosion.sh\nbash scripts/check-wg7b-evolution.sh", "README WG-7B smoke")
path.write_text(text)


# --- Worldgen docs index -------------------------------------------------------
path = Path("docs/worldgen-rewrite/README.md")
text = path.read_text()
text = replace_once(
    text,
    "[`WG7_EROSION.md`](WG7_EROSION.md) — WG-7A fluvial erosive forcing, hydraulic channel geometry, sediment routing/conservation, and deferred terrain mutation.",
    "[`WG7_EROSION.md`](WG7_EROSION.md) — WG-7A fluvial forcing/sediment conservation plus WG-7B bounded terrain evolution, applied sediment mass, and post-erosion drainage rebuild.",
    "Worldgen docs WG-7 link",
)
old_tail = "WG-7A deliberately leaves WG-4 terrain and WG-6 drainage/hydrology unchanged. Active WG-6C lake depressions are complete first-pass sediment traps, and global generated sediment must close into land, lake, and terminal/ocean deposition. See [`WG7_EROSION.md`](WG7_EROSION.md) for the complete causality, invariants, benchmarks, browser protocol, and WG-7B deferral contract."
new_tail = "WG-7A deliberately leaves WG-4 terrain and WG-6 drainage/hydrology unchanged. Active WG-6C lake depressions are complete first-pass sediment traps, and global generated sediment must close into land, lake, and terminal/ocean deposition.\n\n## WG-7B — bounded terrain evolution and drainage rebuild\n\nWG-7B consumes the accepted WG-7A forcing as immutable input, converts channel incision into bounded cell-average valley erosion, chooses one adaptive direct geomorphic horizon, applies ordinary land deposition from a conservative sediment ledger, creates a distinct evolved solid surface, rebuilds drainage exactly once, and reroutes accepted WG-6B local runoff over the rebuilt DAG. The WG-4 ocean mask remains fixed and WG-5/WG-6 seasonal physics are not rerun inside the stage. See [`WG7_EROSION.md`](WG7_EROSION.md) for the complete causality, invariants, browser contract, benchmarks, and deferred feedback processes."
text = replace_once(text, old_tail, new_tail, "Worldgen docs WG-7B section")
path.write_text(text)


# --- WG-7 detailed design ------------------------------------------------------
path = Path("docs/worldgen-rewrite/WG7_EROSION.md")
text = path.read_text()
text = replace_once(
    text,
    "WG-7 begins geomorphic evolution after the accepted WG-4 physical surface and WG-6 hydrology. WG-7A establishes deterministic erosive forcing and conservative sediment routing without changing terrain. WG-7B is reserved for applying that forcing to terrain and rebuilding drainage after the surface changes.",
    "WG-7 begins geomorphic evolution after the accepted WG-4 physical surface and WG-6 hydrology. WG-7A establishes deterministic erosive forcing and conservative sediment routing without changing terrain. WG-7B consumes that accepted forcing to create a distinct bounded evolved surface, apply conservative sediment deposition, rebuild drainage once, and reroute accepted annual runoff on the rebuilt DAG.",
    "WG-7 docs intro",
)
text = replace_once(
    text,
    "WG-7A advances the cumulative browser/WASM protocol to version `15`. The single cumulative planet request generates WG-7A after WG-6D; there is intentionally no separate browser `generateErosion` path that could create a mismatched physical ancestry.",
    "The cumulative browser/WASM contract is now version `16`. The single cumulative planet request still generates WG-7A after WG-6D and then WG-7B; there is intentionally no separate browser `generateErosion` or `generateEvolution` path that could create mismatched physical ancestry.",
    "WG-7 browser protocol wording",
)
text = replace_once(text, "## Deferred to WG-7B and later", "## WG-7A boundary carried into WG-7B", "WG-7A boundary heading")
text = replace_once(
    text,
    "WG-7B should begin only after the WG-7A forcing, ancestry, resolution interpretation, and sediment-conservation contracts are treated as stable inputs.",
    "WG-7B now consumes WG-7A under exactly those forcing, ancestry, resolution-interpretation, and sediment-conservation contracts.",
    "WG-7A boundary close",
)
wg7b_docs = r'''

## WG-7B stage identity

The implemented terrain-mutation stage is:

```text
geomorphology:bounded-terrain-evolution@1
```

WG-7B does not overwrite WG-4. It owns a distinct evolved-terrain state whose identity includes the stage seed, WG-7B parameter hash, accepted WG-4/WG-6/WG-7A ancestry, evolved surface, rebuilt drainage identity, applied erosion/deposition state, changed-receiver mask, and rerouted post-erosion discharge.

The generation-time causality is:

```text
accepted WG-7A incision potential + channel width + transport capacity
             +
immutable WG-4 solid terrain / sea level / ocean mask
             +
accepted WG-6A receiver DAG + depression membership
             +
accepted WG-6B local runoff + WG-6C active lake depressions
             ↓
channel-to-valley cell-average erosion rate
             ↓
one adaptive bounded geomorphic horizon
             ↓
applied erosion + conservative applied-sediment routing
             ↓
ordinary land deposition + lake sink + terminal/ocean sink
             ↓
distinct evolved solid surface
             ↓
one WG-6A drainage rebuild on evolved terrain
             ↓
accepted WG-6B local runoff rerouted over rebuilt drainage
```

### Valley-footprint terrain response

WG-7A incision is a channel-bed diagnostic and must not lower an entire dual cell by the same depth. WG-7B expands the accepted hydraulic channel width into a bounded coarse valley width and converts incision to cell-average erosion:

```text
valley_width = clamp(channel_width * valley_width_multiplier,
                     minimum_valley_width,
                     maximum_valley_width)
valley_area = min(receiver_segment_length * valley_width, dual_cell_area)
resolved_erosion_rate = incision_potential * valley_area / dual_cell_area
```

Default valley parameters are multiplier `3`, minimum width `100 m`, and maximum width `20,000 m`.

### One adaptive direct geomorphic horizon

WG-7B is not a year-stepped landscape simulator. It first estimates the maximum resolved erosion or ordinary-land deposition rate, then chooses one direct duration:

```text
duration = min(maximum_geomorphic_years,
               maximum_resolved_elevation_change / maximum_resolved_rate)
```

The defaults are `250,000 years` and `120 m`. The stage therefore remains a small set of dense passes plus one drainage rebuild instead of repeatedly invoking climate/hydrology and erosion.

Applied erosion is additionally limited by available relief. Existing WG-4 land remains at least `1 m` above the fixed WG-4 sea level, and an eroding land cell remains at least `0.1 m` above its accepted downstream land receiver. These are v1 stability/base-level guards, not claims of detailed channel-bed geometry.

### Applied sediment ledger

After bounded erosion is known, WG-7B recomputes sediment production from the **actually applied erosion volume** rather than carrying forward WG-7A diagnostic production unchanged. The default source and deposited-sediment densities are both `1,800 kg/m³`.

Applied sediment is routed once over the accepted pre-erosion WG-6A DAG using the accepted WG-7A transport-capacity field. Ordinary land deposition is converted to elevation gain on the evolved surface. Every member of an active WG-6C depression remains a complete lake sink, and terminal/ocean export is retained as an explicit sink. Lake and terminal/ocean sinks do not construct lake fill or deltas in WG-7B v1.

The required mass invariant is:

```text
applied sediment generated
  = ordinary land deposition
  + active-lake sink
  + terminal/ocean sink
```

Permanent CI requires relative closure at or below `1e-10`.

### Evolved terrain and fixed coastline

`evolved_solid_elevation_m` is a new WG-7B field. WG-4 `solid_elevation_m`, its component decomposition, topography hash, solved sea level, and submerged mask remain immutable upstream truth.

WG-7B v1 deliberately preserves the WG-4 ocean mask. Existing land cannot erode through the fixed sea-level clearance and existing ocean cells are not raised by terminal sediment. This avoids claiming coastline migration before coastal construction and water-volume/sea-level feedback are modeled together.

### One drainage rebuild and cheap runoff reroute

After applying the terrain delta, WG-7B invokes the accepted WG-6A drainage solver exactly once on the evolved solid surface while retaining the WG-4 ocean mask. It records which land samples changed receiver and the complete rebuilt contributing-area state internally.

WG-7B does **not** rerun WG-5 climate, WG-6C lake equilibrium, or WG-6D seasonal hydrology. For a first post-erosion hydrologic diagnostic, it reroutes the accepted WG-6B local annual runoff field over the rebuilt receiver DAG. This preserves annual runoff mass and exposes how changed terrain alters potential flow concentration without introducing an implicit iterative hydro-geomorphic loop.

Permanent CI requires rebuilt drainage area closure and post-erosion runoff closure at or below `1e-10`.

### WG-7B state contract

`TerrainEvolutionState` exposes or retains:

- evolved solid elevation;
- signed terrain delta;
- applied erosion depth;
- applied ordinary-land deposition depth;
- applied sediment source/load/deposition ledgers;
- a receiver-changed mask;
- the rebuilt drainage state;
- post-erosion potential annual discharge.

Metrics include the selected geomorphic duration, eroded/depositional/receiver-changed sample counts, maximum and mean terrain-change magnitudes, applied sediment source/sink totals and closure, maximum post-erosion potential discharge and runoff closure, parameter/upstream hashes, evolved-surface hash, rebuilt-drainage hash, and final terrain-evolution hash.

### Browser and Lab contract

Protocol `16` carries WG-7B in the same cumulative climate/planet result after WG-7A. The primary Lab exposes:

- evolved solid elevation;
- signed terrain elevation delta;
- applied erosion;
- applied ordinary-land deposition;
- changed receiver locations;
- post-erosion contributing area;
- post-erosion potential discharge.

The Lab validates that WG-7B references the exact displayed WG-4 topography, WG-6A drainage, WG-6B runoff, WG-6C lake, and WG-7A erosion identities. It also reports the selected horizon, terrain-change counts/magnitudes, sediment sink split and closure, post-erosion runoff closure, and WG-7B hashes.

### WG-7B invariants

1. Every input aligns on the canonical fine topology.
2. WG-7B requires exact accepted WG-4/WG-6/WG-7A ancestry.
3. WG-4 terrain identity and submerged mask remain unchanged.
4. Channel incision is converted to cell-average valley erosion before terrain mutation.
5. The direct geomorphic horizon is finite, nonnegative, and bounded by the parameterized duration/elevation-change limits.
6. Applied erosion cannot cross the fixed WG-4 land/sea or accepted downstream base-level guards.
7. Sediment production is recomputed from actually applied erosion.
8. Applied sediment closes into ordinary land deposition, active-lake sinks, and terminal/ocean sinks.
9. Only ordinary land deposition modifies elevation in WG-7B v1.
10. Drainage is rebuilt exactly once after terrain mutation.
11. Accepted WG-6B local runoff is rerouted on that rebuilt DAG without rerunning climate or seasonal hydrology.
12. Every public WG-7B diagnostic and ancestry identity participates in deterministic hashing.

### Deferred beyond WG-7B

WG-7B intentionally does not yet model:

- coastline migration or a new water-volume/sea-level solve;
- evolving lake capacity, bathymetry, spill thresholds, or lake infill;
- delta, alluvial-fan, estuary, or other coastal construction;
- iterative climate/runoff/lake/erosion feedback after each terrain update;
- explicit channel cross-sections, migration, avulsion, or floodplains;
- hillslope diffusion, mass wasting, regolith, weathering, or soil production;
- glacier flow and glacial erosion/deposition;
- detailed lithology, chemistry, resources, Regions, Features, or gameplay geography.

WG-7B is therefore the first bounded terrain-response pass, not a terminal landscape-evolution model.
'''
if "## WG-7B stage identity" in text:
    raise SystemExit("WG-7B docs section already exists")
text += wg7b_docs
path.write_text(text)


# --- Validation gates ----------------------------------------------------------
path = Path("docs/worldgen-rewrite/VALIDATION.md")
text = path.read_text()
text = replace_once(
    text,
    "browser/WASM protocol v15 carries WG-7A in the same cumulative planet result",
    "browser/WASM protocol v16 carries WG-7A and WG-7B in the same cumulative planet result",
    "Validation browser protocol",
)
wg7b_validation = r'''

## WG-7B bounded terrain-evolution gates

WG-7B is accepted as one deterministic bounded terrain-response pass after the stable WG-7A forcing stage:

- all WG-7B inputs and outputs align on the canonical fine topology;
- WG-4 topography, WG-6A drainage, WG-6B runoff, WG-6C lake, and WG-7A erosion identities must match exactly before mutation begins;
- WG-4 solid terrain identity, sea level, and submerged/ocean mask remain immutable upstream truth;
- channel incision is converted through receiver-segment length and a bounded valley footprint before becoming cell-average terrain lowering;
- the selected direct geomorphic horizon is positive for the fixed smoke case, no greater than `250,000 years` by default, and is shortened when required to respect the default `120 m` resolved elevation-change cap;
- fixed-smoke erosion and ordinary land deposition are both nonempty;
- actual applied sediment production is recomputed from bounded applied erosion volume;
- applied sediment closes into ordinary land deposition, complete active-lake-depression sinks, and terminal/ocean sinks within `1e-10`;
- only ordinary land deposition changes terrain in WG-7B v1; lake and terminal/ocean sinks remain explicit unresolved construction sinks;
- drainage is rebuilt exactly once on the evolved surface using the fixed WG-4 ocean mask;
- rebuilt drainage area closes within `1e-10`;
- the accepted WG-6B local runoff field is rerouted once over the rebuilt drainage DAG without rerunning climate, lake equilibrium, or seasonal hydrology;
- post-erosion runoff closes within `1e-10`;
- receiver changes are explicitly diagnosed rather than inferred from visual inspection;
- protocol v16 transports the WG-7B state in the same cumulative planet request, and the primary Lab verifies exact WG-4/WG-6/WG-7A ancestry before displaying the evolved state;
- deterministic WG-7B hashing covers the evolved surface, applied erosion/deposition ledgers, receiver-change mask, post-erosion drainage identity, post-erosion discharge, parameters, and accepted upstream identities.

`bash scripts/check-wg7b-evolution.sh` is the permanent fixed L4 acceptance path. Final L4/L6/L7 and fixed-ancestry L6/L7 benchmark values are recorded in `WG7_EROSION.md` after the exact-head benchmark matrix.
'''
if "## WG-7B bounded terrain-evolution gates" in text:
    raise SystemExit("WG-7B validation section already exists")
text += wg7b_validation
path.write_text(text)
