from pathlib import Path
import re

p = Path("index.html")
s = p.read_text()
s = s.replace('id="worldgen-coarse-level" type="number" min="0" max="6" value="5"',
              'id="worldgen-coarse-level" type="number" min="0" max="6" value="6"', 1)
start = s.index('          <label>View preset')
end = s.index('          <details id="worldgen-overlays"', start)
block = '''          <label>View preset
            <select id="worldgen-preset">
              <option value="physical-world">Physical world</option>
              <option value="tectonic-history">Tectonic history</option>
              <option value="crust-lithology">Crust &amp; lithology</option>
              <option value="climate">Climate</option>
              <option value="hydrologic-atlas">Hydrologic atlas</option>
              <option value="seasonal-world">Seasonal physical world</option>
              <option value="geomorphic-processes">Geomorphic evolution</option>
              <option value="custom" selected>Custom</option>
            </select>
          </label>
          <label>Category
            <select id="worldgen-diagnostic-category">
              <option value="world">World</option>
              <option value="tectonics-history">Tectonics &amp; history</option>
              <option value="crust-lithosphere">Crust &amp; lithosphere</option>
              <option value="lithology-substrate">Lithology &amp; substrate</option>
              <option value="topography-forcing">Topography &amp; forcing</option>
              <option value="climate">Climate</option>
              <option value="hydrology">Hydrology</option>
              <option value="geomorphology">Geomorphology</option>
              <option value="technical">Technical / provenance</option>
            </select>
          </label>
          <label>Diagnostic
            <select id="worldgen-visualization">
              <optgroup label="World" data-diagnostic-category="world">
                <option value="physical-world">Final physical world</option>
                <option value="physical-elevation" selected>Physical elevation / bathymetry</option>
                <option value="relative-elevation">Elevation above sea level</option>
                <option value="solid-elevation">Solid elevation / datum</option>
                <option value="land-water">Land / water</option>
                <option value="water-depth">Bathymetry / water depth</option>
              </optgroup>
              <optgroup label="Tectonics &amp; history" data-diagnostic-category="tectonics-history">
                <option value="plates">Macro plate ownership</option>
                <option value="kinematic-domains">Refined kinematic domains</option>
                <option value="historical-origin">Ancestral origin plates</option>
                <option value="historical-fragments">Persistent crust fragments</option>
                <option value="historical-current">Current plate ownership</option>
                <option value="historical-provenance">WG-3 crust provenance</option>
                <option value="historical-crust-birth-age">Crust formation / birth age</option>
                <option value="historical-event">Latest historical event type</option>
                <option value="historical-event-age">Latest historical event age</option>
                <option value="historical-rift">Historical rift intensity</option>
                <option value="historical-rift-age">Historical rift age</option>
                <option value="historical-shear">Historical shear intensity</option>
                <option value="historical-suture">Historical suture intensity</option>
                <option value="historical-suture-age">Historical suture age</option>
                <option value="historical-passive-margin">Passive margin potential</option>
                <option value="historical-active-orogen">Active orogen intensity</option>
                <option value="historical-fossil-orogen">Fossil orogen intensity</option>
              </optgroup>
              <optgroup label="Crust &amp; lithosphere" data-diagnostic-category="crust-lithosphere">
                <option value="crust-type">Crust type</option>
                <option value="crust-age">Crust age</option>
                <option value="crust-thickness">Crust thickness</option>
                <option value="orogeny-history">Orogenic history</option>
                <option value="ridge-history">Ridge history</option>
                <option value="trench-history">Trench history</option>
                <option value="strength">Lithospheric strength</option>
                <option value="weakness">Lithospheric weakness</option>
                <option value="dynamic-support">Mantle dynamic support (index)</option>
                <option value="structural-zones">Structural zone type</option>
                <option value="fragmentation">Fragmentation propensity</option>
              </optgroup>
              <optgroup label="Lithology &amp; substrate" data-diagnostic-category="lithology-substrate">
                <option value="bedrock-class">Bedrock class</option>
                <option value="rock-strength">Rock strength</option>
                <option value="lithology-erodibility">Lithology erodibility</option>
                <option value="permeability">Permeability</option>
                <option value="weathering-susceptibility">Weathering susceptibility</option>
                <option value="fines-fraction">Fines fraction</option>
                <option value="carbonate-fraction">Carbonate fraction</option>
              </optgroup>
              <optgroup label="Topography &amp; forcing" data-diagnostic-category="topography-forcing">
                <option value="isostatic">Isostatic support</option>
                <option value="thermal">Oceanic thermal subsidence</option>
                <option value="orogenic-relief">Orogenic / collision uplift</option>
                <option value="ridge-relief">Ridge relief</option>
                <option value="rift-basin">Rift / basin subsidence</option>
                <option value="trench-relief">Trench relief</option>
                <option value="arc-relief">Volcanic arc relief</option>
                <option value="mantle-relief">Mantle dynamic support (relief)</option>
              </optgroup>
              <optgroup label="Climate" data-diagnostic-category="climate">
                <option value="temperature">Annual mean temperature</option>
                <option value="seasonal-temperature">Seasonal temperature</option>
                <option value="temperature-range">Annual temperature range</option>
                <option value="annual-insolation">Annual mean insolation</option>
                <option value="seasonal-insolation">Seasonal insolation amplitude</option>
                <option value="sst">Annual mean sea-surface temperature</option>
                <option value="seasonal-sst">Seasonal sea-surface temperature</option>
                <option value="wind-speed">Annual mean wind speed</option>
                <option value="surface-pressure">Surface atmospheric pressure</option>
                <option value="current-speed">Annual mean current speed</option>
                <option value="ocean-heat">Ocean heat transport</option>
                <option value="humidity">Atmospheric specific humidity</option>
                <option value="precipitation">Annual precipitation</option>
                <option value="seasonal-precipitation">Seasonal precipitation</option>
                <option value="precip-seasonality">Annual precipitation seasonality</option>
                <option value="potential-evaporation">Potential evaporation</option>
                <option value="moisture-balance">Moisture balance</option>
                <option value="aridity">Aridity index</option>
                <option value="snowfall">Snowfall fraction</option>
                <option value="persistent-snow">Persistent snow potential</option>
                <option value="sea-ice">Sea-ice potential</option>
              </optgroup>
              <optgroup label="Hydrology" data-diagnostic-category="hydrology">
                <option value="contributing-area">Contributing drainage area</option>
                <option value="basins">Drainage basins / outlets</option>
                <option value="flow-direction">Flow receivers</option>
                <option value="depression-depth">Depression depth</option>
                <option value="depressions">Depression regions</option>
                <option value="escape-elevation">Hydrologic escape elevation</option>
                <option value="potential-discharge">Potential annual discharge</option>
                <option value="annual-runoff">Annual runoff depth</option>
                <option value="runoff-fraction">Runoff fraction</option>
                <option value="actual-et">Actual evapotranspiration</option>
                <option value="realized-discharge">Realized annual discharge</option>
                <option value="lake-depth">Equilibrium lake depth</option>
                <option value="lake-state">Lake state</option>
                <option value="lake-fraction">Lake surface fraction</option>
                <option value="seasonal-realized-discharge">Seasonal realized discharge</option>
                <option value="seasonal-flow-presence">Realized-flow presence fraction</option>
                <option value="seasonal-flow-regime">Intermittent / perennial flow regime</option>
                <option value="seasonal-snow-storage">Seasonal snow storage</option>
                <option value="reconciliation-lake-depth-delta">Lake depth change after erosion</option>
                <option value="reconciliation-lake-change">Lake state changed</option>
                <option value="reconciliation-realized-discharge-delta">Annual realized-discharge change</option>
                <option value="reconciliation-flow-presence-delta">Flow-presence change</option>
                <option value="reconciliation-flow-regime-change">Flow regime changed</option>
              </optgroup>
              <optgroup label="Geomorphology" data-diagnostic-category="geomorphology">
                <option value="erosion-effective-discharge">Effective erosive discharge</option>
                <option value="erosion-channel-slope">Channel slope</option>
                <option value="erosion-channel-width">Hydraulic channel width</option>
                <option value="erosion-erodibility">Inherited erodibility</option>
                <option value="erosion-incision-potential">Incision potential</option>
                <option value="erosion-sediment-supply">Local sediment supply</option>
                <option value="erosion-sediment-load">Routed sediment load</option>
                <option value="erosion-sediment-deposition">Sediment deposition</option>
                <option value="evolution-solid-elevation">Evolved solid elevation</option>
                <option value="evolution-terrain-delta">Terrain elevation delta</option>
                <option value="evolution-applied-erosion">Applied erosion depth</option>
                <option value="evolution-applied-deposition">Applied land deposition depth</option>
                <option value="evolution-receiver-change">Changed drainage receivers</option>
                <option value="evolution-contributing-area">Post-erosion contributing area</option>
                <option value="evolution-potential-discharge">Post-erosion potential discharge</option>
                <option value="infill-solid-elevation">Final post-infill solid elevation</option>
                <option value="infill-fill-depth">Lake sediment fill depth</option>
              </optgroup>
              <optgroup label="Technical / provenance" data-diagnostic-category="technical">
                <option value="inherited-mask">Inherited coarse samples</option>
                <option value="provenance">Nearest coarse provenance</option>
                <option value="boundary-provenance">Boundary provenance</option>
                <option value="mesh">Fine topology mesh</option>
              </optgroup>
            </select>
          </label>
          <div id="worldgen-diagnostic-legend" class="worldgen-diagnostic-legend" aria-live="polite"></div>
'''
s = s[:start] + block + s[end:]
p.write_text(s)

