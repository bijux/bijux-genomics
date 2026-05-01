#![allow(clippy::too_many_lines)]

use std::fmt::Write as _;

#[allow(clippy::wildcard_imports)]
use super::*;

type SummaryStatus = (BTreeMap<String, String>, BTreeMap<String, String>, BTreeMap<String, String>);

fn generate_tool_ids_content(workspace: &Workspace) -> Result<String> {
    let statuses = governed_container_statuses(workspace)?;
    let mut out = String::from(
        "# GENERATED FILE - DO NOT EDIT\n# Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-tool-ids\n# format: <tool_id><TAB><status>\n",
    );
    for (tool_id, status) in statuses {
        let _ = writeln!(out, "{tool_id}\t{status}");
    }
    Ok(out)
}

pub(super) fn generate_tool_ids(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run generate-tool-ids -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "containers/TOOL_IDS.txt", usage)?;
    write_utf8(&out, &generate_tool_ids_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

pub(super) fn check_tool_id_manifest(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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
            "containers/TOOL_IDS.txt drift; regenerate with cargo run -p bijux-dna-dev -- containers run generate-tool-ids\n",
        ));
    }

    let expected_ids = actual
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .filter_map(|line| line.split_once('\t').map(|(tool_id, _)| tool_id.to_string()))
        .collect::<BTreeSet<_>>();
    let file_ids = governed_container_file_ids(workspace)?;
    let unknown = file_ids.difference(&expected_ids).cloned().collect::<Vec<_>>();
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
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-tool-name-map -->".to_string(),
        String::new(),
        "# Tool Name Mapping".to_string(),
        String::new(),
        "- Root contract: [containers/README.md](../README.md)".to_string(),
        "- Container docs index: [containers/docs/index.md](index.md)".to_string(),
        "- Tool ID manifest: [containers/TOOL_IDS.txt](../TOOL_IDS.txt)".to_string(),
        "- Tool ID contract: [containers/docs/TOOL_IDS_CONTRACT.md](TOOL_IDS_CONTRACT.md)"
            .to_string(),
        "- Tool docs index: [containers/docs/tools/index.md](tools/index.md)".to_string(),
        String::new(),
        "| Tool ID | Expected Binary | Status |".to_string(),
        "|---|---|---|".to_string(),
    ];
    for (tool_id, status) in statuses {
        let row = rows.get(&tool_id).cloned().unwrap_or_default();
        let expected_bin =
            row.get("expected_bin").and_then(toml::Value::as_str).unwrap_or(&tool_id);
        lines.push(format!("| `{tool_id}` | `{}` | `{status}` |", expected_bin.trim()));
    }
    Ok(format!("{}\n", lines.join("\n")))
}

