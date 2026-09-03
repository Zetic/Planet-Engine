from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text()


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content)


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


# 1) Expose the inherited WG-3.75 diagnostic fields from the already-generated WG-4 object.
bridge_path = "rust/interlink-worldgen-wasm/src/topography_bridge.rs"
bridge = read(bridge_path)
bridge = replace_once(
    bridge,
    """    pub fn crust_kind(&self) -> Vec<u8> {\n        self.inherited.crust_kind.clone()\n    }\n    pub fn boundary_samples(&self) -> Vec<u32> {\n        self.boundaries.flattened_samples()\n    }\n    pub fn geological_boundary_regimes(&self) -> Vec<u8> {\n        self.boundaries.geological_regimes()\n    }\n""",
    """    pub fn crust_kind(&self) -> Vec<u8> {\n        self.inherited.crust_kind.clone()\n    }\n    pub fn nearest_coarse_source(&self) -> Vec<u32> {\n        self.inherited.map.nearest_coarse_source.clone()\n    }\n    pub fn inherited_sample_mask(&self) -> Vec<u8> {\n        self.inherited.map.inherited_sample_mask.clone()\n    }\n    pub fn crust_age_myr(&self) -> Vec<f32> {\n        self.inherited.crust_age_myr.clone()\n    }\n    pub fn crust_thickness_km(&self) -> Vec<f32> {\n        self.inherited.crust_thickness_km.clone()\n    }\n    pub fn orogenic_history(&self) -> Vec<f32> {\n        self.inherited.orogenic_history.clone()\n    }\n    pub fn ridge_history(&self) -> Vec<f32> {\n        self.inherited.ridge_history.clone()\n    }\n    pub fn trench_history(&self) -> Vec<f32> {\n        self.inherited.trench_history.clone()\n    }\n    pub fn strength_index(&self) -> Vec<f32> {\n        self.inherited.strength_index.clone()\n    }\n    pub fn weakness_index(&self) -> Vec<f32> {\n        self.inherited.weakness_index.clone()\n    }\n    pub fn mantle_dynamic_support_index(&self) -> Vec<f32> {\n        self.inherited.mantle_dynamic_support_index.clone()\n    }\n    pub fn structural_zone_kind(&self) -> Vec<u8> {\n        self.inherited.structural_zone_kind.clone()\n    }\n    pub fn fragmentation_propensity(&self) -> Vec<f32> {\n        self.inherited.fragmentation_propensity.clone()\n    }\n    pub fn kinematic_domain_ids(&self) -> Vec<u16> {\n        self.inherited.kinematic_domain_ids.clone()\n    }\n    pub fn boundary_samples(&self) -> Vec<u32> {\n        self.boundaries.flattened_samples()\n    }\n    pub fn boundary_kinds(&self) -> Vec<u8> {\n        self.boundaries.tectonic_kinds()\n    }\n    pub fn geological_boundary_regimes(&self) -> Vec<u8> {\n        self.boundaries.geological_regimes()\n    }\n    pub fn boundary_coarse_source_indices(&self) -> Vec<u32> {\n        self.boundaries.coarse_boundary_indices()\n    }\n""",
    "topography bridge inherited fields",
)
write(bridge_path, bridge)

# 2) Extend the browser result contract with only the upstream fields needed by the restored views.
protocol_path = "src/worldgen/protocol.ts"
protocol = read(protocol_path)
protocol = replace_once(
    protocol,
    """  plateIds: Uint16Array;\n  crustKind: Uint8Array;\n  boundarySamples: Uint32Array;\n  geologicalBoundaryRegimes: Uint8Array;\n""",
    """  plateIds: Uint16Array;\n  crustKind: Uint8Array;\n  nearestCoarseSource: Uint32Array;\n  inheritedSampleMask: Uint8Array;\n  crustAgeMyr: Float32Array;\n  crustThicknessKm: Float32Array;\n  orogenicHistory: Float32Array;\n  ridgeHistory: Float32Array;\n  trenchHistory: Float32Array;\n  strengthIndex: Float32Array;\n  weaknessIndex: Float32Array;\n  mantleDynamicSupportIndex: Float32Array;\n  structuralZoneKind: Uint8Array;\n  fragmentationPropensity: Float32Array;\n  kinematicDomainIds: Uint16Array;\n  boundarySamples: Uint32Array;\n  boundaryKinds: Uint8Array;\n  geologicalBoundaryRegimes: Uint8Array;\n  boundaryCoarseSourceIndices: Uint32Array;\n""",
    "topography protocol diagnostic fields",
)
write(protocol_path, protocol)

