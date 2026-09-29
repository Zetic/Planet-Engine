use crate::{
    build_refinement_map, derive_stage_seed, generate_historical_lithosphere,
    refine_categorical_u16, refine_categorical_u8, refine_scalar_f32_with_domains, CrustKind,
    CrustalModel, GeodesicTopology, GeologicalBoundary, GeologicalBoundaryRegime, GeologyMetrics,
    HistoricalEventKind, HistoricalLithosphereModel, HistoricalLithosphereRequest,
    PlanetPhysicalParameters, PlanetTopology, PlateBoundaryEdge, PlateBoundaryKind,
    PlateScaleClass, PlateSummary, RefinementMap, StageIdentity, SubductionPolarity,
    TectonicMetrics, TectonicModel, TectonicPlate, WorldgenError, GEOLOGY_STAGE_ID,
    GEOLOGY_STAGE_VERSION, TECTONICS_STAGE_ID, TECTONICS_STAGE_VERSION,
};
use std::f64::consts::PI;

const MODERN_TECTONICS_NAMESPACE: &str =
    "worldgen:geology:historical-lithosphere:modern-tectonics:v2";
const HISTORICAL_GEOLOGY_NAMESPACE: &str =
    "worldgen:geology:historical-lithosphere:crust-projection:v2";
const HISTORICAL_PROPERTIES_NAMESPACE: &str =
    "worldgen:geology:historical-lithosphere:crust-properties:v2";
const HISTORICAL_HISTORY_NAMESPACE: &str =
    "worldgen:geology:historical-lithosphere:event-raster:v1";
pub const HISTORICAL_INHERITANCE_STAGE_ID: &str = "geology:historical-inheritance";
pub const HISTORICAL_INHERITANCE_STAGE_VERSION: u32 = 1;
const OCEANIC_PROVINCE_BIT: u16 = 0x8000;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const MILLION_YEARS: f64 = 1_000_000.0;

