#![allow(clippy::expect_used, clippy::too_many_lines)]

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
fn bench_local_vcf_damage_filter_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-damage-filter-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "runs/bench/local-smoke/vcf.damage_filter/bcftools/damage_filtered.vcf.gz"
    );

    let repo_root = support::repo_root().expect("repo root");
    let output_vcf =
        repo_root.join("runs/bench/local-smoke/vcf.damage_filter/bcftools/damage_filtered.vcf.gz");
    let output_tbi = repo_root
        .join("runs/bench/local-smoke/vcf.damage_filter/bcftools/damage_filtered.vcf.gz.tbi");
    let metrics_path =
        repo_root.join("runs/bench/local-smoke/vcf.damage_filter/bcftools/metrics.json");
    let summary_path = repo_root
        .join("runs/bench/local-smoke/vcf.damage_filter/bcftools/damage_filter_summary.json");
    let counts_path = repo_root
        .join("runs/bench/local-smoke/vcf.damage_filter/bcftools/damage_filter_counts.json");
    let warnings_path =
        repo_root.join("runs/bench/local-smoke/vcf.damage_filter/bcftools/warnings.json");
    let damage_manifest_path = repo_root
        .join("runs/bench/local-smoke/vcf.damage_filter/bcftools/damage_genotype_manifest.json");
    let manifest_path =
        repo_root.join("runs/bench/local-smoke/vcf.damage_filter/bcftools/stage-result.json");
    let input_vcf = repo_root
        .join("runs/bench/local-smoke/vcf.damage_filter/bcftools/artifacts/input/damage_input.vcf");

    assert!(output_vcf.is_file(), "expected output VCF at {}", output_vcf.display());
    assert!(output_tbi.is_file(), "expected output index at {}", output_tbi.display());
    assert!(metrics_path.is_file(), "expected metrics at {}", metrics_path.display());
    assert!(summary_path.is_file(), "expected summary at {}", summary_path.display());
    assert!(counts_path.is_file(), "expected counts at {}", counts_path.display());
    assert!(warnings_path.is_file(), "expected warnings at {}", warnings_path.display());
    assert!(
        damage_manifest_path.is_file(),
        "expected damage manifest at {}",
        damage_manifest_path.display()
    );
    assert!(manifest_path.is_file(), "expected stage result at {}", manifest_path.display());
    assert!(input_vcf.is_file(), "expected synthetic input at {}", input_vcf.display());

    let counts_raw = std::fs::read_to_string(&counts_path).expect("read counts");
    let counts: serde_json::Value = serde_json::from_str(&counts_raw).expect("parse counts");
    assert_eq!(counts.pointer("/counts/low_qual").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        counts.pointer("/counts/damage_ratio_exceeded").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        counts.pointer("/counts/terminal_damage_filtered").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(counts.pointer("/counts/kept").and_then(serde_json::Value::as_u64), Some(2));

    let manifest_raw = std::fs::read_to_string(&manifest_path).expect("read manifest");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(
        manifest.get("stage_id").and_then(serde_json::Value::as_str),
        Some("vcf.damage_filter")
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
        Some("bijux-dna bench local run-vcf-damage-filter-smoke --tool-id bcftools")
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
    assert_eq!(outputs.len(), 7);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("damage_filtered_vcf")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/local-smoke/vcf.damage_filter/bcftools/damage_filtered.vcf.gz")
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("metrics_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("runs/bench/local-smoke/vcf.damage_filter/bcftools/metrics.json")
    }));
}
