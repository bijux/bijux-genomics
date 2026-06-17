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

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
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
fn bench_readiness_real_output_parser_smoke_report_governs_retained_family_parsers() {
    let payload = run_cli_json(&["bench", "readiness", "render-real-output-parser-smoke", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.real_output_parser_smoke.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/tools/real-output-parser-smoke.json")
    );
    assert_eq!(payload.get("family_count").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(
        payload.get("passed_family_count").and_then(serde_json::Value::as_u64),
        Some(25)
    );
    assert_eq!(
        payload.get("failed_family_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );

    let parser_surface_counts = payload
        .get("parser_surface_counts")
        .and_then(serde_json::Value::as_object)
        .expect("parser surface counts");
    assert_eq!(
        parser_surface_counts
            .get("serde_json::<BamAlignmentProvenanceV1>")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        parser_surface_counts
            .get("serde_json::<BamAuthenticityAdvisoryV1>")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        parser_surface_counts
            .get("serde_json::<BamKinshipSummaryV1>")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        parser_surface_counts
            .get("serde_json::<Value> + governed keys")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 25);

    assert!(rows.iter().any(|row| {
        row.get("family_id").and_then(serde_json::Value::as_str) == Some("alignment")
            && row.get("representative_tool_id").and_then(serde_json::Value::as_str)
                == Some("bowtie2")
            && row.get("parser_surface").and_then(serde_json::Value::as_str)
                == Some("serde_json::<BamAlignmentProvenanceV1>")
            && row.get("parsed_schema_version").and_then(serde_json::Value::as_str)
                == Some("bijux.bam.alignment_provenance.v1")
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));

    assert!(rows.iter().any(|row| {
        row.get("family_id").and_then(serde_json::Value::as_str)
            == Some("authenticity_assessment")
            && row.get("representative_tool_id").and_then(serde_json::Value::as_str)
                == Some("authenticct")
            && row.get("parser_surface").and_then(serde_json::Value::as_str)
                == Some("serde_json::<BamAuthenticityAdvisoryV1>")
            && row.get("parsed_schema_version").and_then(serde_json::Value::as_str)
                == Some("bijux.bam.authenticity_advisory.v1")
            && row.get("normalized_snapshot")
                .and_then(|value| value.get("pmd_like_signal_present"))
                .and_then(serde_json::Value::as_bool)
                == Some(true)
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));

    assert!(rows.iter().any(|row| {
        row.get("family_id").and_then(serde_json::Value::as_str)
            == Some("kinship_relatedness")
            && row.get("representative_tool_id").and_then(serde_json::Value::as_str)
                == Some("king")
            && row.get("parser_surface").and_then(serde_json::Value::as_str)
                == Some("serde_json::<BamKinshipSummaryV1>")
            && row.get("parsed_schema_version").and_then(serde_json::Value::as_str)
                == Some("bijux.bam.kinship_summary.v1")
            && row
                .get("normalized_snapshot")
                .and_then(|value| value.get("observed_max_overlap_snps"))
                .and_then(serde_json::Value::as_u64)
                == Some(6)
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));

    assert!(rows.iter().any(|row| {
        row.get("family_id").and_then(serde_json::Value::as_str)
            == Some("genotyping_and_population_inference")
            && row.get("representative_tool_id").and_then(serde_json::Value::as_str)
                == Some("angsd")
            && row.get("parser_surface").and_then(serde_json::Value::as_str)
                == Some("serde_json::<Value> + governed keys")
            && row.get("parsed_schema_version").and_then(serde_json::Value::as_str)
                == Some("bijux.bam.genotyping.v1")
            && row
                .get("normalized_snapshot")
                .and_then(|value| value.get("producer"))
                .and_then(serde_json::Value::as_str)
                == Some("bam.genotyping")
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));
}

#[test]
fn bench_readiness_real_output_parser_smoke_writes_governed_json_file() {
    let output = run_cli(&["bench", "readiness", "render-real-output-parser-smoke"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let repo_root = support::repo_root().expect("repo root");
    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/tools/real-output-parser-smoke.json");

    let report_path = repo_root.join(rendered_path.trim());
    assert!(report_path.is_file(), "real-output parser smoke report JSON must exist");

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.real_output_parser_smoke.v1")
    );
    assert_eq!(payload.get("family_count").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(
        payload.get("passed_family_count").and_then(serde_json::Value::as_u64),
        Some(25)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(
        rows.iter().any(|row| {
            row.get("family_id").and_then(serde_json::Value::as_str)
                == Some("taxonomy_classification")
                && row.get("parser_surface").and_then(serde_json::Value::as_str)
                    == Some("fastq::parse_screen_taxonomy_report")
                && row.get("proof_path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                    path.ends_with("screen_taxonomy/classification_report.json")
                })
        }),
        "report file must retain governed taxonomy parser coverage"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("family_id").and_then(serde_json::Value::as_str)
                == Some("sex_and_haplogroup_inference")
                && row.get("parser_surface").and_then(serde_json::Value::as_str)
                    == Some("bam::parse_sex_json")
                && row
                    .get("normalized_snapshot")
                    .and_then(|value| value.get("method"))
                    .and_then(serde_json::Value::as_str)
                    == Some("rxy")
        }),
        "report file must retain governed BAM sex parser coverage"
    );
}
