#![allow(clippy::too_many_lines)]

use super::{
    env_or_default, env_or_empty, failure_lines, load_toml, read_json, read_utf8, registry_tool_id,
    success_line, table_array_strings, table_bool, table_string, BTreeMap, BTreeSet,
    ContainerCommandOutcome, PathBuf, Regex, Result, Workspace,
};

pub(in super::super::super) fn vcf_imputation_core_tools() -> [&'static str; 8] {
    ["glimpse", "impute5", "minimac4", "shapeit5", "beagle", "eagle", "bcftools", "plink2"]
}

pub(in super::super::super) fn load_summary_rows(
    path: &std::path::Path,
) -> Result<BTreeMap<String, serde_json::Value>> {
    let summary = read_json(path)?;
    Ok(summary
        .get("items")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .map(|value| value.trim().to_string())?;
            Some((tool, row))
        })
        .collect())
}

pub(in super::super::super) fn normalized_parity_output(value: &str) -> String {
    value
        .chars()
        .map(
            |ch| {
                if ch.is_ascii_alphanumeric() || ch == '.' {
                    ch.to_ascii_lowercase()
                } else {
                    ' '
                }
            },
        )
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(in super::super::super) fn check_vcf_imputation_toolchain(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let required =
        load_toml(&workspace.path("configs/ci/tools/required_tools_vcf_downstream.toml"))?;
    let registry =
        load_toml(&workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"))?;
    let registry_vcf = load_toml(&workspace.path("configs/ci/registry/tool_registry_vcf.toml"))?;

    let required_set = required
        .get("required_tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(|tool| tool.trim().to_string()))
        .collect::<BTreeSet<_>>();
    let registry_rows = registry
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_table().cloned())
        .collect::<Vec<_>>();
    let registry_vcf_rows = registry_vcf
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_table().cloned())
        .collect::<Vec<_>>();
    let registry_ids = registry_rows
        .iter()
        .map(registry_tool_id)
        .filter(|tool| !tool.is_empty())
        .collect::<BTreeSet<_>>();
    let rows = registry_rows
        .into_iter()
        .map(|row| (registry_tool_id(&row), row))
        .filter(|(tool, _)| !tool.is_empty())
        .collect::<BTreeMap<_, _>>();
    let rows_vcf = registry_vcf_rows
        .into_iter()
        .map(|row| (registry_tool_id(&row), row))
        .filter(|(tool, _)| !tool.is_empty())
        .collect::<BTreeMap<_, _>>();

    let mut errors = Vec::new();
    let missing_in_required = registry_ids.difference(&required_set).cloned().collect::<Vec<_>>();
    let missing_in_registry = required_set.difference(&registry_ids).cloned().collect::<Vec<_>>();
    if !missing_in_required.is_empty() {
        errors.push(format!(
            "required_tools_vcf_downstream missing registry ids: {missing_in_required:?}"
        ));
    }
    if !missing_in_registry.is_empty() {
        errors.push(format!(
            "required_tools_vcf_downstream has unknown ids: {missing_in_registry:?}"
        ));
    }

    for tool in vcf_imputation_core_tools() {
        let row = rows.get(tool).or_else(|| rows_vcf.get(tool));
        let Some(row) = row else {
            errors.push(format!("{tool}: missing in VCF registry surfaces"));
            continue;
        };
        if !table_bool(row, "container") {
            errors.push(format!("{tool}: container=false in vcf downstream registry"));
        }
        let runtimes = table_array_strings(row, "runtimes").into_iter().collect::<BTreeSet<_>>();
        if !runtimes.contains("docker") || !runtimes.contains("apptainer") {
            errors
                .push(format!("{tool}: runtimes must include docker+apptainer, got {runtimes:?}"));
        }
        for key in
            ["smoke_version_cmd", "smoke_help_cmd", "version_cmd", "help_cmd", "expected_bin"]
        {
            if table_string(row, key).is_empty() {
                errors.push(format!("{tool}: missing {key}"));
            }
        }
        let dockerfile = table_string(row, "dockerfile");
        let apptainer_def = table_string(row, "apptainer_def");
        if dockerfile.is_empty() || !workspace.path(&dockerfile).exists() {
            errors.push(format!(
                "{tool}: dockerfile missing: {}",
                if dockerfile.is_empty() { "<empty>" } else { &dockerfile }
            ));
        }
        if apptainer_def.is_empty() || !workspace.path(&apptainer_def).exists() {
            errors.push(format!(
                "{tool}: apptainer_def missing: {}",
                if apptainer_def.is_empty() { "<empty>" } else { &apptainer_def }
            ));
        }
        let license_file = workspace.path(&format!("containers/licenses/{tool}.license.toml"));
        if !license_file.exists() {
            errors.push(format!(
                "{tool}: missing license metadata {}",
                workspace.rel(&license_file).display()
            ));
        }
        let tool_doc = workspace.path(&format!("containers/docs/tools/{tool}.md"));
        if !tool_doc.exists() {
            errors.push(format!("{tool}: missing tool doc {}", workspace.rel(&tool_doc).display()));
        }
    }

    if errors.is_empty() {
        return success_line(format!(
            "vcf imputation toolchain check: OK ({}) core tools",
            vcf_imputation_core_tools().len()
        ));
    }
    failure_lines("vcf imputation toolchain check: FAILED", &errors)
}

pub(in super::super::super) fn check_imputation_runtime_constraints(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let doc_path = workspace.path("containers/docs/IMPUTATION_RUNTIME_CONSTRAINTS.md");
    if !doc_path.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!("missing {}\n", doc_path.display())));
    }
    let doc = read_utf8(&doc_path)?;
    let mut errors = Vec::new();
    for tool in vcf_imputation_core_tools() {
        if !doc.contains(&format!("| `{tool}` |")) {
            errors.push(format!("missing constraints row for {tool}"));
        }
    }
    for column in ["cpu_threads_min", "ram_gb_min", "scratch_gb_min"] {
        if !doc.contains(column) {
            errors.push(format!("constraints column {column} is required"));
        }
    }
    if errors.is_empty() {
        return success_line("imputation runtime constraints: OK");
    }
    failure_lines("imputation runtime constraints: FAILED", &errors)
}

