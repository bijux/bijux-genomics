pub fn select_trim_tools(tools: &[String], allow_experimental: bool) -> Result<Vec<String>> {
    let mut allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_TRIM_READS);
    if allow_experimental {
        if !allowlist.iter().any(|tool| tool.as_str() == "seqpurge") {
            allowlist.push(bijux_dna_core::ids::ToolId::from_static("seqpurge"));
        }
    } else {
        allowlist.retain(|tool| tool.as_str() != "seqpurge");
    }
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_validate_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_VALIDATE_READS);
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_filter_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_FILTER_READS);
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_merge_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_MERGE_PAIRS);
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_correct_tools(tools: &[String], allow_experimental: bool) -> Result<Vec<String>> {
    let mut allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_CORRECT_ERRORS);
    if !allow_experimental {
        allowlist.retain(|tool| tool.as_str() == "rcorrector");
    }
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_qc_post_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_REPORT_QC);
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_umi_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_EXTRACT_UMIS);
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_screen_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_SCREEN_TAXONOMY);
    select_tools_with_allowlist(tools, &allowlist)
}

pub fn select_stats_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_PROFILE_READS);
    select_tools_with_allowlist(tools, &allowlist)
}

fn select_tools_with_allowlist(
    tools: &[String],
    allowlist: &[bijux_dna_core::ids::ToolId],
) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}

#[must_use]
pub fn apply_tool_overrides(
    base: BTreeMap<String, String>,
    profile: BTreeMap<String, String>,
    cli_overrides: BTreeMap<String, String>,
    forced_overrides: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut merged = base;
    for (stage, tool) in profile {
        merged.insert(stage, tool);
    }
    for (stage, tool) in cli_overrides {
        merged.insert(stage, tool);
    }
    for (stage, tool) in forced_overrides {
        merged.insert(stage, tool);
    }
    merged
}

#[must_use]
pub fn fastq_pipeline_id_catalog(profile_id: &str) -> Vec<String> {
    if let Ok(profile) =
        bijux_dna_pipelines::registry::profile_by_id(bijux_dna_pipelines::Domain::Fastq, profile_id)
    {
        return profile
            .capabilities
            .required_stages
            .iter()
            .filter(|stage| stage.starts_with(bijux_dna_domain_fastq::STAGE_PREFIX))
            .map(|stage| (*stage).to_string())
            .collect();
    }
    required_id_catalog()
}
