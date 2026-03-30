use super::examples::examples_run;
use super::*;

pub(super) fn hpc_validate_frontend_constraints(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- hpc run validate-frontend-constraints -- [--dry-run|--confirm]",
        );
    }
    let mut dry_run = true;
    for arg in args {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--confirm" => dry_run = false,
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    if dry_run {
        return success_line("[dry-run] validate-frontend-constraints (pass --confirm to execute)");
    }
    let policy_path = PathBuf::from(std::env::var("POLICY_TOML").unwrap_or_else(|_| {
        workspace
            .path("configs/ci/tools/hpc_frontend_build_policy.toml")
            .display()
            .to_string()
    }));
    let min_tmp_gb = std::env::var("MIN_TMP_GB")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(4);
    let min_work_gb = std::env::var("MIN_WORK_GB")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(10);
    let work_dir = std::env::var("WORK_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("ISO_ROOT").map_or_else(|_| workspace.path("artifacts"), PathBuf::from)
        });
    let policy: TomlValue = toml::from_str(&read_utf8(&policy_path)?)?;
    let host = hostname(workspace)?;
    if host_matches_policy(
        &host,
        policy
            .get("compute_hostname_regex")
            .and_then(TomlValue::as_str)
            .unwrap_or_default(),
    )? {
        if std::env::var("CI").is_ok()
            || std::env::var("REQUIRE_FRONTEND").ok().as_deref() == Some("1")
        {
            return Ok(OpsCommandOutcome::failure(format!(
                "frontend constraints: refusing compute host '{host}'\n"
            )));
        }
        return success_line(format!("frontend constraints: SKIP (compute host {host})"));
    }
    let frontend_pattern = policy
        .get("frontend_hostname_regex")
        .and_then(TomlValue::as_str)
        .unwrap_or_default();
    if !frontend_pattern.is_empty() && !host_matches_policy(&host, frontend_pattern)? {
        if std::env::var("CI").is_ok()
            || std::env::var("REQUIRE_FRONTEND").ok().as_deref() == Some("1")
        {
            return Ok(OpsCommandOutcome::failure(format!(
                "frontend constraints: host '{host}' does not match frontend pattern\n"
            )));
        }
        return success_line(format!(
            "frontend constraints: SKIP (host {host} not frontend)"
        ));
    }
    let tmp_gb = free_space_gb(Path::new("/tmp"))?;
    let work_gb = free_space_gb(&work_dir)?;
    if tmp_gb < min_tmp_gb {
        return Ok(OpsCommandOutcome::failure(format!(
            "frontend constraints: /tmp free {tmp_gb}GB < required {min_tmp_gb}GB\n"
        )));
    }
    if work_gb < min_work_gb {
        return Ok(OpsCommandOutcome::failure(format!(
            "frontend constraints: work dir free {work_gb}GB < required {min_work_gb}GB ({})\n",
            work_dir.display()
        )));
    }
    let test_dir = work_dir.join(format!("hpc-frontend-constraints.{}", std::process::id()));
    bijux_dna_infra::ensure_dir(&test_dir)?;
    bijux_dna_infra::write_bytes(test_dir.join(".write_test"), [])?;
    bijux_dna_infra::remove_file(&test_dir.join(".write_test"))?;
    fs::remove_dir(&test_dir)?;
    let module_state = if command_exists(workspace, "module")? {
        let output = run_program(workspace, "module", &["avail".to_string()])?;
        if !output.is_success() {
            return Ok(OpsCommandOutcome::failure(
                "frontend constraints: module command exists but module avail failed\n",
            ));
        }
        "available"
    } else {
        "not_used"
    };
    success_line(format!(
        "frontend constraints: OK (tmp={tmp_gb}GB work={work_gb}GB modules={module_state})"
    ))
}

