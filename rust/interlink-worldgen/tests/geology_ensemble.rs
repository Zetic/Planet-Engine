use interlink_worldgen::{
    build_icosphere, generate_crust_and_history, generate_tectonics, CrustKind,
    GeologicalBoundaryRegime, GeologyRequest, PlanetPhysicalParameters, SubductionPolarity,
    TectonicsRequest,
};
use std::collections::BTreeSet;

#[test]
fn multi_seed_geology_retains_physical_crust_separation_and_causal_history() {
    let topology = build_icosphere(4).expect("WG-3 ensemble topology");
    let parameters = PlanetPhysicalParameters::earthlike_reference();
    let seeds = [
        "wg3-ensemble-a",
        "wg3-ensemble-b",
        "wg3-ensemble-c",
        "wg3-ensemble-d",
        "wg3-ensemble-e",
    ];

    let mut regime_population = BTreeSet::new();
    let mut mixed_plate_worlds = 0_usize;
    let mut ridge_near_age_sum = 0.0_f64;
    let mut ridge_near_count = 0_usize;
    let mut ridge_far_age_sum = 0.0_f64;
    let mut ridge_far_count = 0_usize;

    for seed in seeds {
        let tectonics = generate_tectonics(&topology, &TectonicsRequest::new(seed, 16), parameters)
            .expect("WG-3 ensemble tectonics");
        let tectonic_hash_before = tectonics.metrics.tectonic_hash;
        let geology = generate_crust_and_history(
            &topology,
            &tectonics,
            &GeologyRequest::new(seed),
            parameters,
        )
        .expect("WG-3 ensemble geology");

        assert_eq!(
            tectonic_hash_before, tectonics.metrics.tectonic_hash,
            "WG-3 must not mutate accepted WG-2 truth"
        );
        assert!((0.28..=0.44).contains(&geology.metrics.continental_area_fraction));
        assert!((0.045..=0.13).contains(&geology.metrics.transitional_area_fraction));
        assert!(geology.metrics.oceanic_area_fraction >= 0.43);
        assert!(
            geology.metrics.mean_continental_age_myr > geology.metrics.mean_oceanic_age_myr * 3.0
        );
        assert!(
            geology.metrics.mean_continental_thickness_km
                > geology.metrics.mean_oceanic_thickness_km * 3.0
        );
        assert!(geology.metrics.mean_oceanic_age_myr <= 220.0);

        if geology
            .plate_summaries
            .iter()
            .any(|plate| plate.continental_fraction > 0.05 && plate.oceanic_fraction > 0.05)
        {
            mixed_plate_worlds += 1;
        }

        for boundary in &geology.boundaries {
            regime_population.insert(boundary.regime as u8);
            if matches!(
                boundary.regime,
                GeologicalBoundaryRegime::OceanicSubduction
                    | GeologicalBoundaryRegime::OceanContinentSubduction
            ) {
                assert_ne!(
                    boundary.subduction_polarity,
                    SubductionPolarity::None,
                    "subduction must identify a subducting side"
                );
            }
        }

        for sample in 0..geology.crust_kind.len() {
            if geology.crust_kind[sample] != CrustKind::Oceanic as u8 {
                continue;
            }
            let ridge = geology.ridge_history[sample];
            let age = f64::from(geology.crust_age_myr[sample]);
            if ridge >= 0.60 {
                ridge_near_age_sum += age;
                ridge_near_count += 1;
            } else if ridge <= 0.12 {
                ridge_far_age_sum += age;
                ridge_far_count += 1;
            }
        }
    }

    assert!(mixed_plate_worlds >= 4, "at least four reference worlds should demonstrate that plate identity and crust type are independent");
    assert!(
        regime_population.len() >= 5,
        "the ensemble should exercise several distinct geological boundary regimes"
    );
    assert!(
        ridge_near_count > 0 && ridge_far_count > 0,
        "the ensemble must contain oceanic samples both near and far from spreading influence"
    );
    let ridge_near_age = ridge_near_age_sum / ridge_near_count as f64;
    let ridge_far_age = ridge_far_age_sum / ridge_far_count as f64;
    assert!(ridge_near_age < ridge_far_age * 0.75, "oceanic crust near ridges should be materially younger than crust far from ridges: near={ridge_near_age:.2} Myr far={ridge_far_age:.2} Myr");
}
