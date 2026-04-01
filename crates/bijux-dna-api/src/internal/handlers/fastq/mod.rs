use bijux_dna_analyze::BenchmarkRecord;
use bijux_dna_planner_fastq::stage_api::RawFailure;

pub(crate) mod explain;
pub(crate) mod jobs;
use crate::internal::fastq::stages;
pub(crate) mod summary;

pub(crate) use crate::internal::fastq::stage_ids::{
    STAGE_CORRECT_ERRORS, STAGE_EXTRACT_UMIS, STAGE_FILTER_READS, STAGE_MERGE_PAIRS,
    STAGE_PREPROCESS_SUMMARY, STAGE_PROFILE_READS, STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY,
    STAGE_TRIM_POLYG_TAILS, STAGE_TRIM_READS, STAGE_TRIM_TERMINAL_DAMAGE, STAGE_VALIDATE_READS,
};
pub(crate) use bijux_dna_domain_fastq::stages::ids::{
    STAGE_DETECT_ADAPTERS, STAGE_FILTER_LOW_COMPLEXITY, STAGE_INDEX_REFERENCE,
};
pub use explain::{write_explain_md, write_explain_plan_json};
pub use stages::cluster_otus::bench_fastq_cluster_otus;
pub use stages::correct_errors::bench_fastq_correct;
pub use stages::deplete_host::bench_fastq_deplete_host;
pub use stages::deplete_reference_contaminants::bench_fastq_deplete_reference_contaminants;
pub use stages::deplete_rrna::bench_fastq_deplete_rrna;
pub use stages::detect_adapters::bench_fastq_detect_adapters;
pub use stages::extract_umis::bench_fastq_umi;
pub use stages::filter_low_complexity::bench_fastq_filter_low_complexity;
pub use stages::filter_reads::bench_fastq_filter;
pub use stages::index_reference::bench_fastq_index_reference;
pub use stages::infer_asvs::bench_fastq_infer_asvs;
pub use stages::merge_pairs::bench_fastq_merge;
pub use stages::normalize_abundance::bench_fastq_normalize_abundance;
pub use stages::normalize_primers::bench_fastq_normalize_primers;
pub use stages::preprocess::{bench_fastq_preprocess, fastq_preprocess_run};
pub use stages::profile_overrepresented_sequences::bench_fastq_profile_overrepresented;
pub use stages::profile_read_lengths::bench_fastq_profile_read_lengths;
pub use stages::profile_reads::bench_fastq_stats_neutral;
pub use stages::remove_chimeras::bench_fastq_remove_chimeras;
pub use stages::remove_duplicates::bench_fastq_remove_duplicates;
pub use stages::report_qc::bench_fastq_qc_post;
pub use stages::screen_taxonomy::bench_fastq_screen;
pub use stages::trim_polyg_tails::bench_fastq_trim_polyg_tails;
pub use stages::trim_reads::bench_fastq_trim;
pub use stages::trim_terminal_damage::bench_fastq_trim_terminal_damage;
pub use stages::validate_reads::bench_fastq_validate_reads;
pub use stages::validate_reads::bench_fastq_validate_reads as bench_fastq_validate_pre;
pub(crate) use summary::StageExecutionSummary;
/// Benchmarking result for a FASTQ stage.
///
/// Stability: v1 (stable).
/// Stability: v1
pub struct BenchOutcome<M: bijux_dna_analyze::StageMetricSchema> {
    pub records: Vec<BenchmarkRecord<M>>,
    pub failures: Vec<RawFailure>,
    pub bench_dir: std::path::PathBuf,
    pub explain: bool,
}

#[cfg(test)]
mod plan_tests {
    use bijux_dna_planner_fastq::stage_api::{polyx_unsupported_warning, STAGE_TRIM_READS};

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
    fn pipeline_id_catalog_are_stable_for_default_fastq() {
        let stages =
            bijux_dna_planner_fastq::fastq_pipeline_id_catalog("fastq-to-fastq__default__v1");
        assert!(stages.contains(&STAGE_TRIM_READS.as_str().to_string()));
    }
}
