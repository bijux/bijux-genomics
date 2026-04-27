use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

/// # Errors
/// Returns an error if any requested trim tool is not admitted for `fastq.trim_reads`.
pub fn select_trim_tools(tools: &[String], _allow_experimental: bool) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_TRIM_READS);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested validation tool is not admitted for `fastq.validate_reads`.
pub fn select_validate_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_VALIDATE_READS);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested adapter detection tool is not admitted for
/// `fastq.detect_adapters`.
pub fn select_detect_adapters_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_DETECT_ADAPTERS);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested read-length profiling tool is not admitted for
/// `fastq.profile_read_lengths`.
pub fn select_profile_read_lengths_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS,
    );
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested filter tool is not admitted for `fastq.filter_reads`.
pub fn select_filter_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_FILTER_READS);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested low-complexity filter tool is not admitted for
/// `fastq.filter_low_complexity`.
pub fn select_filter_low_complexity_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::STAGE_FILTER_LOW_COMPLEXITY,
    );
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested merge tool is not admitted for `fastq.merge_pairs`.
pub fn select_merge_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_MERGE_PAIRS);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested deduplication tool is not admitted for
/// `fastq.remove_duplicates`.
pub fn select_remove_duplicates_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_REMOVE_DUPLICATES);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested chimera-removal tool is not admitted for
/// `fastq.remove_chimeras`.
pub fn select_remove_chimeras_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_CHIMERAS,
    );
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested primer-normalization tool is not admitted for
/// `fastq.normalize_primers`.
pub fn select_normalize_primers_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_PRIMERS,
    );
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if `fastq.infer_asvs` has no admitted runtime tools or if any requested tool
/// is not admitted for the stage.
pub fn select_infer_asvs_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::stages::ids::STAGE_INFER_ASVS,
    );
    if allowlist.is_empty() {
        return Err(anyhow!("fastq.infer_asvs has no admitted runtime tools"));
    }
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested abundance-normalization tool is not admitted for
/// `fastq.normalize_abundance`.
pub fn select_normalize_abundance_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_ABUNDANCE,
    );
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested OTU clustering tool is not admitted for
/// `fastq.cluster_otus`.
pub fn select_cluster_otus_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::stages::ids::STAGE_CLUSTER_OTUS,
    );
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested correction tool is not admitted for
/// `fastq.correct_errors`.
pub fn select_correct_tools(tools: &[String], _allow_experimental: bool) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_CORRECT_ERRORS);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested QC aggregation tool is not admitted for `fastq.report_qc`.
pub fn select_qc_post_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_REPORT_QC);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested UMI extraction tool is not admitted for `fastq.extract_umis`.
pub fn select_umi_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_EXTRACT_UMIS);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested reference-indexing tool is not admitted for
/// `fastq.index_reference`.
pub fn select_index_reference_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::stages::ids::STAGE_INDEX_REFERENCE,
    );
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested taxonomy-screening tool is not admitted for
/// `fastq.screen_taxonomy`.
pub fn select_screen_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_SCREEN_TAXONOMY);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested host-depletion tool is not admitted for `fastq.deplete_host`.
pub fn select_deplete_host_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_HOST,
    );
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested contaminant-depletion tool is not admitted for
/// `fastq.deplete_reference_contaminants`.
pub fn select_deplete_reference_contaminants_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_REFERENCE_CONTAMINANTS,
    );
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested rRNA depletion tool is not admitted for `fastq.deplete_rrna`.
pub fn select_deplete_rrna_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_DEPLETE_RRNA);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested read-statistics tool is not admitted for
/// `fastq.profile_reads`.
pub fn select_stats_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist =
        crate::selection::allowed_tools_for_stage(&bijux_dna_domain_fastq::STAGE_PROFILE_READS);
    select_tools_with_allowlist(tools, &allowlist)
}

/// # Errors
/// Returns an error if any requested overrepresented-sequence profiling tool is not admitted for
/// `fastq.profile_overrepresented_sequences`.
pub fn select_profile_overrepresented_tools(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(
        &bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
    );
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
pub fn apply_toolset_overrides(
    base: BTreeMap<String, Vec<String>>,
    profile: BTreeMap<String, Vec<String>>,
    cli_overrides: BTreeMap<String, Vec<String>>,
    forced_overrides: BTreeMap<String, Vec<String>>,
) -> BTreeMap<String, Vec<String>> {
    fn normalize_toolset(tool_ids: Vec<String>) -> Vec<String> {
        let mut normalized =
            tool_ids.into_iter().map(|tool_id| tool_id.to_ascii_lowercase()).collect::<Vec<_>>();
        normalized.sort();
        normalized.dedup();
        normalized
    }

    let mut merged = base
        .into_iter()
        .map(|(stage_id, tool_ids)| (stage_id, normalize_toolset(tool_ids)))
        .collect::<BTreeMap<_, _>>();
    for (stage_id, tool_ids) in profile {
        merged.insert(stage_id, normalize_toolset(tool_ids));
    }
    for (stage_id, tool_ids) in cli_overrides {
        merged.insert(stage_id, normalize_toolset(tool_ids));
    }
    for (stage_id, tool_ids) in forced_overrides {
        merged.insert(stage_id, normalize_toolset(tool_ids));
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
    Vec::new()
}
