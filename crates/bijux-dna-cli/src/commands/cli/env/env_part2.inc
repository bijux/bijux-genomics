/// # Errors
/// Returns an error if registry parsing, Apptainer build/smoke, or manifest writes fail.
#[allow(clippy::too_many_lines)]
pub fn ensure_apptainer_images(
    registry_path: &Path,
    hpc_root: &Path,
    domain: &str,
    stages_csv: &str,
    force_smoke: bool,
    repair_mismatch: bool,
) -> Result<EnsureImagesReport> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect::<std::collections::BTreeMap<_, _>>();
    let stages = parse_stage_registry_rows(&raw)?
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect::<std::collections::BTreeMap<_, _>>();
    let stage_ids = normalize_stage_ids(domain, stages_csv);

    let root = hpc_root.to_path_buf();
    let containers_root = root.join("bijux-dna-containers");
    let data_root = root.join("bijux-dna-data");
    let results_root = root.join("bijux-dna-results");
    bijux_dna_api::v1::api::run::ensure_dir(&containers_root)?;
    bijux_dna_api::v1::api::run::ensure_dir(&data_root)?;
    bijux_dna_api::v1::api::run::ensure_dir(&results_root)?;

    let mut reports = Vec::new();
    let mut built = 0usize;
    let mut reused = 0usize;
    let mut quick_smoked = 0usize;
    let mut failed = 0usize;
    let mut auto_demoted = Vec::new();

    for stage_id in &stage_ids {
        let Some(stage) = stages.get(stage_id) else {
            return Err(anyhow!("stage not found in registry: {stage_id}"));
        };
        let mut stage_tools = stage.primary_tools.clone();
        stage_tools.extend(stage.optional_alternatives.clone());
        stage_tools.extend(stage.validation_tools.clone());
        stage_tools.extend(stage.reporting_tools.clone());
        stage_tools.sort();
        stage_tools.dedup();

        for tool_id in stage_tools {
            let Some(tool) = tools.get(&tool_id) else {
                continue;
            };
            let has_apptainer = tool.runtimes.iter().any(|runtime| runtime == "apptainer");
            if !has_apptainer {
                continue;
            }
            let Some(def_rel) = tool.apptainer_def.as_deref() else {
                continue;
            };
            let expected_digest = expected_registry_digest(tool)
                .ok_or_else(|| anyhow!("tool {tool_id} is missing sha256 pin in registry"))?;
            let tool_dir = containers_root.join(&tool_id);
            bijux_dna_api::v1::api::run::ensure_dir(&tool_dir)?;
            let sif_path = tool_dir.join(format!("{expected_digest}.sif"));
            let smoke_manifest_path =
                tool_dir.join(format!("{expected_digest}.smoke_manifest.json"));
            let compat_smoke_manifest_path = tool_dir.join("smoke_manifest.json");
            let mut built_this = false;

            if sif_path.exists() {
                let actual = hash_file_sha256_hex(&sif_path)?;
                if actual != expected_digest {
                    if repair_mismatch {
                        quarantine_file(
                            &sif_path,
                            &containers_root.join("quarantine"),
                            "digest_mismatch",
                        )?;
                    } else {
                        return Err(anyhow!(
                            "digest mismatch for existing SIF {}; rerun with --repair-mismatch",
                            sif_path.display()
                        ));
                    }
                }
            }
            quarantine_unexpected_sifs(
                &tool_dir,
                &expected_digest,
                repair_mismatch,
                &containers_root,
            )?;
            if sif_path.exists() {
                reused += 1;
            } else {
                let def_path = std::env::current_dir()?.join(def_rel);
                build_apptainer_image(&def_path, &sif_path)?;
                built_this = true;
                built += 1;
            }

            let actual_digest = hash_file_sha256_hex(&sif_path)?;
            if actual_digest != expected_digest {
                return Err(anyhow!(
                    "digest mismatch for {tool_id}: expected {expected_digest}, got {actual_digest}"
                ));
            }

            let do_quick_smoke = force_smoke || should_run_weekly_quick_smoke(&smoke_manifest_path);
            let version_cmd = tool
                .smoke_version_cmd
                .clone()
                .or(tool.version_cmd.clone())
                .filter(|v| !v.trim().is_empty())
                .or_else(|| Some(format!("{tool_id} --version")))
                .unwrap_or_else(|| format!("{tool_id} --version"));
            let help_cmd = tool
                .smoke_help_cmd
                .clone()
                .or(tool.help_cmd.clone())
                .filter(|v| !v.trim().is_empty())
                .or_else(|| Some(format!("{tool_id} --help")))
                .unwrap_or_else(|| format!("{tool_id} --help"));
            let require_help = tool.smoke_require_help.unwrap_or(true);
            let mut status = "ok".to_string();
            if do_quick_smoke {
                quick_smoked += 1;
                let smoke = run_smoke_with_manifest(
                    &sif_path,
                    &tool_id,
                    stage_id,
                    &expected_digest,
                    &actual_digest,
                    &version_cmd,
                    &help_cmd,
                    require_help,
                    &data_root,
                    &results_root,
                )?;
                if smoke.status != "ok" {
                    status = "wrapper_failed_auto_demoted".to_string();
                    failed += 1;
                    auto_demoted.push(tool_id.clone());
                }
                bijux_dna_infra::atomic_write_json(&smoke_manifest_path, &smoke)
                    .with_context(|| format!("write {}", smoke_manifest_path.display()))?;
                bijux_dna_infra::atomic_write_json(&compat_smoke_manifest_path, &smoke)
                    .with_context(|| format!("write {}", compat_smoke_manifest_path.display()))?;
            }

            reports.push(EnsureToolReport {
                tool_id: tool_id.clone(),
                stage_id: stage_id.clone(),
                sif_path: sif_path.display().to_string(),
                expected_digest: expected_digest.clone(),
                actual_digest,
                built: built_this,
                quick_smoked: do_quick_smoke,
                status,
            });
        }
    }

    if !auto_demoted.is_empty() {
        let payload = serde_json::json!({
            "schema_version": "bijux.apptainer_auto_demote.v1",
            "tools": auto_demoted,
            "updated_at_unix_s": now_unix_s(),
            "reason": "help/version smoke failure",
        });
        let path = containers_root.join("auto_demoted_tools.json");
        bijux_dna_infra::atomic_write_json(&path, &payload)
            .with_context(|| format!("write {}", path.display()))?;
    }

    Ok(EnsureImagesReport {
        schema_version: "bijux.apptainer.ensure_images.v1",
        domain: domain.to_string(),
        stages: stage_ids,
        tools: reports,
        built,
        reused,
        quick_smoked,
        failed,
    })
}

