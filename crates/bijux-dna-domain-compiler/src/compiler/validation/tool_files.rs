use super::{
    anyhow, bail, ensure_status, has_supported_placeholder_forbidden_token,
    is_tool_meaningful_in_domain, placeholders_allowed, read_yaml, validate_tool_output_subset,
    BTreeMap, BTreeSet, Context, DomainToolLoose, Result, ValidateOptions, DEFAULT_COMPILE_SCOPE,
};
use std::path::Path;

fn domain_tool_key(dom: &str, tool_id: &str) -> String {
    format!("{dom}::{tool_id}")
}

pub(super) struct ToolValidationState<'a> {
    pub(super) ids: &'a mut BTreeMap<String, String>,
    pub(super) capabilities: &'a mut BTreeMap<String, BTreeSet<String>>,
    pub(super) statuses: &'a mut BTreeMap<String, String>,
    pub(super) metrics_schemas: &'a mut BTreeMap<String, String>,
}

pub(super) fn validate_tool_files(
    options: &ValidateOptions,
    dom: &str,
    artifact_vocab: &BTreeMap<String, BTreeSet<String>>,
    shared_tool_domains: &BTreeMap<String, BTreeSet<String>>,
    state: &mut ToolValidationState<'_>,
) -> Result<()> {
    let tool_glob = options.domain_dir.join(dom).join("tools");
    if !tool_glob.exists() {
        return Ok(());
    }
    for path in super::collect_yaml_files(&tool_glob)? {
        let tool: DomainToolLoose = read_yaml(&path)?;
        let tool_raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if tool.tool_id.is_empty() {
            bail!("{} missing tool_id", path.display());
        }
        ensure_status(&tool.status, &path)?;
        if has_supported_placeholder_forbidden_token(&tool_raw)
            && !placeholders_allowed(&tool.status)
        {
            bail!(
                "{} contains placeholder token; placeholders are allowed only under status=planned",
                path.display()
            );
        }
        if dom != "vcf" && tool.scope != DEFAULT_COMPILE_SCOPE {
            bail!("{} invalid tool scope {}", path.display(), tool.scope);
        }
        if tool.default_version.trim() == "0.0.0" {
            bail!("{} default_version=0.0.0 is forbidden", path.display());
        }
        if !is_tool_meaningful_in_domain(dom, &tool.tool_id) {
            bail!(
                "{} tool_id {} is not meaningful in {} domain",
                path.display(),
                tool.tool_id,
                dom
            );
        }
        let has_declared_stage_claims = tool.declared_stage_ids().next().is_some();
        if dom != "vcf"
            && (!has_declared_stage_claims
                || tool.default_version.is_empty()
                || tool.upstream.is_empty()
                || tool.pin_strategy.is_empty()
                || tool.license.is_empty()
                || tool.citation.is_empty()
                || tool.version_cmd.is_empty()
                || tool.help_cmd.is_empty()
                || tool.expected_artifacts.is_empty()
                || tool.capabilities.is_empty()
                || tool.metrics_schema_id.is_empty()
                || tool.comparability_notes.is_empty())
        {
            bail!("{} missing required tool fields", path.display());
        }
        if !tool.capabilities.is_empty() {
            state.capabilities.insert(
                domain_tool_key(dom, &tool.tool_id),
                tool.capabilities.iter().cloned().collect(),
            );
        }
        if dom != "vcf" && tool.status == "supported" {
            if tool.stage_ids.is_empty() {
                bail!(
                    "{} supported tool {} missing governed stage_ids",
                    path.display(),
                    tool.tool_id
                );
            }
            let artifact_ids = artifact_vocab
                .get(dom)
                .ok_or_else(|| anyhow!("missing artifact vocab for domain {dom}"))?;
            for artifact in &tool.expected_artifacts {
                if !artifact_ids.contains(artifact) {
                    bail!(
                        "{} expected_artifact `{}` is outside {} artifact vocabulary",
                        path.display(),
                        artifact,
                        dom
                    );
                }
            }
            if tool.capabilities.is_empty() {
                bail!("{} supported tool {} missing capabilities", path.display(), tool.tool_id);
            }
            let mut stage_specs = Vec::new();
            for stage_id in &tool.stage_ids {
                let stage_domain = stage_id.split('.').next().unwrap_or(dom);
                let stage_path =
                    options.domain_dir.join(stage_domain).join("stages").join(format!(
                        "{}.yaml",
                        stage_id
                            .split_once('.')
                            .map_or(stage_id.as_str(), |(_, suffix)| suffix)
                            .replace('.', "_")
                    ));
                if stage_path.exists() {
                    let stage_yaml_raw =
                        std::fs::read_to_string(&stage_path).with_context(|| {
                            format!("read stage for output validation {}", stage_path.display())
                        })?;
                    stage_specs.push((stage_id.as_str(), stage_yaml_raw));
                }
            }
            validate_tool_output_subset(&tool_raw, &stage_specs, &path)?;
            let dockerfile = options
                .domain_dir
                .parent()
                .unwrap_or(&options.domain_dir)
                .join("containers")
                .join("docker")
                .join("arm64")
                .join(format!("Dockerfile.{}", tool.tool_id));
            let apptainer = options
                .domain_dir
                .parent()
                .unwrap_or(&options.domain_dir)
                .join("containers")
                .join("apptainer")
                .join(format!("{}.def", tool.tool_id));
            if !dockerfile.exists() && !apptainer.exists() {
                bail!(
                    "{} supported tool {} missing container mapping ({} / {})",
                    path.display(),
                    tool.tool_id,
                    dockerfile.display(),
                    apptainer.display()
                );
            }
        }
        if let Some(previous) = state.ids.get(&tool.tool_id) {
            let allowed_domains = shared_tool_domains.get(&tool.tool_id).ok_or_else(|| {
                anyhow!("duplicate tool_id {} in {} and {}", tool.tool_id, previous, path.display())
            })?;
            let previous_domain =
                tool_path_domain(&options.domain_dir, previous).unwrap_or_default();
            if !allowed_domains.contains(dom) || !allowed_domains.contains(&previous_domain) {
                bail!(
                    "duplicate tool_id {} in {} and {} outside shared domains {:?}",
                    tool.tool_id,
                    previous,
                    path.display(),
                    allowed_domains
                );
            }
        } else {
            state.ids.insert(tool.tool_id.clone(), path.display().to_string());
        }
        let scoped_key = domain_tool_key(dom, &tool.tool_id);
        state.statuses.insert(scoped_key.clone(), tool.status.clone());
        state.metrics_schemas.insert(scoped_key, tool.metrics_schema_id.clone());
    }
    Ok(())
}

fn tool_path_domain(domain_dir: &Path, path: &str) -> Option<String> {
    Path::new(path)
        .strip_prefix(domain_dir)
        .ok()?
        .components()
        .next()?
        .as_os_str()
        .to_str()
        .map(ToString::to_string)
}
