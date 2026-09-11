use super::{render_vcf_imputation_metrics_ready, DEFAULT_VCF_IMPUTATION_METRICS_READY_PATH};
use crate::commands::benchmark::repo_locking::{acquire_cwd_lock, RepoProcessLock};
fn repo_root() -> std::path::PathBuf {
    crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
}
#[test]
fn imputation_metrics_readiness_covers_every_supported_backend() {
    let repo_root = repo_root();
    let _cwd_lock = acquire_cwd_lock();
    let _repo_lock =
        RepoProcessLock::acquire(&repo_root, "benchmark-readiness-mutators").expect("repo lock");
    let output_root = tempfile::tempdir().expect("tempdir");
    let report = render_vcf_imputation_metrics_ready(
        &repo_root,
        output_root.path().join(DEFAULT_VCF_IMPUTATION_METRICS_READY_PATH),
    )
    .expect("build imputation metrics readiness");

    assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_imputation_metrics_ready.v1");
    assert_eq!(
        report.output_path,
        "artifacts/benchmark-smoke/readiness/vcf/imputation-metrics-ready.json"
    );
    assert_eq!(report.retained_row_count, 4);
    assert_eq!(report.active_row_count, 4);
    assert_eq!(report.complete_row_count, 4);
    assert_eq!(report.incomplete_row_count, 0);
    assert!(report.ok);
    let tool_ids = report
        .rows
        .iter()
        .map(|row| row.tool_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        tool_ids,
        std::collections::BTreeSet::from(["beagle", "glimpse", "impute5", "minimac4"])
    );

    let row = report.rows.iter().find(|row| row.tool_id == "beagle").expect("beagle row");
    assert_eq!(row.stage_id, "vcf.imputation_metrics");
    assert_eq!(row.tool_id, "beagle");
    assert!(row.command_ready);
    assert!(row.output_ready);
    assert!(row.parser_ready);
    assert!(row.expected_result_ready);
    assert!(row.report_ready);
    assert!(row.expected_metrics.iter().any(|metric| metric == "concordance"));
    assert!(row.expected_metrics.iter().any(|metric| metric == "dosage_r2"));
    assert!(row.report_metric_columns.iter().any(|metric| metric == "concordance"));
    assert!(row.report_metric_columns.iter().any(|metric| metric == "dosage_r2"));
    assert!(row.smoke_ready);
    assert_eq!(row.coverage_status, "complete");
    assert_eq!(row.smoke_concordance, Some(1.0));
    assert_eq!(row.smoke_dosage_r2, Some(0.775));

    let impute5 = report.rows.iter().find(|row| row.tool_id == "impute5").expect("IMPUTE5 row");
    assert_eq!(impute5.tool_status, "supported");
    assert!(impute5.command_ready);
    assert!(impute5.output_ready);
    assert!(impute5.parser_ready);
    assert!(impute5.expected_result_ready);
    assert!(impute5.report_ready);
    assert!(impute5.smoke_ready);
    assert_eq!(impute5.coverage_status, "complete");
    assert!(impute5.missing_surfaces.is_empty());
    assert!(impute5.smoke_command.ends_with("--tool-id impute5"));
    assert!(report.violations.is_empty());
}
