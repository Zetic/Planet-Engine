from pathlib import Path
import os
import re

p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()

bulk = os.environ.get('EVAP_BULK', '0.0010')
substeps = os.environ.get('MOISTURE_SUBSTEPS', '4')
cfl = os.environ.get('MOISTURE_CFL', '0.65')
conv_rh = os.environ.get('CONVERGENCE_RH', '0.65')
conv_eff = os.environ.get('CONVERGENCE_EFF', '0.25')

s = s.replace('pub const CLIMATE_STAGE_VERSION: u32 = 3;', 'pub const CLIMATE_STAGE_VERSION: u32 = 4;')

old = '''    pub evaporation_relaxation: f64,\n    pub moisture_transport_cfl: f64,\n    pub condensation_relative_humidity: f64,'''
new = '''    pub evaporation_bulk_transfer_coefficient: f64,\n    pub moisture_transport_substeps: u8,\n    pub moisture_transport_cfl_limit: f64,\n    pub convergence_precipitation_relative_humidity: f64,\n    pub convergence_precipitation_efficiency: f64,\n    pub condensation_relative_humidity: f64,'''
if old not in s:
    raise SystemExit('parameter field anchor missing')
s = s.replace(old, new, 1)

old = '''            evaporation_relaxation: 0.055,\n            moisture_transport_cfl: 0.025,\n            condensation_relative_humidity: 0.80,'''
new = f'''            evaporation_bulk_transfer_coefficient: {bulk},\n            moisture_transport_substeps: {substeps},\n            moisture_transport_cfl_limit: {cfl},\n            convergence_precipitation_relative_humidity: {conv_rh},\n            convergence_precipitation_efficiency: {conv_eff},\n            condensation_relative_humidity: 0.80,'''
if old not in s:
    raise SystemExit('parameter default anchor missing')
s = s.replace(old, new, 1)

s = s.replace('''            self.evaporation_relaxation,\n            self.moisture_transport_cfl,\n            self.orographic_precipitation_strength,''', '''            self.evaporation_bulk_transfer_coefficient,\n            self.moisture_transport_cfl_limit,\n            self.orographic_precipitation_strength,''', 1)

s = s.replace('''            self.ocean_advection_cfl_limit,\n            self.condensation_relative_humidity,\n            self.condensation_efficiency,''', '''            self.ocean_advection_cfl_limit,\n            self.moisture_transport_cfl_limit,\n            self.convergence_precipitation_relative_humidity,\n            self.convergence_precipitation_efficiency,\n            self.condensation_relative_humidity,\n            self.condensation_efficiency,''', 1)

anchor = '''        if self.atmospheric_heat_solver_iterations == 0\n            || self.atmospheric_heat_solver_iterations > 32\n        {\n            return Err("atmospheric heat solver iterations must be from 1 through 32");\n        }'''
insert = anchor + '''\n        if self.moisture_transport_substeps == 0 || self.moisture_transport_substeps > 32 {\n            return Err("moisture transport substeps must be from 1 through 32");\n        }'''
if anchor not in s:
    raise SystemExit('validation anchor missing')
s = s.replace(anchor, insert, 1)

anchor = '''        hash = fnv_update(hash, &[self.atmospheric_heat_solver_iterations]);'''
insert = anchor + '''\n        hash = fnv_update(hash, &[self.moisture_transport_substeps]);'''
if anchor not in s:
    raise SystemExit('hash byte anchor missing')
s = s.replace(anchor, insert, 1)

s = s.replace('''            self.evaporation_relaxation,\n            self.moisture_transport_cfl,\n            self.condensation_relative_humidity,''', '''            self.evaporation_bulk_transfer_coefficient,\n            self.moisture_transport_cfl_limit,\n            self.convergence_precipitation_relative_humidity,\n            self.convergence_precipitation_efficiency,\n            self.condensation_relative_humidity,''', 1)

