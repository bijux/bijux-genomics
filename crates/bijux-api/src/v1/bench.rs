//! Benchmarking helpers for v1.
//!
//! Stability: v1 (stable).

pub use bijux_analyze::compare::compare_runs_with_baseline;
pub use bijux_analyze::{build_rankings, compare_runs, print_bench_schema, RankInput};

pub use crate::args::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};
pub use bijux_core::selection::{objective_spec, Objective, ObjectiveSpec, ObjectiveWeights};
pub use bijux_domain_bam::{bam_stage_completeness, BamStage};
pub use bijux_domain_fastq::banks as fastq_banks;
pub use bijux_domain_fastq::{
    adapter_bank_path, adapter_presets_path, benchmark_runs, contaminant_motifs_path,
    contaminant_presets_path, contaminant_references_dir, load_adapter_bank, load_adapter_presets,
    load_contaminant_motifs, load_contaminant_presets, load_polyx_bank, load_polyx_presets,
    polyx_bank_path, polyx_presets_path, qc_class_for_stage, write_benchmark_exports,
    AdapterPresetsV1, BenchCorpusId, EffectiveAdapterSet, QcClass, ReadScope, STAGES,
};
pub use bijux_planner_fastq::stage_api::args as fastq_args;

pub use crate::bam_router::{bench_bam_pipeline, bench_bam_stage};
pub use crate::fastq_router::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_preprocess,
    bench_fastq_qc_post, bench_fastq_screen, bench_fastq_stats_neutral, bench_fastq_trim,
    bench_fastq_umi, bench_fastq_validate_pre,
};
