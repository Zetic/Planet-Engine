from pathlib import Path
import os

coupling = os.environ.get('SURFACE_ALBEDO_COUPLING', '0.15')

p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()
repls = {
    '    pub land_albedo: f64,\n    pub ocean_albedo: f64,\n    pub snow_ice_albedo: f64,': '    pub land_albedo: f64,\n    pub ocean_albedo: f64,\n    pub surface_albedo_shortwave_coupling: f64,\n    pub snow_ice_albedo: f64,',
    '            land_albedo: 0.24,\n            ocean_albedo: 0.07,\n            snow_ice_albedo: 0.62,': f'            land_albedo: 0.24,\n            ocean_albedo: 0.07,\n            surface_albedo_shortwave_coupling: {coupling},\n            snow_ice_albedo: 0.62,',
    '            self.land_albedo,\n            self.ocean_albedo,\n            self.snow_ice_albedo,': '            self.land_albedo,\n            self.ocean_albedo,\n            self.surface_albedo_shortwave_coupling,\n            self.snow_ice_albedo,',
}
for old, new in repls.items():
    if old not in s:
        raise SystemExit(f'missing coupling anchor:\n{old}')
    s = s.replace(old, new, 1)

old = '''                let absorbed = (solar
                    * (1.0 - physical.atmospheric_shortwave_reflectivity)
                    * (1.0 - albedo)
                    + planet.internal_heat_flux_w_per_m2)
                    .max(0.0);
'''
new = '''                let effective_albedo = (physical.atmospheric_shortwave_reflectivity
                    + parameters.surface_albedo_shortwave_coupling
                        * (1.0 - physical.atmospheric_shortwave_reflectivity)
                        * albedo)
                    .clamp(0.0, 0.95);
                let absorbed = (solar * (1.0 - effective_albedo)
                    + planet.internal_heat_flux_w_per_m2)
                    .max(0.0);
'''
if old not in s:
    raise SystemExit('missing reduced shortwave formula anchor')
s = s.replace(old, new, 1)
p.write_text(s)

p = Path('rust/interlink-worldgen/src/climate_calibration.rs')
s = p.read_text()
old = '''        f64::from(climate.annual_mean_insolation_w_m2[index])
            * (1.0 - request.physical.atmospheric_shortwave_reflectivity)
            * (1.0 - albedo)
'''
new = '''        let effective_albedo = request.physical.atmospheric_shortwave_reflectivity
            + request.parameters.surface_albedo_shortwave_coupling
                * (1.0 - request.physical.atmospheric_shortwave_reflectivity)
                * albedo;
        f64::from(climate.annual_mean_insolation_w_m2[index])
            * (1.0 - effective_albedo.clamp(0.0, 0.95))
'''
if old not in s:
    raise SystemExit('missing calibration reduced shortwave anchor')
p.write_text(s.replace(old, new, 1))
