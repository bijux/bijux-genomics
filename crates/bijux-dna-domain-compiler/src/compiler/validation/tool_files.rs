use super::{
    anyhow, bail, ensure_status, has_supported_placeholder_forbidden_token,
    is_tool_meaningful_in_domain, placeholders_allowed, read_yaml, validate_tool_output_subset,
    BTreeMap, BTreeSet, ContainerToolContract, Context, DomainToolLoose, Result, ValidateOptions,
    DEFAULT_COMPILE_SCOPE,
};
use std::path::Path;

pub(super) struct ToolValidationState<'a> {
    pub(super) ids: &'a mut BTreeMap<String, String>,
    pub(super) capabilities: &'a mut BTreeMap<String, BTreeSet<String>>,
    pub(super) statuses: &'a mut BTreeMap<String, String>,
    pub(super) metrics_schemas: &'a mut BTreeMap<String, String>,
    pub(super) domain_capabilities: &'a mut BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    pub(super) domain_statuses: &'a mut BTreeMap<String, BTreeMap<String, String>>,
    pub(super) domain_metrics_schemas: &'a mut BTreeMap<String, BTreeMap<String, String>>,
}

fn validate_supported_execution(
    path: &Path,
    tool_id: &str,
    support_disposition: &str,
    external_artifact_supported: bool,
    runtimes: &[String],
) -> Result<()> {
    match (support_disposition, external_artifact_supported) {
        ("retained", false) if runtimes.is_empty() => {
            bail!(
                "{} supported tool {} has no catalog-authorized runtime",
                path.display(),
                tool_id
            );
        }
        ("unsupported", true) if !runtimes.is_empty() => {
            bail!(
                "{} external artifact tool {} must not claim a redistributable runtime",
                path.display(),
                tool_id
            );
        }
        ("retained", false) | ("unsupported", true) => {}
        _ => {
            bail!(
                "{} supported tool {} conflicts with catalog disposition {}",
                path.display(),
                tool_id,
                support_disposition
            );
        }
    }
    Ok(())
}

pub(super) fn validate_tool_files(
    options: &ValidateOptions,
    dom: &str,
    artifact_vocab: &BTreeMap<String, BTreeSet<String>>,
    container_catalog: &BTreeMap<String, ContainerToolContract>,
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
        let container_contract = container_catalog.get(&tool.tool_id).ok_or_else(|| {
            anyhow!(
                "{} tool {} is absent from containers/contracts/catalog.toml",
                path.display(),
                tool.tool_id
            )
        })?;
        if !container_contract.domains.iter().any(|domain| domain == dom) {
            bail!(
                "{} tool {} is not authorized for domain {} by containers/contracts/catalog.toml",
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
            state
                .capabilities
                .insert(tool.tool_id.clone(), tool.capabilities.iter().cloned().collect());
            state
                .domain_capabilities
                .entry(dom.to_string())
                .or_default()
                .insert(tool.tool_id.clone(), tool.capabilities.iter().cloned().collect());
        }
        if tool.status == "supported" {
            validate_supported_execution(
                &path,
                &tool.tool_id,
                &container_contract.support_disposition,
                container_contract.external_artifact_supported,
                &container_contract.runtimes(),
            )?;
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
        }
        if let Some(previous) = state.ids.get(&tool.tool_id) {
            let previous_domain =
                tool_path_domain(&options.domain_dir, previous).unwrap_or_default();
            if !container_contract.domains.iter().any(|domain| domain == &previous_domain) {
                bail!(
                    "duplicate tool_id {} in {} and {} outside catalog domains {:?}",
                    tool.tool_id,
                    previous,
                    path.display(),
                    container_contract.domains
                );
            }
        } else {
            state.ids.insert(tool.tool_id.clone(), path.display().to_string());
        }
        state.statuses.insert(tool.tool_id.clone(), tool.status.clone());
        state.metrics_schemas.insert(tool.tool_id.clone(), tool.metrics_schema_id.clone());
        state
            .domain_statuses
            .entry(dom.to_string())
            .or_default()
            .insert(tool.tool_id.clone(), tool.status.clone());
        state
            .domain_metrics_schemas
            .entry(dom.to_string())
            .or_default()
            .insert(tool.tool_id.clone(), tool.metrics_schema_id.clone());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_domain_tool_cannot_promote_unsupported_catalog_tool() {
        let result = validate_supported_execution(
            Path::new("domain/bam/tools/angsd.yaml"),
            "angsd",
            "unsupported",
            false,
            &[],
        );
        let Err(error) = result else {
            panic!("unsupported catalog tool must fail");
        };

        assert!(error.to_string().contains("conflicts with catalog disposition unsupported"));
    }

    #[test]
    fn supported_domain_tool_requires_catalog_runtime() {
        let result = validate_supported_execution(
            Path::new("domain/bam/tools/tool.yaml"),
            "tool",
            "retained",
            false,
            &[],
        );
        let Err(error) = result else {
            panic!("runtime-less supported tool must fail");
        };

        assert!(error.to_string().contains("has no catalog-authorized runtime"));
    }

    #[test]
    fn supported_external_artifact_requires_no_redistributable_runtime() {
        assert!(
            validate_supported_execution(
                Path::new("domain/bam/tools/angsd.yaml"),
                "angsd",
                "unsupported",
                true,
                &[],
            )
            .is_ok(),
            "external artifact execution must remain supported"
        );

        let result = validate_supported_execution(
            Path::new("domain/bam/tools/angsd.yaml"),
            "angsd",
            "unsupported",
            true,
            &["apptainer".to_string()],
        );
        let Err(error) = result else {
            panic!("external artifact must not claim a redistributable runtime");
        };
        assert!(error.to_string().contains("must not claim a redistributable runtime"));
    }
}
