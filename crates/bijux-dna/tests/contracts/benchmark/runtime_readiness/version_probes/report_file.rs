#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

#[test]
fn bench_readiness_version_probes_writes_governed_json_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let runtime_root = tempfile::tempdir().expect("runtime tempdir");

    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux-dna"));
    support::configure_isolated_runtime_resolution(&mut command, runtime_root.path());
    let output = command
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-version-probes"])
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
    assert_eq!(
        rendered_path.trim(),
        "artifacts/benchmark-smoke/readiness/tools/version-probes.json"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read version probes json");
    let parsed: serde_json::Value = serde_json::from_str(&payload).expect("parse version probes");

    assert_eq!(
        parsed.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.version_probes.v1")
    );
    let rows = parsed.get("rows").and_then(serde_json::Value::as_array).expect("rows");
    let row_count = rows.len() as u64;
    let ready_count = support::json_u64(&parsed, "ready_count").expect("ready count");
    let unavailable_count =
        support::json_u64(&parsed, "unavailable_count").expect("unavailable count");
    assert_eq!(parsed.get("row_count").and_then(serde_json::Value::as_u64), Some(row_count));
    assert_eq!(ready_count + unavailable_count, row_count);
    assert_eq!(unavailable_count, 0);
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("beagle")
            && row.get("version_probe_status").and_then(serde_json::Value::as_str) == Some("ready")
            && row.get("expected_version_regex").and_then(serde_json::Value::as_str)
                == Some("beagle [0-9]+([.][0-9]+)?")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastuniq")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("external_artifact")
            && row.get("resolution_target").and_then(serde_json::Value::as_str)
                == Some("external-artifact://fastuniq")
            && row.get("version_probe_status").and_then(serde_json::Value::as_str) == Some("ready")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("shapeit5")
            && row.get("version_probe_status").and_then(serde_json::Value::as_str) == Some("ready")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("apptainer_image")
    }));
    assert!(rows.iter().any(|row| {
        row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink")
            && row.get("version_probe_status").and_then(serde_json::Value::as_str) == Some("ready")
            && row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("apptainer_image")
    }));
}
