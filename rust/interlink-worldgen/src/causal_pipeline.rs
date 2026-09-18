use crate::{
    derive_stage_seed, generate_pre_orogenic_lithosphere, generate_tectonic_history,
    generate_tectonic_orogen_provinces, GeodesicTopology, InheritedBoundarySet, LithosphereRequest,
    InheritedStructureKind, OrogenProvinceKind, OrogenProvinceModel, OrogenProvinceRequest,
    PlanetPhysicalParameters,
    PlanetTopology, PreOrogenicLithosphereModel, PreOrogenicLithosphereRequest, StageIdentity,
    TectonicHistoryModel, TectonicHistoryRequest, TectonicModel, TopographyMetrics,
    TopographyParameters, TopographyRequest, TopographyState, WorldgenError,
};
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

pub const TECTONIC_TOPOGRAPHY_STAGE_ID: &str = "terrain:initial-topography";
pub const TECTONIC_TOPOGRAPHY_STAGE_VERSION: u32 = 14;
const TECTONIC_TOPOGRAPHY_NAMESPACE: &str = "terrain:orogen-topology-and-connected-ocean:v3";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const CRUST_OCEANIC: u8 = 1;
const CRUST_TRANSITIONAL: u8 = 2;

#[derive(Clone, Debug, PartialEq)]
pub struct LithosphericModel {
    legacy: crate::lithosphere::LithosphericModel,
    pub tectonic_history: TectonicHistoryModel,
    pub pre_orogenic: PreOrogenicLithosphereModel,
    pub orogen_provinces: OrogenProvinceModel,
}

impl Deref for LithosphericModel {
    type Target = crate::lithosphere::LithosphericModel;
    fn deref(&self) -> &Self::Target {
        &self.legacy
    }
}
impl DerefMut for LithosphericModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.legacy
    }
}
impl LithosphericModel {
    pub fn orogen_province_hash_hex(&self) -> String {
        self.orogen_provinces.metrics.province_hash_hex()
    }
}

