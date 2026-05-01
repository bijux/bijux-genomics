use bijux_dna_core::ids::StageId;

pub const PLANNER_VERSION: &str = "bijux-dna-planner-fastq.v1";
pub const TOOL_SEQKIT: &str = "seqkit";
pub const STAGE_REPORT_AGGREGATE: StageId = StageId::from_static("report.aggregate");
pub const STAGE_COMPARE_STAGE_TOOLS: StageId =
    StageId::from_static("benchmark.compare_stage_tools");
pub const STAGE_SELECT_STAGE_TOOL: StageId = StageId::from_static("benchmark.select_stage_tool");
pub const STAGE_PREPROCESS_SUMMARY: StageId = StageId::from_static("fastq.preprocess");

pub use bijux_dna_domain_fastq::stages::ids::{
    STAGE_DEPLETE_HOST, STAGE_DEPLETE_REFERENCE_CONTAMINANTS,
};
pub use bijux_dna_domain_fastq::BenchResultsRepository;
pub use bijux_dna_domain_fastq::{
    STAGE_CLUSTER_OTUS, STAGE_CORRECT_ERRORS, STAGE_DEPLETE_RRNA, STAGE_DETECT_ADAPTERS,
    STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY, STAGE_FILTER_READS, STAGE_INFER_ASVS,
    STAGE_MERGE_PAIRS, STAGE_NORMALIZE_ABUNDANCE, STAGE_NORMALIZE_PRIMERS, STAGE_PROFILE_READS,
    STAGE_REMOVE_CHIMERAS, STAGE_REMOVE_DUPLICATES, STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY,
    STAGE_TRIM_READS, STAGE_TRIM_TERMINAL_DAMAGE, STAGE_VALIDATE_READS,
};

pub use crate::compose::{StageArtifactInputBinding, StageArtifactInputPolicy};
pub(crate) use crate::pipeline_defaults::pipeline_spec_from_stage_catalog;
pub use crate::pipeline_defaults::{default_pipeline_spec, DefaultPipelineOptions};
pub use crate::planner::*;
pub use crate::preprocess::chunking::{plan_chunked_preprocess_contract, ChunkedFastqInput};
pub use crate::preprocess::{apply_preprocess_policy, PreprocessPolicyDecision};
pub use crate::preprocess::{
    plan_preprocess, preprocess_decisions, resolve_preprocess_pipeline, CorrectDecisionTrace,
    MergeDecisionTrace, PreprocessDecisions,
};
pub use crate::report_stage::report_stage_step;
pub use crate::selection::args;
