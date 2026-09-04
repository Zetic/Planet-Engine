mod boundary_refinement;
mod climate;
mod climate_calibration;
mod climate_multiresolution;
mod coordinates;
mod diagnostics;
mod drainage;
mod fields;
mod geology;
mod hydroclimate;
mod lakes;
mod lithosphere;
mod parameters;
mod random;
mod refinement;
mod runoff;
mod seasonal;
mod seasonal_lakes;
mod tectonics;
mod topography;
mod topology;

use std::fmt;

pub use boundary_refinement::{
    inherit_boundary_interfaces, InheritedBoundaryEdge, InheritedBoundarySet,
};
pub use climate::{
    generate_coupled_climate, generate_coupled_climate_reference_with_diagnostics,
    generate_coupled_climate_with_diagnostics, ClimateGenerationDiagnostics, ClimateMetrics,
    ClimateParameters, ClimatePhysicalParameters, ClimateRequest, ClimateState, CLIMATE_STAGE_ID,
    CLIMATE_STAGE_VERSION,
};
pub use climate_calibration::{
    build_climate_calibration_report, ClimateCalibrationReport, ClimateLatitudeBand,
};
pub use coordinates::{
    anchor_origin_cartesian, cartesian_to_local_enu, great_circle_distance_m,
    lat_lon_degrees_from_unit_vector, local_enu_to_cartesian, tangent_basis,
    unit_vector_from_lat_lon_degrees, LocalEnuPosition, SurfaceAnchor, TangentBasis,
};
pub use diagnostics::{FieldStatistics, StageIdentity};
pub use drainage::{
    generate_drainage_topology, DrainageBasin, DrainageDepression, DrainageMetrics,
    DrainageRequest, DrainageState, DRAINAGE_OUTLET_INTERNAL, DRAINAGE_OUTLET_NONE,
    DRAINAGE_OUTLET_OCEAN, DRAINAGE_STAGE_ID, DRAINAGE_STAGE_VERSION,
};
pub use fields::{DenseU16Field, MAX_SYNTHETIC_SAMPLES};
pub use geology::{
    generate_crust_and_history, CrustKind, CrustalModel, GeologicalBoundary,
    GeologicalBoundaryRegime, GeologyMetrics, GeologyRequest, PlateScaleClass, PlateSummary,
    SubductionPolarity, GEOLOGY_STAGE_ID, GEOLOGY_STAGE_VERSION,
};
pub use hydroclimate::{
    build_hydroclimate_closure_report, HydroclimateClosureReport, HydroclimateLatitudeBand,
};
pub use lakes::{
    generate_lakes_closed_basins, LakeMetrics, LakeParameters, LakeRecord, LakeRequest, LakeState,
    LAKE_KIND_ENDORHEIC, LAKE_KIND_NONE, LAKE_KIND_OVERFLOWING, LAKE_KIND_TERMINAL_STORAGE,
    LAKE_STAGE_ID, LAKE_STAGE_VERSION,
};
pub use lithosphere::{
    generate_lithosphere, LithosphereMetrics, LithosphereRequest, LithosphericModel,
    StructuralZoneKind, TectonicFragment, TectonicFragmentKind, LITHOSPHERE_STAGE_ID,
    LITHOSPHERE_STAGE_VERSION, MAX_TECTONIC_FRAGMENTS,
};
pub use parameters::PlanetPhysicalParameters;
pub use random::derive_stage_seed;
pub use refinement::{
    build_refinement_map, inherit_physical_state, refine_categorical_u16, refine_categorical_u8,
    refine_scalar_f32, refine_scalar_f32_with_domains, refine_scalar_f64, refine_vector3_f64,
    InheritedPhysicalState, RefinementMap, RefinementMetrics, MULTIRES_STAGE_ID,
    MULTIRES_STAGE_VERSION,
};
pub use runoff::{
    generate_runoff_discharge, RunoffMetrics, RunoffParameters, RunoffRequest, RunoffState,
    RUNOFF_STAGE_ID, RUNOFF_STAGE_VERSION,
};
pub use seasonal::{
    generate_seasonal_hydrology, SeasonalHydrologyMetrics, SeasonalHydrologyParameters,
    SeasonalHydrologyRequest, SeasonalHydrologyState, SEASONAL_HYDROLOGY_STAGE_ID,
    SEASONAL_HYDROLOGY_STAGE_VERSION,
};
pub use tectonics::{
    generate_tectonics, PlateBoundaryEdge, PlateBoundaryKind, TectonicMetrics, TectonicModel,
    TectonicPlate, TectonicsRequest, MAX_TECTONIC_PLATES, MIN_TECTONIC_PLATES, TECTONICS_STAGE_ID,
    TECTONICS_STAGE_VERSION,
};
pub use topography::{
    generate_initial_topography, TopographyMetrics, TopographyParameters, TopographyRequest,
    TopographyState, TOPOGRAPHY_STAGE_ID, TOPOGRAPHY_STAGE_VERSION,
};
pub use topology::{
    build_icosphere, expected_edge_count, expected_face_count, expected_sample_count,
    GeodesicTopology, PlanetTopology, TopologyMetrics, INVALID_SAMPLE_ID, MAX_TOPOLOGY_LEVEL,
};

