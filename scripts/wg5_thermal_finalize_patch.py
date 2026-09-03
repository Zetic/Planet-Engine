from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    s = p.read_text()
    if old not in s:
        raise SystemExit(f"missing anchor in {path}:\n{old}")
    p.write_text(s.replace(old, new, 1))


climate_path = "rust/interlink-worldgen/src/climate.rs"
p = Path(climate_path)
s = p.read_text()

mean_neighbor = '''fn mean_neighbor(topology: &GeodesicTopology, values: &[f64], sample: usize) -> f64 {
    let neighbors = topology.neighbors_of(sample as u32);
    if neighbors.is_empty() {
        return values[sample];
    }
    neighbors
        .iter()
        .map(|neighbor| values[*neighbor as usize])
        .sum::<f64>()
        / neighbors.len() as f64
}

'''
if mean_neighbor not in s:
    raise SystemExit("missing obsolete mean_neighbor")
s = s.replace(mean_neighbor, "", 1)

helper_anchor = "fn atmospheric_surface_height_m(submerged: bool, elevation_above_sea_level_m: f64) -> f64 {"
helper = '''pub(crate) fn effective_shortwave_albedo(
    atmospheric_reflectivity: f64,
    surface_coupling: f64,
    surface_albedo: f64,
) -> f64 {
    (atmospheric_reflectivity
        + surface_coupling * (1.0 - atmospheric_reflectivity) * surface_albedo)
        .clamp(0.0, 0.95)
}

'''
if helper_anchor not in s:
    raise SystemExit("missing shortwave helper anchor")
s = s.replace(helper_anchor, helper + helper_anchor, 1)

old_formula = '''                let effective_albedo = (physical.atmospheric_shortwave_reflectivity
                    + parameters.surface_albedo_shortwave_coupling
                        * (1.0 - physical.atmospheric_shortwave_reflectivity)
                        * albedo)
                    .clamp(0.0, 0.95);
'''
new_formula = '''                let effective_albedo = effective_shortwave_albedo(
                    physical.atmospheric_shortwave_reflectivity,
                    parameters.surface_albedo_shortwave_coupling,
                    albedo,
                );
'''
if old_formula not in s:
    raise SystemExit("missing inline reduced shortwave formula")
s = s.replace(old_formula, new_formula, 1)

hash_anchor = '''            self.convergence_temperature_rms_k,
            self.land_albedo,
            self.ocean_albedo,
            self.snow_ice_albedo,
'''
hash_replacement = '''            self.convergence_temperature_rms_k,
            self.land_albedo,
            self.ocean_albedo,
            self.surface_albedo_shortwave_coupling,
            self.snow_ice_albedo,
'''
if hash_anchor not in s:
    raise SystemExit("missing climate model hash coupling anchor")
s = s.replace(hash_anchor, hash_replacement, 1)

if s.count("self.surface_albedo_shortwave_coupling") < 2:
    raise SystemExit("surface shortwave coupling is not covered by validation and hash")