pub fn generate_lithosphere<T: PlanetTopology>(
    topology: &T,
    tectonics: &TectonicModel,
    geology: &crate::CrustalModel,
    request: &LithosphereRequest,
) -> Result<LithosphericModel, WorldgenError> {
    let legacy = crate::lithosphere::generate_lithosphere(topology, tectonics, geology, request)?;

    // The public compatibility signature predates planet-aware lithosphere generation. The
    // current product pipeline is Earthlike, so carry the new causal stages here until the
    // post-deformation lithosphere API is replaced in the later cleanup PR.
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let tectonic_history = generate_tectonic_history(
        topology,
        tectonics,
        &TectonicHistoryRequest::new(request.seed.as_str()),
        planet,
    )?;
    let pre_orogenic = generate_pre_orogenic_lithosphere(
        topology,
        tectonics,
        &tectonic_history,
        geology,
        &PreOrogenicLithosphereRequest::new(request.seed.as_str()),
    )?;
    let orogen_provinces = generate_tectonic_orogen_provinces(
        topology,
        tectonics,
        &tectonic_history,
        geology,
        &pre_orogenic,
        &OrogenProvinceRequest::new(request.seed.as_str()),
        planet,
    )?;

    Ok(LithosphericModel {
        legacy,
        tectonic_history,
        pre_orogenic,
        orogen_provinces,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct InheritedPhysicalState {
    legacy: crate::refinement::InheritedPhysicalState,
    pub causal_inheritance_hash: u64,
    pub orogen_province_hash: u64,
    pub province_ids: Vec<u16>,
    pub province_kind: Vec<u8>,
    // Deliberately shadows the legacy field so existing browser diagnostics named
    // "Orogenic history" now visualize the new province intensity rather than the old tube field.
    pub orogenic_history: Vec<f32>,
    pub boundary_distance_km: Vec<f32>,
    pub mountain_core_index: Vec<f32>,
    pub interior_resistance_index: Vec<f32>,
    pub volcanic_arc_history: Vec<f32>,
    pub crustal_root_index: Vec<f32>,
    pub plateau_index: Vec<f32>,
    pub fold_thrust_index: Vec<f32>,
    pub foreland_basin_index: Vec<f32>,
    pub backarc_extension_index: Vec<f32>,
    pub suture_index: Vec<f32>,
    pub transpression_index: Vec<f32>,
    pub maturity_index: Vec<f32>,
    pub shortening_index: Vec<f32>,
    pub local_width_km: Vec<f32>,
    pub source_along_strike_fraction: Vec<f32>,
}

impl Deref for InheritedPhysicalState {
    type Target = crate::refinement::InheritedPhysicalState;
    fn deref(&self) -> &Self::Target {
        &self.legacy
    }
}
impl DerefMut for InheritedPhysicalState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.legacy
    }
}
impl InheritedPhysicalState {
    pub fn inheritance_hash(&self) -> u64 {
        self.causal_inheritance_hash
    }
    pub fn inheritance_hash_hex(&self) -> String {
        format!("{:016x}", self.inheritance_hash())
    }
    pub fn parameter_hash_hex(&self) -> String {
        self.legacy.parameter_hash_hex()
    }
    pub fn orogen_province_hash_hex(&self) -> String {
        format!("{:016x}", self.orogen_province_hash)
    }

    /// Release fine-grid tectonic province scratch once WG-4 has materialized durable relief.
    /// `orogenic_history` remains because it is an interactive browser diagnostic; the other
    /// province rasters are construction intermediates and must not survive into WG-5+.
    pub fn release_topography_scratch(&mut self) {
        self.province_ids = Vec::new();
        self.province_kind = Vec::new();
        self.boundary_distance_km = Vec::new();
        self.mountain_core_index = Vec::new();
        self.interior_resistance_index = Vec::new();
        self.volcanic_arc_history = Vec::new();
        self.crustal_root_index = Vec::new();
        self.plateau_index = Vec::new();
        self.fold_thrust_index = Vec::new();
        self.foreland_basin_index = Vec::new();
        self.backarc_extension_index = Vec::new();
        self.suture_index = Vec::new();
        self.transpression_index = Vec::new();
        self.maturity_index = Vec::new();
        self.shortening_index = Vec::new();
        self.local_width_km = Vec::new();
        self.source_along_strike_fraction = Vec::new();

        // Legacy inherited fields whose last consumer is WG-4. Preserve only browser diagnostics
        // plus the four mechanical fields consumed later by WG-7A erosion.
        self.legacy.crust_province_id = Vec::new();
        self.legacy.crust_density_kg_per_m3 = Vec::new();
        self.legacy.buoyancy_index = Vec::new();
        self.legacy.orogenic_history = Vec::new();
        self.legacy.rift_history = Vec::new();
        self.legacy.subduction_history = Vec::new();
        self.legacy.volcanic_arc_history = Vec::new();
        self.legacy.transform_history = Vec::new();
        self.legacy.subsidence_history = Vec::new();
        self.legacy.basin_potential = Vec::new();
        self.legacy.crustal_strain = Vec::new();
        self.legacy.effective_elastic_thickness_km = Vec::new();
        self.legacy.thermal_anomaly_index = Vec::new();
        self.legacy.mantle_upwelling_index = Vec::new();
        self.legacy.compensated_buoyancy_index = Vec::new();
        self.legacy.fragment_ids = Vec::new();
    }

    /// WG-7A is the last consumer of inherited structural fabric.
    pub fn release_post_erosion_scratch(&mut self) {
        self.legacy.structural_fabric_strength = Vec::new();
    }
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub fn inherit_physical_state(
    fine_topology: &GeodesicTopology,
    coarse_level: u8,
    tectonics: &TectonicModel,
    geology: &crate::CrustalModel,
    lithosphere: &LithosphericModel,
    parameters: PlanetPhysicalParameters,
) -> Result<InheritedPhysicalState, WorldgenError> {
    let legacy = crate::refinement::inherit_physical_state(
        fine_topology,
        coarse_level,
        tectonics,
        geology,
        lithosphere,
        parameters,
    )?;
    let map = &legacy.map;
    let domains = &tectonics.plate_ids;
    let orogens = &lithosphere.orogen_provinces;

    let refine = |values: &[f32]| {
        crate::refinement::refine_scalar_f32_with_domains(
            fine_topology,
            coarse_level,
            values,
            map,
            domains,
        )
    };

    let province_ids = crate::refinement::refine_categorical_u16(map, &orogens.province_ids)?;
    let province_kind = crate::refinement::refine_categorical_u8(map, &orogens.province_kind)?;
    let orogenic_history = refine(&orogens.orogenic_intensity)?;
    let boundary_distance_km = refine(&orogens.boundary_distance_km)?;
    let mountain_core_index = refine(&orogens.mountain_core_index)?;
    let interior_resistance_index = refine(&orogens.interior_resistance_index)?;
    let volcanic_arc_history = refine(&orogens.volcanic_arc_index)?;
    let crustal_root_index = refine(&orogens.crustal_root_index)?;
    let plateau_index = refine(&orogens.plateau_index)?;
    let fold_thrust_index = refine(&orogens.fold_thrust_index)?;
    let foreland_basin_index = refine(&orogens.foreland_basin_index)?;
    let backarc_extension_index = refine(&orogens.backarc_extension_index)?;
    let suture_index = refine(&orogens.suture_index)?;
    let transpression_index = refine(&orogens.transpression_index)?;
    let maturity_index = refine(&orogens.maturity_index)?;
    let shortening_index = refine(&orogens.shortening_index)?;
    let local_width_km = refine(&orogens.local_width_km)?;
    let source_along_strike_fraction = refine(&orogens.source_along_strike_fraction)?;

    let mut causal_hash = FNV_OFFSET_BASIS;
    causal_hash = fnv_update(causal_hash, &legacy.inheritance_hash.to_le_bytes());
    causal_hash = fnv_update(causal_hash, &orogens.metrics.province_hash.to_le_bytes());
    for value in &orogenic_history {
        causal_hash = fnv_update(causal_hash, &value.to_bits().to_le_bytes());
    }

    Ok(InheritedPhysicalState {
        legacy,
        causal_inheritance_hash: causal_hash,
        orogen_province_hash: orogens.metrics.province_hash,
        province_ids,
        province_kind,
        orogenic_history,
        boundary_distance_km,
        mountain_core_index,
        interior_resistance_index,
        volcanic_arc_history,
        crustal_root_index,
        plateau_index,
        fold_thrust_index,
        foreland_basin_index,
        backarc_extension_index,
        suture_index,
        transpression_index,
        maturity_index,
        shortening_index,
        local_width_km,
        source_along_strike_fraction,
    })
}

fn explicit_mechanical_structure(kind: u8) -> bool {
    kind == InheritedStructureKind::PaleoSuture as u8
        || kind == InheritedStructureKind::InheritedRift as u8
        || kind == InheritedStructureKind::ShearZone as u8
        || kind == InheritedStructureKind::ContinentalMargin as u8
}

fn mechanical_edge_domain_factor(
    inherited: &InheritedPhysicalState,
    sample: usize,
    neighbor: usize,
) -> f64 {
    if inherited.kinematic_domain_ids[neighbor] == inherited.kinematic_domain_ids[sample] {
        1.0
    } else {
        0.25
    }
}

fn relax_quiet_provenance_isostasy(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    values: &mut [f32],
) {
    const PASSES: usize = 2;
    const RELAXATION: f64 = 0.32;
    let areas = topology.dual_area_steradians();

    for _ in 0..PASSES {
        for sample in 0..topology.metrics().sample_count {
            let a = sample as usize;
            for neighbor in topology.neighbors_of(sample) {
                if *neighbor <= sample {
                    continue;
                }
                let b = *neighbor as usize;
                let quiet_contact = inherited.crust_province_id[a] != inherited.crust_province_id[b]
                    && inherited.plate_ids[a] == inherited.plate_ids[b]
                    && inherited.crust_kind[a] == inherited.crust_kind[b]
                    && inherited.crust_kind[a] != CRUST_OCEANIC
                    && inherited.province_kind[a] == 0
                    && inherited.province_kind[b] == 0
                    && !explicit_mechanical_structure(inherited.structural_zone_kind[a])
                    && !explicit_mechanical_structure(inherited.structural_zone_kind[b]);
                if !quiet_contact {
                    continue;
                }

                // Relax only the topographic expression of a quiet provenance contact. The
                // inherited crustal properties remain unchanged. Area-weighted pair exchange
                // conserves the isostatic load while removing a categorical elevation step.
                let area_a = areas[a].max(1.0e-12);
                let area_b = areas[b].max(1.0e-12);
                let value_a = f64::from(values[a]);
                let value_b = f64::from(values[b]);
                let mean = (value_a * area_a + value_b * area_b) / (area_a + area_b);
                values[a] = (value_a + RELAXATION * (mean - value_a)) as f32;
                values[b] = (value_b + RELAXATION * (mean - value_b)) as f32;
            }
        }
    }
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
                let domain_factor = mechanical_edge_domain_factor(inherited, sample, neighbor);
                let weight = interface / center * domain_factor;
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
            // Preserve tectonic province edges much more strongly than the old broad smoothing pass.
            lambda *= (1.0 - 0.62 * weakness) * (1.0 - 0.30 * fabric);
            next[sample] = current[sample] + lambda * (neighbor_mean - current[sample]);
        }
        std::mem::swap(&mut current, &mut next);
    }
    current
}

