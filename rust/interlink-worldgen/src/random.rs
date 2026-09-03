const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

fn fnv1a_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Stable namespace-isolated seed derivation. This deliberately does not use
/// DefaultHasher so generator identity is not tied to standard-library details.
pub fn derive_stage_seed(seed: &str, namespace: &str) -> u64 {
    let hash = fnv1a_update(FNV_OFFSET_BASIS, b"project-interlink-worldgen\0");
    let hash = fnv1a_update(hash, seed.as_bytes());
    let hash = fnv1a_update(hash, b"\0");
    fnv1a_update(hash, namespace.as_bytes())
}

/// SplitMix64 finalizer used as a deterministic coordinate mixer.
pub fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

pub fn coordinate_value(stage_seed: u64, x: u32, y: u32) -> u64 {
    let packed = (u64::from(y) << 32) | u64::from(x);
    mix64(stage_seed ^ packed)
}

pub fn hash_u16_field(width: u32, height: u32, values: &[u16]) -> u64 {
    let mut hash = fnv1a_update(FNV_OFFSET_BASIS, b"dense-u16-field:v1\0");
    hash = fnv1a_update(hash, &width.to_le_bytes());
    hash = fnv1a_update(hash, &height.to_le_bytes());
    for value in values {
        hash = fnv1a_update(hash, &value.to_le_bytes());
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_namespaces_are_stable_and_isolated() {
        let a = derive_stage_seed("seed", "synthetic:v1");
        assert_eq!(a, derive_stage_seed("seed", "synthetic:v1"));
        assert_ne!(a, derive_stage_seed("seed", "tectonics:v1"));
        assert_ne!(a, derive_stage_seed("other", "synthetic:v1"));
    }
}
