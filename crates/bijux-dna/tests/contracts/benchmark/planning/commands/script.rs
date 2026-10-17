#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

use crate::support;

#[test]
fn bench_readiness_render_commands_writes_bash_parseable_script() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(support::cli_binary_path())
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-commands"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let script_path = repo_root.join("artifacts/benchmark-smoke/readiness/rendered-commands.sh");
    assert!(script_path.is_file(), "rendered command script must exist");

    let script = std::fs::read_to_string(&script_path).expect("read rendered command script");
    assert!(script.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n"));
    // The renderer's real-shell contract verifies admitted CWD and literal argv at each depth.
    // This public route retains the complete governed roster and Bash syntax checks.
    assert!(script.contains("# fastq.detect_adapters / "));
    assert!(script.contains("# bam.damage / mapdamage2"));

    let syntax = Command::new("bash").arg("-n").arg(&script_path).output().expect("run bash -n");
    assert!(syntax.status.success(), "bash -n failed: {}", String::from_utf8_lossy(&syntax.stderr));
}
