use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::infrastructure::process::ProcessRunner;
use crate::infrastructure::workspace::Workspace;
use crate::model::container::{ContainerCommandOutcome, NativeContainerCommandKey};

pub fn run_native_container_command(
    key: &NativeContainerCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    match key {
        NativeContainerCommandKey::ContainerRuntimeCheck => {
            ensure_no_args("container-runtime-check", args)?;
            run_container_runtime_check()
        }
        NativeContainerCommandKey::GenerateToolIds => generate_tool_ids(workspace, args),
        NativeContainerCommandKey::CheckToolIdManifest => {
            ensure_no_args("check-tool-id-manifest", args)?;
            check_tool_id_manifest(workspace)
        }
        NativeContainerCommandKey::GenerateToolNameMap => generate_tool_name_map(workspace, args),
        NativeContainerCommandKey::CheckToolNameMapGenerated => {
            ensure_no_args("check-tool-name-map-generated", args)?;
            check_tool_name_map_generated(workspace)
        }
        NativeContainerCommandKey::GenerateContainerIndex => generate_container_index(workspace, args),
        NativeContainerCommandKey::CheckContainerIndex => {
            ensure_no_args("check-index", args)?;
            check_container_index(workspace)
        }
        NativeContainerCommandKey::GenerateLicenseMetadata => {
            generate_license_metadata(workspace, args)
        }
        NativeContainerCommandKey::CheckLicenseMetadata => {
            ensure_no_args("check-license-metadata", args)?;
            check_license_metadata(workspace)
        }
        NativeContainerCommandKey::CheckLicenseIndexGenerated => {
            ensure_no_args("check-license-index-generated", args)?;
            check_license_index_generated(workspace)
        }
        NativeContainerCommandKey::GenerateQaMatrix => generate_qa_matrix(workspace, args),
        NativeContainerCommandKey::CheckQaMatrixGenerated => {
            ensure_no_args("check-qa-matrix-generated", args)?;
            check_qa_matrix_generated(workspace)
        }
        NativeContainerCommandKey::GenerateToolDocs => generate_tool_docs(workspace, args),
        NativeContainerCommandKey::CheckToolDocsGenerated => {
            ensure_no_args("check-tool-docs-generated", args)?;
            check_tool_docs_generated(workspace)
        }
        NativeContainerCommandKey::GenerateNetworkUsage => generate_network_usage(workspace, args),
        NativeContainerCommandKey::CheckNetworkDisclosure => {
            check_network_disclosure(workspace, args)
        }
        NativeContainerCommandKey::ExtractVersionMap => extract_version_map(workspace, args),
        NativeContainerCommandKey::GenerateVersionLock => generate_version_lock(workspace, args),
        NativeContainerCommandKey::CheckVersionLock => {
            ensure_no_args("check-version-lock", args)?;
            check_version_lock(workspace)
        }
        NativeContainerCommandKey::CheckVersionAuthority => {
            ensure_no_args("check-version-authority", args)?;
            check_version_authority(workspace)
        }
        NativeContainerCommandKey::GenerateVersionsIndexSha => {
            generate_versions_index_sha(workspace, args)
        }
        NativeContainerCommandKey::CheckVersionsIndexSha => {
            ensure_no_args("check-versions-index-sha", args)?;
            check_versions_index_sha(workspace)
        }
        NativeContainerCommandKey::Summary => summary(workspace, args),
        NativeContainerCommandKey::EnvPrep => run_env_prep(workspace, args),
        NativeContainerCommandKey::EnvSmoke => run_env_smoke(workspace, args),
        NativeContainerCommandKey::ContainerSmoke => run_container_smoke(workspace, args),
        NativeContainerCommandKey::ContainersSmoke => run_containers_smoke(workspace, args),
        NativeContainerCommandKey::SmokeContainersDockerArm64 => {
            ensure_no_args("smoke-containers-docker-arm64", args)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-docker-arm64.sh",
                &[
                    ("TOOLS", env_or_empty("TOOLS")),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/docker-arm64", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeContainersDockerAmd64 => {
            ensure_no_args("smoke-containers-docker-amd64", args)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-docker-amd64.sh",
                &[
                    ("TOOLS", env_or_empty("TOOLS")),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/docker-amd64", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeContainersApptainer => {
            ensure_no_args("smoke-containers-apptainer", args)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-apptainer.sh",
                &[
                    ("TOOLS", env_or_empty("TOOLS")),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/apptainer", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeCntainersApptainerBijuxRun => {
            ensure_no_args("smoke-cntainers-apptainer-bijux-run", args)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-apptainer.sh",
                &[
                    ("TOOLS", env_or_empty("TOOLS")),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    ("SMOKE_RUN_MODE", "bijux-run".to_string()),
                    ("SMOKE_LEVEL", "contract".to_string()),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/apptainer-bijux-run", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeCntainersApptainerApptainerRun => {
            ensure_no_args("smoke-cntainers-apptainer-apptainer-run", args)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-apptainer.sh",
                &[
                    ("TOOLS", env_or_empty("TOOLS")),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    ("SMOKE_RUN_MODE", "apptainer-run".to_string()),
                    ("SMOKE_LEVEL", "contract".to_string()),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/apptainer-apptainer-run", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeCntainersApptainerVerify => {
            ensure_no_args("smoke-cntainers-apptainer-verify", args)?;
            let mut envs = artifact_env(workspace)?;
            envs.push((
                "PYTHONPATH".to_string(),
                pythonpath_with_tooling("scripts/tooling/python"),
            ));
            run_program_with_env(
                workspace,
                "python3",
                &[
                    "-m".to_string(),
                    "bijux_dna_tools.compare_apptainer_smoke".to_string(),
                    container_artifact_dir(),
                ],
                &envs,
            )
        }
        NativeContainerCommandKey::SmokeCrossRuntimeVerify => {
            ensure_no_args("smoke-cross-runtime-verify", args)?;
            run_program_with_env(
                workspace,
                "./scripts/containers/check-cross-runtime-smoke.sh",
                &[
                    format!("{}/docker-arm64", container_artifact_dir()),
                    format!("{}/apptainer", container_artifact_dir()),
                ],
                &artifact_env(workspace)?,
            )
        }
        NativeContainerCommandKey::SmokeToolkitDockerArm64 => {
            ensure_no_args("smoke-toolkit-docker-arm64", args)?;
            let toolkit = required_env("TOOLKIT")?;
            let tools = resolve_toolkit_tools(workspace, &toolkit)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-docker-arm64.sh",
                &[
                    ("TOOLS", tools),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    ("SMOKE_LEVEL", "contract".to_string()),
                    ("SAVE_TAR", "0".to_string()),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/docker-arm64", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::SmokeToolkitApptainer => {
            ensure_no_args("smoke-toolkit-apptainer", args)?;
            let toolkit = required_env("TOOLKIT")?;
            let tools = resolve_toolkit_tools(workspace, &toolkit)?;
            smoke_runtime_script(
                workspace,
                "scripts/containers/smoke-apptainer.sh",
                &[
                    ("TOOLS", tools),
                    ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                    ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                    ("SMOKE_LEVEL", "contract".to_string()),
                    (
                        "ARTIFACT_DIR",
                        format!("{}/apptainer", container_artifact_dir()),
                    ),
                ],
            )
        }
        NativeContainerCommandKey::BuildImages => {
            ensure_no_args("build-images", args)?;
            let tools = if env_or_empty("TOOLS").is_empty() {
                primary_tools_csv(workspace)?
            } else {
                env_or_empty("TOOLS")
            };
            run_build_contract(workspace, &tools)
        }
        NativeContainerCommandKey::BuildTool => {
            ensure_no_args("build-tool", args)?;
            run_build_contract(workspace, &required_env("TOOLS")?)
        }
        NativeContainerCommandKey::BuildAll => {
            ensure_no_args("build-all", args)?;
            run_build_contract(workspace, &primary_tools_csv(workspace)?)
        }
        NativeContainerCommandKey::BuildBundle => {
            ensure_no_args("build-bundle", args)?;
            let toolkit = required_env("TOOLKIT")?;
            run_build_contract(workspace, &resolve_toolkit_tools(workspace, &toolkit)?)
        }
        NativeContainerCommandKey::TestImages => run_test_images(workspace, args),
        NativeContainerCommandKey::TestImagesStage => run_test_images_stage(workspace, args),
        NativeContainerCommandKey::TestImagesTool => run_test_images_tool(workspace, args),
        NativeContainerCommandKey::ImageSmokeVcf => run_image_smoke_vcf(workspace, args),
        NativeContainerCommandKey::ImageQa => run_image_qa(workspace, args),
        NativeContainerCommandKey::ApptainerEnsure => run_apptainer_ensure(workspace, args),
        NativeContainerCommandKey::ApptainerEnsureStage => {
            run_apptainer_ensure_stage(workspace, args)
        }
    }
}

fn run_container_runtime_check() -> Result<ContainerCommandOutcome> {
    let system_type = std::env::var("SYSTEM_TYPE").unwrap_or_else(|_| "local".to_string());
    let container_type = checked_container_type()?;
    Ok(ContainerCommandOutcome::success(format!(
        "SYSTEM_TYPE={system_type} CONTAINER_TYPE={container_type}\n"
    )))
}

fn success_line(line: impl Into<String>) -> Result<ContainerCommandOutcome> {
    Ok(ContainerCommandOutcome::success(format!("{}\n", line.into())))
}

fn failure_lines(title: &str, errors: &[String]) -> Result<ContainerCommandOutcome> {
    let mut stderr = String::new();
    stderr.push_str(title);
    stderr.push('\n');
    for error in errors {
        stderr.push_str(error);
        if !error.ends_with('\n') {
            stderr.push('\n');
        }
    }
    Ok(ContainerCommandOutcome::failure(stderr))
}

fn read_utf8(path: &std::path::Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

fn write_utf8(path: &std::path::Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn load_toml(path: &std::path::Path) -> Result<toml::Value> {
    toml::from_str(&read_utf8(path)?).with_context(|| format!("parse TOML {}", path.display()))
}

fn registry_tool_rows(workspace: &Workspace) -> Result<Vec<toml::map::Map<String, toml::Value>>> {
    let mut rows = Vec::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let value = load_toml(&workspace.path(rel))?;
        if let Some(entries) = value.get("tools").and_then(toml::Value::as_array) {
            for entry in entries {
                if let Some(table) = entry.as_table() {
                    rows.push(table.clone());
                }
            }
        }
    }
    Ok(rows)
}

fn registry_tool_map(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let mut rows = BTreeMap::new();
    for row in registry_tool_rows(workspace)? {
        let tool_id = row
            .get("id")
            .or_else(|| row.get("tool_id"))
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !tool_id.is_empty() {
            rows.insert(tool_id, row);
        }
    }
    Ok(rows)
}

fn governed_container_file_ids(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(workspace.path("containers/docker/arm64"))
        .with_context(|| format!("read {}", workspace.path("containers/docker/arm64").display()))?
        .filter_map(std::result::Result::ok)
    {
        if let Some(tool_id) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.strip_prefix("Dockerfile."))
        {
            ids.insert(tool_id.to_string());
        }
    }
    for entry in fs::read_dir(workspace.path("containers/apptainer/lunarc"))
        .with_context(|| format!("read {}", workspace.path("containers/apptainer/lunarc").display()))?
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("def") {
            if let Some(tool_id) = path.file_stem().and_then(|name| name.to_str()) {
                ids.insert(tool_id.to_string());
            }
        }
    }
    Ok(ids)
}

fn governed_container_statuses(workspace: &Workspace) -> Result<BTreeMap<String, String>> {
    let registry = registry_tool_map(workspace)?;
    let versions = tool_versions(workspace)?;
    let mut statuses = BTreeMap::new();
    for tool_id in governed_container_file_ids(workspace)? {
        let status = registry
            .get(&tool_id)
            .map(|row| table_string(row, "status"))
            .filter(|value| !value.is_empty())
            .or_else(|| {
                versions
                    .get(&tool_id)
                    .map(|row| table_string(row, "status"))
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| "experimental".to_string());
        statuses.insert(tool_id, status);
    }
    for (tool_id, row) in registry {
        let status = table_string(&row, "status");
        if !status.is_empty() {
            statuses.entry(tool_id).or_insert(status);
        }
    }
    Ok(statuses)
}

fn is_non_bijux_apptainer_source(workspace: &Workspace, tool_id: &str) -> bool {
    let apptainer = workspace.path(&format!("containers/apptainer/lunarc/{tool_id}.def"));
    apptainer.exists()
        && (read_utf8(&apptainer)
            .unwrap_or_default()
            .contains("NON_BIJUX_SOURCES.md")
            || matches!(
                tool_id,
                "bcftools"
                    | "beagle"
                    | "eagle"
                    | "eigensoft"
                    | "germline"
                    | "glimpse"
                    | "ibdhap"
                    | "ibdne"
                    | "impute5"
                    | "minimac4"
                    | "shapeit5"
            ))
}

fn tool_versions(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let value = load_toml(&workspace.path("containers/versions/versions.toml"))?;
    let Some(table) = value.as_table() else {
        return Ok(BTreeMap::new());
    };
    let mut rows = BTreeMap::new();
    for (tool, entry) in table {
        if let Some(entry_table) = entry.as_table() {
            rows.insert(tool.clone(), entry_table.clone());
        }
    }
    Ok(rows)
}

fn table_string(table: &toml::map::Map<String, toml::Value>, key: &str) -> String {
    table
        .get(key)
        .map(toml_value_string)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn table_bool(table: &toml::map::Map<String, toml::Value>, key: &str) -> bool {
    table
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn table_array_strings(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .map(toml_value_string)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn toml_value_string(value: &toml::Value) -> String {
    match value {
        toml::Value::String(value) => value.clone(),
        toml::Value::Integer(value) => value.to_string(),
        toml::Value::Float(value) => value.to_string(),
        toml::Value::Boolean(value) => value.to_string(),
        toml::Value::Datetime(value) => value.to_string(),
        toml::Value::Array(values) => values
            .iter()
            .map(toml_value_string)
            .collect::<Vec<_>>()
            .join(","),
        toml::Value::Table(_) => String::new(),
    }
}

fn markdown_code_value(value: &str) -> String {
    value.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn has_shell_word(line: &str, word: &str) -> bool {
    line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
        .any(|token| token == word)
}

fn line_has_network_command(line: &str) -> bool {
    let lowered = line.to_ascii_lowercase();
    lowered.contains("git clone")
        || lowered.contains("apt-get update")
        || has_shell_word(&lowered, "curl")
        || has_shell_word(&lowered, "wget")
}

fn read_json(path: &std::path::Path) -> Result<serde_json::Value> {
    serde_json::from_str(&read_utf8(path)?).with_context(|| format!("parse JSON {}", path.display()))
}

fn json_string_pretty(value: &serde_json::Value) -> Result<String> {
    Ok(format!("{}\n", serde_json::to_string_pretty(value)?))
}

fn git_last_modified_timestamp(workspace: &Workspace, rel_path: &str) -> String {
    std::process::Command::new("git")
        .arg("-C")
        .arg(&workspace.root)
        .args(["log", "-1", "--format=%cI", "--", rel_path])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

fn out_path_arg(
    workspace: &Workspace,
    args: &[String],
    default_rel: &str,
    usage: &str,
) -> Result<PathBuf> {
    match args {
        [] => Ok(workspace.path(default_rel)),
        [single] if single == "--help" || single == "-h" => Err(anyhow!(usage.to_string())),
        [single] => Ok(path_from_arg(workspace, single)),
        _ => Err(anyhow!(usage.to_string())),
    }
}

fn path_from_arg(workspace: &Workspace, arg: &str) -> PathBuf {
    let path = PathBuf::from(arg);
    if path.is_absolute() {
        path
    } else {
        workspace.root.join(path)
    }
}

fn generate_tool_ids_content(workspace: &Workspace) -> Result<String> {
    let statuses = governed_container_statuses(workspace)?;
    let mut out = String::from(
        "# GENERATED FILE - DO NOT EDIT\n# Regenerate with: cargo run -p bijux-dev-dna -- containers run generate-tool-ids\n# format: <tool_id><TAB><status>\n",
    );
    for (tool_id, status) in statuses {
        out.push_str(&format!("{tool_id}\t{status}\n"));
    }
    Ok(out)
}

fn generate_tool_ids(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dev-dna -- containers run generate-tool-ids -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "containers/TOOL_IDS.txt", usage)?;
    write_utf8(&out, &generate_tool_ids_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

fn check_tool_id_manifest(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let manifest = workspace.path("containers/TOOL_IDS.txt");
    if !manifest.is_file() {
        return Ok(ContainerCommandOutcome::failure(
            "missing tool id manifest: containers/TOOL_IDS.txt\n",
        ));
    }
    let expected = generate_tool_ids_content(workspace)?;
    let actual = read_utf8(&manifest)?;
    if actual != expected {
        return Ok(ContainerCommandOutcome::failure(
            "containers/TOOL_IDS.txt drift; regenerate with cargo run -p bijux-dev-dna -- containers run generate-tool-ids\n",
        ));
    }

    let expected_ids = actual
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| line.split_once('\t').map(|(tool_id, _)| tool_id.to_string()))
        .collect::<BTreeSet<_>>();
    let file_ids = governed_container_file_ids(workspace)?;
    let unknown = file_ids
        .difference(&expected_ids)
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        let mut stderr =
            String::from("container filename tool IDs missing from containers/TOOL_IDS.txt:\n");
        for tool_id in unknown {
            stderr.push_str(&tool_id);
            stderr.push('\n');
        }
        return Ok(ContainerCommandOutcome::failure(stderr));
    }
    success_line("tool id manifest: OK")
}

fn generate_tool_name_map_content(workspace: &Workspace) -> Result<String> {
    let rows = registry_tool_map(workspace)?;
    let statuses = governed_container_statuses(workspace)?;
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dev-dna -- containers run generate-tool-name-map -->".to_string(),
        String::new(),
        "# Tool Name Mapping".to_string(),
        String::new(),
        "| Tool ID | Expected Binary | Status |".to_string(),
        "|---|---|---|".to_string(),
    ];
    for (tool_id, status) in statuses {
        let row = rows.get(&tool_id).cloned().unwrap_or_default();
        let expected_bin = row
            .get("expected_bin")
            .and_then(toml::Value::as_str)
            .unwrap_or(&tool_id);
        lines.push(format!(
            "| `{tool_id}` | `{}` | `{status}` |",
            expected_bin.trim()
        ));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn generate_tool_name_map(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dev-dna -- containers run generate-tool-name-map -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(
        workspace,
        args,
        "containers/docs/TOOL_NAME_MAP.md",
        usage,
    )?;
    write_utf8(&out, &generate_tool_name_map_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

fn check_tool_name_map_generated(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let target = workspace.path("containers/docs/TOOL_NAME_MAP.md");
    if read_utf8(&target)? != generate_tool_name_map_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(
            "tool name map drift: regenerate with cargo run -p bijux-dev-dna -- containers run generate-tool-name-map\n",
        ));
    }
    success_line("tool name map generated: OK")
}

fn generate_container_index_content(workspace: &Workspace) -> Result<String> {
    let tool_ids_path = workspace.path("containers/TOOL_IDS.txt");
    if !tool_ids_path.is_file() {
        return Err(anyhow!("missing {}", tool_ids_path.display()));
    }
    let mut rows = Vec::new();
    for line in read_utf8(&tool_ids_path)?.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((tool_id, status)) = line.split_once('\t') else {
            return Err(anyhow!("invalid TOOL_IDS row: {line}"));
        };
        let apptainer = workspace.path(&format!("containers/apptainer/lunarc/{tool_id}.def"));
        let docker_arm64 = workspace.path(&format!("containers/docker/arm64/Dockerfile.{tool_id}"));
        let docker_amd64 = workspace.path(&format!("containers/docker/amd64/Dockerfile.{tool_id}"));
        let apptainer_source = if apptainer.exists() {
            if read_utf8(&apptainer).unwrap_or_default().contains("NON_BIJUX_SOURCES.md")
                || tool_id == "bcftools"
                || tool_id == "beagle"
                || tool_id == "eagle"
                || tool_id == "eigensoft"
                || tool_id == "germline"
                || tool_id == "glimpse"
                || tool_id == "ibdhap"
                || tool_id == "ibdne"
                || tool_id == "impute5"
                || tool_id == "minimac4"
                || tool_id == "shapeit5"
            {
                "non-bijux"
            } else {
                "bijux"
            }
        } else {
            "none"
        };
        let docker_source = match (docker_arm64.exists(), docker_amd64.exists()) {
            (true, true) => "arm64+amd64",
            (true, false) => "arm64",
            (false, true) => "amd64",
            (false, false) => "none",
        };
        rows.push((tool_id.to_string(), status.to_string(), apptainer_source.to_string(), docker_source.to_string()));
    }
    let mut lines = vec![
        "# Containers Docs Index".to_string(),
        String::new(),
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: cargo run -p bijux-dev-dna -- containers run generate-index -->".to_string(),
        String::new(),
        "Purpose: Authoritative tool/container index for container governance and CI checks.".to_string(),
        String::new(),
        "## Strict TOC".to_string(),
        "- Entry point: `containers/index.md`".to_string(),
        "- Policy: `containers/docs/PROMOTION_POLICY.md`".to_string(),
        "- Lifecycle: `containers/docs/TOOL_LIFECYCLE.md`".to_string(),
        "- Version authority: `containers/docs/VERSION_AUTHORITY.md`".to_string(),
        "- Lock lifecycle: `containers/docs/LOCK_LIFECYCLE.md`".to_string(),
        "- HPC frontend build authority: `containers/docs/FRONTEND_BUILD_AUTHORITY.md`".to_string(),
        "- Build + style rules: `containers/docs/STYLE.md`".to_string(),
        "- Smoke: `containers/docs/SMOKE_CONTRACT.md`".to_string(),
        "- Lock/versioning: `containers/versions/LOCK.md`".to_string(),
        "- Promotion/demotion: `containers/docs/PROMOTION_POLICY.md`".to_string(),
        "- Network disclosure: `containers/docs/NETWORK_USAGE.md`".to_string(),
        "- Security boundary: `containers/docs/SECURITY_BOUNDARY.md`".to_string(),
        "- Multiarch policy: `containers/docs/MULTIARCH_POLICY.md`".to_string(),
        "- Licenses: `containers/licenses/`".to_string(),
        "- SBOM + vulnerability hooks: `cargo run -p bijux-dev-dna -- containers run check-sbom-artifacts`, `cargo run -p bijux-dev-dna -- containers run check-vuln-hook`".to_string(),
        "- Exceptions: `containers/docker/NONROOT_EXCEPTIONS.md`, `containers/docker/ENTRYPOINT_EXCEPTIONS.md`, `containers/docs/PLANNED.md`".to_string(),
        "- Tool ID contract: `containers/docs/TOOL_IDS_CONTRACT.md`".to_string(),
        String::new(),
        "## Authority".to_string(),
        "- Tool IDs + lifecycle status: `containers/TOOL_IDS.txt` (generated from registry).".to_string(),
        "- Registry SSoT: `configs/ci/registry/tool_registry*.toml` defines tool existence and lifecycle.".to_string(),
        "- Container version metadata: `containers/versions/versions.toml` + `containers/versions/lock.json`.".to_string(),
        "- Non-bijux provenance: `containers/apptainer/lunarc/NON_BIJUX_SOURCES.md`.".to_string(),
        "- Ownership map: `containers/OWNERS.toml`.".to_string(),
        String::new(),
        "## Tool Container Coverage".to_string(),
        "| tool_id | status | apptainer_source | docker_source |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    for (tool_id, status, ap_src, docker_src) in rows {
        lines.push(format!("| `{tool_id}` | `{status}` | `{ap_src}` | `{docker_src}` |"));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn generate_container_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dev-dna -- containers run generate-index -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "containers/docs/index.md", usage)?;
    write_utf8(&out, &generate_container_index_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

fn check_container_index(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let target = workspace.path("containers/docs/index.md");
    if read_utf8(&target)? != generate_container_index_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(
            "containers/docs/index.md drift; regenerate with cargo run -p bijux-dev-dna -- containers run generate-index\n",
        ));
    }
    success_line("containers index: OK")
}

#[derive(Debug, Clone)]
struct LicenseMetadataEntry {
    tool: String,
    kind: String,
    spdx: String,
    upstream_url: String,
    upstream_version: String,
    upstream_checksum: String,
    file_content: String,
}

fn license_metadata_entries(workspace: &Workspace) -> Result<Vec<LicenseMetadataEntry>> {
    let registry = registry_tool_map(workspace)?;
    let versions = tool_versions(workspace)?;
    let mut entries = Vec::new();
    for (tool, _status) in governed_container_statuses(workspace)? {
        let row = registry.get(&tool).cloned().unwrap_or_default();
        let version_row = versions.get(&tool);
        let kind = if is_non_bijux_apptainer_source(workspace, &tool)
            || table_string(&row, "apptainer_def").contains("/non-bijux/")
        {
            "non-bijux".to_string()
        } else {
            "bijux".to_string()
        };
        let source = version_row
            .map(|value| table_string(value, "source"))
            .filter(|value| !value.is_empty())
            .or_else(|| {
                let upstream = table_string(&row, "upstream");
                (!upstream.is_empty()).then_some(upstream)
            })
            .unwrap_or_else(|| "https://example.invalid/unknown-source".to_string());
        let version = version_row
            .map(|value| table_string(value, "version"))
            .filter(|value| !value.is_empty())
            .or_else(|| {
                let registry_version = table_string(&row, "version");
                (!registry_version.is_empty()).then_some(registry_version)
            })
            .unwrap_or_else(|| "unknown".to_string());
        let source_sha = version_row
            .map(|value| table_string(value, "source_sha256"))
            .unwrap_or_default();
        let checksum = if source_sha.len() == 64 {
            format!("sha256:{source_sha}")
        } else {
            format!("sha256:{}", sha256_hex(format!("{tool}:{source}:{version}").as_bytes()))
        };
        let spdx = version_row
            .map(|value| table_string(value, "upstream_license"))
            .filter(|value| !value.is_empty())
            .or_else(|| {
                let license = table_string(&row, "license_ref");
                (!license.is_empty()).then_some(license)
            })
            .unwrap_or_else(|| "NOASSERTION".to_string());
        let file_content = [
            "# schema_version = 1".to_string(),
            "# owner = bijux-dna-platform".to_string(),
            format!("tool_id = \"{tool}\""),
            format!("container_kind = \"{kind}\""),
            format!("spdx = \"{spdx}\""),
            format!("upstream_license_id = \"{spdx}\""),
            format!("upstream_url = \"{source}\""),
            format!("upstream_version = \"{version}\""),
            format!("upstream_checksum = \"{checksum}\""),
            "redistribution_note = \"Redistribution follows upstream license obligations; verify notice/source requirements before release.\"".to_string(),
            format!("citation = \"upstream:{source}\""),
            format!("version = \"{version}\""),
            String::new(),
        ]
        .join("\n");
        entries.push(LicenseMetadataEntry {
            tool,
            kind,
            spdx,
            upstream_url: source,
            upstream_version: version,
            upstream_checksum: checksum,
            file_content,
        });
    }
    Ok(entries)
}

fn generate_license_index_content(entries: &[LicenseMetadataEntry]) -> String {
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dev-dna -- containers run generate-license-metadata -->".to_string(),
        String::new(),
        "# Container License Index".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Defines the generated index of container-related license metadata for registered tools."
            .to_string(),
        String::new(),
        "## Scope".to_string(),
        "Covers tool id, container kind, SPDX identifier, upstream source, version, and checksum evidence.".to_string(),
        String::new(),
        "## Non-goals".to_string(),
        "- Providing legal advice or replacing upstream license texts.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Every containerized tool in registry scope must have a corresponding license metadata row.".to_string(),
        "- Regenerated output is the sole authority for this index document.".to_string(),
        String::new(),
        "| Tool | Kind | SPDX | Upstream | Version | Checksum |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ];
    for entry in entries {
        lines.push(format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
            entry.tool,
            entry.kind,
            entry.spdx,
            entry.upstream_url,
            entry.upstream_version,
            entry.upstream_checksum
        ));
    }
    format!("{}\n", lines.join("\n"))
}

fn generate_license_metadata(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dev-dna -- containers run generate-license-metadata -- [<output-dir> [<index-path>]]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let (out_dir, doc_out) = match args {
        [] => (
            workspace.path("containers/licenses"),
            workspace.path("docs/30-operations/CONTAINER_LICENSE_INDEX.md"),
        ),
        [dir] => (
            path_from_arg(workspace, dir),
            workspace.path("docs/30-operations/CONTAINER_LICENSE_INDEX.md"),
        ),
        [dir, index] => (path_from_arg(workspace, dir), path_from_arg(workspace, index)),
        _ => return Err(anyhow!(usage.to_string())),
    };
    let entries = license_metadata_entries(workspace)?;
    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    let expected_files = entries
        .iter()
        .map(|entry| format!("{}.license.toml", entry.tool))
        .collect::<BTreeSet<_>>();
    for path in fs::read_dir(&out_dir)
        .with_context(|| format!("read {}", out_dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
    {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !expected_files.contains(name) {
            fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
        }
    }
    for entry in &entries {
        write_utf8(
            &out_dir.join(format!("{}.license.toml", entry.tool)),
            &entry.file_content,
        )?;
    }
    write_utf8(&doc_out, &generate_license_index_content(&entries))?;
    Ok(ContainerCommandOutcome::success(format!(
        "generated {}\ngenerated {}\n",
        out_dir.display(),
        doc_out.display()
    )))
}

fn check_license_metadata(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let license_dir = workspace.path("containers/licenses");
    let registry = registry_tool_map(workspace)?;
    let versions = tool_versions(workspace)?;
    let mut errors = Vec::new();
    let governed_statuses = governed_container_statuses(workspace)?;
    let expected_files = governed_statuses
        .keys()
        .map(|tool| format!("{tool}.license.toml"))
        .collect::<BTreeSet<_>>();
    for path in fs::read_dir(&license_dir)
        .with_context(|| format!("read {}", license_dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
    {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !expected_files.contains(name) {
            errors.push(format!(
                "unexpected stale license metadata: {}",
                workspace.rel(&path).display()
            ));
        }
    }
    for tool in governed_statuses.keys() {
        let row = registry.get(tool).cloned().unwrap_or_default();
        let meta = license_dir.join(format!("{tool}.license.toml"));
        if !meta.exists() {
            errors.push(format!("missing {}", workspace.rel(&meta).display()));
            continue;
        }
        let value = load_toml(&meta)?;
        let Some(data) = value.as_table() else {
            errors.push(format!("{} is not a TOML table", workspace.rel(&meta).display()));
            continue;
        };
        for key in [
            "tool_id",
            "container_kind",
            "spdx",
            "upstream_license_id",
            "upstream_url",
            "upstream_version",
            "upstream_checksum",
            "redistribution_note",
            "citation",
            "version",
        ] {
            if table_string(data, key).is_empty() {
                errors.push(format!("{} missing key: {key}", workspace.rel(&meta).display()));
            }
        }
        if table_string(data, "tool_id") != *tool {
            errors.push(format!("{} tool_id mismatch", workspace.rel(&meta).display()));
        }
        let upstream_url = table_string(data, "upstream_url");
        if !(upstream_url.starts_with("http://") || upstream_url.starts_with("https://")) {
            errors.push(format!(
                "{} upstream_url must be URL",
                workspace.rel(&meta).display()
            ));
        }
        let checksum = table_string(data, "upstream_checksum");
        let checksum_ok = checksum.starts_with("sha256:")
            && checksum.len() == "sha256:".len() + 64
            && checksum["sha256:".len()..]
                .chars()
                .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase());
        if !checksum_ok {
            errors.push(format!(
                "{} upstream_checksum must be exact sha256:<64hex>",
                workspace.rel(&meta).display()
            ));
        }
        let redistribution_note = table_string(data, "redistribution_note").to_lowercase();
        if redistribution_note.is_empty()
            || matches!(redistribution_note.as_str(), "unknown" | "n/a")
        {
            errors.push(format!(
                "{} redistribution_note must be explicit",
                workspace.rel(&meta).display()
            ));
        }

        let apptainer_def = table_string(&row, "apptainer_def");
        if apptainer_def.contains("/non-bijux/") || is_non_bijux_apptainer_source(workspace, tool)
        {
            let version_row = versions.get(tool).cloned().unwrap_or_default();
            let source = table_string(&version_row, "source");
            let version = table_string(&version_row, "version");
            if source.is_empty() || source != upstream_url {
                errors.push(format!(
                    "{} non-bijux upstream_url must match versions.toml source",
                    workspace.rel(&meta).display()
                ));
            }
            if version.is_empty() || table_string(data, "upstream_version") != version {
                errors.push(format!(
                    "{} non-bijux upstream_version must match versions.toml version",
                    workspace.rel(&meta).display()
                ));
            }
            if checksum == format!("sha256:{}", "0".repeat(64)) {
                errors.push(format!(
                    "{} non-bijux upstream_checksum must not be placeholder zeros",
                    workspace.rel(&meta).display()
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("license metadata: OK");
    }
    failure_lines("license metadata check failed:", &errors)
}

fn check_license_index_generated(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let target = workspace.path("docs/30-operations/CONTAINER_LICENSE_INDEX.md");
    let expected = generate_license_index_content(&license_metadata_entries(workspace)?);
    if read_utf8(&target)? != expected {
        return Ok(ContainerCommandOutcome::failure(
            "license index drift: regenerate with cargo run -p bijux-dev-dna -- containers run generate-license-metadata\n",
        ));
    }
    success_line("license index generated: OK")
}

fn generate_network_usage_content(workspace: &Workspace) -> Result<String> {
    let mut items = Vec::new();
    let mut recipe_paths = walkdir::WalkDir::new(workspace.path("containers"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| {
            path.extension().and_then(|ext| ext.to_str()) == Some("def")
                || path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("Dockerfile."))
        })
        .collect::<Vec<_>>();
    recipe_paths.sort();
    for path in recipe_paths {
        let text = read_utf8(&path).unwrap_or_default();
        let commands = text
            .lines()
            .filter_map(|line| {
                let normalized = line.trim();
                line_has_network_command(normalized).then_some(normalized.to_string())
            })
            .take(20)
            .collect::<Vec<_>>();
        let mut item = serde_json::Map::new();
        item.insert(
            "commands".to_string(),
            serde_json::Value::Array(
                commands
                    .iter()
                    .cloned()
                    .map(serde_json::Value::String)
                    .collect(),
            ),
        );
        item.insert(
            "network_required".to_string(),
            serde_json::Value::Bool(!commands.is_empty()),
        );
        item.insert(
            "path".to_string(),
            serde_json::Value::String(workspace.rel(&path).display().to_string()),
        );
        items.push(serde_json::Value::Object(item));
    }
    let mut payload = serde_json::Map::new();
    payload.insert("items".to_string(), serde_json::Value::Array(items));
    payload.insert(
        "schema_version".to_string(),
        serde_json::Value::String("bijux.container.network_usage.v1".to_string()),
    );
    json_string_pretty(&serde_json::Value::Object(payload))
}

fn generate_network_usage(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dev-dna -- containers run generate-network-usage -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(
        workspace,
        args,
        "artifacts/containers/network_usage.json",
        usage,
    )?;
    write_utf8(&out, &generate_network_usage_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

fn check_network_disclosure(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let offline = match args {
        [] => false,
        [single] if single == "--offline" => true,
        [single] if single == "--help" || single == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dev-dna -- containers run check-network-disclosure -- [--offline]",
            )
        }
        _ => return Err(anyhow!("Usage: cargo run -p bijux-dev-dna -- containers run check-network-disclosure -- [--offline]")),
    } || std::env::var("BIJUX_OFFLINE").as_deref() == Ok("1");

    let report = std::env::var("ISO_ROOT")
        .map(PathBuf::from)
        .map(|root| root.join("containers/network_usage.json"))
        .unwrap_or_else(|_| workspace.path("artifacts/containers/network_usage.json"));
    write_utf8(&report, &generate_network_usage_content(workspace)?)?;

    let network_doc = workspace.path("containers/docs/NETWORK_USAGE.md");
    if !network_doc.is_file() {
        return Ok(ContainerCommandOutcome::failure(
            "missing containers/docs/NETWORK_USAGE.md\n",
        ));
    }
    let doc = read_utf8(&network_doc)?;
    let tool_ids = read_utf8(&workspace.path("containers/TOOL_IDS.txt"))?
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| line.split_once('\t').map(|(tool_id, _)| tool_id.to_string()))
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    let mut runtime_network_true = Vec::new();
    for tool_id in tool_ids {
        let meta = workspace.path(&format!("containers/network/{tool_id}.network.toml"));
        if !meta.exists() {
            errors.push(format!(
                "missing per-tool network metadata: {}",
                workspace.rel(&meta).display()
            ));
            continue;
        }
        let value = load_toml(&meta)?;
        let Some(data) = value.as_table() else {
            errors.push(format!("{} must contain a TOML table", workspace.rel(&meta).display()));
            continue;
        };
        for key in ["tool_id", "runtime_network", "build_network", "notes"] {
            if !data.contains_key(key) {
                errors.push(format!(
                    "{} missing key '{key}'",
                    workspace.rel(&meta).display()
                ));
            }
        }
        if table_string(data, "tool_id") != tool_id {
            errors.push(format!("{} tool_id mismatch", workspace.rel(&meta).display()));
        }
        if table_bool(data, "runtime_network") {
            runtime_network_true.push(tool_id);
        }
    }
    for tool_id in runtime_network_true {
        if !doc.contains(&format!("`{tool_id}`")) {
            errors.push(format!(
                "containers/docs/NETWORK_USAGE.md must list runtime-network tool `{tool_id}`"
            ));
        }
    }
    if !errors.is_empty() {
        return failure_lines("network disclosure metadata check failed:", &errors);
    }

    if offline {
        let payload = read_json(&report)?;
        let blocked = payload
            .get("items")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter(|row| {
                row.get("network_required")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
            })
            .filter_map(|row| row.get("path").and_then(serde_json::Value::as_str))
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if !blocked.is_empty() {
            return failure_lines(
                "offline mode blocked: network-required container recipes detected:",
                &blocked,
            );
        }
        return Ok(ContainerCommandOutcome::success(
            "network disclosure metadata: OK\nnetwork disclosure/offline: OK\n",
        ));
    }
    Ok(ContainerCommandOutcome::success(
        "network disclosure metadata: OK\nnetwork disclosure: OK\n",
    ))
}

fn tool_docs_content(workspace: &Workspace) -> Result<BTreeMap<String, String>> {
    let versions = tool_versions(workspace)?;
    let mut licenses = BTreeMap::new();
    for path in fs::read_dir(workspace.path("containers/licenses"))
        .with_context(|| format!("read {}", workspace.path("containers/licenses").display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
    {
        if let Some(tool) = path.file_stem().and_then(|name| name.to_str()) {
            let tool = tool.trim_end_matches(".license").to_string();
            if let Some(table) = load_toml(&path)?.as_table() {
                licenses.insert(tool, table.clone());
            }
        }
    }

    let mut network = BTreeMap::new();
    for path in fs::read_dir(workspace.path("containers/network"))
        .with_context(|| format!("read {}", workspace.path("containers/network").display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
    {
        if let Some(tool) = path.file_stem().and_then(|name| name.to_str()) {
            if let Some(table) = load_toml(&path)?.as_table() {
                network.insert(tool.to_string(), table.clone());
            }
        }
    }

    let mut status = BTreeMap::new();
    let artifacts_dir = workspace.path("artifacts/containers");
    if artifacts_dir.is_dir() {
        for path in fs::read_dir(&artifacts_dir)
            .with_context(|| format!("read {}", artifacts_dir.display()))?
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        {
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("summary.json" | "report.json")
            ) {
                continue;
            }
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&read_utf8(&path).unwrap_or_default()) else {
                continue;
            };
            let tool = value
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !tool.is_empty() {
                status.insert(
                    tool,
                    value.get("status")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("unknown")
                        .to_string(),
                );
            }
        }
    }

    let docker_ids = fs::read_dir(workspace.path("containers/docker/arm64"))
        .with_context(|| format!("read {}", workspace.path("containers/docker/arm64").display()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_prefix("Dockerfile."))
                .map(ToString::to_string)
        })
        .collect::<BTreeSet<_>>();
    let apptainer_ids = fs::read_dir(workspace.path("containers/apptainer/lunarc"))
        .with_context(|| format!("read {}", workspace.path("containers/apptainer/lunarc").display()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| entry.path().file_stem().and_then(|name| name.to_str()).map(ToString::to_string))
        .collect::<BTreeSet<_>>();

    let mut outputs = BTreeMap::new();
    let mut index_lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: cargo run -p bijux-dev-dna -- containers run generate-tool-docs -->"
            .to_string(),
        "# Tool Docs Index".to_string(),
        String::new(),
    ];
    for (tool, version_row) in &versions {
        let license_row = licenses.get(tool).cloned().unwrap_or_default();
        let network_row = network.get(tool).cloned().unwrap_or_default();
        let mut runtimes = Vec::new();
        if docker_ids.contains(tool) {
            runtimes.push("docker-arm64");
        }
        if apptainer_ids.contains(tool) {
            runtimes.push("apptainer");
        }
        let mut limitations = Vec::new();
        if table_bool(&network_row, "runtime_network") {
            limitations.push("Runtime network access required.".to_string());
        }
        if runtimes.is_empty() {
            limitations.push("No runtime implementation currently present.".to_string());
        }
        if limitations.is_empty() {
            limitations.push("No declared limitations.".to_string());
        }
        let mut lines = vec![
            "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
            "<!-- source: cargo run -p bijux-dev-dna -- containers run generate-tool-docs -->"
                .to_string(),
            format!("# {tool}"),
            String::new(),
            "Purpose: generated per-tool container contract summary.".to_string(),
            String::new(),
            format!("- Version: `{}`", table_string(version_row, "version")),
            format!(
                "- License: `{}`",
                {
                    let spdx = table_string(&license_row, "spdx");
                    if spdx.is_empty() {
                        let upstream = table_string(&license_row, "upstream_license");
                        if upstream.is_empty() {
                            "unknown".to_string()
                        } else {
                            upstream
                        }
                    } else {
                        spdx
                    }
                }
            ),
            format!(
                "- Runtime support: `{}`",
                if runtimes.is_empty() {
                    "none".to_string()
                } else {
                    runtimes.join(", ")
                }
            ),
            format!(
                "- Smoke status: `{}`",
                status
                    .get(tool)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string())
            ),
            String::new(),
            "## Known Limitations".to_string(),
        ];
        for limitation in limitations {
            lines.push(format!("- {limitation}"));
        }
        outputs.insert(format!("{tool}.md"), format!("{}\n", lines.join("\n")));
        index_lines.push(format!("- `{tool}`: `containers/docs/tools/{tool}.md`"));
    }
    outputs.insert("index.md".to_string(), format!("{}\n", index_lines.join("\n")));
    Ok(outputs)
}

fn generate_tool_docs(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dev-dna -- containers run generate-tool-docs -- [<output-dir>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out_dir = match args {
        [] => workspace.path("containers/docs/tools"),
        [dir] => path_from_arg(workspace, dir),
        _ => return Err(anyhow!(usage.to_string())),
    };
    fs::create_dir_all(&out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    let outputs = tool_docs_content(workspace)?;
    let expected_files = outputs.keys().cloned().collect::<BTreeSet<_>>();
    for path in fs::read_dir(&out_dir)
        .with_context(|| format!("read {}", out_dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
    {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !expected_files.contains(name) {
            fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
        }
    }
    for (name, content) in outputs {
        write_utf8(&out_dir.join(name), &content)?;
    }
    success_line(format!("generated tool docs under {}", out_dir.display()))
}

fn check_tool_docs_generated(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let expected = tool_docs_content(workspace)?;
    let target_dir = workspace.path("containers/docs/tools");
    let mut actual = BTreeMap::new();
    for path in fs::read_dir(&target_dir)
        .with_context(|| format!("read {}", target_dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
    {
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            actual.insert(name.to_string(), read_utf8(&path)?);
        }
    }
    if actual != expected {
        return Ok(ContainerCommandOutcome::failure(
            "tool docs drift: regenerate with cargo run -p bijux-dev-dna -- containers run generate-tool-docs\n",
        ));
    }
    success_line("tool docs generated: OK")
}

fn load_summary_status(
    workspace: &Workspace,
) -> Result<(
    BTreeMap<String, String>,
    BTreeMap<String, String>,
    BTreeMap<String, String>,
)> {
    let summary_json = workspace.path("artifacts/containers/summary.json");
    let lock_json = workspace.path("containers/versions/lock.json");
    let mut status_from_summary = BTreeMap::new();
    let mut docker_digest_from_summary = BTreeMap::new();
    let mut apptainer_digest_from_summary = BTreeMap::new();
    if summary_json.exists() {
        if let Ok(payload) = read_json(&summary_json) {
            if let Some(items) = payload.get("items").and_then(serde_json::Value::as_array) {
                for item in items {
                    let tool = item
                        .get("tool")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    let runtime = item
                        .get("runtime")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    let status = item
                        .get("status")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    let digest = item
                        .get("resolved_image_digest")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if tool.is_empty() {
                        continue;
                    }
                    if runtime == "apptainer" {
                        if !status.is_empty() {
                            status_from_summary.insert(tool.clone(), status);
                        }
                        if !digest.is_empty() {
                            apptainer_digest_from_summary.insert(tool.clone(), digest);
                        }
                    } else if runtime == "docker-arm64" && !digest.is_empty() {
                        docker_digest_from_summary.insert(tool.clone(), digest);
                    }
                }
            }
        }
    }
    let mut lock_docker_digest = BTreeMap::new();
    let mut lock_apptainer_digest = BTreeMap::new();
    if lock_json.exists() {
        if let Ok(payload) = read_json(&lock_json) {
            if let Some(items) = payload.get("items").and_then(serde_json::Value::as_array) {
                for item in items {
                    let tool = item
                        .get("tool")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if tool.is_empty() {
                        continue;
                    }
                    let docker_digest = item
                        .get("resolved_image_digest")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if !docker_digest.is_empty() {
                        lock_docker_digest.insert(tool.clone(), docker_digest);
                    }
                    let apptainer_digest = item
                        .get("sif_digest_sha256")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if !apptainer_digest.is_empty() {
                        lock_apptainer_digest.insert(tool, apptainer_digest);
                    }
                }
            }
        }
    }
    for (tool, digest) in lock_docker_digest {
        docker_digest_from_summary.entry(tool).or_insert(digest);
    }
    for (tool, digest) in lock_apptainer_digest {
        apptainer_digest_from_summary.entry(tool).or_insert(digest);
    }
    Ok((
        status_from_summary,
        docker_digest_from_summary,
        apptainer_digest_from_summary,
    ))
}

fn generate_qa_matrix_content(workspace: &Workspace) -> Result<String> {
    let registry = registry_tool_map(workspace)?;
    let (
        status_from_summary,
        docker_digest_from_summary,
        apptainer_digest_from_summary,
    ) = load_summary_status(workspace)?;
    let mut rows = Vec::new();
    for (tool, row) in registry {
        if !table_array_strings(&row, "runtimes")
            .iter()
            .any(|runtime| runtime == "apptainer")
        {
            continue;
        }
        rows.push((
            tool.clone(),
            table_string(&row, "apptainer_def"),
            table_string(&row, "smoke_version_cmd"),
            table_string(&row, "smoke_help_cmd"),
            table_string(&row, "smoke_minimal_cmd"),
            {
                let exit_code = table_string(&row, "smoke_minimal_exit_code");
                if exit_code.is_empty() {
                    "0".to_string()
                } else {
                    exit_code
                }
            },
            {
                let rationale = table_string(&row, "smoke_minimal_rationale");
                if rationale.is_empty() {
                    "minimal command contract".to_string()
                } else {
                    rationale
                }
            },
            status_from_summary
                .get(&tool)
                .cloned()
                .unwrap_or_else(|| {
                    let status = table_string(&row, "status");
                    if status.is_empty() {
                        "unknown".to_string()
                    } else {
                        status
                    }
                }),
            docker_digest_from_summary
                .get(&tool)
                .cloned()
                .unwrap_or_else(|| "-".to_string()),
            apptainer_digest_from_summary
                .get(&tool)
                .cloned()
                .unwrap_or_else(|| "-".to_string()),
        ));
    }

    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dev-dna -- containers run generate-qa-matrix -->".to_string(),
        String::new(),
        "# APPTAINER_QA_MATRIX".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Generated matrix for Apptainer-enabled tools and required QA gates.".to_string(),
        String::new(),
        "## Scope".to_string(),
        "Derived from tool registries and container metadata fields.".to_string(),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing full per-tool smoke manifests.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Tool row exists iff registry runtimes include `apptainer`.".to_string(),
        "- `apptainer_def` and smoke command fields are surfaced for QA checks.".to_string(),
        String::new(),
        "| Tool ID | Apptainer Def | Smoke Version | Smoke Help | Smoke Minimal | Minimal Exit | Docker Digest | Apptainer Digest | Minimal Rationale | QA Rule | Status |".to_string(),
        "|---|---|---|---|---|---|---|---|---|---|---|".to_string(),
    ];
    for (
        tool,
        apptainer_def,
        smoke_version_cmd,
        smoke_help_cmd,
        smoke_minimal_cmd,
        smoke_minimal_exit_code,
        smoke_minimal_rationale,
        status,
        docker_digest,
        apptainer_digest,
    ) in rows
    {
        lines.push(format!(
            "| `{tool}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `build+smoke required` | `{}` |",
            if apptainer_def.is_empty() { "-" } else { &apptainer_def },
            if smoke_version_cmd.is_empty() { "-".to_string() } else { markdown_code_value(&smoke_version_cmd) }.as_str(),
            if smoke_help_cmd.is_empty() { "-".to_string() } else { markdown_code_value(&smoke_help_cmd) }.as_str(),
            if smoke_minimal_cmd.is_empty() { "-".to_string() } else { markdown_code_value(&smoke_minimal_cmd) }.as_str(),
            smoke_minimal_exit_code,
            docker_digest,
            apptainer_digest,
            markdown_code_value(&smoke_minimal_rationale),
            status,
        ));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn generate_qa_matrix(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dev-dna -- containers run generate-qa-matrix -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = match args {
        [] => workspace.path("docs/30-operations/APPTAINER_QA_MATRIX.md"),
        [path] if path.starts_with('-') => {
            return Ok(ContainerCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: format!("refusing unsafe output path (starts with '-'): {path}\n"),
            })
        }
        [path] => path_from_arg(workspace, path),
        _ => return Err(anyhow!(usage.to_string())),
    };
    write_utf8(&out, &generate_qa_matrix_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

fn check_qa_matrix_generated(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let target = workspace.path("docs/30-operations/APPTAINER_QA_MATRIX.md");
    if read_utf8(&target)? != generate_qa_matrix_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(
            "qa matrix drift: regenerate with cargo run -p bijux-dev-dna -- containers run generate-qa-matrix\n",
        ));
    }
    success_line("qa matrix generated: OK")
}

#[derive(Serialize)]
struct VersionMapItem {
    tool: String,
    version: String,
    status: String,
    source: String,
    source_sha256: String,
    pinned_commit: String,
    date_pinned: String,
}

fn extract_version_map_content(workspace: &Workspace) -> Result<String> {
    let versions = tool_versions(workspace)?;
    let items = versions
        .into_iter()
        .map(|(tool, row)| VersionMapItem {
            tool,
            version: row.get("version").and_then(toml::Value::as_str).unwrap_or_default().to_string(),
            status: row.get("status").and_then(toml::Value::as_str).unwrap_or("production").to_string(),
            source: row.get("source").and_then(toml::Value::as_str).unwrap_or_default().to_string(),
            source_sha256: row.get("source_sha256").and_then(toml::Value::as_str).unwrap_or_default().to_string(),
            pinned_commit: row.get("pinned_commit").and_then(toml::Value::as_str).unwrap_or_default().to_string(),
            date_pinned: row.get("date_pinned").and_then(toml::Value::as_str).unwrap_or_default().to_string(),
        })
        .collect::<Vec<_>>();
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "bijux.container.version_map.v1",
            "source": "containers/versions/versions.toml",
            "items": items,
        }))?
    ))
}

fn extract_version_map(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dev-dna -- containers run extract-version-map -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "artifacts/containers/version_map.json", usage)?;
    write_utf8(&out, &extract_version_map_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

fn generate_versions_index_sha_content(workspace: &Workspace) -> Result<String> {
    let versions_dir = workspace.path("containers/versions");
    let mut rows = Vec::new();
    for entry in fs::read_dir(&versions_dir)
        .with_context(|| format!("read {}", versions_dir.display()))?
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        if name == "index.sha256" {
            continue;
        }
        let digest = sha256_hex(&fs::read(&path).with_context(|| format!("read {}", path.display()))?);
        rows.push((name.to_string(), digest));
    }
    rows.sort();
    let payload = rows
        .into_iter()
        .map(|(name, digest)| format!("{digest}  {name}"))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!("{payload}\n"))
}

fn generate_versions_index_sha(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dev-dna -- containers run generate-versions-index-sha -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "containers/versions/index.sha256", usage)?;
    write_utf8(&out, &generate_versions_index_sha_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

fn check_versions_index_sha(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let expected = workspace.path("containers/versions/index.sha256");
    if read_utf8(&expected)? != generate_versions_index_sha_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(
            "versions index sha drift: regenerate with cargo run -p bijux-dev-dna -- containers run generate-versions-index-sha\n",
        ));
    }
    success_line("versions index sha: OK")
}

fn generate_version_lock_content(workspace: &Workspace) -> Result<String> {
    let version_map: serde_json::Value =
        serde_json::from_str(&extract_version_map_content(workspace)?)?;
    let generator_path = workspace.path("crates/bijux-dev-dna/src/native/containers.rs");
    let versions_path = workspace.path("containers/versions/versions.toml");

    let manifest_candidates = [
        workspace.path("artifacts/containers"),
        workspace.path("artifacts/containers/manifests"),
    ];
    let mut docker_digest_by_tool = BTreeMap::new();
    let mut apptainer_sif_sha256_by_tool = BTreeMap::new();
    let mut frontend_sif_sha256_by_tool = BTreeMap::new();
    let mut frontend_smoke_version_output_sha256_by_tool = BTreeMap::new();
    let mut size_by_tool = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for base in manifest_candidates {
        if !base.exists() {
            continue;
        }
        for entry in fs::read_dir(&base)
            .with_context(|| format!("read {}", base.display()))?
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
            if matches!(name, "lock.json" | "summary.json" | "report.json") || !seen.insert(path.clone()) {
                continue;
            }
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&read_utf8(&path).unwrap_or_default()) else {
                continue;
            };
            let tool = value.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default().trim().to_string();
            let runtime = value.get("runtime").and_then(serde_json::Value::as_str).unwrap_or_default().trim().to_string();
            let digest = value.get("resolved_image_digest").and_then(serde_json::Value::as_str).unwrap_or_default().trim().to_string();
            let size = value.get("image_size_bytes").and_then(serde_json::Value::as_i64).unwrap_or(0);
            if tool.is_empty() {
                continue;
            }
            if runtime.starts_with("docker") {
                docker_digest_by_tool.insert(tool.clone(), digest);
            } else if runtime == "apptainer" {
                apptainer_sif_sha256_by_tool.insert(tool.clone(), digest);
            }
            if size > 0 {
                size_by_tool.insert(tool, size);
            }
        }
    }

    let frontend_digests = workspace.path("artifacts/containers/hpc/frontend-sif-digests.json");
    if frontend_digests.is_file() {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&read_utf8(&frontend_digests)?) {
            if let Some(items) = value.get("items").and_then(serde_json::Value::as_array) {
                for row in items {
                    let tool = row.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default().trim();
                    let sha = row.get("sha256").and_then(serde_json::Value::as_str).unwrap_or_default().trim();
                    if !tool.is_empty() && !sha.is_empty() {
                        frontend_sif_sha256_by_tool.insert(tool.to_string(), sha.to_string());
                    }
                }
            }
        }
    }

    let frontend_smoke_summary = workspace.path("artifacts/containers/hpc/frontend-smoke/summary.json");
    if frontend_smoke_summary.is_file() {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&read_utf8(&frontend_smoke_summary)?) {
            if let Some(items) = value.get("items").and_then(serde_json::Value::as_array) {
                for row in items {
                    let tool = row.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default().trim();
                    let output = row
                        .get("normalized_version_output")
                        .and_then(serde_json::Value::as_str)
                        .or_else(|| row.get("version_output").and_then(serde_json::Value::as_str))
                        .unwrap_or_default()
                        .trim()
                        .to_lowercase();
                    if !tool.is_empty() && !output.is_empty() {
                        frontend_smoke_version_output_sha256_by_tool
                            .insert(tool.to_string(), sha256_hex(output.as_bytes()));
                    }
                }
            }
        }
    }

    let mut items = Vec::new();
    for row in version_map.get("items").and_then(serde_json::Value::as_array).cloned().unwrap_or_default() {
        let tool = row.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
        let canonical = serde_json::to_string(&row)?;
        items.push(serde_json::json!({
            "tool": tool,
            "version": row.get("version").and_then(serde_json::Value::as_str).unwrap_or_default(),
            "status": row.get("status").and_then(serde_json::Value::as_str).unwrap_or_default(),
            "source": row.get("source").and_then(serde_json::Value::as_str).unwrap_or_default(),
            "source_sha256": row.get("source_sha256").and_then(serde_json::Value::as_str).unwrap_or_default(),
            "pinned_commit": row.get("pinned_commit").and_then(serde_json::Value::as_str).unwrap_or_default(),
            "resolved_image_digest": docker_digest_by_tool.get(&tool).cloned().unwrap_or_default(),
            "resolved_sif_sha256": apptainer_sif_sha256_by_tool.get(&tool).cloned().unwrap_or_default(),
            "sif_digest_sha256": apptainer_sif_sha256_by_tool.get(&tool).cloned().unwrap_or_default(),
            "frontend_resolved_sif_sha256": frontend_sif_sha256_by_tool.get(&tool).cloned().unwrap_or_default(),
            "frontend_sif_digest_sha256": frontend_sif_sha256_by_tool.get(&tool).cloned().unwrap_or_default(),
            "frontend_smoke_version_output_sha256": frontend_smoke_version_output_sha256_by_tool.get(&tool).cloned().unwrap_or_default(),
            "image_size_bytes": size_by_tool.get(&tool).copied().unwrap_or(0),
            "entry_sha256": sha256_hex(canonical.as_bytes()),
        }));
    }

    let output = serde_json::json!({
        "schema_version": "bijux.container.version_lock.v3",
        "source": "containers/versions/versions.toml",
        "version_map_source": "artifacts/containers/version_map.json",
        "build_manifests_source": "artifacts/containers/manifests/*.json",
        "build_date_utc": git_last_modified_timestamp(workspace, "containers/versions/versions.toml"),
        "builder_platform": "arm64",
        "generator_script": "cargo run -p bijux-dev-dna -- containers run generate-version-lock",
        "generator_sha256": sha256_hex(&fs::read(&generator_path).with_context(|| format!("read {}", generator_path.display()))?),
        "source_sha256": sha256_hex(&fs::read(&versions_path).with_context(|| format!("read {}", versions_path.display()))?),
        "items": items,
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&output)?))
}

fn generate_version_lock(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dev-dna -- containers run generate-version-lock -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "containers/versions/lock.json", usage)?;
    write_utf8(&out, &generate_version_lock_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

fn check_version_lock(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let lock = workspace.path("containers/versions/lock.json");
    if read_utf8(&lock)? != generate_version_lock_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(
            "version lock drift: regenerate with cargo run -p bijux-dev-dna -- containers run generate-version-lock\n",
        ));
    }
    success_line("version lock: OK")
}

fn check_version_authority(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let violations = std::process::Command::new("find")
        .arg(workspace.path("containers"))
        .args(["-type", "f", "(", "-iname", "*version*", "-o", "-iname", "*lock*", ")"])
        .output()
        .with_context(|| "scan container version/lock files".to_string())?;
    let listing = String::from_utf8_lossy(&violations.stdout);
    let forbidden = listing
        .lines()
        .map(|line| workspace.rel(&PathBuf::from(line)).display().to_string())
        .filter(|rel| rel.starts_with("containers/"))
        .filter(|rel| !rel.starts_with("containers/docs/"))
        .filter(|rel| {
            !matches!(
                rel.as_str(),
                "containers/versions/versions.toml"
                    | "containers/versions/lock.json"
                    | "containers/versions/LOCK.md"
                    | "containers/versions/index.md"
            )
        })
        .collect::<Vec<_>>();
    if !forbidden.is_empty() {
        let mut stderr =
            String::from("non-canonical version/lock files found under containers/ (use containers/versions/* only):\n");
        stderr.push_str(&forbidden.join("\n"));
        stderr.push('\n');
        return Ok(ContainerCommandOutcome::failure(stderr));
    }

    let lock: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path("containers/versions/lock.json"))?)?;
    let versions_path = workspace.path("containers/versions/versions.toml");
    let generator_path = workspace.path("crates/bijux-dev-dna/src/native/containers.rs");
    let mut errors = Vec::new();
    if lock
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .is_none_or(|value| value != "bijux.container.version_lock.v3")
    {
        errors.push("- lock.json schema_version must be bijux.container.version_lock.v3".to_string());
    }
    if lock.get("source").and_then(serde_json::Value::as_str) != Some("containers/versions/versions.toml") {
        errors.push("- lock.json source must be containers/versions/versions.toml".to_string());
    }
    if lock
        .get("build_date_utc")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        errors.push("- lock.json must include build_date_utc".to_string());
    }
    if lock.get("builder_platform").and_then(serde_json::Value::as_str) != Some("arm64") {
        errors.push("- lock.json builder_platform must be arm64".to_string());
    }
    if lock.get("generator_script").and_then(serde_json::Value::as_str)
        != Some("cargo run -p bijux-dev-dna -- containers run generate-version-lock")
    {
        errors.push("- lock.json generator_script must reference bijux-dev-dna".to_string());
    }
    let expected_gen_sha =
        sha256_hex(&fs::read(&generator_path).with_context(|| format!("read {}", generator_path.display()))?);
    if lock.get("generator_sha256").and_then(serde_json::Value::as_str) != Some(expected_gen_sha.as_str()) {
        errors.push("- lock.json generator_sha256 does not match bijux-dev-dna container generator".to_string());
    }
    let expected_sha =
        sha256_hex(&fs::read(&versions_path).with_context(|| format!("read {}", versions_path.display()))?);
    if lock.get("source_sha256").and_then(serde_json::Value::as_str) != Some(expected_sha.as_str()) {
        errors.push("- lock.json source_sha256 does not match versions.toml".to_string());
    }
    if lock.get("items").and_then(serde_json::Value::as_array).is_none_or(|items| items.is_empty()) {
        errors.push("- lock.json items must be a non-empty list".to_string());
    }

    let version_source_marker = "VERSION_SOURCE: containers/versions/versions.toml";
    for root in [
        workspace.path("containers/apptainer"),
        workspace.path("containers/docker/arm64"),
    ] {
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let ext = entry.path().extension().and_then(|ext| ext.to_str());
            let file_name = entry.path().file_name().and_then(|name| name.to_str()).unwrap_or_default();
            if ext != Some("def") && !file_name.starts_with("Dockerfile.") {
                continue;
            }
            let raw = read_utf8(entry.path()).unwrap_or_default();
            if !raw.contains(version_source_marker) {
                errors.push(format!(
                    "- version authority: missing VERSION_SOURCE marker in {}",
                    workspace.rel(entry.path()).display()
                ));
            }
        }
    }

    if errors.is_empty() {
        return success_line("version authority: OK");
    }
    failure_lines("version authority check failed:", &errors)
}

fn summary(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let mut json_out = None::<PathBuf>;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                if let Some(value) = args.get(index + 1).filter(|value| !value.starts_with("--")) {
                    json_out = Some(path_from_arg(workspace, value));
                    index += 2;
                } else {
                    json_out = Some(workspace.path("artifacts/containers/summary.json"));
                    index += 1;
                }
            }
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dev-dna -- containers run summary -- [--json <output-path>]",
                );
            }
            other => {
                return Ok(ContainerCommandOutcome {
                    exit_code: 2,
                    stdout: String::new(),
                    stderr: format!("unknown arg: {other}\n"),
                });
            }
        }
    }

    let manifest_dir = std::env::var("MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace.path("artifacts/containers"));
    if !manifest_dir.is_dir() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!("no manifests found: {}\n", manifest_dir.display()),
        });
    }

    let mut rows = Vec::new();
    for entry in fs::read_dir(&manifest_dir)
        .with_context(|| format!("read {}", manifest_dir.display()))?
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let Ok(data) = serde_json::from_str::<serde_json::Value>(&read_utf8(&path).unwrap_or_default()) else {
            continue;
        };
        let tool = data.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
        let runtime = data.get("runtime").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
        let status = data.get("status").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
        if tool.is_empty() || runtime.is_empty() {
            continue;
        }
        let log = manifest_dir.join(format!("logs/{runtime}/{tool}.log"));
        rows.push(serde_json::json!({
            "tool": tool,
            "runtime": runtime,
            "status": status,
            "log": log.display().to_string(),
            "manifest": path.display().to_string(),
            "declared_version": data.get("declared_version").cloned().unwrap_or(serde_json::Value::Null),
            "version_output": data.get("version_output").cloned().unwrap_or(serde_json::Value::Null),
            "normalized_version_output": data.get("normalized_version_output").cloned().unwrap_or(serde_json::Value::Null),
            "resolved_image_digest": data.get("resolved_image_digest").cloned().unwrap_or(serde_json::Value::Null),
            "sif_digest_sha256": data.get("sif_digest_sha256").cloned().unwrap_or(serde_json::Value::Null),
            "image_size_bytes": data.get("image_size_bytes").cloned().unwrap_or(serde_json::Value::Null),
            "packages_hash": data.get("packages_hash").cloned().unwrap_or(serde_json::Value::Null),
            "sbom_path": data.get("sbom_path").cloned().unwrap_or(serde_json::Value::Null),
            "self_report_path": data.get("self_report_path").cloned().unwrap_or(serde_json::Value::Null),
            "smoke_log_path": data.get("smoke_log_path").cloned().unwrap_or(serde_json::Value::Null),
            "smoke_log_dir": data.get("smoke_log_dir").cloned().unwrap_or(serde_json::Value::Null),
        }));
    }
    rows.sort_by(|left, right| {
        let left_key = (
            left.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default(),
            left.get("runtime").and_then(serde_json::Value::as_str).unwrap_or_default(),
        );
        let right_key = (
            right.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default(),
            right.get("runtime").and_then(serde_json::Value::as_str).unwrap_or_default(),
        );
        left_key.cmp(&right_key)
    });
    let mut stdout = String::from("tool\truntime\tresult\tlog\n");
    for row in &rows {
        stdout.push_str(row.get("tool").and_then(serde_json::Value::as_str).unwrap_or_default());
        stdout.push('\t');
        stdout.push_str(row.get("runtime").and_then(serde_json::Value::as_str).unwrap_or_default());
        stdout.push('\t');
        stdout.push_str(row.get("status").and_then(serde_json::Value::as_str).unwrap_or_default());
        stdout.push('\t');
        stdout.push_str(row.get("log").and_then(serde_json::Value::as_str).unwrap_or_default());
        stdout.push('\n');
    }
    if let Some(json_out_path) = json_out {
        let payload = serde_json::json!({
            "schema_version": "bijux.container.summary.v1",
            "items": rows,
        });
        write_utf8(&json_out_path, &format!("{}\n", serde_json::to_string_pretty(&payload)?))?;
    }
    Ok(ContainerCommandOutcome::success(stdout))
}