#[derive(Clone, Debug, PartialEq)]
pub struct HistoricalFrontend {
    pub historical: HistoricalLithosphereModel,
    pub tectonics: TectonicModel,
    pub geology: CrustalModel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InheritedHistoricalIdentity {
    pub stage: StageIdentity,
    pub map: RefinementMap,
    pub origin_plate_ids: Vec<u16>,
    pub fragment_ids: Vec<u16>,
    pub current_plate_ids: Vec<u16>,
    pub crust_kind: Vec<u8>,
    pub crust_birth_age_myr: Vec<f32>,
    pub identity_hash: u64,
}

impl InheritedHistoricalIdentity {
    pub fn identity_hash_hex(&self) -> String {
        format!("{:016x}", self.identity_hash)
    }
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn unit_random(value: u64) -> f64 {
    ((mix64(value) >> 11) as f64) * (1.0 / 9_007_199_254_740_992.0)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(value: [f64; 3], factor: f64) -> [f64; 3] {
    [value[0] * factor, value[1] * factor, value[2] * factor]
}

fn norm(value: [f64; 3]) -> f64 {
    dot(value, value).sqrt()
}

fn normalize_or(value: [f64; 3], fallback: [f64; 3]) -> [f64; 3] {
    let magnitude = norm(value);
    if magnitude > 1.0e-15 {
        scale(value, 1.0 / magnitude)
    } else {
        fallback
    }
}

fn arc_radians(a: [f64; 3], b: [f64; 3]) -> f64 {
    dot(a, b).clamp(-1.0, 1.0).acos()
}

fn velocity_m_per_year(
    angular_velocity_rad_per_myr: [f64; 3],
    position: [f64; 3],
    planet_radius_m: f64,
) -> [f64; 3] {
    scale(
        cross(angular_velocity_rad_per_myr, position),
        planet_radius_m / MILLION_YEARS,
    )
}

fn classify_boundary(normal_rate: f64, shear_rate: f64) -> PlateBoundaryKind {
    let relative_speed = normal_rate.hypot(shear_rate);
    if relative_speed <= 1.0e-12 || normal_rate.abs() < relative_speed * 0.35 {
        PlateBoundaryKind::Transform
    } else if normal_rate < 0.0 {
        PlateBoundaryKind::Convergent
    } else {
        PlateBoundaryKind::Divergent
    }
}

fn boundary_pair(edge: &PlateBoundaryEdge) -> (u16, u16) {
    if edge.plate_a < edge.plate_b {
        (edge.plate_a, edge.plate_b)
    } else {
        (edge.plate_b, edge.plate_a)
    }
}

fn normal_fraction(edge: &PlateBoundaryEdge) -> f64 {
    let speed = edge
        .normal_rate_m_per_year
        .hypot(edge.shear_rate_m_per_year);
    if speed <= 1.0e-12 {
        0.0
    } else {
        (edge.normal_rate_m_per_year.abs() / speed).clamp(0.0, 1.0)
    }
}

fn stabilize_boundary_kinds<T: PlanetTopology>(
    topology: &T,
    boundaries: &mut [PlateBoundaryEdge],
) {
    let mut incident = vec![Vec::<usize>::new(); topology.sample_count() as usize];
    for (index, edge) in boundaries.iter().enumerate() {
        incident[edge.sample_a as usize].push(index);
        incident[edge.sample_b as usize].push(index);
    }

    // Two deterministic local passes are enough to remove one-edge regime chatter while
    // preserving genuine transitions where the velocity normal component remains decisive.
    for _ in 0..2 {
        let previous = boundaries
            .iter()
            .map(|edge| edge.kind)
            .collect::<Vec<_>>();
        for edge_index in 0..boundaries.len() {
            let edge = &boundaries[edge_index];
            let pair = boundary_pair(edge);
            let mut convergent = 0_u8;
            let mut divergent = 0_u8;
            let mut transform = 0_u8;

            for sample in [edge.sample_a, edge.sample_b] {
                for neighbor_index in &incident[sample as usize] {
                    if *neighbor_index == edge_index
                        || boundary_pair(&boundaries[*neighbor_index]) != pair
                    {
                        continue;
                    }
                    match previous[*neighbor_index] {
                        PlateBoundaryKind::Convergent => convergent = convergent.saturating_add(1),
                        PlateBoundaryKind::Divergent => divergent = divergent.saturating_add(1),
                        PlateBoundaryKind::Transform => transform = transform.saturating_add(1),
                    }
                }
            }

            let fraction = normal_fraction(edge);
            let current = previous[edge_index];
            let signed_kind = if edge.normal_rate_m_per_year < 0.0 {
                PlateBoundaryKind::Convergent
            } else {
                PlateBoundaryKind::Divergent
            };
            let signed_support = match signed_kind {
                PlateBoundaryKind::Convergent => convergent,
                PlateBoundaryKind::Divergent => divergent,
                PlateBoundaryKind::Transform => 0,
            };
            let opposite_support = match signed_kind {
                PlateBoundaryKind::Convergent => divergent,
                PlateBoundaryKind::Divergent => convergent,
                PlateBoundaryKind::Transform => 0,
            };

            let next = if transform >= 2
                && transform > signed_support
                && fraction < 0.50
            {
                PlateBoundaryKind::Transform
            } else if current == PlateBoundaryKind::Transform
                && signed_support >= 2
                && signed_support > transform
                && fraction >= 0.35
            {
                // Neighbor agreement may remove a one-edge transform interruption, but it may
                // not promote a shear-dominated edge into extension or convergence below the
                // physical normal-motion threshold used by the base classifier.
                signed_kind
            } else if current != PlateBoundaryKind::Transform
                && fraction < 0.30
                && (transform > 0 || opposite_support >= 2)
            {
                // Near a normal-rate sign reversal, use a short transform transition rather than
                // alternating convergence/divergence on adjacent graph edges.
                PlateBoundaryKind::Transform
            } else {
                current
            };
            boundaries[edge_index].kind = next;
        }
    }
}

fn build_modern_boundaries<T: PlanetTopology>(
    topology: &T,
    owners: &[u16],
    plates: &[TectonicPlate],
    planet_radius_m: f64,
) -> Vec<PlateBoundaryEdge> {
    let mut boundaries = Vec::new();
    for sample_a in 0..topology.sample_count() {
        let position_a = topology.unit_position(sample_a);
        for sample_b in topology.neighbors(sample_a) {
            if *sample_b <= sample_a {
                continue;
            }
            let plate_a = owners[sample_a as usize];
            let plate_b = owners[*sample_b as usize];
            if plate_a == plate_b {
                continue;
            }
            let position_b = topology.unit_position(*sample_b);
            let midpoint = normalize_or(add(position_a, position_b), position_a);
            let normal = normalize_or(sub(position_b, position_a), position_a);
            let tangent = normalize_or(cross(midpoint, normal), [1.0, 0.0, 0.0]);
            let velocity_a = velocity_m_per_year(
                plates[plate_a as usize].angular_velocity_rad_per_myr,
                midpoint,
                planet_radius_m,
            );
            let velocity_b = velocity_m_per_year(
                plates[plate_b as usize].angular_velocity_rad_per_myr,
                midpoint,
                planet_radius_m,
            );
            let relative = sub(velocity_b, velocity_a);
            let normal_rate = dot(relative, normal);
            let shear_rate = dot(relative, tangent);
            boundaries.push(PlateBoundaryEdge {
                sample_a,
                sample_b: *sample_b,
                plate_a,
                plate_b,
                kind: classify_boundary(normal_rate, shear_rate),
                normal_rate_m_per_year: normal_rate,
                shear_rate_m_per_year: shear_rate,
            });
        }
    }
    stabilize_boundary_kinds(topology, &mut boundaries);
    boundaries
}

fn modern_tectonic_hash(
    stage_seed: u64,
    plates: &[TectonicPlate],
    owners: &[u16],
    boundaries: &[PlateBoundaryEdge],
    history_hash: u64,
) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, b"tectonics:historical-modern:v2\0");
    hash = fnv_update(hash, &stage_seed.to_le_bytes());
    hash = fnv_update(hash, &history_hash.to_le_bytes());
    for plate in plates {
        hash = fnv_update(hash, &plate.id.to_le_bytes());
        hash = fnv_update(hash, &plate.seed_sample.to_le_bytes());
        for value in plate.angular_velocity_rad_per_myr {
            hash = fnv_update(hash, &value.to_bits().to_le_bytes());
        }
        hash = fnv_update(hash, &plate.area_steradians.to_bits().to_le_bytes());
    }
    for owner in owners {
        hash = fnv_update(hash, &owner.to_le_bytes());
    }
    for boundary in boundaries {
        hash = fnv_update(hash, &boundary.sample_a.to_le_bytes());
        hash = fnv_update(hash, &boundary.sample_b.to_le_bytes());
        hash = fnv_update(hash, &[boundary.kind as u8]);
        hash = fnv_update(
            hash,
            &boundary.normal_rate_m_per_year.to_bits().to_le_bytes(),
        );
        hash = fnv_update(
            hash,
            &boundary.shear_rate_m_per_year.to_bits().to_le_bytes(),
        );
    }
    hash
}

fn minimum_seed_separation<T: PlanetTopology>(topology: &T, plates: &[TectonicPlate]) -> f64 {
    let mut minimum = f64::INFINITY;
    for left in 0..plates.len() {
        for right in (left + 1)..plates.len() {
            minimum = minimum.min(arc_radians(
                topology.unit_position(plates[left].seed_sample),
                topology.unit_position(plates[right].seed_sample),
            ));
        }
    }
    minimum
}

pub fn project_historical_modern_tectonics<T: PlanetTopology>(
    topology: &T,
    historical: &HistoricalLithosphereModel,
    seed: &str,
    planet: PlanetPhysicalParameters,
) -> Result<TectonicModel, WorldgenError> {
    planet
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    let count = topology.sample_count() as usize;
    if historical.origin_plate_ids.len() != count
        || historical.current_plate_ids.len() != count
        || historical.ancestral_tectonics.plate_ids.len() != count
    {
        return Err(WorldgenError::InvalidTectonics(
            "historical ownership does not match modern tectonic topology",
        ));
    }
    let modern_count = historical.metrics.modern_plate_count as usize;
    if modern_count == 0 {
        return Err(WorldgenError::InvalidTectonics(
            "historical frontend requires at least one modern plate",
        ));
    }

    if historical.current_plate_angular_velocities_rad_per_myr.len() != modern_count
        || historical
            .current_plate_angular_velocities_rad_per_myr
            .iter()
            .flatten()
            .any(|value| !value.is_finite())
    {
        return Err(WorldgenError::InvalidTectonics(
            "historical frontend is missing evolved modern plate kinematics",
        ));
    }

    let ancestral_count = historical.ancestral_tectonics.plates.len();
    let mut ancestry_area = vec![0.0_f64; modern_count];
    let mut contribution_area = vec![vec![0.0_f64; ancestral_count]; modern_count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let origin = historical.origin_plate_ids[index] as usize;
        let current = historical.current_plate_ids[index] as usize;
        if origin >= ancestral_count || current >= modern_count {
            return Err(WorldgenError::InvalidTectonics(
                "historical ownership contains an invalid plate id",
            ));
        }
        let area = topology.area_steradians(sample);
        ancestry_area[current] += area;
        contribution_area[current][origin] += area;
    }

    let mut representative_seed = vec![u32::MAX; modern_count];
    let mut representative_support = vec![f64::NEG_INFINITY; modern_count];
    let mut representative_pole = vec![[0.0_f64, 0.0, 1.0]; modern_count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let current = historical.current_plate_ids[index] as usize;
        let origin = historical.origin_plate_ids[index] as usize;
        let same_neighbors = topology
            .neighbors(sample)
            .iter()
            .filter(|neighbor| {
                historical.current_plate_ids[**neighbor as usize] as usize == current
            })
            .count() as f64;
        let ancestry_support =
            contribution_area[current][origin] / ancestry_area[current].max(1.0e-12);
        let support = same_neighbors + ancestry_support * 0.45;
        if support > representative_support[current]
            || (support == representative_support[current] && sample < representative_seed[current])
        {
            representative_support[current] = support;
            representative_seed[current] = sample;
            representative_pole[current] = historical.ancestral_tectonics.plates[origin].euler_pole;
        }
    }

    let mut plate_area = vec![0.0_f64; modern_count];
    for sample in 0..topology.sample_count() {
        plate_area[historical.current_plate_ids[sample as usize] as usize] +=
            topology.area_steradians(sample);
    }

    let mut plates = Vec::with_capacity(modern_count);
    for current in 0..modern_count {
        if ancestry_area[current] <= 0.0 || representative_seed[current] == u32::MAX {
            return Err(WorldgenError::InvalidTectonics(
                "historical modern projection created an empty plate",
            ));
        }
        let angular_velocity = historical.current_plate_angular_velocities_rad_per_myr[current];
        let euler_pole = normalize_or(angular_velocity, representative_pole[current]);
        plates.push(TectonicPlate {
            id: current as u16,
            seed_sample: representative_seed[current],
            euler_pole,
            angular_velocity_rad_per_myr: angular_velocity,
            area_steradians: plate_area[current],
        });
    }

    for plate in &plates {
        if historical.current_plate_ids[plate.seed_sample as usize] != plate.id {
            return Err(WorldgenError::InvalidTectonics(
                "historical modern plate seed is not owned by its projected plate",
            ));
        }
    }

    let boundaries = build_modern_boundaries(
        topology,
        &historical.current_plate_ids,
        &plates,
        planet.radius_m,
    );
    if boundaries.is_empty() {
        return Err(WorldgenError::InvalidTectonics(
            "historical modern projection produced no plate boundaries",
        ));
    }

    let mut convergent = 0_u32;
    let mut divergent = 0_u32;
    let mut transform = 0_u32;
    for boundary in &boundaries {
        match boundary.kind {
            PlateBoundaryKind::Convergent => convergent += 1,
            PlateBoundaryKind::Divergent => divergent += 1,
            PlateBoundaryKind::Transform => transform += 1,
        }
    }
    let total_area = plate_area.iter().sum::<f64>().max(1.0e-12);
    let area_fractions = plate_area
        .iter()
        .map(|area| *area / total_area)
        .collect::<Vec<_>>();
    let minimum_plate_area_fraction = area_fractions.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum_plate_area_fraction = area_fractions
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let mean_reference_speed_mm_per_year = plates
        .iter()
        .map(|plate| plate.reference_speed_mm_per_year(planet.radius_m))
        .sum::<f64>()
        / plates.len() as f64;
    let minimum_seed_separation_rad = minimum_seed_separation(topology, &plates);
    let stage_seed = derive_stage_seed(seed, MODERN_TECTONICS_NAMESPACE);
    let tectonic_hash = modern_tectonic_hash(
        stage_seed,
        &plates,
        &historical.current_plate_ids,
        &boundaries,
        historical.metrics.history_hash,
    );

    Ok(TectonicModel {
        stage: StageIdentity {
            id: TECTONICS_STAGE_ID,
            version: TECTONICS_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        plates,
        plate_ids: historical.current_plate_ids.clone(),
        boundaries,
        metrics: TectonicMetrics {
            sample_count: topology.sample_count(),
            requested_plate_count: historical.metrics.requested_plate_count,
            plate_count: historical.metrics.modern_plate_count,
            boundary_edge_count: convergent + divergent + transform,
            convergent_edge_count: convergent,
            divergent_edge_count: divergent,
            transform_edge_count: transform,
            minimum_plate_area_fraction,
            maximum_plate_area_fraction,
            mean_plate_area_fraction: 1.0 / historical.metrics.modern_plate_count as f64,
            minimum_seed_separation_rad,
            mean_reference_speed_mm_per_year,
            tectonic_hash,
        },
    })
}

fn crust_kind(value: u8) -> CrustKind {
    match value {
        value if value == CrustKind::Continental as u8 => CrustKind::Continental,
        value if value == CrustKind::Transitional as u8 => CrustKind::Transitional,
        _ => CrustKind::Oceanic,
    }
}

fn buoyancy_index(thickness_km: f64, density_kg_per_m3: f64) -> f64 {
    let density_component = (2950.0 - density_kg_per_m3) / 260.0;
    let thickness_component = (thickness_km - 20.0) / 32.0 * 0.48;
    (density_component + thickness_component).clamp(-1.0, 1.0)
}

#[derive(Clone, Debug)]
struct MaterialProperties {
    provinces: Vec<u16>,
    composite_age_myr: Vec<f32>,
    oceanic_age_myr: Vec<f32>,
    continental_basement_age_myr: Vec<f32>,
    thickness_km: Vec<f32>,
    density_kg_per_m3: Vec<f32>,
    buoyancy_index: Vec<f32>,
}

fn smooth_continental_basement_age<T: PlanetTopology>(
    topology: &T,
    historical: &HistoricalLithosphereModel,
) -> Vec<f32> {
    let mut current = historical
        .crust_birth_age_myr
        .iter()
        .enumerate()
        .map(|(sample, age)| match crust_kind(historical.crust_kind[sample]) {
            CrustKind::Continental => age.clamp(450.0, 3500.0),
            CrustKind::Transitional => age.clamp(80.0, 1200.0),
            CrustKind::Oceanic => 0.0,
        })
        .collect::<Vec<_>>();
    let mut next = current.clone();

    // Basement age is historical metadata, not a fragment paint layer. Diffuse the inherited
    // formation clock through contiguous continental material so quiet welded provenance
    // contacts cannot survive as exact age polygons. Oceanic material is excluded entirely.
    for _ in 0..10 {
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            let kind = crust_kind(historical.crust_kind[index]);
            if matches!(kind, CrustKind::Oceanic) {
                next[index] = 0.0;
                continue;
            }
            let mut sum = 0.0_f64;
            let mut weight = 0.0_f64;
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                if crust_kind(historical.crust_kind[ni]) != kind {
                    continue;
                }
                sum += f64::from(current[ni]);
                weight += 1.0;
            }
            next[index] = if weight > 0.0 {
                (0.28 * f64::from(current[index]) + 0.72 * (sum / weight)) as f32
            } else {
                current[index]
            };
        }
        std::mem::swap(&mut current, &mut next);
    }
    current
}

