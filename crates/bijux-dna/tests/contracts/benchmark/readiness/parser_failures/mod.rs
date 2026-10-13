#![allow(clippy::expect_used)]

use std::process::Output;

use crate::support;

#[cfg(feature = "bam_downstream")]
mod fastq_bam;
mod vcf;

fn run_cli(label: &str, args: &[&str]) -> (support::RepoSandbox, Output) {
    let sandbox = support::RepoSandbox::new(label).expect("parser failure repository sandbox");
    let home = sandbox.path().join("artifacts/parser-failure-process/home");
    let scratch = sandbox.path().join("artifacts/parser-failure-process/scratch");
    std::fs::create_dir_all(&home).expect("create child home");
    std::fs::create_dir_all(&scratch).expect("create child scratch");
    let output = sandbox
        .bijux_dna_command()
        .env("HOME", &home)
        .env("TMPDIR", &scratch)
        .env("TMP", &scratch)
        .env("TEMP", &scratch)
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli");
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (sandbox, output)
}

fn run_cli_json(label: &str, args: &[&str]) -> serde_json::Value {
    let (_sandbox, output) = run_cli(label, args);
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}