fn run_env_prep(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("env-prep", args)?;
    let container_type = checked_container_type()?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let mut argv = bijux_command_prefix();
    argv.extend([
        "environment".to_string(),
        "prep".to_string(),
        container_type,
    ]);
    if !stage.is_empty() {
        argv.push("--stage".to_string());
        argv.push(stage);
    } else {
        argv.push(tools);
    }
    run_argv(workspace, &argv)
}

fn run_env_smoke(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("env-smoke", args)?;
    let container_type = checked_container_type()?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let mut argv = bijux_command_prefix();
    argv.extend([
        "environment".to_string(),
        "smoke".to_string(),
        container_type,
    ]);
    if !stage.is_empty() {
        argv.push("--stage".to_string());
        argv.push(stage);
    } else {
        argv.push(tools);
    }
    run_argv(workspace, &argv)
}

fn run_container_smoke(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("container-smoke", args)?;
    let tools = env_or_empty("TOOLS");
    let stage = env_or_empty("STAGE");
    require_tools_or_stage(&tools, &stage)?;
    let prep = run_env_prep(workspace, &[])?;
    if !prep.is_success() {
        return Ok(prep);
    }
    let smoke = run_env_smoke(workspace, &[])?;
    Ok(merge_outcomes(prep, smoke))
}

