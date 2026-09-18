use crate::StageIdentity;

pub const GEOLOGY_STAGE_ID: &str = "geology:crust-history";
pub const GEOLOGY_STAGE_VERSION: u32 = 5;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrustKind {
    Oceanic = 1,
    Transitional = 2,
    Continental = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlateScaleClass {
    Major = 1,
    Intermediate = 2,
    Minor = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeologicalBoundaryRegime {
    OceanicSubduction = 1,
    OceanContinentSubduction = 2,
    ContinentalCollision = 3,
    OceanicRidge = 4,
    ContinentalRift = 5,
    TransitionalDivergence = 6,
    Transform = 7,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubductionPolarity {
    None = 0,
    PlateA = 1,
    PlateB = 2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeologyRequest {
    pub seed: String,
}

impl GeologyRequest {
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeologicalBoundary {
    pub sample_a: u32,
    pub sample_b: u32,
    pub plate_a: u16,
    pub plate_b: u16,
    pub regime: GeologicalBoundaryRegime,
    pub subduction_polarity: SubductionPolarity,
    pub normal_rate_m_per_year: f64,
    pub shear_rate_m_per_year: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlateSummary {
    pub plate_id: u16,
    pub area_steradians: f64,
    pub area_fraction: f64,
    pub scale_class: PlateScaleClass,
    pub continental_fraction: f64,
    pub transitional_fraction: f64,
    pub oceanic_fraction: f64,
    pub mean_crust_age_myr: f64,
    pub mean_crust_thickness_km: f64,
    pub convergent_boundary_fraction: f64,
    pub divergent_boundary_fraction: f64,
    pub transform_boundary_fraction: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GeologyMetrics {
    pub sample_count: u32,
    pub continental_area_fraction: f64,
    pub transitional_area_fraction: f64,
    pub oceanic_area_fraction: f64,
    /// Compatibility summary of continental basement age. This is historical metadata, not a
    /// direct topographic control.
    pub mean_continental_age_myr: f64,
    pub mean_oceanic_age_myr: f64,
    pub mean_continental_reworking_age_myr: f64,
    pub mean_continental_thickness_km: f64,
    pub mean_oceanic_thickness_km: f64,
    pub oceanic_subduction_edges: u32,
    pub ocean_continent_subduction_edges: u32,
    pub continental_collision_edges: u32,
    pub oceanic_ridge_edges: u32,
    pub continental_rift_edges: u32,
    pub transitional_divergence_edges: u32,
    pub transform_edges: u32,
    pub geology_hash: u64,
}

impl GeologyMetrics {
    pub fn geology_hash_hex(&self) -> String {
        format!("{:016x}", self.geology_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CrustalModel {
    pub stage: StageIdentity,
    pub province_seed: u64,
    pub property_seed: u64,
    pub history_seed: u64,

    pub crust_kind: Vec<u8>,
    /// Persistent ancestry/provenance only. A province boundary is not itself a mechanical or
    /// topographic discontinuity.
    pub crust_province_id: Vec<u16>,

    /// Compatibility composite age. Oceanic cells carry seafloor age; continental cells carry
    /// basement formation age. Consumers that need physical behavior should use the explicit
    /// clocks below rather than this mixed field.
    pub crust_age_myr: Vec<f32>,
    /// Time since creation at a spreading system. Zero on non-oceanic material.
    pub oceanic_age_myr: Vec<f32>,
    /// Formation age of continental basement. Zero on oceanic material. Historical metadata only.
    pub continental_basement_age_myr: Vec<f32>,
    /// Time since the last major tectonothermal reworking event. On stable continental material
    /// with no younger event this approaches basement age.
    pub last_tectonic_reworking_age_myr: Vec<f32>,
    /// Bounded present-day stability derived from reworking age and tectonic disturbance.
    /// Intended for rheology/lithospheric-root state, not direct elevation.
    pub continental_stability_index: Vec<f32>,

    pub crust_thickness_km: Vec<f32>,
    pub crust_density_kg_per_m3: Vec<f32>,
    pub buoyancy_index: Vec<f32>,
    pub orogenic_history: Vec<f32>,
    pub rift_history: Vec<f32>,
    pub ridge_history: Vec<f32>,
    pub subduction_history: Vec<f32>,
    pub trench_history: Vec<f32>,
    pub volcanic_arc_history: Vec<f32>,
    pub transform_history: Vec<f32>,
    pub subsidence_history: Vec<f32>,
    pub basin_potential: Vec<f32>,
    pub crustal_strain: Vec<f32>,
    pub boundaries: Vec<GeologicalBoundary>,
    pub plate_summaries: Vec<PlateSummary>,
    pub metrics: GeologyMetrics,
}

impl CrustalModel {
    pub fn boundary_regimes(&self) -> Vec<u8> {
        self.boundaries
            .iter()
            .map(|edge| edge.regime as u8)
            .collect()
    }

    pub fn subduction_polarities(&self) -> Vec<u8> {
        self.boundaries
            .iter()
            .map(|edge| edge.subduction_polarity as u8)
            .collect()
    }

    pub fn plate_scale_classes(&self) -> Vec<u8> {
        self.plate_summaries
            .iter()
            .map(|plate| plate.scale_class as u8)
            .collect()
    }
}
