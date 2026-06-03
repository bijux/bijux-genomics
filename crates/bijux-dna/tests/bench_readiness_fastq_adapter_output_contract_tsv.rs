#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_adapter_output_contract_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-fastq-adapter-output-contract"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/fastq-adapter-output-contract.tsv");
    assert!(tsv_path.is_file(), "FASTQ adapter output contract TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read FASTQ adapter output contract");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tstage_id\tadapter_status\toutput_contract_status\tstage_output_ids\tstage_expected_artifact_ids\tdeclared_output_ids\texecution_expected_output_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tstdout_path_template\tstderr_path_template\tstage_result_manifest_path_template\tmissing_declarations\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 75, "TSV must retain the governed FASTQ 75-row slice");
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "seqkit_stats\tfastq.profile_reads\trunnable\tcomplete\tqc_json,qc_tsv,qc_plots_dir\tqc_json,qc_tsv,qc_plots_dir\t"
            )
        }),
        "TSV must retain the governed profile-reads contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with("bowtie2\tfastq.deplete_host\trunnable\tcomplete\t")
                && row.contains("\thost_depletion_report_json\t")
        }),
        "TSV must retain the governed bowtie2 host-depletion contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "diamond\tfastq.screen_taxonomy\tdeclared_only\tmissing_adapter\tclassification_report_json,screen_report_tsv\t\t\t\t\t\t\t\t\tadapter\t"
            )
        }),
        "TSV must keep the planned diamond taxonomy row explicit as missing an adapter"
    );
}
