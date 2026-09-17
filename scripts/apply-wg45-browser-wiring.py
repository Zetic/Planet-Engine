from pathlib import Path


def replace(path, old, new, count=1):
    p = Path(path)
    s = p.read_text()
    if old not in s:
        raise SystemExit(f'{path}: anchor not found: {old[:120]!r}')
    p.write_text(s.replace(old, new, count))

# Rust cumulative browser bridge.
path = 'rust/interlink-worldgen-wasm/src/climate_bridge.rs'
p = Path(path)
s = p.read_text()
s = s.replace(
    'generate_initial_topography, generate_lake_sediment_infill, generate_lakes_closed_basins,\n    generate_lithosphere, generate_lithosphere_from_history, generate_post_erosion_hydrology,',
    'generate_initial_topography, generate_lake_sediment_infill, generate_lakes_closed_basins,\n    generate_lithology_substrate, generate_lithosphere, generate_lithosphere_from_history,\n    generate_post_erosion_hydrology,',
    1,
)
s = s.replace(
    'InheritedPhysicalState, LakeRequest, LakeSedimentInfillRequest, LakeSedimentInfillState,\n    LithosphereRequest, PlanetPhysicalParameters, PostErosionHydrologyMetrics,',
    'InheritedPhysicalState, LakeRequest, LakeSedimentInfillRequest, LakeSedimentInfillState,\n    LithologyRequest, LithologyState, LithosphereRequest, PlanetPhysicalParameters,\n    PostErosionHydrologyMetrics,',
    1,
)
s = s.replace('const GENERATION_STAGE_COUNT: u32 = 18;', 'const GENERATION_STAGE_COUNT: u32 = 19;', 1)
s = s.replace(
    '    boundaries: InheritedBoundarySet,\n    terrain: TopographyState,\n    climate: ClimateState,',
    '    boundaries: InheritedBoundarySet,\n    terrain: TopographyState,\n    lithology: LithologyState,\n    climate: ClimateState,',
    1,
)
old = '''        report_generation_progress(progress, "topography", 7, 1, 1);
        inherited.release_topography_scratch();
        let coarse_topology_hash = coarse_topology.metrics().topology_hash_hex();
'''
new = '''        report_generation_progress(progress, "topography", 7, 1, 1);
        report_generation_progress(progress, "lithology-substrate", 8, 0, 1);
        let lithology = generate_lithology_substrate(
            &fine_topology,
            &inherited,
            &historical_identity,
            &LithologyRequest::new(seed.as_str()),
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
        report_generation_progress(progress, "lithology-substrate", 8, 1, 1);
        inherited.release_topography_scratch();
        let coarse_topology_hash = coarse_topology.metrics().topology_hash_hex();
'''
if old not in s:
    raise SystemExit('climate bridge topography anchor not found')
s = s.replace(old, new, 1)
# Shift all post-lithology progress slots.
for label, old_index, new_index in [
    ('climate-spinup', 8, 9),
    ('drainage-topology', 9, 10),
    ('runoff-discharge', 10, 11),
    ('lake-equilibrium', 11, 12),
    ('seasonal-hydrology', 12, 13),
    ('fluvial-erosion-sediment', 13, 14),
    ('bounded-terrain-evolution', 14, 15),
    ('post-erosion-hydrology', 15, 16),
    ('lake-sediment-infill', 16, 17),
]:
    s = s.replace(f'progress, "{label}", {old_index},', f'progress, "{label}", {new_index},')
    s = s.replace(f'            "{label}",\n            {old_index},', f'            "{label}",\n            {new_index},')
s = s.replace(
    '            boundaries,\n            terrain,\n            climate,',
    '            boundaries,\n            terrain,\n            lithology,\n            climate,',
    1,
)
getter_anchor = '''    pub fn crust_birth_age_myr(&self) -> Vec<f32> {
        self.historical_identity.crust_birth_age_myr.clone()
    }
'''
getters = getter_anchor + '''    pub fn lithology_stage_id(&self) -> String {
        self.lithology.stage.id.to_owned()
    }
    pub fn lithology_stage_version(&self) -> u32 {
        self.lithology.stage.version
    }
    pub fn lithology_stage_seed_hex(&self) -> String {
        format!("{:016x}", self.lithology.stage.derived_seed)
    }
    pub fn lithology_hash_hex(&self) -> String {
        self.lithology.metrics.lithology_hash_hex()
    }
    pub fn bedrock_class(&self) -> Vec<u8> {
        self.lithology.bedrock_class.clone()
    }
    pub fn rock_strength_index(&self) -> Vec<f32> {
        self.lithology.rock_strength_index.clone()
    }
    pub fn lithology_erodibility_index(&self) -> Vec<f32> {
        self.lithology.erodibility_index.clone()
    }
    pub fn permeability_index(&self) -> Vec<f32> {
        self.lithology.permeability_index.clone()
    }
    pub fn weathering_susceptibility(&self) -> Vec<f32> {
        self.lithology.weathering_susceptibility.clone()
    }
    pub fn fines_fraction(&self) -> Vec<f32> {
        self.lithology.fines_fraction.clone()
    }
    pub fn carbonate_fraction(&self) -> Vec<f32> {
        self.lithology.carbonate_fraction.clone()
    }
'''
if getter_anchor not in s:
    raise SystemExit('climate bridge getter anchor not found')