# 3) Forward those fields through the Worker without running the inheritance stage again.
worker_path = "src/worldgen/worldgenWorker.ts"
worker = read(worker_path)
worker = replace_once(
    worker,
    """  radius_m(): number; surface_gravity_m_s2(): number; surface_water_mass_kg(): number; equivalent_global_water_depth_m(): number; ocean_water_density_kg_per_m3(): number; isostatic_mantle_density_kg_per_m3(): number;\n  positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array; plate_ids(): Uint16Array; crust_kind(): Uint8Array; boundary_samples(): Uint32Array; geological_boundary_regimes(): Uint8Array;\n""",
    """  radius_m(): number; surface_gravity_m_s2(): number; surface_water_mass_kg(): number; equivalent_global_water_depth_m(): number; ocean_water_density_kg_per_m3(): number; isostatic_mantle_density_kg_per_m3(): number; internal_heat_flux_w_per_m2(): number; mantle_thermal_expansivity_per_k(): number;\n  positions(): Float64Array; faces(): Uint32Array; neighbor_offsets(): Uint32Array; neighbors(): Uint32Array; plate_ids(): Uint16Array; crust_kind(): Uint8Array; nearest_coarse_source(): Uint32Array; inherited_sample_mask(): Uint8Array; crust_age_myr(): Float32Array; crust_thickness_km(): Float32Array; orogenic_history(): Float32Array; ridge_history(): Float32Array; trench_history(): Float32Array; strength_index(): Float32Array; weakness_index(): Float32Array; mantle_dynamic_support_index(): Float32Array; structural_zone_kind(): Uint8Array; fragmentation_propensity(): Float32Array; kinematic_domain_ids(): Uint16Array; boundary_samples(): Uint32Array; boundary_kinds(): Uint8Array; geological_boundary_regimes(): Uint8Array; boundary_coarse_source_indices(): Uint32Array;\n""",
    "worker WasmTopography surface",
)
worker = replace_once(
    worker,
    """    const plateIds = output.plate_ids(); const crustKind = output.crust_kind(); const boundarySamples = output.boundary_samples(); const geologicalBoundaryRegimes = output.geological_boundary_regimes();\n""",
    """    const plateIds = output.plate_ids(); const crustKind = output.crust_kind(); const nearestCoarseSource = output.nearest_coarse_source(); const inheritedSampleMask = output.inherited_sample_mask(); const crustAgeMyr = output.crust_age_myr(); const crustThicknessKm = output.crust_thickness_km();\n    const orogenicHistory = output.orogenic_history(); const ridgeHistory = output.ridge_history(); const trenchHistory = output.trench_history(); const strengthIndex = output.strength_index(); const weaknessIndex = output.weakness_index(); const mantleDynamicSupportIndex = output.mantle_dynamic_support_index(); const structuralZoneKind = output.structural_zone_kind(); const fragmentationPropensity = output.fragmentation_propensity(); const kinematicDomainIds = output.kinematic_domain_ids();\n    const boundarySamples = output.boundary_samples(); const boundaryKinds = output.boundary_kinds(); const geologicalBoundaryRegimes = output.geological_boundary_regimes(); const boundaryCoarseSourceIndices = output.boundary_coarse_source_indices();\n""",
    "worker topography field reads",
)
worker = replace_once(
    worker,
    """      parameters: { radiusM: output.radius_m(), surfaceGravityMS2: output.surface_gravity_m_s2(), surfaceWaterMassKg: output.surface_water_mass_kg(), equivalentGlobalWaterDepthM: output.equivalent_global_water_depth_m(), oceanWaterDensityKgPerM3: output.ocean_water_density_kg_per_m3(), isostaticMantleDensityKgPerM3: output.isostatic_mantle_density_kg_per_m3(), internalHeatFluxWPerM2: 0, mantleThermalExpansivityPerK: 0 },\n      positions, faces, neighborOffsets, neighbors, plateIds, crustKind, boundarySamples, geologicalBoundaryRegimes,\n""",
    """      parameters: { radiusM: output.radius_m(), surfaceGravityMS2: output.surface_gravity_m_s2(), surfaceWaterMassKg: output.surface_water_mass_kg(), equivalentGlobalWaterDepthM: output.equivalent_global_water_depth_m(), oceanWaterDensityKgPerM3: output.ocean_water_density_kg_per_m3(), isostaticMantleDensityKgPerM3: output.isostatic_mantle_density_kg_per_m3(), internalHeatFluxWPerM2: output.internal_heat_flux_w_per_m2(), mantleThermalExpansivityPerK: output.mantle_thermal_expansivity_per_k() },\n      positions, faces, neighborOffsets, neighbors, plateIds, crustKind, nearestCoarseSource, inheritedSampleMask, crustAgeMyr, crustThicknessKm, orogenicHistory, ridgeHistory, trenchHistory, strengthIndex, weaknessIndex, mantleDynamicSupportIndex, structuralZoneKind, fragmentationPropensity, kinematicDomainIds, boundarySamples, boundaryKinds, geologicalBoundaryRegimes, boundaryCoarseSourceIndices,\n""",
    "worker topography result",
)
worker = replace_once(
    worker,
    """      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-topography', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.crustKind.buffer, result.boundarySamples.buffer, result.geologicalBoundaryRegimes.buffer, result.isostaticElevationM.buffer, result.thermalElevationM.buffer, result.orogenicElevationM.buffer, result.ridgeElevationM.buffer, result.riftBasinElevationM.buffer, result.trenchElevationM.buffer, result.arcElevationM.buffer, result.mantleDynamicElevationM.buffer, result.solidElevationM.buffer, result.elevationAboveSeaLevelM.buffer, result.waterDepthM.buffer, result.submergedMask.buffer]); return;\n""",
    """      workerScope.postMessage({ protocolVersion: WORLDGEN_PROTOCOL_VERSION, requestId: command.requestId, type: 'generated-topography', payload: result }, [result.positions.buffer, result.faces.buffer, result.neighborOffsets.buffer, result.neighbors.buffer, result.plateIds.buffer, result.crustKind.buffer, result.nearestCoarseSource.buffer, result.inheritedSampleMask.buffer, result.crustAgeMyr.buffer, result.crustThicknessKm.buffer, result.orogenicHistory.buffer, result.ridgeHistory.buffer, result.trenchHistory.buffer, result.strengthIndex.buffer, result.weaknessIndex.buffer, result.mantleDynamicSupportIndex.buffer, result.structuralZoneKind.buffer, result.fragmentationPropensity.buffer, result.kinematicDomainIds.buffer, result.boundarySamples.buffer, result.boundaryKinds.buffer, result.geologicalBoundaryRegimes.buffer, result.boundaryCoarseSourceIndices.buffer, result.isostaticElevationM.buffer, result.thermalElevationM.buffer, result.orogenicElevationM.buffer, result.ridgeElevationM.buffer, result.riftBasinElevationM.buffer, result.trenchElevationM.buffer, result.arcElevationM.buffer, result.mantleDynamicElevationM.buffer, result.solidElevationM.buffer, result.elevationAboveSeaLevelM.buffer, result.waterDepthM.buffer, result.submergedMask.buffer]); return;\n""",
    "worker topography transfer list",
)
write(worker_path, worker)

