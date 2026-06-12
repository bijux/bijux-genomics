#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_render_command_argv_writes_governed_jsonl() {
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
        .args(["bench", "readiness", "render-command-argv"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let jsonl_path = repo_root.join("benchmarks/readiness/rendered-commands.argv.jsonl");
    assert!(jsonl_path.is_file(), "rendered command argv JSONL must exist");

    let jsonl = std::fs::read_to_string(&jsonl_path).expect("read rendered command argv JSONL");
    let rows = jsonl.lines().collect::<Vec<_>>();
    assert_eq!(rows.len(), 115);
    assert!(rows.iter().all(|line| {
        serde_json::from_str::<serde_json::Value>(line).ok().is_some_and(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str).is_some()
                && row.get("tool_id").and_then(serde_json::Value::as_str).is_some()
                && row
                    .get("argv")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|argv| argv.first().and_then(serde_json::Value::as_str).is_some())
        })
    }));
}
