use super::{
    artifact_root_path, config_snapshot_inputs_changed, config_tree_snapshot_text,
    ensure_help_only, failure_lines, fs, generate_tool_index, merge_outcomes, read_utf8,
    resolve_optional_output_arg, resolve_workspace_path, run_check_ids, run_native_ops_command,
    run_program, sha256_hex, success_line, tooling_cargo_targets, tooling_ci_clippy_executors,
    tooling_ci_fmt, write_utf8, BTreeSet, Context, NativeOpsCommandKey, OpsCommandOutcome, Regex,
    Result, WalkDir, Workspace,
};

pub(in super::super) fn tooling_flake_hunt(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mut expr = None;
    let mut runs = 20usize;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--expr" => {
                expr = args.get(index + 1).cloned();
                index += 2;
            }
            "--runs" => {
                runs = args
                    .get(index + 1)
                    .context("missing value for --runs")?
                    .parse::<usize>()
                    .context("parse --runs")?;
                index += 2;
            }
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run flake-hunt -- --expr '<nextest-filter>' [--runs N]",
                )
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }
    let expr = expr.context("--expr is required")?;
    let log_dir = artifact_root_path(workspace)?.join("flake-hunt");
    bijux_dna_infra::ensure_dir(&log_dir)
        .with_context(|| format!("create {}", log_dir.display()))?;
    let mut stdout = String::new();
    let mut failures = 0usize;
    for run_index in 1..=runs {
        stdout.push_str(&format!("[{run_index}/{runs}] nextest run -E {expr}\n"));
        let outcome = tooling_cargo_targets(
            workspace,
            &[
                "nextest-run".to_string(),
                "--config-file".to_string(),
                "configs/rust/nextest.toml".to_string(),
                "--profile".to_string(),
                "flake".to_string(),
                "-E".to_string(),
                expr.clone(),
            ],
        )?;
        bijux_dna_infra::write_bytes(
            log_dir.join("last.log"),
            format!("{}{}", outcome.stdout, outcome.stderr),
        )
        .with_context(|| format!("write {}", log_dir.join("last.log").display()))?;
        if outcome.is_success() {
            stdout.push_str("  PASS\n");
        } else {
            failures += 1;
            stdout.push_str("  FAIL\n");
            stdout.push_str(&outcome.stdout);
            stdout.push_str(&outcome.stderr);
        }
    }
    stdout.push_str(&format!(
        "Expression: {expr}\nRuns: {runs}\nPassed: {}\nFailed: {failures}\n",
        runs - failures
    ));
    if failures == 0 {
        return Ok(OpsCommandOutcome::success(stdout));
    }
    Ok(OpsCommandOutcome::failure(stdout))
}

pub(in super::super) fn tooling_lint_fast(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("lint-fast", args)?;
    let base_ref = std::env::var("LINT_FAST_BASE_REF")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            let head_prev = run_program(
                workspace,
                "git",
                &["rev-parse".to_string(), "--verify".to_string(), "HEAD~1".to_string()],
            );
            match head_prev {
                Ok(outcome) if outcome.is_success() => "HEAD~1".to_string(),
                _ => "HEAD".to_string(),
            }
        });
    let diff = run_program(
        workspace,
        "git",
        &["diff".to_string(), "--name-only".to_string(), format!("{base_ref}..HEAD")],
    )?;
    let changed = diff.stdout.lines().filter(|line| !line.trim().is_empty()).collect::<Vec<_>>();
    let mut stdout = String::new();
    if changed.is_empty() {
        run_check_ids(&mut stdout, &["check-config-schema", "check-automation-interface"])?;
        stdout.push_str("lint-fast: no changed files; running config+automation lint baseline\n");
        return Ok(OpsCommandOutcome::success(stdout));
    }
    let mut need_fmt = false;
    let mut need_clippy = false;
    let mut need_docs = false;
    let mut need_configs = false;
    let mut need_automation = false;
    for file in &changed {
        if file.ends_with(".rs")
            || *file == "Cargo.toml"
            || *file == "Cargo.lock"
            || file.starts_with("crates/")
        {
            need_fmt = true;
            need_clippy = true;
        }
        if file.starts_with("docs/") || file.ends_with("README.md") {
            need_docs = true;
        }
        if file.starts_with("configs/") || file.starts_with("assets/reference/") {
            need_configs = true;
        }
        if file.starts_with("makes/") || *file == "Makefile" {
            need_automation = true;
        }
    }
    if need_fmt {
        stdout.push_str("lint-fast: running rustfmt\n");
        let outcome = tooling_ci_fmt(workspace, &[])?;
        if !outcome.is_success() {
            return Ok(merge_outcomes(OpsCommandOutcome::success(stdout), outcome));
        }
    }
    if need_clippy {
        stdout.push_str("lint-fast: running clippy for executor/runtime subset\n");
        let outcome = tooling_ci_clippy_executors(workspace, &[])?;
        if !outcome.is_success() {
            return Ok(merge_outcomes(OpsCommandOutcome::success(stdout), outcome));
        }
    }
    if need_docs {
        stdout.push_str("lint-fast: running docs checks\n");
        let docs_outcome =
            run_native_ops_command(NativeOpsCommandKey::DocsCheckDocLinks, workspace, &[])?;
        if !docs_outcome.is_success() {
            return Ok(merge_outcomes(OpsCommandOutcome::success(stdout), docs_outcome));
        }
        stdout.push_str(&docs_outcome.stdout);
        run_check_ids(&mut stdout, &["check-docs-build-contract"])?;
    }
    if need_configs {
        stdout.push_str("lint-fast: running config checks\n");
        run_check_ids(&mut stdout, &["check-config-schema", "check-config-layout"])?;
    }
    if need_automation {
        stdout.push_str("lint-fast: running automation interface checks\n");
        run_check_ids(
            &mut stdout,
            &[
                "check-automation-interface",
                "check-clippy-allowlist-growth",
                "check-rustflags-consistency",
            ],
        )?;
    }
    stdout.push_str("lint-fast: OK\n");
    Ok(OpsCommandOutcome::success(stdout))
}

