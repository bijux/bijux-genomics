use super::{
    anyhow, bail, ensure_status, has_supported_placeholder_forbidden_token, is_umbrella_stage,
    placeholders_allowed, read_yaml, BTreeMap, BTreeSet, Context, DomainStage, Result,
    ValidateOptions, DEFAULT_COMPILE_SCOPE,
};

pub(super) fn validate_stage_files(
    options: &ValidateOptions,
    dom: &str,
    artifact_vocab: &BTreeMap<String, BTreeSet<String>>,
    metric_vocab: &BTreeMap<String, BTreeSet<String>>,
    stage_ids: &mut BTreeMap<String, String>,
) -> Result<()> {
    let stage_glob = options.domain_dir.join(dom).join("stages");
    if !stage_glob.exists() {
        return Ok(());
    }
    for path in super::collect_yaml_files(&stage_glob)? {
        let stage: DomainStage = read_yaml(&path)?;
        let stage_raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
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
        if stage.compatible_tools.is_empty()
            && (stage.status == "supported" || stage.planned_out_of_scope.is_empty())
        {
            bail!("{} missing compatible_tools or planned_out_of_scope", path.display());
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
            "asset_lock_registry",
            "barcode_kit_bank",
            "polyx_bank",
            "primer_bank",
            "contaminant_db_bank",
            "contaminant_database_lock",
            "reference_bank",
            "contamination_db_bank",
            "provenance_registry",
            "rrna_database_lock",
            "taxonomy_database_lock",
            "instrument_artifact_bank",
            "none",
        ]);
        for hook in &stage.bank_hooks {
            if !allowed_bank_hooks.contains(hook.as_str()) {
                bail!("{} bank_hook `{}` is outside the allowed vocabulary", path.display(), hook);
            }
        }
        let input_names =
            stage.inputs.iter().map(|port| port.name.clone()).collect::<BTreeSet<_>>();
        let output_names =
            stage.outputs.iter().map(|port| port.name.clone()).collect::<BTreeSet<_>>();
        if input_names.len() != stage.inputs.len() {
            bail!("{} has duplicate input port names", path.display());
        }
        if output_names.len() != stage.outputs.len() {
            bail!("{} has duplicate output port names", path.display());
        }
        for port in &stage.inputs {
            if port.name.trim().is_empty() {
                bail!("{} has input with empty name", path.display());
            }
            if port.data_type.trim().is_empty() || port.cardinality.trim().is_empty() {
                bail!("{} has input missing data_type/cardinality", path.display());
            }
        }
        for port in &stage.outputs {
            if port.name.trim().is_empty() {
                bail!("{} has output with empty name", path.display());
            }
            if port.data_type.trim().is_empty() || port.cardinality.trim().is_empty() {
                bail!("{} has output missing data_type/cardinality", path.display());
            }
        }
        for required in &stage.required_inputs {
            if !input_names.contains(required) {
                bail!("{} required_inputs references missing input `{required}`", path.display());
            }
        }
        for required in &stage.required_outputs {
            if !output_names.contains(required) {
                bail!("{} required_outputs references missing output `{required}`", path.display());
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
        if dom != "vcf" && stage.scope != DEFAULT_COMPILE_SCOPE {
            bail!("{} invalid stage scope {}", path.display(), stage.scope);
        }
        if dom == "vcf" && stage.scope != "post_vcf" {
            bail!("{} invalid VCF stage scope {}", path.display(), stage.scope);
        }
        if stage.domain != dom {
            bail!(
                "{} stage {} declares domain {} but is filed under domain/{}",
                path.display(),
                stage.stage_id,
                stage.domain,
                dom
            );
        }
        if !stage.stage_id.starts_with(&format!("{}.", stage.domain)) {
            bail!(
                "{} stage_id {} must be namespaced by domain {}",
                path.display(),
                stage.stage_id,
                stage.domain
            );
        }
        if let Some(previous) = stage_ids.insert(stage.stage_id.clone(), path.display().to_string())
        {
            bail!("duplicate stage_id {} in {} and {}", stage.stage_id, previous, path.display());
        }
    }
    Ok(())
}
