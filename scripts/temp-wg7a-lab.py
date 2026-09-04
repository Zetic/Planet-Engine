from pathlib import Path


def once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one target, found {count}")
    return text.replace(old, new, 1)


# Cumulative Planet Engine Lab HTML.
path = Path("index.html")
text = path.read_text()
text = once(text, "<title>Planet Engine · Through WG-6D</title>", "<title>Planet Engine · Through WG-7A</title>", "page title")
text = once(text, "<p class=\"worldgen-lab-kicker\">PLANET ENGINE · THROUGH WG-6D</p>", "<p class=\"worldgen-lab-kicker\">PLANET ENGINE · THROUGH WG-7A</p>", "page kicker")
text = once(
    text,
    "<p>Generate one deterministic physical planet through WG-6D, then inspect topology, tectonics, geology, lithosphere, topography, climate, drainage, lakes, snowmelt timing, and seasonal realized flow from one matched physical state.</p>",
    "<p>Generate one deterministic physical planet through WG-7A, then inspect topology, tectonics, geology, lithosphere, topography, climate, drainage, lakes, seasonal realized flow, erosive forcing, and sediment routing from one matched physical state.</p>",
    "page intro",
)
text = once(text, 'value="interlink-wg6d"', 'value="interlink-wg7a"', "default seed")
seasonal_group = """          <optgroup label="Hydrology · Seasonal flow / storage (WG-6D)">
"""
erosion_group = """          <optgroup label="Geomorphology · Fluvial erosion / sediment (WG-7A)">
            <option value="erosion-effective-discharge">Effective erosive discharge</option>
            <option value="erosion-channel-slope">Channel slope</option>
            <option value="erosion-channel-width">Hydraulic channel width</option>
            <option value="erosion-erodibility">Inherited erodibility</option>
            <option value="erosion-incision-potential">Incision potential</option>
            <option value="erosion-sediment-supply">Local sediment supply</option>
            <option value="erosion-sediment-load">Routed sediment load</option>
            <option value="erosion-sediment-deposition">Sediment deposition</option>
          </optgroup>
          <optgroup label="Hydrology · Seasonal flow / storage (WG-6D)">
"""
text = once(text, seasonal_group, erosion_group, "WG-7A diagnostic group")
old_note = """        <strong>Current physical frontier: WG-6D</strong>
        <p>One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff/discharge, WG-6C lake equilibrium, and WG-6D seasonal hydrology in one cumulative Rust/WASM result. WG-6D references the exact accepted WG-5, WG-6A, WG-6B, and WG-6C identities.</p>
        <p>Overlays remain independent of the selected diagnostic, so seasonal hydrology can be compared directly against topography, coastlines, tectonic boundaries, winds, and currents. WG-6D uses the retained WG-5 orbital phases, carries snow storage and snowmelt timing, routes phase discharge over WG-6A, and advances WG-6C lake control volumes through the year before classifying realized flow as dry, intermittent, or perennial.</p>
        <p>The physical surface remains pre-erosional. River incision, sediment transport, glacier flow, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.</p>
"""
new_note = """        <strong>Current physical frontier: WG-7A</strong>
        <p>One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, WG-4 topography, WG-5 coupled climate, WG-6A drainage topology, WG-6B annual runoff/discharge, WG-6C lake equilibrium, WG-6D seasonal hydrology, and WG-7A fluvial erosion/sediment diagnostics in one cumulative Rust/WASM result. WG-7A references the exact accepted inheritance, topography, drainage, lake, and seasonal-hydrology identities.</p>
        <p>WG-7A derives peak-sensitive effective discharge, channel slope and hydraulic width, inherited erodibility, bounded incision potential, sediment production, carrying capacity, downstream sediment load, and deposition. Active WG-6C lake depressions are first-pass complete sediment traps, and global generated sediment is conserved into land, lake, and terminal/ocean deposition.</p>
        <p>WG-7A is deliberately non-mutating: the displayed WG-4 surface and WG-6 drainage/hydrology remain unchanged while erosional forcing is validated. Applied incision, valley development, sedimentary fill, drainage recalculation, glacier flow, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.</p>
"""
text = once(text, old_note, new_note, "frontier note")
path.write_text(text)


