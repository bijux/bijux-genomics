#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn repo_process_lock_returns_contention_without_polling() -> anyhow::Result<()> {
    let name = format!("repo-process-lock-contention-{}", std::process::id());
    let repo_root = support::repo_root()?;
    let lock_path = repo_root.join("artifacts/test-locks").join(format!("{name}.lock"));
    let lock = support::RepoProcessLock::acquire(&name)?;

    let Err(error) = support::RepoProcessLock::acquire(&name) else {
        anyhow::bail!("a concurrently held repository lock must report contention");
    };
    assert!(error.to_string().contains("already held"));
    assert!(lock_path.is_file(), "the OS lock anchor must be a regular file");
    drop(lock);
    Ok(())
}

#[test]
fn repo_process_lock_releases_on_drop() -> anyhow::Result<()> {
    let name = format!("repo-process-lock-release-{}", std::process::id());
    let lock = support::RepoProcessLock::acquire(&name)?;
    drop(lock);
    let replacement = support::RepoProcessLock::acquire(&name)?;
    drop(replacement);
    Ok(())
}

#[test]
fn repository_lock_implementations_have_no_polling() -> anyhow::Result<()> {
    let repo_root = support::repo_root()?;
    for relative in [
        "crates/bijux-dna-api/tests/support/workspace_paths.rs",
        "crates/bijux-dna-science/tests/fastq_environment_slice.rs",
        "crates/bijux-dna/tests/contracts/banks/bank_fixtures.rs",
        "crates/bijux-dna/src/commands/benchmark/repo_locking.rs",
    ] {
        let source = std::fs::read_to_string(repo_root.join(relative))?;
        for forbidden in ["TEST_LOCK_POLL_INTERVAL", "thread::sleep", "owner.pid"] {
            assert!(!source.contains(forbidden), "{relative} retains `{forbidden}` polling state");
        }
    }
    Ok(())
}