fn quarantine_unexpected_sifs(
    tool_dir: &Path,
    expected_digest: &str,
    repair_mismatch: bool,
    containers_root: &Path,
) -> Result<()> {
    let mut offenders = Vec::new();
    for entry in
        std::fs::read_dir(tool_dir).with_context(|| format!("read {}", tool_dir.display()))?
    {
        let path = entry?.path();
        let is_sif = path
            .extension()
            .and_then(|v| v.to_str())
            .is_some_and(|v| v.eq_ignore_ascii_case("sif"));
        if !is_sif {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|v| v.to_str()) else {
            continue;
        };
        if stem != expected_digest {
            offenders.push(path);
        }
    }
    if offenders.is_empty() {
        return Ok(());
    }
    if !repair_mismatch {
        return Err(anyhow!(
            "unexpected SIF(s) for tool at {}. rerun with --repair-mismatch",
            tool_dir.display()
        ));
    }
    let quarantine_root = containers_root.join("quarantine");
    for path in offenders {
        quarantine_file(&path, &quarantine_root, "unexpected_digest")?;
    }
    Ok(())
}

fn quarantine_file(path: &Path, quarantine_root: &Path, reason: &str) -> Result<()> {
    bijux_dna_api::v1::api::run::ensure_dir(quarantine_root)?;
    let tool = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|v| v.to_str())
        .unwrap_or("unknown_tool");
    let dest_dir = quarantine_root.join(tool);
    bijux_dna_api::v1::api::run::ensure_dir(&dest_dir)?;
    let name = path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("artifact.sif");
    let dest = dest_dir.join(format!("{name}.{}.{}", reason, now_unix_s()));
    std::fs::rename(path, &dest)
        .with_context(|| format!("quarantine {} -> {}", path.display(), dest.display()))?;
    Ok(())
}

