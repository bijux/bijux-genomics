fn normalize_stage_ids(domain: &str, stages_csv: &str) -> Vec<String> {
    let mut stage_ids = stages_csv
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| {
            if item.contains('.') {
                item.to_string()
            } else {
                format!("{domain}.{item}")
            }
        })
        .collect::<Vec<_>>();
    stage_ids.sort();
    stage_ids.dedup();
    stage_ids
}

fn expected_registry_digest(tool: &RegistryRow) -> Option<String> {
    let pin = tool.pinned_commit.as_deref().unwrap_or("");
    if let Some(digest) = pin.strip_prefix("sha256:") {
        return Some(digest.to_string());
    }
    let container_ref = tool.container_ref.as_deref().unwrap_or("");
    container_ref
        .split("@sha256:")
        .nth(1)
        .map(std::string::ToString::to_string)
}

fn build_apptainer_image(def_path: &Path, sif_path: &Path) -> Result<()> {
    if let Some(parent) = sif_path.parent() {
        bijux_dna_api::v1::api::run::ensure_dir(parent)?;
    }
    let cmd = format!(
        "apptainer build --force '{}' '{}'",
        sif_path.display(),
        def_path.display()
    );
    if let Err(err) = bijux_dna_api::v1::api::env::run_shell_capture(&cmd) {
        return Err(anyhow!(
            "apptainer build failed for {}: {}",
            def_path.display(),
            err
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_smoke_with_manifest(
    sif_path: &Path,
    tool_id: &str,
    stage_id: &str,
    expected_digest: &str,
    actual_digest: &str,
    version_cmd: &str,
    help_cmd: &str,
    require_help: bool,
    probe_commands: &[String],
    java_heap_mb: Option<u64>,
    upstream: &str,
    data_root: &Path,
    results_root: &Path,
) -> SmokeManifest {
    let effective_probes = if probe_commands.is_empty() {
        let mut probes = vec![version_cmd.to_string()];
        if require_help {
            probes.push(help_cmd.to_string());
        }
        probes
    } else {
        probe_commands.to_vec()
    };
    let mut outputs = Vec::new();
    let mut probe_failures = 0usize;
    for probe in &effective_probes {
        let applied = apply_heap_policy(tool_id, probe, java_heap_mb);
        match run_apptainer_exec(sif_path, &applied, data_root, results_root) {
            Ok(output) => outputs.push(output),
            Err(_) => probe_failures += 1,
        }
    }
    let version_out = if outputs.is_empty() {
        String::new()
    } else {
        outputs[0].clone()
    };
    let help_ok = if probe_commands.is_empty() {
        if require_help {
            probe_failures == 0 && outputs.len() >= 2
        } else {
            probe_failures == 0
        }
    } else {
        probe_failures == 0
    };
    let parsed_version = parse_first_version(&version_out).unwrap_or_default();
    let fallback_version = version_out
        .lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .unwrap_or("n/a")
        .to_string();
    let status = if help_ok && !parsed_version.is_empty() {
        "ok"
    } else {
        "wrapper_failed"
    };
    let image_build_timestamp_unix_s = std::fs::metadata(sif_path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|ts| ts.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map_or(0, |dur| dur.as_secs());
    SmokeManifest {
        schema_version: "bijux.apptainer.smoke_manifest.v2",
        tool_id: tool_id.to_string(),
        stage_id: stage_id.to_string(),
        status: status.to_string(),
        expected_digest: expected_digest.to_string(),
        actual_digest: actual_digest.to_string(),
        version_cmd: version_cmd.to_string(),
        help_cmd: help_cmd.to_string(),
        version: if parsed_version.is_empty() {
            fallback_version
        } else {
            parsed_version
        },
        version_output_first_line: version_out.lines().next().unwrap_or("").to_string(),
        help_ok,
        quick_smoke: true,
        probe_commands: effective_probes,
        java_heap_mb,
        upstream: upstream.to_string(),
        image_build_timestamp_unix_s,
        checked_at_unix_s: now_unix_s(),
    }
}

fn apply_heap_policy(tool_id: &str, command: &str, java_heap_mb: Option<u64>) -> String {
    let Some(heap_mb) = java_heap_mb else {
        return command.to_string();
    };
    if !matches!(tool_id, "bbduk" | "bbmerge") {
        return command.to_string();
    }
    if command.contains("-Xmx") {
        return command.to_string();
    }
    let mut parts = command.split_whitespace();
    let Some(bin) = parts.next() else {
        return command.to_string();
    };
    let rest = parts.collect::<Vec<_>>().join(" ");
    if rest.is_empty() {
        format!("{bin} -Xmx{heap_mb}m")
    } else {
        format!("{bin} -Xmx{heap_mb}m {rest}")
    }
}

fn run_apptainer_exec(
    sif_path: &Path,
    command: &str,
    data_root: &Path,
    results_root: &Path,
) -> Result<String> {
    if !data_root.exists() {
        return Err(anyhow!("input bind root missing: {}", data_root.display()));
    }
    if !results_root.exists() {
        return Err(anyhow!(
            "output bind root missing: {}",
            results_root.display()
        ));
    }
    let banks_root = data_root.join("banks");
    if !banks_root.exists() {
        return Err(anyhow!("db bind root missing: {}", banks_root.display()));
    }
    let input_bind = format!("{}:/bijux/input:ro", data_root.display());
    let output_bind = format!("{}:/bijux/output:rw", results_root.display());
    let db_bind = format!("{}:/bijux/db:ro", banks_root.display());
    let cmd = format!(
        "apptainer exec --containall --cleanenv --net --network none --bind '{}' --bind '{}' --bind '{}' '{}' sh -lc '{}'",
        input_bind,
        output_bind,
        db_bind,
        sif_path.display(),
        command.replace('\'', "'\\''")
    );
    bijux_dna_api::v1::api::env::run_shell_capture(&cmd)
        .with_context(|| format!("apptainer exec {}", sif_path.display()))
}

fn hash_file_sha256_hex(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .with_context(|| format!("read {}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn should_run_weekly_quick_smoke(manifest_path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(manifest_path) else {
        return true;
    };
    let Ok(modified) = meta.modified() else {
        return true;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return true;
    };
    age >= Duration::from_secs(7 * 24 * 3600)
}

fn now_unix_s() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |dur| dur.as_secs())
}

fn parse_stage_registry_rows(raw: &str) -> Result<Vec<StageRegistryRow>> {
    let mut rows = Vec::new();
    let mut current: Option<StageRegistryRow> = None;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[[stages]]" {
            if let Some(row) = current.take() {
                rows.push(row);
            }
            current = Some(StageRegistryRow::default());
            continue;
        }
        let Some(row) = current.as_mut() else {
            continue;
        };
        if let Some(value) = parse_toml_string(trimmed, "id") {
            row.id = value;
        } else if let Some(values) = parse_toml_array(trimmed, "required_tool_roles") {
            row.required_tool_roles = values;
        } else if let Some(values) = parse_toml_array(trimmed, "primary_tools") {
            row.primary_tools = values;
        } else if let Some(values) = parse_toml_array(trimmed, "optional_alternatives") {
            row.optional_alternatives = values;
        } else if let Some(values) = parse_toml_array(trimmed, "validation_tools") {
            row.validation_tools = values;
        } else if let Some(values) = parse_toml_array(trimmed, "reporting_tools") {
            row.reporting_tools = values;
        }
    }
    if let Some(row) = current {
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(anyhow!("missing [[stages]] entries"));
    }
    Ok(rows)
}

fn parse_toml_string(line: &str, key: &str) -> Option<String> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    let value = rhs.trim();
    if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2) {
        return None;
    }
    Some(value[1..value.len() - 1].to_string())
}

