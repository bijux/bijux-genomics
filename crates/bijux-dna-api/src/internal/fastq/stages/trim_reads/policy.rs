use super::{anyhow, json_string, Result, STAGE_TRIM_READS};

pub(super) fn adapter_bank_requested(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqTrimArgs,
) -> bool {
    args.adapter_bank_preset.is_some()
        || args.adapter_bank.is_some()
        || args.adapter_bank_file.is_some()
        || !args.enable_adapters.is_empty()
        || !args.disable_adapters.is_empty()
}

pub(super) fn normalized_adapter_policy(
    policy: Option<&str>,
    explicit_bank_selection: bool,
) -> Result<Option<String>> {
    match policy {
        None if explicit_bank_selection => Ok(Some("bank".to_string())),
        None => Ok(None),
        Some("none") => Ok(Some("none".to_string())),
        Some("auto") => Ok(Some("auto".to_string())),
        Some("bank") => Ok(Some("bank".to_string())),
        Some("ancient_strict") => Ok(Some("ancient_strict".to_string())),
        Some(other) => Err(anyhow!(
            "adapter_policy must be one of `none`, `auto`, `bank`, or `ancient_strict`, received `{other}`"
        )),
    }
}

pub(super) fn normalized_polyx_policy(
    policy: Option<&str>,
    explicit_bank_selection: bool,
) -> Result<Option<String>> {
    match policy {
        None if explicit_bank_selection => Ok(Some("bank".to_string())),
        None => Ok(None),
        Some("none") => Ok(Some("none".to_string())),
        Some("trim") => Ok(Some("trim".to_string())),
        Some("bank") => Ok(Some("bank".to_string())),
        Some(other) => Err(anyhow!(
            "polyx_policy must be one of `none`, `trim`, or `bank`, received `{other}`"
        )),
    }
}

pub(super) fn normalized_contaminant_policy(
    policy: Option<&str>,
    explicit_bank_selection: bool,
) -> Result<Option<String>> {
    match policy {
        None if explicit_bank_selection => Ok(Some("bank".to_string())),
        None => Ok(None),
        Some("none") => Ok(Some("none".to_string())),
        Some("bank") => Ok(Some("bank".to_string())),
        Some(other) => Err(anyhow!(
            "contaminant_policy must be one of `none` or `bank`, received `{other}`"
        )),
    }
}

pub(super) fn adapter_policy_uses_bank(policy: Option<&str>) -> bool {
    matches!(policy, Some("bank" | "ancient_strict"))
}

pub(super) fn polyx_policy_uses_bank(policy: Option<&str>) -> bool {
    matches!(policy, Some("bank"))
}

pub(super) fn contaminant_policy_uses_bank(policy: Option<&str>) -> bool {
    matches!(policy, Some("bank"))
}

pub(super) fn benchmark_query_context(
    adapter_context: Option<&serde_json::Value>,
    polyx_context: Option<&serde_json::Value>,
    contaminant_context: Option<&serde_json::Value>,
) -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    let mut context =
        bijux_dna_domain_fastq::governed_stage_bench_query_context(STAGE_TRIM_READS.as_str())?;
    if let Some(bank_hash) = json_string(adapter_context, "bank_hash") {
        context = context.with_bank_hash("adapter_bank", bank_hash);
    }
    if let Some(bank_hash) = json_string(polyx_context, "bank_hash") {
        context = context.with_bank_hash("polyx_bank", bank_hash);
    }
    if let Some(bank_hash) = json_string(contaminant_context, "bank_hash") {
        context = context.with_bank_hash("contaminant_bank", bank_hash);
    }
    Ok(context)
}