pub(super) fn generate_tool_name_map(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run generate-tool-name-map -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "containers/docs/TOOL_NAME_MAP.md", usage)?;
    write_utf8(&out, &generate_tool_name_map_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

pub(super) fn check_tool_name_map_generated(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let target = workspace.path("containers/docs/TOOL_NAME_MAP.md");
    if read_utf8(&target)? != generate_tool_name_map_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(
            "tool name map drift: regenerate with cargo run -p bijux-dna-dev -- containers run generate-tool-name-map\n",
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
        let apptainer = workspace.path(&format!("containers/apptainer/shared/{tool_id}.def"));
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
        rows.push((
            tool_id.to_string(),
            status.to_string(),
            apptainer_source.to_string(),
            docker_source.to_string(),
        ));
    }
    let mut lines = vec![
        "# Containers Docs Index".to_string(),
        String::new(),
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: cargo run -p bijux-dna-dev -- containers run generate-index -->".to_string(),
        String::new(),
        "Purpose: Authoritative tool/container index for container governance and CI checks.".to_string(),
        String::new(),
        "## Strict TOC".to_string(),
        "- Root contract: [containers/README.md](../README.md)".to_string(),
        "- Entry point: [containers/index.md](../index.md)".to_string(),
        "- Policy: [containers/docs/PROMOTION_POLICY.md](PROMOTION_POLICY.md)".to_string(),
        "- Lifecycle: [containers/docs/TOOL_LIFECYCLE.md](TOOL_LIFECYCLE.md)".to_string(),
        "- Version authority: [containers/docs/VERSION_AUTHORITY.md](VERSION_AUTHORITY.md)"
            .to_string(),
        "- Lock lifecycle: [containers/docs/LOCK_LIFECYCLE.md](LOCK_LIFECYCLE.md)".to_string(),
        "- HPC frontend build authority: [containers/docs/FRONTEND_BUILD_AUTHORITY.md](FRONTEND_BUILD_AUTHORITY.md)"
            .to_string(),
        "- Build + style rules: [containers/docs/STYLE.md](STYLE.md)".to_string(),
        "- Smoke: [containers/docs/SMOKE_CONTRACT.md](SMOKE_CONTRACT.md)".to_string(),
        "- Lock/versioning: [containers/versions/LOCK.md](../versions/LOCK.md)".to_string(),
        "- Network disclosure: [containers/docs/NETWORK_USAGE.md](NETWORK_USAGE.md)".to_string(),
        "- Security boundary: [containers/docs/SECURITY_BOUNDARY.md](SECURITY_BOUNDARY.md)"
            .to_string(),
        "- Multiarch policy: [containers/docs/MULTIARCH_POLICY.md](MULTIARCH_POLICY.md)"
            .to_string(),
        "- GHCR publication: [containers/docs/GHCR_PUBLISH.md](GHCR_PUBLISH.md)".to_string(),
        "- GHCR packages view: `https://github.com/bijux?tab=packages&repo_name=bijux-genomics`".to_string(),
        "- Licenses: [containers/licenses/README.md](../licenses/README.md)".to_string(),
        "- SBOM + vulnerability hooks: `cargo run -p bijux-dna-dev -- containers run check-sbom-artifacts`, `cargo run -p bijux-dna-dev -- containers run check-vuln-hook`".to_string(),
        "- Exceptions: [containers/docker/NONROOT_EXCEPTIONS.md](../docker/NONROOT_EXCEPTIONS.md), [containers/docker/ENTRYPOINT_EXCEPTIONS.md](../docker/ENTRYPOINT_EXCEPTIONS.md), [containers/docs/PLANNED.md](PLANNED.md)".to_string(),
        "- Tool ID contract: [containers/docs/TOOL_IDS_CONTRACT.md](TOOL_IDS_CONTRACT.md)"
            .to_string(),
        String::new(),
        "## Authority".to_string(),
        "- Tool IDs + lifecycle status: [containers/TOOL_IDS.txt](../TOOL_IDS.txt) (generated from registry).".to_string(),
        "- Registry SSoT: `configs/ci/registry/tool_registry*.toml` defines tool existence and lifecycle.".to_string(),
        "- Container version metadata: [containers/versions/versions.toml](../versions/versions.toml) + [containers/versions/lock.json](../versions/lock.json).".to_string(),
        "- GHCR Docker arm64 matrix: `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-publish-matrix -- artifacts/containers/ghcr/docker-arm64-publish-matrix.json`.".to_string(),
        "- GHCR Apptainer matrix: `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-apptainer-publish-matrix -- artifacts/containers/ghcr/apptainer-publish-matrix.json`.".to_string(),
        "- Non-bijux provenance: [containers/apptainer/shared/NON_BIJUX_SOURCES.md](../apptainer/shared/NON_BIJUX_SOURCES.md).".to_string(),
        "- Ownership map: [containers/OWNERS.toml](../OWNERS.toml).".to_string(),
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

pub(super) fn generate_container_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run generate-index -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "containers/docs/index.md", usage)?;
    write_utf8(&out, &generate_container_index_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

pub(super) fn check_container_index(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let target = workspace.path("containers/docs/index.md");
    if read_utf8(&target)? != generate_container_index_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(
            "containers/docs/index.md drift; regenerate with cargo run -p bijux-dna-dev -- containers run generate-index\n",
        ));
    }
    success_line("containers index: OK")
}

#[derive(Debug, Clone, Copy)]
enum GhcrRuntimeKind {
    DockerArm64,
    Apptainer,
}

impl GhcrRuntimeKind {
    fn command_id(self) -> &'static str {
        match self {
            Self::DockerArm64 => "generate-ghcr-publish-matrix",
            Self::Apptainer => "generate-ghcr-apptainer-publish-matrix",
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::DockerArm64 => "docker-arm64",
            Self::Apptainer => "apptainer",
        }
    }

    fn default_output_path(self) -> &'static str {
        match self {
            Self::DockerArm64 => "artifacts/containers/ghcr/docker-arm64-publish-matrix.json",
            Self::Apptainer => "artifacts/containers/ghcr/apptainer-publish-matrix.json",
        }
    }

    fn package_slug(self, tool_id: &str) -> String {
        match self {
            Self::DockerArm64 => format!("docker-arm64-{tool_id}"),
            Self::Apptainer => format!("apptainer-{tool_id}"),
        }
    }

    fn usage(self) -> String {
        format!(
            "Usage: cargo run -p bijux-dna-dev -- containers run {} -- [<output-path>] [--tool <tool-id>]... [--status <status>]... [--package-prefix <prefix>]",
            self.command_id()
        )
    }
}