/// # Errors
/// Returns an error if stage id is malformed.
pub fn parse_stage_domain(stage: &str) -> Result<String> {
    let Some((domain, _)) = stage.split_once('.') else {
        return Err(anyhow!("stage must be fully qualified, got `{stage}`"));
    };
    if domain.is_empty() {
        return Err(anyhow!("stage must include domain prefix, got `{stage}`"));
    }
    Ok(domain.to_string())
}

/// # Errors
/// Returns an error if the containers inventory cannot be read.
pub fn sif_inventory(root: &Path) -> Result<SifInventoryReport> {
    let containers_dir = root.join("bijux-dna-containers");
    let mut entries = Vec::new();
    let mut stack = vec![containers_dir.clone()];
    while let Some(dir) = stack.pop() {
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let is_sif = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sif"));
            if !is_sif {
                continue;
            }
            let tool_id = path
                .parent()
                .and_then(Path::file_name)
                .and_then(|v| v.to_str())
                .unwrap_or("unknown")
                .to_string();
            let digest = hash_file_sha256_hex(&path)?;
            let stem = path
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or_default();
            let manifest = path
                .parent()
                .map(|p| p.join(format!("{stem}.smoke_manifest.json")));
            let smoke_raw = manifest
                .as_ref()
                .and_then(|p| std::fs::read_to_string(p).ok());
            let smoke_status = smoke_raw
                .as_deref()
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
                .and_then(|v| v.get("status").and_then(|s| s.as_str()).map(str::to_string));
            entries.push(SifInventoryEntry {
                tool_id,
                sif_path: path.display().to_string(),
                sha256: digest,
                smoke_manifest_path: manifest
                    .as_ref()
                    .filter(|p| p.exists())
                    .map(|p| p.display().to_string()),
                smoke_status,
            });
        }
    }
    entries.sort_by(|a, b| a.sif_path.cmp(&b.sif_path));
    Ok(SifInventoryReport {
        schema_version: "bijux.sif_inventory.v1",
        containers_dir: containers_dir.display().to_string(),
        entries,
    })
}

/// # Errors
/// Returns an error if the markdown QA matrix cannot be generated.
pub fn generate_apptainer_qa_matrix_markdown(root: &Path) -> Result<String> {
    let inventory = sif_inventory(root)?;
    let mut lines = vec![
        "# Apptainer QA Matrix".to_string(),
        String::new(),
        format!("Generated at unix_s={}", now_unix_s()),
        String::new(),
        "| Tool | Build OK | Smoke OK | Run OK | SIF |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ];
    for entry in inventory.entries {
        let smoke_ok = entry.smoke_status.as_deref().is_some_and(|v| v == "ok");
        let build_ok = true;
        let run_ok = smoke_ok;
        lines.push(format!(
            "| {} | {} | {} | {} | `{}` |",
            entry.tool_id,
            if build_ok { "yes" } else { "no" },
            if smoke_ok { "yes" } else { "no" },
            if run_ok { "yes" } else { "no" },
            entry.sif_path
        ));
    }
    lines.push(String::new());
    Ok(lines.join("\n"))
}

/// # Errors
/// Returns an error if registry cannot be read.
pub fn print_registry_list_tools(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let mut tools = parse_tools_registry_rows(&raw)?
        .into_iter()
        .filter(|row| row.status != "planned" && row.status != "out_of_scope")
        .map(|row| row.id)
        .collect::<Vec<_>>();
    tools.sort();
    tools.dedup();
    for tool in tools {
        println!("{tool}");
    }
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read.
pub fn print_registry_tools(registry_path: &Path, stage: Option<&str>, kind: &str) -> Result<()> {
    if let Some(stage) = stage {
        let tools = registry_tools_for_stage(registry_path, stage, kind)?;
        println!("{}", tools.join(","));
        return Ok(());
    }
    print_registry_list_tools(registry_path)
}

/// # Errors
/// Returns an error if registry cannot be read.
pub fn print_registry_list_stages(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let mut stages = parse_stage_registry_rows(&raw)?
        .into_iter()
        .map(|stage| stage.id)
        .collect::<Vec<_>>();
    stages.sort();
    for stage in stages {
        println!("{stage}");
    }
    Ok(())
}

/// # Errors
/// Returns an error if id is not found or registry cannot be parsed.
pub fn print_registry_show(registry_path: &Path, id: &str) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    if let Some(tool) = parse_tools_registry_rows(&raw)?
        .into_iter()
        .find(|tool| tool.id == id)
    {
        crate::commands::cli::render::json::print_pretty(&serde_json::json!({
            "id": tool.id,
            "version": tool.version,
            "upstream": tool.upstream,
            "runtimes": tool.runtimes,
            "dockerfile": tool.dockerfile,
            "apptainer_def": tool.apptainer_def,
            "version_cmd": tool.version_cmd,
            "help_cmd": tool.help_cmd,
            "healthcheck_cmd": tool.healthcheck_cmd,
            "expected_bin": tool.expected_bin,
            "expected_version_regex": tool.expected_version_regex,
            "pinned_commit": tool.pinned_commit,
        }))?;
        return Ok(());
    }
    if let Some(stage) = parse_stage_registry_rows(&raw)?
        .into_iter()
        .find(|stage| stage.id == id)
    {
        crate::commands::cli::render::json::print_pretty(&serde_json::json!({
            "id": stage.id,
            "primary_tools": stage.primary_tools,
            "optional_alternatives": stage.optional_alternatives,
            "validation_tools": stage.validation_tools,
            "reporting_tools": stage.reporting_tools,
        }))?;
        return Ok(());
    }
    Err(anyhow!("registry id not found: {id}"))
}

/// # Errors
/// Returns an error if id is not found or registry cannot be parsed.
pub fn print_registry_show_tool(registry_path: &Path, id: &str) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let Some(tool) = parse_tools_registry_rows(&raw)?
        .into_iter()
        .find(|tool| tool.id == id)
    else {
        return Err(anyhow!("tool not found in registry: {id}"));
    };
    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "id": tool.id,
        "version": tool.version,
        "upstream": tool.upstream,
        "runtimes": tool.runtimes,
        "dockerfile": tool.dockerfile,
        "apptainer_def": tool.apptainer_def,
        "version_cmd": tool.version_cmd,
        "help_cmd": tool.help_cmd,
        "healthcheck_cmd": tool.healthcheck_cmd,
        "expected_bin": tool.expected_bin,
        "expected_version_regex": tool.expected_version_regex,
        "pinned_commit": tool.pinned_commit,
    }))?;
    Ok(())
}

