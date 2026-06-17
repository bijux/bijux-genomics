#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

pub(crate) fn seed_tool_smoke_manifests(repo_root: &std::path::Path) {
    let version_probes: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo_root.join("benchmarks/readiness/tools/version-probes.json"))
            .expect("read version probes"),
    )
    .expect("parse version probes");
    let rows = version_probes
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .expect("version probe rows");

    for row in rows {
        let tool_id = row.get("tool_id").and_then(serde_json::Value::as_str).expect("tool id");
        let resolution_kind = row
            .get("resolution_kind")
            .and_then(serde_json::Value::as_str)
            .expect("resolution kind");
        let version_probe_status = row
            .get("version_probe_status")
            .and_then(serde_json::Value::as_str)
            .expect("version probe status");
        let runtime_probe_paths = row
            .get("runtime_probe_paths")
            .and_then(serde_json::Value::as_array)
            .expect("runtime probe paths")
            .clone();
        let registry_paths = row
            .get("registry_paths")
            .and_then(serde_json::Value::as_array)
            .expect("registry paths")
            .clone();

        if resolution_kind == "host_binary" && version_probe_status == "ready" {
            let manifest = serde_json::json!({
                "schema_version": "bijux.bench.host_tool_smoke_manifest.v1",
                "tool_id": tool_id,
                "status": "ok",
                "resolution_kind": resolution_kind,
                "resolution_target": row.get("resolution_target").cloned().unwrap_or(serde_json::Value::String(String::new())),
                "command_entrypoint": row.get("command_entrypoint").cloned().unwrap_or(serde_json::Value::Null),
                "declared_command": row.get("version_cmd").and_then(serde_json::Value::as_str).unwrap_or("bijux-dna --version"),
                "applied_command": [tool_id, "--version"],
                "working_directory": ".",
                "exit_code": 0,
                "stdout": "bijux-dna 0.1.0\n",
                "stderr": "",
                "version": "0.1.0",
                "version_parser_kind": row.get("version_parser_kind").and_then(serde_json::Value::as_str).unwrap_or("first_dotted_numeric_token"),
                "expected_version_regex": row.get("expected_version_regex").and_then(serde_json::Value::as_str).unwrap_or("v?[0-9]+[.][0-9]+"),
                "version_matches_regex": true,
                "runtime_probe_paths": runtime_probe_paths,
                "registry_paths": registry_paths,
                "checked_at_unix_s": 0
            });
            let manifest_path =
                repo_root.join("runs/bench/tool-smoke/host").join(tool_id).join("manifest.json");
            std::fs::create_dir_all(manifest_path.parent().expect("host manifest parent"))
                .expect("create host manifest dir");
            std::fs::write(
                &manifest_path,
                serde_json::to_vec_pretty(&manifest).expect("host json"),
            )
            .expect("write host manifest");
            continue;
        }

        if resolution_kind == "host_binary" {
            continue;
        }

        let manifest = if version_probe_status == "ready" {
            serde_json::json!({
                "schema_version": "bijux.bench.container_tool_smoke_manifest.v1",
                "tool_id": tool_id,
                "domains": row.get("domains").cloned().unwrap_or(serde_json::Value::Array(Vec::new())),
                "active_stage_ids": row.get("active_stage_ids").cloned().unwrap_or(serde_json::Value::Array(Vec::new())),
                "status": "ok",
                "resolution_kind": resolution_kind,
                "resolution_target": row.get("resolution_target").cloned().unwrap_or(serde_json::Value::String(String::new())),
                "smoke_runtime": "docker-arm64",
                "command_entrypoint": row.get("command_entrypoint").cloned().unwrap_or(serde_json::Value::Null),
                "declared_command": format!("bijux-dna env smoke docker-arm64 {tool_id}"),
                "applied_command": ["artifacts/rust/target/debug/bijux-dna-dev", "containers", "run", "smoke-containers-docker-arm64"],
                "version_cmd": row.get("version_cmd").cloned().unwrap_or(serde_json::Value::Null),
                "help_cmd": row.get("help_cmd").cloned().unwrap_or(serde_json::Value::Null),
                "working_directory": ".",
                "exit_code": 0,
                "stdout": "smoke ok\n",
                "stderr": "",
                "unavailable_reason": serde_json::Value::Null,
                "runtime_probe_paths": runtime_probe_paths,
                "registry_paths": registry_paths,
                "checked_at_unix_s": 0
            })
        } else {
            serde_json::json!({
                "schema_version": "bijux.bench.container_tool_smoke_manifest.v1",
                "tool_id": tool_id,
                "domains": row.get("domains").cloned().unwrap_or(serde_json::Value::Array(Vec::new())),
                "active_stage_ids": row.get("active_stage_ids").cloned().unwrap_or(serde_json::Value::Array(Vec::new())),
                "status": "unavailable_with_reason",
                "resolution_kind": resolution_kind,
                "resolution_target": row.get("resolution_target").cloned().unwrap_or(serde_json::Value::String(String::new())),
                "smoke_runtime": serde_json::Value::Null,
                "command_entrypoint": row.get("command_entrypoint").cloned().unwrap_or(serde_json::Value::Null),
                "declared_command": serde_json::Value::Null,
                "applied_command": [],
                "version_cmd": row.get("version_cmd").cloned().unwrap_or(serde_json::Value::Null),
                "help_cmd": row.get("help_cmd").cloned().unwrap_or(serde_json::Value::Null),
                "working_directory": ".",
                "exit_code": serde_json::Value::Null,
                "stdout": "",
                "stderr": "",
                "unavailable_reason": row.get("unavailable_reason").cloned().unwrap_or(serde_json::Value::String("governed unavailable".to_string())),
                "runtime_probe_paths": runtime_probe_paths,
                "registry_paths": registry_paths,
                "checked_at_unix_s": 0
            })
        };
        let manifest_path =
            repo_root.join("runs/bench/tool-smoke/container").join(tool_id).join("manifest.json");
        std::fs::create_dir_all(manifest_path.parent().expect("container manifest parent"))
            .expect("create container manifest dir");
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("container json"),
        )
        .expect("write container manifest");
    }
}

