#[derive(Clone, Debug, PartialEq)]
pub struct FieldStatistics {
    pub sample_count: u64,
    pub minimum: u16,
    pub maximum: u16,
    pub mean: f64,
    pub hash: u64,
}

impl FieldStatistics {
    pub fn hash_hex(&self) -> String {
        format!("{:016x}", self.hash)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageIdentity {
    pub id: &'static str,
    pub version: u32,
    pub derived_seed: u64,
}