fn run_containers_smoke(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("containers-smoke", args)?;
    checked_container_type()?;
    let list = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec!["registry".to_string(), "list-stages".to_string()],
        ]
        .concat(),
    )?;
    if !list.is_success() {
        return Ok(list);
    }
    let mut aggregate = ContainerCommandOutcome::success(String::new());
    for stage in list
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let header = format!("== stage {stage}\n");
        aggregate.stdout.push_str(&header);
        let prep = run_argv(
            workspace,
            &[
                bijux_command_prefix(),
                vec![
                    "environment".to_string(),
                    "prep".to_string(),
                    checked_container_type()?,
                    "--stage".to_string(),
                    stage.to_string(),
                ],
            ]
            .concat(),
        )?;
        aggregate = merge_outcomes(aggregate, prep.clone());
        if !prep.is_success() {
            return Ok(aggregate);
        }
        let smoke = run_argv(
            workspace,
            &[
                bijux_command_prefix(),
                vec![
                    "environment".to_string(),
                    "smoke".to_string(),
                    checked_container_type()?,
                    "--stage".to_string(),
                    stage.to_string(),
                ],
            ]
            .concat(),
        )?;
        aggregate = merge_outcomes(aggregate, smoke.clone());
        if !smoke.is_success() {
            return Ok(aggregate);
        }
    }
    Ok(aggregate)
}