p = Path("styles/worldgenLab.css")
s = p.read_text()
if ".worldgen-diagnostic-legend {" not in s:
    s += '''
.worldgen-diagnostic-legend { display:grid; gap:7px; padding:10px; border:1px solid #253246; background:#0a121c; color:#9eb0c4; font-size:11px; line-height:1.35; }
.worldgen-diagnostic-legend > strong { color:#c1d0df; font-size:12px; font-weight:600; }
.worldgen-diagnostic-legend p { margin:0; color:#8197ae; }
.worldgen-diagnostic-legend small { color:#6f849d; }
.worldgen-legend-swatches { display:grid; grid-template-columns:repeat(auto-fit,minmax(125px,1fr)); gap:5px 8px; }
.worldgen-legend-swatches > span { display:flex; align-items:center; gap:6px; min-width:0; }
.worldgen-legend-swatches i { flex:0 0 13px; width:13px; height:13px; border:1px solid rgba(255,255,255,0.18); }
.worldgen-legend-ramp { height:12px; border:1px solid rgba(255,255,255,0.18); }
.worldgen-legend-scale { display:grid; grid-template-columns:1fr 1fr 1fr; gap:6px; color:#a9bfd8; font-variant-numeric:tabular-nums; }
.worldgen-legend-scale span:nth-child(2) { text-align:center; }
.worldgen-legend-scale span:last-child { text-align:right; }
'''
p.write_text(s)