fn build_material_properties<T: PlanetTopology>(
    topology: &T,
    historical: &HistoricalLithosphereModel,
    property_seed: u64,
) -> MaterialProperties {
    let count = historical.crust_kind.len();
    let basement_age = smooth_continental_basement_age(topology, historical);
    let mut provinces = Vec::with_capacity(count);
    let mut composite_age = Vec::with_capacity(count);
    let mut oceanic_age = Vec::with_capacity(count);
    let mut thickness = Vec::with_capacity(count);
    let mut density = Vec::with_capacity(count);
    let mut buoyancy = Vec::with_capacity(count);

    for sample in 0..count {
        let fragment_id = historical.fragment_ids[sample];
        let kind = crust_kind(historical.crust_kind[sample]);
        provinces.push(if matches!(kind, CrustKind::Oceanic) {
            OCEANIC_PROVINCE_BIT | (fragment_id & !OCEANIC_PROVINCE_BIT)
        } else {
            fragment_id & !OCEANIC_PROVINCE_BIT
        });

        // Property noise is sample-owned, not fragment-owned. Crossing a quiet ancestry contact
        // therefore cannot change the baseline crust simply because the bookkeeping id changed.
        let jitter = unit_random(
            property_seed ^ (sample as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
        ) * 2.0
            - 1.0;
        let seafloor_age = if matches!(kind, CrustKind::Oceanic) {
            historical.crust_birth_age_myr[sample].clamp(0.0, 220.0)
        } else {
            0.0
        };
        let basement = basement_age[sample];

        let (cell_thickness, cell_density) = match kind {
            CrustKind::Continental => (
                // Preserve the accepted Earth-like mean continental column without using
                // basement formation age as a thickness/density shortcut. Spatial variation is
                // sample-owned; event history below supplies the causal thickening/thinning.
                (40.5 + jitter * 1.40).clamp(36.5, 44.5),
                (2738.0 + jitter * 14.0).clamp(2698.0, 2778.0),
            ),
            CrustKind::Transitional => (
                (19.0 + jitter * 1.8).clamp(14.0, 24.0),
                (2875.0 + jitter * 18.0).clamp(2830.0, 2925.0),
            ),
            CrustKind::Oceanic => {
                let thermal_maturity = (f64::from(seafloor_age) / 220.0).clamp(0.0, 1.0).sqrt();
                (
                    (6.35 + thermal_maturity * 0.55 + jitter * 0.08).clamp(5.8, 7.4),
                    (2910.0 + thermal_maturity * 42.0 + jitter * 6.0).clamp(2880.0, 2980.0),
                )
            }
        };

        let compatibility_age = match kind {
            CrustKind::Oceanic => seafloor_age,
            CrustKind::Continental | CrustKind::Transitional => basement,
        };
        composite_age.push(compatibility_age);
        oceanic_age.push(seafloor_age);
        thickness.push(cell_thickness as f32);
        density.push(cell_density as f32);
        buoyancy.push(buoyancy_index(cell_thickness, cell_density) as f32);
    }

    MaterialProperties {
        provinces,
        composite_age_myr: composite_age,
        oceanic_age_myr: oceanic_age,
        continental_basement_age_myr: basement_age,
        thickness_km: thickness,
        density_kg_per_m3: density,
        buoyancy_index: buoyancy,
    }
}

fn build_reworking_clock_and_stability<T: PlanetTopology>(
    topology: &T,
    historical: &HistoricalLithosphereModel,
    boundaries: &[GeologicalBoundary],
    basement_age_myr: &[f32],
    orogen: &[f32],
    rift: &[f32],
    subduction: &[f32],
    transform: &[f32],
) -> (Vec<f32>, Vec<f32>) {
    let count = topology.sample_count() as usize;
    let mut reworking_age = (0..count)
        .map(|sample| match crust_kind(historical.crust_kind[sample]) {
            CrustKind::Oceanic => historical.crust_birth_age_myr[sample].clamp(0.0, 220.0),
            CrustKind::Continental | CrustKind::Transitional => basement_age_myr[sample],
        })
        .collect::<Vec<_>>();

    for event in &historical.events {
        if matches!(event.kind, HistoricalEventKind::Spreading) {
            continue;
        }
        for sample in [event.geometry_sample_a, event.geometry_sample_b] {
            let index = sample as usize;
            if index < count && !matches!(crust_kind(historical.crust_kind[index]), CrustKind::Oceanic)
            {
                reworking_age[index] = reworking_age[index].min(event.age_myr.max(0.0));
            }
        }
    }
    for boundary in boundaries {
        if matches!(boundary.regime, GeologicalBoundaryRegime::OceanicRidge) {
            continue;
        }
        for sample in [boundary.sample_a, boundary.sample_b] {
            let index = sample as usize;
            if !matches!(crust_kind(historical.crust_kind[index]), CrustKind::Oceanic) {
                reworking_age[index] = 0.0;
            }
        }
    }

    // Major tectonothermal reworking affects a belt around the event geometry. Propagate the
    // youngest event clock only a bounded number of coarse cells; this records reworking history
    // without turning fragment membership into a continent-wide age reset.
    let mut next = reworking_age.clone();
    for _ in 0..6 {
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            let kind = crust_kind(historical.crust_kind[index]);
            if matches!(kind, CrustKind::Oceanic) {
                next[index] = reworking_age[index];
                continue;
            }
            let mut youngest = reworking_age[index];
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                if crust_kind(historical.crust_kind[ni]) == kind {
                    youngest = youngest.min(reworking_age[ni]);
                }
            }
            next[index] = youngest;
        }
        std::mem::swap(&mut reworking_age, &mut next);
    }

    let stability = (0..count)
        .map(|sample| {
            if !matches!(crust_kind(historical.crust_kind[sample]), CrustKind::Continental) {
                return 0.0;
            }
            let recovery = ((f64::from(reworking_age[sample]) - 180.0) / 1350.0).clamp(0.0, 1.0);
            let disturbance = f64::from(orogen[sample])
                .max(f64::from(rift[sample]))
                .max(f64::from(subduction[sample]))
                .max(f64::from(transform[sample]))
                .clamp(0.0, 1.0);
            (recovery * (1.0 - 0.82 * disturbance)).clamp(0.0, 1.0) as f32
        })
        .collect::<Vec<_>>();

    (reworking_age, stability)
}