fn workspace_repo_name(workspace: &Workspace) -> Result<String> {
    workspace.root.file_name().and_then(|name| name.to_str()).map(ToOwned::to_owned).ok_or_else(
        || anyhow!("unable to resolve repository name from {}", workspace.root.display()),
    )
}

fn ghcr_packages_view_url(repo_name: &str) -> String {
    format!("https://github.com/bijux?tab=packages&repo_name={repo_name}")
}

fn ghcr_package_page_url(repo_name: &str, package_slug: &str) -> String {
    format!("https://github.com/bijux/{repo_name}/pkgs/container/{repo_name}%2F{package_slug}")
}

fn ghcr_package_prefix(workspace: &Workspace) -> Result<String> {
    Ok(format!("ghcr.io/bijux/{}", workspace_repo_name(workspace)?))
}

fn parse_ghcr_publish_matrix_args(
    workspace: &Workspace,
    runtime: GhcrRuntimeKind,
    args: &[String],
) -> Result<(PathBuf, String, BTreeSet<String>, BTreeSet<String>)> {
    let usage = runtime.usage();
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return Err(anyhow!(usage));
    }

    let mut output = workspace.path(runtime.default_output_path());
    let mut package_prefix = ghcr_package_prefix(workspace)?;
    let mut tools = BTreeSet::new();
    let mut statuses = BTreeSet::new();

    let mut index = 0;
    if let Some(first) = args.first() {
        if !first.starts_with("--") {
            output = path_from_arg(workspace, first);
            index = 1;
        }
    }

    while index < args.len() {
        match args[index].as_str() {
            "--tool" => {
                let Some(tool) = args.get(index + 1) else {
                    return Err(anyhow!(usage.clone()));
                };
                tools.insert(tool.trim().to_string());
                index += 2;
            }
            "--status" => {
                let Some(status) = args.get(index + 1) else {
                    return Err(anyhow!(usage.clone()));
                };
                statuses.insert(status.trim().to_string());
                index += 2;
            }
            "--package-prefix" => {
                let Some(prefix) = args.get(index + 1) else {
                    return Err(anyhow!(usage.clone()));
                };
                package_prefix = prefix.trim().trim_end_matches('/').to_string();
                index += 2;
            }
            _ => return Err(anyhow!(usage.clone())),
        }
    }

    Ok((output, package_prefix, tools, statuses))
}

