use super::{
    projection::build_vcf_parsers_report_ready_report, state::ReadinessCounts,
    VcfParsersReportReadyGoalCheck, DEFAULT_VCF_PARSERS_REPORT_READY_PATH,
    VCF_PARSERS_REPORT_READY_SCHEMA_VERSION,
};
fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from("declared-repo")
}
#[test]
fn vcf_parsers_report_ready_report_marks_failed_goal_ids_and_counts() {
    let root = repo_root();
    let checks = vec![
        VcfParsersReportReadyGoalCheck {
            goal_id: 246,
            surface: "vcf normalized metrics schemas".to_string(),
            output_path: Some(
                "artifacts/benchmark-smoke/readiness/vcf/vcf-schema-validation.json".to_string(),
            ),
            ok: true,
            detail: "ok".to_string(),
        },
        VcfParsersReportReadyGoalCheck {
            goal_id: 260,
            surface: "vcf parser/report ready pair completeness".to_string(),
            output_path: Some(DEFAULT_VCF_PARSERS_REPORT_READY_PATH.to_string()),
            ok: false,
            detail: "pair mismatch".to_string(),
        },
    ];

    let report = build_vcf_parsers_report_ready_report(
        &root,
        &root.join(DEFAULT_VCF_PARSERS_REPORT_READY_PATH),
        checks,
        ReadinessCounts {
            parser_fixtures: 39,
            active_fixtures: 21,
            expected_results: 21,
            report_rows: 21,
            metrics: 35,
        },
    );

    assert_eq!(report.schema_version, VCF_PARSERS_REPORT_READY_SCHEMA_VERSION);
    assert_eq!(report.output_path, DEFAULT_VCF_PARSERS_REPORT_READY_PATH);
    assert_eq!(report.checked_goal_count, 2);
    assert_eq!(report.passed_goal_count, 1);
    assert_eq!(report.failed_goal_count, 1);
    assert_eq!(report.failing_goal_ids, vec![260]);
    assert_eq!(report.parser_fixture_row_count, 39);
    assert_eq!(report.active_parser_fixture_row_count, 21);
    assert_eq!(report.expected_result_row_count, 21);
    assert_eq!(report.report_map_row_count, 21);
    assert_eq!(report.comparable_metric_row_count, 35);
    assert!(!report.ok);
}