pub(super) fn hpc_run_frontend_mini_e2e(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- hpc run run-frontend-mini-e2e -- [--dry-run|--confirm]",
        );
    }
    let mut dry_run = true;
    for arg in args {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--confirm" => dry_run = false,
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    if dry_run {
        return success_line("[dry-run] run-frontend-mini-e2e (pass --confirm to execute)");
    }
    let validation = hpc_validate_frontend_constraints(workspace, &["--confirm".to_string()])?;
    if !validation.is_success() {
        return Ok(validation);
    }
    let run_id = std::env::var("ISO_RUN_ID")
        .unwrap_or_else(|_| Utc::now().format("%Y%m%dT%H%M%SZ").to_string());
    let out_dir = std::env::var("OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            artifact_root_path(workspace)
                .unwrap_or_else(|_| workspace.path("artifacts"))
                .join("hpc/frontend-mini-e2e")
                .join(&run_id)
        });
    bijux_dna_infra::ensure_dir(&out_dir)?;
    let mut status = 0;
    for (example_id, label) in [
        ("vcf_downstream_vcf_full_mini", "vcf"),
        ("fastq_edna_mini", "fastq"),
    ] {
        let example_out = out_dir.join(label);
        bijux_dna_infra::ensure_dir(&example_out)?;
        let start = Utc::now();
        let outcome = examples_run(
            workspace,
            &["--allow-non-isolate".to_string(), example_id.to_string()],
        )?;
        write_utf8(&example_out.join("runner.stdout.log"), &outcome.stdout)?;
        write_utf8(&example_out.join("runner.stderr.log"), &outcome.stderr)?;
        if !outcome.is_success() {
            status = 1;
        }
        let src = artifact_root_path(workspace)?
            .join("examples")
            .join(example_id);
        for name in [
            "plan.json",
            "explain.json",
            "report.json",
            "run_report.json",
            "metrics.json",
            "logs.txt",
        ] {
            let path = src.join(name);
            if path.exists() {
                let _ = fs::copy(&path, example_out.join(name));
            }
        }
        let domain_hash = sha256_hex(&workspace.path(&format!("domain/{label}/index.yaml")))?;
        let example_toml = find_example_dir(workspace, example_id)?
            .context("resolve example dir")?
            .join("example.toml");
        let config_hash = sha256_hex(&example_toml)?;
        let lock_hash = sha256_hex(&workspace.path("containers/versions/lock.json"))?;
        write_json_pretty(
            &example_out.join("frontend_run_meta.json"),
            &json!({
                "schema_version": "bijux.frontend.mini.e2e.v1",
                "example_id": example_id,
                "label": label,
                "start_utc": start.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                "end_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                "exit_code": outcome.exit_code,
                "host": hostname(workspace)?,
                "tool_versions_ref": "artifacts/containers/hpc/frontend-smoke/summary.json",
                "container_lock_sha256": lock_hash,
                "domain_hash_sha256": domain_hash,
                "config_hash_sha256": config_hash,
            }),
        )?;
    }
    write_json_pretty(
        &out_dir.join("summary.json"),
        &json!({
            "schema_version": "bijux.frontend.mini.e2e.summary.v1",
            "run_id": run_id,
            "out_dir": out_dir.display().to_string(),
            "status": if status == 0 { "ok" } else { "fail" },
            "examples": [
                {"id": "vcf_downstream_vcf_full_mini", "artifact_dir": out_dir.join("vcf").display().to_string()},
                {"id": "fastq_edna_mini", "artifact_dir": out_dir.join("fastq").display().to_string()},
            ]
        }),
    )?;
    Ok(OpsCommandOutcome {
        exit_code: status,
        stdout: format!("{}\n", out_dir.join("summary.json").display()),
        stderr: String::new(),
    })
}