fn ghcr_publish_matrix_value(
    workspace: &Workspace,
    runtime: GhcrRuntimeKind,
    package_prefix: &str,
    tools_filter: &BTreeSet<String>,
    status_filter: &BTreeSet<String>,
) -> Result<serde_json::Value> {
    let repository = workspace_repo_name(workspace)?;
    let registry = registry_tool_map(workspace)?;
    let versions = tool_versions(workspace)?;
    let statuses = governed_container_statuses(workspace)?;
    let runtime_tools = match runtime {
        GhcrRuntimeKind::DockerArm64 => docker_tool_ids(workspace)?,
        GhcrRuntimeKind::Apptainer => apptainer_tool_ids(workspace),
    };

    let mut items = Vec::new();
    for tool_id in runtime_tools {
        if !tools_filter.is_empty() && !tools_filter.contains(&tool_id) {
            continue;
        }
        let status = statuses.get(&tool_id).cloned().unwrap_or_else(|| "experimental".to_string());
        if !status_filter.is_empty() && !status_filter.contains(&status) {
            continue;
        }

        let version_row = versions.get(&tool_id).cloned().unwrap_or_default();
        let registry_row = registry.get(&tool_id).cloned().unwrap_or_default();
        let tool_version = table_string(&version_row, "version").trim().to_string();
        let resolved_version = if tool_version.is_empty() {
            let registry_version = table_string(&registry_row, "version");
            if registry_version.trim().is_empty() {
                "unknown".to_string()
            } else {
                registry_version.trim().to_string()
            }
        } else {
            tool_version
        };
        let package_slug = runtime.package_slug(&tool_id);
        let package_ref = format!("{package_prefix}/{package_slug}");
        let registry_status = table_string(&registry_row, "status");
        let smoke_version_cmd = table_string(&registry_row, "smoke_version_cmd");
        let smoke_help_cmd = table_string(&registry_row, "smoke_help_cmd");
        let smoke_minimal_cmd = table_string(&registry_row, "smoke_minimal_cmd");
        let smoke_minimal_exit_code = table_string(&registry_row, "smoke_minimal_exit_code");
        let smoke_require_help = if registry_row.contains_key("smoke_require_help") {
            table_bool(&registry_row, "smoke_require_help")
        } else {
            true
        };
        let smoke_probes = table_array_strings(&registry_row, "smoke_probes");
        let push_latest = if registry_status.is_empty() {
            statuses.get(&tool_id).is_some_and(|value| value == "production")
        } else {
            registry_status == "production"
        };
        let mut item = serde_json::json!({
            "tool_id": tool_id.clone(),
            "runtime": runtime.id(),
            "status": status,
            "tool_version": resolved_version,
            "package_slug": package_slug,
            "package_ref": package_ref,
            "package_url": ghcr_package_page_url(&repository, &runtime.package_slug(&tool_id)),
            "push_latest": push_latest,
        });
        if let Some(map) = item.as_object_mut() {
            match runtime {
                GhcrRuntimeKind::DockerArm64 => {
                    map.insert(
                        "dockerfile".to_string(),
                        serde_json::Value::String(format!(
                            "containers/docker/arm64/Dockerfile.{tool_id}"
                        )),
                    );
                    map.insert(
                        "build_context".to_string(),
                        serde_json::Value::String(".".to_string()),
                    );
                    map.insert(
                        "platform".to_string(),
                        serde_json::Value::String("linux/arm64".to_string()),
                    );
                    map.insert(
                        "image_ref".to_string(),
                        serde_json::Value::String(package_ref.clone()),
                    );
                }
                GhcrRuntimeKind::Apptainer => {
                    map.insert(
                        "artifact_path".to_string(),
                        serde_json::Value::String(format!(
                            "artifacts/containers/apptainer/sif/{tool_id}.sif"
                        )),
                    );
                    map.insert(
                        "artifact_uri".to_string(),
                        serde_json::Value::String(format!("oras://{package_ref}")),
                    );
                    map.insert(
                        "platform".to_string(),
                        serde_json::Value::String("linux/amd64".to_string()),
                    );
                }
            }
            map.insert(
                "smoke_version_cmd".to_string(),
                serde_json::Value::String(smoke_version_cmd.clone()),
            );
            map.insert(
                "smoke_help_cmd".to_string(),
                serde_json::Value::String(smoke_help_cmd.clone()),
            );
            map.insert(
                "smoke_require_help".to_string(),
                serde_json::Value::Bool(smoke_require_help),
            );
            map.insert(
                "smoke_minimal_cmd".to_string(),
                serde_json::Value::String(smoke_minimal_cmd.clone()),
            );
            map.insert(
                "smoke_minimal_exit_code".to_string(),
                serde_json::Value::String(smoke_minimal_exit_code),
            );
            map.insert(
                "smoke_probes".to_string(),
                serde_json::Value::Array(
                    smoke_probes.iter().cloned().map(serde_json::Value::String).collect(),
                ),
            );
        }
        items.push(item);
    }

    if !tools_filter.is_empty() {
        let resolved_tools = items
            .iter()
            .filter_map(|item| item.get("tool_id").and_then(serde_json::Value::as_str))
            .map(ToOwned::to_owned)
            .collect::<BTreeSet<_>>();
        let missing = tools_filter.difference(&resolved_tools).cloned().collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(anyhow!(
                "unknown or non-docker tool ids in GHCR publish matrix request: {}",
                missing.join(", ")
            ));
        }
    }

    Ok(serde_json::json!({
        "schema_version": "bijux.container.ghcr_publish_matrix.v2",
        "runtime": runtime.id(),
        "package_prefix": package_prefix,
        "packages_view_url": ghcr_packages_view_url(&repository),
        "repository": repository,
        "generated_at_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "items": items,
    }))
}

