from pathlib import Path
import os

sw = os.environ.get('SW_REFLECTIVITY', '0.22')
diff = os.environ.get('HEAT_DIFFUSIVITY', '100000.0')
exchange = os.environ.get('AIR_SEA_EXCHANGE', '8.0')
depth = os.environ.get('MIXED_LAYER_DEPTH', '50.0')
iters = os.environ.get('HEAT_SOLVER_ITERS', '10')

p = Path('rust/interlink-worldgen/src/climate.rs')
s = p.read_text()

repls = {
    'pub const CLIMATE_STAGE_VERSION: u32 = 2;': 'pub const CLIMATE_STAGE_VERSION: u32 = 3;',
    '    pub atmospheric_specific_heat_j_per_kg_k: f64,\n    pub atmospheric_longwave_optical_depth: f64,': '    pub atmospheric_specific_heat_j_per_kg_k: f64,\n    pub atmospheric_shortwave_reflectivity: f64,\n    pub atmospheric_longwave_optical_depth: f64,',
    '            atmospheric_specific_heat_j_per_kg_k: 1_004.0,\n            atmospheric_longwave_optical_depth: 0.90,': f'            atmospheric_specific_heat_j_per_kg_k: 1_004.0,\n            atmospheric_shortwave_reflectivity: {sw},\n            atmospheric_longwave_optical_depth: 0.90,',
    '        if !self.atmospheric_longwave_optical_depth.is_finite()\n            || self.atmospheric_longwave_optical_depth < 0.0': '        if !self.atmospheric_shortwave_reflectivity.is_finite()\n            || !(0.0..1.0).contains(&self.atmospheric_shortwave_reflectivity)\n        {\n            return Err("atmospheric shortwave reflectivity must be finite and within [0, 1)");\n        }\n        if !self.atmospheric_longwave_optical_depth.is_finite()\n            || self.atmospheric_longwave_optical_depth < 0.0',
    '            self.atmospheric_specific_heat_j_per_kg_k,\n            self.atmospheric_longwave_optical_depth,': '            self.atmospheric_specific_heat_j_per_kg_k,\n            self.atmospheric_shortwave_reflectivity,\n            self.atmospheric_longwave_optical_depth,',
    '    pub atmospheric_heat_relaxation: f64,\n    pub air_sea_exchange_relaxation: f64,': '    pub atmospheric_heat_diffusivity_m2_s: f64,\n    pub atmospheric_heat_solver_iterations: u8,\n    pub air_sea_exchange_coefficient_w_m2_k: f64,\n    pub ocean_mixed_layer_depth_m: f64,',
    '            atmospheric_heat_relaxation: 0.16,\n            air_sea_exchange_relaxation: 0.14,': f'            atmospheric_heat_diffusivity_m2_s: {diff},\n            atmospheric_heat_solver_iterations: {iters},\n            air_sea_exchange_coefficient_w_m2_k: {exchange},\n            ocean_mixed_layer_depth_m: {depth},',
    '            self.maximum_wind_speed_m_s,\n            self.ocean_wind_coupling,': '            self.maximum_wind_speed_m_s,\n            self.atmospheric_heat_diffusivity_m2_s,\n            self.air_sea_exchange_coefficient_w_m2_k,\n            self.ocean_mixed_layer_depth_m,\n            self.ocean_wind_coupling,',
    '            self.land_thermal_relaxation,\n            self.ocean_thermal_relaxation,\n            self.atmospheric_heat_relaxation,\n            self.air_sea_exchange_relaxation,': '            self.land_thermal_relaxation,\n            self.ocean_thermal_relaxation,',
    '        if self.ocean_current_correction_iterations > 24 {': '        if self.atmospheric_heat_solver_iterations == 0\n            || self.atmospheric_heat_solver_iterations > 32\n        {\n            return Err("atmospheric heat solver iterations must be from 1 through 32");\n        }\n        if self.ocean_current_correction_iterations > 24 {',
    '        hash = fnv_update(hash, &[self.ocean_current_correction_iterations]);': '        hash = fnv_update(hash, &[self.ocean_current_correction_iterations]);\n        hash = fnv_update(hash, &[self.atmospheric_heat_solver_iterations]);',
    '            self.land_thermal_relaxation,\n            self.ocean_thermal_relaxation,\n            self.atmospheric_heat_relaxation,\n            self.air_sea_exchange_relaxation,\n            self.wind_thermal_gradient_scale,': '            self.land_thermal_relaxation,\n            self.ocean_thermal_relaxation,\n            self.atmospheric_heat_diffusivity_m2_s,\n            self.air_sea_exchange_coefficient_w_m2_k,\n            self.ocean_mixed_layer_depth_m,\n            self.wind_thermal_gradient_scale,',
}
for old, new in repls.items():
    if old not in s:
        raise SystemExit(f'missing climate anchor:\n{old}')
    s = s.replace(old, new, 1)

