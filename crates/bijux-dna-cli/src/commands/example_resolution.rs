fn normalize_species_id_for_path(cwd: &Path, raw: &str) -> Result<String> {
    let resolved = resolve_species_alias(cwd, raw)?;
    let words = resolved
        .split_whitespace()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    if words.len() == 2 {
        return Ok(format!("{}_{}", words[0], words[1]));
    }
    if raw
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && raw.contains('_')
    {
        return Ok(raw.to_string());
    }
    Err(anyhow!(
        "example species must be latin binomial or canonical species_id, got `{raw}`"
    ))
}

fn resolve_species_alias(cwd: &Path, raw: &str) -> Result<String> {
    let path = bijux_dna_infra::configs_file(&cwd, "runtime/species_aliases.toml");
    let input = raw.trim();
    let input_key = input.to_ascii_lowercase();
    let raw_toml = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&raw_toml).with_context(|| format!("parse {}", path.display()))?;
    let table = value
        .get("aliases")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| anyhow!("{} missing [aliases] table", path.display()))?;
    Ok(table
        .get(&input_key)
        .and_then(toml::Value::as_str)
        .map_or_else(|| input.to_string(), str::to_string))
}

fn load_example(cwd: &Path, id: &str) -> Result<(ExampleSpec, PathBuf)> {
    let root = cwd.join("examples").join(id);
    let path = root.join("example.toml");
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let spec: ExampleSpec =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok((spec, root))
}

fn stage_exists_in_registry(cwd: &Path, stage_id: &str) -> Result<bool> {
    let raw = fs::read_to_string(bijux_dna_infra::configs_file(
        &cwd,
        "ci/registry/tool_registry.toml",
    ))?;
    let doc: toml::Value = toml::from_str(&raw)?;
    let Some(stages) = doc.get("stages").and_then(toml::Value::as_array) else {
        return Ok(false);
    };
    Ok(stages.iter().any(|row| {
        row.get("id")
            .and_then(toml::Value::as_str)
            .is_some_and(|id| id == stage_id)
    }))
}

fn primary_tool_for_stage(cwd: &Path, stage_id: &str) -> Option<String> {
    let raw =
        fs::read_to_string(bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml"))
            .ok()?;
    let doc: toml::Value = toml::from_str(&raw).ok()?;
    let stages = doc.get("stages")?.as_array()?;
    let row = stages.iter().find(|row| {
        row.get("id")
            .and_then(toml::Value::as_str)
            .is_some_and(|id| id == stage_id)
    })?;
    row.get("primary_tools")
        .and_then(toml::Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}
