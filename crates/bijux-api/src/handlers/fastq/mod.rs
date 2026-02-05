use bijux_analyze::BenchmarkRecord;
use bijux_planner_fastq::stage_api::RawFailure;

mod correct;
mod explain;
mod filter;
mod jobs;
mod merge;
mod preprocess;
mod qc_post;
mod screen;
mod stats_neutral;
mod summary;
mod trim;
mod umi;
mod validate_pre;

pub use bijux_planner_fastq::stage_api::{
    STAGE_CORRECT, STAGE_FILTER, STAGE_MERGE, STAGE_PREPROCESS, STAGE_QC_POST, STAGE_SCREEN,
    STAGE_STATS_NEUTRAL, STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};

pub use correct::bench_fastq_correct;
pub use explain::{write_explain_md, write_explain_plan_json};
pub use filter::bench_fastq_filter;
pub use merge::bench_fastq_merge;
pub use preprocess::{bench_fastq_preprocess, fastq_preprocess_run};
pub use qc_post::bench_fastq_qc_post;
pub use screen::bench_fastq_screen;
pub use stats_neutral::bench_fastq_stats_neutral;
pub use summary::StageExecutionSummary;
pub use trim::bench_fastq_trim;
pub use umi::bench_fastq_umi;
pub use validate_pre::bench_fastq_validate_pre;
/// Benchmarking result for a FASTQ stage.
///
/// Stability: v1 (stable).
pub struct BenchOutcome<M: bijux_analyze::StageMetricSchema> {
    pub records: Vec<BenchmarkRecord<M>>,
    pub failures: Vec<RawFailure>,
    pub bench_dir: std::path::PathBuf,
    pub explain: bool,
}

#[cfg(test)]
mod plan_tests {
    use bijux_planner_fastq::stage_api::{polyx_unsupported_warning, STAGE_TRIM};

    #[test]
    fn polyx_warning_emits_for_unsupported_tools() {
        let polyx_bank = serde_json::json!({
            "bank_id": "polyx.default",
            "preset": "illumina_twocolor",
        });
        let warning = polyx_unsupported_warning("cutadapt", Some(&polyx_bank), true);
        assert!(warning.is_some());
        let warning = polyx_unsupported_warning("fastp", Some(&polyx_bank), true);
        assert!(warning.is_none());
        let warning = polyx_unsupported_warning("cutadapt", None, true);
        assert!(warning.is_none());
        let warning = polyx_unsupported_warning("cutadapt", Some(&polyx_bank), false);
        assert!(warning.is_none());
    }

    #[test]
    fn pipeline_stage_ids_are_stable_for_default_fastq() {
        let stages = bijux_planner_fastq::fastq_pipeline_stage_ids("fastq-to-fastq__default__v1");
        assert!(stages.contains(&STAGE_TRIM.as_str().to_string()));
    }
}
