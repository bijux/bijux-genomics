#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn repo_root_honors_valid_shard_workspace_override() -> anyhow::Result<()> {
    let _cwd_guard =
        support::CWD_LOCK.lock().map_err(|error| anyhow::anyhow!("cwd lock: {error}"))?;
    let _env_guard = support::EnvGuard::new()?;
    let shard_workspace = tempfile::tempdir()?;
    std::fs::write(shard_workspace.path().join("Cargo.lock"), "")?;
    std::fs::create_dir(shard_workspace.path().join("crates"))?;
    std::fs::create_dir(shard_workspace.path().join("configs"))?;
    std::env::set_var("BIJUX_REPO_ROOT", shard_workspace.path());

    assert_eq!(support::repo_root()?, shard_workspace.path());
    Ok(())
}

#[test]
fn repo_process_lock_returns_contention_without_polling() -> anyhow::Result<()> {
    let _cwd_guard =
        support::CWD_LOCK.lock().map_err(|error| anyhow::anyhow!("cwd lock: {error}"))?;
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
    let _cwd_guard =
        support::CWD_LOCK.lock().map_err(|error| anyhow::anyhow!("cwd lock: {error}"))?;
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
    let _cwd_guard =
        support::CWD_LOCK.lock().map_err(|error| anyhow::anyhow!("cwd lock: {error}"))?;
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
    let _cwd_guard =
        support::CWD_LOCK.lock().map_err(|error| anyhow::anyhow!("cwd lock: {error}"))?;
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

#[test]
fn nextest_full_profile_serializes_every_declared_benchmark_repository_mutator(
) -> anyhow::Result<()> {
    let _cwd_guard =
        support::CWD_LOCK.lock().map_err(|error| anyhow::anyhow!("cwd lock: {error}"))?;
    let repo_root = support::repo_root()?;
    let config = std::fs::read_to_string(repo_root.join("configs/rust/nextest.toml"))?;
    assert!(config.contains("[test-groups.benchmark-repository-mutators]"));
    assert!(!config.contains("[test-groups.micro-benchmark-mutators]"));
    assert!(!config.contains("[test-groups.vcf-benchmark-mutators]"));

    let integration_root = repo_root.join("crates/bijux-dna/tests");
    for entry in walkdir::WalkDir::new(&integration_root)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension == "rs"))
    {
        let source = std::fs::read_to_string(entry.path())?;
        if source.contains("RepoProcessLock::acquire(\"micro-benchmark-mutators\")")
            || source.contains("RepoProcessLock::acquire(\"benchmark-readiness-mutators\")")
        {
            let binary =
                entry.path().file_stem().and_then(|value| value.to_str()).ok_or_else(|| {
                    anyhow::anyhow!("integration test path has no UTF-8 file stem")
                })?;
            assert!(
                config.contains(binary),
                "configs/rust/nextest.toml does not assign `{binary}` to the benchmark repository mutator group"
            );
        }
    }

    let unit_root = repo_root.join("crates/bijux-dna/src/commands/benchmark");
    for entry in walkdir::WalkDir::new(&unit_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension == "rs"))
    {
        let source = std::fs::read_to_string(entry.path())?;
        if source.contains("RepoProcessLock::acquire")
            || source.contains("benchmark_readiness_test_lock(&root)")
        {
            let module = entry
                .path()
                .strip_prefix(&unit_root)?
                .with_extension("")
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "::");
            let module_name =
                entry.path().file_stem().and_then(|value| value.to_str()).ok_or_else(|| {
                    anyhow::anyhow!("unit test source path has no UTF-8 file stem")
                })?;
            assert!(
                config.contains(module_name),
                "configs/rust/nextest.toml does not assign `{module}` unit tests to the benchmark repository mutator group"
            );
        }
    }

    for module in ["local_vcf_demography_smoke", "local_vcf_ibd_smoke"] {
        assert!(
            config.contains(module),
            "configs/rust/nextest.toml does not assign `{module}` output-locking tests to the benchmark repository mutator group"
        );
    }
    Ok(())
}