/// # Errors
/// Returns an error if tool cannot be resolved from registry.
pub fn verify_registry_tool(registry_path: &Path, id: &str) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let Some(tool) = parse_tools_registry_rows(&raw)?
        .into_iter()
        .find(|tool| tool.id == id)
    else {
        return Err(anyhow!("tool not found in registry: {id}"));
    };
    let pin = tool
        .pinned_commit
        .clone()
        .unwrap_or_else(|| "missing".to_string());
    let version_cmd = tool.version_cmd.clone().unwrap_or_default();
    let help_cmd = tool.help_cmd.clone().unwrap_or_default();
    let healthcheck_cmd = tool
        .healthcheck_cmd
        .clone()
        .unwrap_or_else(|| help_cmd.clone());
    let expected_version_regex = tool
        .expected_version_regex
        .clone()
        .unwrap_or_else(|| "v?[0-9]+\\.[0-9]+([.-][0-9A-Za-z]+)?".to_string());
    let version_output =
        run_shell_capture(&version_cmd).unwrap_or_else(|err| format!("error:{err}"));
    let help_output = run_shell_capture(&help_cmd).unwrap_or_else(|err| format!("error:{err}"));
    let health_output =
        run_shell_capture(&healthcheck_cmd).unwrap_or_else(|err| format!("error:{err}"));
    let version_matches_regex = Regex::new(&expected_version_regex)
        .ok()
        .is_some_and(|regex| regex.is_match(&version_output));
    let parsed_version =
        parse_first_version(&version_output).unwrap_or_else(|| "unknown".to_string());

    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "tool_id": tool.id,
        "pin": pin,
        "entrypoint": tool.expected_bin,
        "version_cmd": version_cmd,
        "help_cmd": help_cmd,
        "healthcheck_cmd": healthcheck_cmd,
        "expected_version_regex": expected_version_regex,
        "version_output_parse": parsed_version,
        "version_output_matches_regex": version_matches_regex,
        "version_output_sample": version_output.lines().next().unwrap_or(""),
        "help_ok": !help_output.starts_with("error:"),
        "healthcheck_ok": !health_output.starts_with("error:"),
    }))?;
    Ok(())
}

