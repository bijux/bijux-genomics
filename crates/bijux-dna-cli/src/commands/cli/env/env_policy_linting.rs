/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn print_registry_audit_fix_suggestions(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?;
    for tool in tools {
        let mut suggestions = Vec::new();
        if tool.bindings.is_empty() {
            if tool.stage_ids.is_empty() {
                suggestions.push("bindings = [\"<domain.stage>\"]".to_string());
            } else {
                suggestions.push(format!("bindings = {}", toml_array_inline(&tool.stage_ids)));
            }
        }
        if tool.domains.is_empty() {
            let mut domains = tool
                .bindings
                .iter()
                .chain(tool.stage_ids.iter())
                .filter_map(|stage_id| stage_id.split('.').next().map(str::to_string))
                .collect::<Vec<_>>();
            domains.sort();
            domains.dedup();
            if !domains.is_empty() {
                suggestions.push(format!("domains = {}", toml_array_inline(&domains)));
            }
        }
        if tool.tool_role.as_deref().unwrap_or("").trim().is_empty() {
            suggestions.push("tool_role = \"<aligner|screen|trimmer|qc|filter|validator|merger|corrector|transform>\"".to_string());
        }
        if let Some(domain) = tool.domain.clone().filter(|_| !tool.bindings.is_empty()) {
            let mismatch = tool.bindings.iter().any(|stage_id| {
                stage_id
                    .split('.')
                    .next()
                    .is_some_and(|d| d != domain.as_str())
            });
            if mismatch {
                suggestions.push(format!("# domain mismatch: current domain = \"{domain}\""));
            }
        }
        if suggestions.is_empty() {
            continue;
        }
        println!("[[tools]] # id = \"{}\"", tool.id);
        for line in suggestions {
            println!("{line}");
        }
        println!();
    }
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn registry_binding_violations(
    registry_path: &Path,
    domain: Option<&str>,
) -> Result<Vec<String>> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?;
    let stages = parse_stage_registry_rows(&raw)?;
    let stage_ids = stages.into_iter().map(|row| row.id).collect::<HashSet<_>>();

    let mut offenders = Vec::new();
    for tool in tools {
        if tool.id.is_empty() || tool.status == "planned" || tool.status == "out_of_scope" {
            continue;
        }
        let bindings = if tool.bindings.is_empty() {
            tool.stage_ids.clone()
        } else {
            tool.bindings.clone()
        };
        if bindings.is_empty() {
            offenders.push(format!("tool={} missing non-empty bindings", tool.id));
            continue;
        }
        if let Some(dom) = domain {
            let relevant = bindings
                .iter()
                .any(|stage_id| stage_id.starts_with(&format!("{dom}.")));
            if !relevant {
                continue;
            }
        }
        for stage_id in &bindings {
            if !stage_ids.contains(stage_id) {
                offenders.push(format!(
                    "tool={} binding references unknown stage {}",
                    tool.id, stage_id
                ));
            }
            if let Some(dom) = domain {
                if !stage_id.starts_with(&format!("{dom}.")) {
                    offenders.push(format!(
                        "tool={} binding {} crosses requested domain {}",
                        tool.id, stage_id, dom
                    ));
                }
            }
        }
    }

    offenders.sort();
    offenders.dedup();
    Ok(offenders)
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn print_registry_binding_violations(registry_path: &Path, domain: Option<&str>) -> Result<()> {
    let offenders = registry_binding_violations(registry_path, domain)?;
    if offenders.is_empty() {
        println!("binding_violations: none");
        return Ok(());
    }
    for offender in offenders {
        println!("{offender}");
    }
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn policy_clean_report(registry_path: &Path, domain: &str) -> Result<PolicyCleanReport> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?;
    let stages = parse_stage_registry_rows(&raw)?;
    let tool_by_id = tools
        .iter()
        .map(|tool| (tool.id.clone(), tool))
        .collect::<std::collections::BTreeMap<_, _>>();

    let binding_offenders = registry_binding_violations(registry_path, Some(domain))?;

    let role_offenders = role_policy_offenders(&stages, &tool_by_id, domain);
    let smoke_offenders = smoke_policy_offenders(tools, domain);

    let checks = vec![
        PolicyCheckResult {
            name: "binding_policy".to_string(),
            ok: binding_offenders.is_empty(),
            detail: if binding_offenders.is_empty() {
                "ok".to_string()
            } else {
                binding_offenders.join("; ")
            },
        },
        PolicyCheckResult {
            name: "role_policy".to_string(),
            ok: role_offenders.is_empty(),
            detail: if role_offenders.is_empty() {
                "ok".to_string()
            } else {
                role_offenders.join("; ")
            },
        },
        PolicyCheckResult {
            name: "smoke_policy".to_string(),
            ok: smoke_offenders.is_empty(),
            detail: if smoke_offenders.is_empty() {
                "ok".to_string()
            } else {
                smoke_offenders.join("; ")
            },
        },
    ];
    let ok = checks.iter().all(|check| check.ok);
    Ok(PolicyCleanReport {
        schema_version: "bijux.policy.clean.v1",
        domain: domain.to_string(),
        ok,
        checks,
    })
}

fn role_policy_offenders(
    stages: &[StageRegistryRow],
    tool_by_id: &std::collections::BTreeMap<String, &RegistryRow>,
    domain: &str,
) -> Vec<String> {
    let mut offenders = Vec::new();
    for stage in stages {
        if !stage.id.starts_with(&format!("{domain}.")) {
            continue;
        }
        let required = stage
            .required_tool_roles
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        if required.is_empty() {
            offenders.push(format!("stage={} missing required_tool_roles", stage.id));
            continue;
        }
        for tool_id in stage_tool_ids(stage) {
            match tool_by_id.get(&tool_id) {
                Some(tool) if required.contains(tool.tool_role.as_deref().unwrap_or("")) => {}
                Some(tool) => offenders.push(format!(
                    "stage={} tool={} role={} not in {:?}",
                    stage.id,
                    tool_id,
                    tool.tool_role.as_deref().unwrap_or(""),
                    stage.required_tool_roles
                )),
                None => offenders.push(format!("stage={} unknown tool={tool_id}", stage.id)),
            }
        }
    }
    offenders.sort();
    offenders.dedup();
    offenders
}

fn stage_tool_ids(stage: &StageRegistryRow) -> Vec<String> {
    let mut ids = stage.primary_tools.clone();
    ids.extend(stage.optional_alternatives.clone());
    ids.extend(stage.validation_tools.clone());
    ids.extend(stage.reporting_tools.clone());
    ids.sort();
    ids.dedup();
    ids
}

fn smoke_policy_offenders(tools: Vec<RegistryRow>, domain: &str) -> Vec<String> {
    let mut offenders = Vec::new();
    for tool in tools {
        if tool.status != "supported" || !tool_in_domain(&tool, domain) {
            continue;
        }
        let version_cmd = tool
            .smoke_version_cmd
            .as_deref()
            .or(tool.version_cmd.as_deref())
            .unwrap_or("")
            .trim();
        let help_cmd = tool
            .smoke_help_cmd
            .as_deref()
            .or(tool.help_cmd.as_deref())
            .unwrap_or("")
            .trim();
        let require_help = tool.smoke_require_help.unwrap_or(true);
        if version_cmd.is_empty() || (require_help && help_cmd.is_empty()) {
            offenders.push(format!(
                "tool={} missing smoke commands (version/help policy)",
                tool.id
            ));
        }
    }
    offenders.sort();
    offenders.dedup();
    offenders
}

fn tool_in_domain(tool: &RegistryRow, domain: &str) -> bool {
    tool.bindings
        .iter()
        .chain(tool.stage_ids.iter())
        .any(|stage| stage.starts_with(&format!("{domain}.")))
}

/// # Errors
/// Returns an error if HPC registry lint fails.
pub fn lint_registry_hpc(
    cwd: &Path,
    registry_path: &Path,
    domain: Option<&str>,
    stages_csv: Option<&str>,
) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect::<HashMap<_, _>>();
    let mut stages = parse_stage_registry_rows(&raw)?;
    if let Some(csv) = stages_csv {
        let normalized = normalize_stage_ids(domain.unwrap_or("fastq"), csv);
        stages.retain(|row| normalized.contains(&row.id));
    } else if let Some(dom) = domain {
        let prefix = format!("{dom}.");
        stages.retain(|row| row.id.starts_with(&prefix));
    }

    let mut offenders = Vec::new();
    for stage in stages {
        let mut stage_tools = stage.primary_tools.clone();
        stage_tools.extend(stage.optional_alternatives);
        stage_tools.extend(stage.validation_tools);
        stage_tools.extend(stage.reporting_tools);
        stage_tools.sort();
        stage_tools.dedup();
        for tool_id in stage_tools {
            let Some(tool) = tools.get(&tool_id) else {
                offenders.push(format!(
                    "stage={} tool={} missing [[tools]] row",
                    stage.id, tool_id
                ));
                continue;
            };
            let Some(def_rel) = tool.apptainer_def.as_deref() else {
                offenders.push(format!(
                    "stage={} tool={} missing apptainer_def",
                    stage.id, tool_id
                ));
                continue;
            };
            let def_path = cwd.join(def_rel);
            if !def_path.exists() {
                offenders.push(format!(
                    "stage={} tool={} apptainer_def missing at {}",
                    stage.id,
                    tool_id,
                    def_path.display()
                ));
                continue;
            }
            let raw_def = std::fs::read_to_string(&def_path)
                .with_context(|| format!("read {}", def_path.display()))?;
            if !raw_def
                .lines()
                .any(|line| line.trim_start().starts_with("Bootstrap:"))
            {
                offenders.push(format!(
                    "stage={} tool={} apptainer_def missing Bootstrap header",
                    stage.id, tool_id
                ));
            }
            if !raw_def.contains("%post") {
                offenders.push(format!(
                    "stage={} tool={} apptainer_def missing %post section",
                    stage.id, tool_id
                ));
            }
        }
    }
    if !offenders.is_empty() {
        return Err(anyhow!(
            "registry lint --hpc failed:\n{}",
            offenders.join("\n")
        ));
    }
    println!("registry lint --hpc: ok");
    Ok(())
}

/// # Errors
/// Returns an error if registry policy checks fail.
pub fn print_registry_doctor(registry_path: &Path, domain: Option<&str>) -> Result<()> {
    let domain = domain.unwrap_or("fastq");
    let report = policy_clean_report(registry_path, domain)?;
    println!("registry doctor domain={domain}");
    for check in &report.checks {
        let status = if check.ok { "ok" } else { "failed" };
        println!("- {}: {status}", check.name);
        if !check.ok {
            println!("  detail: {}", check.detail);
        }
    }
    if report.ok {
        println!("registry doctor: policy-clean");
        Ok(())
    } else {
        Err(anyhow!("registry doctor failed for domain={domain}"))
    }
}

