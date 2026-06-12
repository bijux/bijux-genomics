#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_comparable_metrics_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-fastq-comparable-metrics"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/fastq-comparable-metrics.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read FASTQ comparable metrics TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\tcomparison_contract_status\ttool_count\ttool_ids\tdefault_tool_id\tcorpus_status\tshared_metric_field_count\tshared_metric_fields\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 3);
    assert!(
        rows.iter().any(|row| {
            row == &"fastq.index_reference\tdeclared\t2\tbowtie2_build,star\tbowtie2_build\tasset:reference-index-assets\t1\tindex_build_exit_code\tstage `fastq.index_reference` publishes governed shared comparable metrics `index_build_exit_code` for same-stage tool comparison while corpus routing remains `asset:reference-index-assets`"
        }),
        "TSV must retain the governed FASTQ index-reference comparable row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastq.validate_reads\tdeclared\t5\tfastq_scan,fastqc,fastqvalidator,fqtools,seqtk\tfastqvalidator\tfixture:corpus-01-mini\t1\tformat_validation_pass_rate\tstage `fastq.validate_reads` publishes governed shared comparable metrics `format_validation_pass_rate` for same-stage tool comparison while corpus routing remains `fixture:corpus-01-mini`"
        }),
        "TSV must retain the governed FASTQ validation comparable row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"fastq.profile_overrepresented_sequences\tdeclared\t3\tfastq_scan,fastqc,seqkit\tfastqc\tfixture:corpus-01-mini\t3\tsequence_count,flagged_sequences,top_fraction\tstage `fastq.profile_overrepresented_sequences` publishes governed shared comparable metrics `sequence_count, flagged_sequences, top_fraction` for same-stage tool comparison while corpus routing remains `fixture:corpus-01-mini`"
        }),
        "TSV must retain the governed FASTQ overrepresented comparable row"
    );
}
