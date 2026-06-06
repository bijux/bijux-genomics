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
fn bench_local_vcf_reference_compatibility_reports_governed_contig_parity() {
    let payload =
        run_cli_json(&["bench", "local", "validate-vcf-reference-compatibility", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_vcf_reference_compatibility.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/local-ready/vcf/reference-compatibility.json")
    );
    assert_eq!(payload.get("corpus_id").and_then(serde_json::Value::as_str), Some("vcf-mini"));
    assert_eq!(
        payload.get("reference_id").and_then(serde_json::Value::as_str),
        Some("vcf-mini-reference")
    );
    assert_eq!(
        payload.get("fasta_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/corpora/vcf-mini/reference/vcf_mini_reference.fasta")
    );
    assert_eq!(
        payload.get("fai_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/corpora/vcf-mini/reference/vcf_mini_reference.fasta.fai")
    );
    assert_eq!(
        payload.get("dict_path").and_then(serde_json::Value::as_str),
        Some("tests/fixtures/corpora/vcf-mini/reference/vcf_mini_reference.dict")
    );
    assert_eq!(payload.get("contig_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("status").and_then(serde_json::Value::as_str), Some("compatible"));
    assert_eq!(
        payload
            .get("reference_contigs")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["chr1", "chr2"])
    );
    assert_eq!(
        payload
            .get("vcf_contigs")
            .and_then(serde_json::Value::as_array)
            .map(|values| values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()),
        Some(vec!["chr1", "chr2"])
    );
    assert!(
        payload
            .get("missing_contigs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|values| values.is_empty())
    );
    assert!(
        payload
            .get("extra_contigs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|values| values.is_empty())
    );

    let variant_sets = payload
        .get("variant_sets")
        .and_then(serde_json::Value::as_array)
        .expect("variant_sets array");
    assert_eq!(variant_sets.len(), 5);
    assert!(variant_sets.iter().any(|row| {
        row.get("variant_role").and_then(serde_json::Value::as_str) == Some("multisample")
            && row
                .get("contigs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|values| {
                    values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
                        == vec!["chr1", "chr2"]
                })
    }));
}