p = Path("src/worldgen/diagnostics/worldgenClimateLabStandalone.ts")
s = p.read_text()
s = s.replace("  WORLDGEN_INVALID_SAMPLE_ID,\n  WORLDGEN_PROTOCOL_VERSION,",
              "  WORLDGEN_INVALID_SAMPLE_ID,\n  WORLDGEN_CLIMATE_COARSE_MAX_LEVEL,\n  WORLDGEN_CLIMATE_FINE_MAX_LEVEL,\n  WORLDGEN_PROTOCOL_VERSION,", 1)
s = s.replace("const preset = element<HTMLSelectElement>('worldgen-preset');\nconst visualization = element<HTMLSelectElement>('worldgen-visualization');",
              "const preset = element<HTMLSelectElement>('worldgen-preset');\nconst diagnosticCategory = element<HTMLSelectElement>('worldgen-diagnostic-category');\nconst visualization = element<HTMLSelectElement>('worldgen-visualization');\nconst diagnosticLegend = element<HTMLElement>('worldgen-diagnostic-legend');", 1)
s = s.replace("  geology: 'Geological history',\n  lithosphere: 'Lithosphere',",
              "  geology: 'Geological history',\n  lithosphere: 'Lithosphere',\n  'lithology-substrate': 'Lithology / substrate',", 1)

marker = "\ntype ViewPreset = { mode: string; overlays: string[] };"
if marker not in s:
    raise SystemExit("view preset marker not found")