insert_anchor = '#[derive(Clone, Copy, Debug)]\nstruct OceanProjectionEdge {'
helper = r'''#[derive(Clone, Copy, Debug)]
struct AtmosphericHeatEdge {
    a: usize,
    b: usize,
    geometric_conductance: f64,
}

#[derive(Clone, Debug)]
struct AtmosphericHeatGeometry {
    edges: Vec<AtmosphericHeatEdge>,
    diagonal_geometry: Vec<f64>,
}

fn build_atmospheric_heat_geometry(
    topology: &GeodesicTopology,
    radius_m: f64,
) -> AtmosphericHeatGeometry {
    let count = topology.metrics().sample_count as usize;
    let mut edges = Vec::new();
    let mut diagonal_geometry = vec![0.0; count];
    for a in 0..count {
        for ((neighbor, arc), interface_arc) in topology
            .neighbors_of(a as u32)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(a as u32).iter())
            .zip(topology.neighbor_interface_arc_lengths_of(a as u32).iter())
        {
            let b = *neighbor as usize;
            if b <= a {
                continue;
            }
            let distance_m = (*arc * radius_m).max(1.0);
            let interface_length_m = (*interface_arc * radius_m).max(1.0);
            let geometric_conductance = (interface_length_m / distance_m).max(1.0e-12);
            edges.push(AtmosphericHeatEdge {
                a,
                b,
                geometric_conductance,
            });
            diagonal_geometry[a] += geometric_conductance;
            diagonal_geometry[b] += geometric_conductance;
        }
    }
    AtmosphericHeatGeometry {
        edges,
        diagonal_geometry,
    }
}

fn apply_atmospheric_heat_matrix(
    geometry: &AtmosphericHeatGeometry,
    thermal_capacity_j_k: &[f64],
    diffusion_scale_j_k: f64,
    values: &[f64],
    output: &mut [f64],
) {
    for i in 0..values.len() {
        output[i] = thermal_capacity_j_k[i] * values[i];
    }
    for edge in &geometry.edges {
        let contribution = diffusion_scale_j_k
            * edge.geometric_conductance
            * (values[edge.a] - values[edge.b]);
        output[edge.a] += contribution;
        output[edge.b] -= contribution;
    }
}

fn diffuse_atmospheric_heat(
    geometry: &AtmosphericHeatGeometry,
    temperature: &mut [f64],
    pressure_pa: &[f64],
    cell_area_m2: &[f64],
    planet: PlanetPhysicalParameters,
    physical: ClimatePhysicalParameters,
    parameters: ClimateParameters,
    phase_seconds: f64,
) {
    if planet.reference_surface_pressure_pa <= 0.0
        || parameters.atmospheric_heat_diffusivity_m2_s <= 0.0
        || geometry.edges.is_empty()
    {
        return;
    }

    let reference_column_capacity_j_m2_k = planet.reference_surface_pressure_pa
        / planet.surface_gravity_m_s2
        * physical.atmospheric_specific_heat_j_per_kg_k;
    let diffusion_scale_j_k = phase_seconds
        * parameters.atmospheric_heat_diffusivity_m2_s
        * reference_column_capacity_j_m2_k;
    let mut capacity = vec![0.0; temperature.len()];
    let mut rhs = vec![0.0; temperature.len()];
    let mut diagonal = vec![0.0; temperature.len()];
    for i in 0..temperature.len() {
        let column_capacity = (pressure_pa[i] / planet.surface_gravity_m_s2
            * physical.atmospheric_specific_heat_j_per_kg_k)
            .max(reference_column_capacity_j_m2_k * 0.02);
        capacity[i] = column_capacity * cell_area_m2[i];
        rhs[i] = capacity[i] * temperature[i];
        diagonal[i] = capacity[i] + diffusion_scale_j_k * geometry.diagonal_geometry[i];
    }

    let mut x = temperature.to_vec();
    let mut matrix_x = vec![0.0; x.len()];
    apply_atmospheric_heat_matrix(geometry, &capacity, diffusion_scale_j_k, &x, &mut matrix_x);
    let mut residual = rhs
        .iter()
        .zip(matrix_x.iter())
        .map(|(b, ax)| b - ax)
        .collect::<Vec<_>>();
    let mut preconditioned = residual
        .iter()
        .enumerate()
        .map(|(i, r)| r / diagonal[i].max(1.0e-18))
        .collect::<Vec<_>>();
    let mut direction = preconditioned.clone();
    let mut matrix_direction = vec![0.0; x.len()];
    let mut rho = residual
        .iter()
        .zip(preconditioned.iter())
        .map(|(r, z)| r * z)
        .sum::<f64>();

    for _ in 0..usize::from(parameters.atmospheric_heat_solver_iterations) {
        if !rho.is_finite() || rho <= 1.0e-18 {
            break;
        }
        apply_atmospheric_heat_matrix(
            geometry,
            &capacity,
            diffusion_scale_j_k,
            &direction,
            &mut matrix_direction,
        );
        let denominator = direction
            .iter()
            .zip(matrix_direction.iter())
            .map(|(d, ad)| d * ad)
            .sum::<f64>();
        if !denominator.is_finite() || denominator <= 1.0e-18 {
            break;
        }
        let alpha = rho / denominator;
        for i in 0..x.len() {
            x[i] += alpha * direction[i];
            residual[i] -= alpha * matrix_direction[i];
            preconditioned[i] = residual[i] / diagonal[i].max(1.0e-18);
        }
        let next_rho = residual
            .iter()
            .zip(preconditioned.iter())
            .map(|(r, z)| r * z)
            .sum::<f64>();
        if !next_rho.is_finite() || next_rho <= 1.0e-18 {
            break;
        }
        let beta = next_rho / rho;
        for i in 0..direction.len() {
            direction[i] = preconditioned[i] + beta * direction[i];
        }
        rho = next_rho;
    }

    for i in 0..temperature.len() {
        temperature[i] = x[i].clamp(120.0, 355.0);
    }
}

fn exchange_air_sea_heat(
    air_temperature_k: &mut f64,
    sea_surface_temperature_k: &mut f64,
    pressure_pa: f64,
    planet: PlanetPhysicalParameters,
    physical: ClimatePhysicalParameters,
    parameters: ClimateParameters,
    phase_seconds: f64,
) {
    if pressure_pa <= 0.0 || parameters.air_sea_exchange_coefficient_w_m2_k <= 0.0 {
        return;
    }
    const WATER_DENSITY_KG_M3: f64 = 1_000.0;
    const WATER_SPECIFIC_HEAT_J_KG_K: f64 = 3_990.0;
    let air_capacity = (pressure_pa / planet.surface_gravity_m_s2
        * physical.atmospheric_specific_heat_j_per_kg_k)
        .max(1.0);
    let ocean_capacity = (parameters.ocean_mixed_layer_depth_m
        * WATER_DENSITY_KG_M3
        * WATER_SPECIFIC_HEAT_J_KG_K)
        .max(1.0);
    let total_capacity = air_capacity + ocean_capacity;
    let equilibrium = (air_capacity * *air_temperature_k
        + ocean_capacity * *sea_surface_temperature_k)
        / total_capacity;
    let difference = *air_temperature_k - *sea_surface_temperature_k;
    let decay_rate = parameters.air_sea_exchange_coefficient_w_m2_k
        * (1.0 / air_capacity + 1.0 / ocean_capacity);
    let remaining_difference = difference * (-decay_rate * phase_seconds).exp();
    *air_temperature_k = (equilibrium
        + ocean_capacity / total_capacity * remaining_difference)
        .clamp(120.0, 355.0);
    *sea_surface_temperature_k = (equilibrium
        - air_capacity / total_capacity * remaining_difference)
        .clamp(250.0, 330.0);
}

'''
if insert_anchor not in s:
    raise SystemExit('missing atmospheric helper insertion anchor')