pub(super) fn generate_ghcr_publish_matrix(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = GhcrRuntimeKind::DockerArm64.usage();
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }

    let (output, package_prefix, tools, statuses) =
        parse_ghcr_publish_matrix_args(workspace, GhcrRuntimeKind::DockerArm64, args)?;
    let payload = ghcr_publish_matrix_value(
        workspace,
        GhcrRuntimeKind::DockerArm64,
        &package_prefix,
        &tools,
        &statuses,
    )?;
    write_utf8(&output, &json_string_pretty(&payload)?)?;
    success_line(format!("generated {}", output.display()))
}

pub(super) fn generate_ghcr_apptainer_publish_matrix(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = GhcrRuntimeKind::Apptainer.usage();
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }

    let (output, package_prefix, tools, statuses) =
        parse_ghcr_publish_matrix_args(workspace, GhcrRuntimeKind::Apptainer, args)?;
    let payload = ghcr_publish_matrix_value(
        workspace,
        GhcrRuntimeKind::Apptainer,
        &package_prefix,
        &tools,
        &statuses,
    )?;
    write_utf8(&output, &json_string_pretty(&payload)?)?;
    success_line(format!("generated {}", output.display()))
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
        let source_sha =
            version_row.map(|value| table_string(value, "source_sha256")).unwrap_or_default();
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
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-license-metadata -->".to_string(),
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

pub(super) fn generate_license_metadata(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run generate-license-metadata -- [<output-dir> [<index-path>]]";
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
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let expected_files =
        entries.iter().map(|entry| format!("{}.license.toml", entry.tool)).collect::<BTreeSet<_>>();
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
            bijux_dna_infra::remove_file(&path)
                .with_context(|| format!("remove {}", path.display()))?;
        }
    }
    for entry in &entries {
        write_utf8(&out_dir.join(format!("{}.license.toml", entry.tool)), &entry.file_content)?;
    }
    write_utf8(&doc_out, &generate_license_index_content(&entries))?;
    Ok(ContainerCommandOutcome::success(format!(
        "generated {}\ngenerated {}\n",
        out_dir.display(),
        doc_out.display()
    )))
}