helpers = r'''
type LegendItem = { label: string; color: string };
const DIAGNOSTIC_UNITS: Record<string, string> = {
  'solid-elevation':'m','relative-elevation':'m','water-depth':'m','isostatic':'m','thermal':'m','orogenic-relief':'m',
  'ridge-relief':'m','rift-basin':'m','trench-relief':'m','arc-relief':'m','mantle-relief':'m','historical-crust-birth-age':'Myr',
  'historical-event-age':'Myr','historical-rift-age':'Myr','historical-suture-age':'Myr','crust-age':'Myr','crust-thickness':'km',
  'annual-insolation':'W/m²','seasonal-insolation':'W/m²','temperature':'K','seasonal-temperature':'K','temperature-range':'K',
  'sst':'K','seasonal-sst':'K','surface-pressure':'Pa','wind-speed':'m/s','current-speed':'m/s','humidity':'kg/kg',
  'precipitation':'mm/yr','seasonal-precipitation':'mm/yr','potential-evaporation':'mm/yr','moisture-balance':'mm/yr',
  'reconciliation-lake-depth-delta':'m','infill-solid-elevation':'m','infill-fill-depth':'m','evolution-solid-elevation':'m',
  'evolution-terrain-delta':'m','evolution-applied-erosion':'m','evolution-applied-deposition':'m','erosion-channel-width':'m',
  'erosion-incision-potential':'m/yr','erosion-effective-discharge':'m³/s','seasonal-realized-discharge':'m³/s',
  'reconciliation-realized-discharge-delta':'m³/s','evolution-potential-discharge':'m³/s','erosion-sediment-supply':'kg/s',
  'erosion-sediment-load':'kg/s','erosion-sediment-deposition':'kg/s'
};
const CATEGORICAL_LEGENDS: Record<string, LegendItem[]> = {
  'land-water': [{label:'Land',color:'#a99b72'},{label:'Ocean / water',color:'#214d7a'}],
  'crust-type': [{label:'Continental',color:'#b79a72'},{label:'Transitional',color:'#9aab87'},{label:'Oceanic',color:'#477aa3'}],
  'bedrock-class': [
    {label:'Oceanic basalt',color:'#355f7c'},{label:'Oceanic sediment',color:'#768896'},{label:'Crystalline basement',color:'#9c765d'},
    {label:'Orogenic metamorphic',color:'#7b657d'},{label:'Arc volcanic',color:'#a94c3d'},{label:'Rift volcanic',color:'#b97842'},
    {label:'Clastic sedimentary',color:'#c0a477'},{label:'Carbonate platform',color:'#ddd5a5'},{label:'Accreted terrane',color:'#6f8b68'}
  ],
  'structural-zones': [{label:'None',color:'#425362'},{label:'Suture',color:'#ff7466'},{label:'Rift',color:'#ffb45d'},{label:'Transform',color:'#c690ff'},{label:'Continental margin',color:'#65d7ac'}],
  'seasonal-flow-regime': [{label:'Dry',color:'#31423c'},{label:'Intermittent',color:'#e3a54f'},{label:'Perennial',color:'#4ea7dd'},{label:'Ocean',color:'#102c43'}],
  'lake-state': [{label:'No lake',color:'#31423c'},{label:'Endorheic',color:'#3aa7c9'},{label:'Overflowing',color:'#63d0a5'},{label:'Terminal storage',color:'#9b78d0'},{label:'Ocean',color:'#102c43'}],
  'inherited-mask': [{label:'Inherited coarse sample',color:'#f4e27a'},{label:'Fine-only sample',color:'#5794c8'}]
};
const IDENTITY_MODES = new Set(['plates','kinematic-domains','historical-origin','historical-fragments','historical-current','historical-provenance','provenance','boundary-provenance','basins','flow-direction','depressions']);
const LOG_DISPLAY_MODES = new Set(['seasonal-realized-discharge','evolution-contributing-area','evolution-potential-discharge','erosion-effective-discharge','erosion-sediment-load','erosion-sediment-supply','erosion-sediment-deposition']);
function diagnosticOption(mode = visualization.value): HTMLOptionElement | null {
  return Array.from(visualization.options).find(option => option.value === mode) ?? null;
}
function categoryForDiagnostic(mode: string): string | null {
  const group = diagnosticOption(mode)?.parentElement;
  return group instanceof HTMLOptGroupElement ? group.dataset.diagnosticCategory ?? null : null;
}
function setDiagnosticCategory(categoryId: string, preferredMode?: string): void {
  const groups = Array.from(visualization.querySelectorAll<HTMLOptGroupElement>('optgroup[data-diagnostic-category]'));
  let first: HTMLOptionElement | null = null;
  for (const group of groups) {
    const active = group.dataset.diagnosticCategory === categoryId;
    group.hidden = !active;
    group.disabled = !active;
    if (active && !first) first = group.querySelector('option');
  }
  if (preferredMode && categoryForDiagnostic(preferredMode) === categoryId) visualization.value = preferredMode;
  else if (first) visualization.value = first.value;
}
function selectDiagnostic(mode: string): void {
  const categoryId = categoryForDiagnostic(mode);
  if (categoryId) {
    diagnosticCategory.value = categoryId;
    setDiagnosticCategory(categoryId, mode);
  } else visualization.value = mode;
}
function diagnosticLabel(mode = visualization.value): string {
  return diagnosticOption(mode)?.textContent?.trim() || mode;
}
function formatLegendNumber(mode: string, value: number): string {
  let display = LOG_DISPLAY_MODES.has(mode) ? Math.expm1(value) : value;
  if (!Number.isFinite(display)) display = 0;
  const abs = Math.abs(display);
  const digits = abs >= 1000 ? 0 : abs >= 100 ? 1 : abs >= 10 ? 2 : abs >= 1 ? 2 : 3;
  const unit = DIAGNOSTIC_UNITS[mode];
  return display.toFixed(digits) + (unit ? ' ' + unit : '');
}
function addLegendSwatches(items: LegendItem[]): void {
  const list = document.createElement('div');
  list.className = 'worldgen-legend-swatches';
  for (const item of items) {
    const row = document.createElement('span');
    const swatch = document.createElement('i');
    swatch.style.background = item.color;
    row.append(swatch, document.createTextNode(item.label));
    list.append(row);
  }
  diagnosticLegend.append(list);
}
function addLegendGradient(lowColor: string, highColor: string, lowLabel: string, middleLabel: string, highLabel: string): void {
  const ramp = document.createElement('div');
  ramp.className = 'worldgen-legend-ramp';
  ramp.style.background = 'linear-gradient(90deg, ' + lowColor + ', ' + highColor + ')';
  const labels = document.createElement('div');
  labels.className = 'worldgen-legend-scale';
  for (const value of [lowLabel,middleLabel,highLabel]) {
    const span = document.createElement('span');
    span.textContent = value;
    labels.append(span);
  }
  diagnosticLegend.append(ramp, labels);
}
function customGradientForMode(mode: string): [string,string,string,string,string] | null {
  const gradients: Record<string,[number,number,string]> = {
    'runoff-fraction':[48,205,'fraction'],'actual-et':[42,168,'mm/yr'],'annual-runoff':[44,218,'mm/yr'],
    'potential-discharge':[215,18,'m³/s'],'realized-discharge':[205,35,'m³/s'],'lake-fraction':[210,175,'fraction'],
    'lake-depth':[220,175,'m'],'depression-depth':[55,270,'m'],'escape-elevation':[220,20,'m'],'contributing-area':[225,42,'km²']
  };
  const entry = gradients[mode];
  if (!entry) return null;
  return [drainageScalarColor(0,entry[0],entry[1]),drainageScalarColor(1,entry[0],entry[1]),'low ' + entry[2],'mid','high ' + entry[2]];
}
function refreshDiagnosticLegend(): void {
  diagnosticLegend.replaceChildren();
  const mode = visualization.value;
  const heading = document.createElement('strong');
  heading.textContent = diagnosticLabel(mode);
  const description = document.createElement('p');
  description.textContent = diagnosticLabel(mode) + ' diagnostic. Colors below are the display encoding for this view.';
  diagnosticLegend.append(heading, description);
  const categorical = CATEGORICAL_LEGENDS[mode];
  if (categorical) {
    addLegendSwatches(categorical);
    return;
  }
  if (IDENTITY_MODES.has(mode)) {
    addLegendSwatches([{label:'ID / domain A',color:historicalIdentityColor(1,42)},{label:'ID / domain B',color:historicalIdentityColor(2,42)},{label:'ID / domain C',color:historicalIdentityColor(3,42)}]);
    const note = document.createElement('small');
    note.textContent = 'Hue identifies a deterministic categorical ID; color ordering is not numeric.';
    diagnosticLegend.append(note);
    return;
  }
  if (mode === 'physical-world' || mode === 'physical-elevation') {
    addLegendSwatches([{label:'Deep ocean',color:'#20516c'},{label:'Shelf / shallow sea',color:'#a4dce1'},{label:'Lowland',color:'#7faa55'},{label:'Highland',color:'#b49b63'}]);
    return;
  }
  if (mode === 'mesh') {
    addLegendSwatches([{label:'Fine topology edge',color:'#5d7890'}]);
    return;
  }
  if (current) {
    const field = scalarField(current, mode, orbitalPhase());
    if (field) {
      const middle = (field.minimum + field.maximum) * 0.5;
      addLegendGradient('hsl(' + field.lowHue + ' 68% 37%)','hsl(' + field.highHue + ' 68% 60%)',
        formatLegendNumber(mode,field.minimum),formatLegendNumber(mode,middle),formatLegendNumber(mode,field.maximum));
      return;
    }
  }
  const custom = customGradientForMode(mode);
  if (custom) addLegendGradient(custom[0],custom[1],custom[2],custom[3],custom[4]);
  else {
    const note = document.createElement('small');
    note.textContent = 'Legend becomes data-scaled after a planet has been generated.';
    diagnosticLegend.append(note);
  }
}
function bedrockLabel(kind: number): string {
  if (kind === WORLDGEN_BEDROCK_OCEANIC_BASALT) return 'Oceanic basalt';
  if (kind === WORLDGEN_BEDROCK_OCEANIC_SEDIMENT) return 'Oceanic sediment';
  if (kind === WORLDGEN_BEDROCK_CRYSTALLINE_BASEMENT) return 'Crystalline basement';
  if (kind === WORLDGEN_BEDROCK_OROGENIC_METAMORPHIC) return 'Orogenic metamorphic';
  if (kind === WORLDGEN_BEDROCK_ARC_VOLCANIC) return 'Arc volcanic';
  if (kind === WORLDGEN_BEDROCK_RIFT_VOLCANIC) return 'Rift volcanic';
  if (kind === WORLDGEN_BEDROCK_CLASTIC_SEDIMENTARY) return 'Clastic sedimentary';
  if (kind === WORLDGEN_BEDROCK_CARBONATE_PLATFORM) return 'Carbonate platform';
  if (kind === WORLDGEN_BEDROCK_ACCRETED_TERRANE) return 'Accreted terrane';
  return 'Bedrock ' + kind;
}
function selectedDiagnosticSampleText(result: WorldgenClimateResult, sample: number): string {
  const mode = visualization.value;
  if (mode === 'land-water') return result.submergedMask[sample] ? 'Water' : 'Land';
  if (mode === 'bedrock-class') return bedrockLabel(result.bedrockClass[sample]!);
  if (mode === 'plates') return 'Plate ' + result.plateIds[sample]!.toLocaleString();
  if (mode === 'kinematic-domains') return 'Kinematic domain ' + result.kinematicDomainIds[sample]!.toLocaleString();
  if (mode === 'historical-origin') return 'Ancestral plate ' + result.originPlateIds[sample]!.toLocaleString();
  if (mode === 'historical-fragments') return 'Fragment ' + result.historicalFragmentIds[sample]!.toLocaleString();
  if (mode === 'historical-current') return 'Current plate ' + result.currentPlateIds[sample]!.toLocaleString();
  if (mode === 'annual-runoff') return result.localRunoffMm[sample]!.toFixed(1) + ' mm/yr runoff';
  if (mode === 'runoff-fraction') return (result.runoffFraction[sample]! * 100).toFixed(1) + '% runoff';
  if (mode === 'actual-et') return result.actualEvapotranspirationMm[sample]!.toFixed(1) + ' mm/yr AET';
  if (mode === 'potential-discharge') return result.potentialDischargeM3S[sample]!.toFixed(1) + ' m³/s potential discharge';
  if (mode === 'realized-discharge') return result.realizedDischargeM3S[sample]!.toFixed(1) + ' m³/s realized discharge';
  if (mode === 'lake-depth') return result.lakeDepthM[sample]!.toFixed(1) + ' m lake depth';
  if (mode === 'lake-fraction') return (result.lakeFraction[sample]! * 100).toFixed(1) + '% lake fraction';
  if (mode === 'contributing-area') return (result.contributingAreaM2[sample]! / 1e6).toFixed(1) + ' km² contributing area';
  const field = scalarField(result, mode, orbitalPhase());
  if (field) return diagnosticLabel(mode) + ' ' + formatLegendNumber(mode,field.values[sample]!);
  return diagnosticLabel(mode);
}
'''
s = s.replace(marker, "\n" + helpers + marker, 1)

