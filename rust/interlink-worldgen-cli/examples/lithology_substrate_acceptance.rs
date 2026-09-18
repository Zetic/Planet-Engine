use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, generate_lithology_substrate,
    generate_lithosphere_from_history, inherit_historical_identity, inherit_physical_state,
    BedrockClass, CrustKind, HistoricalLithosphereRequest, LithologyRequest, LithosphereRequest,
    PlanetPhysicalParameters, PlanetTopology,
};
use std::collections::BTreeSet;

const SEEDS: [&str; 4] = ["interlink-wg7c", "1", "2", "lithology-substrate-holdout"];

#[derive(Debug)]
struct Report {
    seed: &'static str,
    class_count: usize,
    hard_strength: f64,
    soft_strength: f64,
    hard_erodibility: f64,
    soft_erodibility: f64,
    carbonate_mean: f64,
    ancestry_relabel_invariant: bool,
    hash: String,
}

fn mean(values: &[f32], mask: impl Fn(usize) -> bool) -> Option<f64> {
    let mut sum = 0.0_f64;
    let mut count = 0_u64;
    for (index, value) in values.iter().enumerate() {
        if mask(index) {
            sum += f64::from(*value);
            count += 1;
        }
    }
    (count > 0).then_some(sum / count as f64)
}

fn run_seed(seed: &'static str) -> Result<Report, String> {
    let planet = PlanetPhysicalParameters::earthlike_reference();
    let coarse_level = 4_u8;
    let fine_level = 6_u8;
    let coarse = build_icosphere(coarse_level).map_err(|error| error.to_string())?;
    let fine = build_icosphere(fine_level).map_err(|error| error.to_string())?;
    let frontend = generate_historical_frontend(
        &coarse,
        &HistoricalLithosphereRequest::new(seed, 16),
        planet,
    )
    .map_err(|error| error.to_string())?;
    let lithosphere = generate_lithosphere_from_history(
        &coarse,
        &frontend.historical,
        &frontend.tectonics,
        &frontend.geology,
        &LithosphereRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;
    let inherited = inherit_physical_state(
        &fine,
        coarse_level,
        &frontend.tectonics,
        &frontend.geology,
        &lithosphere,
        planet,
    )
    .map_err(|error| error.to_string())?;
    let identity = inherit_historical_identity(&fine, coarse_level, &frontend.historical)
        .map_err(|error| error.to_string())?;
    let state =
        generate_lithology_substrate(&fine, &inherited, &identity, &LithologyRequest::new(seed))
            .map_err(|error| error.to_string())?;

    let count = fine.sample_count() as usize;
    let fields: [&[f32]; 6] = [
        &state.rock_strength_index,
        &state.erodibility_index,
        &state.permeability_index,
        &state.weathering_susceptibility,
        &state.fines_fraction,
        &state.carbonate_fraction,
    ];
    if state.bedrock_class.len() != count || fields.iter().any(|field| field.len() != count) {
        return Err(format!(
            "{seed}: lithology fields do not cover fine topology"
        ));
    }
    for field in fields {
        if field
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(format!(
                "{seed}: lithology property escaped normalized bounds"
            ));
        }
    }

    let classes = state.bedrock_class.iter().copied().collect::<BTreeSet<_>>();
    if classes.len() < 5 {
        return Err(format!(
            "{seed}: lithology collapsed to {} classes",
            classes.len()
        ));
    }

    for sample in 0..count {
        let class = state.bedrock_class[sample];
        let oceanic_class = class == BedrockClass::OceanicBasalt as u8
            || class == BedrockClass::OceanicSediment as u8;
        if inherited.crust_kind[sample] == CrustKind::Oceanic as u8 && !oceanic_class {
            return Err(format!(
                "{seed}: oceanic crust received continental bedrock class"
            ));
        }
        if inherited.crust_kind[sample] != CrustKind::Oceanic as u8 && oceanic_class {
            return Err(format!(
                "{seed}: non-oceanic crust received oceanic bedrock class"
            ));
        }
    }

    let hard = |index: usize| {
        let class = state.bedrock_class[index];
        class == BedrockClass::CrystallineBasement as u8
            || class == BedrockClass::OrogenicMetamorphic as u8
    };
    let soft = |index: usize| {
        let class = state.bedrock_class[index];
        class == BedrockClass::ClasticSedimentary as u8
            || class == BedrockClass::OceanicSediment as u8
    };
    let hard_strength = mean(&state.rock_strength_index, hard)
        .ok_or_else(|| format!("{seed}: no hard-rock samples"))?;
    let soft_strength = mean(&state.rock_strength_index, soft)
        .ok_or_else(|| format!("{seed}: no sedimentary samples"))?;
    let hard_erodibility = mean(&state.erodibility_index, hard).unwrap();
    let soft_erodibility = mean(&state.erodibility_index, soft).unwrap();
    if hard_strength <= soft_strength + 0.12 {
        return Err(format!(
            "{seed}: substrate strength contrast is too weak ({hard_strength:.3} vs {soft_strength:.3})"
        ));
    }
    if soft_erodibility <= hard_erodibility + 0.12 {
        return Err(format!(
            "{seed}: substrate erodibility contrast is too weak ({soft_erodibility:.3} vs {hard_erodibility:.3})"
        ));
    }

    let carbonate_mean = mean(&state.carbonate_fraction, |index| {
        state.bedrock_class[index] == BedrockClass::CarbonatePlatform as u8
    })
    .unwrap_or(0.0);
    if classes.contains(&(BedrockClass::CarbonatePlatform as u8)) && carbonate_mean < 0.60 {
        return Err(format!(
            "{seed}: carbonate platforms lack carbonate identity"
        ));
    }

    // Genealogical IDs must be observational only. Deterministically relabel ancestral plate and
    // fragment identities while holding every inherited physical field fixed; WG-4.5 substrate
    // must remain bit-identical.
    let mut relabeled_identity = identity.clone();
    for value in &mut relabeled_identity.origin_plate_ids {
        *value = value.wrapping_mul(61).wrapping_add(7);
    }
    for value in &mut relabeled_identity.fragment_ids {
        *value = value.wrapping_mul(73).wrapping_add(19);
    }
    let relabeled_state = generate_lithology_substrate(
        &fine,
        &inherited,
        &relabeled_identity,
        &LithologyRequest::new(seed),
    )
    .map_err(|error| error.to_string())?;
    let ancestry_relabel_invariant = relabeled_state == state;
    if !ancestry_relabel_invariant {
        return Err(format!(
            "{seed}: categorical ancestry labels leaked into lithology physics"
        ));
    }

    Ok(Report {
        seed,
        class_count: classes.len(),
        hard_strength,
        soft_strength,
        hard_erodibility,
        soft_erodibility,
        carbonate_mean,
        ancestry_relabel_invariant,
        hash: state.metrics.lithology_hash_hex(),
    })
}

fn main() {
    let mut failures = Vec::new();
    for seed in SEEDS {
        match run_seed(seed) {
            Ok(report) => println!(
                "WG-4.5 lithology: seed={} classes={} strength={:.3}/{:.3} erod={:.3}/{:.3} carbonate={:.3} ancestry-relabel={} hash={}",
                report.seed,
                report.class_count,
                report.hard_strength,
                report.soft_strength,
                report.hard_erodibility,
                report.soft_erodibility,
                report.carbonate_mean,
                report.ancestry_relabel_invariant,
                report.hash,
            ),
            Err(error) => failures.push(error),
        }
    }
    if failures.is_empty() {
        return;
    }
    for failure in failures {
        eprintln!("{failure}");
    }
    std::process::exit(1);
}