pub(super) fn check_license_metadata(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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
            errors.push(format!("{} upstream_url must be URL", workspace.rel(&meta).display()));
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
        if apptainer_def.contains("/non-bijux/") || is_non_bijux_apptainer_source(workspace, tool) {
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

pub(super) fn check_license_index_generated(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let target = workspace.path("docs/30-operations/CONTAINER_LICENSE_INDEX.md");
    let expected = generate_license_index_content(&license_metadata_entries(workspace)?);
    if read_utf8(&target)? != expected {
        return Ok(ContainerCommandOutcome::failure(
            "license index drift: regenerate with cargo run -p bijux-dna-dev -- containers run generate-license-metadata\n",
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
        .map(walkdir::DirEntry::into_path)
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
                commands.iter().cloned().map(serde_json::Value::String).collect(),
            ),
        );
        item.insert("network_required".to_string(), serde_json::Value::Bool(!commands.is_empty()));
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

pub(super) fn generate_network_usage(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run generate-network-usage -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out = out_path_arg(workspace, args, "artifacts/containers/network_usage.json", usage)?;
    write_utf8(&out, &generate_network_usage_content(workspace)?)?;
    success_line(format!("generated {}", out.display()))
}

pub(super) fn check_network_disclosure(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let offline = match args {
        [] => false,
        [single] if single == "--offline" => true,
        [single] if single == "--help" || single == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- containers run check-network-disclosure -- [--offline]",
            )
        }
        _ => return Err(anyhow!("Usage: cargo run -p bijux-dna-dev -- containers run check-network-disclosure -- [--offline]")),
    } || std::env::var("BIJUX_OFFLINE").as_deref() == Ok("1");

    let report = std::env::var("ISO_ROOT").map(PathBuf::from).map_or_else(
        |_| workspace.path("artifacts/containers/network_usage.json"),
        |root| root.join("containers/network_usage.json"),
    );
    write_utf8(&report, &generate_network_usage_content(workspace)?)?;

    let network_doc = workspace.path("containers/docs/NETWORK_USAGE.md");
    if !network_doc.is_file() {
        return Ok(ContainerCommandOutcome::failure("missing containers/docs/NETWORK_USAGE.md\n"));
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
                errors.push(format!("{} missing key '{key}'", workspace.rel(&meta).display()));
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
                row.get("network_required").and_then(serde_json::Value::as_bool).unwrap_or(false)
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
            let Ok(value) =
                serde_json::from_str::<serde_json::Value>(&read_utf8(&path).unwrap_or_default())
            else {
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
                    value
                        .get("status")
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
    let apptainer_ids = fs::read_dir(workspace.path("containers/apptainer/shared"))
        .with_context(|| {
            format!("read {}", workspace.path("containers/apptainer/shared").display())
        })?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            entry.path().file_stem().and_then(|name| name.to_str()).map(ToString::to_string)
        })
        .collect::<BTreeSet<_>>();

    let mut outputs = BTreeMap::new();
    let mut index_lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: cargo run -p bijux-dna-dev -- containers run generate-tool-docs -->"
            .to_string(),
        "# Tool Docs Index".to_string(),
        String::new(),
        "- Root contract: [containers/README.md](../../README.md)".to_string(),
        "- Container docs index: [containers/docs/index.md](../index.md)".to_string(),
        "- Tool name map: [containers/docs/TOOL_NAME_MAP.md](../TOOL_NAME_MAP.md)".to_string(),
        "- Tool ID manifest: [containers/TOOL_IDS.txt](../../TOOL_IDS.txt)".to_string(),
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
            "<!-- source: cargo run -p bijux-dna-dev -- containers run generate-tool-docs -->"
                .to_string(),
            format!("# {tool}"),
            String::new(),
            "Purpose: generated per-tool container contract summary.".to_string(),
            String::new(),
            "- Root contract: [containers/README.md](../../README.md)".to_string(),
            "- Tool docs index: [containers/docs/tools/index.md](index.md)".to_string(),
            "- Tool name map: [containers/docs/TOOL_NAME_MAP.md](../TOOL_NAME_MAP.md)"
                .to_string(),
            "- Version inventory: [containers/versions/versions.toml](../../versions/versions.toml)"
                .to_string(),
            "- License index: [containers/licenses/README.md](../../licenses/README.md)"
                .to_string(),
            format!(
                "- Tool license record: [containers/licenses/{tool}.license.toml](../../licenses/{tool}.license.toml)"
            ),
            String::new(),
            format!("- Version: `{}`", table_string(version_row, "version")),
            format!("- License: `{}`", {
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
            }),
            format!(
                "- Runtime support: `{}`",
                if runtimes.is_empty() { "none".to_string() } else { runtimes.join(", ") }
            ),
            format!(
                "- Smoke status: `{}`",
                status.get(tool).cloned().unwrap_or_else(|| "unknown".to_string())
            ),
            String::new(),
            "## Known Limitations".to_string(),
        ];
        for limitation in limitations {
            lines.push(format!("- {limitation}"));
        }
        outputs.insert(format!("{tool}.md"), format!("{}\n", lines.join("\n")));
        index_lines.push(format!("- [{tool}]({tool}.md)"));
    }
    outputs.insert("index.md".to_string(), format!("{}\n", index_lines.join("\n")));
    Ok(outputs)
}