#[test]
fn bench_readiness_retained_toolset_executable_local_reports_goal_430_gate() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    seed_tool_smoke_manifests(&repo_root);

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-retained-toolset-executable-local", "--json"])
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
        Some("bijux.bench.readiness.retained_toolset_executable_local.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/tools/RETAINED_TOOLSET_EXECUTABLE_LOCAL.json")
    );
    assert_eq!(payload.get("checked_goal_count").and_then(serde_json::Value::as_u64), Some(9));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("retained_tool_count").and_then(serde_json::Value::as_u64), Some(71));
    assert_eq!(
        payload.get("executable_resolution_row_count").and_then(serde_json::Value::as_u64),
        Some(71)
    );
    assert_eq!(
        payload.get("version_probe_row_count").and_then(serde_json::Value::as_u64),
        Some(71)
    );
    assert_eq!(payload.get("host_smoke_tool_count").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        payload.get("container_smoke_tool_count").and_then(serde_json::Value::as_u64),
        Some(70)
    );
    assert_eq!(payload.get("parser_family_count").and_then(serde_json::Value::as_u64), Some(25));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let checks = payload.get("checks").and_then(serde_json::Value::as_array).expect("checks");
    assert_eq!(checks.len(), 9);
    assert!(checks.iter().any(|check| {
        check.get("goal_id").and_then(serde_json::Value::as_u64) == Some(425)
            && check.get("ok").and_then(serde_json::Value::as_bool) == Some(true)
            && check
                .get("detail")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|detail| detail.contains("unavailable_count=2"))
    }));
}

#[test]
fn bench_readiness_retained_toolset_executable_local_writes_gate_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    seed_tool_smoke_manifests(&repo_root);

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-retained-toolset-executable-local"])
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
        "benchmarks/readiness/tools/RETAINED_TOOLSET_EXECUTABLE_LOCAL.json"
    );

    let payload: serde_json::Value = serde_json::from_slice(
        &std::fs::read(repo_root.join(rendered_path.trim())).expect("read retained gate"),
    )
    .expect("parse retained gate");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.retained_toolset_executable_local.v1")
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(payload.get("failed_goal_count").and_then(serde_json::Value::as_u64), Some(0));
}