fn area_weighted_mean(values: &[f64], areas: &[f64]) -> f64 {
    let area = areas.iter().sum::<f64>();
    if area <= 0.0 {
        return 0.0;
    }
    values
        .iter()
        .zip(areas.iter())
        .map(|(value, area)| value * area)
        .sum::<f64>()
        / area
}

fn area_weighted_quantile(values: &[f64], areas: &[f64], q: f64) -> f64 {
    let mut pairs = values
        .iter()
        .copied()
        .zip(areas.iter().copied())
        .collect::<Vec<_>>();
    pairs.sort_by(|left, right| left.0.total_cmp(&right.0));
    let target = areas.iter().sum::<f64>() * q.clamp(0.0, 1.0);
    let mut cumulative = 0.0;
    for (value, area) in pairs {
        cumulative += area;
        if cumulative >= target {
            return value;
        }
    }
    values.last().copied().unwrap_or(0.0)
}

fn major_ocean_reservoir_seed_mask(
    topology: &GeodesicTopology,
    crust_kind: &[u8],
    provisional_submerged: &[u8],
) -> Vec<u8> {
    let count = topology.metrics().sample_count as usize;
    let mut visited = vec![false; count];
    let mut components: Vec<(f64, Vec<usize>)> = Vec::new();
    let mut candidate_area = 0.0_f64;
    let surface_area = topology.dual_area_steradians().iter().sum::<f64>();

    for sample in 0..count {
        if visited[sample]
            || crust_kind[sample] != CRUST_OCEANIC
            || provisional_submerged[sample] == 0
        {
            continue;
        }
        visited[sample] = true;
        let mut queue = VecDeque::from([sample as u32]);
        let mut members = Vec::new();
        let mut area = 0.0_f64;
        while let Some(current) = queue.pop_front() {
            let index = current as usize;
            members.push(index);
            area += topology.dual_area_steradians()[index];
            for neighbor in topology.neighbors(current) {
                let neighbor = *neighbor as usize;
                if !visited[neighbor]
                    && crust_kind[neighbor] == CRUST_OCEANIC
                    && provisional_submerged[neighbor] != 0
                {
                    visited[neighbor] = true;
                    queue.push_back(neighbor as u32);
                }
            }
        }
        candidate_area += area;
        components.push((area, members));
    }

    components.sort_by(|left, right| right.0.total_cmp(&left.0));
    // Global ocean reservoirs must be broad components, not tiny trapped oceanic slivers.
    // Marginal seas do not need to be seeds: they join automatically if a marine path reaches them.
    let minimum_reservoir_area = (surface_area * 0.0025).max(candidate_area * 0.015);
    let mut seeds = vec![0_u8; count];
    let mut kept = 0usize;
    for (area, members) in &components {
        if *area + 1.0e-15 < minimum_reservoir_area {
            continue;
        }
        kept += 1;
        for sample in members {
            seeds[*sample] = 1;
        }
    }
    if kept == 0 {
        if let Some((_, members)) = components.first() {
            for sample in members {
                seeds[*sample] = 1;
            }
        }
    }
    seeds
}

