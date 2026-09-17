use interlink_worldgen::{
    build_icosphere, generate_historical_frontend, generate_lithology_substrate,
    generate_lithosphere_from_history, inherit_historical_identity, inherit_physical_state,
    BedrockClass, CrustKind, HistoricalLithosphereRequest, LithologyRequest, LithosphereRequest,
    PlanetPhysicalParameters, PlanetTopology,
};
use std::collections::{BTreeMap, BTreeSet};

const SEEDS: [&str; 4] = [
    "interlink-wg7c",
    "1",
    "2",
    "lithology-substrate-holdout",
];

#[derive(Debug)]
struct Report {
    seed: &'static str,
    class_count: usize,
    hard_strength: f64,
    soft_strength: f64,
    hard_erodibility: f64,
    soft_erodibility: f64,
    carbonate_mean: f64,
    fragment_strength_range: f64,
    boundary_contrast_ratio: f64,
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
    let state = generate_lithology_substrate(
        &fine,
        &inherited,
        &identity,
        &LithologyRequest::new(seed),
    )
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
        return Err(format!("{seed}: lithology fields do not cover fine topology"));
    }
    for field in fields {
        if field.iter().any(|value| !value.is_finite() || !(0.0..=1.0).contains(value)) {
            return Err(format!("{seed}: lithology property escaped normalized bounds"));
        }
    }

    let classes = state.bedrock_class.iter().copied().collect::<BTreeSet<_>>();
    if classes.len() < 5 {
        return Err(format!("{seed}: lithology collapsed to {} classes", classes.len()));
    }

    for sample in 0..count {
        let class = state.bedrock_class[sample];
        let oceanic_class = class == BedrockClass::OceanicBasalt as u8
            || class == BedrockClass::OceanicSediment as u8;
        if inherited.crust_kind[sample] == CrustKind::Oceanic as u8 && !oceanic_class {
            return Err(format!("{seed}: oceanic crust received continental bedrock class"));
        }
        if inherited.crust_kind[sample] != CrustKind::Oceanic as u8 && oceanic_class {
            return Err(format!("{seed}: non-oceanic crust received oceanic bedrock class"));
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
        return Err(format!("{seed}: carbonate platforms lack carbonate identity"));
    }

    let mut fragments = BTreeMap::<u16, (f64, u64)>::new();
    for sample in 0..count {
        let entry = fragments.entry(identity.fragment_ids[sample]).or_default();
        entry.0 += f64::from(state.rock_strength_index[sample]);
        entry.1 += 1;
    }
    let fragment_means = fragments
        .values()
        .filter(|(_, n)| *n >= 8)
        .map(|(sum, n)| sum / *n as f64)
        .collect::<Vec<_>>();
    let fragment_strength_range = fragment_means
        .iter()
        .copied()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), value| {
            (lo.min(value), hi.max(value))
        });
    let fragment_strength_range = if fragment_means.is_empty() {
        0.0
    } else {
        fragment_strength_range.1 - fragment_strength_range.0
    };
    if fragment_strength_range < 0.08 {
        return Err(format!("{seed}: fragment provenance is not materially legible"));
    }

    let mut same_sum = 0.0_f64;
    let mut same_edges = 0_u64;
    let mut cross_sum = 0.0_f64;
    let mut cross_edges = 0_u64;
    for sample in 0..fine.sample_count() {
        for neighbor in fine.neighbors(sample) {
            if *neighbor <= sample {
                continue;
            }
            let a = sample as usize;
            let b = *neighbor as usize;
            let contrast = (f64::from(state.rock_strength_index[a])
                - f64::from(state.rock_strength_index[b]))
            .abs()
                + (f64::from(state.erodibility_index[a])
                    - f64::from(state.erodibility_index[b]))
                .abs()
                + (f64::from(state.carbonate_fraction[a])
                    - f64::from(state.carbonate_fraction[b]))
                .abs();
            if identity.fragment_ids[a] == identity.fragment_ids[b] {
                same_sum += contrast;
                same_edges += 1;
            } else {
                cross_sum += contrast;
                cross_edges += 1;
            }
        }
    }
    let same_mean = same_sum / same_edges.max(1) as f64;
    let cross_mean = cross_sum / cross_edges.max(1) as f64;
    let boundary_contrast_ratio = cross_mean / same_mean.max(1.0e-9);
    if boundary_contrast_ratio < 0.85 {
        return Err(format!(
            "{seed}: material boundaries are less legible than fragment interiors ({boundary_contrast_ratio:.2}x)"
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
        fragment_strength_range,
        boundary_contrast_ratio,
        hash: state.metrics.lithology_hash_hex(),
    })
}

fn main() {
    let mut failures = Vec::new();
    for seed in SEEDS {
        match run_seed(seed) {
            Ok(report) => println!(
                "WG-4.5 lithology: seed={} classes={} strength={:.3}/{:.3} erod={:.3}/{:.3} carbonate={:.3} fragment-range={:.3} boundary={:.2}x hash={}",
                report.seed,
                report.class_count,
                report.hard_strength,
                report.soft_strength,
                report.hard_erodibility,
                report.soft_erodibility,
                report.carbonate_mean,
                report.fragment_strength_range,
                report.boundary_contrast_ratio,
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