old = """const VIEW_PRESETS: Record<string, ViewPreset> = {
  'custom': { mode: 'physical-elevation', overlays: [] },
  'physical-world': { mode: 'physical-world', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'cryosphere'] },
  'hydrologic-atlas': { mode: 'physical-world', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'basin-divides'] },
  'seasonal-world': { mode: 'seasonal-realized-discharge', overlays: ['evolved-topography', 'coastline', 'final-lakes', 'winds'] },
  'geomorphic-processes': { mode: 'evolution-terrain-delta', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'tectonic-boundaries'] },
};"""
new = """const VIEW_PRESETS: Record<string, ViewPreset> = {
  'custom': { mode: 'physical-elevation', overlays: [] },
  'physical-world': { mode: 'physical-world', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'cryosphere'] },
  'tectonic-history': { mode: 'historical-current', overlays: ['coastline', 'tectonic-boundaries', 'geological-boundaries'] },
  'crust-lithology': { mode: 'bedrock-class', overlays: ['coastline', 'tectonic-boundaries'] },
  'climate': { mode: 'precipitation', overlays: ['coastline', 'winds'] },
  'hydrologic-atlas': { mode: 'physical-world', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'basin-divides'] },
  'seasonal-world': { mode: 'seasonal-realized-discharge', overlays: ['evolved-topography', 'coastline', 'final-lakes', 'winds'] },
  'geomorphic-processes': { mode: 'evolution-terrain-delta', overlays: ['evolved-topography', 'coastline', 'final-rivers', 'final-lakes', 'tectonic-boundaries'] },
};"""
if old not in s:
    raise SystemExit("preset block not found")
