use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_initial_topography, generate_lithosphere,
    generate_tectonics, inherit_boundary_interfaces, inherit_physical_state,
    GeologicalBoundaryRegime, GeologyRequest, LithosphereRequest, PlanetPhysicalParameters,
    TectonicsRequest, TopographyRequest,
};

const CRUST_CONTINENTAL: u8 = 3;
const STRUCTURE_SUTURE: u8 = 1;
const INHERITED_OROGENY_SCALE_M: f64 = 1_200.0;

#[derive(Clone, Copy, Debug, Default)]
struct WeightedMoments {
    weight: f64,
    sum: f64,
    sum_sq: f64,
}

impl WeightedMoments {
    fn add(&mut self, value: f64, weight: f64) {
        if !value.is_finite() || !weight.is_finite() || weight <= 0.0 {
            return;
        }
        self.weight += weight;
        self.sum += value * weight;
        self.sum_sq += value * value * weight;
    }

    fn mean(self) -> f64 {
        if self.weight > 0.0 {
            self.sum / self.weight
        } else {
            0.0
        }
    }

    fn cv(self) -> f64 {
        let mean = self.mean();
        if self.weight <= 0.0 || mean.abs() <= 1.0e-12 {
            return 0.0;
        }
        let variance = (self.sum_sq / self.weight - mean * mean).max(0.0);
        variance.sqrt() / mean.abs()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct InheritedOrogenProfile {
    continental_area_m2: f64,
    core: WeightedMoments,
    shoulder: WeightedMoments,
    background: WeightedMoments,
    response_factor: WeightedMoments,
    suture_factor: WeightedMoments,
    nonsuture_factor: WeightedMoments,
    strong_factor: WeightedMoments,
    weak_factor: WeightedMoments,
}

impl InheritedOrogenProfile {
    fn shoulder_to_core(self) -> f64 {
        let core = self.core.mean();
        if core > 0.0 {
            self.shoulder.mean() / core
        } else {
            0.0
        }
    }

    fn background_to_core(self) -> f64 {
        let core = self.core.mean();
        if core > 0.0 {
            self.background.mean() / core
        } else {
            0.0
        }
    }
}

#[derive(Default)]
struct EnsembleSummary {
    measured: u32,
    core_sum_m: f64,
    maximum_shoulder_ratio: f64,
    maximum_background_ratio: f64,
    response_cv_sum: f64,
    minimum_response_cv: f64,
    minimum_suture_to_nonsuture: f64,
    maximum_suture_to_nonsuture: f64,
    minimum_strong_to_weak: f64,
}

impl EnsembleSummary {
    fn absorb(&mut self, profile: InheritedOrogenProfile) {
        if profile.core.weight <= 0.0 || profile.response_factor.weight <= 0.0 {
            return;
        }
        let response_cv = profile.response_factor.cv();
        let suture_to_nonsuture = if profile.nonsuture_factor.mean() > 0.0 {
            profile.suture_factor.mean() / profile.nonsuture_factor.mean()
        } else {
            0.0
        };
        let strong_to_weak = if profile.weak_factor.mean() > 0.0 {
            profile.strong_factor.mean() / profile.weak_factor.mean()
        } else {
            0.0
        };
        self.measured += 1;
        self.core_sum_m += profile.core.mean();
        self.maximum_shoulder_ratio = self.maximum_shoulder_ratio.max(profile.shoulder_to_core());
        self.maximum_background_ratio = self
            .maximum_background_ratio
            .max(profile.background_to_core());
        self.response_cv_sum += response_cv;
        self.minimum_response_cv = if self.measured == 1 {
            response_cv
        } else {
            self.minimum_response_cv.min(response_cv)
        };
        self.minimum_suture_to_nonsuture = if self.measured == 1 {
            suture_to_nonsuture
        } else if suture_to_nonsuture > 0.0 {
            self.minimum_suture_to_nonsuture.min(suture_to_nonsuture)
        } else {
            self.minimum_suture_to_nonsuture
        };
        self.maximum_suture_to_nonsuture =
            self.maximum_suture_to_nonsuture.max(suture_to_nonsuture);
        self.minimum_strong_to_weak = if self.measured == 1 {
            strong_to_weak
        } else if strong_to_weak > 0.0 {
            self.minimum_strong_to_weak.min(strong_to_weak)
        } else {
            self.minimum_strong_to_weak
        };
    }

    fn mean_core_m(&self) -> f64 {
        self.core_sum_m / f64::from(self.measured.max(1))
    }

    fn mean_response_cv(&self) -> f64 {
        self.response_cv_sum / f64::from(self.measured.max(1))
    }
}

fn inherited_profile(
    topology: &interlink_worldgen::GeodesicTopology,
    inherited: &interlink_worldgen::InheritedPhysicalState,
    terrain: &interlink_worldgen::TopographyState,
    planet: PlanetPhysicalParameters,
) -> InheritedOrogenProfile {
    let mut profile = InheritedOrogenProfile::default();
    for index in 0..inherited.orogenic_history.len() {
        if inherited.crust_kind[index] != CRUST_CONTINENTAL {
            continue;
        }
        let area = topology.dual_area_steradians()[index] * planet.radius_m * planet.radius_m;
        profile.continental_area_m2 += area;
        let history = f64::from(inherited.orogenic_history[index]).clamp(0.0, 1.0);
        let relief = f64::from(terrain.orogenic_elevation_m[index]).max(0.0);
        if history >= 0.70 {
            profile.core.add(relief, area);
        } else if (0.30..0.50).contains(&history) {
            profile.shoulder.add(relief, area);
        } else if (0.10..0.20).contains(&history) {
            profile.background.add(relief, area);
        }
        if history >= 0.25 {
            let factor = relief / (INHERITED_OROGENY_SCALE_M * history).max(1.0e-9);
            profile.response_factor.add(factor, area);
            if inherited.structural_zone_kind[index] == STRUCTURE_SUTURE {
                profile.suture_factor.add(factor, area);
            } else {
                profile.nonsuture_factor.add(factor, area);
            }
            let te_km = f64::from(inherited.effective_elastic_thickness_km[index]);
            if te_km >= 48.0 {
                profile.strong_factor.add(factor, area);
            } else if te_km <= 28.0 {
                profile.weak_factor.add(factor, area);
            }
        }
    }
    profile
}

fn main() -> Result<(), String> {
    let calibration = [
        "3",
        "interlink-wg7c",
        "wg4-boundary-a",
        "wg4-boundary-b",
        "wg4-boundary-c",
        "wg4-boundary-d",
    ];
    let holdout = [
        "1",
        "2",
        "wg4-morphology-holdout-a",
        "wg4-morphology-holdout-b",
        "wg4-morphology-holdout-c",
        "wg4-morphology-holdout-d",
    ];
    let coarse_level = 5_u8;
    let fine_level = 7_u8;
    let plates = 16_u16;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;
    let mut summary = EnsembleSummary::default();

    for (cohort, seeds) in [
        ("calibration", calibration.as_slice()),
        ("holdout", holdout.as_slice()),
    ] {
        for seed in seeds {
            let tectonics =
                generate_tectonics(&coarse, &TectonicsRequest::new(*seed, plates), planet)
                    .map_err(|error| error.to_string())?;
            let geology = generate_crust_and_history(
                &coarse,
                &tectonics,
                &GeologyRequest::new(*seed),
                planet,
            )
            .map_err(|error| error.to_string())?;
            let lithosphere = generate_lithosphere(
                &coarse,
                &tectonics,
                &geology,
                &LithosphereRequest::new(*seed),
            )
            .map_err(|error| error.to_string())?;
            let inherited = inherit_physical_state(
                &fine,
                coarse_level,
                &tectonics,
                &geology,
                &lithosphere,
                planet,
            )
            .map_err(|error| error.to_string())?;
            let mut boundaries = inherit_boundary_interfaces(
                &coarse,
                &fine,
                &tectonics,
                &geology,
                &inherited.plate_ids,
            )
            .map_err(|error| error.to_string())?;
            boundaries.boundaries.retain(|edge| {
                edge.geological_regime != GeologicalBoundaryRegime::ContinentalCollision
            });
            boundaries.boundary_hash ^= 0x6b51_2f9c_173d_a804;
            let terrain = generate_initial_topography(
                &fine,
                &inherited,
                &boundaries,
                planet,
                &TopographyRequest::new(*seed),
            )
            .map_err(|error| error.to_string())?;
            let profile = inherited_profile(&fine, &inherited, &terrain, planet);
            let suture_ratio = if profile.nonsuture_factor.mean() > 0.0 {
                profile.suture_factor.mean() / profile.nonsuture_factor.mean()
            } else {
                0.0
            };
            let strong_ratio = if profile.weak_factor.mean() > 0.0 {
                profile.strong_factor.mean() / profile.weak_factor.mean()
            } else {
                0.0
            };
            println!(
                "WG-4 inherited orogen morphology cohort={cohort} seed={seed} core={:.2}m shoulder/core={:.4} background/core={:.4} response_cv={:.4} suture/non={:.4} strong/weak={:.4}",
                profile.core.mean(),
                profile.shoulder_to_core(),
                profile.background_to_core(),
                profile.response_factor.cv(),
                suture_ratio,
                strong_ratio,
            );
            summary.absorb(profile);
        }
    }

    println!(
        "WG-4 inherited orogen morphology acceptance: seeds={} mean_core={:.2}m max_shoulder/core={:.4} max_background/core={:.4} mean_response_cv={:.4} min_response_cv={:.4} min_suture/non={:.4} max_suture/non={:.4} min_strong/weak={:.4}",
        summary.measured,
        summary.mean_core_m(),
        summary.maximum_shoulder_ratio,
        summary.maximum_background_ratio,
        summary.mean_response_cv(),
        summary.minimum_response_cv,
        summary.minimum_suture_to_nonsuture,
        summary.maximum_suture_to_nonsuture,
        summary.minimum_strong_to_weak,
    );

    if summary.measured < 12 {
        return Err(format!(
            "inherited orogen morphology exercised only {} worlds",
            summary.measured
        ));
    }
    if summary.mean_core_m() < 1_000.0 {
        return Err(format!(
            "inherited orogen cores became too weak: mean {:.2} m",
            summary.mean_core_m()
        ));
    }
    if summary.maximum_shoulder_ratio > 0.40 {
        return Err(format!(
            "inherited orogen shoulders remain too broad: ratio {:.4}",
            summary.maximum_shoulder_ratio
        ));
    }
    if summary.maximum_background_ratio > 0.145 {
        return Err(format!(
            "inherited orogen background remains too broad: ratio {:.4}",
            summary.maximum_background_ratio
        ));
    }
    if summary.minimum_response_cv < 0.07 {
        return Err(format!(
            "inherited orogen response remains too uniform: minimum CV {:.4}",
            summary.minimum_response_cv
        ));
    }
    if summary.minimum_suture_to_nonsuture < 1.05 {
        return Err(format!(
            "suture structure no longer strengthens inherited orogens: minimum ratio {:.4}",
            summary.minimum_suture_to_nonsuture
        ));
    }
    Ok(())
}
