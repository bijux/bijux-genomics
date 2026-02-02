use bijux_analyze::BenchmarkRecord;
use bijux_stages_fastq::RawFailure;

mod banks;
mod correct;
mod explain;
mod filter;
mod jobs;
mod merge;
mod preprocess;
mod qc_post;
mod screen;
mod smart_pipeline;
mod stats_neutral;
mod summary;
mod trim;
mod umi;
mod validate_pre;

pub use correct::bench_fastq_correct;
pub use explain::{write_explain_md, write_explain_plan_json};
pub use filter::bench_fastq_filter;
pub use merge::bench_fastq_merge;
pub use preprocess::{bench_fastq_preprocess, fastq_preprocess_plan, fastq_preprocess_run};
pub use qc_post::bench_fastq_qc_post;
pub use screen::bench_fastq_screen;
pub use stats_neutral::bench_fastq_stats_neutral;
pub use trim::bench_fastq_trim;
pub use umi::bench_fastq_umi;
pub use validate_pre::bench_fastq_validate_pre;
pub struct BenchOutcome<M: bijux_analyze::StageMetricSchema> {
    pub records: Vec<BenchmarkRecord<M>>,
    pub failures: Vec<RawFailure>,
    pub bench_dir: std::path::PathBuf,
    pub explain: bool,
}

#[cfg(test)]
mod tests {
    use super::banks::polyx_unsupported_warning;
    use super::fastq_preprocess_plan;
    use std::path::PathBuf;

    fn base_args() -> bijux_stages_fastq::args::BenchFastqPreprocessArgs {
        bijux_stages_fastq::args::BenchFastqPreprocessArgs {
            sample_id: "sample".to_string(),
            r1: PathBuf::from("reads_R1.fastq.gz"),
            r2: None,
            out: PathBuf::from("out"),
            strict: false,
            auto: true,
            objective: bijux_core::selection::Objective::Balanced,
            bench_corpus: None,
            allow_partial: false,
            replicates: 1,
            jobs: 1,
            ci_bootstrap: None,
            adapter_bank_preset: None,
            adapter_bank: None,
            adapter_bank_file: None,
            enable_adapters: Vec::new(),
            disable_adapters: Vec::new(),
            polyx_preset: None,
            contaminant_preset: None,
            enable_contaminant_removal: false,
            no_qc_post: false,
            force_merge: false,
            enable_correct: false,
        }
    }

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
    fn preprocess_plan_single_end_has_no_merge() {
        let args = base_args();
        let plan = fastq_preprocess_plan(&args);
        assert!(plan.stages.contains(&"fastq.trim".to_string()));
        assert!(!plan.stages.contains(&"fastq.merge".to_string()));
    }

    #[test]
    fn preprocess_plan_paired_includes_merge_when_forced() {
        let mut args = base_args();
        args.r2 = Some(PathBuf::from("reads_R2.fastq.gz"));
        args.force_merge = true;
        let plan = fastq_preprocess_plan(&args);
        assert!(plan.stages.contains(&"fastq.merge".to_string()));
    }
}
