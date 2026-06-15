#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_removed_from_scope_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-removed-from-scope"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/removed-from-scope.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read removed-from-scope tsv");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tstatus\tadapter_status\tscope_exit_kind\tstage_removed_from_active_scope\ttool_removed_from_active_scope\tabsent_from_active_matrix\tabsent_from_rendered_commands\tabsent_from_expected_results\tabsent_from_full_benchmark_report"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 21);
    assert!(rows.iter().any(|row| {
        row == &"fastq\tfastq.index_reference\tbowtie2_build\tnot_assigned\treference_fasta+reference_index_output\tfastq.adapter.index_reference\tfastq.parser.index_reference\tfastq_index_reference_v1\tnot_benchmark_ready\trunnable\tbenchmark_not_ready\ttrue\ttrue\ttrue\ttrue\ttrue\ttrue"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf\tvcf.imputation_metrics\tbeagle\tvcf_production_regression\tvcf_cohort_with_panel\tvcf.adapter.panel_workflow\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.imputation-metrics.v1\tplanned\tdeclared_only\tlifecycle_not_active\ttrue\ttrue\ttrue\ttrue\ttrue\ttrue"
    }));
    assert!(
        rows.iter().all(|row| {
            let columns = row.split('\t').collect::<Vec<_>>();
            columns.len() == 17
                && columns[13] == "true"
                && columns[14] == "true"
                && columns[15] == "true"
                && columns[16] == "true"
        }),
        "removed bindings must stay absent from every governed active downstream surface"
    );
}
