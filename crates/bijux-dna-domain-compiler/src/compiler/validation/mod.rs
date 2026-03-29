use super::*;

mod catalog_validation;
mod index_rules;

use self::catalog_validation::{
    validate_domain_vocabularies, validate_reference_catalogs, DomainVocabularies,
};
use self::index_rules::validate_domain_indexes_and_pipelines;

/// Validate authored domain files and cross-domain invariants.
///
/// # Errors
///
/// Returns an error when required files are missing, schemas/invariants are
/// violated, or domain catalogs are inconsistent.
#[allow(clippy::too_many_lines)]
pub fn validate_domain(options: &ValidateOptions) -> Result<()> {
    for rel in [
        "fastq/stages/_schema.yaml",
        "bam/stages/_schema.yaml",
        "vcf/stages/_schema.yaml",
        "fastq/tools/_schema.yaml",
        "bam/tools/_schema.yaml",
        "vcf/tools/_schema.yaml",
        "fastq/artifacts.yaml",
        "bam/artifacts.yaml",
        "vcf/artifacts.yaml",
        "fastq/metrics.yaml",
        "bam/metrics.yaml",
        "vcf/metrics.yaml",
        "fastq/index.yaml",
        "bam/index.yaml",
        "vcf/index.yaml",
    ] {
        super::compile::require_exists(&options.domain_dir.join(rel))?;
    }
    let workspace_root = options.domain_dir.parent().unwrap_or(&options.domain_dir);
    validate_reference_catalogs(workspace_root)?;

    let mut tool_ids = BTreeMap::<String, String>::new();
    let mut stage_ids = BTreeMap::<String, String>::new();
    let mut tool_capabilities = BTreeMap::<String, BTreeSet<String>>::new();
    let mut tool_statuses = BTreeMap::<String, String>::new();
    let mut tool_metrics_schemas = BTreeMap::<String, String>::new();
    let DomainVocabularies {
        artifact_vocab,
        metric_vocab,
    } = validate_domain_vocabularies(&options.domain_dir)?;

    for dom in ["fastq", "bam", "vcf"] {
        let stage_glob = options.domain_dir.join(dom).join("stages");
        if stage_glob.exists() {
            for entry in std::fs::read_dir(&stage_glob)
                .with_context(|| format!("read {}", stage_glob.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                    continue;
                }
                if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                    continue;
                }
                let stage: DomainStage = read_yaml(&path)?;
                let stage_raw = std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                if stage.stage_id.is_empty() {
                    bail!("{} missing stage_id", path.display());
                }
                if is_umbrella_stage(&stage.stage_id) {
                    bail!(
                        "{} stage_id {} is an umbrella stage and must be split into concrete stage IDs",
                        path.display(),
                        stage.stage_id
                    );
                }
                if dom != "vcf" {
                    let artifact_ids = artifact_vocab
                        .get(dom)
                        .ok_or_else(|| anyhow!("missing artifact vocab for domain {dom}"))?;
                    let metric_ids = metric_vocab
                        .get(dom)
                        .ok_or_else(|| anyhow!("missing metric vocab for domain {dom}"))?;
                    if stage.inputs.is_empty() {
                        bail!("{} missing inputs", path.display());
                    }
                    if stage.outputs.is_empty() {
                        bail!("{} missing outputs", path.display());
                    }
                    if stage.compatible_tools.is_empty() {
                        bail!("{} missing compatible_tools", path.display());
                    }
                    if stage.invariants.is_empty() {
                        bail!("{} missing invariants", path.display());
                    }
                    if stage.assumptions.is_empty() {
                        bail!("{} missing assumptions", path.display());
                    }
                    if stage.bank_hooks.is_empty() {
                        bail!("{} missing bank_hooks", path.display());
                    }
                    if stage.metrics.is_empty() {
                        bail!("{} missing metrics", path.display());
                    }
                    if stage.allowed_missingness.is_empty() && stage.status == "supported" {
                        bail!("{} missing allowed_missingness", path.display());
                    }
                    for output in &stage.outputs {
                        if !artifact_ids.contains(&output.name) {
                            bail!(
                                "{} stage output `{}` is outside {} artifact vocabulary",
                                path.display(),
                                output.name,
                                dom
                            );
                        }
                    }
                    for output in &stage.required_outputs {
                        if !artifact_ids.contains(output) {
                            bail!(
                                "{} required_output `{}` is outside {} artifact vocabulary",
                                path.display(),
                                output,
                                dom
                            );
                        }
                    }
                    for metric in &stage.metrics {
                        if !metric_ids.contains(&metric.name) {
                            bail!(
                                "{} metric `{}` is outside {} metric vocabulary",
                                path.display(),
                                metric.name,
                                dom
                            );
                        }
                    }
                    let allowed_bank_hooks = BTreeSet::from([
                        "adapter_bank",
                        "polyx_bank",
                        "contaminant_db_bank",
                        "reference_bank",
                        "contamination_db_bank",
                        "none",
                    ]);
                    for hook in &stage.bank_hooks {
                        if !allowed_bank_hooks.contains(hook.as_str()) {
                            bail!(
                                "{} bank_hook `{}` is outside the allowed vocabulary",
                                path.display(),
                                hook
                            );
                        }
                    }
                }
                let input_names = stage
                    .inputs
                    .iter()
                    .map(|port| port.name.clone())
                    .collect::<BTreeSet<_>>();
                let output_names = stage
                    .outputs
                    .iter()
                    .map(|port| port.name.clone())
                    .collect::<BTreeSet<_>>();
                for port in &stage.inputs {
                    if port.data_type.trim().is_empty() || port.cardinality.trim().is_empty() {
                        bail!("{} has input missing data_type/cardinality", path.display());
                    }
                }
                for port in &stage.outputs {
                    if port.data_type.trim().is_empty() || port.cardinality.trim().is_empty() {
                        bail!(
                            "{} has output missing data_type/cardinality",
                            path.display()
                        );
                    }
                }
                for required in &stage.required_inputs {
                    if !input_names.contains(required) {
                        bail!(
                            "{} required_inputs references missing input `{required}`",
                            path.display()
                        );
                    }
                }
                for required in &stage.required_outputs {
                    if !output_names.contains(required) {
                        bail!(
                            "{} required_outputs references missing output `{required}`",
                            path.display()
                        );
                    }
                }
                for metric in &stage.metrics {
                    if metric.name.trim().is_empty() {
                        bail!("{} has metric with empty name", path.display());
                    }
                }
                ensure_status(&stage.status, &path)?;
                if has_supported_placeholder_forbidden_token(&stage_raw)
                    && !placeholders_allowed(&stage.status)
                {
                    bail!(
                        "{} contains placeholder token; placeholders are allowed only under status=planned",
                        path.display()
                    );
                }
                if dom != "vcf" && stage.scope != "pre_hpc_pre_vcf" {
                    bail!("{} invalid stage scope {}", path.display(), stage.scope);
                }
                if dom != "vcf" && stage.domain != dom {
                    bail!(
                        "{} stage {} declares domain {} but is filed under domain/{}",
                        path.display(),
                        stage.stage_id,
                        stage.domain,
                        dom
                    );
                }
                if dom != "vcf" && !stage.stage_id.starts_with(&format!("{}.", stage.domain)) {
                    bail!(
                        "{} stage_id {} must be namespaced by domain {}",
                        path.display(),
                        stage.stage_id,
                        stage.domain
                    );
                }
                if let Some(prev) =
                    stage_ids.insert(stage.stage_id.clone(), path.display().to_string())
                {
                    bail!(
                        "duplicate stage_id {} in {} and {}",
                        stage.stage_id,
                        prev,
                        path.display()
                    );
                }
            }
        }

        let tool_glob = options.domain_dir.join(dom).join("tools");
        if tool_glob.exists() {
            for entry in std::fs::read_dir(&tool_glob)
                .with_context(|| format!("read {}", tool_glob.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                    continue;
                }
                if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                    continue;
                }
                let tool: DomainToolLoose = read_yaml(&path)?;
                let tool_raw = std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
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
                if dom != "vcf" && tool.scope != "pre_hpc_pre_vcf" {
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
                    tool_capabilities.insert(
                        tool.tool_id.clone(),
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
                        bail!(
                            "{} supported tool {} missing capabilities",
                            path.display(),
                            tool.tool_id
                        );
                    }
                    let mut stage_specs = Vec::new();
                    for stage_id in &tool.stage_ids {
                        let stage_domain = stage_id.split('.').next().unwrap_or(dom);
                        let stage_path =
                            options
                                .domain_dir
                                .join(stage_domain)
                                .join("stages")
                                .join(format!(
                                    "{}.yaml",
                                    stage_id
                                        .split_once('.')
                                        .map_or(stage_id.as_str(), |(_, suffix)| suffix)
                                        .replace('.', "_")
                                ));
                        if stage_path.exists() {
                            let stage_yaml_raw = std::fs::read_to_string(&stage_path)
                                .with_context(|| {
                                    format!(
                                        "read stage for output validation {}",
                                        stage_path.display()
                                    )
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
                tool_ids
                    .entry(tool.tool_id.clone())
                    .or_insert_with(|| path.display().to_string());
                tool_statuses.insert(tool.tool_id.clone(), tool.status.clone());
                tool_metrics_schemas.insert(tool.tool_id.clone(), tool.metrics_schema_id.clone());
            }
        }
    }

    let fastq_canonical = bijux_dna_domain_fastq::stages::ids::STAGES
        .iter()
        .map(|id| id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let bam_canonical = bijux_dna_domain_bam::stage_specs::BamStage::all()
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    // Accept additional domain-declared stages so domain specs can evolve ahead
    // of canonical stage catalogs; still enforce that canonical stages are present.
    for stage_id in &fastq_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("fastq stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }
    for stage_id in &bam_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("bam stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }

    validate_domain_indexes_and_pipelines(
        options,
        &stage_ids,
        &tool_ids,
        &tool_capabilities,
        &tool_statuses,
        &tool_metrics_schemas,
    )?;

    println!("domain-validate: OK");
    Ok(())
}