fn classify_geological_boundary(
    kinds: &[u8],
    buoyancy: &[f32],
    edge: &PlateBoundaryEdge,
) -> (GeologicalBoundaryRegime, SubductionPolarity) {
    let kind_a = crust_kind(kinds[edge.sample_a as usize]);
    let kind_b = crust_kind(kinds[edge.sample_b as usize]);
    match edge.kind {
        PlateBoundaryKind::Transform => (
            GeologicalBoundaryRegime::Transform,
            SubductionPolarity::None,
        ),
        PlateBoundaryKind::Divergent => {
            if matches!(kind_a, CrustKind::Oceanic) && matches!(kind_b, CrustKind::Oceanic) {
                (
                    GeologicalBoundaryRegime::OceanicRidge,
                    SubductionPolarity::None,
                )
            } else if matches!(kind_a, CrustKind::Continental)
                && matches!(kind_b, CrustKind::Continental)
            {
                (
                    GeologicalBoundaryRegime::ContinentalRift,
                    SubductionPolarity::None,
                )
            } else {
                (
                    GeologicalBoundaryRegime::TransitionalDivergence,
                    SubductionPolarity::None,
                )
            }
        }
        PlateBoundaryKind::Convergent => match (kind_a, kind_b) {
            (CrustKind::Continental, CrustKind::Continental) => (
                GeologicalBoundaryRegime::ContinentalCollision,
                SubductionPolarity::None,
            ),
            (CrustKind::Oceanic, CrustKind::Continental)
            | (CrustKind::Oceanic, CrustKind::Transitional) => (
                GeologicalBoundaryRegime::OceanContinentSubduction,
                SubductionPolarity::PlateA,
            ),
            (CrustKind::Continental, CrustKind::Oceanic)
            | (CrustKind::Transitional, CrustKind::Oceanic) => (
                GeologicalBoundaryRegime::OceanContinentSubduction,
                SubductionPolarity::PlateB,
            ),
            (CrustKind::Oceanic, CrustKind::Oceanic) => {
                let polarity =
                    if buoyancy[edge.sample_a as usize] <= buoyancy[edge.sample_b as usize] {
                        SubductionPolarity::PlateA
                    } else {
                        SubductionPolarity::PlateB
                    };
                (GeologicalBoundaryRegime::OceanicSubduction, polarity)
            }
            _ => (
                GeologicalBoundaryRegime::ContinentalCollision,
                SubductionPolarity::None,
            ),
        },
    }
}

