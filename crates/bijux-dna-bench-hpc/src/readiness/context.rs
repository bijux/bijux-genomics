use super::ports::LocalReadinessBackend;
use std::path::Path;
/// Governed input paths supplied by their existing declarations, without a second registry.
pub struct ReadinessInputPaths<'a> {
    pub fastq_fixture: &'a str,
    pub bam_fixture: &'a str,
    pub damage_fixture: &'a str,
    pub edna_fixture: &'a str,
    pub amplicon_fixture: &'a str,
    pub taxonomy_fixture: &'a str,
    pub corpus_stage_compatibility: &'a str,
    pub fake_run_root: &'a str,
}
/// Request-scoped local readiness composition with no implicit working-directory lookup.
pub struct ReadinessContext<'a> {
    pub repo_root: &'a Path,
    pub backend: &'a dyn LocalReadinessBackend,
    pub paths: ReadinessInputPaths<'a>,
}