pub const WORLDGEN_ENGINE_VERSION: u32 = 9;
pub const SYNTHETIC_STAGE_ID: &str = "foundation:synthetic";
pub const SYNTHETIC_STAGE_VERSION: u32 = 1;
const SYNTHETIC_NAMESPACE: &str = "worldgen:foundation:synthetic:v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldgenError {
    InvalidDimensions(&'static str),
    InvalidParameters(&'static str),
    InvalidTopology(&'static str),
    InvalidCoordinate(&'static str),
    InvalidTectonics(&'static str),
    InvalidGeology(&'static str),
    InvalidLithosphere(&'static str),
    InvalidRefinement(&'static str),
    InvalidTopography(&'static str),
    InvalidClimate(&'static str),
    InvalidHydrology(&'static str),
}
impl fmt::Display for WorldgenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions(message)
            | Self::InvalidParameters(message)
            | Self::InvalidTopology(message)
            | Self::InvalidCoordinate(message)
            | Self::InvalidTectonics(message)
            | Self::InvalidGeology(message)
            | Self::InvalidLithosphere(message)
            | Self::InvalidRefinement(message)
            | Self::InvalidTopography(message)
            | Self::InvalidClimate(message)
            | Self::InvalidHydrology(message) => formatter.write_str(message),
        }
    }
}
impl std::error::Error for WorldgenError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntheticRequest {
    pub seed: String,
    pub width: u32,
    pub height: u32,
}
impl SyntheticRequest {
    pub fn new(seed: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            seed: seed.into(),
            width,
            height,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct SyntheticDiagnostic {
    pub generator_version: u32,
    pub stage: StageIdentity,
    pub parameters: PlanetPhysicalParameters,
    pub field: DenseU16Field,
    pub statistics: FieldStatistics,
}

fn triangular_wave(index: u32, period: u32) -> u32 {
    if period <= 1 {
        return 0;
    }
    let doubled = u64::from(index) * 2 * u64::from(u16::MAX);
    let scaled = (doubled / u64::from(period - 1)) as u32;
    if scaled <= u32::from(u16::MAX) {
        scaled
    } else {
        2 * u32::from(u16::MAX) - scaled
    }
}

/// WG-0 proof field. This is intentionally not terrain. It remains available as a transport/determinism regression while later stages introduce physical planetary state.
pub fn generate_synthetic(
    request: &SyntheticRequest,
) -> Result<SyntheticDiagnostic, WorldgenError> {
    let parameters = PlanetPhysicalParameters::earthlike_reference();
    parameters
        .validate()
        .map_err(WorldgenError::InvalidParameters)?;
    let count = fields::checked_sample_count(request.width, request.height)?;
    let stage_seed = random::derive_stage_seed(&request.seed, SYNTHETIC_NAMESPACE);
    let mut values = Vec::with_capacity(count);
    for y in 0..request.height {
        let lat_shape = triangular_wave(y, request.height);
        for x in 0..request.width {
            let lon_shape = triangular_wave(x, request.width);
            let noise = (random::coordinate_value(stage_seed, x, y) & 0xffff) as u32;
            let structured = (u32::from(u16::MAX) - lat_shape) / 2 + lon_shape / 3;
            values.push(((noise * 3 + structured * 2) / 5).min(u32::from(u16::MAX)) as u16);
        }
    }
    let field = DenseU16Field::from_values(request.width, request.height, values)?;
    let statistics = field.statistics();
    Ok(SyntheticDiagnostic {
        generator_version: WORLDGEN_ENGINE_VERSION,
        stage: StageIdentity {
            id: SYNTHETIC_STAGE_ID,
            version: SYNTHETIC_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        parameters,
        field,
        statistics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn synthetic_generation_is_deterministic_and_seed_sensitive() {
        let request = SyntheticRequest::new("wg0-regression", 96, 48);
        let a = generate_synthetic(&request).unwrap();
        let b = generate_synthetic(&request).unwrap();
        assert_eq!(a.statistics.hash, b.statistics.hash);
        assert_eq!(a.field.values(), b.field.values());
        let changed =
            generate_synthetic(&SyntheticRequest::new("wg0-regression-b", 96, 48)).unwrap();
        assert_ne!(a.statistics.hash, changed.statistics.hash);
    }
    #[test]
    fn synthetic_generation_reports_dense_field_statistics() {
        let result = generate_synthetic(&SyntheticRequest::new("stats", 32, 16)).unwrap();
        assert_eq!(result.generator_version, WORLDGEN_ENGINE_VERSION);
        assert_eq!(result.stage.id, SYNTHETIC_STAGE_ID);
        assert_eq!(result.statistics.sample_count, 512);
        assert!(result.statistics.minimum <= result.statistics.maximum);
        assert!(result.statistics.mean.is_finite());
        assert_eq!(result.statistics.hash_hex().len(), 16);
    }
}
