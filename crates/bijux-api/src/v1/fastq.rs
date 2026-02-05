//! FASTQ domain helpers for v1.

pub use bijux_planner_fastq::stage_api as fastq_banks;
pub use bijux_planner_fastq::stage_api::{
    adapter_bank_path, adapter_presets_path, benchmark_runs, contaminant_motifs_path,
    contaminant_presets_path, contaminant_references_dir, load_adapter_bank, load_adapter_presets,
    load_contaminant_motifs, load_contaminant_presets, load_polyx_bank, load_polyx_presets,
    polyx_bank_path, polyx_presets_path, qc_class_for_stage, write_benchmark_exports,
    AdapterPresetsV1, BenchCorpusId, EffectiveAdapterSet, QcClass, ReadScope, STAGES,
};

pub use bijux_planner_fastq::stage_api::args as fastq_args;

pub use crate::fastq_router::{
    bench_fastq_correct, bench_fastq_filter, bench_fastq_merge, bench_fastq_preprocess,
    bench_fastq_qc_post, bench_fastq_screen, bench_fastq_stats_neutral, bench_fastq_trim,
    bench_fastq_umi, bench_fastq_validate_pre, fastq_preprocess_plan, fastq_preprocess_run,
};