s = s.replace(getter_anchor, getters, 1)
p.write_text(s)

# Browser protocol.
path = 'src/worldgen/protocol.ts'
p = Path(path)
s = p.read_text()
s = s.replace('export const WORLDGEN_PROTOCOL_VERSION = 21;', 'export const WORLDGEN_PROTOCOL_VERSION = 22;', 1)
structure_anchor = '''export const WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN = 4;
export const WORLDGEN_FRAGMENT_TERRANE = 1;
'''
bedrock_constants = '''export const WORLDGEN_STRUCTURE_CONTINENTAL_MARGIN = 4;
export const WORLDGEN_BEDROCK_OCEANIC_BASALT = 1;
export const WORLDGEN_BEDROCK_OCEANIC_SEDIMENT = 2;
export const WORLDGEN_BEDROCK_CRYSTALLINE_BASEMENT = 3;
export const WORLDGEN_BEDROCK_OROGENIC_METAMORPHIC = 4;
export const WORLDGEN_BEDROCK_ARC_VOLCANIC = 5;
export const WORLDGEN_BEDROCK_RIFT_VOLCANIC = 6;
export const WORLDGEN_BEDROCK_CLASTIC_SEDIMENTARY = 7;
export const WORLDGEN_BEDROCK_CARBONATE_PLATFORM = 8;
export const WORLDGEN_BEDROCK_ACCRETED_TERRANE = 9;
export const WORLDGEN_FRAGMENT_TERRANE = 1;
'''
if structure_anchor not in s:
    raise SystemExit('protocol structure anchor not found')
s = s.replace(structure_anchor, bedrock_constants, 1)
s = s.replace(
    '  lithosphereHash: string;\n  inheritanceHash: string;',
    '  lithosphereHash: string;\n  lithologyHash: string;\n  inheritanceHash: string;',
    1,
)
identity_anchor = '''  crustBirthAgeMyr: Float32Array;
  latestHistoricalEventKind: Uint8Array;
'''
identity_fields = '''  crustBirthAgeMyr: Float32Array;
  lithologyStage: WorldgenStageMetadata;
  bedrockClass: Uint8Array;
  rockStrengthIndex: Float32Array;
  lithologyErodibilityIndex: Float32Array;
  permeabilityIndex: Float32Array;
  weatheringSusceptibility: Float32Array;
  finesFraction: Float32Array;
  carbonateFraction: Float32Array;
  latestHistoricalEventKind: Uint8Array;
'''
# This anchor occurs in WorldgenClimateResult only once with exact neighboring names.
if identity_anchor not in s:
    raise SystemExit('protocol climate lithology anchor not found')
s = s.replace(identity_anchor, identity_fields, 1)
p.write_text(s)

# Worker Wasm interface and packaging.
path = 'src/worldgen/worldgenWorker.ts'
p = Path(path)
s = p.read_text()
s = s.replace(
    '  historical_identity_hash_hex(): string; historical_morphology_hash_hex(): string;\n  origin_plate_ids(): Uint16Array; historical_fragment_ids(): Uint16Array; current_plate_ids(): Uint16Array; crust_province_id(): Uint16Array; crust_birth_age_myr(): Float32Array;',
    '  historical_identity_hash_hex(): string; historical_morphology_hash_hex(): string;\n  origin_plate_ids(): Uint16Array; historical_fragment_ids(): Uint16Array; current_plate_ids(): Uint16Array; crust_province_id(): Uint16Array; crust_birth_age_myr(): Float32Array;\n  lithology_stage_id(): string; lithology_stage_version(): number; lithology_stage_seed_hex(): string; lithology_hash_hex(): string; bedrock_class(): Uint8Array; rock_strength_index(): Float32Array; lithology_erodibility_index(): Float32Array; permeability_index(): Float32Array; weathering_susceptibility(): Float32Array; fines_fraction(): Float32Array; carbonate_fraction(): Float32Array;',
    1,
)
s = s.replace("    progress('packaging', 17, 18, 0, 1);", "    progress('packaging', 18, 19, 0, 1);", 1)
extract_anchor = '''    const originPlateIds = output.origin_plate_ids(); const historicalFragmentIds = output.historical_fragment_ids(); const currentPlateIds = output.current_plate_ids(); const crustProvinceId = output.crust_province_id(); const crustBirthAgeMyr = output.crust_birth_age_myr();
'''
extract_new = extract_anchor + '''    const bedrockClass = output.bedrock_class(); const rockStrengthIndex = output.rock_strength_index(); const lithologyErodibilityIndex = output.lithology_erodibility_index(); const permeabilityIndex = output.permeability_index(); const weatheringSusceptibility = output.weathering_susceptibility(); const finesFraction = output.fines_fraction(); const carbonateFraction = output.carbonate_fraction();
'''
if extract_anchor not in s:
    raise SystemExit('worker extraction anchor not found')
