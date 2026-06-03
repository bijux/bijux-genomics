#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_adapter_output_contract_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-bam-adapter-output-contract"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/bam-adapter-output-contract.tsv");
    assert!(tsv_path.is_file(), "BAM adapter output contract TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read BAM adapter output contract");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tstage_id\tadapter_status\toutput_contract_status\tstage_output_ids\tstage_expected_artifact_ids\tdeclared_output_ids\texecution_expected_output_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tstdout_path_template\tstderr_path_template\tstage_result_manifest_path_template\tmissing_declarations\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 51, "TSV must retain the governed BAM 51-row slice");
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "samtools\tbam.validate\tplannable\tcomplete\tvalidation_report,flagstat,stage_metrics\tvalidation_report,flagstat,stage_metrics\t"
            )
        }),
        "TSV must retain the governed samtools validate contract row"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with(
                "bcftools\tbam.genotyping\tdeclared_only\tmissing_adapter\tgenotyping_report,summary,stage_metrics\t\t\t\t\t\t\t\t\tadapter\t"
            )
        }),
        "TSV must keep the planned bcftools genotyping row explicit as missing an adapter"
    );
}