insert_test_anchor = '''    #[test]
    fn atmospheric_surface_ignores_submerged_relief() {
'''
new_tests = '''    #[test]
    fn effective_shortwave_albedo_preserves_atmospheric_and_surface_causality() {
        let ocean = effective_shortwave_albedo(0.25, 0.25, 0.07);
        let land = effective_shortwave_albedo(0.25, 0.25, 0.24);
        let snow = effective_shortwave_albedo(0.25, 0.25, 0.62);
        assert!((ocean - 0.263_125).abs() < 1.0e-12);
        assert!((land - 0.295).abs() < 1.0e-12);
        assert!((snow - 0.366_25).abs() < 1.0e-12);
        assert!(ocean < land && land < snow);
        assert_eq!(effective_shortwave_albedo(0.25, 0.0, 0.62), 0.25);
    }

    #[test]
    fn thermal_parameters_participate_in_state_identity() {
        let mut physical = ClimatePhysicalParameters::default();
        let physical_hash = physical.parameter_hash();
        physical.atmospheric_shortwave_reflectivity += 0.01;
        assert_ne!(physical.parameter_hash(), physical_hash);

        let mut model = ClimateParameters::default();
        let model_hash = model.parameter_hash();
        model.surface_albedo_shortwave_coupling += 0.01;
        assert_ne!(model.parameter_hash(), model_hash);
    }

    #[test]
    fn air_sea_exchange_conserves_combined_column_heat() {
        const WATER_DENSITY_KG_M3: f64 = 1_000.0;
        const WATER_SPECIFIC_HEAT_J_KG_K: f64 = 3_990.0;
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let physical = ClimatePhysicalParameters::default();
        let parameters = ClimateParameters::default();
        let pressure = planet.reference_surface_pressure_pa;
        let air_capacity = pressure / planet.surface_gravity_m_s2
            * physical.atmospheric_specific_heat_j_per_kg_k;
        let ocean_capacity = parameters.ocean_mixed_layer_depth_m
            * WATER_DENSITY_KG_M3
            * WATER_SPECIFIC_HEAT_J_KG_K;
        let mut air = 280.0;
        let mut sea = 300.0;
        let before = air_capacity * air + ocean_capacity * sea;
        exchange_air_sea_heat(
            &mut air,
            &mut sea,
            pressure,
            planet,
            physical,
            parameters,
            21_600.0,
        );
        let after = air_capacity * air + ocean_capacity * sea;
        assert!((after - before).abs() / before.abs() < 1.0e-12);
        assert!(air > 280.0);
        assert!(sea < 300.0);
    }

    #[test]
    fn implicit_atmospheric_diffusion_conserves_capacity_weighted_heat() {
        let geometry = AtmosphericHeatGeometry {
            edges: vec![AtmosphericHeatEdge {
                a: 0,
                b: 1,
                geometric_conductance: 1.0,
            }],
            diagonal_geometry: vec![1.0, 1.0],
        };
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let physical = ClimatePhysicalParameters::default();
        let parameters = ClimateParameters::default();
        let pressure = [planet.reference_surface_pressure_pa; 2];
        let area = [1.0e12, 1.0e12];
        let column_capacity = pressure[0] / planet.surface_gravity_m_s2
            * physical.atmospheric_specific_heat_j_per_kg_k;
        let mut temperature = [300.0, 280.0];
        let before = column_capacity * area[0] * temperature[0]
            + column_capacity * area[1] * temperature[1];
        diffuse_atmospheric_heat(
            &geometry,
            &mut temperature,
            &pressure,
            &area,
            planet,
            physical,
            parameters,
            86_400.0,
        );
        let after = column_capacity * area[0] * temperature[0]
            + column_capacity * area[1] * temperature[1];
        assert!((after - before).abs() / before.abs() < 1.0e-10);
        assert!(temperature[0] < 300.0);
        assert!(temperature[1] > 280.0);
    }

'''
if insert_test_anchor not in s:
    raise SystemExit("missing climate unit-test insertion anchor")
s = s.replace(insert_test_anchor, new_tests + insert_test_anchor, 1)
p.write_text(s)

# Calibration diagnostics now include atmospheric reflectivity, so use accurate names
# and the exact same effective-albedo helper as the generator.
calibration_path = "rust/interlink-worldgen/src/climate_calibration.rs"
p = Path(calibration_path)
s = p.read_text()
s = s.replace(
    "use crate::{\n",
    "use crate::climate::effective_shortwave_albedo;\nuse crate::{\n",
    1,
)
s = s.replace("clear_surface_absorbed_shortwave", "effective_absorbed_shortwave")
s = s.replace("clear_asr", "effective_asr")
old_calibration_formula = '''        let effective_albedo = request.physical.atmospheric_shortwave_reflectivity
            + request.parameters.surface_albedo_shortwave_coupling
                * (1.0 - request.physical.atmospheric_shortwave_reflectivity)
                * albedo;
        f64::from(climate.annual_mean_insolation_w_m2[index])
            * (1.0 - effective_albedo.clamp(0.0, 0.95))
'''
new_calibration_formula = '''        let effective_albedo = effective_shortwave_albedo(
            request.physical.atmospheric_shortwave_reflectivity,
            request.parameters.surface_albedo_shortwave_coupling,
            albedo,
        );
        f64::from(climate.annual_mean_insolation_w_m2[index]) * (1.0 - effective_albedo)
'''
if old_calibration_formula not in s:
    raise SystemExit("missing calibration effective shortwave formula")