fn build_geological_boundaries(
    kinds: &[u8],
    buoyancy: &[f32],
    tectonics: &TectonicModel,
) -> Vec<GeologicalBoundary> {
    tectonics
        .boundaries
        .iter()
        .map(|edge| {
            let (regime, subduction_polarity) = classify_geological_boundary(kinds, buoyancy, edge);
            GeologicalBoundary {
                sample_a: edge.sample_a,
                sample_b: edge.sample_b,
                plate_a: edge.plate_a,
                plate_b: edge.plate_b,
                regime,
                subduction_polarity,
                normal_rate_m_per_year: edge.normal_rate_m_per_year,
                shear_rate_m_per_year: edge.shear_rate_m_per_year,
            }
        })
        .collect()
}

fn diffuse_max<T: PlanetTopology>(
    topology: &T,
    seeds: &[f64],
    passes: usize,
    retention: f64,
) -> Vec<f32> {
    let mut current = seeds.to_vec();
    let mut next = vec![0.0_f64; current.len()];
    for _ in 0..passes {
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            let neighbor_max = topology
                .neighbors(sample)
                .iter()
                .map(|neighbor| current[*neighbor as usize])
                .fold(0.0_f64, f64::max);
            next[index] = current[index].max(neighbor_max * retention).clamp(0.0, 1.0);
        }
        std::mem::swap(&mut current, &mut next);
    }
    current.into_iter().map(|value| value as f32).collect()
}

#[allow(clippy::type_complexity)]
fn build_history_fields<T: PlanetTopology>(
    topology: &T,
    historical: &HistoricalLithosphereModel,
    boundaries: &[GeologicalBoundary],
) -> (
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
    Vec<f32>,
) {
    let count = topology.sample_count() as usize;
    let mut orogen_seed = vec![0.0_f64; count];
    let mut rift_seed = vec![0.0_f64; count];
    let mut ridge_seed = vec![0.0_f64; count];
    let mut subduction_seed = vec![0.0_f64; count];
    let mut trench_seed = vec![0.0_f64; count];
    let mut arc_seed = vec![0.0_f64; count];
    let mut transform_seed = vec![0.0_f64; count];

    let register = |field: &mut [f64], sample: u32, strength: f64| {
        let index = sample as usize;
        if index < field.len() {
            field[index] = field[index].max(strength.clamp(0.0, 1.0));
        }
    };

    for event in &historical.events {
        let age_decay = (1.0 - f64::from(event.age_myr) / 520.0).clamp(0.18, 1.0);
        let strength = f64::from(event.strength) * age_decay;
        for sample in [event.geometry_sample_a, event.geometry_sample_b] {
            match event.kind {
                HistoricalEventKind::Rift => register(&mut rift_seed, sample, strength),
                HistoricalEventKind::Spreading => register(&mut ridge_seed, sample, strength),
                HistoricalEventKind::Subduction => {
                    register(&mut subduction_seed, sample, strength);
                    register(&mut trench_seed, sample, strength * 0.88);
                    register(&mut arc_seed, sample, strength * 0.72);
                }
                HistoricalEventKind::Collision => register(&mut orogen_seed, sample, strength),
                HistoricalEventKind::Transform => register(&mut transform_seed, sample, strength),
                HistoricalEventKind::Accretion => {
                    register(&mut orogen_seed, sample, strength * 0.72);
                    register(&mut arc_seed, sample, strength * 0.48);
                }
                HistoricalEventKind::Capture => register(&mut orogen_seed, sample, strength * 0.18),
                HistoricalEventKind::MicroplateFormation => {
                    register(&mut transform_seed, sample, strength * 0.82);
                    register(&mut orogen_seed, sample, strength * 0.12);
                }
            }
        }
    }

    for boundary in boundaries {
        let rate = boundary
            .normal_rate_m_per_year
            .abs()
            .max(boundary.shear_rate_m_per_year.abs());
        let strength = (0.35 + rate / 0.08 * 0.65).clamp(0.0, 1.0);
        for sample in [boundary.sample_a, boundary.sample_b] {
            match boundary.regime {
                GeologicalBoundaryRegime::ContinentalCollision => {
                    register(&mut orogen_seed, sample, strength)
                }
                GeologicalBoundaryRegime::OceanicSubduction
                | GeologicalBoundaryRegime::OceanContinentSubduction => {
                    register(&mut subduction_seed, sample, strength);
                    register(&mut trench_seed, sample, strength * 0.92);
                    register(&mut arc_seed, sample, strength * 0.78);
                }
                GeologicalBoundaryRegime::OceanicRidge => {
                    register(&mut ridge_seed, sample, strength)
                }
                GeologicalBoundaryRegime::ContinentalRift
                | GeologicalBoundaryRegime::TransitionalDivergence => {
                    register(&mut rift_seed, sample, strength)
                }
                GeologicalBoundaryRegime::Transform => {
                    register(&mut transform_seed, sample, strength)
                }
            }
        }
    }

    let orogen = diffuse_max(topology, &orogen_seed, 7, 0.78);
    let rift = diffuse_max(topology, &rift_seed, 5, 0.72);
    let ridge = diffuse_max(topology, &ridge_seed, 4, 0.68);
    let subduction = diffuse_max(topology, &subduction_seed, 5, 0.72);
    let trench = diffuse_max(topology, &trench_seed, 3, 0.62);
    let arc = diffuse_max(topology, &arc_seed, 5, 0.70);
    let transform = diffuse_max(topology, &transform_seed, 4, 0.68);

    let mut subsidence = Vec::with_capacity(count);
    let mut basin = Vec::with_capacity(count);
    let mut strain = Vec::with_capacity(count);
    for index in 0..count {
        let local_subsidence = (f64::from(rift[index]) * 0.58
            + f64::from(trench[index]) * 0.18
            + f64::from(transform[index]) * 0.10)
            .clamp(0.0, 1.0);
        let local_basin = (local_subsidence * 0.72
            + f64::from(rift[index]) * 0.22
            + f64::from(orogen[index]) * 0.08)
            .clamp(0.0, 1.0);
        let local_strain = f64::from(orogen[index])
            .max(f64::from(rift[index]))
            .max(f64::from(subduction[index]))
            .max(f64::from(transform[index]));
        subsidence.push(local_subsidence as f32);
        basin.push(local_basin as f32);
        strain.push(local_strain as f32);
    }

    (
        orogen, rift, ridge, subduction, trench, arc, transform, subsidence, basin, strain,
    )
}

