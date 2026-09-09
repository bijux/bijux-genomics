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
fn vcf_aggregate_readiness_writers_acquire_repository_lock_before_mutation() -> anyhow::Result<()> {
    let _cwd_guard =
        support::CWD_LOCK.lock().map_err(|error| anyhow::anyhow!("cwd lock: {error}"))?;
    let repo_root = support::repo_root()?;
    for (relative, mutation_marker) in [
        (
            "crates/bijux-dna/tests/contracts/benchmark/vcf_readiness/inventory/all_retained_tools_complete.rs",
            "EnvGuard::new()",
        ),
        (
            "crates/bijux-dna/tests/contracts/benchmark/vcf_readiness/inventory/all_retained_tools_complete_file.rs",
            "EnvGuard::new()",
        ),
        (
            "crates/bijux-dna/src/commands/benchmark/readiness/vcf_imputation_metrics_ready.rs",
            "build_vcf_imputation_metrics_ready_report(",
        ),
    ] {
        let source = std::fs::read_to_string(repo_root.join(relative))?;
        let lock_position = source
            .find("RepoProcessLock::acquire")
            .ok_or_else(|| anyhow::anyhow!("{relative} does not acquire the repository lock"))?;
        let mutation_position = source
            .rfind(mutation_marker)
            .ok_or_else(|| anyhow::anyhow!("{relative} no longer contains `{mutation_marker}`"))?;
        assert!(
            lock_position < mutation_position,
            "{relative} acquires the repository lock after starting governed output mutation"
        );
    }
    Ok(())
}

#[test]
fn nextest_full_profile_serializes_benchmark_product_writers() -> anyhow::Result<()> {
    let _cwd_guard =
        support::CWD_LOCK.lock().map_err(|error| anyhow::anyhow!("cwd lock: {error}"))?;
    let repo_root = support::repo_root()?;
    let config = std::fs::read_to_string(repo_root.join("configs/rust/nextest.toml"))?;
    assert!(config.contains("[test-groups.benchmark-repository-mutators]"));
    let group = config
        .split("[test-groups.benchmark-repository-mutators]\n")
        .nth(1)
        .and_then(|tail| tail.split("\n[").next())
        .ok_or_else(|| anyhow::anyhow!("benchmark repository mutator group is missing"))?;
    assert!(group.contains("max-threads = 1"));
    assert!(!config.contains("[test-groups.micro-benchmark-mutators]"));
    assert!(!config.contains("[test-groups.vcf-benchmark-mutators]"));
    assert!(config.contains(
        "binary(/^(?:bench_|benchmark_readiness$|config_|dev_crates_|fixtures_|plan_validate_)/)"
    ));
    assert!(config.contains("test(/^commands::benchmark::/)"));

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
                binary == "benchmark_readiness"
                    || ["bench_", "config_", "dev_crates_", "fixtures_", "plan_validate_"]
                        .iter()
                        .any(|prefix| binary.starts_with(prefix)),
                "repository-mutating integration binary `{binary}` escaped the full-profile prefix contract"
            );
        }
    }

    let unit_root = repo_root.join("crates/bijux-dna/src/commands/benchmark");
    let mut mutating_unit_count = 0;
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
            mutating_unit_count += 1;
        }
    }
    assert!(mutating_unit_count > 0, "benchmark unit mutator inventory became vacuous");
    Ok(())
}

#[test]
fn repository_sandbox_replaces_inherited_workspace_and_output_authorities() -> anyhow::Result<()> {
    let _cwd_guard =
        support::CWD_LOCK.lock().map_err(|error| anyhow::anyhow!("cwd lock: {error}"))?;
    let _restore = support::EnvGuard::new()?;
    let original = support::repo_root()?;
    for (key, value) in [
        ("BIJUX_REPO_ROOT", original.clone()),
        ("ARTIFACT_ROOT", original.join("artifacts")),
        ("ISO_ROOT", original.join("artifacts")),
    ] {
        std::env::set_var(key, value);
    }
    {
        let _lock = support::RepoProcessLock::acquire("sandbox-authority-contract")?;
        let _sandbox = support::EnvGuard::new()?;
        let root = support::repo_root()?;
        assert_ne!(root, original);
        assert_eq!(root, std::env::current_dir()?);
        assert_eq!(support::crate_root("bijux-dna")?, root.join("crates/bijux-dna"));
        for key in ["ARTIFACT_ROOT", "ISO_ROOT"] {
            assert_eq!(std::env::var_os(key), Some(root.join("artifacts").into_os_string()));
        }
        assert_eq!(std::env::var_os("GIT_WORK_TREE"), Some(root.into_os_string()));
    }
    assert_eq!(support::repo_root()?, original);
    assert_eq!(
        std::env::var_os("ARTIFACT_ROOT"),
        Some(original.join("artifacts").into_os_string())
    );
    Ok(())
}