pub(super) fn generate_tool_docs(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run generate-tool-docs -- [<output-dir>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let out_dir = match args {
        [] => workspace.path("containers/docs/tools"),
        [dir] => path_from_arg(workspace, dir),
        _ => return Err(anyhow!(usage.to_string())),
    };
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
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
            bijux_dna_infra::remove_file(&path)
                .with_context(|| format!("remove {}", path.display()))?;
        }
    }
    for (name, content) in outputs {
        write_utf8(&out_dir.join(name), &content)?;
    }
    success_line(format!("generated tool docs under {}", out_dir.display()))
}

pub(super) fn check_tool_docs_generated(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
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
            "tool docs drift: regenerate with cargo run -p bijux-dna-dev -- containers run generate-tool-docs\n",
        ));
    }
    success_line("tool docs generated: OK")
}

fn load_summary_status(workspace: &Workspace) -> SummaryStatus {
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
    (status_from_summary, docker_digest_from_summary, apptainer_digest_from_summary)
}

fn generate_qa_matrix_content(workspace: &Workspace) -> Result<String> {
    let registry = registry_tool_map(workspace)?;
    let (status_from_summary, docker_digest_from_summary, apptainer_digest_from_summary) =
        load_summary_status(workspace);
    let mut rows = Vec::new();
    for (tool, row) in registry {
        if !table_array_strings(&row, "runtimes").iter().any(|runtime| runtime == "apptainer") {
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
            status_from_summary.get(&tool).cloned().unwrap_or_else(|| {
                let status = table_string(&row, "status");
                if status.is_empty() {
                    "unknown".to_string()
                } else {
                    status
                }
            }),
            docker_digest_from_summary.get(&tool).cloned().unwrap_or_else(|| "-".to_string()),
            apptainer_digest_from_summary.get(&tool).cloned().unwrap_or_else(|| "-".to_string()),
        ));
    }

    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-qa-matrix -->".to_string(),
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
        "## Authority Surfaces".to_string(),
        "- [docs/30-operations/CONTAINERS.md](CONTAINERS.md)".to_string(),
        "- [docs/30-operations/HPC_FRONTEND_RUNBOOK.md](HPC_FRONTEND_RUNBOOK.md)".to_string(),
        "- [containers/docs/FRONTEND_BUILD_AUTHORITY.md](../../containers/docs/FRONTEND_BUILD_AUTHORITY.md)"
            .to_string(),
        "- [containers/docs/SMOKE_CONTRACT.md](../../containers/docs/SMOKE_CONTRACT.md)"
            .to_string(),
        "- [containers/docs/NETWORK_USAGE.md](../../containers/docs/NETWORK_USAGE.md)"
            .to_string(),
        "- [containers/docs/PLANNED.md](../../containers/docs/PLANNED.md)".to_string(),
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