s = s.replace(old_calibration_formula, new_calibration_formula, 1)
p.write_text(s)

for path in [
    "rust/interlink-worldgen-cli/examples/climate_calibration.rs",
    "rust/interlink-worldgen/tests/climate_calibration.rs",
]:
    p = Path(path)
    s = p.read_text().replace(
        "clear_surface_absorbed_shortwave", "effective_absorbed_shortwave"
    )
    if path.endswith("examples/climate_calibration.rs"):
        s = s.replace("clear_surface_asr_w_m2", "effective_asr_w_m2")
    p.write_text(s)

# Stage identity and physical diagnostics across the WASM boundary.
bridge_path = "rust/interlink-worldgen-wasm/src/climate_bridge.rs"
p = Path(bridge_path)
s = p.read_text()
anchor = '''    pub fn atmospheric_specific_heat_j_per_kg_k(&self) -> f64 { self.climate_physical.atmospheric_specific_heat_j_per_kg_k }
    pub fn atmospheric_longwave_optical_depth(&self) -> f64 { self.climate_physical.atmospheric_longwave_optical_depth }
'''
replacement = '''    pub fn atmospheric_specific_heat_j_per_kg_k(&self) -> f64 { self.climate_physical.atmospheric_specific_heat_j_per_kg_k }
    pub fn atmospheric_shortwave_reflectivity(&self) -> f64 { self.climate_physical.atmospheric_shortwave_reflectivity }
    pub fn atmospheric_longwave_optical_depth(&self) -> f64 { self.climate_physical.atmospheric_longwave_optical_depth }
'''
if anchor not in s:
    raise SystemExit("missing WASM climate physical getter anchor")
p.write_text(s.replace(anchor, replacement, 1))

bridge_test = Path("rust/interlink-worldgen-wasm/tests/climate_bridge.rs")
s = bridge_test.read_text()
s = s.replace('assert_eq!(output.stage_version(), 2);', 'assert_eq!(output.stage_version(), 3);', 1)
s = s.replace(
    '    assert!(output.atmospheric_mean_molar_mass_kg_per_mol() > 0.0);\n',
    '    assert!(output.atmospheric_mean_molar_mass_kg_per_mol() > 0.0);\n    assert!(output.atmospheric_shortwave_reflectivity() > 0.0);\n',
    1,
)
bridge_test.write_text(s)

# Update WG-5 design documentation.
doc = Path("docs/worldgen-rewrite/WG5_CLIMATE.md")
s = doc.read_text()
old_intro = "WG-5 converts the accepted WG-4 physical surface into a deterministic climatology. It is a generation-time physical solve, not a perpetual post-generation weather simulation. The corrected climate algorithm is stage version `2`; version `2` tightens orbital, atmospheric-transport, acceptance, diagnostic, and state-identity semantics without changing the browser protocol shape."
new_intro = "WG-5 converts the accepted WG-4 physical surface into a deterministic climatology. It is a generation-time physical solve, not a perpetual post-generation weather simulation. The thermally recalibrated climate algorithm is stage version `3`; version `3` rebuilds reduced shortwave forcing, atmospheric heat redistribution, and air-sea heat exchange while retaining the existing browser climate-state shape."
if old_intro not in s:
    raise SystemExit("missing WG5 climate intro anchor")
s = s.replace(old_intro, new_intro, 1)
thermal_anchor = "WG-5 intentionally includes a reduced B+ surface-ocean circulation model."
thermal_text = """Stage `3` separates unresolved atmospheric/background shortwave reflection from the fraction of local surface albedo that reaches the reduced top-of-atmosphere budget. The Earth-like reference uses atmospheric shortwave reflectivity `0.25` and surface-albedo coupling `0.25`; land, ocean, snow, and ice therefore remain causally distinct without exposing their raw surface-albedo contrast directly to the planetary budget. This is a reduced cloud/atmosphere masking term, not an explicit cloud field.\n\nAtmospheric heat redistribution is now a conservative geometry-aware diffusion solve. Mesh interfaces use physical interface length and center distance, atmospheric thermal capacity uses local pressure, cell area, and specific heat, and a deterministic diagonally-preconditioned conjugate-gradient solve advances the implicit diffusion step. The Earth-like reference diffusivity is `2.0e6 m^2/s`. Air-sea exchange is likewise heat-capacity-aware: an `8 W/m^2/K` exchange coefficient couples the atmospheric column to a `14 m` effective mixed layer while conserving their combined column heat absent diagnostic clamps.\n\n"""
if thermal_anchor not in s:
    raise SystemExit("missing WG5 thermal paragraph anchor")