fn run_build_contract(workspace: &Workspace, tools_csv: &str) -> Result<ContainerCommandOutcome> {
    let container_type = checked_container_type()?;
    if container_type == "apptainer" {
        smoke_runtime_script(
            workspace,
            "scripts/containers/smoke-apptainer.sh",
            &[
                ("TOOLS", tools_csv.to_string()),
                ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                ("SMOKE_LEVEL", "build".to_string()),
                (
                    "ARTIFACT_DIR",
                    format!("{}/apptainer", container_artifact_dir()),
                ),
            ],
        )
    } else {
        smoke_runtime_script(
            workspace,
            "scripts/containers/smoke-docker-arm64.sh",
            &[
                ("TOOLS", tools_csv.to_string()),
                ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                ("SMOKE_LEVEL", "build".to_string()),
                ("SAVE_TAR", "0".to_string()),
                (
                    "ARTIFACT_DIR",
                    format!("{}/docker-arm64", container_artifact_dir()),
                ),
            ],
        )
    }
}

fn run_test_images(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("test-images", args)?;
    let container_type = checked_container_type()?;
    let stage = env_or_empty("STAGE");
    let tools = env_or_empty("TOOLS");
    if container_type == "docker-arm64" {
        let tools_csv = if !stage.is_empty() {
            list_tools_for_stage(workspace, &stage)?
        } else if !tools.is_empty() {
            tools
        } else {
            primary_tools_csv(workspace)?
        };
        return smoke_runtime_script(
            workspace,
            "scripts/containers/smoke-docker-arm64.sh",
            &[
                ("TOOLS", tools_csv),
                ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                ("SMOKE_LEVEL", "contract".to_string()),
                ("SAVE_TAR", "0".to_string()),
                ("ARTIFACT_DIR", container_artifact_dir()),
            ],
        );
    }
    if !stage.is_empty() {
        return run_env_smoke(workspace, &[]);
    }
    if !tools.is_empty() {
        return run_env_smoke(workspace, &[]);
    }
    run_containers_smoke(workspace, &[])
}