# 4) Make the root Lab cumulative through WG-4 and use planet-level action language.
html = r'''<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Planet Engine · WG-4 Lab</title>
  <link rel="stylesheet" href="styles/base.css">
  <link rel="stylesheet" href="styles/worldgenLab.css">
</head>
<body class="worldgen-lab-body">
  <main class="worldgen-lab">
    <header class="worldgen-lab-header">
      <div>
        <p class="worldgen-lab-kicker">PLANET ENGINE · THROUGH WG-4</p>
        <h1>PLANET ENGINE LAB</h1>
        <p>Generate one deterministic physical planet, then inspect its topology, inherited tectonic and geological state, lithosphere, and WG-4 pre-erosional topography from the same result.</p>
      </div>
    </header>

    <section class="worldgen-lab-controls" aria-label="Planet generation controls">
      <label>Seed <input id="worldgen-seed" type="text" value="interlink-wg4"></label>
      <label>Coarse physical level <input id="worldgen-coarse-level" type="number" min="0" max="6" value="4"></label>
      <label>Fine planet level <input id="worldgen-level" type="number" min="0" max="7" value="6"></label>
      <label>Plate count <input id="worldgen-plates" type="number" min="4" max="48" value="16"></label>
      <label>Projection
        <select id="worldgen-projection">
          <option value="globe">Orthographic globe</option>
          <option value="map">Equirectangular map</option>
        </select>
      </label>
      <label>Diagnostic
        <select id="worldgen-visualization">
          <optgroup label="Physical surface (WG-4)">
            <option value="relative-elevation" selected>Elevation above sea level</option>
            <option value="solid-elevation">Solid elevation / datum</option>
            <option value="land-water">Land / water</option>
            <option value="water-depth">Bathymetry / water depth</option>
          </optgroup>
          <optgroup label="Topographic forcing (WG-4)">
            <option value="isostatic">Isostatic support</option>
            <option value="thermal">Oceanic thermal subsidence</option>
            <option value="orogenic-relief">Orogenic / collision uplift</option>
            <option value="ridge-relief">Ridge relief</option>
            <option value="rift-basin">Rift / basin subsidence</option>
            <option value="trench-relief">Trench relief</option>
            <option value="arc-relief">Volcanic arc relief</option>
            <option value="mantle-relief">Mantle dynamic support (relief)</option>
          </optgroup>
          <optgroup label="Inheritance (WG-3.75)">
            <option value="inherited-mask">Inherited coarse samples</option>
            <option value="provenance">Nearest coarse provenance</option>
            <option value="boundary-provenance">Boundary provenance</option>
          </optgroup>
          <optgroup label="Tectonics">
            <option value="plates">Macro plate ownership</option>
            <option value="kinematic-domains">Refined kinematic domains</option>
            <option value="tectonic-boundaries">Fine tectonic boundaries</option>
            <option value="geological-boundaries">Fine geological regimes</option>
          </optgroup>
          <optgroup label="Crust / history">
            <option value="crust-type">Crust type</option>
            <option value="crust-age">Crust age</option>
            <option value="crust-thickness">Crust thickness</option>
            <option value="orogeny-history">Orogenic history</option>
            <option value="ridge-history">Ridge history</option>
            <option value="trench-history">Trench history</option>
          </optgroup>
          <optgroup label="Lithosphere">
            <option value="strength">Lithospheric strength</option>
            <option value="weakness">Lithospheric weakness</option>
            <option value="dynamic-support">Mantle dynamic support (index)</option>
            <option value="structural-zones">Structural zone type</option>
            <option value="fragmentation">Fragmentation propensity</option>
          </optgroup>
          <optgroup label="Topology">
            <option value="mesh">Fine topology mesh</option>
          </optgroup>
        </select>
      </label>
      <button id="worldgen-generate" type="button">Generate Planet</button>
    </section>

    <p id="worldgen-status" class="worldgen-lab-status">Initializing Planet Engine Worker…</p>

    <section class="worldgen-lab-grid">
      <div class="worldgen-lab-viewport">
        <canvas id="worldgen-field" aria-label="Planet Engine physical diagnostics"></canvas>
      </div>
      <aside>
        <h2>Physical diagnostics</h2>
        <div id="worldgen-metrics" class="worldgen-lab-metrics"></div>
        <div class="worldgen-lab-note">
          <strong>Current physical frontier: WG-4</strong>
          <p>One generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, and initial-topography pipeline. Diagnostic modes inspect that same generated planet rather than regenerating earlier stages.</p>
          <p>WG-4 adds crustal isostatic support, oceanic age subsidence, collision/ridge/rift/subduction morphology, inherited basin tendency, broad mantle support, lithospheric mechanical filtering, and a water-volume-derived sea-level solution.</p>
          <p>This remains pre-erosional physical topography. Climate, drainage, river incision, sediment transport, glaciation, detailed lithology, resource deposits, Regions, Features, and gameplay integration remain downstream.</p>
        </div>
      </aside>
    </section>
  </main>
  <script type="module" src="dist/worldgen/diagnostics/worldgenTopographyLabStandalone.js"></script>
</body>
</html>
'''
write("index.html", html)
write("worldgen-lab.html", html)

