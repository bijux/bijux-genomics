#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_fastq_normalized_metrics_schema_writes_governed_schema_file() {
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
        .args(["bench", "readiness", "render-fastq-normalized-metrics-schema"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/schemas/fastq-normalized-metrics.v1.json");

    let schema_path = repo_root.join(rendered_path.trim());
    let schema = std::fs::read_to_string(&schema_path).expect("read schema");
    let parsed: serde_json::Value = serde_json::from_str(&schema).expect("parse schema");

    assert_eq!(
        parsed.get("$schema").and_then(serde_json::Value::as_str),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert_eq!(
        parsed.get("$id").and_then(serde_json::Value::as_str),
        Some("bijux.schemas.bench.fastq-normalized-metrics.v1")
    );
    assert_eq!(parsed.get("oneOf").and_then(serde_json::Value::as_array).map(Vec::len), Some(27));

    let stage_defs = parsed
        .get("$defs")
        .and_then(|value| value.get("stages"))
        .and_then(serde_json::Value::as_object)
        .expect("stage defs");
    assert_eq!(stage_defs.len(), 27);

    let detect_duplicates = stage_defs
        .get("fastq.detect_duplicates_premerge")
        .and_then(|value| value.get("allOf"))
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(1))
        .expect("duplicate-signal extension");
    assert_eq!(
        detect_duplicates.get("x-bijux-extension-id").and_then(serde_json::Value::as_str),
        Some("fastq_detect_duplicates_premerge_v1")
    );
    let required_keys = detect_duplicates
        .get("required")
        .and_then(serde_json::Value::as_array)
        .expect("required keys");
    assert!(required_keys.contains(&serde_json::json!("schema_version")));
    assert!(required_keys.contains(&serde_json::json!("stage")));
    assert!(required_keys.contains(&serde_json::json!("report_json")));
    assert!(required_keys.contains(&serde_json::json!("duplicate_count")));
    assert!(required_keys.contains(&serde_json::json!("duplicate_fraction")));
    assert!(required_keys.contains(&serde_json::json!("inspected_pair_count")));

    let trim_reads = stage_defs
        .get("fastq.trim_reads")
        .and_then(|value| value.get("allOf"))
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(1))
        .expect("trim-reads extension");
    let trim_reads_required = trim_reads
        .get("required")
        .and_then(serde_json::Value::as_array)
        .expect("trim-reads required keys");
    for key in ["reads_retained", "reads_dropped", "bases_removed"] {
        assert!(
            trim_reads_required.contains(&serde_json::json!(key)),
            "trim_reads normalized schema must require `{key}`"
        );
    }

    let trim_polyg = stage_defs
        .get("fastq.trim_polyg_tails")
        .and_then(|value| value.get("allOf"))
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(1))
        .expect("trim-polyg extension");
    let trim_polyg_required = trim_polyg
        .get("required")
        .and_then(serde_json::Value::as_array)
        .expect("trim-polyg required keys");
    for key in [
        "reads_retained",
        "reads_dropped",
        "bases_removed",
        "trimmed_tail_count",
        "bases_trimmed_polyg",
    ] {
        assert!(
            trim_polyg_required.contains(&serde_json::json!(key)),
            "trim_polyg_tails normalized schema must require `{key}`"
        );
    }

    let trim_terminal_damage = stage_defs
        .get("fastq.trim_terminal_damage")
        .and_then(|value| value.get("allOf"))
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(1))
        .expect("trim-terminal-damage extension");
    let trim_terminal_damage_required = trim_terminal_damage
        .get("required")
        .and_then(serde_json::Value::as_array)
        .expect("trim-terminal-damage required keys");
    for key in ["reads_retained", "bases_removed", "trim_5p_bases", "trim_3p_bases"] {
        assert!(
            trim_terminal_damage_required.contains(&serde_json::json!(key)),
            "trim_terminal_damage normalized schema must require `{key}`"
        );
    }

    let filter_low_complexity = stage_defs
        .get("fastq.filter_low_complexity")
        .and_then(|value| value.get("allOf"))
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(1))
        .expect("filter-low-complexity extension");
    let filter_low_complexity_required = filter_low_complexity
        .get("required")
        .and_then(serde_json::Value::as_array)
        .expect("filter-low-complexity required keys");
    for key in [
        "filtered_fastq_r1",
        "reads_dropped",
        "reads_retained",
        "reads_removed",
        "reads_removed_low_complexity",
    ] {
        assert!(
            filter_low_complexity_required.contains(&serde_json::json!(key)),
            "filter_low_complexity normalized schema must require `{key}`"
        );
    }
}
