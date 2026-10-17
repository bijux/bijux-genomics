//! A finite descendant holding the real probe streams must not extend the declared deadline.
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use super::{run_apptainer_smoke, smoke_request, write_executable};
use anyhow::{ensure, Context as _, Result};

const ROOT: &str = "BIJUX_QA_DESCENDANT_FIXTURE";
const CASE: &str =
    "mod_contracts_apptainer_smoke_rs::smoke_bounds_execution_and_persists_immutable_streams";

pub(super) fn run_child_if_requested() -> Result<bool> {
    let Some(root) = std::env::var_os(ROOT) else {
        return Ok(false);
    };
    ensure!(
        std::env::args().skip(1).collect::<Vec<_>>() == [CASE, "--exact", "--nocapture"],
        "exact descendant child case"
    );
    let root = PathBuf::from(root);
    ensure!(root.is_absolute() && root.canonicalize()? == root, "private descendant fixture root");
    let executable = root.join("apptainer");
    write_executable(&executable, "#!/bin/sh\n/bin/sleep 4 &\nchild=$!\nprintf '%s %s\\n' \"$$\" \"$child\" >> \"$BIJUX_QA_DESCENDANT_PIDS\"\nprintf 'provider ready stdout\\n'\nprintf 'provider ready stderr\\n' >&2\nwait \"$child\"\n")?;
    let mut request = smoke_request(&root, executable, "contract-tool 1[.]0")?;
    request.timeout = Duration::from_secs(1);
    request.evidence_root = Some(root.join("evidence"));
    let start = Instant::now();
    let manifest = run_apptainer_smoke(&request)?;
    let elapsed = start.elapsed();
    ensure!(
        manifest.status == "wrapper_failed"
            && !manifest.probe_results.is_empty()
            && manifest.probe_results.iter().all(|probe| probe.timed_out),
        "actual timeout classification remains authoritative"
    );
    for probe in &manifest.probe_results {
        let stdout_path = probe.stdout_path.as_deref().context("actual timeout stdout path")?;
        let stderr_path = probe.stderr_path.as_deref().context("actual timeout stderr path")?;
        ensure!(
            std::fs::read_to_string(stdout_path)?.contains("provider ready stdout")
                && std::fs::read_to_string(stderr_path)?.contains("provider ready stderr"),
            "provider body must run and complete partial streams must persist"
        );
        let stdout_sha256 = bijux_dna_infra::hash_file_sha256(std::path::Path::new(stdout_path))?;
        let stderr_sha256 = bijux_dna_infra::hash_file_sha256(std::path::Path::new(stderr_path))?;
        ensure!(
            probe.stdout_sha256.as_deref() == Some(stdout_sha256.as_str())
                && probe.stderr_sha256.as_deref() == Some(stderr_sha256.as_str()),
            "actual persisted stream digest custody"
        );
    }
    let pids = std::fs::read_to_string(root.join("provider-pids.log"))?;
    ensure!(!pids.trim().is_empty(), "actual provider and descendant PID observation");
    eprintln!("QA_DESCENDANT_DEADLINE_CONTROLS_PASS: actual provider ready, timeout flags, complete partial streams and digests; elapsed={elapsed:?}; provider_pids={pids:?}");
    // Two one-second probes have ample scheduling/drain tolerance below two inherited four-second waits.
    ensure!(
        elapsed < Duration::from_secs(6),
        "probe deadline waited for inherited descendant pipes: {elapsed:?}"
    );
    Ok(true)
}

pub(super) fn require_descendant_stream_deadline() -> Result<()> {
    let temporary = bijux_dna_testkit::tempdir_for("qa-descendant-stream-deadline");
    let root = temporary.path().canonicalize()?;
    let executable = std::env::current_exe()?.canonicalize()?;
    let digest = bijux_dna_infra::hash_file_sha256(&executable)?;
    let mut command = Command::new(&executable);
    command
        .args([CASE, "--exact", "--nocapture"])
        .env(ROOT, &root)
        .env("BIJUX_QA_DESCENDANT_PIDS", root.join("provider-pids.log"))
        .env("HOME", &root)
        .env("TEST_TMP_DIR", &root)
        .env("TMPDIR", &root)
        .env("TMP", &root)
        .env("TEMP", &root);
    let result = bijux_dna_process::capture(
        &mut command,
        bijux_dna_process::CaptureLimits {
            timeout: Duration::from_secs(12),
            max_output_bytes: 65_536,
        },
        "owned descendant-stream deadline fixture",
    )?;
    ensure!(
        !result.timed_out && !result.output_limit_exceeded,
        "outer fixture transport deadline: {result:?}"
    );
    ensure!(bijux_dna_infra::hash_file_sha256(&executable)? == digest, "test executable changed");
    ensure!(
        result.status.success(),
        "descendant deadline child failed: stdout={} stderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    ensure!(
        String::from_utf8_lossy(&result.stdout).contains("running 1 test"),
        "exact child test executes"
    );
    Ok(())
}
