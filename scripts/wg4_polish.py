from pathlib import Path


def patch(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if new in text:
        return
    if old not in text:
        raise RuntimeError(f"missing patch anchor in {path}: {old[:120]!r}")
    file.write_text(text.replace(old, new, 1))


# Make inherited structural categories and refined kinematic domains materially
# affect WG-4's mechanical expression without creating new terrain forcing.
patch(
    "rust/interlink-worldgen/src/topography.rs",
    "const CRUST_TRANSITIONAL: u8 = 2;\n",
    "const CRUST_TRANSITIONAL: u8 = 2;\nconst STRUCTURE_SUTURE: u8 = 1;\nconst STRUCTURE_RIFT: u8 = 2;\n",
)
patch(
    "rust/interlink-worldgen/src/topography.rs",
    "        inherited.structural_fabric_strength.len(),\n        inherited.kinematic_domain_ids.len(),",
    "        inherited.structural_fabric_strength.len(),\n        inherited.structural_zone_kind.len(),\n        inherited.kinematic_domain_ids.len(),",
)
patch(
    "rust/interlink-worldgen/src/topography.rs",
    "                let weight = interface / center;\n                weighted_sum += current[neighbor] * weight;",
    "                let domain_factor = if inherited.kinematic_domain_ids[neighbor]\n                    == inherited.kinematic_domain_ids[sample]\n                {\n                    1.0\n                } else {\n                    0.35\n                };\n                let weight = interface / center * domain_factor;\n                weighted_sum += current[neighbor] * weight;",
)
patch(
    "rust/interlink-worldgen/src/topography.rs",
    "        let structural_focus =\n            1.0 + 0.25 * f64::from(inherited.structural_fabric_strength[i]).clamp(0.0, 1.0);",
    "        let fabric = f64::from(inherited.structural_fabric_strength[i]).clamp(0.0, 1.0);\n        let collision_focus = 1.0\n            + fabric\n                * if inherited.structural_zone_kind[i] == STRUCTURE_SUTURE {\n                    0.55\n                } else {\n                    0.20\n                };\n        let rift_focus = 1.0\n            + fabric\n                * if inherited.structural_zone_kind[i] == STRUCTURE_RIFT {\n                    0.55\n                } else {\n                    0.20\n                };",
)
patch(
    "rust/interlink-worldgen/src/topography.rs",
    "            + p.collision_uplift_scale_m * collision_kernel * structural_focus;",
    "            + p.collision_uplift_scale_m * collision_kernel * collision_focus;",
)
patch(
    "rust/interlink-worldgen/src/topography.rs",
    "            * (rift_kernel * structural_focus + 0.55 * f64::from(inherited.rift_history[i]))",
    "            * (rift_kernel * rift_focus + 0.55 * f64::from(inherited.rift_history[i]))",
)

# Preserve the complete explicit physical-profile contract through the WG-4 bridge.
patch(
    "rust/interlink-worldgen-wasm/src/topography_bridge.rs",
    "    pub fn isostatic_mantle_density_kg_per_m3(&self) -> f64 {\n        self.parameters.isostatic_mantle_density_kg_per_m3\n    }\n",
    "    pub fn isostatic_mantle_density_kg_per_m3(&self) -> f64 {\n        self.parameters.isostatic_mantle_density_kg_per_m3\n    }\n    pub fn internal_heat_flux_w_per_m2(&self) -> f64 {\n        self.parameters.internal_heat_flux_w_per_m2\n    }\n    pub fn mantle_thermal_expansivity_per_k(&self) -> f64 {\n        self.parameters.mantle_thermal_expansivity_per_k\n    }\n",
)
patch(
    "src/worldgen/worldgenWorker.ts",
    "  radius_m(): number; surface_gravity_m_s2(): number; surface_water_mass_kg(): number; equivalent_global_water_depth_m(): number; ocean_water_density_kg_per_m3(): number; isostatic_mantle_density_kg_per_m3(): number;",
    "  radius_m(): number; surface_gravity_m_s2(): number; surface_water_mass_kg(): number; equivalent_global_water_depth_m(): number; ocean_water_density_kg_per_m3(): number; isostatic_mantle_density_kg_per_m3(): number; internal_heat_flux_w_per_m2(): number; mantle_thermal_expansivity_per_k(): number;",
)
patch(
    "src/worldgen/worldgenWorker.ts",
    "internalHeatFluxWPerM2: 0, mantleThermalExpansivityPerK: 0",
    "internalHeatFluxWPerM2: output.internal_heat_flux_w_per_m2(), mantleThermalExpansivityPerK: output.mantle_thermal_expansivity_per_k()",
)

# State the next physical frontier now that WG-4 is implemented in this PR.
readme = Path("README.md")
text = readme.read_text()
if "WG-5 climate (next)" not in text:
    text = text.replace(
        "WG-4 initial physical topography\n```",
        "WG-4 initial physical topography\n          ↓\nWG-5 climate (next)\n```",
        1,
    )
    readme.write_text(text)