helper_anchor = '''fn build_ocean_projection_geometry(\n'''
helper = r'''#[derive(Clone, Copy, Debug)]
struct AtmosphericMoistureEdge {
    a: usize,
    b: usize,
    a_east: f64,
    a_north: f64,
    b_east: f64,
    b_north: f64,
    interface_length_m: f64,
}

fn build_atmospheric_moisture_edges(
    topology: &GeodesicTopology,
    east_bases: &[[f64; 3]],
    north_bases: &[[f64; 3]],
    radius_m: f64,
) -> Vec<AtmosphericMoistureEdge> {
    let mut edges = Vec::new();
    for a in 0..topology.metrics().sample_count as usize {
        for (neighbor, interface_arc) in topology
            .neighbors_of(a as u32)
            .iter()
            .zip(topology.neighbor_interface_arc_lengths_of(a as u32).iter())
        {
            let b = *neighbor as usize;
            if b <= a {
                continue;
            }
            let Some((a_east, a_north)) =
                edge_direction_components(topology, a, b, east_bases[a], north_bases[a])
            else {
                continue;
            };
            let Some((b_east, b_north)) =
                edge_direction_components(topology, b, a, east_bases[b], north_bases[b])
            else {
                continue;
            };
            edges.push(AtmosphericMoistureEdge {
                a,
                b,
                a_east,
                a_north,
                b_east,
                b_north,
                interface_length_m: (*interface_arc * radius_m).max(1.0),
            });
        }
    }
    edges
}

fn advect_moisture_substep(
    edges: &[AtmosphericMoistureEdge],
    moisture_mass: &mut [f64],
    cell_area_m2: &[f64],
    wind_east: &[f64],
    wind_north: &[f64],
    substep_seconds: f64,
    cfl_limit: f64,
) -> Vec<f64> {
    let mut requested = Vec::<(usize, usize, f64)>::with_capacity(edges.len());
    let mut requested_outflow = vec![0.0; moisture_mass.len()];
    for edge in edges {
        let outward_a = wind_east[edge.a] * edge.a_east + wind_north[edge.a] * edge.a_north;
        let outward_b = wind_east[edge.b] * edge.b_east + wind_north[edge.b] * edge.b_north;
        let normal_speed = 0.5 * (outward_a - outward_b);
        if normal_speed.abs() <= 1.0e-12 {
            continue;
        }
        let (donor, receiver) = if normal_speed >= 0.0 {
            (edge.a, edge.b)
        } else {
            (edge.b, edge.a)
        };
        let donor_column_moisture = moisture_mass[donor] / cell_area_m2[donor].max(1.0);
        let mass = donor_column_moisture
            * normal_speed.abs()
            * edge.interface_length_m
            * substep_seconds;
        if mass > 0.0 {
            requested.push((donor, receiver, mass));
            requested_outflow[donor] += mass;
        }
    }
    let mut donor_scale = vec![1.0; moisture_mass.len()];
    for i in 0..moisture_mass.len() {
        let allowed = moisture_mass[i] * cfl_limit;
        if requested_outflow[i] > allowed && requested_outflow[i] > 0.0 {
            donor_scale[i] = allowed / requested_outflow[i];
        }
    }
    let mut delta = vec![0.0; moisture_mass.len()];
    for (donor, receiver, mass) in requested {
        let transfer = mass * donor_scale[donor];
        delta[donor] -= transfer;
        delta[receiver] += transfer;
    }
    for i in 0..moisture_mass.len() {
        moisture_mass[i] = (moisture_mass[i] + delta[i]).max(0.0);
    }
    delta
}

'''
if helper_anchor not in s:
    raise SystemExit('moisture helper insertion anchor missing')
s = s.replace(helper_anchor, helper + helper_anchor, 1)

anchor = '''    let atmospheric_heat_geometry = build_atmospheric_heat_geometry(topology, planet.radius_m);'''
insert = anchor + '''\n    let atmospheric_moisture_edges = build_atmospheric_moisture_edges(\n        topology,\n        &east_bases,\n        &north_bases,\n        planet.radius_m,\n    );'''
if anchor not in s:
    raise SystemExit('geometry anchor missing')
s = s.replace(anchor, insert, 1)

