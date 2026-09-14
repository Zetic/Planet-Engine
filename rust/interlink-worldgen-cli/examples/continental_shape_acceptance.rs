use interlink_worldgen::{
    analyze_continental_morphology, build_icosphere, generate_crust_and_history,
    generate_tectonics, GeologyRequest, PlanetPhysicalParameters, TectonicsRequest,
};

fn main() -> Result<(), String> {
    let topology = build_icosphere(4).map_err(|error| error.to_string())?;
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let seeds = [
        "continental-shape-a",
        "continental-shape-b",
        "continental-shape-c",
        "continental-shape-d",
        "interlink-wg7c",
        "3",
    ];
    let mut hierarchy_worlds = 0_usize;
    let mut elongated_worlds = 0_usize;
    let mut noncompact_worlds = 0_usize;
    let mut multiplate_worlds = 0_usize;
    let mut cv_sum = 0.0_f64;
    let mut satellite_area_sum = 0.0_f64;
    let mut constricted_sum = 0.0_f64;
    let mut fine_complexity_sum = 0.0_f64;
    let mut medium_complexity_sum = 0.0_f64;
    let mut coarse_complexity_sum = 0.0_f64;

    for seed in seeds {
        let tectonics = generate_tectonics(&topology, &TectonicsRequest::new(seed, 16), planet)
            .map_err(|error| error.to_string())?;
        let geology =
            generate_crust_and_history(&topology, &tectonics, &GeologyRequest::new(seed), planet)
                .map_err(|error| error.to_string())?;
        let morphology =
            analyze_continental_morphology(&topology, &geology.crust_kind, &tectonics.plate_ids)
                .map_err(|error| error.to_string())?;

        if morphology.significant_component_count < 2 {
            return Err(format!(
                "{seed}: continental assembly collapsed to fewer than two significant components"
            ));
        }

        let cv = morphology.component_area_coefficient_of_variation;
        let hierarchy = morphology.largest_to_median_area_ratio;
        let max_elongation = morphology.maximum_elongation;
        let max_compactness = morphology.maximum_compactness;
        let multiplate = morphology.has_major_multiplate_component;
        let bounded_fractions = [
            morphology.satellite_area_fraction,
            morphology.secondary_complement_area_fraction,
            morphology.constricted_sample_fraction,
        ];
        if bounded_fractions
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(format!(
                "{seed}: continental morphology emitted a non-finite or out-of-range fraction"
            ));
        }
        let complexity_values = [
            morphology.coastline_complexity_fine,
            morphology.coastline_complexity_medium,
            morphology.coastline_complexity_coarse,
        ];
        if complexity_values
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(format!(
                "{seed}: continental morphology emitted invalid coastline complexity"
            ));
        }

        cv_sum += cv;
        satellite_area_sum += morphology.satellite_area_fraction;
        constricted_sum += morphology.constricted_sample_fraction;
        fine_complexity_sum += morphology.coastline_complexity_fine;
        medium_complexity_sum += morphology.coastline_complexity_medium;
        coarse_complexity_sum += morphology.coastline_complexity_coarse;
        hierarchy_worlds += usize::from(hierarchy >= 1.80);
        elongated_worlds += usize::from(max_elongation >= 1.25);
        noncompact_worlds += usize::from(max_compactness >= 1.15);
        multiplate_worlds += usize::from(multiplate);

        println!(
            "{seed}: significant={} all={} CV={cv:.3} largest/median={hierarchy:.3} max_elong={max_elongation:.3} max_compact={max_compactness:.3} multiplate={multiplate} satellites={} satellite_area={:.4} constricted={:.4} secondary_complement={} secondary_complement_area={:.4} coast_complexity(f/m/c)={:.3}/{:.3}/{:.3} smoothing={}/{}",
            morphology.significant_component_count,
            morphology.all_component_count,
            morphology.satellite_component_count,
            morphology.satellite_area_fraction,
            morphology.constricted_sample_fraction,
            morphology.secondary_complement_component_count,
            morphology.secondary_complement_area_fraction,
            morphology.coastline_complexity_fine,
            morphology.coastline_complexity_medium,
            morphology.coastline_complexity_coarse,
            morphology.medium_smoothing_rounds,
            morphology.coarse_smoothing_rounds,
        );
    }

    let coupling_seed = "continental-tectonic-coupling";
    let tectonics_12 =
        generate_tectonics(&topology, &TectonicsRequest::new(coupling_seed, 12), planet)
            .map_err(|error| error.to_string())?;
    let tectonics_20 =
        generate_tectonics(&topology, &TectonicsRequest::new(coupling_seed, 20), planet)
            .map_err(|error| error.to_string())?;
    let geology_12 = generate_crust_and_history(
        &topology,
        &tectonics_12,
        &GeologyRequest::new(coupling_seed),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let geology_20 = generate_crust_and_history(
        &topology,
        &tectonics_20,
        &GeologyRequest::new(coupling_seed),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let changed_fraction = geology_12
        .crust_kind
        .iter()
        .zip(geology_20.crust_kind.iter())
        .filter(|(a, b)| a != b)
        .count() as f64
        / geology_12.crust_kind.len() as f64;
    println!("plate-layout coupling changed crust-kind fraction={changed_fraction:.4}");

    let world_count = seeds.len() as f64;
    let mean_cv = cv_sum / world_count;
    let mean_satellite_area = satellite_area_sum / world_count;
    let mean_constricted = constricted_sum / world_count;
    let mean_fine_complexity = fine_complexity_sum / world_count;
    let mean_medium_complexity = medium_complexity_sum / world_count;
    let mean_coarse_complexity = coarse_complexity_sum / world_count;
    println!(
        "ensemble observability: mean_satellite_area={mean_satellite_area:.4} mean_constricted={mean_constricted:.4} mean_coast_complexity(f/m/c)={mean_fine_complexity:.3}/{mean_medium_complexity:.3}/{mean_coarse_complexity:.3}",
    );

    if hierarchy_worlds < 4 {
        return Err(format!(
            "only {hierarchy_worlds}/6 worlds show a material continental size hierarchy"
        ));
    }
    if elongated_worlds < 4 {
        return Err(format!(
            "only {elongated_worlds}/6 worlds contain a materially elongated continental component"
        ));
    }
    if noncompact_worlds < 4 {
        return Err(format!(
            "only {noncompact_worlds}/6 worlds contain a non-circular continental outline"
        ));
    }
    if multiplate_worlds < 3 {
        return Err(format!(
            "only {multiplate_worlds}/6 worlds assemble a major continent across tectonic domains"
        ));
    }
    if mean_cv < 0.55 {
        return Err(format!(
            "ensemble continental component-size CV remained too uniform: {mean_cv:.3}"
        ));
    }
    // WG-3 v3 must materially improve the graph-scale defects measured by the
    // observability-only v2 baseline while still retaining coarse continental form.
    if mean_satellite_area > 0.0018 {
        return Err(format!(
            "ensemble satellite continental area remains too fragmented: {mean_satellite_area:.4}"
        ));
    }
    if mean_constricted > 0.0175 {
        return Err(format!(
            "ensemble continental masks retain too many graph-scale constrictions: {mean_constricted:.4}"
        ));
    }
    if mean_coarse_complexity < 6.0 {
        return Err(format!(
            "ensemble coarse continental complexity collapsed toward overly simple macro-shapes: {mean_coarse_complexity:.3}"
        ));
    }
    if changed_fraction < 0.03 {
        return Err(format!("continental partition remains effectively independent of tectonic plate layout: changed={changed_fraction:.4}"));
    }
    Ok(())
}