fn province_relief(inherited: &InheritedPhysicalState, index: usize) -> (f64, f64) {
    let intensity = f64::from(inherited.orogenic_history[index]).clamp(0.0, 1.0);
    let mountain_core = f64::from(inherited.mountain_core_index[index]).clamp(0.0, 1.0);
    let resistance = f64::from(inherited.interior_resistance_index[index]).clamp(0.0, 1.0);
    let root = f64::from(inherited.crustal_root_index[index]).clamp(0.0, 1.0);
    let plateau = f64::from(inherited.plateau_index[index]).clamp(0.0, 1.0);
    let fold = f64::from(inherited.fold_thrust_index[index]).clamp(0.0, 1.0);
    let foreland = f64::from(inherited.foreland_basin_index[index]).clamp(0.0, 1.0);
    let volcanic_arc = f64::from(inherited.volcanic_arc_history[index]).clamp(0.0, 1.0);
    let backarc = f64::from(inherited.backarc_extension_index[index]).clamp(0.0, 1.0);
    let suture = f64::from(inherited.suture_index[index]).clamp(0.0, 1.0);
    let transpression = f64::from(inherited.transpression_index[index]).clamp(0.0, 1.0);
    let maturity = f64::from(inherited.maturity_index[index]).clamp(0.0, 1.0);
    let shortening = f64::from(inherited.shortening_index[index]).clamp(0.0, 1.0);
    let kind = inherited.province_kind[index];
    let subduction = kind == OrogenProvinceKind::CordilleranArc as u8
        || kind == OrogenProvinceKind::IslandArc as u8;
    let tectonic_gain = 0.82 + 0.18 * maturity + 0.22 * shortening;
    let broad_transmission = 0.72 + 0.28 * (1.0 - resistance);

    if subduction {
        // Back-arc extension is a conditional broad subsidence tendency, not a mandatory marine
        // trench behind every volcanic arc.  Tie its modest deflection to the actual arc load.
        let arc_load = (0.55 * mountain_core + 0.45 * volcanic_arc).clamp(0.0, 1.0);
        let backarc_deflection = 260.0 * backarc * (0.25 + 0.75 * arc_load);
        let arc_relief = 1_500.0 * mountain_core.powf(1.08)
            + volcanic_arc * (2_450.0 + 800.0 * maturity)
            + 360.0 * fold
            + 120.0 * intensity
            - backarc_deflection;
        return (0.0, arc_relief);
    }

    let crust_scale = match inherited.crust_kind[index] {
        CRUST_OCEANIC => 0.18,
        CRUST_TRANSITIONAL => 0.62,
        _ => 1.0,
    };
    // Foreland subsidence is flexural response to an actual mountain load.  v13 treated the
    // foreland index itself as a -1.35 km topographic command, creating a continuous below-sea
    // moat beside almost every range.  Keep the basin broad and shallow unless a substantial load
    // exists, while moving more collision relief into the crustal root/hinterland.
    let mountain_load = (0.58 * mountain_core + 0.27 * root + 0.15 * fold).clamp(0.0, 1.0);
    let foreland_deflection = 320.0 * foreland * mountain_load.powf(1.20);
    // Boundary-first modern plates distribute collision systems more evenly and expose broad
    // low-core portions of accretion belts that the old ownership-growth geometry often buried
    // inside a larger domain. Preserve those mechanically active continental belts with a
    // moderate crustal-thickening pedestal rather than forcing every accepted orogen to depend
    // on a narrow mountain-core raster. Terrane accretion receives the stronger support because
    // its added crust is mechanically real even where the topographic core remains coastal.
    let continental_collision_pedestal = if kind == OrogenProvinceKind::ContinentalCollision as u8 {
        820.0 * intensity * (0.55 + 0.45 * shortening)
    } else {
        0.0
    };
    let terrane_accretion_pedestal = if kind == OrogenProvinceKind::TerraneAccretion as u8 {
        3_000.0 * intensity * (0.55 + 0.45 * maturity)
    } else {
        0.0
    };
    let collision_relief = crust_scale
        * tectonic_gain
        * (4_300.0 * mountain_core.powf(1.10)
            + 3_050.0 * root * broad_transmission
            + 1_350.0 * plateau * broad_transmission
            + 1_150.0 * fold
            + 2_000.0 * transpression
            + 360.0 * intensity
            + continental_collision_pedestal
            + terrane_accretion_pedestal
            - foreland_deflection
            - 70.0 * suture);
    (collision_relief, 0.0)
}