# 5) Replace the WG-4 Lab controller with a cumulative, single-generation renderer.
lab_source = r'''import { createWorldgenClient } from '../worldgenClient.js';
import {
  WORLDGEN_BOUNDARY_CONVERGENT,
  WORLDGEN_BOUNDARY_DIVERGENT,
  WORLDGEN_BOUNDARY_TRANSFORM,
  WORLDGEN_CRUST_CONTINENTAL,
  WORLDGEN_CRUST_OCEANIC,
  WORLDGEN_CRUST_TRANSITIONAL,
  WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION,
  WORLDGEN_GEOLOGY_CONTINENTAL_RIFT,
  WORLDGEN_GEOLOGY_OCEANIC_RIDGE,
  WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION,
  WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION,
  WORLDGEN_GEOLOGY_TRANSFORM,
  WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE,
  WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN,
  WORLDGEN_STRUCTURE_NONE,
  WORLDGEN_STRUCTURE_RIFT,
  WORLDGEN_STRUCTURE_SUTURE,
  WORLDGEN_STRUCTURE_TRANSFORM,
  type WorldgenTopographyResult,
} from '../protocol.js';

type ScalarField = { values: Float32Array; minimum: number; maximum: number; lowHue: number; highHue: number };
type ProjectionBuffers = { x: Float32Array; y: Float32Array; visible: Uint8Array };
type DrawBucket = { color: string; indices: Uint32Array };
type StyleCache = { result: WorldgenTopographyResult | null; mode: string; sampleBuckets: DrawBucket[]; boundaryBuckets: DrawBucket[] };

const PALETTE_STEPS = 256;

function element<T extends HTMLElement>(id: string): T {
  const target = document.getElementById(id);
  if (!target) throw new Error(`Planet Engine Lab is missing #${id}.`);
  return target as T;
}
function metric(container: HTMLElement, label: string, value: string): void {
  const item = document.createElement('div');
  const key = document.createElement('strong');
  const detail = document.createElement('span');
  key.textContent = label;
  detail.textContent = value;
  item.append(key, detail);
  container.appendChild(item);
}
function plateColor(id: number): string { return `hsl(${(id * 137.507764 + 18) % 360} 60% 55%)`; }
function provenanceColor(source: number): string { return `hsl(${(source * 137.507764 + 42) % 360} 58% 54%)`; }
function crustColor(kind: number): string {
  if (kind === WORLDGEN_CRUST_CONTINENTAL) return '#b79a72';
  if (kind === WORLDGEN_CRUST_TRANSITIONAL) return '#9aab87';
  if (kind === WORLDGEN_CRUST_OCEANIC) return '#477aa3';
  return '#d7e2ef';
}
function structuralColor(kind: number): string {
  if (kind === WORLDGEN_STRUCTURE_SUTURE) return '#ff7466';
  if (kind === WORLDGEN_STRUCTURE_RIFT) return '#ffb45d';
  if (kind === WORLDGEN_STRUCTURE_TRANSFORM) return '#c690ff';
  if (kind === WORLDGEN_STRUCTURE_CONTINENT_MARGIN) return '#65d7ac';
  if (kind === WORLDGEN_STRUCTURE_NONE) return '#425362';
  return '#d7e2ef';
}
function tectonicBoundaryColor(kind: number): string {
  if (kind === WORLDGEN_BOUNDARY_CONVERGENT) return '#ff7272';
  if (kind === WORLDGEN_BOUNDARY_DIVERGENT) return '#64d7ff';
  if (kind === WORLDGEN_BOUNDARY_TRANSFORM) return '#ffd36a';
  return '#d7e2ef';
}
function geologicalBoundaryColor(regime: number): string {
  if (regime === WORLDGEN_GEOLOGY_OCEANIC_SUBDUCTION) return '#5a8fff';
  if (regime === WORLDGEN_GEOLOGY_OCEAN_CONTINENT_SUBDUCTION) return '#8a70ff';
  if (regime === WORLDGEN_GEOLOGY_CONTINENTAL_COLLISION) return '#ff6969';
  if (regime === WORLDGEN_GEOLOGY_OCEANIC_RIDGE) return '#4ee8df';
  if (regime === WORLDGEN_GEOLOGY_CONTINENTAL_RIFT) return '#ffb65c';
  if (regime === WORLDGEN_GEOLOGY_TRANSITIONAL_DIVERGENCE) return '#e8cf66';
  if (regime === WORLDGEN_GEOLOGY_TRANSFORM) return '#d59cff';
  return '#d7e2ef';
}
function scalarField(result: WorldgenTopographyResult, mode: string): ScalarField | null {
  switch (mode) {
    case 'solid-elevation': return { values: result.solidElevationM, minimum: -12_000, maximum: 8_000, lowHue: 225, highHue: 25 };
    case 'relative-elevation': return { values: result.elevationAboveSeaLevelM, minimum: -10_000, maximum: 6_000, lowHue: 220, highHue: 35 };
    case 'water-depth': return { values: result.waterDepthM, minimum: 0, maximum: 10_000, lowHue: 195, highHue: 245 };
    case 'isostatic': return { values: result.isostaticElevationM, minimum: 0, maximum: 10_000, lowHue: 210, highHue: 25 };
    case 'thermal': return { values: result.thermalElevationM, minimum: -5_000, maximum: 0, lowHue: 260, highHue: 185 };
    case 'orogenic-relief': return { values: result.orogenicElevationM, minimum: 0, maximum: 6_000, lowHue: 55, highHue: 350 };
    case 'ridge-relief': return { values: result.ridgeElevationM, minimum: 0, maximum: 3_000, lowHue: 220, highHue: 165 };
    case 'rift-basin': return { values: result.riftBasinElevationM, minimum: -3_500, maximum: 0, lowHue: 250, highHue: 35 };
    case 'trench-relief': return { values: result.trenchElevationM, minimum: -7_000, maximum: 0, lowHue: 285, highHue: 210 };
    case 'arc-relief': return { values: result.arcElevationM, minimum: 0, maximum: 3_000, lowHue: 50, highHue: 5 };
    case 'mantle-relief': return { values: result.mantleDynamicElevationM, minimum: -1_200, maximum: 1_200, lowHue: 245, highHue: 25 };
    case 'crust-age': return { values: result.crustAgeMyr, minimum: 0, maximum: 3500, lowHue: 205, highHue: 24 };
    case 'crust-thickness': return { values: result.crustThicknessKm, minimum: 5, maximum: 56, lowHue: 205, highHue: 350 };
    case 'orogeny-history': return { values: result.orogenicHistory, minimum: 0, maximum: 1, lowHue: 50, highHue: 350 };
    case 'ridge-history': return { values: result.ridgeHistory, minimum: 0, maximum: 1, lowHue: 225, highHue: 170 };
    case 'trench-history': return { values: result.trenchHistory, minimum: 0, maximum: 1, lowHue: 200, highHue: 260 };
    case 'strength': return { values: result.strengthIndex, minimum: 0, maximum: 1, lowHue: 0, highHue: 135 };
    case 'weakness': return { values: result.weaknessIndex, minimum: 0, maximum: 1, lowHue: 205, highHue: 15 };
    case 'dynamic-support': return { values: result.mantleDynamicSupportIndex, minimum: -1, maximum: 1, lowHue: 245, highHue: 25 };
    case 'fragmentation': return { values: result.fragmentationPropensity, minimum: 0, maximum: 1, lowHue: 210, highHue: 0 };
    default: return null;
  }
}
function scalarColor(value: number, field: ScalarField): string {
  const t = Math.max(0, Math.min(1, (value - field.minimum) / Math.max(1e-12, field.maximum - field.minimum)));
  const quantized = Math.round(t * (PALETTE_STEPS - 1)) / (PALETTE_STEPS - 1);
  const hue = field.lowHue + (field.highHue - field.lowHue) * quantized;
  return `hsl(${hue} 68% ${38 + quantized * 22}%)`;
}
function bucketize(count: number, colorAt: (index: number) => string): DrawBucket[] {
  const buckets = new Map<string, number[]>();
  for (let index = 0; index < count; index += 1) {
    const color = colorAt(index);
    const entries = buckets.get(color);
    if (entries) entries.push(index); else buckets.set(color, [index]);
  }
  return Array.from(buckets, ([color, indices]) => ({ color, indices: Uint32Array.from(indices) }));
}
function sampleColor(result: WorldgenTopographyResult, mode: string, sample: number, field: ScalarField | null): string {
  if (mode === 'land-water') return result.submergedMask[sample] ? '#214d7a' : '#a99b72';
  if (mode === 'plates' || mode === 'tectonic-boundaries' || mode === 'geological-boundaries' || mode === 'boundary-provenance') return plateColor(result.plateIds[sample]!);
  if (mode === 'kinematic-domains') return plateColor(result.kinematicDomainIds[sample]!);
  if (mode === 'crust-type') return crustColor(result.crustKind[sample]!);
  if (mode === 'structural-zones') return structuralColor(result.structuralZoneKind[sample]!);
  if (mode === 'provenance') return provenanceColor(result.nearestCoarseSource[sample]!);
  if (mode === 'inherited-mask') return result.inheritedSampleMask[sample] ? '#f4e27a' : '#5794c8';
  if (field) return scalarColor(field.values[sample]!, field);
  return '#8297aa';
}
function buildStyleCache(result: WorldgenTopographyResult, mode: string): StyleCache {
  const field = scalarField(result, mode);
  const sampleBuckets = mode === 'mesh' ? [] : bucketize(result.metrics.fineSampleCount, sample => sampleColor(result, mode, sample, field));
  let boundaryBuckets: DrawBucket[] = [];
  if (mode === 'tectonic-boundaries') boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => tectonicBoundaryColor(result.boundaryKinds[boundary]!));
  else if (mode === 'geological-boundaries') boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => geologicalBoundaryColor(result.geologicalBoundaryRegimes[boundary]!));
  else if (mode === 'boundary-provenance') boundaryBuckets = bucketize(result.metrics.fineBoundaryEdgeCount, boundary => provenanceColor(result.boundaryCoarseSourceIndices[boundary]!));
  return { result, mode, sampleBuckets, boundaryBuckets };
}

function projectSamples(result: WorldgenTopographyResult, projection: string, yaw: number, pitch: number, width: number, height: number, buffers: ProjectionBuffers): void {
  const count = result.metrics.fineSampleCount;
  const positions = result.positions;
  if (projection === 'map') {
    for (let sample = 0; sample < count; sample += 1) {
      const offset = sample * 3;
      const px = positions[offset]!;
      const py = positions[offset + 1]!;
      const pz = positions[offset + 2]!;
      buffers.x[sample] = (Math.atan2(py, px) + Math.PI) / (2 * Math.PI) * width;
      buffers.y[sample] = (Math.PI / 2 - Math.asin(Math.max(-1, Math.min(1, pz)))) / Math.PI * height;
      buffers.visible[sample] = 1;
    }
    return;
  }
  const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch);
  const radius = Math.min(width, height) * 0.44;
  for (let sample = 0; sample < count; sample += 1) {
    const offset = sample * 3;
    const px = positions[offset]!;
    const py = positions[offset + 1]!;
    const pz = positions[offset + 2]!;
    const x1 = cy * px - sy * py;
    const y1 = sy * px + cy * py;
    const rotatedX = cp * x1 + sp * pz;
    const rotatedZ = -sp * x1 + cp * pz;
    buffers.x[sample] = width / 2 + y1 * radius;
    buffers.y[sample] = height / 2 - rotatedZ * radius;
    buffers.visible[sample] = rotatedX >= 0 ? 1 : 0;
  }
}

let styleCache: StyleCache = { result: null, mode: '', sampleBuckets: [], boundaryBuckets: [] };
function renderPlanet(canvas: HTMLCanvasElement, result: WorldgenTopographyResult, projection: string, mode: string, yaw: number, pitch: number, buffers: ProjectionBuffers, interactive: boolean): void {
  const width = 1100;
  const height = projection === 'map' ? 550 : 760;
  if (canvas.width !== width) canvas.width = width;
  if (canvas.height !== height) canvas.height = height;
  const context = canvas.getContext('2d');
  if (!context) throw new Error('Planet Engine Lab could not acquire a 2D canvas context.');
  context.fillStyle = '#08101a';
  context.fillRect(0, 0, width, height);
  projectSamples(result, projection, yaw, pitch, width, height, buffers);

  if (projection === 'globe') {
    context.beginPath();
    context.arc(width / 2, height / 2, Math.min(width, height) * 0.44, 0, Math.PI * 2);
    context.strokeStyle = '#5d7890';
    context.lineWidth = 1;
    context.stroke();
  }

  if (mode === 'mesh') {
    context.beginPath();
    context.strokeStyle = '#35536d';
    context.lineWidth = 0.65;
    for (let sample = 0; sample < result.metrics.fineSampleCount; sample += 1) {
      if (!buffers.visible[sample]) continue;
      const ax = buffers.x[sample]!;
      const ay = buffers.y[sample]!;
      for (let cursor = result.neighborOffsets[sample]!; cursor < result.neighborOffsets[sample + 1]!; cursor += 1) {
        const neighbor = result.neighbors[cursor]!;
        if (neighbor <= sample || !buffers.visible[neighbor]) continue;
        const bx = buffers.x[neighbor]!;
        if (projection === 'map' && Math.abs(ax - bx) > width / 2) continue;
        context.moveTo(ax, ay);
        context.lineTo(bx, buffers.y[neighbor]!);
      }
    }
    context.stroke();
    return;
  }

  if (styleCache.result !== result || styleCache.mode !== mode) styleCache = buildStyleCache(result, mode);
  const count = result.metrics.fineSampleCount;
  const pointRadius = count > 100_000 ? 0.8 : count > 30_000 ? 1.15 : count > 5_000 ? 2 : 3;
  const fastPoints = interactive && count > 20_000;
  const boundaryMode = styleCache.boundaryBuckets.length > 0;
  context.globalAlpha = boundaryMode ? 0.28 : 0.94;
  for (const bucket of styleCache.sampleBuckets) {
    context.fillStyle = bucket.color;
    if (!fastPoints) context.beginPath();
    for (let cursor = 0; cursor < bucket.indices.length; cursor += 1) {
      const sample = bucket.indices[cursor]!;
      if (!buffers.visible[sample]) continue;
      const x = buffers.x[sample]!;
      const y = buffers.y[sample]!;
      if (fastPoints) context.fillRect(x - 0.75, y - 0.75, 1.5, 1.5);
      else { context.moveTo(x + pointRadius, y); context.arc(x, y, pointRadius, 0, Math.PI * 2); }
    }
    if (!fastPoints) context.fill();
  }
  context.globalAlpha = 1;

  if (boundaryMode) {
    context.lineCap = 'round';
    context.lineWidth = mode === 'boundary-provenance' ? 1.4 : 2.0;
    for (const bucket of styleCache.boundaryBuckets) {
      context.strokeStyle = bucket.color;
      context.beginPath();
      for (let cursor = 0; cursor < bucket.indices.length; cursor += 1) {
        const boundary = bucket.indices[cursor]!;
        const a = result.boundarySamples[boundary * 2]!;
        const b = result.boundarySamples[boundary * 2 + 1]!;
        if (!buffers.visible[a] || !buffers.visible[b]) continue;
        const ax = buffers.x[a]!;
        const bx = buffers.x[b]!;
        if (projection === 'map' && Math.abs(ax - bx) > width / 2) continue;
        context.moveTo(ax, buffers.y[a]!);
        context.lineTo(bx, buffers.y[b]!);
      }
      context.stroke();
    }
  }
}

const seed = element<HTMLInputElement>('worldgen-seed');
const coarseLevel = element<HTMLInputElement>('worldgen-coarse-level');
const fineLevel = element<HTMLInputElement>('worldgen-level');
const plates = element<HTMLInputElement>('worldgen-plates');
const projection = element<HTMLSelectElement>('worldgen-projection');
const visualization = element<HTMLSelectElement>('worldgen-visualization');
const generate = element<HTMLButtonElement>('worldgen-generate');
const status = element<HTMLElement>('worldgen-status');
const metrics = element<HTMLElement>('worldgen-metrics');
const canvas = element<HTMLCanvasElement>('worldgen-field');
const client = createWorldgenClient();
let current: WorldgenTopographyResult | null = null;
let buffers: ProjectionBuffers | null = null;
let yaw = -0.65;
let pitch = 0.25;
let drag: { x: number; y: number; yaw: number; pitch: number } | null = null;
let frameRequest = 0;

function redraw(interactive = false): void {
  if (!current || !buffers) return;
  renderPlanet(canvas, current, projection.value, visualization.value, yaw, pitch, buffers, interactive);
}
function scheduleRedraw(interactive: boolean): void {
  if (frameRequest) return;
  frameRequest = requestAnimationFrame(() => { frameRequest = 0; redraw(interactive && drag !== null); });
}
function showMetrics(result: WorldgenTopographyResult): void {
  metrics.replaceChildren();
  metric(metrics, 'Engine / stage', `v${result.engineVersion} · ${result.stage.id}@${result.stage.version}`);
  metric(metrics, 'Resolution', `L${result.coarseLevel} → L${result.fineLevel}`);
  metric(metrics, 'Samples', result.metrics.fineSampleCount.toLocaleString());
  metric(metrics, 'Topography hash', result.metrics.topographyHash);
  metric(metrics, 'Elevation', `${result.metrics.minimumSolidElevationM.toFixed(0)} m → ${result.metrics.maximumSolidElevationM.toFixed(0)} m`);
  metric(metrics, 'Hypsometry', `P05 ${result.metrics.p05SolidElevationM.toFixed(0)} · P50 ${result.metrics.medianSolidElevationM.toFixed(0)} · P95 ${result.metrics.p95SolidElevationM.toFixed(0)} m`);
  metric(metrics, 'Sea level', result.metrics.hasSeaLevel ? `${result.metrics.seaLevelM.toFixed(1)} m` : 'dry profile');
  metric(metrics, 'Land / ocean', `${(result.metrics.landAreaFraction * 100).toFixed(1)}% / ${(result.metrics.oceanAreaFraction * 100).toFixed(1)}%`);
  metric(metrics, 'Mean land / water', `${result.metrics.meanLandElevationM.toFixed(0)} m / ${result.metrics.meanWaterDepthM.toFixed(0)} m`);
  metric(metrics, 'Deepest water', `${result.metrics.maximumWaterDepthM.toFixed(0)} m`);
  metric(metrics, 'Water volume error', result.metrics.waterVolumeRelativeError.toExponential(2));
  metric(metrics, 'Planet thermal', `${result.parameters.internalHeatFluxWPerM2.toFixed(3)} W/m² · α ${result.parameters.mantleThermalExpansivityPerK.toExponential(2)} K⁻¹`);
  metric(metrics, 'Safety clamps', result.metrics.clampedSampleCount.toLocaleString());
  metric(metrics, 'Upstream', `I ${result.metrics.inheritanceHash} · B ${result.metrics.boundaryHash}`);
  metric(metrics, 'Duration', `${result.stage.durationMs.toFixed(1)} ms`);
}
async function generatePlanet(): Promise<void> {
  generate.disabled = true;
  status.textContent = 'Generating one physical planet through WG-4 in Rust/WASM…';
  try {
    const loaded = await client.generateTopography({ seed: seed.value, coarseLevel: Number(coarseLevel.value), fineLevel: Number(fineLevel.value), plateCount: Number(plates.value) });
    current = loaded;
    buffers = { x: new Float32Array(loaded.metrics.fineSampleCount), y: new Float32Array(loaded.metrics.fineSampleCount), visible: new Uint8Array(loaded.metrics.fineSampleCount) };
    styleCache = { result: null, mode: '', sampleBuckets: [], boundaryBuckets: [] };
    showMetrics(loaded);
    redraw(false);
    status.textContent = `Planet ready through WG-4: ${loaded.metrics.fineSampleCount.toLocaleString()} samples, ${(loaded.metrics.landAreaFraction * 100).toFixed(1)}% land, ${(loaded.metrics.oceanAreaFraction * 100).toFixed(1)}% ocean.`;
  } catch (error) {
    status.textContent = error instanceof Error ? error.message : String(error);
  } finally {
    generate.disabled = false;
  }
}

generate.addEventListener('click', () => void generatePlanet());
projection.addEventListener('change', () => redraw(false));
visualization.addEventListener('change', () => redraw(false));
canvas.addEventListener('pointerdown', event => {
  if (projection.value !== 'globe') return;
  drag = { x: event.clientX, y: event.clientY, yaw, pitch };
  canvas.setPointerCapture(event.pointerId);
});
canvas.addEventListener('pointermove', event => {
  if (!drag || projection.value !== 'globe') return;
  yaw = drag.yaw + (event.clientX - drag.x) * 0.007;
  pitch = Math.max(-1.45, Math.min(1.45, drag.pitch + (event.clientY - drag.y) * 0.007));
  scheduleRedraw(true);
});
canvas.addEventListener('pointerup', event => {
  drag = null;
  if (canvas.hasPointerCapture(event.pointerId)) canvas.releasePointerCapture(event.pointerId);
  scheduleRedraw(false);
});
canvas.addEventListener('pointercancel', () => { drag = null; scheduleRedraw(false); });
window.addEventListener('beforeunload', () => { if (frameRequest) cancelAnimationFrame(frameRequest); client.dispose(); });
void generatePlanet();
'''
write("src/worldgen/diagnostics/worldgenTopographyLabStandalone.ts", lab_source)

