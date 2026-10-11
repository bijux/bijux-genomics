use super::{
    projection::build_vcf_adapters_ready_report, state::ReadinessCounts, VcfAdaptersReadyGoalCheck,
    DEFAULT_VCF_ADAPTERS_READY_PATH, VCF_ADAPTERS_READY_SCHEMA_VERSION,
};
fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from("declared-repo")
}
#[test]
fn vcf_adapters_ready_report_marks_failed_goal_ids_and_pair_counts() {
    let root = repo_root();
    let checks = vec![
        VcfAdaptersReadyGoalCheck {
            goal_id: 231,
            surface: "vcf tool-serving map".to_string(),
            output_path: Some(
                "artifacts/benchmark-smoke/readiness/vcf/vcf-tool-serving-map.tsv".to_string(),
            ),
            ok: true,
            detail: "ok".to_string(),
        },
        VcfAdaptersReadyGoalCheck {
            goal_id: 245,
            surface: "vcf adapter benchmark-ready pair completeness".to_string(),
            output_path: Some(DEFAULT_VCF_ADAPTERS_READY_PATH.to_string()),
            ok: false,
            detail: "pair mismatch".to_string(),
        },
    ];

    let report = build_vcf_adapters_ready_report(
        &root,
        &root.join(DEFAULT_VCF_ADAPTERS_READY_PATH),
        checks,
        ReadinessCounts {
            benchmark_ready: 15,
            adapter_complete: 15,
            output_complete: 15,
            rendered_commands: 15,
        },
    );

    assert_eq!(report.schema_version, VCF_ADAPTERS_READY_SCHEMA_VERSION);
    assert_eq!(report.output_path, DEFAULT_VCF_ADAPTERS_READY_PATH);
    assert_eq!(report.checked_goal_count, 2);
    assert_eq!(report.passed_goal_count, 1);
    assert_eq!(report.failed_goal_count, 1);
    assert_eq!(report.failing_goal_ids, vec![245]);
    assert_eq!(report.benchmark_ready_pair_count, 15);
    assert_eq!(report.adapter_complete_pair_count, 15);
    assert_eq!(report.output_complete_pair_count, 15);
    assert_eq!(report.rendered_command_pair_count, 15);
    assert!(!report.ok);
}
