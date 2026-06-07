#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_rendered_commands_write_bash_parseable_script() {
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
        .args(["bench", "readiness", "render-all-domain-commands"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let script_path = repo_root.join("target/bench-readiness/rendered-commands-all-domains.sh");
    assert!(script_path.is_file(), "all-domain rendered command script must exist");

    let script = std::fs::read_to_string(&script_path).expect("read all-domain rendered script");
    assert!(script.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n"));
    assert!(
        script.contains("repo_root=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/../..\" && pwd)\"")
    );
    assert!(script.contains("# fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2 / fastq / fastq.screen_taxonomy / kraken2"));
    assert!(script.contains(
        "# bam:corpus-01-kinship-mini:bam.kinship:sample-set:king / bam / bam.kinship / king"
    ));
    assert!(script.contains(
        "# vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools / vcf / vcf.call / bcftools"
    ));
    assert!(script.contains("cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.kinship"));
    assert!(script.contains("kraken2 --db"));
    assert!(script.contains("bcftools mpileup"));
    assert!(script.contains(" | bcftools call "));
    assert!(!script.to_ascii_lowercase().contains("todo"));
    assert!(!script.to_ascii_lowercase().contains("placeholder"));
    assert!(!script.to_ascii_lowercase().contains("echo execute"));

    let syntax = Command::new("bash").arg("-n").arg(&script_path).output().expect("run bash -n");
    assert!(syntax.status.success(), "bash -n failed: {}", String::from_utf8_lossy(&syntax.stderr));
}