s = s.replace(extract_anchor, extract_new, 1)
s = s.replace(
    'coarseTopologyHash: output.coarse_topology_hash_hex(), fineTopologyHash: output.fine_topology_hash_hex(), tectonicHash: output.tectonic_hash_hex(), geologyHash: output.geology_hash_hex(), lithosphereHash: output.lithosphere_hash_hex(), inheritanceHash:',
    'coarseTopologyHash: output.coarse_topology_hash_hex(), fineTopologyHash: output.fine_topology_hash_hex(), tectonicHash: output.tectonic_hash_hex(), geologyHash: output.geology_hash_hex(), lithosphereHash: output.lithosphere_hash_hex(), lithologyHash: output.lithology_hash_hex(), inheritanceHash:',
    1,
)
result_anchor = 'positions, neighborOffsets, neighbors, originPlateIds, historicalFragmentIds, currentPlateIds, crustProvinceId, crustBirthAgeMyr, latestHistoricalEventKind,'
result_new = 'positions, neighborOffsets, neighbors, originPlateIds, historicalFragmentIds, currentPlateIds, crustProvinceId, crustBirthAgeMyr, lithologyStage: { id: output.lithology_stage_id(), version: output.lithology_stage_version(), stageSeed: output.lithology_stage_seed_hex(), durationMs: 0 }, bedrockClass, rockStrengthIndex, lithologyErodibilityIndex, permeabilityIndex, weatheringSusceptibility, finesFraction, carbonateFraction, latestHistoricalEventKind,'
if result_anchor not in s:
    raise SystemExit('worker result anchor not found')
s = s.replace(result_anchor, result_new, 1)
# Include new buffers in final transferable list wherever the climate arrays are enumerated.
transfer_anchor = 'originPlateIds.buffer, historicalFragmentIds.buffer, currentPlateIds.buffer, crustProvinceId.buffer, crustBirthAgeMyr.buffer,'
if transfer_anchor in s:
    s = s.replace(transfer_anchor, transfer_anchor + ' bedrockClass.buffer, rockStrengthIndex.buffer, lithologyErodibilityIndex.buffer, permeabilityIndex.buffer, weatheringSusceptibility.buffer, finesFraction.buffer, carbonateFraction.buffer,', 1)
p.write_text(s)

# Main diagnostic selector.
path = 'index.html'
p = Path(path)
s = p.read_text()
option_anchor = '''            <option value="historical-fossil-orogen">Fossil orogen intensity</option>
          </optgroup>
'''
options = option_anchor + '''          <optgroup label="WG-4.5 lithology / substrate">
            <option value="bedrock-class">Bedrock class</option>
            <option value="rock-strength">Rock strength</option>
            <option value="lithology-erodibility">Lithology erodibility</option>
            <option value="permeability">Permeability</option>
            <option value="weathering-susceptibility">Weathering susceptibility</option>
            <option value="fines-fraction">Fines fraction</option>
            <option value="carbonate-fraction">Carbonate fraction</option>
          </optgroup>
'''
if option_anchor not in s:
    raise SystemExit('index diagnostic option anchor not found')
s = s.replace(option_anchor, options, 1)
p.write_text(s)

# Diagnostic renderer.
path = 'src/worldgen/diagnostics/worldgenClimateLabStandalone.ts'
p = Path(path)
s = p.read_text()
import_anchor = '  WORLDGEN_BOUNDARY_TRANSFORM,\n  WORLDGEN_CRUST_CONTINENTAL,'
import_new = '''  WORLDGEN_BOUNDARY_TRANSFORM,
  WORLDGEN_BEDROCK_ACCRETED_TERRANE,
  WORLDGEN_BEDROCK_ARC_VOLCANIC,
  WORLDGEN_BEDROCK_CARBONATE_PLATFORM,
  WORLDGEN_BEDROCK_CLASTIC_SEDIMENTARY,
  WORLDGEN_BEDROCK_CRYSTALLINE_BASEMENT,
  WORLDGEN_BEDROCK_OCEANIC_BASALT,
  WORLDGEN_BEDROCK_OCEANIC_SEDIMENT,
  WORLDGEN_BEDROCK_OROGENIC_METAMORPHIC,
  WORLDGEN_BEDROCK_RIFT_VOLCANIC,
  WORLDGEN_CRUST_CONTINENTAL,'''