pub fn generate_initial_topography(
    topology: &GeodesicTopology,
    inherited: &InheritedPhysicalState,
    boundaries: &InheritedBoundarySet,
    planet: PlanetPhysicalParameters,
    request: &TopographyRequest,
) -> Result<TopographyState, WorldgenError> {
    request
        .parameters
        .validate()
        .map_err(WorldgenError::InvalidTopography)?;

    // Compute the accepted non-orogenic WG-4 components, then discard the legacy radial
    // collision/arc fields before constructing the final surface.
    let mut baseline = crate::topography::generate_initial_topography(
        topology, inherited, boundaries, planet, request,
    )?;
    relax_quiet_provenance_isostasy(
        topology,
        inherited,
        &mut baseline.isostatic_elevation_m,
    );
    let count = topology.metrics().sample_count as usize;
    if inherited.orogenic_history.len() != count
        || inherited.crustal_root_index.len() != count
        || inherited.mountain_core_index.len() != count
        || inherited.boundary_distance_km.len() != count
        || inherited.province_ids.len() != count
    {
        return Err(WorldgenError::InvalidTopography(
            "tectonic orogen province inheritance is not aligned to WG-4 topology",
        ));
    }

    let mut orogenic = vec![0.0_f64; count];
    let mut arc = vec![0.0_f64; count];
    let mut raw = vec![0.0_f64; count];
    for i in 0..count {
        let (collision_relief, arc_relief) = province_relief(inherited, i);
        orogenic[i] = collision_relief;
        arc[i] = arc_relief;

        // Preserve the actual crustal-isostatic state.  The causal cut already discards the
        // legacy *orogenic elevation* field; attenuating all isostatic support wherever legacy
        // orogenic history was strong carved an artificial low corridor around the replacement
        // range.  Thick continental crust remains buoyant regardless of which relief model owns
        // the active mountain load.
        raw[i] = f64::from(baseline.isostatic_elevation_m[i])
            + f64::from(baseline.thermal_elevation_m[i])
            + orogenic[i]
            + f64::from(baseline.ridge_elevation_m[i])
            + f64::from(baseline.rift_basin_elevation_m[i])
            + f64::from(baseline.trench_elevation_m[i])
            + arc[i]
            + f64::from(baseline.mantle_dynamic_elevation_m[i]);
    }

    let mut solid = mechanically_filter(topology, &raw, inherited, request.parameters);
    let datum = area_weighted_mean(&solid, topology.dual_area_steradians());
    for value in &mut solid {
        *value -= datum;
    }

    let mut clamped_sample_count = 0_u32;
    for value in &mut solid {
        let clamped = value.clamp(-20_000.0, 15_000.0);
        if clamped.to_bits() != value.to_bits() {
            clamped_sample_count += 1;
            *value = clamped;
        }
    }

    // Global sea level may only inundate terrain that is reached from oceanic crust through
    // a below-water path.  Closed continental depressions can remain below the global datum
    // without becoming magic inland ocean; WG-6 is responsible for their lake hydrology.
    let solid_f32 = solid.iter().map(|value| *value as f32).collect::<Vec<_>>();
    // Use the old threshold solve only to discover broad submerged oceanic reservoirs.  It does
    // not define final water state.  This prevents every tiny oceanic crust remnant from becoming
    // an independent marine-water source inside a collision zone.
    let provisional =
        crate::surface_water::solve_hydrostatic_surface_water_f64(topology, &solid, planet)?;
    let ocean_seed_mask = major_ocean_reservoir_seed_mask(
        topology,
        &inherited.crust_kind,
        &provisional.submerged_mask,
    );
    let water = crate::surface_water::solve_hydrostatic_surface_water_connected_f64(
        topology,
        &solid,
        planet,
        &ocean_seed_mask,
    )?;
    let sea_level = water.metrics.sea_level_m;
    let mut land_area = 0.0;
    let mut ocean_area = 0.0;
    let mut land_elevation_area_sum = 0.0;
    let mut water_depth_area_sum = 0.0;
    let mut maximum_water_depth_m = 0.0_f64;
    for i in 0..count {
        let area = topology.dual_area_steradians()[i];
        if water.submerged_mask[i] != 0 {
            ocean_area += area;
            let depth = f64::from(water.water_depth_m[i]);
            water_depth_area_sum += depth * area;
            maximum_water_depth_m = maximum_water_depth_m.max(depth);
        } else {
            land_area += area;
            land_elevation_area_sum += f64::from(water.elevation_above_sea_level_m[i]) * area;
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

    let stage_seed = derive_stage_seed(&request.seed, TECTONIC_TOPOGRAPHY_NAMESPACE);
    let parameter_hash = request.parameters.parameter_hash();
    let mut topography_hash = FNV_OFFSET_BASIS;
    topography_hash = fnv_update(topography_hash, TECTONIC_TOPOGRAPHY_STAGE_ID.as_bytes());
    topography_hash = fnv_update(
        topography_hash,
        &TECTONIC_TOPOGRAPHY_STAGE_VERSION.to_le_bytes(),
    );
    topography_hash = fnv_update(topography_hash, &stage_seed.to_le_bytes());
    topography_hash = fnv_update(topography_hash, &parameter_hash.to_le_bytes());
    topography_hash = fnv_update(topography_hash, &planet.parameter_hash().to_le_bytes());
    topography_hash = fnv_update(
        topography_hash,
        &inherited.causal_inheritance_hash.to_le_bytes(),
    );
    topography_hash = fnv_update(topography_hash, &boundaries.boundary_hash.to_le_bytes());
    topography_hash = fnv_update(
        topography_hash,
        &inherited.orogen_province_hash.to_le_bytes(),
    );
    for value in &solid {
        topography_hash = fnv_update(topography_hash, &value.to_bits().to_le_bytes());
    }
    for value in &water.water_depth_m {
        topography_hash = fnv_update(topography_hash, &value.to_bits().to_le_bytes());
    }

    let minimum_solid_elevation_m = solid.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum_solid_elevation_m = solid.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let metrics = TopographyMetrics {
        sample_count: count as u32,
        minimum_solid_elevation_m,
        maximum_solid_elevation_m,
        mean_solid_elevation_m: area_weighted_mean(&solid, topology.dual_area_steradians()),
        p05_solid_elevation_m: area_weighted_quantile(
            &solid,
            topology.dual_area_steradians(),
            0.05,
        ),
        median_solid_elevation_m: area_weighted_quantile(
            &solid,
            topology.dual_area_steradians(),
            0.50,
        ),
        p95_solid_elevation_m: area_weighted_quantile(
            &solid,
            topology.dual_area_steradians(),
            0.95,
        ),
        sea_level_m: sea_level,
        land_area_fraction,
        ocean_area_fraction,
        mean_land_elevation_m,
        mean_water_depth_m,
        maximum_water_depth_m,
        target_water_volume_m3: planet.surface_water_volume_m3(),
        solved_water_volume_m3: water.metrics.solved_water_volume_m3,
        water_volume_relative_error: water.metrics.water_volume_relative_error,
        clamped_sample_count,
        parameter_hash,
        topography_hash,
    };

    Ok(TopographyState {
        stage: StageIdentity {
            id: TECTONIC_TOPOGRAPHY_STAGE_ID,
            version: TECTONIC_TOPOGRAPHY_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        metrics,
        isostatic_elevation_m: baseline.isostatic_elevation_m,
        thermal_elevation_m: baseline.thermal_elevation_m,
        orogenic_elevation_m: orogenic.into_iter().map(|value| value as f32).collect(),
        ridge_elevation_m: baseline.ridge_elevation_m,
        rift_basin_elevation_m: baseline.rift_basin_elevation_m,
        trench_elevation_m: baseline.trench_elevation_m,
        arc_elevation_m: arc.into_iter().map(|value| value as f32).collect(),
        mantle_dynamic_elevation_m: baseline.mantle_dynamic_elevation_m,
        solid_elevation_m: solid_f32,
        elevation_above_sea_level_m: water.elevation_above_sea_level_m,
        water_depth_m: water.water_depth_m,
        submerged_mask: water.submerged_mask,
    })
}
