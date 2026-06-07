#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[test]
fn bench_local_vcf_call_diploid_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-call-diploid-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "target/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz"
    );

    let repo_root = support::repo_root().expect("repo root");
    let output_vcf = repo_root.join("target/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz");
    let output_tbi =
        repo_root.join("target/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz.tbi");
    let metrics_path = repo_root.join("target/local-smoke/vcf.call_diploid/bcftools/metrics.json");
    let manifest_path =
        repo_root.join("target/local-smoke/vcf.call_diploid/bcftools/stage-result.json");
    let materialized_reference = repo_root.join(
        "target/local-smoke/vcf.call_diploid/bcftools/artifacts/reference/corpus_01_bam_reference.fasta",
    );
    let materialized_reference_fai = repo_root.join(
        "target/local-smoke/vcf.call_diploid/bcftools/artifacts/reference/corpus_01_bam_reference.fasta.fai",
    );
    let governed_reference_fai = repo_root.join(
        "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta.fai",
    );

    assert!(output_vcf.is_file(), "expected output VCF at {}", output_vcf.display());
    assert!(output_tbi.is_file(), "expected output index at {}", output_tbi.display());
    assert!(metrics_path.is_file(), "expected metrics at {}", metrics_path.display());
    assert!(manifest_path.is_file(), "expected stage result at {}", manifest_path.display());
    assert!(
        materialized_reference.is_file(),
        "expected materialized reference at {}",
        materialized_reference.display()
    );
    assert!(
        materialized_reference_fai.is_file(),
        "expected materialized reference index at {}",
        materialized_reference_fai.display()
    );
    assert!(
        !governed_reference_fai.exists(),
        "governed BAM reference should not be mutated at {}",
        governed_reference_fai.display()
    );

    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(
        manifest.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.call_diploid")
    );
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("bcftools")
    );
    assert_eq!(
        manifest
            .get("command")
            .and_then(|value| value.get("rendered"))
            .and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-call-diploid-smoke --tool-id bcftools")
    );
    assert_eq!(
        manifest
            .get("runtime")
            .and_then(|value| value.get("exit_code"))
            .and_then(serde_json::Value::as_i64),
        Some(0)
    );

    let outputs =
        manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs array");
    assert_eq!(outputs.len(), 3);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("diploid_vcf")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz")
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("metrics_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.call_diploid/bcftools/metrics.json")
    }));
}