pub(super) fn hpc_benchmark_sync_pull(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- hpc run benchmark-sync-pull -- [--dry-run|--confirm] [--include-profile <name>] [--exclude-profile <name>]",
        );
    }
    let benchmark_workspace = load_benchmark_workspace_paths(workspace)?;
    validate_benchmark_sync_roots(&benchmark_workspace)?;
    let mut dry_run = true;
    let mut include_profile = benchmark_workspace
        .sync_default_include_profile
        .clone()
        .unwrap_or_else(|| "pull-results-default".to_string());
    let mut exclude_profile = benchmark_workspace
        .sync_default_exclude_profile
        .clone()
        .unwrap_or_else(|| "pull-full-default".to_string());
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            "--confirm" => {
                dry_run = false;
                index += 1;
            }
            "--include" | "--include-profile" => {
                include_profile = args
                    .get(index + 1)
                    .context("missing value for include profile")?
                    .clone();
                index += 2;
            }
            "--exclude" | "--exclude-profile" => {
                exclude_profile = args
                    .get(index + 1)
                    .context("missing value for exclude profile")?
                    .clone();
                index += 2;
            }
            other if other.starts_with("--include=") || other.starts_with("--include-profile=") => {
                include_profile = other.split('=').nth(1).unwrap_or_default().to_string();
                index += 1;
            }
            other if other.starts_with("--exclude=") || other.starts_with("--exclude-profile=") => {
                exclude_profile = other.split('=').nth(1).unwrap_or_default().to_string();
                index += 1;
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    let benchmark_host = env_or_contract(
        "BENCHMARK_SYNC_HOST",
        benchmark_workspace.remote_ssh_host.as_deref(),
        "workspace.remote.ssh_host",
    )?;
    let benchmark_repo_dir = env_or_contract(
        "BENCHMARK_SYNC_REPO_ROOT",
        benchmark_workspace.remote_repo_root.as_deref(),
        "workspace.remote.repo_root",
    )?;
    let benchmark_workspace_root = Path::new(&benchmark_repo_dir)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.display().to_string())
        .ok_or_else(|| anyhow!("benchmark sync repo root must have a parent directory"))?;
    let default_pull_base = benchmark_workspace
        .sync_default_pull_base
        .clone()
        .or_else(|| benchmark_workspace.local_results_root.clone());
    let pull_base = env_or_contract(
        "BENCHMARK_SYNC_PULL_BASE",
        default_pull_base.as_deref(),
        "workspace.sync.defaults.pull_base or workspace.local.results_root",
    )?;
    let pull_dest = env_or_default("BENCHMARK_SYNC_PULL_DEST", "");
    let default_pull_mode = benchmark_workspace
        .sync_default_pull_mode
        .as_deref()
        .unwrap_or("results");
    let pull_mode = env_or_default("BENCHMARK_SYNC_MODE", default_pull_mode);
    let default_results_root = benchmark_workspace.remote_results_root.clone();
    let benchmark_results_root = env_or_contract(
        "BENCHMARK_SYNC_RESULTS_ROOT",
        default_results_root.as_deref(),
        "workspace.remote.results_root",
    )?;
    let default_containers_root = benchmark_workspace.remote_containers_root.clone();
    let benchmark_containers_root = env_or_contract(
        "BENCHMARK_SYNC_CONTAINERS_ROOT",
        default_containers_root.as_deref(),
        "workspace.remote.containers_root",
    )?;
    let default_corpus_root = benchmark_workspace.remote_corpus_root.clone();
    let benchmark_corpus_root = env_or_contract(
        "BENCHMARK_SYNC_CORPUS_ROOT",
        default_corpus_root.as_deref(),
        "workspace.remote.corpus_root",
    )?;
    let include_containers_manifest_default = if benchmark_workspace
        .sync_default_include_containers_manifest
        .unwrap_or(false)
    {
        "1"
    } else {
        "0"
    };
    let include_containers_manifest = env_or_default(
        "BENCHMARK_SYNC_INCLUDE_CONTAINERS_MANIFEST",
        include_containers_manifest_default,
    ) == "1";
    let data_manifest_glob = env_or_default(
        "BENCHMARK_SYNC_DATA_MANIFEST_GLOB",
        benchmark_workspace
            .sync_default_data_manifest_glob
            .as_deref()
            .unwrap_or(""),
    );
    let profiles_cfg = workspace.path("configs/hpc/benchmark_sync_profiles.toml");
    let mut pull_full_exclude = workspace.path("configs/hpc/rsync/pull-full-excludes.txt");
    let mut pull_results_include = workspace.path("configs/hpc/rsync/pull-results-includes.txt");
    let sync_profiles = load_benchmark_sync_profiles(&profiles_cfg)?;
    let include_sync_profile = benchmark_sync_profile(&sync_profiles, &include_profile);
    let exclude_sync_profile = benchmark_sync_profile(&sync_profiles, &exclude_profile);
    if let Some(rel) = exclude_sync_profile.and_then(|profile| profile.exclude_file.as_deref()) {
        pull_full_exclude = workspace.path(rel);
    }
    if let Some(rel) = include_sync_profile.and_then(|profile| profile.include_file.as_deref()) {
        pull_results_include = workspace.path(rel);
    }
    let effective_data_manifest_glob = if data_manifest_glob.trim().is_empty() {
        include_sync_profile
            .map(|profile| profile.data_manifest_globs.join(","))
            .unwrap_or_default()
    } else {
        data_manifest_glob.clone()
    };
    let home = env_or_default("HOME", "");
    let use_governed_results_root = pull_mode == "results"
        && pull_dest.is_empty()
        && benchmark_workspace.local_results_root.is_some();
    let configured_pull_destination = include_sync_profile
        .and_then(|profile| profile.pull_destination.as_deref())
        .and_then(|key| benchmark_workspace_lookup(&benchmark_workspace, key));
    let dest = default_pull_destination(
        &pull_dest,
        configured_pull_destination,
        &pull_base,
        &home,
        use_governed_results_root,
    );
    let layout_conflicts =
        remote_layout_conflicts(workspace, &benchmark_host, &benchmark_workspace)?;
    if !layout_conflicts.is_empty() {
        return Ok(OpsCommandOutcome::failure(format!(
            "refusing pull: remote benchmark layout is ambiguous\n{}\n",
            layout_conflicts.join("\n")
        )));
    }
    if dry_run {
        return success_line(format!(
            "[dry-run] would pull mode={pull_mode} from {benchmark_host} to {}",
            dest.display()
        ));
    }
    if !use_governed_results_root && pull_dest.is_empty() && dest.exists() {
        return Ok(OpsCommandOutcome::failure(format!(
            "refusing pull: destination already exists: {}\n",
            dest.display()
        )));
    }
    bijux_dna_infra::ensure_dir(&dest)?;
    let mut pulled_paths = Vec::new();
    let mut pulled_path_mappings = Vec::new();
    if pull_mode == "full" {
        let outcome = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                format!("--exclude-from={}", pull_full_exclude.display()),
                format!("{benchmark_host}:{benchmark_workspace_root}/"),
                format!("{}/", dest.display()),
            ],
        )?;
        if !outcome.is_success() {
            return Ok(outcome);
        }
        pulled_paths.push(format!("{benchmark_workspace_root}/"));
        pulled_path_mappings.push(json!({
            "remote_path": format!("{benchmark_workspace_root}/"),
            "local_path": format!("{}/", dest.display()),
        }));
    } else if benchmark_workspace.remote_results_root.is_some() {
        let local_path =
            pull_benchmark_sync_tree(workspace, &benchmark_host, &benchmark_results_root, &dest)?;
        pulled_paths.push(format!("{benchmark_results_root}/"));
        pulled_path_mappings.push(json!({
            "remote_path": format!("{benchmark_results_root}/"),
            "local_path": format!("{}/", local_path.display()),
        }));
        if include_containers_manifest {
            let manifest_root = format!("{benchmark_containers_root}/manifest");
            if remote_path_exists(workspace, &benchmark_host, &manifest_root)? {
                let local_path =
                    pull_benchmark_sync_tree(workspace, &benchmark_host, &manifest_root, &dest)?;
                pulled_paths.push(format!("{manifest_root}/"));
                pulled_path_mappings.push(json!({
                    "remote_path": format!("{manifest_root}/"),
                    "local_path": format!("{}/", local_path.display()),
                }));
            }
        }
        if !effective_data_manifest_glob.is_empty() {
            for rel in effective_data_manifest_glob
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                let clean_rel = rel.trim_start_matches('/');
                let remote_path = format!("{benchmark_corpus_root}/{clean_rel}");
                let local_path =
                    pull_benchmark_sync_path(workspace, &benchmark_host, &remote_path, &dest)?;
                pulled_paths.push(remote_path);
                pulled_path_mappings.push(json!({
                    "remote_path": format!("{benchmark_corpus_root}/{clean_rel}"),
                    "local_path": local_path.display().to_string(),
                }));
            }
        }
    } else {
        let outcome = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                format!("--include-from={}", pull_results_include.display()),
                format!("{benchmark_host}:{benchmark_workspace_root}/"),
                format!("{}/", dest.display()),
            ],
        )?;
        if !outcome.is_success() {
            return Ok(outcome);
        }
        pulled_paths.push(format!("{benchmark_results_root}/"));
        pulled_path_mappings.push(json!({
            "remote_path": format!("{benchmark_results_root}/"),
            "local_path": format!("{}/", dest.display()),
        }));
        if include_containers_manifest {
            bijux_dna_infra::ensure_dir(dest.join("bijux-dna-container"))?;
            let _ = run_program(
                workspace,
                "rsync",
                &[
                    "-az".to_string(),
                    format!("{benchmark_host}:{benchmark_containers_root}/manifest/"),
                    dest.join("bijux-dna-container/manifest")
                        .display()
                        .to_string(),
                ],
            )?;
            pulled_paths.push(format!("{benchmark_containers_root}/manifest/"));
            pulled_path_mappings.push(json!({
                "remote_path": format!("{benchmark_containers_root}/manifest/"),
                "local_path": dest.join("bijux-dna-container/manifest").display().to_string(),
            }));
        }
        if !effective_data_manifest_glob.is_empty() {
            for rel in effective_data_manifest_glob
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                let clean_rel = rel.trim_start_matches('/');
                let target = dest
                    .join(benchmark_corpus_dir_name(&benchmark_workspace))
                    .join(clean_rel);
                if let Some(parent) = target.parent() {
                    bijux_dna_infra::ensure_dir(parent)?;
                }
                let _ = run_program(
                    workspace,
                    "rsync",
                    &[
                        "-az".to_string(),
                        format!("{benchmark_host}:{benchmark_corpus_root}/{clean_rel}"),
                        target.display().to_string(),
                    ],
                )?;
                pulled_paths.push(format!("{benchmark_corpus_root}/{clean_rel}"));
                pulled_path_mappings.push(json!({
                    "remote_path": format!("{benchmark_corpus_root}/{clean_rel}"),
                    "local_path": target.display().to_string(),
                }));
            }
        }
    }
    let remote_commit = benchmark_sync_revision(workspace, &benchmark_host, &benchmark_repo_dir)?;
    let remote_hostname = trim_newline(
        &run_program(
            workspace,
            "ssh",
            &[
                benchmark_host.clone(),
                "hostname -f 2>/dev/null || hostname".to_string(),
            ],
        )?
        .stdout,
    );
    write_json_pretty(
        &dest.join("PULLED_FROM.json"),
        &json!({
            "schema_version": "bijux.benchmark.pull.v1",
            "remote_host": benchmark_host,
            "remote_hostname": remote_hostname,
            "remote_root": benchmark_workspace_root,
            "remote_repo": benchmark_repo_dir,
            "remote_cache_root": benchmark_workspace.remote_cache_root,
            "local_destination": dest.display().to_string(),
            "local_cache_mirror_root": benchmark_workspace.local_cache_mirror_root,
            "include_profile": include_profile,
            "exclude_profile": exclude_profile,
            "workspace_scope": include_sync_profile.and_then(|profile| profile.workspace_scope.clone()),
            "data_manifest_globs": effective_data_manifest_glob
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>(),
            "remote_commit": remote_commit,
            "pulled_at_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            "pull_mode": pull_mode,
            "paths": pulled_paths,
            "path_mappings": pulled_path_mappings,
        }),
    )?;
    success_line(format!("pulled_to={}", dest.display()))
}