fn apply_history_to_properties(
    kinds: &[u8],
    orogen: &[f32],
    rift: &[f32],
    thickness: &mut [f32],
    density: &mut [f32],
    buoyancy: &mut [f32],
) {
    for index in 0..kinds.len() {
        let kind = crust_kind(kinds[index]);
        let thickening = match kind {
            CrustKind::Continental => f64::from(orogen[index]) * 8.5,
            CrustKind::Transitional => f64::from(orogen[index]) * 4.0,
            CrustKind::Oceanic => f64::from(orogen[index]) * 0.8,
        };
        let thinning = match kind {
            CrustKind::Continental => f64::from(rift[index]) * 6.0,
            CrustKind::Transitional => f64::from(rift[index]) * 4.0,
            CrustKind::Oceanic => f64::from(rift[index]) * 0.4,
        };
        let new_thickness = (f64::from(thickness[index]) + thickening - thinning).clamp(4.0, 58.0);
        let density_shift = f64::from(rift[index]) * 14.0 - f64::from(orogen[index]) * 10.0;
        let new_density = (f64::from(density[index]) + density_shift).clamp(2520.0, 3180.0);
        thickness[index] = new_thickness as f32;
        density[index] = new_density as f32;
        buoyancy[index] = buoyancy_index(new_thickness, new_density) as f32;
    }
}

fn boundary_interface_length<T: PlanetTopology>(topology: &T, a: u32, b: u32) -> f64 {
    let neighbors = topology.neighbors(a);
    let interfaces = topology.neighbor_interface_arc_lengths_rad(a);
    neighbors
        .iter()
        .position(|neighbor| *neighbor == b)
        .and_then(|index| interfaces.get(index).copied())
        .unwrap_or(0.0)
}

fn build_plate_summaries<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    kinds: &[u8],
    ages: &[f32],
    thickness: &[f32],
) -> Vec<PlateSummary> {
    let plate_count = tectonics.plates.len();
    let mean_area = 4.0 * PI / plate_count as f64;
    let mut continental = vec![0.0_f64; plate_count];
    let mut transitional = vec![0.0_f64; plate_count];
    let mut oceanic = vec![0.0_f64; plate_count];
    let mut weighted_age = vec![0.0_f64; plate_count];
    let mut weighted_thickness = vec![0.0_f64; plate_count];
    let mut convergent = vec![0.0_f64; plate_count];
    let mut divergent = vec![0.0_f64; plate_count];
    let mut transform = vec![0.0_f64; plate_count];

    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let plate = tectonics.plate_ids[index] as usize;
        let area = topology.area_steradians(sample);
        match crust_kind(kinds[index]) {
            CrustKind::Continental => continental[plate] += area,
            CrustKind::Transitional => transitional[plate] += area,
            CrustKind::Oceanic => oceanic[plate] += area,
        }
        weighted_age[plate] += area * f64::from(ages[index]);
        weighted_thickness[plate] += area * f64::from(thickness[index]);
    }

    for boundary in &tectonics.boundaries {
        let length = boundary_interface_length(topology, boundary.sample_a, boundary.sample_b);
        for plate in [boundary.plate_a as usize, boundary.plate_b as usize] {
            match boundary.kind {
                PlateBoundaryKind::Convergent => convergent[plate] += length,
                PlateBoundaryKind::Divergent => divergent[plate] += length,
                PlateBoundaryKind::Transform => transform[plate] += length,
            }
        }
    }

    tectonics
        .plates
        .iter()
        .map(|plate| {
            let index = plate.id as usize;
            let area = plate.area_steradians.max(1.0e-12);
            let boundary_total = convergent[index] + divergent[index] + transform[index];
            PlateSummary {
                plate_id: plate.id,
                area_steradians: plate.area_steradians,
                area_fraction: plate.area_steradians / (4.0 * PI),
                scale_class: if plate.area_steradians >= mean_area * 1.5 {
                    PlateScaleClass::Major
                } else if plate.area_steradians <= mean_area * 0.65 {
                    PlateScaleClass::Minor
                } else {
                    PlateScaleClass::Intermediate
                },
                continental_fraction: continental[index] / area,
                transitional_fraction: transitional[index] / area,
                oceanic_fraction: oceanic[index] / area,
                mean_crust_age_myr: weighted_age[index] / area,
                mean_crust_thickness_km: weighted_thickness[index] / area,
                convergent_boundary_fraction: if boundary_total > 0.0 {
                    convergent[index] / boundary_total
                } else {
                    0.0
                },
                divergent_boundary_fraction: if boundary_total > 0.0 {
                    divergent[index] / boundary_total
                } else {
                    0.0
                },
                transform_boundary_fraction: if boundary_total > 0.0 {
                    transform[index] / boundary_total
                } else {
                    0.0
                },
            }
        })
        .collect()
}

fn geology_hash(stage_seed: u64, model: &CrustalModel, history_hash: u64) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, b"geology:historical-crust-projection:v2\0");
    hash = fnv_update(hash, &stage_seed.to_le_bytes());
    hash = fnv_update(hash, &history_hash.to_le_bytes());
    hash = fnv_update(hash, &model.crust_kind);
    for value in &model.crust_province_id {
        hash = fnv_update(hash, &value.to_le_bytes());
    }
    for field in [
        &model.crust_age_myr,
        &model.oceanic_age_myr,
        &model.continental_basement_age_myr,
        &model.last_tectonic_reworking_age_myr,
        &model.continental_stability_index,
        &model.crust_thickness_km,
        &model.crust_density_kg_per_m3,
        &model.buoyancy_index,
        &model.orogenic_history,
        &model.rift_history,
        &model.ridge_history,
        &model.subduction_history,
        &model.trench_history,
        &model.volcanic_arc_history,
        &model.transform_history,
        &model.subsidence_history,
        &model.basin_potential,
        &model.crustal_strain,
    ] {
        for value in field {
            hash = fnv_update(hash, &value.to_bits().to_le_bytes());
        }
    }
    hash
}

