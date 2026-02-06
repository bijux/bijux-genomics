use bijux_analyze::BenchmarkRecord;
use bijux_planner_fastq::stage_api::RawFailure;

mod explain;
mod jobs;
mod stages;
mod summary;

pub use bijux_planner_fastq::stage_api::{
    STAGE_CORRECT, STAGE_FILTER, STAGE_MERGE, STAGE_PREPROCESS, STAGE_QC_POST, STAGE_SCREEN,
    STAGE_STATS_NEUTRAL, STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};

pub use crate::fastq_stats_neutral::bench_fastq_stats_neutral;
pub use explain::{write_explain_md, write_explain_plan_json};
pub use stages::correct::bench_fastq_correct;
pub use stages::filter::bench_fastq_filter;
pub use stages::merge::bench_fastq_merge;
pub use stages::preprocess::{bench_fastq_preprocess, fastq_preprocess_run};
pub use stages::qc_post::bench_fastq_qc_post;
pub use stages::screen::bench_fastq_screen;
pub use stages::trim::bench_fastq_trim;
pub use stages::umi::bench_fastq_umi;
pub use stages::validate_pre::bench_fastq_validate_pre;
pub(crate) use summary::StageExecutionSummary;
/// Benchmarking result for a FASTQ stage.
///
/// Stability: v1 (stable).
/// Stability: v1
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
