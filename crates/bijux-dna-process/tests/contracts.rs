#![cfg(unix)]
use bijux_dna_process::{capture, CaptureLimits};
use std::process::Command;
use std::time::{Duration, Instant};

fn probe(
    body: &str,
    budget: usize,
    timeout: Duration,
) -> anyhow::Result<bijux_dna_process::SupervisedOutput> {
    capture(
        Command::new("sh").args(["-c", body]),
        CaptureLimits { timeout, max_output_bytes: budget },
        "capture contract",
    )
}

#[test]
fn raw_streams_and_nonzero_exit_remain_separate() -> anyhow::Result<()> {
    let output = probe("printf '\\377out'; printf err >&2; exit 7", 20, Duration::from_secs(2))?;
    assert_eq!(output.stdout, b"\xffout");
    assert_eq!(output.stderr, b"err");
    assert_eq!(output.status.code(), Some(7));
    assert!(!output.timed_out && !output.output_limit_exceeded);
    Ok(())
}

#[test]
fn both_streams_consume_one_budget_and_exact_limit_is_allowed() -> anyhow::Result<()> {
    let exact = probe("printf 12345678; printf abcdefg >&2", 15, Duration::from_secs(2))?;
    assert!(exact.status.success());
    assert!(!exact.output_limit_exceeded);
    assert_eq!(exact.stdout.len() + exact.stderr.len(), 15);
    let over = probe("printf 12345678; printf abcdefgh >&2", 15, Duration::from_secs(2))?;
    assert!(over.output_limit_exceeded);
    assert_eq!(over.stdout.len() + over.stderr.len(), 15);
    Ok(())
}

#[test]
fn continuous_output_cancels_without_unbounded_collection() -> anyhow::Result<()> {
    let start = Instant::now();
    let output = probe("while :; do printf 0123456789; done", 1024, Duration::from_secs(2))?;
    assert!(output.output_limit_exceeded);
    assert!(!output.timed_out);
    assert_eq!(output.stdout.len() + output.stderr.len(), 1024);
    assert!(start.elapsed() < Duration::from_secs(3));
    Ok(())
}

#[test]
fn one_deadline_covers_parent_exit_and_descendant_pipes() -> anyhow::Result<()> {
    let start = Instant::now();
    let output = probe("printf ready; sleep 60 & exit 0", 1024, Duration::from_millis(100))?;
    assert!(output.timed_out);
    assert_eq!(output.stdout, b"ready");
    assert!(start.elapsed() < Duration::from_secs(3));
    Ok(())
}

#[test]
fn one_deadline_covers_closed_pipes_and_live_child() -> anyhow::Result<()> {
    let start = Instant::now();
    let output = probe("exec 1>&- 2>&-; sleep 60", 1024, Duration::from_millis(100))?;
    assert!(output.timed_out);
    assert!(start.elapsed() < Duration::from_secs(3));
    Ok(())
}

#[test]
fn invalid_deadline_rejects_before_launch() {
    let result = capture(
        &mut Command::new("unavailable-capture-contract-executable"),
        CaptureLimits { timeout: Duration::ZERO, max_output_bytes: 1024 },
        "invalid deadline",
    );
    assert!(result.is_err_and(|error| error.to_string().contains("deadline must be positive")));
}