# Lab implementation.
path = Path("src/worldgen/diagnostics/worldgenClimateLabStandalone.ts")
text = path.read_text()
text = once(
    text,
    "const LAKE_MODES = new Set(['realized-discharge', 'lake-depth', 'lake-state', 'lake-fraction']);\n",
    "const EROSION_MODES = new Set(['erosion-effective-discharge', 'erosion-channel-slope', 'erosion-channel-width', 'erosion-erodibility', 'erosion-incision-potential', 'erosion-sediment-supply', 'erosion-sediment-load', 'erosion-sediment-deposition']);\nconst LAKE_MODES = new Set(['realized-discharge', 'lake-depth', 'lake-state', 'lake-fraction']);\n",
    "erosion mode set",
)
scalar_anchor = """    case 'seasonal-snow-storage': {
      seasonalPhaseRate(result.seasonalPhaseSnowStorageMm, result.seasonalMetrics.orbitalPhaseCount, result.seasonalMetrics.sampleCount, phase, seasonalScratch);
      for (let index = 0; index < scalarScratch.length; index += 1) {
        const value = Math.max(0, seasonalScratch[index]!);
        scalarScratch[index] = value / (value + 500);
      }
      return { values: scalarScratch, minimum: 0, maximum: 1, lowHue: 225, highHue: 175 };
    }
"""
scalar_insert = scalar_anchor + """    case 'erosion-effective-discharge': {
      for (let index = 0; index < scalarScratch.length; index += 1) scalarScratch[index] = Math.log1p(Math.max(0, result.effectiveDischargeM3S[index]!));
      return { values: scalarScratch, minimum: 0, maximum: Math.log1p(Math.max(1e-6, result.erosionMetrics.maximumEffectiveDischargeM3S)), lowHue: 205, highHue: 20 };
    }
    case 'erosion-channel-slope': return { values: result.channelSlope, minimum: 0, maximum: Math.max(1e-6, result.erosionMetrics.maximumChannelSlope), lowHue: 52, highHue: 350 };
    case 'erosion-channel-width': return { values: result.channelWidthM, minimum: 0, maximum: Math.max(1, result.erosionMetrics.maximumChannelWidthM), lowHue: 210, highHue: 30 };
    case 'erosion-erodibility': return { values: result.erodibilityIndex, minimum: 0.05, maximum: 1, lowHue: 125, highHue: 5 };
    case 'erosion-incision-potential': return { values: result.incisionPotentialMPerYear, minimum: 0, maximum: Math.max(1e-9, result.erosionMetrics.maximumIncisionPotentialMPerYear), lowHue: 55, highHue: 345 };
    case 'erosion-sediment-load': {
      for (let index = 0; index < scalarScratch.length; index += 1) scalarScratch[index] = Math.log1p(Math.max(0, result.sedimentLoadKgS[index]!));
      return { values: scalarScratch, minimum: 0, maximum: Math.log1p(Math.max(1e-6, result.erosionMetrics.maximumSedimentLoadKgS)), lowHue: 45, highHue: 300 };
    }
    case 'erosion-sediment-supply': {
      let maximum = 0;
      for (let index = 0; index < scalarScratch.length; index += 1) {
        const value = Math.log1p(Math.max(0, result.localSedimentSupplyKgS[index]!));
        scalarScratch[index] = value;
        maximum = Math.max(maximum, value);
      }
      return { values: scalarScratch, minimum: 0, maximum: Math.max(1e-6, maximum), lowHue: 58, highHue: 325 };
    }
    case 'erosion-sediment-deposition': {
      let maximum = 0;
      for (let index = 0; index < scalarScratch.length; index += 1) {
        const value = Math.log1p(Math.max(0, result.sedimentDepositionKgS[index]!));
        scalarScratch[index] = value;
        maximum = Math.max(maximum, value);
      }
      return { values: scalarScratch, minimum: 0, maximum: Math.max(1e-6, maximum), lowHue: 35, highHue: 285 };
    }
"""
text = once(text, scalar_anchor, scalar_insert, "erosion scalar diagnostics")
text = once(
    text,
    "  if (mode === 'inherited-mask') return result.inheritedSampleMask[sample] ? '#f4e27a' : '#5794c8';\n  if (field) return scalarColor(field.values[sample]!, field);\n",
    "  if (mode === 'inherited-mask') return result.inheritedSampleMask[sample] ? '#f4e27a' : '#5794c8';\n  if (EROSION_MODES.has(mode) && result.submergedMask[sample]) return '#102c43';\n  if (field) return scalarColor(field.values[sample]!, field);\n",
    "erosion ocean mask",
)
text = once(
    text,
    "  'seasonal-hydrology': 'Seasonal hydrology',\n  packaging: 'Packaging / transfer',\n",
    "  'seasonal-hydrology': 'Seasonal hydrology',\n  'fluvial-erosion-sediment': 'Fluvial erosion / sediment',\n  packaging: 'Packaging / transfer',\n",
    "erosion generation label",
)
metrics_anchor = "  metric(metrics, 'WG-6D seasonal hash', result.seasonalMetrics.seasonalHydrologyHash);\n"
metrics_new = metrics_anchor + """  metric(metrics, 'WG-7A / stage', `v${result.engineVersion} · ${result.erosionStage.id}@${result.erosionStage.version}`);
  metric(metrics, 'WG-7A erosive samples / lake traps', `${result.erosionMetrics.erosiveSampleCount.toLocaleString()} erosive · ${result.erosionMetrics.activeLakeTrapCount.toLocaleString()} lake traps receiving sediment`);
  metric(metrics, 'WG-7A effective flow / slope / width', `${result.erosionMetrics.maximumEffectiveDischargeM3S.toFixed(1)} m³/s · ${result.erosionMetrics.maximumChannelSlope.toFixed(5)} · ${result.erosionMetrics.maximumChannelWidthM.toFixed(1)} m max`);
  metric(metrics, 'WG-7A max incision potential', `${result.erosionMetrics.maximumIncisionPotentialMPerYear.toFixed(6)} m/yr`);
  metric(metrics, 'WG-7A sediment generation', `${result.erosionMetrics.totalSedimentGeneratedKgS.toFixed(1)} kg/s`);
  metric(metrics, 'WG-7A deposition land / lake / terminal-ocean', `${result.erosionMetrics.totalLandDepositionKgS.toFixed(1)} / ${result.erosionMetrics.totalLakeDepositionKgS.toFixed(1)} / ${result.erosionMetrics.totalTerminalOceanDepositionKgS.toFixed(1)} kg/s`);
  metric(metrics, 'WG-7A sediment closure', result.erosionMetrics.sedimentConservationRelativeError.toExponential(2));
  metric(metrics, 'WG-7A erosion hash', result.erosionMetrics.fluvialErosionHash);
"""
text = once(text, metrics_anchor, metrics_new, "erosion Lab metrics")
text = once(
    text,
    "  status.textContent = 'Generating one physical planet through WG-6D seasonal hydrology in Rust/WASM…';\n",
    "  status.textContent = 'Generating one physical planet through WG-7A fluvial erosion and sediment diagnostics in Rust/WASM…';\n",
    "erosion generation status",
)
ancestry_anchor = """    if (loaded.seasonalMetrics.lakeHash !== loaded.lakeMetrics.lakeHash) throw new Error('WG-6D lake identity does not match accepted WG-6C state.');
"""
ancestry_new = ancestry_anchor + """    if (loaded.erosionMetrics.inheritanceHash !== loaded.metrics.inheritanceHash) throw new Error('WG-7A inheritance identity does not match accepted fine physical state.');
    if (loaded.erosionMetrics.topographyHash !== loaded.metrics.topographyHash) throw new Error('WG-7A topography identity does not match accepted WG-4 terrain.');
    if (loaded.erosionMetrics.drainageHash !== loaded.drainageMetrics.drainageHash) throw new Error('WG-7A drainage identity does not match accepted WG-6A topology.');
    if (loaded.erosionMetrics.lakeHash !== loaded.lakeMetrics.lakeHash) throw new Error('WG-7A lake identity does not match accepted WG-6C state.');
    if (loaded.erosionMetrics.seasonalHydrologyHash !== loaded.seasonalMetrics.seasonalHydrologyHash) throw new Error('WG-7A seasonal identity does not match accepted WG-6D hydrology.');
"""
text = once(text, ancestry_anchor, ancestry_new, "erosion ancestry checks")
text = once(
    text,
    "    generationStep.textContent = `${loaded.metrics.spinupYears} climate spin-up years · ${loaded.drainageMetrics.basinCount.toLocaleString()} basins · ${loaded.lakeMetrics.lakeCount.toLocaleString()} equilibrium lakes · ${loaded.seasonalMetrics.intermittentFlowSampleCount.toLocaleString()} intermittent flow cells`;\n",
    "    generationStep.textContent = `${loaded.metrics.spinupYears} climate spin-up years · ${loaded.drainageMetrics.basinCount.toLocaleString()} basins · ${loaded.lakeMetrics.lakeCount.toLocaleString()} equilibrium lakes · ${loaded.erosionMetrics.erosiveSampleCount.toLocaleString()} erosive cells`;\n",
    "erosion generation summary",
)
text = once(
    text,
    "    status.textContent = `Planet ready through WG-6D: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.lakeMetrics.lakeCount.toLocaleString()} equilibrium lakes, ${loaded.seasonalMetrics.intermittentFlowSampleCount.toLocaleString()} intermittent and ${loaded.seasonalMetrics.perennialFlowSampleCount.toLocaleString()} perennial realized-flow cells.`;\n",
    "    status.textContent = `Planet ready through WG-7A: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${loaded.erosionMetrics.erosiveSampleCount.toLocaleString()} erosive cells, ${loaded.erosionMetrics.totalSedimentGeneratedKgS.toFixed(1)} kg/s generated sediment, closure ${loaded.erosionMetrics.sedimentConservationRelativeError.toExponential(2)}.`;\n",
    "erosion ready status",
)
path.write_text(text)


