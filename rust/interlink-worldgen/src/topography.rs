use crate::{
    derive_stage_seed, GeodesicTopology, GeologicalBoundaryRegime, InheritedBoundarySet,
    InheritedPhysicalState, PlanetPhysicalParameters, StageIdentity, SubductionPolarity,
    WorldgenError,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub const TOPOGRAPHY_STAGE_ID: &str = "terrain:initial-topography";
pub const TOPOGRAPHY_STAGE_VERSION: u32 = 1;
const TOPOGRAPHY_NAMESPACE: &str = "terrain:structure:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const DISTANCE_EPSILON_M: f64 = 1.0e-6;
const CRUST_OCEANIC: u8 = 1;
const CRUST_TRANSITIONAL: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopographyParameters {
    pub isostatic_scale: f64,
    pub oceanic_subsidence_100_myr_m: f64,
    pub inherited_orogeny_scale_m: f64,
    pub collision_uplift_scale_m: f64,
    pub collision_width_m: f64,
    pub ridge_uplift_scale_m: f64,
    pub ridge_width_m: f64,
    pub rift_subsidence_scale_m: f64,
    pub rift_width_m: f64,
    pub basin_subsidence_scale_m: f64,
    pub trench_depth_scale_m: f64,
    pub trench_width_m: f64,
    pub arc_uplift_scale_m: f64,
    pub arc_peak_offset_m: f64,
    pub arc_width_m: f64,
    pub mantle_dynamic_scale_m: f64,
    pub mechanical_filter_iterations: u8,
    pub mechanical_filter_min_lambda: f64,
    pub mechanical_filter_max_lambda: f64,
}

impl Default for TopographyParameters {
    fn default() -> Self {
        Self {
            isostatic_scale: 1.0,
            oceanic_subsidence_100_myr_m: 2_900.0,
            inherited_orogeny_scale_m: 1_900.0,
            collision_uplift_scale_m: 3_600.0,
            collision_width_m: 850_000.0,
            ridge_uplift_scale_m: 2_000.0,
            ridge_width_m: 600_000.0,
            rift_subsidence_scale_m: 1_050.0,
            rift_width_m: 450_000.0,
            basin_subsidence_scale_m: 1_350.0,
            trench_depth_scale_m: 5_600.0,
            trench_width_m: 180_000.0,
            arc_uplift_scale_m: 2_300.0,
            arc_peak_offset_m: 220_000.0,
            arc_width_m: 260_000.0,
            mantle_dynamic_scale_m: 1_000.0,
            mechanical_filter_iterations: 4,
            mechanical_filter_min_lambda: 0.08,
            mechanical_filter_max_lambda: 0.34,
        }
    }
}

impl TopographyParameters {
    pub fn validate(&self) -> Result<(), &'static str> {
        let positive = [
            self.isostatic_scale,
            self.oceanic_subsidence_100_myr_m,
            self.inherited_orogeny_scale_m,
            self.collision_uplift_scale_m,
            self.collision_width_m,
            self.ridge_uplift_scale_m,
            self.ridge_width_m,
            self.rift_subsidence_scale_m,
            self.rift_width_m,
            self.basin_subsidence_scale_m,
            self.trench_depth_scale_m,
            self.trench_width_m,
            self.arc_uplift_scale_m,
            self.arc_peak_offset_m,
            self.arc_width_m,
            self.mantle_dynamic_scale_m,
        ];
        if positive
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err("topography physical scales must be finite and positive");
        }
        if self.mechanical_filter_iterations > 32 {
            return Err("topography mechanical filter iterations exceed supported bound");
        }
        if !self.mechanical_filter_min_lambda.is_finite()
            || !self.mechanical_filter_max_lambda.is_finite()
            || self.mechanical_filter_min_lambda < 0.0
            || self.mechanical_filter_max_lambda > 0.5
            || self.mechanical_filter_min_lambda > self.mechanical_filter_max_lambda
        {
            return Err("topography mechanical filter lambda bounds are invalid");
        }
        Ok(())
    }

    pub fn parameter_hash(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        for value in [
            self.isostatic_scale,
            self.oceanic_subsidence_100_myr_m,
            self.inherited_orogeny_scale_m,
            self.collision_uplift_scale_m,
            self.collision_width_m,
            self.ridge_uplift_scale_m,
            self.ridge_width_m,
            self.rift_subsidence_scale_m,
            self.rift_width_m,
            self.basin_subsidence_scale_m,
            self.trench_depth_scale_m,
            self.trench_width_m,
            self.arc_uplift_scale_m,
            self.arc_peak_offset_m,
            self.arc_width_m,
            self.mantle_dynamic_scale_m,
            self.mechanical_filter_min_lambda,
            self.mechanical_filter_max_lambda,
        ] {
            hash = fnv_update(hash, &value.to_bits().to_le_bytes());
        }
        fnv_update(hash, &[self.mechanical_filter_iterations])
    }

    pub fn parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.parameter_hash())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopographyRequest {
    pub seed: String,
    pub parameters: TopographyParameters,
}

