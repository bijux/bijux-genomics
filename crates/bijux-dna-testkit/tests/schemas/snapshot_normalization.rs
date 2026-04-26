use std::env;

#[test]
fn snapshot_name_includes_package_bucket_and_test_name() {
    let name = bijux_dna_testkit::snapshot_name("schemas", "public_api");

    assert_eq!(name, "bijux-dna-testkit__schemas__public_api");
}

#[test]
fn snapshot_normalize_text_normalizes_windows_paths_and_line_endings() {
    let raw = "C:\\Users\\alice\\repo\\file.txt\r\nnext";
    let normalized = bijux_dna_testkit::snapshot_normalize_text(raw);
    assert!(normalized.contains("C:/Users/alice/repo/file.txt"));
    assert!(!normalized.contains('\r'));
}

#[test]
fn install_snapshot_env_enforces_locale_and_timezone() {
    env::set_var("TZ", "Europe/Berlin");
    env::set_var("LC_ALL", "de_DE.UTF-8");
    bijux_dna_testkit::install_snapshot_env();
    assert_eq!(env::var("TZ").unwrap_or_default(), "UTC");
    assert_eq!(env::var("LC_ALL").unwrap_or_default(), "C");
}

#[test]
fn snapshot_normalize_json_keeps_malformed_duration_like_strings() {
    let raw = serde_json::json!({"value": "1.2.3s", "duration": "12ms", "valid": "1.25s"});
    let normalized = bijux_dna_testkit::snapshot_normalize_json(&raw);
    assert_eq!(normalized["value"], "1.2.3s");
    assert_eq!(normalized["valid"], "<DURATION>");
    assert!(normalized.get("duration").is_none());
}

#[test]
fn snapshot_normalize_json_keeps_non_timestamp_labels() {
    let raw = serde_json::json!({
        "label": "T-cell sample: A-B",
        "observed": "2024-01-01T02:03:04Z"
    });
    let normalized = bijux_dna_testkit::snapshot_normalize_json(&raw);
    assert_eq!(normalized["label"], "T-cell sample: A-B");
    assert_eq!(normalized["observed"], "<TIMESTAMP>");
}

#[test]
fn snapshot_normalize_text_ignores_empty_environment_values() {
    let previous = env::var_os("COMPUTERNAME");
    env::set_var("COMPUTERNAME", "");
    let normalized = bijux_dna_testkit::snapshot_normalize_text("abc");
    if let Some(previous) = previous {
        env::set_var("COMPUTERNAME", previous);
    } else {
        env::remove_var("COMPUTERNAME");
    }
    assert_eq!(normalized, "abc");
}

#[test]
fn snapshot_normalize_json_redacts_artifact_paths_inside_messages() {
    let raw = serde_json::json!({
        "message": "wrote /repo/artifacts/tmp/run-123/result.json successfully"
    });
    let normalized = bijux_dna_testkit::snapshot_normalize_json(&raw);
    assert_eq!(normalized["message"], "wrote result.json successfully");
}

#[test]
fn snapshot_normalize_json_preserves_tmp_command_paths() {
    let raw = serde_json::json!({
        "command": "mkdir -p '<TMPDIR>/<TMP>/out/fastqc' && 'fastqc' --outdir '<TMPDIR>/<TMP>/out/fastqc'"
    });
    let normalized = bijux_dna_testkit::snapshot_normalize_json(&raw);
    assert_eq!(normalized["command"], raw["command"]);
}

#[test]
fn snapshot_normalize_json_preserves_regex_escapes_in_commands() {
    let raw = serde_json::json!({
        "command": "python - <<'PY'\nimport re\nre.findall(r'EXAMINED:\\s*(\\d+)', text)\nPY"
    });
    let normalized = bijux_dna_testkit::snapshot_normalize_json(&raw);
    assert_eq!(normalized["command"], raw["command"]);
}

#[test]
fn snapshot_normalize_json_strips_camel_case_unstable_fields() {
    let raw = serde_json::json!({
        "startedAt": "2024-01-01T02:03:04Z",
        "elapsedMs": 12,
        "stable": true
    });
    let normalized = bijux_dna_testkit::snapshot_normalize_json(&raw);
    assert_eq!(normalized, serde_json::json!({"stable": true}));
}