pub(super) fn hpc_benchmark_sync_push(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
    {
        return success_line(
            "Usage: cargo run -p bijux-dna-dev -- hpc run benchmark-sync-push -- [--dry-run|--confirm] [--exclude-profile <name>]",
        );
    }
    let mut dry_run = true;
    let mut exclude_profile = "push-default".to_string();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            "--confirm" => {
                dry_run = false;
                index += 1;
            }
            "--exclude" | "--exclude-profile" => {
                exclude_profile = args
                    .get(index + 1)
                    .context("missing value for exclude profile")?
                    .clone();
                index += 2;
            }
            other if other.starts_with("--exclude=") || other.starts_with("--exclude-profile=") => {
                exclude_profile = other.split('=').nth(1).unwrap_or_default().to_string();
                index += 1;
            }
            other => return Err(anyhow!("unknown arg: {other}")),
        }
    }
    let benchmark_workspace = load_benchmark_workspace_paths(workspace)?;
    validate_benchmark_sync_roots(&benchmark_workspace)?;
    let profiles_cfg = workspace.path("configs/hpc/benchmark_sync_profiles.toml");
    let mut exclude_file = workspace.path("configs/hpc/rsync/push-excludes.txt");
    if profiles_cfg.is_file() {
        if let Some(rel) =
            benchmark_sync_profile_path(&profiles_cfg, &exclude_profile, "exclude_file")?
        {
            exclude_file = workspace.path(&rel);
        }
    }
    let benchmark_host = env_or_contract(
        "BENCHMARK_SYNC_HOST",
        benchmark_workspace.remote_ssh_host.as_deref(),
        "workspace.remote.ssh_host",
    )?;
    let benchmark_repo_dir = env_or_contract(
        "BENCHMARK_SYNC_REPO_ROOT",
        benchmark_workspace.remote_repo_root.as_deref(),
        "workspace.remote.repo_root",
    )?;
    let clean_context_default = if benchmark_workspace
        .sync_default_clean_context
        .unwrap_or(true)
    {
        "1"
    } else {
        "0"
    };
    let allow_dirty_default = if benchmark_workspace
        .sync_default_allow_dirty
        .unwrap_or(false)
    {
        "1"
    } else {
        "0"
    };
    let clean_context =
        env_or_default("BENCHMARK_SYNC_CLEAN_CONTEXT", clean_context_default) == "1";
    let allow_dirty = env_or_default("BENCHMARK_SYNC_ALLOW_DIRTY", allow_dirty_default) == "1";
    if !allow_dirty {
        let dirty = run_program(
            workspace,
            "git",
            &["status".to_string(), "--short".to_string()],
        )?;
        if !dirty.stdout.trim().is_empty() {
            return Ok(OpsCommandOutcome::failure(
                "refusing push: local git tree is dirty (set BENCHMARK_SYNC_ALLOW_DIRTY=1 to override)\n",
            ));
        }
    }
    if dry_run {
        return success_line(format!(
            "[dry-run] would sync repo to {benchmark_host}:{benchmark_repo_dir}"
        ));
    }
    let mkdir = run_program(
        workspace,
        "ssh",
        &[
            benchmark_host.clone(),
            format!("mkdir -p '{benchmark_repo_dir}'"),
        ],
    )?;
    if !mkdir.is_success() {
        return Ok(mkdir);
    }
    if clean_context {
        let temp_root = temp_subdir(workspace, "benchmark-sync-push")?;
        let files_from = temp_root.join("files.txt");
        let sync_source = temp_root.join("BENCHMARK_SYNC_SOURCE.json");
        let tracked = run_program(workspace, "git", &["ls-files".to_string()])?;
        if !tracked.is_success() {
            return Ok(tracked);
        }
        write_utf8(&files_from, &tracked.stdout)?;
        write_benchmark_sync_source(workspace, &sync_source)?;
        let sync = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                "--delete".to_string(),
                format!("--files-from={}", files_from.display()),
                "./".to_string(),
                format!("{benchmark_host}:{benchmark_repo_dir}/"),
            ],
        )?;
        if !sync.is_success() {
            return Ok(sync);
        }
        let sync_source_copy = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                sync_source.display().to_string(),
                format!("{benchmark_host}:{benchmark_repo_dir}/BENCHMARK_SYNC_SOURCE.json"),
            ],
        )?;
        if !sync_source_copy.is_success() {
            return Ok(sync_source_copy);
        }
    } else {
        let temp_root = temp_subdir(workspace, "benchmark-sync-push")?;
        let sync_source = temp_root.join("BENCHMARK_SYNC_SOURCE.json");
        write_benchmark_sync_source(workspace, &sync_source)?;
        let sync = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                "--delete".to_string(),
                format!("--exclude-from={}", exclude_file.display()),
                "./".to_string(),
                format!("{benchmark_host}:{benchmark_repo_dir}/"),
            ],
        )?;
        if !sync.is_success() {
            return Ok(sync);
        }
        let sync_source_copy = run_program(
            workspace,
            "rsync",
            &[
                "-az".to_string(),
                sync_source.display().to_string(),
                format!("{benchmark_host}:{benchmark_repo_dir}/BENCHMARK_SYNC_SOURCE.json"),
            ],
        )?;
        if !sync_source_copy.is_success() {
            return Ok(sync_source_copy);
        }
    }
    let remote_commit = benchmark_sync_revision(workspace, &benchmark_host, &benchmark_repo_dir)?;
    success_line(format!(
        "remote_repo={benchmark_repo_dir}\nremote_commit={remote_commit}"
    ))
}
