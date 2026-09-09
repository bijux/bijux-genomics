//! Typed acquisition boundary for benchmark profile evidence.

use anyhow::Result;
use bijux_dna_bench_model::{BenchmarkLocalCorpusStageCompatibilityReport, VcfStageMatrixRow};
use std::collections::BTreeMap;

/// Supplies governed evidence from a caller-selected repository or captured source.
/// Providers retain acquisition/validation responsibility; profile rules compare
/// the required bindings and preserve provider failures. No process API is exposed.
pub trait PipelineProfileEvidence {
    /// Return the corpus/stage compatibility report admitted by its owner.
    fn corpus_stage_compatibility(&self) -> Result<BenchmarkLocalCorpusStageCompatibilityReport>;
    /// Return the governed VCF matrix indexed by canonical stage identity.
    fn vcf_stage_matrix(&self) -> Result<BTreeMap<String, VcfStageMatrixRow>>;
}