s = s.replace(old,new,1)
s = s.replace("  visualization.value = definition.mode;\n", "  selectDiagnostic(definition.mode);\n", 1)
s = s.replace("  updateOverlaySummary();\n  redraw(false);\n  updateAnimation();\n}\nfunction formatDuration",
              "  updateOverlaySummary();\n  refreshDiagnosticLegend();\n  redraw(false);\n  updateAnimation();\n}\nfunction formatDuration",1)

s = re.sub(r"(  const basin = current\.basinId\[sample\].*?;\n)",
           r"\1  const diagnosticValue = selectedDiagnosticSampleText(current, sample);\n", s, count=1)
s = re.sub(r"  cellInspector\.textContent = .*?;\n",
           "  cellInspector.textContent = 'Cell ' + sample.toLocaleString() + ' · ' + (degree === 5 ? 'pentagon' : 'hexagon') + ' · plate ' + current.plateIds[sample]!.toLocaleString() + ' · ' + surface + ' · ' + basin + ' · ' + diagnosticValue;\n",
           s, count=1)

s = s.replace("    showMetrics(loaded);\n    crashRecorder.record('lab', 'viewer-metrics-render-complete');",
              "    showMetrics(loaded);\n    refreshDiagnosticLegend();\n    crashRecorder.record('lab', 'viewer-metrics-render-complete');",1)

