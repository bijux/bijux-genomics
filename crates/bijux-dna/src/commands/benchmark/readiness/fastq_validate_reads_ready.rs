use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
#[cfg(test)]
use bijux_dna_bench_fastq::proofs::validate_reads::FASTQ_VALIDATE_READS_READY_SCHEMA_VERSION;
pub(crate) use bijux_dna_bench_fastq::proofs::validate_reads::{
    FastqValidateReadsReadyReport, DEFAULT_FASTQ_VALIDATE_READS_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_fastq_validate_reads_ready(
    args: &parse::BenchReadinessRenderFastqValidateReadsReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_validate_reads_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_VALIDATE_READS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_fastq_validate_reads_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqValidateReadsReadyReport> {
    bijux_dna_bench_fastq::proofs::validate_reads::render_fastq_validate_reads_ready(
        &super::fastq_proof_evidence::validation(repo_root),
        &output_path,
    )
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_fastq_validate_reads_ready, DEFAULT_FASTQ_VALIDATE_READS_READY_PATH,
        FASTQ_VALIDATE_READS_READY_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_fastq_validate_reads_ready_reports_complete_active_validator_rows() {
        let root = repo_root();
        let report = render_fastq_validate_reads_ready(
            &root,
            PathBuf::from(DEFAULT_FASTQ_VALIDATE_READS_READY_PATH),
        )
        .expect("render fastq validate-reads readiness");

        assert_eq!(report.schema_version, FASTQ_VALIDATE_READS_READY_SCHEMA_VERSION);
        assert_eq!(
            report.output_path,
            "artifacts/benchmark-smoke/readiness/fastq/validate-reads-ready.json"
        );
        assert_eq!(report.active_row_count, 5);
        assert_eq!(report.complete_row_count, 5);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.sample_case_count, 2);
        assert_eq!(report.smoke_sample_ids, vec!["toy-pe".to_string(), "toy-se".to_string()]);
        assert_eq!(report.smoke_layouts, vec!["paired_end".to_string(), "single_end".to_string()]);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);

        let fastqvalidator = report
            .rows
            .iter()
            .find(|row| row.tool_id == "fastqvalidator")
            .expect("fastqvalidator row");
        assert_eq!(
            fastqvalidator.result_id,
            "fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastqvalidator"
        );
        assert_eq!(fastqvalidator.report_section_id, "input_readiness");
        assert_eq!(fastqvalidator.summary_table_id, "validation_intake");
        assert_eq!(fastqvalidator.command_readiness_kind, "smoke");
        assert_eq!(
            fastqvalidator.normalized_metrics_output_id.as_deref(),
            Some("validation_report")
        );
        assert!(fastqvalidator
            .expected_outputs
            .iter()
            .any(|artifact_id| artifact_id == "validation_report"));
        assert!(fastqvalidator
            .expected_outputs
            .iter()
            .any(|artifact_id| artifact_id == "validated_reads_manifest"));
        assert!(fastqvalidator
            .shared_metric_fields
            .iter()
            .any(|field| field == "format_validation_pass_rate"));
        assert!(fastqvalidator
            .validation_report_fields
            .iter()
            .any(|field| field == "failure_class"));
        assert!(fastqvalidator
            .validated_reads_manifest_fields
            .iter()
            .any(|field| field == "paired_mode"));
        assert!(fastqvalidator.command_ready);
        assert!(fastqvalidator.output_ready);
        assert!(fastqvalidator.parser_ready);
        assert!(fastqvalidator.comparable_metrics_ready);
        assert!(fastqvalidator.expected_result_ready);
        assert!(fastqvalidator.report_ready);
        assert!(fastqvalidator.smoke_parseable);
        assert!(fastqvalidator.sample_id_normalized);
        assert!(fastqvalidator.layout_normalized);
        assert!(fastqvalidator.read_count_normalized);
        assert!(fastqvalidator.validation_status_normalized);
        assert!(fastqvalidator.failure_reason_normalized);
        assert_eq!(fastqvalidator.coverage_status, "complete");
        assert!(fastqvalidator.missing_surfaces.is_empty());
    }
}
