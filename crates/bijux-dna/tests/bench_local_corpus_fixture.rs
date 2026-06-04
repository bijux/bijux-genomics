#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_validate_corpus_fixture_json_reports_governed_corpus_01_mini_contract() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new("cargo")
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["run", "-q", "-p", "bijux-dna", "--"])
        .args([
            "bench",
            "local",
            "validate-corpus-fixture",
            "--manifest",
            "tests/fixtures/corpora/corpus-01-mini/manifest.toml",
            "--json",
        ])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse stdout as json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.fastq_corpus_fixture_validation.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/corpora/corpus-01-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-mini")
    );
    assert_eq!(payload.get("compression").and_then(serde_json::Value::as_str), Some("gzip"));
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("single_end_sample_count").and_then(serde_json::Value::as_u64), Some(5));
    assert_eq!(payload.get("paired_end_sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert!(payload.get("valid").and_then(serde_json::Value::as_bool) == Some(true));
    assert!(payload.get("samples").and_then(serde_json::Value::as_array).is_some_and(|samples| {
        samples.len() == 9
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_pe_merge_overlap")
                    && sample.get("layout").and_then(serde_json::Value::as_str) == Some("pe")
                    && sample.get("r1_path").and_then(serde_json::Value::as_str)
                        == Some(
                            "tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz"
                        )
                    && sample.get("r2_path").and_then(serde_json::Value::as_str)
                        == Some(
                            "tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz"
                        )
                    && sample
                        .get("source_paths")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|source_paths| {
                            source_paths
                                == &vec![
                                    serde_json::Value::String(
                                        "assets/toy/core-v1/fastq/reads_with_merge_overlap_R1.fastq"
                                            .to_string(),
                                    ),
                                    serde_json::Value::String(
                                        "assets/toy/core-v1/fastq/reads_with_merge_overlap_R2.fastq"
                                            .to_string(),
                                    ),
                                ]
                        })
                    && sample.get("observed_read_count_r1").and_then(serde_json::Value::as_u64)
                        == Some(2)
                    && sample.get("observed_read_count_r2").and_then(serde_json::Value::as_u64)
                        == Some(2)
                    && sample.get("observed_read_count_total").and_then(serde_json::Value::as_u64)
                        == Some(4)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_pe_duplicate_signals")
                    && sample.get("layout").and_then(serde_json::Value::as_str) == Some("pe")
                    && sample.get("observed_read_count_total").and_then(serde_json::Value::as_u64)
                        == Some(6)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_pe_distinct_pairs")
                    && sample.get("layout").and_then(serde_json::Value::as_str) == Some("pe")
                    && sample.get("observed_read_count_total").and_then(serde_json::Value::as_u64)
                        == Some(4)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_se_adapter_hit")
                    && sample.get("layout").and_then(serde_json::Value::as_str) == Some("se")
                    && sample.get("observed_read_count_total").and_then(serde_json::Value::as_u64)
                        == Some(2)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_se_filter_signals")
                    && sample.get("layout").and_then(serde_json::Value::as_str) == Some("se")
                    && sample.get("observed_read_count_total").and_then(serde_json::Value::as_u64)
                        == Some(3)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("human_like_se_polyg_trim_signals")
                    && sample.get("layout").and_then(serde_json::Value::as_str) == Some("se")
                    && sample.get("observed_read_count_total").and_then(serde_json::Value::as_u64)
                        == Some(3)
            })
    }));
}