# Browser regression coverage.
path = Path("tests/pages.test.ts")
text = path.read_text()
text = text.replace("through WG-6D", "through WG-7A")
text = text.replace("PLANET ENGINE · THROUGH WG-6D", "PLANET ENGINE · THROUGH WG-7A")
text = once(text, "  assert.match(html, /Potential annual discharge/);\n", "  assert.match(html, /Effective erosive discharge/);\n  assert.match(html, /Potential annual discharge/);\n", "page erosion diagnostic")
path.write_text(text)

path = Path("tests/wg7Erosion.test.ts")
text = path.read_text()
text = once(
    text,
    "  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');\n",
    "  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/climate_bridge.rs', 'utf8');\n  const html = fs.readFileSync('index.html', 'utf8');\n  const lab = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');\n",
    "WG-7A Lab test setup",
)
text = once(
    text,
    "  assert.doesNotMatch(worker, /generateErosion/);\n",
    "  assert.doesNotMatch(worker, /generateErosion/);\n  for (const mode of ['erosion-effective-discharge', 'erosion-channel-slope', 'erosion-channel-width', 'erosion-erodibility', 'erosion-incision-potential', 'erosion-sediment-supply', 'erosion-sediment-load', 'erosion-sediment-deposition']) {\n    assert.match(html, new RegExp(mode));\n    assert.match(lab, new RegExp(mode));\n  }\n  assert.match(html, /Current physical frontier: WG-7A/);\n  assert.match(lab, /WG-7A sediment closure/);\n  assert.match(lab, /erosionMetrics\.seasonalHydrologyHash/);\n  assert.match(lab, /erosionMetrics\.topographyHash/);\n",
    "WG-7A Lab assertions",
)
path.write_text(text)

path = Path("tests/worldgenRewrite.test.ts")
text = path.read_text().replace("through WG-6D", "through WG-7A")
path.write_text(text)
