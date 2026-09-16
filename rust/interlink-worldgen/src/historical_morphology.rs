use crate::{
    derive_stage_seed, CrustKind, HistoricalEventKind, HistoricalLithosphereModel, PlanetTopology,
    StageIdentity, TectonicModel, WorldgenError,
};

pub const HISTORICAL_MORPHOLOGY_STAGE_ID: &str = "geology:historical-tectonic-morphology";
pub const HISTORICAL_MORPHOLOGY_STAGE_VERSION: u32 = 1;
const HISTORICAL_MORPHOLOGY_NAMESPACE: &str = "worldgen:geology:historical-tectonic-morphology:v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Debug, PartialEq)]
pub struct HistoricalMorphologyMetrics {
    pub sample_count: u32,
    pub event_count: u32,
    pub active_orogen_sample_count: u32,
    pub fossil_orogen_sample_count: u32,
    pub internal_fossil_orogen_sample_count: u32,
    pub passive_margin_sample_count: u32,
    pub suture_sample_count: u32,
    pub rift_sample_count: u32,
    pub shear_sample_count: u32,
    pub morphology_hash: u64,
}

impl HistoricalMorphologyMetrics {
    pub fn morphology_hash_hex(&self) -> String {
        format!("{:016x}", self.morphology_hash)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoricalMorphologyModel {
    pub stage: StageIdentity,
    pub latest_event_kind: Vec<u8>,
    pub latest_event_age_myr: Vec<f32>,
    pub rift_intensity: Vec<f32>,
    pub rift_age_myr: Vec<f32>,
    pub shear_intensity: Vec<f32>,
    pub shear_age_myr: Vec<f32>,
    pub suture_intensity: Vec<f32>,
    pub suture_age_myr: Vec<f32>,
    pub accretion_intensity: Vec<f32>,
    pub passive_margin_index: Vec<f32>,
    pub active_orogen_intensity: Vec<f32>,
    pub fossil_orogen_intensity: Vec<f32>,
    pub cumulative_shortening_km: Vec<f32>,
    pub cumulative_extension_km: Vec<f32>,
    pub cumulative_shear_km: Vec<f32>,
    pub metrics: HistoricalMorphologyMetrics,
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn hash_f32(mut hash: u64, values: &[f32]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    for value in values {
        hash = fnv_update(hash, &value.to_bits().to_le_bytes());
    }
    hash
}

fn hash_u8(mut hash: u64, values: &[u8]) -> u64 {
    hash = fnv_update(hash, &(values.len() as u64).to_le_bytes());
    fnv_update(hash, values)
}

fn memory_from_age(age_myr: f32, scale_myr: f64) -> f64 {
    1.0 / (1.0 + f64::from(age_myr.max(0.0)) / scale_myr.max(1.0))
}

fn seed_pair(signal: &mut [f32], a: u32, b: u32, value: f64) {
    let value = clamp01(value) as f32;
    signal[a as usize] = signal[a as usize].max(value);
    signal[b as usize] = signal[b as usize].max(value);
}

fn seed_pair_with_age(
    signal: &mut [f32],
    age: &mut [f32],
    a: u32,
    b: u32,
    value: f64,
    source_age_myr: f32,
) {
    let value = clamp01(value) as f32;
    for sample in [a, b] {
        let index = sample as usize;
        if value > signal[index]
            || (value.to_bits() == signal[index].to_bits()
                && (age[index] < 0.0 || source_age_myr < age[index]))
        {
            signal[index] = value;
            age[index] = source_age_myr;
        }
    }
}

fn diffuse_signal<T: PlanetTopology>(
    topology: &T,
    signal: &[f32],
    domain_ids: &[u16],
    passes: usize,
    retention: f64,
    cross_domain_factor: f64,
) -> Vec<f32> {
    let mut current = signal.to_vec();
    let mut next = current.clone();
    for _ in 0..passes {
        next.copy_from_slice(&current);
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            let domain = domain_ids[index];
            let mut strongest = f64::from(current[index]);
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                let domain_factor = if domain_ids[ni] == domain {
                    1.0
                } else {
                    cross_domain_factor
                };
                strongest = strongest.max(f64::from(current[ni]) * retention * domain_factor);
            }
            next[index] = clamp01(strongest) as f32;
        }
        std::mem::swap(&mut current, &mut next);
    }
    current
}

fn diffuse_signal_with_age<T: PlanetTopology>(
    topology: &T,
    signal: &[f32],
    age_myr: &[f32],
    domain_ids: &[u16],
    passes: usize,
    retention: f64,
    cross_domain_factor: f64,
) -> (Vec<f32>, Vec<f32>) {
    let mut current_signal = signal.to_vec();
    let mut current_age = age_myr.to_vec();
    let mut next_signal = current_signal.clone();
    let mut next_age = current_age.clone();
    for _ in 0..passes {
        next_signal.copy_from_slice(&current_signal);
        next_age.copy_from_slice(&current_age);
        for sample in 0..topology.sample_count() {
            let index = sample as usize;
            let domain = domain_ids[index];
            let mut best_signal = f64::from(current_signal[index]);
            let mut best_age = current_age[index];
            for neighbor in topology.neighbors(sample) {
                let ni = *neighbor as usize;
                let domain_factor = if domain_ids[ni] == domain {
                    1.0
                } else {
                    cross_domain_factor
                };
                let candidate = f64::from(current_signal[ni]) * retention * domain_factor;
                if candidate > best_signal + 1.0e-9
                    || ((candidate - best_signal).abs() <= 1.0e-9
                        && current_age[ni] >= 0.0
                        && (best_age < 0.0 || current_age[ni] < best_age))
                {
                    best_signal = candidate;
                    best_age = current_age[ni];
                }
            }
            next_signal[index] = clamp01(best_signal) as f32;
            next_age[index] = best_age;
        }
        std::mem::swap(&mut current_signal, &mut next_signal);
        std::mem::swap(&mut current_age, &mut next_age);
    }
    (current_signal, current_age)
}

fn validate_inputs<T: PlanetTopology>(
    topology: &T,
    historical: &HistoricalLithosphereModel,
    tectonics: &TectonicModel,
) -> Result<(), WorldgenError> {
    let count = topology.sample_count() as usize;
    if historical.fragment_ids.len() != count
        || historical.current_plate_ids.len() != count
        || historical.crust_kind.len() != count
        || tectonics.plate_ids.len() != count
    {
        return Err(WorldgenError::InvalidLithosphere(
            "historical morphology inputs do not match topology",
        ));
    }
    if tectonics.plate_ids != historical.current_plate_ids {
        return Err(WorldgenError::InvalidLithosphere(
            "historical morphology requires modern ownership projected from historical material",
        ));
    }
    if historical.events.iter().any(|event| {
        event.geometry_sample_a >= topology.sample_count()
            || event.geometry_sample_b >= topology.sample_count()
    }) {
        return Err(WorldgenError::InvalidLithosphere(
            "historical morphology event geometry references an invalid sample",
        ));
    }
    Ok(())
}

pub fn build_historical_tectonic_morphology<T: PlanetTopology>(
    topology: &T,
    historical: &HistoricalLithosphereModel,
    tectonics: &TectonicModel,
    seed: &str,
) -> Result<HistoricalMorphologyModel, WorldgenError> {
    validate_inputs(topology, historical, tectonics)?;
    let count = topology.sample_count() as usize;
    let stage_seed = derive_stage_seed(seed, HISTORICAL_MORPHOLOGY_NAMESPACE);

    let mut latest_event_kind = vec![0_u8; count];
    let mut latest_event_age_myr = vec![-1.0_f32; count];
    let mut rift_seed = vec![0.0_f32; count];
    let mut rift_age_seed = vec![-1.0_f32; count];
    let mut shear_seed = vec![0.0_f32; count];
    let mut shear_age_seed = vec![-1.0_f32; count];
    let mut suture_seed = vec![0.0_f32; count];
    let mut suture_age_seed = vec![-1.0_f32; count];
    let mut accretion_seed = vec![0.0_f32; count];
    let mut fossil_orogen_seed = vec![0.0_f32; count];
    let mut shortening_seed = vec![0.0_f32; count];
    let mut extension_seed = vec![0.0_f32; count];
    let mut shear_displacement_seed = vec![0.0_f32; count];

    for event in &historical.events {
        let a = event.geometry_sample_a;
        let b = event.geometry_sample_b;
        let age = event.age_myr.max(0.0);
        let strength = f64::from(event.strength).clamp(0.0, 1.0);
        let memory = memory_from_age(age, 180.0);
        let event_signal = clamp01(0.24 + strength * 0.76) * memory;

        for sample in [a, b] {
            let index = sample as usize;
            if latest_event_age_myr[index] < 0.0 || age < latest_event_age_myr[index] {
                latest_event_age_myr[index] = age;
                latest_event_kind[index] = event.kind as u8;
            }
        }

        match event.kind {
            HistoricalEventKind::Rift | HistoricalEventKind::Spreading => {
                seed_pair_with_age(
                    &mut rift_seed,
                    &mut rift_age_seed,
                    a,
                    b,
                    event_signal,
                    age,
                );
                let extension = event.displacement_km.max(0.0);
                for sample in [a, b] {
                    extension_seed[sample as usize] += extension * 0.5;
                }
            }
            HistoricalEventKind::Transform => {
                seed_pair_with_age(
                    &mut shear_seed,
                    &mut shear_age_seed,
                    a,
                    b,
                    event_signal,
                    age,
                );
                let displacement = event.displacement_km.max(0.0);
                for sample in [a, b] {
                    shear_displacement_seed[sample as usize] += displacement * 0.5;
                }
            }
            HistoricalEventKind::Collision => {
                seed_pair_with_age(
                    &mut suture_seed,
                    &mut suture_age_seed,
                    a,
                    b,
                    event_signal.max(0.30),
                    age,
                );
                seed_pair(
                    &mut fossil_orogen_seed,
                    a,
                    b,
                    event_signal.max(0.24),
                );
                let shortening = event.displacement_km.max(0.0);
                for sample in [a, b] {
                    shortening_seed[sample as usize] += shortening * 0.5;
                }
            }
            HistoricalEventKind::Subduction => {
                seed_pair(
                    &mut fossil_orogen_seed,
                    a,
                    b,
                    event_signal * 0.78,
                );
                let shortening = event.displacement_km.max(0.0);
                for sample in [a, b] {
                    shortening_seed[sample as usize] += shortening * 0.38;
                }
            }
            HistoricalEventKind::Accretion => {
                seed_pair(&mut accretion_seed, a, b, event_signal.max(0.28));
                seed_pair(
                    &mut fossil_orogen_seed,
                    a,
                    b,
                    event_signal * 0.70,
                );
            }
            HistoricalEventKind::Capture => {
                seed_pair(&mut accretion_seed, a, b, event_signal * 0.48);
            }
        }
    }

    let (rift_intensity, rift_age_myr) = diffuse_signal_with_age(
        topology,
        &rift_seed,
        &rift_age_seed,
        &historical.fragment_ids,
        5,
        0.80,
        0.34,
    );
    let (shear_intensity, shear_age_myr) = diffuse_signal_with_age(
        topology,
        &shear_seed,
        &shear_age_seed,
        &historical.fragment_ids,
        4,
        0.78,
        0.52,
    );
    let (suture_intensity, suture_age_myr) = diffuse_signal_with_age(
        topology,
        &suture_seed,
        &suture_age_seed,
        &historical.fragment_ids,
        6,
        0.84,
        0.88,
    );
    let accretion_intensity = diffuse_signal(
        topology,
        &accretion_seed,
        &historical.fragment_ids,
        4,
        0.80,
        0.72,
    );
    let fossil_orogen_intensity = diffuse_signal(
        topology,
        &fossil_orogen_seed,
        &historical.fragment_ids,
        7,
        0.87,
        0.76,
    );

    let mut active_seed = vec![0.0_f32; count];
    let mut current_boundary_mask = vec![0_u8; count];
    for boundary in &tectonics.boundaries {
        current_boundary_mask[boundary.sample_a as usize] = 1;
        current_boundary_mask[boundary.sample_b as usize] = 1;
        if boundary.kind == crate::PlateBoundaryKind::Convergent {
            let rate = (-boundary.normal_rate_m_per_year).max(0.0);
            let intensity = clamp01(0.22 + rate / 0.075);
            seed_pair(
                &mut active_seed,
                boundary.sample_a,
                boundary.sample_b,
                intensity,
            );
        }
    }
    let active_orogen_intensity = diffuse_signal(
        topology,
        &active_seed,
        &historical.current_plate_ids,
        5,
        0.84,
        0.78,
    );

    let mut passive_margin_seed = vec![0.0_f32; count];
    for sample in 0..topology.sample_count() {
        let index = sample as usize;
        if historical.crust_kind[index] == CrustKind::Oceanic as u8 {
            continue;
        }
        let mut ocean_contact = 0.0_f64;
        for neighbor in topology.neighbors(sample) {
            let ni = *neighbor as usize;
            if historical.crust_kind[ni] == CrustKind::Oceanic as u8 {
                ocean_contact = ocean_contact.max(
                    f64::from(rift_intensity[index]).max(f64::from(rift_intensity[ni])),
                );
            }
        }
        passive_margin_seed[index] = clamp01(ocean_contact) as f32;
    }
    let passive_margin_index = diffuse_signal(
        topology,
        &passive_margin_seed,
        &historical.fragment_ids,
        3,
        0.80,
        0.18,
    );

    let cumulative_shortening_km = diffuse_signal(
        topology,
        &shortening_seed
            .iter()
            .map(|value| (f64::from(*value) / 1200.0).clamp(0.0, 1.0) as f32)
            .collect::<Vec<_>>(),
        &historical.fragment_ids,
        4,
        0.80,
        0.70,
    )
    .into_iter()
    .map(|value| value * 1200.0)
    .collect::<Vec<_>>();
    let cumulative_extension_km = diffuse_signal(
        topology,
        &extension_seed
            .iter()
            .map(|value| (f64::from(*value) / 1000.0).clamp(0.0, 1.0) as f32)
            .collect::<Vec<_>>(),
        &historical.fragment_ids,
        4,
        0.80,
        0.38,
    )
    .into_iter()
    .map(|value| value * 1000.0)
    .collect::<Vec<_>>();
    let cumulative_shear_km = diffuse_signal(
        topology,
        &shear_displacement_seed
            .iter()
            .map(|value| (f64::from(*value) / 1200.0).clamp(0.0, 1.0) as f32)
            .collect::<Vec<_>>(),
        &historical.fragment_ids,
        4,
        0.80,
        0.52,
    )
    .into_iter()
    .map(|value| value * 1200.0)
    .collect::<Vec<_>>();

    let active_orogen_sample_count = active_orogen_intensity
        .iter()
        .filter(|value| **value >= 0.20)
        .count() as u32;
    let fossil_orogen_sample_count = fossil_orogen_intensity
        .iter()
        .filter(|value| **value >= 0.20)
        .count() as u32;
    let internal_fossil_orogen_sample_count = fossil_orogen_intensity
        .iter()
        .enumerate()
        .filter(|(index, value)| **value >= 0.20 && current_boundary_mask[*index] == 0)
        .count() as u32;
    let passive_margin_sample_count = passive_margin_index
        .iter()
        .filter(|value| **value >= 0.20)
        .count() as u32;
    let suture_sample_count = suture_intensity
        .iter()
        .filter(|value| **value >= 0.20)
        .count() as u32;
    let rift_sample_count = rift_intensity
        .iter()
        .filter(|value| **value >= 0.20)
        .count() as u32;
    let shear_sample_count = shear_intensity
        .iter()
        .filter(|value| **value >= 0.20)
        .count() as u32;

    let mut morphology_hash = FNV_OFFSET_BASIS;
    morphology_hash = fnv_update(morphology_hash, b"geology:historical-tectonic-morphology:v1\0");
    morphology_hash = fnv_update(morphology_hash, &stage_seed.to_le_bytes());
    morphology_hash = fnv_update(morphology_hash, &historical.metrics.history_hash.to_le_bytes());
    morphology_hash = fnv_update(morphology_hash, &tectonics.metrics.tectonic_hash.to_le_bytes());
    morphology_hash = hash_u8(morphology_hash, &latest_event_kind);
    for values in [
        &latest_event_age_myr,
        &rift_intensity,
        &rift_age_myr,
        &shear_intensity,
        &shear_age_myr,
        &suture_intensity,
        &suture_age_myr,
        &accretion_intensity,
        &passive_margin_index,
        &active_orogen_intensity,
        &fossil_orogen_intensity,
        &cumulative_shortening_km,
        &cumulative_extension_km,
        &cumulative_shear_km,
    ] {
        morphology_hash = hash_f32(morphology_hash, values);
    }

    Ok(HistoricalMorphologyModel {
        stage: StageIdentity {
            id: HISTORICAL_MORPHOLOGY_STAGE_ID,
            version: HISTORICAL_MORPHOLOGY_STAGE_VERSION,
            derived_seed: stage_seed,
        },
        latest_event_kind,
        latest_event_age_myr,
        rift_intensity,
        rift_age_myr,
        shear_intensity,
        shear_age_myr,
        suture_intensity,
        suture_age_myr,
        accretion_intensity,
        passive_margin_index,
        active_orogen_intensity,
        fossil_orogen_intensity,
        cumulative_shortening_km,
        cumulative_extension_km,
        cumulative_shear_km,
        metrics: HistoricalMorphologyMetrics {
            sample_count: topology.sample_count(),
            event_count: historical.events.len() as u32,
            active_orogen_sample_count,
            fossil_orogen_sample_count,
            internal_fossil_orogen_sample_count,
            passive_margin_sample_count,
            suture_sample_count,
            rift_sample_count,
            shear_sample_count,
            morphology_hash,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_icosphere, generate_historical_frontend, HistoricalLithosphereRequest, PlanetPhysicalParameters};

    #[test]
    fn morphology_is_deterministic_and_contains_internal_fossil_structure() {
        let topology = build_icosphere(4).unwrap();
        let planet = PlanetPhysicalParameters::earthlike_reference();
        let frontend = generate_historical_frontend(
            &topology,
            &HistoricalLithosphereRequest::new("historical-morphology-test", 16),
            planet,
        )
        .unwrap();
        let a = build_historical_tectonic_morphology(
            &topology,
            &frontend.historical,
            &frontend.tectonics,
            "historical-morphology-test",
        )
        .unwrap();
        let b = build_historical_tectonic_morphology(
            &topology,
            &frontend.historical,
            &frontend.tectonics,
            "historical-morphology-test",
        )
        .unwrap();
        assert_eq!(a.metrics.morphology_hash, b.metrics.morphology_hash);
        assert_eq!(a.metrics.sample_count, topology.sample_count());
        assert!(a.metrics.rift_sample_count > 0);
        assert!(a.metrics.suture_sample_count > 0);
        assert!(a.metrics.active_orogen_sample_count > 0);
        assert!(a.metrics.fossil_orogen_sample_count > 0);
        assert!(a.metrics.internal_fossil_orogen_sample_count > 0);
    }
}
