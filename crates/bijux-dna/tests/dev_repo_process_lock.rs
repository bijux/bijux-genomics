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
fn json_accessors_reject_missing_and_mistyped_fields() {
    let payload = serde_json::json!({
        "array": [],
        "object": {},
        "wrong_type": 1,
    });

    assert!(support::json_array(&payload, "array").is_ok());
    assert!(support::json_object(&payload, "object").is_ok());
    assert!(support::json_array(&payload, "missing").is_err());
    assert!(support::json_object(&payload, "wrong_type").is_err());
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

#[test]
fn repository_mutating_unit_tests_serialize_before_one_shot_process_lock() -> anyhow::Result<()> {
    let repo_root = support::repo_root()?;
    let source_root = repo_root.join("crates/bijux-dna/src/commands/benchmark");
    for entry in walkdir::WalkDir::new(&source_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension == "rs"))
    {
        let source = std::fs::read_to_string(entry.path())?;
        let takes_process_lock = source.contains("RepoProcessLock::acquire")
            || source.contains("benchmark_readiness_test_lock(&root)");
        if takes_process_lock {
            assert!(
                source.contains("acquire_cwd_lock"),
                "{} takes the one-shot repository process lock without first participating in the event-driven unit-test mutex",
                entry.path().strip_prefix(&repo_root)?.display()
            );
        }
    }
    Ok(())
}