fn run_test_images_stage(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("test-images-stage", args)?;
    if env_or_empty("STAGE").is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set STAGE=<domain.stage|stage> (example: STAGE=fastq.trim)\n"
                .to_string(),
        });
    }
    run_env_smoke(workspace, &[])
}

fn run_test_images_tool(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("test-images-tool", args)?;
    if env_or_empty("TOOLS").is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set TOOLS=<tool_id>\n".to_string(),
        });
    }
    run_env_smoke(workspace, &[])
}

fn run_image_smoke_vcf(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("image-smoke-vcf", args)?;
    let stages = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec!["registry".to_string(), "list-stages".to_string()],
        ]
        .concat(),
    )?;
    if !stages.is_success() {
        return Ok(stages);
    }
    let mut tools = BTreeSet::new();
    for stage in stages
        .stdout
        .lines()
        .map(str::trim)
        .filter(|stage| stage.starts_with("vcf."))
    {
        for tool in list_tools_for_stage(workspace, stage)?
            .split(',')
            .map(str::trim)
            .filter(|tool| !tool.is_empty())
        {
            tools.insert(tool.to_string());
        }
    }
    if tools.is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: no VCF tools found via registry stage/tool mapping\n".to_string(),
        });
    }
    let tools_csv = tools.into_iter().collect::<Vec<_>>().join(",");
    if checked_container_type()? == "apptainer" {
        smoke_runtime_script(
            workspace,
            "scripts/containers/smoke-apptainer.sh",
            &[
                ("TOOLS", tools_csv),
                ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                ("ARTIFACT_DIR", container_artifact_dir()),
            ],
        )
    } else {
        smoke_runtime_script(
            workspace,
            "scripts/containers/smoke-docker-arm64.sh",
            &[
                ("TOOLS", tools_csv),
                ("BIJUX_WORKERS", env_or_default("BIJUX_WORKERS", "1")),
                ("JOBS", env_or_default("BIJUX_WORKERS", "1")),
                ("SMOKE_LEVEL", "contract".to_string()),
                ("SAVE_TAR", "0".to_string()),
                ("ARTIFACT_DIR", container_artifact_dir()),
            ],
        )
    }
}