old = "visualization.addEventListener('change', () => { preset.value = 'custom'; styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 }; redraw(false); updateAnimation(); });"
new = """diagnosticCategory.addEventListener('change', () => {
  preset.value = 'custom';
  setDiagnosticCategory(diagnosticCategory.value);
  styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
  gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 };
  refreshDiagnosticLegend();
  selectedTile = null;
  inspectTile(null);
  redraw(false);
  updateAnimation();
});
visualization.addEventListener('change', () => {
  preset.value = 'custom';
  const categoryId = categoryForDiagnostic(visualization.value);
  if (categoryId) diagnosticCategory.value = categoryId;
  styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] };
  gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 };
  refreshDiagnosticLegend();
  if (selectedTile !== null) inspectTile(selectedTile);
  redraw(false);
  updateAnimation();
});"""
if old not in s:
    raise SystemExit("visualization listener not found")
s = s.replace(old,new,1)
s = s.replace("season.addEventListener('input', () => { updateSeasonLabel(); styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 }; redraw(false); });",
              "season.addEventListener('input', () => { updateSeasonLabel(); styleCache = { result: null, key: '', sampleBuckets: [], boundaryBuckets: [] }; gpuColorCache = { result: null, key: '', colors: new Uint8Array(0), alpha: 0.94 }; refreshDiagnosticLegend(); if (selectedTile !== null) inspectTile(selectedTile); redraw(false); });",1)