impl TopographyRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self {
            seed: seed.into(),
            parameters: TopographyParameters::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopographyMetrics {
    pub sample_count: u32,
    pub minimum_solid_elevation_m: f64,
    pub maximum_solid_elevation_m: f64,
    pub mean_solid_elevation_m: f64,
    pub p05_solid_elevation_m: f64,
    pub median_solid_elevation_m: f64,
    pub p95_solid_elevation_m: f64,
    pub sea_level_m: Option<f64>,
    pub land_area_fraction: f64,
    pub ocean_area_fraction: f64,
    pub mean_land_elevation_m: f64,
    pub mean_water_depth_m: f64,
    pub maximum_water_depth_m: f64,
    pub target_water_volume_m3: f64,
    pub solved_water_volume_m3: f64,
    pub water_volume_relative_error: f64,
    pub clamped_sample_count: u32,
    pub parameter_hash: u64,
    pub topography_hash: u64,
}

impl TopographyMetrics {
    pub fn topography_hash_hex(&self) -> String {
        format!("{:016x}", self.topography_hash)
    }
    pub fn parameter_hash_hex(&self) -> String {
        format!("{:016x}", self.parameter_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopographyState {
    pub stage: StageIdentity,
    pub metrics: TopographyMetrics,
    pub isostatic_elevation_m: Vec<f32>,
    pub thermal_elevation_m: Vec<f32>,
    pub orogenic_elevation_m: Vec<f32>,
    pub ridge_elevation_m: Vec<f32>,
    pub rift_basin_elevation_m: Vec<f32>,
    pub trench_elevation_m: Vec<f32>,
    pub arc_elevation_m: Vec<f32>,
    pub mantle_dynamic_elevation_m: Vec<f32>,
    pub solid_elevation_m: Vec<f32>,
    pub elevation_above_sea_level_m: Vec<f32>,
    pub water_depth_m: Vec<f32>,
    pub submerged_mask: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
struct QueueEntry {
    distance_m: f64,
    source: u32,
    sample: u32,
}

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.distance_m.to_bits() == other.distance_m.to_bits()
            && self.source == other.source
            && self.sample == other.sample
    }
}
impl Eq for QueueEntry {}
impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance_m
            .total_cmp(&self.distance_m)
            .then_with(|| other.source.cmp(&self.source))
            .then_with(|| other.sample.cmp(&self.sample))
    }
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn gaussian(distance_m: f64, width_m: f64) -> f64 {
    if !distance_m.is_finite() {
        return 0.0;
    }
    let x = distance_m / width_m.max(1.0);
    (-x * x).exp()
}

fn offset_gaussian(distance_m: f64, center_m: f64, width_m: f64) -> f64 {
    if !distance_m.is_finite() {
        return 0.0;
    }
    let x = (distance_m - center_m) / width_m.max(1.0);
    (-x * x).exp()
}

fn validate_inputs(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    request: &TopographyRequest,
) -> Result<(), WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    request
        .parameters
        .validate()
        .map_err(WorldgenError::InvalidTopography)?;
    let count = topology.metrics().sample_count as usize;
    let lengths = [
        inherited.plate_ids.len(),
        inherited.crust_kind.len(),
        inherited.crust_age_myr.len(),
        inherited.crust_thickness_km.len(),
        inherited.crust_density_kg_per_m3.len(),
        inherited.orogenic_history.len(),
        inherited.rift_history.len(),
        inherited.ridge_history.len(),
        inherited.trench_history.len(),
        inherited.volcanic_arc_history.len(),
        inherited.subsidence_history.len(),
        inherited.basin_potential.len(),
        inherited.strength_index.len(),
        inherited.weakness_index.len(),
        inherited.effective_elastic_thickness_km.len(),
        inherited.mantle_dynamic_support_index.len(),
        inherited.structural_fabric_strength.len(),
        inherited.kinematic_domain_ids.len(),
    ];
    if lengths.iter().any(|len| *len != count) {
        return Err(WorldgenError::InvalidTopography(
            "WG-4 inherited physical fields are not aligned to the fine topology",
        ));
    }
    if boundaries.boundaries.iter().any(|edge| {
        edge.sample_a as usize >= count
            || edge.sample_b as usize >= count
            || edge.plate_a != inherited.plate_ids[edge.sample_a as usize]
            || edge.plate_b != inherited.plate_ids[edge.sample_b as usize]
    }) {
        return Err(WorldgenError::InvalidTopography(
            "WG-4 boundary provenance is not aligned to inherited fine ownership",
        ));
    }
    Ok(())
}

fn nearest_sources(
    topology: &GeodesicTopology,
    source_strength: &[f64],
    source_domains: Option<&[u16]>,
    radius_m: f64,
) -> (Vec<f64>, Vec<u32>) {
    let count = topology.metrics().sample_count as usize;
    let mut distances = vec![f64::INFINITY; count];
    let mut sources = vec![u32::MAX; count];
    let mut queue = BinaryHeap::new();

    for sample in 0..count {
        if source_strength[sample] > 0.0 {
            distances[sample] = 0.0;
            sources[sample] = sample as u32;
            queue.push(QueueEntry {
                distance_m: 0.0,
                source: sample as u32,
                sample: sample as u32,
            });
        }
    }

    while let Some(entry) = queue.pop() {
        let index = entry.sample as usize;
        if entry.distance_m > distances[index] + DISTANCE_EPSILON_M
            || ((entry.distance_m - distances[index]).abs() <= DISTANCE_EPSILON_M
                && entry.source != sources[index])
        {
            continue;
        }
        let source_domain = source_domains.map(|domains| domains[entry.source as usize]);
        for (neighbor, arc) in topology
            .neighbors_of(entry.sample)
            .iter()
            .zip(topology.neighbor_arc_lengths_of(entry.sample).iter())
        {
            if let (Some(domains), Some(domain)) = (source_domains, source_domain) {
                if domains[*neighbor as usize] != domain {
                    continue;
                }
            }
            let candidate = entry.distance_m + *arc * radius_m;
            let target = *neighbor as usize;
            let better = candidate + DISTANCE_EPSILON_M < distances[target]
                || ((candidate - distances[target]).abs() <= DISTANCE_EPSILON_M
                    && entry.source < sources[target]);
            if better {
                distances[target] = candidate;
                sources[target] = entry.source;
                queue.push(QueueEntry {
                    distance_m: candidate,
                    source: entry.source,
                    sample: *neighbor,
                });
            }
        }
    }

    (distances, sources)
}

fn boundary_source_fields(
    count: usize,
    inherited: &InheritedPhysicalState,
    boundaries: &InheritedBoundarySet,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut collision = vec![0.0_f64; count];
    let mut ridge = vec![0.0_f64; count];
    let mut rift = vec![0.0_f64; count];
    let mut trench = vec![0.0_f64; count];
    let mut arc = vec![0.0_f64; count];

    for edge in &boundaries.boundaries {
        let convergence = clamp01(edge.normal_rate_m_per_year.abs() / 0.08);
        let divergence = clamp01(edge.normal_rate_m_per_year.abs() / 0.06);
        let a = edge.sample_a as usize;
        let b = edge.sample_b as usize;
        match edge.geological_regime {
            GeologicalBoundaryRegime::ContinentalCollision => {
                collision[a] = collision[a].max(0.35 + 0.65 * convergence);
                collision[b] = collision[b].max(0.35 + 0.65 * convergence);
            }
            GeologicalBoundaryRegime::OceanicRidge
            | GeologicalBoundaryRegime::TransitionalDivergence => {
                ridge[a] = ridge[a].max(0.35 + 0.65 * divergence);
                ridge[b] = ridge[b].max(0.35 + 0.65 * divergence);
            }
            GeologicalBoundaryRegime::ContinentalRift => {
                rift[a] = rift[a].max(0.35 + 0.65 * divergence);
                rift[b] = rift[b].max(0.35 + 0.65 * divergence);
            }
            GeologicalBoundaryRegime::OceanicSubduction
            | GeologicalBoundaryRegime::OceanContinentSubduction => {
                let strength = 0.35 + 0.65 * convergence;
                match edge.subduction_polarity {
                    SubductionPolarity::PlateA => {
                        trench[a] = trench[a].max(strength);
                        arc[b] = arc[b].max(strength);
                    }
                    SubductionPolarity::PlateB => {
                        trench[b] = trench[b].max(strength);
                        arc[a] = arc[a].max(strength);
                    }
                    SubductionPolarity::None => {}
                }
            }
            GeologicalBoundaryRegime::Transform => {}
        }
    }

    for i in 0..count {
        collision[i] *= 0.65 + 0.35 * f64::from(inherited.orogenic_history[i]);
        ridge[i] *= 0.65 + 0.35 * f64::from(inherited.ridge_history[i]);
        rift[i] *= 0.65 + 0.35 * f64::from(inherited.rift_history[i]);
        trench[i] *= 0.55 + 0.45 * f64::from(inherited.trench_history[i]);
        arc[i] *= 0.55 + 0.45 * f64::from(inherited.volcanic_arc_history[i]);
    }

    (collision, ridge, rift, trench, arc)
}

fn mechanically_filter(
    topology: &GeodesicTopology,
    raw: &[f64],
    inherited: &InheritedPhysicalState,
    parameters: TopographyParameters,
) -> Vec<f64> {
    let mut current = raw.to_vec();
    let mut next = vec![0.0; raw.len()];
    for _ in 0..parameters.mechanical_filter_iterations {
        for sample in 0..raw.len() {
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;
            let start = topology.neighbor_offsets()[sample] as usize;
            let end = topology.neighbor_offsets()[sample + 1] as usize;
            for cursor in start..end {
                let neighbor = topology.neighbor_indices()[cursor] as usize;
                let center = topology.neighbor_center_arc_lengths_rad_values()[cursor].max(1.0e-12);
                let interface = topology.neighbor_interface_arc_lengths_rad_values()[cursor];
                let weight = interface / center;
                weighted_sum += current[neighbor] * weight;
                weight_sum += weight;
            }
            let neighbor_mean = if weight_sum > 0.0 {
                weighted_sum / weight_sum
            } else {
                current[sample]
            };
            let te = ((f64::from(inherited.effective_elastic_thickness_km[sample]) - 4.0) / 82.0)
                .clamp(0.0, 1.0);
            let weakness = f64::from(inherited.weakness_index[sample]).clamp(0.0, 1.0);
            let fabric = f64::from(inherited.structural_fabric_strength[sample]).clamp(0.0, 1.0);
            let mut lambda = parameters.mechanical_filter_min_lambda
                + (parameters.mechanical_filter_max_lambda
                    - parameters.mechanical_filter_min_lambda)
                    * te;
            lambda *= (1.0 - 0.55 * weakness) * (1.0 - 0.20 * fabric);
            next[sample] = current[sample] + lambda * (neighbor_mean - current[sample]);
        }
        std::mem::swap(&mut current, &mut next);
    }
    current
}

fn area_weighted_mean(values: &[f64], areas_sr: &[f64]) -> f64 {
    let mut sum = 0.0;
    let mut area = 0.0;
    for (value, cell_area) in values.iter().zip(areas_sr.iter()) {
        sum += *value * *cell_area;
        area += *cell_area;
    }
    if area > 0.0 {
        sum / area
    } else {
        0.0
    }
}

fn area_weighted_quantile(values: &[f64], areas_sr: &[f64], q: f64) -> f64 {
    let mut pairs = values
        .iter()
        .copied()
        .zip(areas_sr.iter().copied())
        .collect::<Vec<_>>();
    pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
    let total = pairs.iter().map(|(_, area)| *area).sum::<f64>();
    let target = total * q.clamp(0.0, 1.0);
    let mut cumulative = 0.0;
    for (value, area) in pairs {
        cumulative += area;
        if cumulative >= target {
            return value;
        }
    }
    values.last().copied().unwrap_or(0.0)
}

fn water_volume_at_level(
    elevation_m: &[f64],
    areas_sr: &[f64],
    radius_m: f64,
    sea_level_m: f64,
) -> f64 {
    elevation_m
        .iter()
        .zip(areas_sr.iter())
        .map(|(elevation, area_sr)| {
            (sea_level_m - *elevation).max(0.0) * *area_sr * radius_m * radius_m
        })
        .sum()
}

fn solve_sea_level(
    elevation_m: &[f64],
    areas_sr: &[f64],
    planet: PlanetPhysicalParameters,
) -> (Option<f64>, f64, f64) {
    let target = planet.surface_water_volume_m3();
    if target == 0.0 {
        return (None, 0.0, 0.0);
    }
    let minimum = elevation_m.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = elevation_m
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let mut low = minimum - 1.0;
    let mut high = maximum + planet.equivalent_global_water_depth_m() + 1.0;
    while water_volume_at_level(elevation_m, areas_sr, planet.radius_m, high) < target {
        high += (high - low).max(1_000.0);
    }
    for _ in 0..96 {
        let middle = (low + high) * 0.5;
        let volume = water_volume_at_level(elevation_m, areas_sr, planet.radius_m, middle);
        if volume < target {
            low = middle;
        } else {
            high = middle;
        }
    }
    let sea_level = (low + high) * 0.5;
    let solved = water_volume_at_level(elevation_m, areas_sr, planet.radius_m, sea_level);
    let error = ((solved - target) / target).abs();
    (Some(sea_level), solved, error)
}

pub fn generate_initial_topography(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    request: &TopographyRequest,
) -> Result<TopographyState, WorldgenError> {
    validate_inputs(topology, inherited, boundaries, planet, request)?;
    let count = topology.metrics().sample_count as usize;
    let p = request.parameters;
    let stage_seed = derive_stage_seed(&request.seed, TOPOGRAPHY_NAMESPACE);

    let (collision_sources, ridge_sources, rift_sources, trench_sources, arc_sources) =
        boundary_source_fields(count, inherited, boundaries);
    let (collision_distance, collision_source) =
        nearest_sources(topology, &collision_sources, None, planet.radius_m);
    let (ridge_distance, ridge_source) =
        nearest_sources(topology, &ridge_sources, None, planet.radius_m);
    let (rift_distance, rift_source) =
        nearest_sources(topology, &rift_sources, None, planet.radius_m);
    let (trench_distance, trench_source) = nearest_sources(
        topology,
        &trench_sources,
        Some(&inherited.plate_ids),
        planet.radius_m,
    );
    let (arc_distance, arc_source) = nearest_sources(
        topology,
        &arc_sources,
        Some(&inherited.plate_ids),
        planet.radius_m,
    );

    let mut isostatic = vec![0.0_f64; count];
    let mut thermal = vec![0.0_f64; count];
    let mut orogenic = vec![0.0_f64; count];
    let mut ridge = vec![0.0_f64; count];
    let mut rift_basin = vec![0.0_f64; count];
    let mut trench = vec![0.0_f64; count];
    let mut arc = vec![0.0_f64; count];
    let mut mantle = vec![0.0_f64; count];

    for i in 0..count {
        let thickness_m = f64::from(inherited.crust_thickness_km[i]) * 1_000.0;
        let crust_density = f64::from(inherited.crust_density_kg_per_m3[i]);
        let mantle_density = planet.isostatic_mantle_density_kg_per_m3;
        isostatic[i] =
            thickness_m * (mantle_density - crust_density) / mantle_density * p.isostatic_scale;

        let ocean_weight = match inherited.crust_kind[i] {
            CRUST_OCEANIC => 1.0,
            CRUST_TRANSITIONAL => 0.45,
            _ => 0.0,
        };
        thermal[i] = -p.oceanic_subsidence_100_myr_m
            * (f64::from(inherited.crust_age_myr[i]).max(0.0) / 100.0)
                .sqrt()
                .min(1.55)
            * ocean_weight;

        let structural_focus =
            1.0 + 0.25 * f64::from(inherited.structural_fabric_strength[i]).clamp(0.0, 1.0);
        let collision_kernel = if collision_source[i] == u32::MAX {
            0.0
        } else {
            gaussian(collision_distance[i], p.collision_width_m)
                * collision_sources[collision_source[i] as usize]
        };
        orogenic[i] = p.inherited_orogeny_scale_m * f64::from(inherited.orogenic_history[i])
            + p.collision_uplift_scale_m * collision_kernel * structural_focus;

        let ridge_kernel = if ridge_source[i] == u32::MAX {
            0.0
        } else {
            gaussian(ridge_distance[i], p.ridge_width_m) * ridge_sources[ridge_source[i] as usize]
        };
        ridge[i] =
            p.ridge_uplift_scale_m * ridge_kernel + 500.0 * f64::from(inherited.ridge_history[i]);

        let rift_kernel = if rift_source[i] == u32::MAX {
            0.0
        } else {
            gaussian(rift_distance[i], p.rift_width_m) * rift_sources[rift_source[i] as usize]
        };
        rift_basin[i] = -(p.rift_subsidence_scale_m
            * (rift_kernel * structural_focus + 0.55 * f64::from(inherited.rift_history[i]))
            + p.basin_subsidence_scale_m
                * (0.55 * f64::from(inherited.basin_potential[i])
                    + 0.45 * f64::from(inherited.subsidence_history[i])));

        let trench_kernel = if trench_source[i] == u32::MAX {
            0.0
        } else {
            gaussian(trench_distance[i], p.trench_width_m)
                * trench_sources[trench_source[i] as usize]
        };
        trench[i] = -p.trench_depth_scale_m
            * trench_kernel
            * (0.70 + 0.30 * f64::from(inherited.trench_history[i]));

        let arc_kernel = if arc_source[i] == u32::MAX {
            0.0
        } else {
            offset_gaussian(arc_distance[i], p.arc_peak_offset_m, p.arc_width_m)
                * arc_sources[arc_source[i] as usize]
        };
        arc[i] = p.arc_uplift_scale_m
            * arc_kernel
            * (0.65 + 0.35 * f64::from(inherited.volcanic_arc_history[i]));

        mantle[i] = p.mantle_dynamic_scale_m * f64::from(inherited.mantle_dynamic_support_index[i]);
    }

    let mut raw = vec![0.0_f64; count];
    for i in 0..count {
        raw[i] = isostatic[i]
            + thermal[i]
            + orogenic[i]
            + ridge[i]
            + rift_basin[i]
            + trench[i]
            + arc[i]
            + mantle[i];
    }
    let mut solid = mechanically_filter(topology, &raw, inherited, p);

    let datum = area_weighted_mean(&solid, topology.dual_area_steradians());
    for value in &mut solid {
        *value -= datum;
    }

    let safety_min = -20_000.0;
    let safety_max = 15_000.0;
    let mut clamped_sample_count = 0_u32;
    for value in &mut solid {
        let clamped = value.clamp(safety_min, safety_max);
        if clamped.to_bits() != value.to_bits() {
            clamped_sample_count += 1;
            *value = clamped;
        }
    }

    let (sea_level, solved_water_volume_m3, water_volume_relative_error) =
        solve_sea_level(&solid, topology.dual_area_steradians(), planet);

    let mut above_sea = vec![0.0_f32; count];
    let mut water_depth = vec![0.0_f32; count];
    let mut submerged = vec![0_u8; count];
    let mut land_area = 0.0;
    let mut ocean_area = 0.0;
    let mut land_elevation_area_sum = 0.0;
    let mut water_depth_area_sum = 0.0;
    let mut maximum_water_depth_m = 0.0_f64;
    for i in 0..count {
        let area = topology.dual_area_steradians()[i];
        if let Some(level) = sea_level {
            let relative = solid[i] - level;
            above_sea[i] = relative as f32;
            if relative < 0.0 {
                let depth = -relative;
                water_depth[i] = depth as f32;
                submerged[i] = 1;
                ocean_area += area;
                water_depth_area_sum += depth * area;
                maximum_water_depth_m = maximum_water_depth_m.max(depth);
            } else {
                land_area += area;
                land_elevation_area_sum += relative * area;
            }
        } else {
            above_sea[i] = solid[i] as f32;
            land_area += area;
            land_elevation_area_sum += solid[i] * area;
        }
    }
    let total_area = land_area + ocean_area;
    let land_area_fraction = if total_area > 0.0 {
        land_area / total_area
    } else {
        1.0
    };
    let ocean_area_fraction = if total_area > 0.0 {
        ocean_area / total_area
    } else {
        0.0
    };
    let mean_land_elevation_m = if land_area > 0.0 {
        land_elevation_area_sum / land_area
    } else {
        0.0
    };
    let mean_water_depth_m = if ocean_area > 0.0 {
        water_depth_area_sum / ocean_area
    } else {
        0.0
    };

    let minimum_solid_elevation_m = solid.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum_solid_elevation_m = solid.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mean_solid_elevation_m = area_weighted_mean(&solid, topology.dual_area_steradians());
    let p05_solid_elevation_m =
        area_weighted_quantile(&solid, topology.dual_area_steradians(), 0.05);
    let median_solid_elevation_m =
        area_weighted_quantile(&solid, topology.dual_area_steradians(), 0.50);
    let p95_solid_elevation_m =
        area_weighted_quantile(&solid, topology.dual_area_steradians(), 0.95);

    let model_parameter_hash = p.parameter_hash();
    let mut topography_hash = FNV_OFFSET_BASIS;
    topography_hash = fnv_update(topography_hash, TOPOGRAPHY_STAGE_ID.as_bytes());
    topography_hash = fnv_update(topography_hash, &TOPOGRAPHY_STAGE_VERSION.to_le_bytes());
    topography_hash = fnv_update(topography_hash, &stage_seed.to_le_bytes());
    topography_hash = fnv_update(topography_hash, &model_parameter_hash.to_le_bytes());
    topography_hash = fnv_update(topography_hash, &planet.parameter_hash().to_le_bytes());
    topography_hash = fnv_update(topography_hash, &inherited.inheritance_hash.to_le_bytes());
    topography_hash = fnv_update(topography_hash, &boundaries.boundary_hash.to_le_bytes());
    for value in &solid {
        topography_hash = fnv_update(topography_hash, &value.to_bits().to_le_bytes());
    }
    match sea_level {
        Some(level) => {
            topography_hash = fnv_update(topography_hash, &[1]);
            topography_hash = fnv_update(topography_hash, &level.to_bits().to_le_bytes());
        }
        None => {
            topography_hash = fnv_update(topography_hash, &[0]);
        }
    }
    for value in &water_depth {
        topography_hash = fnv_update(topography_hash, &value.to_bits().to_le_bytes());
    }

    let metrics = TopographyMetrics {
        sample_count: count as u32,
        minimum_solid_elevation_m,
        maximum_solid_elevation_m,
        mean_solid_elevation_m,
        p05_solid_elevation_m,
        median_solid_elevation_m,
        p95_solid_elevation_m,
        sea_level_m: sea_level,
        land_area_fraction,
        ocean_area_fraction,
        mean_land_elevation_m,
        mean_water_depth_m,
        maximum_water_depth_m,
        target_water_volume_m3: planet.surface_water_volume_m3(),
        solved_water_volume_m3,
        water_volume_relative_error,
        clamped_sample_count,
        parameter_hash: model_parameter_hash,
        topography_hash,
    };

    Ok(TopographyState {
        stage: StageIdentity {
            id: TOPOGRAPHY_STAGE_ID,
            version: TOPOGRAPHY_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics,
        isostatic_elevation_m: isostatic.into_iter().map(|value| value as f32).collect(),
        thermal_elevation_m: thermal.into_iter().map(|value| value as f32).collect(),
        orogenic_elevation_m: orogenic.into_iter().map(|value| value as f32).collect(),
        ridge_elevation_m: ridge.into_iter().map(|value| value as f32).collect(),
        rift_basin_elevation_m: rift_basin.into_iter().map(|value| value as f32).collect(),
        trench_elevation_m: trench.into_iter().map(|value| value as f32).collect(),
        arc_elevation_m: arc.into_iter().map(|value| value as f32).collect(),
        mantle_dynamic_elevation_m: mantle.into_iter().map(|value| value as f32).collect(),
        solid_elevation_m: solid.into_iter().map(|value| value as f32).collect(),
        elevation_above_sea_level_m: above_sea,
        water_depth_m: water_depth,
        submerged_mask: submerged,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_icosphere, generate_crust_and_history, generate_lithosphere, generate_tectonics,
        inherit_boundary_interfaces, inherit_physical_state, GeologyRequest, LithosphereRequest,
        TectonicsRequest,
    };

    fn generated(seed: &str, water_mass_kg: Option<f64>) -> TopographyState {
        let mut planet = PlanetPhysicalParameters::earthlike_reference();
        if let Some(value) = water_mass_kg {
            planet.surface_water_mass_kg = value;
        }
        let coarse = build_icosphere(3).unwrap();
        let fine = build_icosphere(4).unwrap();
        let tectonics =
            generate_tectonics(&coarse, &TectonicsRequest::new(seed, 12), planet).unwrap();
        let geology =
            generate_crust_and_history(&coarse, &tectonics, &GeologyRequest::new(seed), planet)
                .unwrap();
        let lithosphere = generate_lithosphere(
            &coarse,
            &tectonics,
            &geology,
            &LithosphereRequest::new(seed),
        )
        .unwrap();
        let inherited =
            inherit_physical_state(&fine, 3, &tectonics, &geology, &lithosphere, planet).unwrap();
        let boundaries =
            inherit_boundary_interfaces(&coarse, &fine, &tectonics, &geology, &inherited.plate_ids)
                .unwrap();
        generate_initial_topography(
            &fine,
            &inherited,
            &boundaries,
            planet,
            &TopographyRequest::new(seed),
        )
        .unwrap()
    }

    #[test]
    fn topography_is_deterministic_finite_and_sample_aligned() {
        let a = generated("wg4-determinism", None);
        let b = generated("wg4-determinism", None);
        assert_eq!(a.metrics.topography_hash, b.metrics.topography_hash);
        assert_eq!(a.solid_elevation_m, b.solid_elevation_m);
        assert_eq!(a.metrics.sample_count as usize, a.solid_elevation_m.len());
        assert!(a.solid_elevation_m.iter().all(|value| value.is_finite()));
        assert!(a
            .water_depth_m
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0));
        assert!(a.metrics.minimum_solid_elevation_m < a.metrics.maximum_solid_elevation_m);
    }

    #[test]
    fn earthlike_water_inventory_is_conserved_by_sea_level_solve() {
        let topography = generated("wg4-water", None);
        assert!(topography.metrics.sea_level_m.is_some());
        assert!(topography.metrics.water_volume_relative_error < 1.0e-10);
        assert!(topography.metrics.ocean_area_fraction > 0.0);
        assert!(topography.metrics.land_area_fraction > 0.0);
        assert!(
            (topography.metrics.land_area_fraction + topography.metrics.ocean_area_fraction - 1.0)
                .abs()
                < 1.0e-12
        );
    }

    #[test]
    fn dry_profile_has_no_fictitious_ocean() {
        let topography = generated("wg4-dry", Some(0.0));
        assert_eq!(topography.metrics.sea_level_m, None);
        assert_eq!(topography.metrics.ocean_area_fraction, 0.0);
        assert!(topography.water_depth_m.iter().all(|value| *value == 0.0));
        assert!(topography.submerged_mask.iter().all(|value| *value == 0));
    }

    #[test]
    fn inherited_geology_produces_nontrivial_tectonic_relief_components() {
        let topography = generated("wg4-components", None);
        let max_abs = |values: &[f32]| {
            values
                .iter()
                .map(|value| value.abs())
                .fold(0.0_f32, f32::max)
        };
        assert!(max_abs(&topography.isostatic_elevation_m) > 100.0);
        assert!(max_abs(&topography.thermal_elevation_m) > 100.0);
        assert!(
            max_abs(&topography.orogenic_elevation_m)
                + max_abs(&topography.ridge_elevation_m)
                + max_abs(&topography.rift_basin_elevation_m)
                > 100.0
        );
        assert!(topography.metrics.clamped_sample_count < topography.metrics.sample_count / 20);
    }
}