# 6) Strengthen the narrow regressions so cumulative views cannot silently disappear again.
wg4_test = r'''import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import {
  WORLDGEN_PROTOCOL_VERSION,
  WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL,
  WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL,
  validateTopographyRequest,
  worldgenTopographyCommand,
} from '../dist/worldgen/protocol.js';

test('WG-4 browser protocol v7 exposes bounded coarse-to-fine topography generation', () => {
  assert.equal(WORLDGEN_PROTOCOL_VERSION, 7);
  assert.equal(WORLDGEN_TOPOGRAPHY_COARSE_MAX_LEVEL, 6);
  assert.equal(WORLDGEN_TOPOGRAPHY_FINE_MAX_LEVEL, 7);
  assert.doesNotThrow(() => validateTopographyRequest({ seed: 'wg4', coarseLevel: 4, fineLevel: 7, plateCount: 18 }));
  assert.throws(() => validateTopographyRequest({ seed: '', coarseLevel: 4, fineLevel: 6, plateCount: 18 }), /seed/i);
  assert.throws(() => validateTopographyRequest({ seed: 'wg4', coarseLevel: 5, fineLevel: 4, plateCount: 18 }), /fine level/i);
  assert.deepEqual(worldgenTopographyCommand(77, { seed: 'wg4', coarseLevel: 4, fineLevel: 6, plateCount: 18 }), { protocolVersion: 7, requestId: 77, type: 'generate-topography', payload: { seed: 'wg4', coarseLevel: 4, fineLevel: 6, plateCount: 18 } });
});

test('Planet Engine Lab keeps every WG-3.75 view and adds WG-4 views cumulatively', () => {
  const html = fs.readFileSync('index.html', 'utf8');
  assert.match(html, /PLANET ENGINE · THROUGH WG-4/);
  assert.match(html, />Generate Planet</);
  for (const term of [
    'Elevation above sea level', 'Bathymetry', 'Isostatic support', 'Oceanic thermal subsidence', 'Orogenic / collision uplift', 'Ridge relief', 'Rift / basin subsidence', 'Trench relief', 'Volcanic arc relief',
    'Inherited coarse samples', 'Nearest coarse provenance', 'Boundary provenance', 'Macro plate ownership', 'Refined kinematic domains', 'Fine tectonic boundaries', 'Fine geological regimes',
    'Crust type', 'Crust age', 'Crust thickness', 'Orogenic history', 'Ridge history', 'Trench history', 'Lithospheric strength', 'Lithospheric weakness', 'Structural zone type', 'Fragmentation propensity', 'Fine topology mesh',
  ]) assert.match(html, new RegExp(term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i'));
  assert.match(html, /one generation runs the accepted topology, tectonic, geological, lithospheric, multiresolution inheritance, and initial-topography pipeline/i);
  assert.doesNotMatch(html, /resource node|Region Inspector|NAV/);
});

test('WG-4 Lab uses one generated topography result for upstream and terrain diagnostics', () => {
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenTopographyLabStandalone.ts', 'utf8');
  assert.match(source, /generateTopography/);
  assert.doesNotMatch(source, /generateInheritance/);
  for (const field of ['nearestCoarseSource', 'inheritedSampleMask', 'kinematicDomainIds', 'boundaryKinds', 'boundaryCoarseSourceIndices', 'crustAgeMyr', 'crustThicknessKm', 'orogenicHistory', 'ridgeHistory', 'trenchHistory', 'strengthIndex', 'weaknessIndex', 'mantleDynamicSupportIndex', 'structuralZoneKind', 'fragmentationPropensity']) assert.match(source, new RegExp(field));
  assert.match(source, /tectonicBoundaryColor/);
  assert.match(source, /geologicalBoundaryColor/);
  assert.match(source, /provenanceColor/);
});

test('WG-4 browser transport preserves inherited diagnostics and physical profile values', () => {
  const worker = fs.readFileSync('src/worldgen/worldgenWorker.ts', 'utf8');
  const protocol = fs.readFileSync('src/worldgen/protocol.ts', 'utf8');
  const bridge = fs.readFileSync('rust/interlink-worldgen-wasm/src/topography_bridge.rs', 'utf8');
  for (const term of ['nearest_coarse_source', 'inherited_sample_mask', 'boundary_kinds', 'boundary_coarse_source_indices', 'kinematic_domain_ids']) assert.match(bridge, new RegExp(term));
  for (const field of ['nearestCoarseSource', 'inheritedSampleMask', 'boundaryKinds', 'boundaryCoarseSourceIndices', 'kinematicDomainIds']) assert.match(protocol, new RegExp(field));
  assert.match(worker, /internalHeatFluxWPerM2:\s*output\.internal_heat_flux_w_per_m2\(\)/);
  assert.match(worker, /mantleThermalExpansivityPerK:\s*output\.mantle_thermal_expansivity_per_k\(\)/);
  assert.doesNotMatch(worker, /internalHeatFluxWPerM2:\s*0/);
  assert.doesNotMatch(worker, /mantleThermalExpansivityPerK:\s*0/);
});

test('WG-4 renderer preserves the high-resolution interaction performance contract', () => {
  const source = fs.readFileSync('src/worldgen/diagnostics/worldgenTopographyLabStandalone.ts', 'utf8');
  assert.match(source, /requestAnimationFrame/);
  assert.match(source, /Float32Array/);
  assert.match(source, /canvas\.width !== width/);
  assert.match(source, /interactive && count > 20_000/);
  assert.doesNotMatch(source, /pointermove[\s\S]{0,500}redraw\(\)/);
});
'''
write("tests/wg4Topography.test.ts", wg4_test)