s = s.replace(thermal_anchor, thermal_text + thermal_anchor, 1)
s = s.replace(
    "- atmospheric specific heat;\n- reduced longwave optical depth.",
    "- atmospheric specific heat;\n- reduced atmospheric shortwave reflectivity;\n- reduced longwave optical depth.",
    1,
)
s = s.replace(
    "`ClimateParameters` separately owns numerical/model choices such as albedo, thermal response, atmospheric heat transport, wind response, current coupling, ocean diffusion/advection, moisture transport, condensation, and orographic precipitation.",
    "`ClimateParameters` separately owns numerical/model choices such as surface-albedo shortwave coupling, land/ocean thermal response, atmospheric heat diffusivity and solver iterations, air-sea exchange and mixed-layer depth, wind response, current coupling, ocean diffusion/advection, moisture transport, condensation, and orographic precipitation.",
    1,
)
doc.write_text(s)

caldoc = Path("docs/worldgen-rewrite/WG5_CALIBRATION.md")
s = caldoc.read_text()
section = r'''

## Stage-3 thermal recalibration

The post-hypsometry thermal pass promotes WG-5 to `climate:coupled-surface@3`. The selected Earth-like reduced thermal parameters are:

- atmospheric shortwave reflectivity: `0.25`;
- surface-albedo shortwave coupling: `0.25`;
- reduced longwave optical depth: `1.20`;
- atmospheric heat diffusivity: `2.0e6 m^2/s`;
- air-sea exchange coefficient: `8 W/m^2/K`;
- effective ocean mixed-layer depth: `14 m`;
- deterministic atmospheric heat solver iterations: `20`.

The shortwave budget now uses an effective albedo composed of background atmospheric reflection plus an attenuated surface contribution. For the default clear land and ocean surface albedos this gives effective values of about `0.295` and `0.263`, respectively, while snow and ice retain stronger causal reflection. The calibration report therefore renames the old `clear_surface_absorbed_shortwave_*` diagnostics to `effective_absorbed_shortwave_*`.

The old neighbor-temperature relaxation is replaced by conservative, geometry-aware implicit atmospheric diffusion. Air-sea exchange now conserves combined atmospheric-column and mixed-layer heat and is parameterized in `W/m^2/K` instead of arbitrary temperature relaxation.

Two strict production-gate acceptance cases established the selected mixed-layer depth without changing the `0.08 K` convergence tolerance or the 10-year maximum spin-up bound:

| case | spin-up | final RMS | mean T | land T | ocean T | mean SST | moisture error |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| L6 `interlink-wg5`, 16 plates | 8 y | 0.073538 K | 283.293 K | 270.864 K | 289.916 K | 290.125 K | 3.74e-13 |
| L7 `ci-wg5-l7`, 24 plates | 10 y | 0.065438 K | 290.472 K | 281.933 K | 292.995 K | 293.308 K | 5.73e-13 |

These are different deterministic acceptance seeds and are not presented as a resolution-convergence pair. Their purpose is to prove the stage-3 thermal solve clears the existing L6/L7 acceptance gate across the intended quality range. The earlier stage-2 measurements in this document remain the historical baseline that motivated the thermal rebuild.

The reported TOA energy imbalance remains a reduced diagnostic proxy rather than a strict closed radiative-energy budget because WG-5 still uses a reduced gray-atmosphere target-temperature formulation and does not model explicit clouds, vertical atmospheric layers, or a full radiative-transfer column.

Moisture-routing cap saturation remains a separate calibration problem and is intentionally not redesigned by this thermal PR.
'''
if "## Stage-3 thermal recalibration" not in s:
    s += section
caldoc.write_text(s)