fn run_image_qa(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("image-qa", args)?;
    let container_type = checked_container_type()?;
    if container_type != "docker-arm64" {
        return Ok(ContainerCommandOutcome::success(format!(
            "skip: image-qa is docker-only (CONTAINER_TYPE={container_type})\n"
        )));
    }
    run_program_with_env(
        workspace,
        "./scripts/run.sh",
        &[
            "tooling".to_string(),
            "image-qa".to_string(),
            "--platform".to_string(),
            env_or_default("PLATFORM", "docker-arm64"),
        ],
        &artifact_env(workspace)?,
    )
}

fn run_apptainer_ensure(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    ensure_no_args("apptainer-ensure", args)?;
    let domain = env_or_empty("DOMAIN");
    let stages = env_or_empty("STAGES");
    if domain.is_empty() || stages.is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set DOMAIN=<domain> and STAGES=<comma-separated>\nexample: make apptainer-ensure DOMAIN=fastq STAGES=validate_pre,trim,filter,stats,qc_post\n".to_string(),
        });
    }
    run_bijux_with_env(
        workspace,
        &[
            "env".to_string(),
            "ensure-images".to_string(),
            "--domain".to_string(),
            domain,
            "--stages".to_string(),
            stages,
        ],
        &[(
            "BIJUX_HPC_ROOT",
            env_or_default("BIJUX_HPC_ROOT", "$HOME/bijux"),
        )],
    )
}