pages_test = r'''import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

test('GitHub Pages root serves the cumulative Planet Engine Lab', () => {
  assert.ok(fs.existsSync('index.html'), 'Pages root requires index.html');
  assert.ok(fs.existsSync('worldgen-lab.html'), 'direct lab URL must remain available');
  assert.ok(fs.existsSync('styles/base.css'), 'Pages lab base styles must be committed');
  assert.ok(fs.existsSync('styles/worldgenLab.css'), 'Pages lab styles must be committed');

  const html = fs.readFileSync('index.html', 'utf8');
  const direct = fs.readFileSync('worldgen-lab.html', 'utf8');
  assert.equal(direct, html, 'Pages root and direct lab URL must expose identical controls');
  assert.match(html, /PLANET ENGINE LAB/);
  assert.match(html, /PLANET ENGINE · THROUGH WG-4/);
  assert.match(html, />Generate Planet</);
  assert.match(html, /dist\/worldgen\/diagnostics\/worldgenTopographyLabStandalone\.js/);
  assert.match(html, /styles\/base\.css/);
  assert.match(html, /styles\/worldgenLab\.css/);
  assert.doesNotMatch(html, /Return to game/i);
});
'''
write("tests/pages.test.ts", pages_test)

# 7) Extend native bridge coverage to ensure the restored arrays align with the same fine planet.
bridge_test_path = "rust/interlink-worldgen-wasm/tests/topography_bridge.rs"
bridge_test = read(bridge_test_path)
bridge_test = replace_once(
    bridge_test,
    """    assert_eq!(\n        output.fine_sample_count() as usize,\n        output.submerged_mask().len()\n    );\n""",
    """    assert_eq!(\n        output.fine_sample_count() as usize,\n        output.submerged_mask().len()\n    );\n    assert_eq!(output.fine_sample_count() as usize, output.nearest_coarse_source().len());\n    assert_eq!(output.fine_sample_count() as usize, output.inherited_sample_mask().len());\n    assert_eq!(output.fine_sample_count() as usize, output.crust_age_myr().len());\n    assert_eq!(output.fine_sample_count() as usize, output.crust_thickness_km().len());\n    assert_eq!(output.fine_sample_count() as usize, output.strength_index().len());\n    assert_eq!(output.fine_sample_count() as usize, output.weakness_index().len());\n    assert_eq!(output.fine_sample_count() as usize, output.kinematic_domain_ids().len());\n    assert_eq!(output.fine_boundary_edge_count() as usize, output.boundary_kinds().len());\n    assert_eq!(output.fine_boundary_edge_count() as usize, output.boundary_coarse_source_indices().len());\n""",
    "topography bridge diagnostic test",
)
write(bridge_test_path, bridge_test)

# 8) Restore the repository's manual-only read-only validation workflow and remove this helper before final commit.
workflow = r'''name: WG-4 Manual Physical Validation

on:
  workflow_dispatch:

permissions:
  contents: read

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - name: Run WG-4 physical validation
        run: |
          cargo test -p interlink-worldgen --test topography_ensemble
          cargo run -p interlink-worldgen-cli -- topography --seed manual-wg4-a --coarse-level 4 --level 6 --plates 18
          cargo run -p interlink-worldgen-cli -- topography --seed manual-wg4-b --coarse-level 4 --level 6 --plates 18
          cargo check -p interlink-worldgen-wasm --target wasm32-unknown-unknown
'''
write(".github/workflows/wg4-debug.yml", workflow)
Path(__file__).unlink()
