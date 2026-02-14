/// # Errors
/// Returns an error if a tool cannot be promoted under registry contracts.
pub fn promote_registry_tool(registry_path: &Path, cwd: &Path, id: &str) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?;
    let Some(tool) = tools.into_iter().find(|tool| tool.id == id) else {
        return Err(anyhow!("tool `{id}` not found in {}", registry_path.display()));
    };

    let mut domains = BTreeSet::new();
    for binding in tool.bindings.iter().chain(tool.stage_ids.iter()) {
        if let Some((domain, _)) = binding.split_once('.') {
            domains.insert(domain.to_string());
        }
    }
    if domains.is_empty() {
        return Err(anyhow!("tool `{id}` has no stage bindings/domains"));
    }

    let mut failures = Vec::new();
    for domain in &domains {
        let report = policy_clean_report(registry_path, domain)?;
        if !report.ok {
            failures.push(format!("domain={domain}: registry not policy-clean"));
        }
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
    if version_cmd.is_empty() || (tool.smoke_require_help.unwrap_or(true) && help_cmd.is_empty()) {
        failures.push("tool has smoke warnings/errors (missing smoke version/help probe)".to_string());
    }

    let mut referenced_in_suite = false;
    let bench_suites = bijux_dna_infra::bench_suites_dir(cwd);
    for root in [bench_suites, cwd.join("examples")] {
        if !root.exists() {
            continue;
        }
        for file in collect_toml_files(&root) {
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            if content.contains(&format!("\"{id}\"")) {
                referenced_in_suite = true;
                break;
            }
        }
        if referenced_in_suite {
            break;
        }
    }
    if !referenced_in_suite {
        failures.push(format!(
            "tool `{id}` not referenced by any benchmark suite (crates/bijux-dna-bench/bench/suites/*.toml or examples/**/bench-suite.toml)"
        ));
    }

    if !failures.is_empty() {
        return Err(anyhow!(
            "registry promote tool {} refused:\n{}",
            id,
            failures.join("\n")
        ));
    }

    let updated_registry = set_registry_tool_status(&raw, id, "supported")?;
    write_text_file(registry_path, &updated_registry)?;

    let versions_path = cwd.join("containers/versions/versions.toml");
    upsert_container_version_entry(
        &versions_path,
        id,
        tool.version.as_deref(),
        tool.upstream.as_deref(),
    )?;

    let manifest_value = crate::commands::cli::env::registry_export_containers_value(registry_path)?;
    let manifest_path = cwd.join("artifacts/container_manifest.json");
    ensure_parent_dir(&manifest_path)?;
    let manifest_pretty =
        serde_json::to_string_pretty(&manifest_value).context("serialize container manifest")?;
    write_text_file(&manifest_path, &format!("{manifest_pretty}\n"))?;

    println!(
        "registry promote tool {id}: updated status + versions.toml + container manifest snapshot"
    );
    Ok(())
}

fn set_registry_tool_status(raw: &str, tool_id: &str, target_status: &str) -> Result<String> {
    let mut lines = raw.lines().map(str::to_string).collect::<Vec<_>>();
    let mut i = 0usize;
    while i < lines.len() {
        if lines[i].trim() != "[[tools]]" {
            i += 1;
            continue;
        }
        let block_start = i;
        let mut block_end = i + 1;
        while block_end < lines.len() && lines[block_end].trim() != "[[tools]]" {
            block_end += 1;
        }
        let mut id_line = None;
        let mut status_line = None;
        for (idx, line) in lines
            .iter()
            .enumerate()
            .take(block_end)
            .skip(block_start + 1)
        {
            let trimmed = line.trim();
            if parse_toml_string(trimmed, "id").as_deref() == Some(tool_id) {
                id_line = Some(idx);
            }
            if parse_toml_string(trimmed, "status").is_some() {
                status_line = Some(idx);
            }
        }
        if id_line.is_some() {
            let replacement = format!("status = \"{target_status}\"");
            if let Some(status_idx) = status_line {
                lines[status_idx] = replacement;
            } else if let Some(id_idx) = id_line {
                lines.insert(id_idx + 1, replacement);
            }
            return Ok(format!("{}\n", lines.join("\n")));
        }
        i = block_end;
    }
    Err(anyhow!("tool `{tool_id}` block not found in registry"))
}

fn normalize_semver_like(value: Option<&str>) -> String {
    let Some(raw) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return "0.0.0".to_string();
    };
    let trimmed = raw.trim_start_matches('v');
    let mut parts = trimmed
        .split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .find(|part| !part.is_empty())
        .unwrap_or_default()
        .split('.')
        .filter(|part| !part.is_empty())
        .take(3)
        .map(str::to_string)
        .collect::<Vec<_>>();
    while parts.len() < 3 {
        parts.push("0".to_string());
    }
    if parts.iter().all(|part| part.chars().all(|ch| ch.is_ascii_digit())) {
        parts.join(".")
    } else {
        "0.0.0".to_string()
    }
}

fn upsert_container_version_entry(
    versions_path: &Path,
    tool_id: &str,
    version: Option<&str>,
    source: Option<&str>,
) -> Result<()> {
    let raw = std::fs::read_to_string(versions_path)
        .with_context(|| format!("read {}", versions_path.display()))?;
    let mut parsed: toml::Value = raw
        .parse()
        .with_context(|| format!("parse {}", versions_path.display()))?;
    let table = parsed
        .as_table_mut()
        .ok_or_else(|| anyhow!("{} must contain a top-level table", versions_path.display()))?;
    let mut row = toml::map::Map::new();
    row.insert(
        "version".to_string(),
        toml::Value::String(normalize_semver_like(version)),
    );
    row.insert(
        "source".to_string(),
        toml::Value::String(
            source
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map_or_else(|| format!("tag:{tool_id}"), str::to_string),
        ),
    );
    row.insert(
        "date_pinned".to_string(),
        toml::Value::String("2026-02-12".to_string()),
    );
    table.insert(tool_id.to_string(), toml::Value::Table(row));
    let rendered = toml::to_string_pretty(&parsed)
        .with_context(|| format!("render {}", versions_path.display()))?;
    write_text_file(versions_path, &format!("{rendered}\n"))?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_api::v1::api::run::ensure_dir(parent)
            .with_context(|| format!("mkdir {}", parent.display()))?;
    }
    Ok(())
}

fn write_text_file(path: &Path, content: &str) -> Result<()> {
    ensure_parent_dir(path)?;
    bijux_dna_api::v1::api::run::write_bytes(path, content.as_bytes())
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn collect_toml_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(std::ffi::OsStr::to_str) == Some("toml") {
                out.push(path);
            }
        }
    }
    out
}

fn toml_array_inline(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{value}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