pub(in super::super::super) fn check_imputation_network_policy(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let doc_path = workspace.path("containers/docs/IMPUTATION_NETWORK_POLICY.md");
    if !doc_path.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!("missing {}\n", doc_path.display())));
    }
    let mut errors = Vec::new();
    for tool in vcf_imputation_core_tools() {
        let path = workspace.path(&format!("containers/network/{tool}.network.toml"));
        if !path.exists() {
            errors.push(format!("missing network metadata: {}", workspace.rel(&path).display()));
            continue;
        }
        let data = load_toml(&path)?;
        if data.get("runtime_network").and_then(toml::Value::as_bool).unwrap_or(true) {
            errors.push(format!("{tool}: runtime_network must be false"));
        }
    }
    if errors.is_empty() {
        return success_line("imputation network policy: OK");
    }
    failure_lines("imputation network policy: FAILED", &errors)
}

pub(in super::super::super) fn check_imputation_hardening(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let nonroot_ex = read_utf8(&workspace.path("containers/docker/NONROOT_EXCEPTIONS.md"))?;
    let entrypoint_ex = read_utf8(&workspace.path("containers/docker/ENTRYPOINT_EXCEPTIONS.md"))?;
    let wildcard_nonroot = nonroot_ex.contains("`*`");
    let wildcard_entrypoint = entrypoint_ex.contains("`*`");
    let user_regex = Regex::new(r"(?m)^USER\s+")?;
    let entrypoint_regex = Regex::new(r"(?m)^ENTRYPOINT\s+\[")?;
    let cmd_regex = Regex::new(r"(?m)^CMD\s+\[")?;
    let mut errors = Vec::new();
    for tool in vcf_imputation_core_tools() {
        let dockerfile = workspace.path(&format!("containers/docker/arm64/Dockerfile.{tool}"));
        if !dockerfile.exists() {
            errors.push(format!("{tool}: missing dockerfile"));
            continue;
        }
        let text = read_utf8(&dockerfile)?;
        if !user_regex.is_match(&text)
            && !wildcard_nonroot
            && !nonroot_ex.contains(&format!("`{tool}`"))
        {
            errors.push(format!("{tool}: runs as root and is not listed in NONROOT_EXCEPTIONS.md"));
        }
        if (!entrypoint_regex.is_match(&text) || !cmd_regex.is_match(&text))
            && !wildcard_entrypoint
            && !entrypoint_ex.contains(&format!("`{tool}`"))
        {
            errors.push(format!(
                "{tool}: missing JSON ENTRYPOINT/CMD and not listed in ENTRYPOINT_EXCEPTIONS.md"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("imputation hardening policy: OK");
    }
    failure_lines("imputation hardening policy: FAILED", &errors)
}

pub(in super::super::super) fn check_imputation_release_smoke(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let docker_summary = PathBuf::from(env_or_default(
        "DOCKER_SUMMARY",
        &workspace.path("artifacts/containers/docker-arm64/summary.json").display().to_string(),
    ));
    let apptainer_summary = PathBuf::from(env_or_default(
        "APPTAINER_SUMMARY",
        &workspace.path("artifacts/containers/apptainer/summary.json").display().to_string(),
    ));
    if !docker_summary.is_file() || !apptainer_summary.is_file() {
        if !env_or_empty("CI").is_empty() {
            return Ok(ContainerCommandOutcome::failure(format!(
                "imputation release smoke: missing summary files docker='{}' apptainer='{}'\n",
                docker_summary.display(),
                apptainer_summary.display()
            )));
        }
        return success_line("imputation release smoke: SKIP (missing local summary files)");
    }

    let docker_rows = load_summary_rows(&docker_summary)?;
    let apptainer_rows = load_summary_rows(&apptainer_summary)?;
    let mut errors = Vec::new();
    for (runtime, rows) in [("docker", &docker_rows), ("apptainer", &apptainer_rows)] {
        for tool in vcf_imputation_core_tools() {
            let Some(row) = rows.get(tool) else {
                errors.push(format!("{runtime}:{tool}: missing summary row"));
                continue;
            };
            if row.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
                errors.push(format!("{runtime}:{tool}: status is not ok"));
            }
            let paths = row
                .get("smoke_output_paths")
                .and_then(serde_json::Value::as_object)
                .cloned()
                .unwrap_or_default();
            for key in ["version", "help"] {
                let output_path = paths
                    .get(key)
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if output_path.is_empty() {
                    errors.push(format!("{runtime}:{tool}: missing smoke_output_paths.{key}"));
                } else if !PathBuf::from(&output_path).exists() {
                    errors.push(format!("{runtime}:{tool}: missing output file {output_path}"));
                }
            }
            if row
                .get("version_output")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                errors.push(format!("{runtime}:{tool}: empty version_output"));
            }
            if row
                .get("resolved_image_digest")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                errors.push(format!("{runtime}:{tool}: missing resolved_image_digest"));
            }
        }
    }
    if errors.is_empty() {
        return success_line("imputation release smoke: OK");
    }
    failure_lines("imputation release smoke: FAILED", &errors)
}