fn parse_toml_array(line: &str, key: &str) -> Option<Vec<String>> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    let value = rhs.trim();
    if !(value.starts_with('[') && value.ends_with(']') && value.len() >= 2) {
        return None;
    }
    let inner = &value[1..value.len() - 1];
    let items = inner
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| token.trim_matches('"').to_string())
        .collect::<Vec<_>>();
    Some(items)
}

fn parse_toml_bool(line: &str, key: &str) -> Option<bool> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    match rhs.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_toml_u64(line: &str, key: &str) -> Option<u64> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    rhs.trim().parse::<u64>().ok()
}

pub fn print_env_info<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) {
    println!("platform: {}", platform.name);
    println!("runner: {}", platform.runner);
    println!("image count: {}", catalog.len());
    println!("cache: {}", cache_dir(platform.runner).to_string_lossy());
}

pub fn env_doctor<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) {
    println!("bijux dna env doctor");
    let runners = available_runners().unwrap_or_default();
    print_check(
        "cache directory writable",
        ensure_cache_writable(platform.runner),
    );
    print_check("runner available", runners.contains(&platform.runner));
    println!("runners: {}", display_runners(&runners));
    for (tool, spec) in catalog {
        let Ok(image) = resolve_image(spec, platform) else {
            continue;
        };
        let exists = docker_image_exists(&image);
        print_check(&format!("image {tool}"), exists);
    }
}

fn ensure_cache_writable(runner: RuntimeKind) -> bool {
    let cache_dir = cache_dir(runner);
    bijux_dna_api::v1::api::run::ensure_dir(&cache_dir).is_ok()
}

fn print_check(name: &str, ok: bool) {
    if ok {
        println!("ok   {name}");
    } else {
        println!("fail {name}");
    }
}

fn display_runners(runners: &[RuntimeKind]) -> String {
    runners
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct HomeGuard {
        original: Option<std::ffi::OsString>,
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            if let Some(value) = self.original.take() {
                std::env::set_var("HOME", value);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn display_runners_is_deterministic() {
        let runners = vec![RuntimeKind::Docker, RuntimeKind::Apptainer];
        assert_eq!(display_runners(&runners), "docker, apptainer");
    }

    #[test]
    fn ensure_cache_writable_uses_home() -> anyhow::Result<()> {
        let temp = bijux_dna_api::v1::api::run::temp_dir("bijux")?;
        let original_home = std::env::var_os("HOME");
        let _guard = HomeGuard {
            original: original_home,
        };
        std::env::set_var("HOME", temp.path());
        assert!(ensure_cache_writable(RuntimeKind::Docker));
        Ok(())
    }
}
