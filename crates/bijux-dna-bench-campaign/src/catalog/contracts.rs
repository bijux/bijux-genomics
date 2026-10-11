#![allow(clippy::expect_used)]
use super::corpus_fastq_stage_catalog_entry as stage_command_spec;
#[test]
fn stage_mapping_preserves_filter_report_dir() {
    let spec = stage_command_spec("fastq.filter_reads").expect("stage spec");
    assert_eq!(spec.bench_subcommand, "filter");
    assert_eq!(spec.report_dir, "filter");
    assert!(spec.strict_resume_report);
}