s = s.replace(insert_anchor, helper + insert_anchor, 1)

# Remove old cp response and build heat geometry.
old = '''    let atmospheric_heat_capacity_response =
        (1_004.0 / physical.atmospheric_specific_heat_j_per_kg_k).clamp(0.25, 4.0);
    let phase_seconds = planet.orbital_period_s / phase_count as f64;
'''
new = '''    let phase_seconds = planet.orbital_period_s / phase_count as f64;
'''
if old not in s:
    raise SystemExit('missing atmospheric heat response anchor')
s = s.replace(old, new, 1)

old = '''    let ocean_projection_geometry = build_ocean_projection_geometry(
        topology,
        &ocean,
        &east_bases,
        &north_bases,
        planet.radius_m,
    );
'''
new = '''    let atmospheric_heat_geometry = build_atmospheric_heat_geometry(topology, planet.radius_m);
    let ocean_projection_geometry = build_ocean_projection_geometry(
        topology,
        &ocean,
        &east_bases,
        &north_bases,
        planet.radius_m,
    );
'''
if old not in s:
    raise SystemExit('missing geometry anchor')
s = s.replace(old, new, 1)

old = '''                let absorbed =
                    (solar * (1.0 - albedo) + planet.internal_heat_flux_w_per_m2).max(0.0);
'''
new = '''                let absorbed = (solar
                    * (1.0 - physical.atmospheric_shortwave_reflectivity)
                    * (1.0 - albedo)
                    + planet.internal_heat_flux_w_per_m2)
                    .max(0.0);
'''
if old not in s:
    raise SystemExit('missing shortwave anchor')
