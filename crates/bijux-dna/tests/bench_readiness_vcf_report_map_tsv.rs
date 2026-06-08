#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_report_map_writes_governed_tsv_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-vcf-report-map"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf-report-map.tsv");

    let payload =
        std::fs::read_to_string(repo_root.join(rendered_path.trim())).expect("read VCF report map");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some("stage_id\ttool_id\tsection_id\tsummary_table\tmetric_columns\tfailure_columns")
    );
    assert!(lines.any(|line| {
        line.starts_with("vcf.stats\tbcftools\tquality_control\tquality_control_metrics\t")
            && line.contains("variant_count,snp_count,indel_count,transition_count,transversion_count,ti_tv,sample_count")
            && line.contains("result_status,reason,parser_id,failure_reason,observed_error")
    }));
    assert!(payload.lines().any(|line| {
        line
            == "vcf.prepare_reference_panel\tbcftools\treference_panel_preparation\treference_panel_readiness\tinput_variants,output_variants,sample_count,sample_ids,sample_consistent,duplicate_sites_removed,normalization_status,parseable\tresult_status,reason,parser_id,failure_reason,observed_error,audit_manifest_path"
    }));
}
