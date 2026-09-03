const GRAVITATIONAL_CONSTANT_M3_KG_S2: f64 = 6.674_30e-11;
const FOUR_PI: f64 = 12.566_370_614_359_172;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanetPhysicalParameters {
    pub radius_m: f64,
    pub surface_gravity_m_s2: f64,
    pub rotation_period_s: f64,
    pub axial_tilt_rad: f64,
    pub orbital_period_s: f64,
    pub stellar_flux_w_m2: f64,
    pub reference_surface_pressure_pa: f64,
    pub surface_water_mass_kg: f64,
    pub ocean_water_density_kg_per_m3: f64,
    pub isostatic_mantle_density_kg_per_m3: f64,
    pub internal_heat_flux_w_per_m2: f64,
    pub mantle_thermal_expansivity_per_k: f64,
}

impl PlanetPhysicalParameters {
    /// Earth-like reference values remain the default physical profile for all
    /// accepted stages. Additional fields make later terrain/climate stages
    /// parameter-complete without silently changing existing Earth-like worlds.
    pub const fn earthlike_reference() -> Self {
        Self {
            radius_m: 6_371_000.0,
            surface_gravity_m_s2: 9.80665,
            rotation_period_s: 86_164.0905,
            axial_tilt_rad: 0.409_092_804_222_328_97,
            orbital_period_s: 31_558_149.763_545_6,
            stellar_flux_w_m2: 1_361.0,
            reference_surface_pressure_pa: 101_325.0,
            surface_water_mass_kg: 1.40e21,
            ocean_water_density_kg_per_m3: 1_025.0,
            isostatic_mantle_density_kg_per_m3: 3_300.0,
            internal_heat_flux_w_per_m2: 0.087,
            mantle_thermal_expansivity_per_k: 3.0e-5,
        }
    }

    pub fn surface_area_m2(&self) -> f64 {
        FOUR_PI * self.radius_m * self.radius_m
    }
    pub fn mass_kg(&self) -> f64 {
        self.surface_gravity_m_s2 * self.radius_m * self.radius_m / GRAVITATIONAL_CONSTANT_M3_KG_S2
    }
    pub fn mean_bulk_density_kg_per_m3(&self) -> f64 {
        let volume = (4.0 / 3.0) * std::f64::consts::PI * self.radius_m.powi(3);
        self.mass_kg() / volume
    }
    pub fn surface_water_volume_m3(&self) -> f64 {
        if self.surface_water_mass_kg == 0.0 {
            0.0
        } else {
            self.surface_water_mass_kg / self.ocean_water_density_kg_per_m3
        }
    }
    pub fn equivalent_global_water_depth_m(&self) -> f64 {
        self.surface_water_volume_m3() / self.surface_area_m2()
    }
    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        for value in [
            self.radius_m,
            self.surface_gravity_m_s2,
            self.rotation_period_s,
            self.axial_tilt_rad,
            self.orbital_period_s,
            self.stellar_flux_w_m2,
            self.reference_surface_pressure_pa,
            self.surface_water_mass_kg,
            self.ocean_water_density_kg_per_m3,
            self.isostatic_mantle_density_kg_per_m3,
            self.internal_heat_flux_w_per_m2,
            self.mantle_thermal_expansivity_per_k,
        ] {
            for byte in value.to_bits().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(FNV_PRIME);
            }
        }
        hash
    }
    pub fn parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.parameter_hash())
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.radius_m.is_finite() || self.radius_m <= 0.0 {
            return Err("planet radius must be finite and positive");
        }
        if !self.surface_gravity_m_s2.is_finite() || self.surface_gravity_m_s2 <= 0.0 {
            return Err("surface gravity must be finite and positive");
        }
        if !self.rotation_period_s.is_finite() || self.rotation_period_s <= 0.0 {
            return Err("rotation period must be finite and positive");
        }
        if !self.axial_tilt_rad.is_finite() {
            return Err("axial tilt must be finite");
        }
        if !self.orbital_period_s.is_finite() || self.orbital_period_s <= 0.0 {
            return Err("orbital period must be finite and positive");
        }
        if !self.stellar_flux_w_m2.is_finite() || self.stellar_flux_w_m2 <= 0.0 {
            return Err("stellar flux must be finite and positive");
        }
        if !self.reference_surface_pressure_pa.is_finite()
            || self.reference_surface_pressure_pa < 0.0
        {
            return Err("reference surface pressure must be finite and non-negative");
        }
        if !self.surface_water_mass_kg.is_finite() || self.surface_water_mass_kg < 0.0 {
            return Err("surface water mass must be finite and non-negative");
        }
        if !self.ocean_water_density_kg_per_m3.is_finite()
            || self.ocean_water_density_kg_per_m3 <= 0.0
        {
            return Err("ocean water density must be finite and positive");
        }
        if !self.isostatic_mantle_density_kg_per_m3.is_finite()
            || self.isostatic_mantle_density_kg_per_m3 <= 0.0
        {
            return Err("isostatic mantle density must be finite and positive");
        }
        if !self.internal_heat_flux_w_per_m2.is_finite() || self.internal_heat_flux_w_per_m2 < 0.0 {
            return Err("internal heat flux must be finite and non-negative");
        }
        if !self.mantle_thermal_expansivity_per_k.is_finite()
            || self.mantle_thermal_expansivity_per_k < 0.0
        {
            return Err("mantle thermal expansivity must be finite and non-negative");
        }
        Ok(())
    }
}

impl Default for PlanetPhysicalParameters {
    fn default() -> Self {
        Self::earthlike_reference()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earthlike_reference_is_physically_valid_and_has_expected_derived_scale() {
        let earth = PlanetPhysicalParameters::earthlike_reference();
        earth.validate().unwrap();
        assert!((5.9e24..6.1e24).contains(&earth.mass_kg()));
        assert!((5_400.0..5_600.0).contains(&earth.mean_bulk_density_kg_per_m3()));
        assert!((2_500.0..2_900.0).contains(&earth.equivalent_global_water_depth_m()));
        assert_eq!(
            earth.parameter_hash(),
            PlanetPhysicalParameters::default().parameter_hash()
        );
    }

    #[test]
    fn alternate_rocky_profiles_can_change_water_inventory_without_changing_default() {
        let earth = PlanetPhysicalParameters::earthlike_reference();
        let mut dry = earth;
        dry.surface_water_mass_kg = 0.0;
        dry.validate().unwrap();
        assert_eq!(dry.equivalent_global_water_depth_m(), 0.0);
        assert_ne!(dry.parameter_hash(), earth.parameter_hash());
        assert_eq!(PlanetPhysicalParameters::default(), earth);
    }
}