s = s.replace(old, new, 1)

old = '''            let previous_temperature = temperature.clone();
            for i in 0..sample_count {
                let neighbor_temperature = mean_neighbor(topology, &previous_temperature, i);
                let atmospheric_transport = if atmosphere_exists {
                    parameters.atmospheric_heat_relaxation
                        * atmospheric_heat_capacity_response
                        * (neighbor_temperature - previous_temperature[i])
                } else {
                    0.0
                };
                let transported_target = radiative_target[i] + atmospheric_transport;
                let relaxation = if ocean[i] {
                    parameters.ocean_thermal_relaxation
                } else {
                    parameters.land_thermal_relaxation
                };
                temperature[i] = (previous_temperature[i]
                    + relaxation * (transported_target - previous_temperature[i]))
                    .clamp(120.0, 355.0);
                if atmosphere_exists {
                    let scale_height = (specific_gas_constant * temperature[i]
                        / planet.surface_gravity_m_s2)
                        .max(1.0);
                    pressure[i] = planet.reference_surface_pressure_pa
                        * (-terrain_height_m[i] / scale_height).exp();
                } else {
                    pressure[i] = 0.0;
                }
            }
'''
new = '''            let previous_temperature = temperature.clone();
            for i in 0..sample_count {
                let relaxation = if ocean[i] {
                    parameters.ocean_thermal_relaxation
                } else {
                    parameters.land_thermal_relaxation
                };
                temperature[i] = (previous_temperature[i]
                    + relaxation * (radiative_target[i] - previous_temperature[i]))
                    .clamp(120.0, 355.0);
            }
            diffuse_atmospheric_heat(
                &atmospheric_heat_geometry,
                &mut temperature,
                &pressure,
                &cell_area_m2,
                planet,
                physical,
                parameters,
                phase_seconds,
            );
            for i in 0..sample_count {
                if atmosphere_exists {
                    let scale_height = (specific_gas_constant * temperature[i]
                        / planet.surface_gravity_m_s2)
                        .max(1.0);
                    pressure[i] = planet.reference_surface_pressure_pa
                        * (-terrain_height_m[i] / scale_height).exp();
                } else {
                    pressure[i] = 0.0;
                }
            }
'''
if old not in s:
    raise SystemExit('missing atmospheric update anchor')
s = s.replace(old, new, 1)

old = '''                next_sst[i] = (previous_sst[i]
                    + advection_delta
                    + parameters.ocean_temperature_diffusion * (neighbor_sst - previous_sst[i])
                    + parameters.air_sea_exchange_relaxation * (temperature[i] - previous_sst[i]))
                    .clamp(260.0, 325.0);
            }
            sea_surface_temperature = next_sst;
            for i in 0..sample_count {
                if ocean[i] {
                    temperature[i] += parameters.air_sea_exchange_relaxation
                        * (sea_surface_temperature[i] - temperature[i]);
                }
            }
'''
new = '''                next_sst[i] = (previous_sst[i]
                    + advection_delta
                    + parameters.ocean_temperature_diffusion * (neighbor_sst - previous_sst[i]))
                    .clamp(250.0, 330.0);
            }
            sea_surface_temperature = next_sst;
            for i in 0..sample_count {
                if ocean[i] {
                    exchange_air_sea_heat(
                        &mut temperature[i],
                        &mut sea_surface_temperature[i],
                        pressure[i],
                        planet,
                        physical,
                        parameters,
                        phase_seconds,
                    );
                }
            }
'''
if old not in s:
    raise SystemExit('missing air-sea exchange anchor')
s = s.replace(old, new, 1)

p.write_text(s)

# Update calibration ASR proxy so it matches the new reduced shortwave budget.
p = Path('rust/interlink-worldgen/src/climate_calibration.rs')
s = p.read_text()
old = '''        f64::from(climate.annual_mean_insolation_w_m2[index]) * (1.0 - albedo)
'''
new = '''        f64::from(climate.annual_mean_insolation_w_m2[index])
            * (1.0 - request.physical.atmospheric_shortwave_reflectivity)
            * (1.0 - albedo)
'''
if old not in s:
    raise SystemExit('missing calibration ASR anchor')
p.write_text(s.replace(old, new, 1))