pub(super) fn generate_qa_matrix(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage =
        "Usage: cargo run -p bijux-dna-dev -- containers run generate-qa-matrix -- [<output-path>]";
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

pub(super) fn check_qa_matrix_generated(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let target = workspace.path("docs/30-operations/APPTAINER_QA_MATRIX.md");
    if read_utf8(&target)? != generate_qa_matrix_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(
            "qa matrix drift: regenerate with cargo run -p bijux-dna-dev -- containers run generate-qa-matrix\n",
        ));
    }
    success_line("qa matrix generated: OK")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ghcr_publish_matrix_args_keeps_default_output_and_prefix() {
        let workspace = Workspace { root: PathBuf::from("/tmp/bijux-genomics") };
        let args = vec![
            "--tool".to_string(),
            "fastp".to_string(),
            "--status".to_string(),
            "production".to_string(),
        ];
        let (output, prefix, tools, statuses) =
            parse_ghcr_publish_matrix_args(&workspace, GhcrRuntimeKind::DockerArm64, &args)
                .unwrap_or_else(|err| panic!("parse args: {err}"));
        assert_eq!(
            output,
            PathBuf::from(
                "/tmp/bijux-genomics/artifacts/containers/ghcr/docker-arm64-publish-matrix.json"
            )
        );
        assert_eq!(prefix, "ghcr.io/bijux/bijux-genomics");
        assert_eq!(tools, BTreeSet::from([String::from("fastp")]));
        assert_eq!(statuses, BTreeSet::from([String::from("production")]));
    }

    #[test]
    fn parse_ghcr_publish_matrix_args_accepts_output_and_prefix_override() {
        let workspace = Workspace { root: PathBuf::from("/tmp/bijux-genomics") };
        let args = vec![
            "reports/publish.json".to_string(),
            "--package-prefix".to_string(),
            "ghcr.io/example/private".to_string(),
        ];
        let (output, prefix, tools, statuses) =
            parse_ghcr_publish_matrix_args(&workspace, GhcrRuntimeKind::DockerArm64, &args)
                .unwrap_or_else(|err| panic!("parse args: {err}"));
        assert_eq!(output, PathBuf::from("/tmp/bijux-genomics/reports/publish.json"));
        assert_eq!(prefix, "ghcr.io/example/private");
        assert!(tools.is_empty());
        assert!(statuses.is_empty());
    }

    #[test]
    fn parse_ghcr_apptainer_publish_matrix_args_keeps_runtime_specific_default_output() {
        let workspace = Workspace { root: PathBuf::from("/tmp/bijux-genomics") };
        let (output, prefix, tools, statuses) =
            parse_ghcr_publish_matrix_args(&workspace, GhcrRuntimeKind::Apptainer, &[])
                .unwrap_or_else(|err| panic!("parse args: {err}"));
        assert_eq!(
            output,
            PathBuf::from(
                "/tmp/bijux-genomics/artifacts/containers/ghcr/apptainer-publish-matrix.json"
            )
        );
        assert_eq!(prefix, "ghcr.io/bijux/bijux-genomics");
        assert!(tools.is_empty());
        assert!(statuses.is_empty());
    }

    #[test]
    fn ghcr_runtime_package_slugs_are_runtime_scoped() {
        assert_eq!(GhcrRuntimeKind::DockerArm64.package_slug("fastp"), "docker-arm64-fastp");
        assert_eq!(GhcrRuntimeKind::Apptainer.package_slug("fastp"), "apptainer-fastp");
    }
}