fn parse_first_version(output: &str) -> Option<String> {
    let mut chars = output.chars().peekable();
    let mut token = String::new();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() {
            token.push(ch);
            while let Some(next) = chars.peek() {
                if next.is_ascii_digit() || *next == '.' || *next == '-' {
                    token.push(*next);
                    let _ = chars.next();
                } else {
                    break;
                }
            }
            if token.contains('.') {
                return Some(token);
            }
            token.clear();
        }
    }
    None
}

/// # Errors
/// Returns an error if id is not found or registry cannot be parsed.
pub fn print_registry_show_stage(registry_path: &Path, id: &str) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let Some(stage) = parse_stage_registry_rows(&raw)?
        .into_iter()
        .find(|stage| stage.id == id)
    else {
        return Err(anyhow!("stage not found in registry: {id}"));
    };
    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "id": stage.id,
        "primary_tools": stage.primary_tools,
        "optional_alternatives": stage.optional_alternatives,
        "validation_tools": stage.validation_tools,
        "reporting_tools": stage.reporting_tools,
    }))?;
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn print_registry_export_json(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let mut tools = parse_tools_registry_rows(&raw)?;
    let mut stages = parse_stage_registry_rows(&raw)?;
    tools.sort_by(|a, b| a.id.cmp(&b.id));
    stages.sort_by(|a, b| a.id.cmp(&b.id));
    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "schema_version": "bijux.registry_export.v1",
        "tools": tools,
        "stages": stages
    }))?;
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn print_registry_coverage_matrix(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut stages = parse_stage_registry_rows(&raw)?;
    stages.sort_by(|a, b| a.id.cmp(&b.id));
    let mut rows = Vec::new();
    for stage in stages {
        let stage_id = stage.id.clone();
        let mut stage_tools = stage.primary_tools.clone();
        stage_tools.extend(stage.optional_alternatives);
        stage_tools.extend(stage.validation_tools);
        stage_tools.extend(stage.reporting_tools);
        stage_tools.sort();
        stage_tools.dedup();
        for tool_id in stage_tools {
            let Some(tool) = tools.get(&tool_id) else {
                continue;
            };
            rows.push(serde_json::json!({
                "stage_id": stage_id,
                "tool_id": tool_id,
                "status": tool.status,
                "runtimes": tool.runtimes,
            }));
        }
    }
    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "schema_version": "bijux.registry.coverage_matrix.v1",
        "rows": rows
    }))?;
    Ok(())
}

#[derive(Default, Serialize)]
struct StageRegistryRow {
    id: String,
    required_tool_roles: Vec<String>,
    primary_tools: Vec<String>,
    optional_alternatives: Vec<String>,
    validation_tools: Vec<String>,
    reporting_tools: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PolicyCheckResult {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct PolicyCleanReport {
    pub schema_version: &'static str,
    pub domain: String,
    pub ok: bool,
    pub checks: Vec<PolicyCheckResult>,
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn print_env_export_json(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let mut tools = parse_tools_registry_rows(&raw)?;
    tools.sort_by(|a, b| a.id.cmp(&b.id));
    let payload = tools
        .into_iter()
        .map(|row| {
            let has_docker = row.runtimes.iter().any(|v| v == "docker") && row.dockerfile.is_some();
            let has_apptainer =
                row.runtimes.iter().any(|v| v == "apptainer") && row.apptainer_def.is_some();
            serde_json::json!({
                "id": row.id,
                "status": row.status,
                "version": row.version,
                "upstream": row.upstream,
                "runtimes": row.runtimes,
                "dockerfile": row.dockerfile,
                "apptainer_def": row.apptainer_def,
                "version_cmd": row.version_cmd,
                "help_cmd": row.help_cmd,
                "expected_bin": row.expected_bin,
                "pinned_commit": row.pinned_commit,
                "has_docker": has_docker,
                "has_apptainer": has_apptainer,
                "has_smoke": row.version_cmd.is_some(),
                "platforms": ["linux/arm64", "linux/amd64"]
            })
        })
        .collect::<Vec<_>>();
    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "schema_version": "bijux.environment_export.v1",
        "tools": payload
    }))?;
    Ok(())
}

