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
fn bench_local_vcf_admixture_smoke_writes_governed_files() {
    let output = run_cli(&["bench", "local", "run-vcf-admixture-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "target/local-smoke/vcf.admixture/plink2/admixture.json"
    );

    let repo_root = support::repo_root().expect("repo root");
    let admixture_tsv_path =
        repo_root.join("target/local-smoke/vcf.admixture/plink2/admixture.tsv");
    let admixture_json_path =
        repo_root.join("target/local-smoke/vcf.admixture/plink2/admixture.json");
    let source_q_matrix_path =
        repo_root.join("target/local-smoke/vcf.admixture/plink2/source_admixture_q_matrix.tsv");
    let source_k_selection_path =
        repo_root.join("target/local-smoke/vcf.admixture/plink2/source_admixture_k_selection.json");
    let source_logs_path =
        repo_root.join("target/local-smoke/vcf.admixture/plink2/source_logs.txt");
    let stage_result_path =
        repo_root.join("target/local-smoke/vcf.admixture/plink2/stage-result.json");

    for path in [
        &admixture_tsv_path,
        &admixture_json_path,
        &source_q_matrix_path,
        &source_k_selection_path,
        &source_logs_path,
        &stage_result_path,
    ] {
        assert!(path.is_file(), "expected file at {}", path.display());
    }

    let admixture_tsv = std::fs::read_to_string(&admixture_tsv_path).expect("read admixture tsv");
    let admixture_lines = admixture_tsv.lines().collect::<Vec<_>>();
    assert_eq!(
        admixture_lines.first().copied(),
        Some("sample_id\tpopulation_id\tpopulation_label\tsex\tK\tstatus\tcluster_1\tcluster_2")
    );
    assert_eq!(admixture_lines.len(), 5);
    let observed_samples = admixture_lines
        .iter()
        .skip(1)
        .map(|line| line.split('\t').next().expect("sample_id column"))
        .collect::<Vec<_>>();
    assert_eq!(observed_samples, vec!["sample_a", "sample_b", "sample_c", "sample_d"]);

    let report_raw = std::fs::read_to_string(&admixture_json_path).expect("read admixture json");
    let report: serde_json::Value =
        serde_json::from_str(&report_raw).expect("parse admixture json");
    assert_eq!(report.get("selected_k").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(report.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(report.get("status").and_then(serde_json::Value::as_str), Some("complete"));

    let source_manifest_raw =
        std::fs::read_to_string(&source_k_selection_path).expect("read source admixture manifest");
    let source_manifest: serde_json::Value =
        serde_json::from_str(&source_manifest_raw).expect("parse source admixture manifest");
    assert_eq!(
        source_manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.vcf.admixture.v1")
    );
    assert_eq!(
        source_manifest
            .get("cluster_headers")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len()),
        Some(2)
    );
    assert_eq!(source_manifest.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));

    let manifest_raw = std::fs::read_to_string(&stage_result_path).expect("read stage result");
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.stage_result.v2")
    );
    assert_eq!(manifest.get("stage_id").and_then(serde_json::Value::as_str), Some("vcf.admixture"));
    assert_eq!(
        manifest.get("tool").and_then(|value| value.get("id")).and_then(serde_json::Value::as_str),
        Some("plink2")
    );
    assert_eq!(
        manifest
            .get("command")
            .and_then(|value| value.get("rendered"))
            .and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local run-vcf-admixture-smoke --tool-id plink2")
    );
    let outputs = manifest.get("outputs").and_then(serde_json::Value::as_array).expect("outputs");
    assert_eq!(outputs.len(), 5);
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("admixture_tsv")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.admixture/plink2/admixture.tsv")
    }));
    assert!(outputs.iter().any(|row| {
        row.get("artifact_id").and_then(serde_json::Value::as_str) == Some("admixture_json")
            && row.get("realized_path").and_then(serde_json::Value::as_str)
                == Some("target/local-smoke/vcf.admixture/plink2/admixture.json")
    }));
}