pattern = re.compile(r'''            if atmosphere_exists \{\n                let mut air_mass = vec!\[0\.0; sample_count\];.*?            \} else \{\n                humidity\.fill\(0\.0\);\n            \}''', re.S)
replacement = r'''            if atmosphere_exists {
                let mut air_mass = vec![0.0; sample_count];
                let mut moisture_mass = vec![0.0; sample_count];
                for i in 0..sample_count {
                    air_mass[i] = pressure[i] / planet.surface_gravity_m_s2 * cell_area_m2[i];
                    moisture_mass[i] = humidity[i] * air_mass[i];
                }
                let moisture_before = moisture_mass.iter().sum::<f64>();
                let mut phase_evaporation = 0.0;
                let mut phase_precipitation = 0.0;

                // Bulk-aerodynamic evaporation is expressed as a surface mass flux
                // rather than a per-phase humidity relaxation, making the source
                // independent of mesh resolution and orbital phase count.
                for i in 0..sample_count {
                    if air_mass[i] <= 0.0 {
                        humidity[i] = 0.0;
                        continue;
                    }
                    let q = moisture_mass[i] / air_mass[i];
                    let wind_speed = norm2(wind_east[i], wind_north[i]).max(1.0);
                    let surface_temperature = if ocean[i] {
                        sea_surface_temperature[i]
                    } else {
                        temperature[i]
                    };
                    let saturation_surface =
                        saturation_specific_humidity(surface_temperature, pressure[i]);
                    let density = pressure[i]
                        / (specific_gas_constant * temperature[i].max(120.0));
                    let evaporation_flux = density
                        * parameters.evaporation_bulk_transfer_coefficient
                        * wind_speed
                        * (saturation_surface - q).max(0.0);
                    let potential_mass =
                        evaporation_flux * cell_area_m2[i] * phase_seconds;
                    potential_evaporation_mass_year[i] += potential_mass;
                    if ocean[i] {
                        moisture_mass[i] += potential_mass;
                        phase_evaporation += potential_mass;
                    }
                }

                // Resolve one seasonal wind state through multiple conservative
                // finite-volume advection substeps. Moisture can therefore cross
                // multiple cells during a phase without an index-order dependency.
                let substep_seconds =
                    phase_seconds / f64::from(parameters.moisture_transport_substeps);
                for _ in 0..usize::from(parameters.moisture_transport_substeps) {
                    let transport_delta = advect_moisture_substep(
                        &atmospheric_moisture_edges,
                        &mut moisture_mass,
                        &cell_area_m2,
                        &wind_east,
                        &wind_north,
                        substep_seconds,
                        parameters.moisture_transport_cfl_limit,
                    );
                    for i in 0..sample_count {
                        if transport_delta[i] <= 0.0 || air_mass[i] <= 0.0 {
                            continue;
                        }
                        let saturation_air =
                            saturation_specific_humidity(temperature[i], pressure[i]);
                        if saturation_air <= 1.0e-12 {
                            continue;
                        }
                        let relative_humidity =
                            (moisture_mass[i] / air_mass[i] / saturation_air).max(0.0);
                        let threshold = parameters.convergence_precipitation_relative_humidity;
                        let activation = if threshold < 1.0 {
                            ((relative_humidity - threshold) / (1.0 - threshold)).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        let convergence_mass = transport_delta[i]
                            * parameters.convergence_precipitation_efficiency
                            * activation;
                        let precipitation_mass = convergence_mass.min(moisture_mass[i]);
                        moisture_mass[i] -= precipitation_mass;
                        precipitation_mass_year[i] += precipitation_mass;
                        precipitation_phase_max[i] =
                            precipitation_phase_max[i].max(precipitation_mass);
                        if temperature[i] <= parameters.snow_temperature_k {
                            cold_precipitation_mass_year[i] += precipitation_mass;
                        }
                        phase_precipitation += precipitation_mass;
                    }
                }

                for i in 0..sample_count {
                    if air_mass[i] <= 0.0 {
                        humidity[i] = 0.0;
                        continue;
                    }
                    let wind_speed = norm2(wind_east[i], wind_north[i]);
                    let current_q = moisture_mass[i] / air_mass[i];
                    let saturation_air = saturation_specific_humidity(temperature[i], pressure[i]);
                    let threshold = saturation_air * parameters.condensation_relative_humidity;
                    let excess_q = (current_q - threshold).max(0.0);
                    let condensation_mass =
                        excess_q * air_mass[i] * parameters.condensation_efficiency;
                    let along_slope = if wind_speed > 0.25 {
                        (wind_east[i] * terrain_gradient_east[i]
                            + wind_north[i] * terrain_gradient_north[i])
                            / wind_speed
                    } else {
                        0.0
                    };
                    let orographic_fraction = (along_slope.max(0.0)
                        * parameters.orographic_precipitation_strength)
                        .clamp(0.0, parameters.maximum_orographic_fraction);
                    let after_condensation = (moisture_mass[i] - condensation_mass).max(0.0);
                    let orographic_mass = after_condensation * orographic_fraction;
                    let precipitation_mass = condensation_mass + orographic_mass;
                    moisture_mass[i] = (after_condensation - orographic_mass).max(0.0);
                    precipitation_mass_year[i] += precipitation_mass;
                    precipitation_phase_max[i] = precipitation_phase_max[i].max(precipitation_mass);
                    if temperature[i] <= parameters.snow_temperature_k {
                        cold_precipitation_mass_year[i] += precipitation_mass;
                        snow_phase_count[i] += 1.0;
                    }
                    if ocean[i] && sea_surface_temperature[i] <= parameters.sea_ice_temperature_k {
                        sea_ice_phase_count[i] += 1.0;
                    }
                    phase_precipitation += precipitation_mass;
                    humidity[i] = (moisture_mass[i] / air_mass[i]).clamp(0.0, 0.2);
                }
                let moisture_after = moisture_mass.iter().sum::<f64>();
                let expected_change = phase_evaporation - phase_precipitation;
                moisture_budget_error_year +=
                    ((moisture_after - moisture_before) - expected_change).abs();
                global_evaporation_year += phase_evaporation;
                global_precipitation_year += phase_precipitation;
            } else {
                humidity.fill(0.0);
            }'''
s2, count = pattern.subn(replacement, s, count=1)
if count != 1:
    raise SystemExit(f'moisture block replacement count {count}')
s = s2

p.write_text(s)

# Keep calibration report compiling against the prototype parameter names.
p = Path('rust/interlink-worldgen/src/climate_calibration.rs')
s = p.read_text()
s = s.replace(
    'let requested_fraction = projected.abs() * phase_seconds / distance_m\n                    * request.parameters.moisture_transport_cfl;\n                if requested_fraction >= 0.22 {',
    'let substep_seconds = phase_seconds\n                    / f64::from(request.parameters.moisture_transport_substeps);\n                let requested_fraction = projected.abs() * substep_seconds / distance_m;\n                if requested_fraction >= request.parameters.moisture_transport_cfl_limit {',
)
p.write_text(s)
