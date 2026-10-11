#![allow(clippy::expect_used)]

use crate::support;

const FASTQ_SCORE_EVIDENCE_COMMANDS: &[&[&str]] = &[
    &["bench", "local", "materialize-stage", "--stage-id", "fastq.filter_reads"],
    &["bench", "local", "materialize-stage", "--stage-id", "fastq.trim_reads"],
    &["bench", "local", "materialize-stage", "--stage-id", "fastq.validate_reads"],
    &["bench", "local", "run-edna-micro-pipeline"],
];

pub(super) fn run_cli(sandbox: &support::RepoSandbox, args: &[&str]) -> std::process::Output {
    let home = tempfile::tempdir().expect("tempdir");

    sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

pub(super) fn materialize_fastq_score_evidence(sandbox: &support::RepoSandbox) {
    for args in FASTQ_SCORE_EVIDENCE_COMMANDS {
        let output = run_cli(sandbox, args);
        assert!(
            output.status.success(),
            "FASTQ score evidence command failed: {}\ncommand: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