fn compute_geology_metrics<T: PlanetTopology>(
    topology: &T,
    kinds: &[u8],
    ages: &[f32],
    reworking_age_myr: &[f32],
    thickness: &[f32],
    boundaries: &[GeologicalBoundary],
    hash: u64,
) -> GeologyMetrics {
    let total_area = (0..topology.sample_count())
        .map(|sample| topology.area_steradians(sample))
        .sum::<f64>()
        .max(1.0e-12);
    let mut area = [0.0_f64; 3];
    let mut age_sum = [0.0_f64; 3];
    let mut reworking_sum = [0.0_f64; 3];
    let mut thickness_sum = [0.0_f64; 3];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        let bucket = match crust_kind(kinds[index]) {
            CrustKind::Oceanic => 0,
            CrustKind::Transitional => 1,
            CrustKind::Continental => 2,
        };
        let cell_area = topology.area_steradians(sample);
        area[bucket] += cell_area;
        age_sum[bucket] += cell_area * f64::from(ages[index]);
        reworking_sum[bucket] += cell_area * f64::from(reworking_age_myr[index]);
        thickness_sum[bucket] += cell_area * f64::from(thickness[index]);
    }
    let mut regime_counts = [0_u32; 7];
    for boundary in boundaries {
        regime_counts[boundary.regime as usize - 1] += 1;
    }
    GeologyMetrics {
        sample_count: topology.sample_count(),
        continental_area_fraction: area[2] / total_area,
        transitional_area_fraction: area[1] / total_area,
        oceanic_area_fraction: area[0] / total_area,
        mean_continental_age_myr: if area[2] > 0.0 {
            age_sum[2] / area[2]
        } else {
            0.0
        },
        mean_oceanic_age_myr: if area[0] > 0.0 {
            age_sum[0] / area[0]
        } else {
            0.0
        },
        mean_continental_reworking_age_myr: if area[2] > 0.0 {
            reworking_sum[2] / area[2]
        } else {
            0.0
        },
        mean_continental_thickness_km: if area[2] > 0.0 {
            thickness_sum[2] / area[2]
        } else {
            0.0
        },
        mean_oceanic_thickness_km: if area[0] > 0.0 {
            thickness_sum[0] / area[0]
        } else {
            0.0
        },
        oceanic_subduction_edges: regime_counts[0],
        ocean_continent_subduction_edges: regime_counts[1],
        continental_collision_edges: regime_counts[2],
        oceanic_ridge_edges: regime_counts[3],
        continental_rift_edges: regime_counts[4],
        transitional_divergence_edges: regime_counts[5],
        transform_edges: regime_counts[6],
        geology_hash: hash,
    }
}

