use crate::{
    derive_stage_seed, CrustFragment, HistoricalEventKind, HistoricalLithosphereModel,
    HistoricalTectonicEvent, PlanetPhysicalParameters, PlanetTopology, WorldgenError,
};
use std::collections::{BTreeMap, BTreeSet};

const DYNAMIC_PLATE_NAMESPACE: &str = "worldgen:geology:dynamic-modern-plates:v2";
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn mix64(mut value: u64) -> u64 {
    value ^= value >> 30;