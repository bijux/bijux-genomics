//! Direct child log ownership and bounded observation of the initial structured record.

use std::fs;
use std::path::Path;
use std::process::Output;
use std::time::Duration;

use anyhow::{ensure, Context, Result};

pub(super) fn run_with_logs(
    command: &mut std::process::Command,
    directory: &Path,
    label: &str,
) -> Result<Output> {
    let stdout_path = directory.join(format!("{label}.stdout.log"));
    let stderr_path = directory.join(format!("{label}.stderr.log"));
    let stdout = bijux_dna_infra::create_file(&stdout_path)
        .with_context(|| format!("create {}", stdout_path.display()))?;
    let stderr = bijux_dna_infra::create_file(&stderr_path)
        .with_context(|| format!("create {}", stderr_path.display()))?;
    let status =
        command.stdin(std::process::Stdio::null()).stdout(stdout).stderr(stderr).spawn()?.wait()?;
    Ok(Output {
        status,
        stdout: fs::read(&stdout_path)
            .with_context(|| format!("read {}", stdout_path.display()))?,
        stderr: fs::read(&stderr_path)
            .with_context(|| format!("read {}", stderr_path.display()))?,
    })
}

/// Inspect only a bounded, complete first record through a separate read handle.
/// The child retains its direct file descriptors if this coordinator is interrupted.
pub(super) fn run_with_start_record(
    command: &mut std::process::Command,
    directory: &Path,
    label: &str,
    mut observe: impl FnMut(&[u8]) -> Result<()>,
) -> Result<Output> {
    use std::io::Read;

    const MAX_START_RECORD_BYTES: usize = 16 * 1024;
    let stdout_path = directory.join(format!("{label}.stdout.log"));
    let stderr_path = directory.join(format!("{label}.stderr.log"));
    let stdout = bijux_dna_infra::create_file(&stdout_path)?;
    let stderr = bijux_dna_infra::create_file(&stderr_path)?;
    // Open before spawn: neither observing nor waiting ever recreates/truncates a live log.
    // A separate open description also leaves the child's write offset untouched.
    let mut reader = fs::File::open(&stdout_path)?;
    let mut child =
        command.stdin(std::process::Stdio::null()).stdout(stdout).stderr(stderr).spawn()?;
    let observed = (|| -> Result<()> {
        let mut first = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let count = reader.read(&mut buffer)?;
            if count > 0 {
                if let Some(end) = buffer[..count].iter().position(|byte| *byte == b'\n') {
                    ensure!(
                        first.len() + end < MAX_START_RECORD_BYTES,
                        "native start record is too large"
                    );
                    first.extend_from_slice(&buffer[..=end]);
                    return std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        observe(&first)
                    }))
                    .unwrap_or_else(|_| Err(anyhow::anyhow!("native start observer panicked")));
                }
                ensure!(
                    first.len() + count < MAX_START_RECORD_BYTES,
                    "native start record is too large"
                );
                first.extend_from_slice(&buffer[..count]);
            } else if child.try_wait()?.is_some() {
                // An empty/partial terminal stream cannot authorize a phase transition.
                return Ok(());
            } else {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    })();
    // Even an invalid record, observer panic, read error or early exit retains child custody.
    let status = child.wait();
    observed?;
    Ok(Output { status: status?, stdout: fs::read(stdout_path)?, stderr: fs::read(stderr_path)? })
}

#[cfg(test)]
mod tests {
    #[test]
    fn startup_observation_preserves_direct_logs_and_reaps_after_observer_errors(
    ) -> anyhow::Result<()> {
        let owner = tempfile::tempdir()?;
        let logs = owner.path().join("artifacts/logs");
        std::fs::create_dir_all(&logs)?;
        for mode in ["valid", "invalid", "panic"] {
            let release = logs.join(format!("{mode}.release"));
            let done = logs.join(format!("{mode}.done"));
            let stdout = logs.join(format!("{mode}.stdout.log"));
            let mut command = std::process::Command::new("sh");
            command.args(["-c", "printf 'start\\n'; attempts=0; while ! test -f \"$RELEASE\"; do attempts=$((attempts+1)); test \"$attempts\" -lt 1000 || exit 91; sleep 0.01; done; printf 'tail\\n'; printf diagnostic >&2; printf done > \"$DONE\""])
                .env("RELEASE", &release).env("DONE", &done);
            let observed = std::cell::Cell::new(false);
            let result = super::run_with_start_record(&mut command, &logs, mode, |line| {
                assert_eq!(line, b"start\n");
                assert_eq!(std::fs::read(&stdout)?, b"start\n");
                assert!(!done.exists(), "observer ran only after child completion");
                observed.set(true);
                // Release the owned child even when the following observer deliberately fails.
                std::fs::write(&release, b"release")?;
                match mode {
                    "invalid" => anyhow::bail!("invalid startup record"),
                    "panic" => panic!("startup observer failed"),
                    _ => Ok(()),
                }
            });
            assert!(observed.get());
            assert_eq!(result.is_ok(), mode == "valid");
            assert_eq!(std::fs::read(&done)?, b"done");
            assert_eq!(std::fs::read(stdout)?, b"start\ntail\n");
            assert_eq!(std::fs::read(logs.join(format!("{mode}.stderr.log")))?, b"diagnostic");
        }
        for (label, script, expected) in
            [("empty", "exit 0", ""), ("partial", "printf partial", "partial")]
        {
            let mut command = std::process::Command::new("sh");
            command.args(["-c", script]);
            let output = super::run_with_start_record(&mut command, &logs, label, |_| {
                panic!("incomplete record authorized a reservation transition")
            })?;
            assert!(output.status.success());
            assert_eq!(output.stdout, expected.as_bytes());
        }
        Ok(())
    }
}
