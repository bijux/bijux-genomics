//! Owner: bijux-dna-bench
use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use super::{
    require_compatibility_fixture, require_list_contains, require_profile_stage,
    LocalPipelineDagConfig, LocalPipelineDagDomain, LocalPipelineDagValidationProfileReport,
    PipelineProfileEvidence,
};

pub(super) fn validate_edna_taxonomy_no_vcf_profile(
    evidence: &dyn PipelineProfileEvidence,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "edna_taxonomy_no_vcf";

    if config
        .nodes
        .iter()
        .any(|node| node.stage_id.starts_with("bam.") || node.stage_id.starts_with("vcf."))
    {
        return Err(anyhow!(
            "{PROFILE_ID}: taxonomy-only pipeline must not admit BAM or VCF germline stages"
        ));
    }
    if !matches!(config.domain, LocalPipelineDagDomain::Fastq) {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must stay in the governed `fastq` domain, found `{}`",
            config.domain.as_str()
        ));
    }
    if config.default_corpus_id != "corpus-02-edna-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed eDNA corpus `corpus-02-edna-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let validate_reads = require_profile_stage(&stage_index, PROFILE_ID, "fastq.validate_reads")?;
    let _filter_reads = require_profile_stage(&stage_index, PROFILE_ID, "fastq.filter_reads")?;
    let screen_taxonomy = require_profile_stage(&stage_index, PROFILE_ID, "fastq.screen_taxonomy")?;
    let report_qc = require_profile_stage(&stage_index, PROFILE_ID, "fastq.report_qc")?;

    require_list_contains(
        &validate_reads.external_inputs,
        "corpus.raw_fastq_reads",
        PROFILE_ID,
        "fastq.validate_reads must declare the governed eDNA FASTQ corpus input",
    )?;
    require_list_contains(
        &screen_taxonomy.depends_on,
        "fastq.filter_reads",
        PROFILE_ID,
        "fastq.screen_taxonomy must remain downstream of FASTQ filtering",
    )?;
    require_list_contains(
        &screen_taxonomy.upstream_inputs,
        "filtered_reads",
        PROFILE_ID,
        "fastq.screen_taxonomy must consume the filtered FASTQ handoff",
    )?;
    require_list_contains(
        &screen_taxonomy.external_inputs,
        "taxonomy_database.root",
        PROFILE_ID,
        "fastq.screen_taxonomy must declare the governed taxonomy database root",
    )?;
    require_list_contains(
        &screen_taxonomy.external_inputs,
        "taxonomy_expected_truth_table",
        PROFILE_ID,
        "fastq.screen_taxonomy must declare the governed expected-taxa contract",
    )?;
    require_list_contains(
        &screen_taxonomy.outputs,
        "taxonomy_classification",
        PROFILE_ID,
        "fastq.screen_taxonomy must keep the classified taxonomy output explicit",
    )?;
    require_list_contains(
        &screen_taxonomy.outputs,
        "unclassified_reads",
        PROFILE_ID,
        "fastq.screen_taxonomy must keep the unclassified-read output explicit",
    )?;
    require_list_contains(
        &screen_taxonomy.outputs,
        "taxonomy_summary",
        PROFILE_ID,
        "fastq.screen_taxonomy must keep the taxonomy summary output explicit",
    )?;
    require_list_contains(
        &report_qc.depends_on,
        "fastq.screen_taxonomy",
        PROFILE_ID,
        "fastq.report_qc must remain downstream of taxonomy screening",
    )?;
    require_list_contains(
        &report_qc.upstream_inputs,
        "taxonomy_summary",
        PROFILE_ID,
        "fastq.report_qc must consume the taxonomy summary handoff",
    )?;

    if report_qc.upstream_inputs.iter().any(|value| value.contains("vcf") || value.contains("bam"))
    {
        return Err(anyhow!(
            "{PROFILE_ID}: taxonomy reporting must not mix BAM or VCF germline outputs into the final report"
        ));
    }

    for node in &config.nodes {
        if (node.stage_id.starts_with("bam.") || node.stage_id.starts_with("vcf."))
            && node.upstream_inputs.iter().any(|value| {
                value == "taxonomy_classification"
                    || value == "unclassified_reads"
                    || value == "taxonomy_summary"
            })
        {
            return Err(anyhow!(
                "{PROFILE_ID}: taxonomy outputs must not bridge into BAM or VCF germline stages"
            ));
        }
    }

    let compatibility_report = evidence.corpus_stage_compatibility()?;
    let compatibility_index = compatibility_report
        .stages
        .iter()
        .map(|entry| (entry.stage_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.screen_taxonomy",
        "corpus-02",
        "corpus-02-edna-mini",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-02-edna-mini for governed edna taxonomy inputs"
            .to_string(),
        "pipeline remains fastq-only and rejects bam or vcf germline stages".to_string(),
        "fastq.validate_reads keeps the governed raw-fastq corpus handoff explicit".to_string(),
        "fastq.screen_taxonomy stays fixture-backed by corpus-02-edna-mini".to_string(),
        "fastq.screen_taxonomy remains downstream of filtered reads".to_string(),
        "fastq.screen_taxonomy keeps taxonomy database and expected-truth contracts explicit"
            .to_string(),
        "fastq.screen_taxonomy keeps classification, unclassified, and summary outputs explicit"
            .to_string(),
        "fastq.report_qc consumes taxonomy summary without mixing bam or vcf germline outputs"
            .to_string(),
    ];
    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}