old = """updateSeasonLabel();
updateZoomLabel();
updateCameraControls();
updateOverlaySummary();
refreshCrashDebugSummary();
applyViewPreset(preset.value);
generationStage.textContent = 'Ready for canonical L8 generation';
generationStep.textContent = 'L5 coarse physical state → L8 final physical planet';
status.textContent = 'Ready. Generate the canonical L5 → L8 physical world when you want to allocate the full-resolution state.';"""
new = """coarseLevel.max = String(WORLDGEN_CLIMATE_COARSE_MAX_LEVEL);
coarseLevel.value = String(WORLDGEN_CLIMATE_COARSE_MAX_LEVEL);
fineLevel.max = String(WORLDGEN_CLIMATE_FINE_MAX_LEVEL);
fineLevel.value = String(WORLDGEN_CLIMATE_FINE_MAX_LEVEL);
updateSeasonLabel();
updateZoomLabel();
updateCameraControls();
updateOverlaySummary();
refreshCrashDebugSummary();
setDiagnosticCategory('world', 'physical-elevation');
applyViewPreset(preset.value);
refreshDiagnosticLegend();
generationStage.textContent = 'Ready for canonical maximum-fidelity generation';
generationStep.textContent = 'L' + WORLDGEN_CLIMATE_COARSE_MAX_LEVEL + ' coarse physical state → L' + WORLDGEN_CLIMATE_FINE_MAX_LEVEL + ' final physical planet';
status.textContent = 'Ready. Generate the canonical L' + WORLDGEN_CLIMATE_COARSE_MAX_LEVEL + ' → L' + WORLDGEN_CLIMATE_FINE_MAX_LEVEL + ' physical world when you want to allocate the full-resolution state.';"""
if old not in s:
    raise SystemExit("init block not found")
s = s.replace(old,new,1)
p.write_text(s)

Path("tests/worldgenDiagnosticsUx.test.ts").write_text('''import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WORLDGEN_CLIMATE_COARSE_MAX_LEVEL, WORLDGEN_CLIMATE_FINE_MAX_LEVEL } from '../dist/worldgen/protocol.js';

test('Planet Engine Lab defaults generation controls to accepted maximum fidelity', () => {
  assert.equal(WORLDGEN_CLIMATE_COARSE_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_CLIMATE_FINE_MAX_LEVEL, 8);
  const html = fs.readFileSync('index.html', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  assert.match(html, /id="worldgen-coarse-level"[^>]*max="6"[^>]*value="6"/);
  assert.match(html, /id="worldgen-level"[^>]*max="8"[^>]*value="8"/);
  assert.match(source, /coarseLevel\.value = String\(WORLDGEN_CLIMATE_COARSE_MAX_LEVEL\)/);
  assert.match(source, /fineLevel\.value = String\(WORLDGEN_CLIMATE_FINE_MAX_LEVEL\)/);
});

test('diagnostics are grouped by physical domain and carry an in-view legend', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenClimateLabStandalone.ts', 'utf8');
  const css = fs.readFileSync('styles/worldgenLab.css', 'utf8');
  assert.match(html, /id="worldgen-diagnostic-category"/);
  assert.match(html, /id="worldgen-diagnostic-legend"/);
  for (const category of ['world','tectonics-history','crust-lithosphere','lithology-substrate','topography-forcing','climate','hydrology','geomorphology','technical']) {
    assert.match(html, new RegExp('data-diagnostic-category="' + category + '"'));
  }
  assert.match(source, /function refreshDiagnosticLegend/);
  assert.match(source, /CATEGORICAL_LEGENDS/);
  assert.match(source, /selectedDiagnosticSampleText/);
  assert.match(css, /\.worldgen-diagnostic-legend/);
  assert.match(css, /\.worldgen-legend-ramp/);
  assert.match(css, /\.worldgen-legend-swatches/);
});

test('every selectable diagnostic belongs to exactly one category', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  const a = html.indexOf('<select id="worldgen-visualization">');
  const b = html.indexOf('</select>', a);
  assert.ok(a >= 0 && b > a);
  const fragment = html.slice(a, b);
  const values = Array.from(fragment.matchAll(/<option value="([^"]+)"/g), match => match[1]);
  assert.ok(values.length > 70);
  assert.equal(new Set(values).size, values.length);
  const groups = Array.from(fragment.matchAll(/<optgroup[^>]*data-diagnostic-category="([^"]+)"[^>]*>([\s\S]*?)<\/optgroup>/g));
  const grouped = groups.flatMap(match => Array.from(match[2].matchAll(/<option value="([^"]+)"/g), option => option[1]));
  assert.deepEqual(grouped.sort(), values.sort());
});
''')