pub(in super::super::super) fn check_imputation_cross_runtime_parity(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let docker_summary = PathBuf::from(env_or_default(
        "DOCKER_SUMMARY",
        &workspace.path("artifacts/containers/docker-arm64/summary.json").display().to_string(),
    ));
    let apptainer_summary = PathBuf::from(env_or_default(
        "APPTAINER_SUMMARY",
        &workspace.path("artifacts/containers/apptainer/summary.json").display().to_string(),
    ));
    if !docker_summary.is_file() || !apptainer_summary.is_file() {
        if !env_or_empty("CI").is_empty() {
            return Ok(ContainerCommandOutcome::failure(format!(
                "imputation cross-runtime parity: missing summary files docker='{}' apptainer='{}'\n",
                docker_summary.display(),
                apptainer_summary.display()
            )));
        }
        return success_line("imputation cross-runtime parity: SKIP (missing local summary files)");
    }

    let docker_rows = load_summary_rows(&docker_summary)?;
    let apptainer_rows = load_summary_rows(&apptainer_summary)?;
    let mut errors = Vec::new();
    for tool in vcf_imputation_core_tools() {
        let Some(docker_row) = docker_rows.get(tool) else {
            errors.push(format!("{tool}: missing from one runtime summary"));
            continue;
        };
        let Some(apptainer_row) = apptainer_rows.get(tool) else {
            errors.push(format!("{tool}: missing from one runtime summary"));
            continue;
        };
        let docker_version = normalized_parity_output(
            docker_row
                .get("version_output")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        let apptainer_version = normalized_parity_output(
            apptainer_row
                .get("version_output")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        );
        if docker_version.is_empty() || apptainer_version.is_empty() {
            errors.push(format!("{tool}: empty version output for parity check"));
            continue;
        }
        if !docker_version.contains(tool) || !apptainer_version.contains(tool) {
            errors.push(format!("{tool}: version outputs do not contain expected tool token"));
            continue;
        }
        let declared = docker_row
            .get("declared_version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        if !declared.is_empty()
            && !matches!(declared.as_str(), "unknown" | "planned" | "latest-pinned")
            && (!docker_version.contains(&declared) || !apptainer_version.contains(&declared))
        {
            errors.push(format!(
                "{tool}: declared_version `{declared}` not present in both runtime outputs"
            ));
        }
    }
    if errors.is_empty() {
        return success_line("imputation cross-runtime parity: OK");
    }
    failure_lines("imputation cross-runtime parity: FAILED", &errors)
}
