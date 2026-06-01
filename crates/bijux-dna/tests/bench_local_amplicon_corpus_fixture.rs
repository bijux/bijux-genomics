#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_validate_amplicon_corpus_fixture_json_reports_governed_corpus_03_contract() {
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
            "tests/fixtures/corpora/corpus-03-amplicon-mini/manifest.toml",
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
        Some("bijux.bench.amplicon_corpus_fixture_validation.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/corpora/corpus-03-amplicon-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-03-amplicon-mini")
    );
    assert_eq!(
        payload.get("assay_id").and_then(serde_json::Value::as_str),
        Some("amplicon_standard")
    );
    assert_eq!(payload.get("marker_id").and_then(serde_json::Value::as_str), Some("16S"));
    assert_eq!(
        payload.get("target_region").and_then(serde_json::Value::as_str),
        Some("bacterial_16s_rrna_full_length")
    );
    assert_eq!(
        payload.get("primer_set_id").and_then(serde_json::Value::as_str),
        Some("16S_universal_v1")
    );
    assert_eq!(
        payload.get("primer_fasta").and_then(serde_json::Value::as_str),
        Some("assets/reference/primers/16S_universal_v1.fasta")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("control_count").and_then(serde_json::Value::as_u64), Some(1));
    assert!(payload.get("valid").and_then(serde_json::Value::as_bool) == Some(true));
    assert!(payload.get("controls").and_then(serde_json::Value::as_array).is_some_and(
        |controls| {
            controls.len() == 1
                && controls.iter().any(|control| {
                    control.get("sample_id").and_then(serde_json::Value::as_str)
                        == Some("chimera-control-se")
                        && control.get("control_kind").and_then(serde_json::Value::as_str)
                            == Some("chimera_positive")
                })
        }
    ));
    assert!(payload.get("samples").and_then(serde_json::Value::as_array).is_some_and(|samples| {
        samples.len() == 4
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("corpus-03-amplicon-se")
                    && sample.get("sample_kind").and_then(serde_json::Value::as_str)
                        == Some("biological")
                    && sample.get("observed_read_count").and_then(serde_json::Value::as_u64)
                        == Some(3)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("chimera-control-se")
                    && sample.get("sample_kind").and_then(serde_json::Value::as_str)
                        == Some("control")
            })
    }));
}
