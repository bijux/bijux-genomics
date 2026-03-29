use sha2::Digest;

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct RegionChunkPlan {
    pub chunk_id: String,
    pub contig: String,
    pub start: u64,
    pub end: u64,
}

pub(crate) fn short_species_context_digest(
    species_id: &str,
    build_id: &str,
    contig_set_digest: &str,
) -> String {
    let seed = format!("{species_id}|{build_id}|{contig_set_digest}");
    let mut hasher = sha2::Sha256::new();
    hasher.update(seed.as_bytes());
    let full = format!("{:x}", hasher.finalize());
    full.chars().take(12).collect()
}
