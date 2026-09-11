use bijux_dna_stage_contract::StagePlanV1;
use std::path::{Path, PathBuf};
/// Published synthetic report with its observed tool identity.
#[derive(Debug, Clone)]
pub struct MaterializedFastqProof {
    pub proof_path: PathBuf,
    pub observed_tool_id: String,
}
/// Explicit correction plan and input files selected by application composition.
pub struct CorrectErrorsProbeInput<'a> {
    pub plan: &'a StagePlanV1,
    pub r1: &'a Path,
    pub r2: Option<&'a Path>,
    pub quality_encoding: bijux_dna_domain_fastq::params::correct::QualityEncoding,
    pub conservative_mode: bool,
}
#[derive(Debug, Clone)]
pub(super) struct FastqReadStats {
    pub reads: u64,
    pub bases: u64,
}
