#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_local_judge_taxonomy_output_json_reports_governed_corpus_02_match() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let output_path = home.path().join("taxonomy-judgment.json");

    let output = Command::new(support::cli_binary_path())
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args([
            "bench",
            "local",
            "judge-taxonomy-output",
            "--manifest",
            "benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml",
            "--report",
            "mock_community_sample_a=benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/observed_taxonomy/mock_community_sample_a.classification_report.json",
            "--report",
            "mock_community_sample_b=benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/observed_taxonomy/mock_community_sample_b.classification_report.json",
            "--output",
            output_path.to_str().expect("utf8 output path"),
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
        Some("bijux.bench.local_taxonomy_output_judgment.v1")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml")
    );
    assert_eq!(
        payload.get("expected_taxa_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/expected_taxa.tsv")
    );
    assert_eq!(payload.get("sample_count").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(payload.get("expectation_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(
        payload.get("matched_expectation_count").and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(
        payload.get("mismatched_expectation_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert!(payload.get("valid").and_then(serde_json::Value::as_bool) == Some(true));
    assert!(payload.get("samples").and_then(serde_json::Value::as_array).is_some_and(|samples| {
        samples.len() == 2
            && samples.iter().all(|sample| {
                sample.get("valid").and_then(serde_json::Value::as_bool) == Some(true)
                    && sample
                        .get("mismatched_expectation_count")
                        .and_then(serde_json::Value::as_u64)
                        == Some(0)
            })
            && samples.iter().any(|sample| {
                sample.get("sample_id").and_then(serde_json::Value::as_str)
                    == Some("mock_community_sample_b")
                    && sample
                        .get("observed_unclassified_percent")
                        .and_then(serde_json::Value::as_f64)
                        == Some(0.0)
                    && sample
                        .get("observed_unclassified_read_count")
                        .and_then(serde_json::Value::as_u64)
                        == Some(0)
                    && sample.get("false_positive_count").and_then(serde_json::Value::as_u64)
                        == Some(0)
                    && sample
                        .get("observed_taxa")
                        .and_then(serde_json::Value::as_array)
                        .is_some_and(|taxa| {
                            taxa.len() == 1
                                && taxa[0].get("name").and_then(serde_json::Value::as_str)
                                    == Some("Halobacterium salinarum")
                        })
            })
    }));

    let written = std::fs::read_to_string(&output_path).expect("read written report");
    let written_payload: serde_json::Value =
        serde_json::from_str(&written).expect("parse written report");
    assert_eq!(written_payload, payload);
}

#[test]
fn bench_local_judge_taxonomy_output_refuses_declared_mismatch_after_publication(
) -> anyhow::Result<()> {
    let _guard = support::CWD_LOCK
        .lock()
        .map_err(|error| anyhow::anyhow!("lock current-directory fixture: {error}"))?;
    let repo_root = support::repo_root()?;
    let output_root = tempfile::tempdir()?;
    let sample_a = output_root.path().join("sample-a.json");
    let sample_b = output_root.path().join("sample-b.json");
    let report_path = output_root.path().join("taxonomy-judgment.json");
    std::fs::write(
        &sample_a,
        serde_json::to_vec(&serde_json::json!({
            "summary_entries": [
                {"label": "Escherichia coli", "percent": 100.0}
            ]
        }))?,
    )?;
    std::fs::write(
        &sample_b,
        serde_json::to_vec(&serde_json::json!({
            "summary_entries": [
                {"label": "Halobacterium salinarum", "percent": 100.0}
            ]
        }))?,
    )?;
    let output = Command::new(support::cli_binary_path())
        .current_dir(&repo_root)
        .env("HOME", output_root.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args([
            "bench",
            "local",
            "judge-taxonomy-output",
            "--manifest",
            "benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml",
            "--report",
        ])
        .arg(format!("mock_community_sample_a={}", sample_a.display()))
        .arg("--report")
        .arg(format!("mock_community_sample_b={}", sample_b.display()))
        .arg("--output")
        .arg(&report_path)
        .arg("--json")
        .output()?;
    let written = std::fs::read(&report_path)?;
    let report: serde_json::Value = serde_json::from_slice(&written)?;
    assert_eq!(report["schema_version"], "bijux.bench.local_taxonomy_output_judgment.v1");
    assert_eq!(report["sample_count"], 2);
    assert_eq!(report["expectation_count"], 6);
    assert_eq!(report["matched_expectation_count"], 5);
    assert_eq!(report["mismatched_expectation_count"], 1);
    assert_eq!(report["valid"], false);
    assert!(
        !output.status.success(),
        "documented expected-presence refusal must return failure after retaining its report; status={} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut printed = written;
    printed.push(b'\n');
    assert_eq!(output.stdout, printed, "invalid report JSON must remain machine-readable");
    assert_eq!(output.status.code(), Some(3), "semantic refusal must use the contract category");
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("operator_failure category=ContractError"), "{diagnostic}");
    assert!(diagnostic.contains("taxonomy output judgment is invalid"), "{diagnostic}");
    Ok(())
}

#[test]
fn bench_local_taxonomy_observation_parse_category_is_independent_of_input_path(
) -> anyhow::Result<()> {
    use std::path::Path;
    let repo_root = support::repo_root()?;
    let private = bijux_dna_testkit::tempdir_for("taxonomy-observation-categories");
    let root = private.path();
    let valid_a = root.join("sample-a.json");
    let valid_b = root.join("sample-b.json");
    std::fs::write(&valid_a, br#"{"summary_entries":[{"label":"Escherichia coli","percent":60},{"label":"Salmonella enterica","percent":40}]}"#)?;
    std::fs::write(
        &valid_b,
        br#"{"summary_entries":[{"label":"Halobacterium salinarum","percent":100}]}"#,
    )?;
    let input_a = std::fs::read(&valid_a)?;
    let input_b = std::fs::read(&valid_b)?;
    let run = |label: &str, mappings: &[(&str, &Path)]| -> anyhow::Result<_> {
        let report = root.join(label).join("judgment.json");
        let mut command = support::bijux_dna_command(&repo_root);
        command
            .env("HOME", root)
            .env("BIJUX_REPO_ROOT", &repo_root)
            .env("BIJUX_SKIP_QA", "1")
            .env("BIJUX_ALLOW_SILVER", "1")
            .env("BIJUX_SKIP_IMAGE_CHECK", "1")
            .env("TEST_TMP_DIR", root)
            .env("TMPDIR", root)
            .env("TMP", root)
            .env("TEMP", root)
            .args([
                "bench",
                "local",
                "judge-taxonomy-output",
                "--manifest",
                "benchmarks/tests/fixtures/corpora/corpus-02-edna-mini/manifest.toml",
            ])
            .arg("--output")
            .arg(&report)
            .arg("--json");
        for (sample, path) in mappings {
            command.arg("--report").arg(format!("{sample}={}", path.display()));
        }
        Ok((command.output()?, report))
    };
    let sample_a = "mock_community_sample_a";
    let sample_b = "mock_community_sample_b";
    let (positive, positive_report) =
        run("positive", &[(sample_a, &valid_a), (sample_b, &valid_b)])?;
    assert!(positive.status.success(), "{}", String::from_utf8_lossy(&positive.stderr));
    let positive_bytes = std::fs::read(&positive_report)?;
    let positive_value: serde_json::Value = serde_json::from_slice(&positive_bytes)?;
    assert_eq!(positive_value["valid"], true);
    let mut printed = positive_bytes;
    printed.push(b'\n');
    assert_eq!(positive.stdout, printed);

    let malformed = root.join("malformed.json");
    std::fs::write(&malformed, b"{")?;
    let (syntax, syntax_report) = run("syntax", &[(sample_a, &malformed), (sample_b, &valid_b)])?;
    assert_eq!(syntax.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&syntax.stderr).contains("EOF while parsing"));
    assert!(!syntax_report.exists());
    let absent = root.join("absent.json");
    let (missing, missing_report) = run("absent", &[(sample_a, &absent), (sample_b, &valid_b)])?;
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("report path is missing"));
    assert!(!missing_report.exists());
    let (duplicate, duplicate_report) =
        run("duplicate", &[(sample_a, &valid_a), (sample_b, &valid_b), (sample_a, &valid_a)])?;
    assert!(!duplicate.status.success());
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("repeats sample_id"));
    assert!(!duplicate_report.exists());
    let (foreign, foreign_report) =
        run("foreign", &[("foreign-sample", &absent), (sample_b, &valid_b)])?;
    let foreign_error = String::from_utf8_lossy(&foreign.stderr);
    assert!(!foreign.status.success());
    assert!(foreign_error.contains("not declared by the fixture manifest"));
    assert!(!foreign_error.contains("report path is missing"));
    assert!(!foreign_report.exists());

    let mut uncategorized = Vec::new();
    for label in ["plain", "contract"] {
        let directory = root.join(label);
        std::fs::create_dir(&directory)?;
        let observation = directory.join("summary.json");
        let raw = br#"{"summary_entries":null,"top_taxa":[]}"#;
        std::fs::write(&observation, raw)?;
        let (output, report) = run(label, &[(sample_a, &observation), (sample_b, &valid_b)])?;
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success());
        assert!(error.contains("must declare a `summary_entries` or `top_taxa` array"), "{error}");
        assert!(output.stdout.is_empty() && !report.exists());
        assert_eq!(std::fs::read(&observation)?.as_slice(), raw);
        if output.status.code() != Some(4)
            || !error.contains("operator_failure category=ParseError")
            || !error.contains("hint id=observer.parse")
        {
            uncategorized.push(format!("{label}: status={} stderr={error}", output.status));
        }
    }
    assert_eq!(std::fs::read(&valid_a)?, input_a);
    assert_eq!(std::fs::read(&valid_b)?, input_b);
    eprintln!("TAXONOMY_PUBLIC_CATEGORY_CONTROLS_PASS: valid pretty publication, malformed JSON exit4, absent input, duplicate mapping, foreign-before-missing; same wrong-shape bytes at plain/contract roots; uncategorized={uncategorized:?}");
    assert!(
        uncategorized.is_empty(),
        "taxonomy observation grammar must use typed ParseError exit4 independently of path: {uncategorized:?}"
    );
    Ok(())
}
