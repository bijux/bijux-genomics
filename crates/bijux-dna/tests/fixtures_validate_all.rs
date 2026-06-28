#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

const EXPECTED_ROWS: &[&[(&str, &str)]] = &[
    &[
        ("fixture_kind", "expected_truth"),
        ("fixture_id", "vcf-mini"),
        ("detail_path", "benchmarks/tests/fixtures/corpora/vcf-mini/expected"),
    ],
    &[
        ("fixture_kind", "database"),
        ("fixture_id", "taxonomy-mini"),
        ("manifest_path", "benchmarks/tests/fixtures/databases/taxonomy-mini/manifest.toml"),
    ],
    &[
        ("fixture_kind", "science_fixture"),
        ("fixture_id", "fastq-trimming-truth"),
        ("manifest_path", "benchmarks/tests/fixtures/science/fastq-trimming-truth/manifest.toml"),
    ],
    &[
        ("fixture_kind", "expected_truth"),
        ("fixture_id", "fastq-trimming-truth"),
        ("detail_path", "benchmarks/tests/fixtures/science/fastq-trimming-truth/expected.json"),
    ],
    &[
        ("fixture_kind", "science_fixture"),
        ("fixture_id", "adna-contamination-truth"),
        (
            "manifest_path",
            "benchmarks/tests/fixtures/science/adna-contamination-truth/manifest.toml",
        ),
    ],
    &[
        ("fixture_kind", "expected_truth"),
        ("fixture_id", "adna-contamination-truth"),
        ("detail_path", "benchmarks/tests/fixtures/science/adna-contamination-truth/expected.json"),
    ],
    &[
        ("fixture_kind", "science_fixture"),
        ("fixture_id", "bam-gc-coverage-truth"),
        ("manifest_path", "benchmarks/tests/fixtures/science/bam-gc-coverage-truth/manifest.toml"),
    ],
    &[
        ("fixture_kind", "expected_truth"),
        ("fixture_id", "bam-gc-coverage-truth"),
        ("detail_path", "benchmarks/tests/fixtures/science/bam-gc-coverage-truth/expected.json"),
    ],
    &[
        ("fixture_kind", "science_fixture"),
        ("fixture_id", "vcf-genotype-truth"),
        ("manifest_path", "benchmarks/tests/fixtures/science/vcf-genotype-truth/manifest.toml"),
    ],
    &[
        ("fixture_kind", "expected_truth"),
        ("fixture_id", "vcf-genotype-truth"),
        ("detail_path", "benchmarks/tests/fixtures/science/vcf-genotype-truth/expected.json"),
    ],
    &[
        ("fixture_kind", "science_fixture"),
        ("fixture_id", "population-structure-truth"),
        (
            "manifest_path",
            "benchmarks/tests/fixtures/science/population-structure-truth/manifest.toml",
        ),
    ],
    &[
        ("fixture_kind", "expected_truth"),
        ("fixture_id", "population-structure-truth"),
        (
            "detail_path",
            "benchmarks/tests/fixtures/science/population-structure-truth/expected.json",
        ),
    ],
    &[
        ("fixture_kind", "science_fixture"),
        ("fixture_id", "segments-demography-truth"),
        (
            "manifest_path",
            "benchmarks/tests/fixtures/science/segments-demography-truth/manifest.toml",
        ),
    ],
    &[
        ("fixture_kind", "expected_truth"),
        ("fixture_id", "segments-demography-truth"),
        (
            "detail_path",
            "benchmarks/tests/fixtures/science/segments-demography-truth/expected.json",
        ),
    ],
];

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _repo_lock =
        support::RepoProcessLock::acquire("benchmark-readiness-mutators").expect("repo lock");
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
fn fixtures_validate_all_reports_benchmark_root_pass_state() {
    let payload = run_cli_json(&[
        "fixtures",
        "validate",
        "--root",
        "benchmarks/tests/fixtures",
        "--all",
        "--json",
    ]);

    assert_metadata(&payload);
    assert_expected_rows(&payload, EXPECTED_ROWS);
}

fn assert_metadata(payload: &serde_json::Value) {
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.fixture_root_validation.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/benchmark-fixture-root-validation.json")
    );
    assert_eq!(
        payload.get("root_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures")
    );
    assert_eq!(payload.get("required_subroot_count").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(payload.get("parser_domain_count").and_then(serde_json::Value::as_u64), Some(3));
    assert_eq!(payload.get("checked_fixture_count").and_then(serde_json::Value::as_u64), Some(54));
    assert_eq!(payload.get("invalid_fixture_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
}

fn assert_expected_rows(payload: &serde_json::Value, expected_rows: &[&[(&str, &str)]]) {
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    for expected in expected_rows {
        assert_row_exists(rows, expected);
    }
}

fn assert_row_exists(rows: &[serde_json::Value], expected: &[(&str, &str)]) {
    assert!(rows.iter().any(|row| row_matches(row, expected)), "missing row matching {expected:?}");
}

fn row_matches(row: &serde_json::Value, expected: &[(&str, &str)]) -> bool {
    expected
        .iter()
        .all(|(key, value)| row.get(*key).and_then(serde_json::Value::as_str) == Some(*value))
}
