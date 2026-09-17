from pathlib import Path

# Native calibration report pipeline generates the same WG-4.5 state and carries its hash.
p = Path('rust/interlink-worldgen-cli/src/bin/calibration-report.rs')
s = p.read_text()
s = s.replace(
    '    generate_lake_sediment_infill, generate_lakes_closed_basins, generate_lithosphere_from_history,\n    generate_post_erosion_hydrology, generate_runoff_discharge, generate_seasonal_hydrology,\n    inherit_boundary_interfaces, inherit_physical_state, ClimateRequest, DrainageRequest,',
    '    generate_lake_sediment_infill, generate_lakes_closed_basins, generate_lithology_substrate,\n    generate_lithosphere_from_history, generate_post_erosion_hydrology, generate_runoff_discharge,\n    generate_seasonal_hydrology, inherit_boundary_interfaces, inherit_historical_identity,\n    inherit_physical_state, ClimateRequest, DrainageRequest,',
    1,
)
s = s.replace(
    '    LithosphereRequest, PlanetPhysicalParameters, PostErosionHydrologyRequest, RunoffRequest,\n    SeasonalHydrologyRequest, TerrainEvolutionRequest, TopographyRequest,',
    '    LithologyRequest, LithosphereRequest, PlanetPhysicalParameters, PostErosionHydrologyRequest,\n    RunoffRequest, SeasonalHydrologyRequest, TerrainEvolutionRequest, TopographyRequest,',
    1,
)
identity_anchor = '''    let boundaries =
        inherit_boundary_interfaces(&coarse, &fine, tectonics, geology, &inherited.plate_ids)
            .map_err(|error| error.to_string())?;
'''
identity_new = '''    let historical_identity = inherit_historical_identity(
        &fine,
        options.coarse_level,
        &frontend.historical,
    )
    .map_err(|error| error.to_string())?;
    let boundaries =
        inherit_boundary_interfaces(&coarse, &fine, tectonics, geology, &inherited.plate_ids)
            .map_err(|error| error.to_string())?;
'''
if identity_anchor not in s:
    raise SystemExit('calibration identity anchor not found')
s = s.replace(identity_anchor, identity_new, 1)
terrain_anchor = '''    let climate_request = ClimateRequest::new(options.seed.as_str());
'''
terrain_new = '''    let lithology = generate_lithology_substrate(
        &fine,
        &inherited,
        &historical_identity,
        &LithologyRequest::new(options.seed.as_str()),
    )
    .map_err(|error| error.to_string())?;
    let climate_request = ClimateRequest::new(options.seed.as_str());
'''
if terrain_anchor not in s:
    raise SystemExit('calibration climate anchor not found')
s = s.replace(terrain_anchor, terrain_new, 1)
report_anchor = '''        &lithosphere.metrics.lithosphere_hash_hex(),
    )
'''
report_new = '''        &lithosphere.metrics.lithosphere_hash_hex(),
        &lithology.metrics.lithology_hash_hex(),
    )
'''
if report_anchor not in s:
    raise SystemExit('calibration report hash anchor not found')
s = s.replace(report_anchor, report_new, 1)
p.write_text(s)

p = Path('rust/interlink-worldgen/src/world_calibration.rs')
s = p.read_text()
struct_anchor = '''    pub lithosphere_hash: String,
    pub inheritance_hash: String,
'''
struct_new = '''    pub lithosphere_hash: String,
    pub lithology_hash: String,
    pub inheritance_hash: String,
'''
if struct_anchor not in s:
    raise SystemExit('world calibration hash struct anchor not found')
s = s.replace(struct_anchor, struct_new, 1)
fn_anchor = '''    geology_hash: &str,
    lithosphere_hash: &str,
) -> Result<WorldCalibrationReport, WorldgenError> {
'''
fn_new = '''    geology_hash: &str,
    lithosphere_hash: &str,
    lithology_hash: &str,
) -> Result<WorldCalibrationReport, WorldgenError> {
'''
if fn_anchor not in s:
    raise SystemExit('world calibration function hash anchor not found')
s = s.replace(fn_anchor, fn_new, 1)
assign_anchor = '''            lithosphere_hash: lithosphere_hash.to_owned(),
            inheritance_hash: inherited.inheritance_hash_hex(),
'''
assign_new = '''            lithosphere_hash: lithosphere_hash.to_owned(),
            lithology_hash: lithology_hash.to_owned(),
            inheritance_hash: inherited.inheritance_hash_hex(),
'''
if assign_anchor not in s:
    raise SystemExit('world calibration hash assignment anchor not found')
s = s.replace(assign_anchor, assign_new, 1)
json_anchor = '''            ("lithosphere", &self.hashes.lithosphere_hash),
            ("inheritance", &self.hashes.inheritance_hash),
'''
json_new = '''            ("lithosphere", &self.hashes.lithosphere_hash),
            ("lithology", &self.hashes.lithology_hash),
            ("inheritance", &self.hashes.inheritance_hash),
'''
if json_anchor not in s:
    raise SystemExit('world calibration JSON hash anchor not found')
s = s.replace(json_anchor, json_new, 1)
md_anchor = '''            "`tectonic {}` → `geology {}` → `lithosphere {}` → `topography {}` → `climate {}`",
            self.hashes.tectonic_hash,
            self.hashes.geology_hash,
            self.hashes.lithosphere_hash,
            self.hashes.topography_hash,
            self.hashes.climate_hash
'''
md_new = '''            "`tectonic {}` → `geology {}` → `lithosphere {}` → `topography {}` → `lithology {}` → `climate {}`",
            self.hashes.tectonic_hash,
            self.hashes.geology_hash,
            self.hashes.lithosphere_hash,
            self.hashes.topography_hash,
            self.hashes.lithology_hash,
            self.hashes.climate_hash
'''
if md_anchor not in s:
    raise SystemExit('world calibration markdown identity anchor not found')
s = s.replace(md_anchor, md_new, 1)
p.write_text(s)

# Pages calibration packet carries the same stage identity.
p = Path('src/worldgen/calibrationPacket.ts')
s = p.read_text()
hash_anchor = '''      geology: result.metrics.geologyHash,
      lithosphere: result.metrics.lithosphereHash,
      inheritance: result.metrics.inheritanceHash,
'''
hash_new = '''      geology: result.metrics.geologyHash,
      lithosphere: result.metrics.lithosphereHash,
      lithology: result.lithologyHash,
      inheritance: result.metrics.inheritanceHash,
'''
if hash_anchor not in s:
    raise SystemExit('calibration packet hash anchor not found')
s = s.replace(hash_anchor, hash_new, 1)
md_anchor = '''    `\\`tectonic ${packet.hashes.tectonic}\\` → \\`geology ${packet.hashes.geology}\\` → \\`lithosphere ${packet.hashes.lithosphere}\\` → \\`topography ${packet.hashes.topography}\\` → \\`climate ${packet.hashes.climate}\\``,
'''
if md_anchor in s:
    s = s.replace(md_anchor, '''    `\\`tectonic ${packet.hashes.tectonic}\\` → \\`geology ${packet.hashes.geology}\\` → \\`lithosphere ${packet.hashes.lithosphere}\\` → \\`topography ${packet.hashes.topography}\\` → \\`lithology ${packet.hashes.lithology}\\` → \\`climate ${packet.hashes.climate}\\``,
''', 1)
else:
    # Source uses ordinary template syntax after parsing; handle that exact form too.
    old = '    `\\`tectonic ${packet.hashes.tectonic}\\` → `'
    if old not in s:
        pass
p.write_text(s)