pub fn project_historical_crust<T: PlanetTopology>(
    topology: &T,
    historical: &HistoricalLithosphereModel,
    tectonics: &TectonicModel,
    seed: &str,
) -> Result<CrustalModel, WorldgenError> {
    let count = topology.sample_count() as usize;
    if historical.crust_kind.len() != count
        || historical.crust_birth_age_myr.len() != count
        || historical.fragment_ids.len() != count
        || tectonics.plate_ids.len() != count
    {
        return Err(WorldgenError::InvalidGeology(
            "historical crust projection does not match topology sample count",
        ));
    }
    if tectonics.plate_ids != historical.current_plate_ids {
        return Err(WorldgenError::InvalidGeology(
            "modern tectonic ownership does not match historical current ownership",
        ));
    }

    let stage_seed = derive_stage_seed(seed, HISTORICAL_GEOLOGY_NAMESPACE);
    let province_seed = historical.metrics.history_hash;
    let property_seed = derive_stage_seed(seed, HISTORICAL_PROPERTIES_NAMESPACE);
    let history_seed = derive_stage_seed(seed, HISTORICAL_HISTORY_NAMESPACE);
    let kinds = historical.crust_kind.clone();
    let mut material = build_material_properties(topology, historical, property_seed);
    let initial_boundaries =
        build_geological_boundaries(&kinds, &material.buoyancy_index, tectonics);
    let (orogen, rift, ridge, subduction, trench, arc, transform, subsidence, basin, strain) =
        build_history_fields(topology, historical, &initial_boundaries);
    let (last_reworking_age_myr, continental_stability_index) =
        build_reworking_clock_and_stability(
            topology,
            historical,
            &initial_boundaries,
            &material.continental_basement_age_myr,
            &orogen,
            &rift,
            &subduction,
            &transform,
        );
    apply_history_to_properties(
        &kinds,
        &orogen,
        &rift,
        &mut material.thickness_km,
        &mut material.density_kg_per_m3,
        &mut material.buoyancy_index,
    );
    let boundaries = build_geological_boundaries(&kinds, &material.buoyancy_index, tectonics);
    let plate_summaries = build_plate_summaries(
        topology,
        tectonics,
        &kinds,
        &material.composite_age_myr,
        &material.thickness_km,
    );

    let placeholder_metrics = GeologyMetrics {
        sample_count: topology.sample_count(),
        continental_area_fraction: 0.0,
        transitional_area_fraction: 0.0,
        oceanic_area_fraction: 0.0,
        mean_continental_age_myr: 0.0,
        mean_oceanic_age_myr: 0.0,
        mean_continental_reworking_age_myr: 0.0,
        mean_continental_thickness_km: 0.0,
        mean_oceanic_thickness_km: 0.0,
        oceanic_subduction_edges: 0,
        ocean_continent_subduction_edges: 0,
        continental_collision_edges: 0,
        oceanic_ridge_edges: 0,
        continental_rift_edges: 0,
        transitional_divergence_edges: 0,
        transform_edges: 0,
        geology_hash: 0,
    };
    let mut model = CrustalModel {
        stage: StageIdentity {
            id: GEOLOGY_STAGE_ID,
            version: GEOLOGY_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        province_seed,
        property_seed,
        history_seed,
        crust_kind: kinds,
        crust_province_id: material.provinces,
        crust_age_myr: material.composite_age_myr,
        oceanic_age_myr: material.oceanic_age_myr,
        continental_basement_age_myr: material.continental_basement_age_myr,
        last_tectonic_reworking_age_myr: last_reworking_age_myr,
        continental_stability_index,
        crust_thickness_km: material.thickness_km,
        crust_density_kg_per_m3: material.density_kg_per_m3,
        buoyancy_index: material.buoyancy_index,
        orogenic_history: orogen,
        rift_history: rift,
        ridge_history: ridge,
        subduction_history: subduction,
        trench_history: trench,
        volcanic_arc_history: arc,
        transform_history: transform,
        subsidence_history: subsidence,
        basin_potential: basin,
        crustal_strain: strain,
        boundaries,
        plate_summaries,
        metrics: placeholder_metrics,
    };
    let hash = geology_hash(stage_seed, &model, historical.metrics.history_hash);
    model.metrics = compute_geology_metrics(
        topology,
        &model.crust_kind,
        &model.crust_age_myr,
        &model.last_tectonic_reworking_age_myr,
        &model.crust_thickness_km,
        &model.boundaries,
        hash,
    );

    if model.crust_age_myr.iter().any(|value| !value.is_finite())
        || model
            .crust_thickness_km
            .iter()
            .any(|value| !value.is_finite())
        || model
            .crust_density_kg_per_m3
            .iter()
            .any(|value| !value.is_finite())
        || model.boundaries.len() != tectonics.boundaries.len()
        || model.plate_summaries.len() != tectonics.plates.len()
    {
        return Err(WorldgenError::InvalidGeology(
            "historical crust projection failed physical completeness checks",
        ));
    }

    Ok(model)
}

pub fn inherit_historical_identity(
    fine_topology: &GeodesicTopology,
    coarse_level: u8,
    historical: &HistoricalLithosphereModel,
) -> Result<InheritedHistoricalIdentity, WorldgenError> {
    let map = build_refinement_map(fine_topology, coarse_level)?;
    if map.metrics.coarse_sample_count != historical.metrics.sample_count {
        return Err(WorldgenError::InvalidRefinement(
            "historical identity coarse sample count does not match refinement source",
        ));
    }
    let origin_plate_ids = refine_categorical_u16(&map, &historical.origin_plate_ids)?;
    let fragment_ids = refine_categorical_u16(&map, &historical.fragment_ids)?;
    let current_plate_ids = refine_categorical_u16(&map, &historical.current_plate_ids)?;
    let crust_kind = refine_categorical_u8(&map, &historical.crust_kind)?;
    let crust_birth_age_myr = refine_scalar_f32_with_domains(
        fine_topology,
        coarse_level,
        &historical.crust_birth_age_myr,
        &map,
        &historical.fragment_ids,
    )?;

    let mut hash = FNV_OFFSET_BASIS;
    hash = fnv_update(hash, HISTORICAL_INHERITANCE_STAGE_ID.as_bytes());
    hash = fnv_update(hash, &historical.metrics.history_hash.to_le_bytes());
    hash = fnv_update(hash, &map.metrics.provenance_hash.to_le_bytes());
    for values in [&origin_plate_ids, &fragment_ids, &current_plate_ids] {
        for value in values {
            hash = fnv_update(hash, &value.to_le_bytes());
        }
    }
    hash = fnv_update(hash, &crust_kind);
    for value in &crust_birth_age_myr {
        hash = fnv_update(hash, &value.to_bits().to_le_bytes());
    }

    Ok(InheritedHistoricalIdentity {
        stage: StageIdentity {
            id: HISTORICAL_INHERITANCE_STAGE_ID,
            version: HISTORICAL_INHERITANCE_STAGE_VERSION,
            derived_seed: historical.metrics.history_hash,
        },
        map,
        origin_plate_ids,
        fragment_ids,
        current_plate_ids,
        crust_kind,
        crust_birth_age_myr,
        identity_hash: hash,
    })
}

pub fn generate_historical_frontend<T: PlanetTopology>(
    topology: &T,
    request: &HistoricalLithosphereRequest,
    planet: PlanetPhysicalParameters,
) -> Result<HistoricalFrontend, WorldgenError> {
    let historical = generate_historical_lithosphere(topology, request, planet)?;
    let tectonics =
        project_historical_modern_tectonics(topology, &historical, &request.seed, planet)?;
    let geology = project_historical_crust(topology, &historical, &tectonics, &request.seed)?;
    Ok(HistoricalFrontend {
        historical,
        tectonics,
        geology,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_icosphere;

    #[test]
    fn historical_frontend_is_deterministic_and_projects_modern_ownership() {
        let topology = build_icosphere(3).unwrap();
        let request = HistoricalLithosphereRequest::new("historical-frontend", 10);
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let first = generate_historical_frontend(&topology, &request, planet).unwrap();
        let second = generate_historical_frontend(&topology, &request, planet).unwrap();
        assert_eq!(
            first.historical.metrics.history_hash,
            second.historical.metrics.history_hash
        );
        assert_eq!(
            first.tectonics.metrics.tectonic_hash,
            second.tectonics.metrics.tectonic_hash
        );
        assert_eq!(
            first.geology.metrics.geology_hash,
            second.geology.metrics.geology_hash
        );
        assert_eq!(
            first.tectonics.plate_ids,
            first.historical.current_plate_ids
        );
        assert_eq!(
            first.tectonics.plates.len(),
            first.historical.metrics.modern_plate_count as usize
        );
        assert!(
            !first.tectonics.plates.is_empty(),
            "forward history must retain at least one present tectonic plate"
        );
        assert_eq!(first.geology.crust_kind, first.historical.crust_kind);
    }

    #[test]
    fn historical_crust_provenance_is_fragment_owned() {
        let topology = build_icosphere(3).unwrap();
        let frontend = generate_historical_frontend(
            &topology,
            &HistoricalLithosphereRequest::new("historical-provenance", 12),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
        for sample in 0..topology.sample_count() as usize {
            let fragment = frontend.historical.fragment_ids[sample] & !OCEANIC_PROVINCE_BIT;
            let province = frontend.geology.crust_province_id[sample] & !OCEANIC_PROVINCE_BIT;
            assert_eq!(fragment, province);
            assert_eq!(
                frontend.tectonics.plate_ids[sample],
                frontend.historical.current_plate_ids[sample]
            );
        }
        assert!(frontend.geology.metrics.continental_area_fraction > 0.0);
        assert!(frontend.geology.metrics.oceanic_area_fraction > 0.0);
        assert!(frontend.geology.metrics.mean_oceanic_age_myr <= 220.0);
    }

    #[test]
    fn modern_boundaries_are_derived_from_historical_current_ownership() {
        let topology = build_icosphere(3).unwrap();
        let frontend = generate_historical_frontend(
            &topology,
            &HistoricalLithosphereRequest::new("historical-boundaries", 9),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
        assert!(!frontend.tectonics.boundaries.is_empty());
        assert_eq!(
            frontend.tectonics.boundaries.len(),
            frontend.geology.boundaries.len()
        );
        for boundary in &frontend.tectonics.boundaries {
            assert_ne!(boundary.plate_a, boundary.plate_b);
            assert_eq!(
                frontend.historical.current_plate_ids[boundary.sample_a as usize],
                boundary.plate_a
            );
            assert_eq!(
                frontend.historical.current_plate_ids[boundary.sample_b as usize],
                boundary.plate_b
            );
        }
    }

    #[test]
    fn historical_identity_inherits_to_fine_topology_without_erasing_lineage() {
        let coarse_level = 3;
        let coarse = build_icosphere(coarse_level).unwrap();
        let fine = build_icosphere(5).unwrap();
        let frontend = generate_historical_frontend(
            &coarse,
            &HistoricalLithosphereRequest::new("historical-inheritance", 10),
            PlanetPhysicalParameters::earthlike_reference(),
        )
        .unwrap();
        let inherited =
            inherit_historical_identity(&fine, coarse_level, &frontend.historical).unwrap();
        assert_eq!(
            inherited.origin_plate_ids.len(),
            fine.metrics().sample_count as usize
        );
        assert_eq!(
            inherited.fragment_ids.len(),
            fine.metrics().sample_count as usize
        );
        assert_eq!(
            inherited.current_plate_ids.len(),
            fine.metrics().sample_count as usize
        );
        for sample in 0..coarse.metrics().sample_count as usize {
            assert_eq!(
                inherited.origin_plate_ids[sample],
                frontend.historical.origin_plate_ids[sample]
            );
            assert_eq!(
                inherited.fragment_ids[sample],
                frontend.historical.fragment_ids[sample]
            );
            assert_eq!(
                inherited.current_plate_ids[sample],
                frontend.historical.current_plate_ids[sample]
            );
        }
        assert_ne!(inherited.identity_hash, 0);
    }
}