pub(in super::super) fn tooling_generate_tool_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-tool-index",
        args,
        "docs/20-science/TOOL_INDEX.md",
    )?;
    generate_tool_index(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(in super::super) fn tooling_check_config_snapshot(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let only_if_changed = match args {
        [] => false,
        [flag] if flag == "--if-config-changed" => true,
        [flag] if flag == "--help" || flag == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- tooling run check-config-snapshot -- [--if-config-changed]",
            )
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run check-config-snapshot -- [--if-config-changed]\n",
            ))
        }
    };

    if only_if_changed && !config_snapshot_inputs_changed(workspace)? {
        return success_line("config snapshot: SKIP (no config or generator changes)");
    }

    let baseline = workspace.path("configs/schema/config_tree.snapshot");
    let actual = workspace.path("artifacts/tmp/config_tree.snapshot.actual");
    if let Some(parent) = actual.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    write_utf8(&actual, &config_tree_snapshot_text(workspace)?)?;

    if read_utf8(&baseline)? != read_utf8(&actual)? {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot drift detected; regenerate via cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot\n",
        ));
    }
    if !read_utf8(&baseline)?
        .lines()
        .any(|line| {
            line.trim()
                == "# generator = cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot"
        })
    {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot header missing generator contract\n",
        ));
    }
    let marker_file = workspace.path("artifacts/configs/config_tree_snapshot.marker");
    if marker_file.is_file() {
        let marker = read_utf8(&marker_file)?;
        let marker_sha = marker
            .lines()
            .find_map(|line| line.strip_prefix("snapshot_sha256="))
            .unwrap_or_default()
            .trim()
            .to_string();
        let actual_sha = sha256_hex(&baseline)?;
        if marker_sha.is_empty() || marker_sha != actual_sha {
            return Ok(OpsCommandOutcome::failure(
                "config snapshot marker is stale: run cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot\n",
            ));
        }
    }
    success_line("config snapshot: OK")
}

pub(in super::super) fn tooling_generate_config_tree_snapshot(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("generate-config-tree-snapshot", args)?;
    let out = workspace.path("configs/schema/config_tree.snapshot");
    let marker_dir = workspace.path("artifacts/configs");
    let marker_file = marker_dir.join("config_tree_snapshot.marker");
    bijux_dna_infra::ensure_dir(&marker_dir)
        .with_context(|| format!("create {}", marker_dir.display()))?;
    if let Some(parent) = out.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    write_utf8(&out, &config_tree_snapshot_text(workspace)?)?;
    write_utf8(
        &marker_file,
        &format!(
            "generator=cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot\nsnapshot_sha256={}\n",
            sha256_hex(&out)?
        ),
    )?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(in super::super) fn tooling_check_config_paths(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-config-paths", args)?;
    let pattern = Regex::new(r"(?:benchmarks/)?configs/[A-Za-z0-9_./-]+\.(toml|md|sha256)")?;
    let mut refs = BTreeSet::new();
    let mut scan_roots = vec![workspace.path("Makefile")];
    scan_roots.extend([
        workspace.path("makes"),
        workspace.path("crates"),
        workspace.path("docs"),
        workspace.path(".github"),
    ]);
    for root in scan_roots {
        if root.is_file() {
            let raw = read_utf8(&root).unwrap_or_default();
            for capture in pattern.find_iter(&raw) {
                refs.insert(
                    capture
                        .as_str()
                        .trim_end_matches(|ch: char| "`\"',;:)".contains(ch))
                        .to_string(),
                );
            }
            continue;
        }
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&root).into_iter().filter_map(std::result::Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let raw = read_utf8(entry.path()).unwrap_or_default();
            for capture in pattern.find_iter(&raw) {
                refs.insert(
                    capture
                        .as_str()
                        .trim_end_matches(|ch: char| "`\"',;:)".contains(ch))
                        .to_string(),
                );
            }
        }
    }
    let allow_missing = BTreeSet::from([
        "configs/runtime/profiles/hpc.toml",
        "configs/tools.toml",
        "configs/lab/config.toml",
    ]);
    let missing = refs
        .into_iter()
        .filter(|rel| !allow_missing.contains(rel.as_str()) && !workspace.path(rel).exists())
        .map(|rel| format!("missing config reference: {rel}"))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return success_line("config path references: OK");
    }
    failure_lines("config path references: FAILED", &missing)
}

pub(in super::super) fn tooling_clean_docs(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let docs_root = match args {
        [] => workspace.path("artifacts/docs"),
        [flag] if flag == "--help" || flag == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- tooling run clean-docs -- [artifacts/docs-root]",
            )
        }
        [path] => resolve_workspace_path(workspace, path),
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run clean-docs -- [artifacts/docs-root]\n",
            ))
        }
    };
    let docs_root_rel = workspace.rel(&docs_root).to_string_lossy().to_string();
    if !docs_root_rel.starts_with("artifacts/") {
        return Ok(OpsCommandOutcome::failure(
            "clean-docs refuses to remove paths outside artifacts/\n",
        ));
    }
    if docs_root.exists() {
        fs::remove_dir_all(&docs_root)
            .with_context(|| format!("remove {}", docs_root.display()))?;
    }
    success_line(format!("removed {}", docs_root.display()))
}