if import_anchor not in s:
    raise SystemExit('diagnostics import anchor not found')
s = s.replace(import_anchor, import_new, 1)
crust_anchor = '''function crustColor(kind: number): string {
  if (kind === WORLDGEN_CRUST_CONTINENTAL) return '#b79a72';
  if (kind === WORLDGEN_CRUST_TRANSITIONAL) return '#9aab87';
  if (kind === WORLDGEN_CRUST_OCEANIC) return '#477aa3';
  return '#d7e2ef';
}
'''
bedrock_fn = crust_anchor + '''function bedrockColor(kind: number): string {
  if (kind === WORLDGEN_BEDROCK_OCEANIC_BASALT) return '#355f7c';
  if (kind === WORLDGEN_BEDROCK_OCEANIC_SEDIMENT) return '#768896';
  if (kind === WORLDGEN_BEDROCK_CRYSTALLINE_BASEMENT) return '#9c765d';
  if (kind === WORLDGEN_BEDROCK_OROGENIC_METAMORPHIC) return '#7b657d';
  if (kind === WORLDGEN_BEDROCK_ARC_VOLCANIC) return '#a94c3d';
  if (kind === WORLDGEN_BEDROCK_RIFT_VOLCANIC) return '#b97842';
  if (kind === WORLDGEN_BEDROCK_CLASTIC_SEDIMENTARY) return '#c0a477';
  if (kind === WORLDGEN_BEDROCK_CARBONATE_PLATFORM) return '#ddd5a5';
  if (kind === WORLDGEN_BEDROCK_ACCRETED_TERRANE) return '#6f8b68';
  return '#d7e2ef';
}
'''
if crust_anchor not in s:
    raise SystemExit('diagnostics crustColor anchor not found')
s = s.replace(crust_anchor, bedrock_fn, 1)
scalar_anchor = "    case 'historical-crust-birth-age': return { values: result.crustBirthAgeMyr, minimum: 0, maximum: 3_500, lowHue: 205, highHue: 24 };\n"
scalar_new = scalar_anchor + '''    case 'rock-strength': return { values: result.rockStrengthIndex, minimum: 0, maximum: 1, lowHue: 95, highHue: 355 };
    case 'lithology-erodibility': return { values: result.lithologyErodibilityIndex, minimum: 0, maximum: 1, lowHue: 160, highHue: 5 };
    case 'permeability': return { values: result.permeabilityIndex, minimum: 0, maximum: 1, lowHue: 35, highHue: 205 };
    case 'weathering-susceptibility': return { values: result.weatheringSusceptibility, minimum: 0, maximum: 1, lowHue: 55, highHue: 300 };
    case 'fines-fraction': return { values: result.finesFraction, minimum: 0, maximum: 1, lowHue: 90, highHue: 25 };
    case 'carbonate-fraction': return { values: result.carbonateFraction, minimum: 0, maximum: 1, lowHue: 210, highHue: 48 };
'''
if scalar_anchor not in s:
    raise SystemExit('diagnostics scalar anchor not found')
s = s.replace(scalar_anchor, scalar_new, 1)
sample_anchor = "  if (mode === 'crust-type') return crustColor(result.crustKind[sample]!);\n"
sample_new = sample_anchor + "  if (mode === 'bedrock-class') return bedrockColor(result.bedrockClass[sample]!);\n"
if sample_anchor not in s:
    raise SystemExit('diagnostics sample anchor not found')
s = s.replace(sample_anchor, sample_new, 1)
p.write_text(s)

# Browser protocol regression names/expectation.
path = 'tests/worldgenRewrite.test.ts'
p = Path(path)
s = p.read_text()
s = s.replace('Planet Engine browser protocol v21', 'Planet Engine browser protocol v22')
p.write_text(s)

# Historical browser contract: require the new cumulative fields and bridge call.
path = 'tests/historicalAncestryBrowser.test.ts'
p = Path(path)
s = p.read_text()
field_anchor = "    'fossilOrogenIntensity',\n"
if field_anchor in s:
    s = s.replace(field_anchor, field_anchor + "    'bedrockClass', 'rockStrengthIndex', 'lithologyErodibilityIndex', 'permeabilityIndex',\n    'weatheringSusceptibility', 'finesFraction', 'carbonateFraction',\n", 1)
p.write_text(s)