fn run_apptainer_ensure_stage(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    ensure_no_args("apptainer-ensure-stage", args)?;
    let domain = env_or_empty("DOMAIN");
    let stages = env_or_empty("STAGES");
    if domain.is_empty() || stages.is_empty() {
        return Ok(ContainerCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "ERROR: set DOMAIN and STAGES for apptainer-ensure-stage\n".to_string(),
        });
    }
    run_bijux_with_env(
        workspace,
        &[
            "env".to_string(),
            "ensure-images".to_string(),
            "--domain".to_string(),
            domain,
            "--stages".to_string(),
            stages,
        ],
        &[(
            "BIJUX_HPC_ROOT",
            env_or_default("BIJUX_HPC_ROOT", "$HOME/bijux"),
        )],
    )
}

fn smoke_runtime_script(
    workspace: &Workspace,
    script: &str,
    overrides: &[(&str, String)],
) -> Result<ContainerCommandOutcome> {
    let mut envs = artifact_env(workspace)?;
    for (key, value) in overrides {
        envs.push(((*key).to_string(), value.clone()));
    }
    run_program_with_env(workspace, &format!("./{script}"), &[], &envs)
}

fn run_bijux_with_env(
    workspace: &Workspace,
    args: &[String],
    overrides: &[(&str, String)],
) -> Result<ContainerCommandOutcome> {
    let mut envs = artifact_env(workspace)?;
    for (key, value) in overrides {
        envs.push(((*key).to_string(), value.clone()));
    }
    let argv = [bijux_command_prefix(), args.to_vec()].concat();
    run_argv_with_env(workspace, &argv, &envs)
}

