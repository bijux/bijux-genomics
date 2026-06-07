#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli_json(args: &[&str]) -> serde_json::Value {
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

    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn plan_validate_adna_gl_pipeline_reports_governed_profile() {
    let payload =
        run_cli_json(&["plan", "validate", "--id", "adna-gl-fastq-bam-vcf", "--strict", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_pipeline_dag_validation.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/configs/pipelines/local/adna-gl-fastq-bam-vcf.toml")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/pipeline-dag/adna-gl-fastq-bam-vcf.json")
    );
    assert_eq!(
        payload.get("pipeline_id").and_then(serde_json::Value::as_str),
        Some("adna-gl-fastq-bam-vcf")
    );
    assert_eq!(payload.get("valid").and_then(serde_json::Value::as_bool), Some(true));

    let profiles = payload
        .get("validation_profiles")
        .and_then(serde_json::Value::as_array)
        .expect("validation profiles");
    let ancient_dna = profiles
        .iter()
        .find(|profile| {
            profile.get("profile_id").and_then(serde_json::Value::as_str) == Some("ancient_dna_gl")
        })
        .expect("ancient-dna gl profile");
    assert_eq!(ancient_dna.get("check_count").and_then(serde_json::Value::as_u64), Some(8));
    assert!(
        ancient_dna.get("checks").and_then(serde_json::Value::as_array).is_some_and(|checks| {
            checks.iter().any(|value| {
                value.as_str()
                    == Some("vcf.qc stays downstream of propagated genotype-likelihood outputs")
            })
        }),
        "plan validate must surface the governed ancient-DNA genotype-likelihood checks"
    );
}