fn run_argv(workspace: &Workspace, argv: &[String]) -> Result<ContainerCommandOutcome> {
    run_argv_with_env(workspace, argv, &[])
}

fn run_argv_with_env(
    workspace: &Workspace,
    argv: &[String],
    envs: &[(String, String)],
) -> Result<ContainerCommandOutcome> {
    let (program, args) = argv
        .split_first()
        .context("container command requires a program")?;
    run_program_with_env(workspace, program, args, envs)
}

fn run_program_with_env(
    workspace: &Workspace,
    program: &str,
    args: &[String],
    envs: &[(String, String)],
) -> Result<ContainerCommandOutcome> {
    let runner = ProcessRunner::new(workspace);
    let output = runner.run_owned_with_env(program, args, envs)?;
    Ok(ContainerCommandOutcome::from_output(output))
}

fn artifact_env(workspace: &Workspace) -> Result<Vec<(String, String)>> {
    let artifact_root = artifact_root_path(workspace)?;
    let cargo_target_dir = artifact_root.join("target");
    let cargo_home = artifact_root.join("cargo/home");
    let tmpdir = artifact_root.join("tmp");
    for dir in [&artifact_root, &cargo_target_dir, &cargo_home, &tmpdir] {
        std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    }
    Ok(vec![
        (
            "ARTIFACT_ROOT".to_string(),
            artifact_root.display().to_string(),
        ),
        ("ISO_ROOT".to_string(), artifact_root.display().to_string()),
        (
            "CARGO_TARGET_DIR".to_string(),
            cargo_target_dir.display().to_string(),
        ),
        ("CARGO_HOME".to_string(), cargo_home.display().to_string()),
        ("TMPDIR".to_string(), tmpdir.display().to_string()),
        ("TMP".to_string(), tmpdir.display().to_string()),
        ("TEMP".to_string(), tmpdir.display().to_string()),
    ])
}

fn artifact_root_path(workspace: &Workspace) -> Result<PathBuf> {
    let configured = std::env::var("ARTIFACT_ROOT").unwrap_or_else(|_| "artifacts".to_string());
    let path = if PathBuf::from(&configured).is_absolute() {
        PathBuf::from(&configured)
    } else {
        workspace.root.join(&configured)
    };
    let display = path.display().to_string();
    if !display.contains("/artifacts") && !display.ends_with("artifacts") {
        return Err(anyhow!(
            "artifact root must stay under artifacts/: {display}"
        ));
    }
    Ok(path)
}

fn primary_tools_csv(workspace: &Workspace) -> Result<String> {
    let result = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec![
                "registry".to_string(),
                "list-tools".to_string(),
                "--kind".to_string(),
                "primary".to_string(),
            ],
        ]
        .concat(),
    )?;
    if !result.is_success() {
        return Ok(String::new());
    }
    Ok(result
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(","))
}

fn list_tools_for_stage(workspace: &Workspace, stage: &str) -> Result<String> {
    let result = run_argv(
        workspace,
        &[
            bijux_command_prefix(),
            vec![
                "registry".to_string(),
                "list-tools".to_string(),
                "--stage".to_string(),
                stage.to_string(),
                "--kind".to_string(),
                "all".to_string(),
            ],
        ]
        .concat(),
    )?;
    if !result.is_success() {
        return Ok(String::new());
    }
    Ok(result
        .stdout
        .replace(',', "\n")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(","))
}

fn resolve_toolkit_tools(workspace: &Workspace, bundle: &str) -> Result<String> {
    let data: toml::Value = toml::from_str(&std::fs::read_to_string(
        workspace.path("configs/ci/tools/toolkit_bundles.toml"),
    )?)?;
    let tools = data
        .get("bundles")
        .and_then(|value| value.get(bundle))
        .and_then(|value| value.get("tools"))
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if tools.is_empty() {
        return Err(anyhow!("unknown or empty toolkit bundle: {bundle}"));
    }
    Ok(tools
        .into_iter()
        .filter_map(|tool| tool.as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>()
        .join(","))
}

fn ensure_no_args(command: &str, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    Err(anyhow!("{command} does not accept positional arguments"))
}

fn checked_container_type() -> Result<String> {
    let container_type = env_or_default("CONTAINER_TYPE", "docker-arm64");
    match container_type.as_str() {
        "docker-arm64" | "docker-amd64" | "apptainer" => Ok(container_type),
        _ => Err(anyhow!(
            "ERROR: unsupported CONTAINER_TYPE={container_type}\nsupported: docker-arm64 | docker-amd64 | apptainer"
        )),
    }
}

fn require_tools_or_stage(tools: &str, stage: &str) -> Result<()> {
    if tools.is_empty() && stage.is_empty() {
        return Err(anyhow!("ERROR: set TOOLS=<tool_id> or STAGE=<stage>"));
    }
    Ok(())
}

fn required_env(key: &str) -> Result<String> {
    let value = env_or_empty(key);
    if value.is_empty() {
        return Err(anyhow!("missing required env var: {key}"));
    }
    Ok(value)
}

fn env_or_empty(key: &str) -> String {
    std::env::var(key).unwrap_or_default()
}

fn env_or_default(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn container_artifact_dir() -> String {
    env_or_default("CONTAINER_ARTIFACT_DIR", "artifacts/containers")
}

fn bijux_command_prefix() -> Vec<String> {
    std::env::var("BIJUX_BIN")
        .unwrap_or_else(|_| "./scripts/run.sh tooling bijux".to_string())
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

fn pythonpath_with_tooling(prefix: &str) -> String {
    match std::env::var("PYTHONPATH") {
        Ok(existing) if !existing.is_empty() => format!("{prefix}:{existing}"),
        _ => prefix.to_string(),
    }
}

fn merge_outcomes(
    mut left: ContainerCommandOutcome,
    right: ContainerCommandOutcome,
) -> ContainerCommandOutcome {
    left.exit_code = if left.exit_code != 0 {
        left.exit_code
    } else {
        right.exit_code
    };
    left.stdout.push_str(&right.stdout);
    left.stderr.push_str(&right.stderr);
    left
}
