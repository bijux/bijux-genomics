use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use super::{
    require_compatibility_fixture, require_list_contains, require_profile_stage,
    LocalPipelineDagConfig, LocalPipelineDagDomain, LocalPipelineDagValidationProfileReport,
    PipelineProfileEvidence,
};

pub(super) fn validate_amplicon_asv_otu_no_vcf_profile(
    evidence: &dyn PipelineProfileEvidence,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "amplicon_asv_otu_no_vcf";

    if config
        .nodes
        .iter()
        .any(|node| node.stage_id.starts_with("bam.") || node.stage_id.starts_with("vcf."))
    {
        return Err(anyhow!(
            "{PROFILE_ID}: amplicon-only pipeline must not admit BAM or VCF germline stages"
        ));
    }
    if !matches!(config.domain, LocalPipelineDagDomain::Fastq) {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must stay in the governed `fastq` domain, found `{}`",
            config.domain.as_str()
        ));
    }
    if config.default_corpus_id != "corpus-03-amplicon-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed amplicon corpus `corpus-03-amplicon-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let validate_reads = require_profile_stage(&stage_index, PROFILE_ID, "fastq.validate_reads")?;
    let normalize_primers =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.normalize_primers")?;
    let infer_asvs = require_profile_stage(&stage_index, PROFILE_ID, "fastq.infer_asvs")?;
    let remove_chimeras = require_profile_stage(&stage_index, PROFILE_ID, "fastq.remove_chimeras")?;
    let cluster_otus = require_profile_stage(&stage_index, PROFILE_ID, "fastq.cluster_otus")?;
    let normalize_abundance =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.normalize_abundance")?;
    let report_qc = require_profile_stage(&stage_index, PROFILE_ID, "fastq.report_qc")?;

    require_list_contains(
        &validate_reads.external_inputs,
        "corpus.raw_amplicon_fastq_reads",
        PROFILE_ID,
        "fastq.validate_reads must declare the governed amplicon FASTQ corpus input",
    )?;
    require_list_contains(
        &normalize_primers.external_inputs,
        "amplicon_primer_contract",
        PROFILE_ID,
        "fastq.normalize_primers must declare the governed primer contract",
    )?;
    require_list_contains(
        &normalize_primers.upstream_inputs,
        "validated_amplicon_reads",
        PROFILE_ID,
        "fastq.normalize_primers must consume validated amplicon reads",
    )?;
    require_list_contains(
        &infer_asvs.depends_on,
        "fastq.normalize_primers",
        PROFILE_ID,
        "fastq.infer_asvs must remain downstream of primer normalization",
    )?;
    require_list_contains(
        &infer_asvs.upstream_inputs,
        "normalized_amplicon_reads",
        PROFILE_ID,
        "fastq.infer_asvs must consume normalized amplicon reads",
    )?;
    require_list_contains(
        &infer_asvs.outputs,
        "asv_table",
        PROFILE_ID,
        "fastq.infer_asvs must keep the ASV table output explicit",
    )?;
    require_list_contains(
        &infer_asvs.outputs,
        "asv_representatives",
        PROFILE_ID,
        "fastq.infer_asvs must keep the ASV representatives output explicit",
    )?;
    require_list_contains(
        &remove_chimeras.depends_on,
        "fastq.infer_asvs",
        PROFILE_ID,
        "fastq.remove_chimeras must remain downstream of ASV inference",
    )?;
    require_list_contains(
        &remove_chimeras.external_inputs,
        "chimera_control_contract",
        PROFILE_ID,
        "fastq.remove_chimeras must declare the governed chimera-control contract",
    )?;
    require_list_contains(
        &remove_chimeras.upstream_inputs,
        "asv_representatives",
        PROFILE_ID,
        "fastq.remove_chimeras must consume ASV representatives",
    )?;
    require_list_contains(
        &remove_chimeras.outputs,
        "non_chimeric_representatives",
        PROFILE_ID,
        "fastq.remove_chimeras must keep non-chimeric representatives explicit",
    )?;
    require_list_contains(
        &cluster_otus.depends_on,
        "fastq.normalize_primers",
        PROFILE_ID,
        "fastq.cluster_otus must remain downstream of primer normalization",
    )?;
    require_list_contains(
        &cluster_otus.depends_on,
        "fastq.remove_chimeras",
        PROFILE_ID,
        "fastq.cluster_otus must remain downstream of chimera removal",
    )?;
    require_list_contains(
        &cluster_otus.upstream_inputs,
        "normalized_amplicon_reads",
        PROFILE_ID,
        "fastq.cluster_otus must consume normalized amplicon reads",
    )?;
    require_list_contains(
        &cluster_otus.upstream_inputs,
        "non_chimeric_representatives",
        PROFILE_ID,
        "fastq.cluster_otus must consume non-chimeric representatives",
    )?;
    require_list_contains(
        &cluster_otus.outputs,
        "otu_table",
        PROFILE_ID,
        "fastq.cluster_otus must keep the OTU table output explicit",
    )?;
    require_list_contains(
        &normalize_abundance.depends_on,
        "fastq.cluster_otus",
        PROFILE_ID,
        "fastq.normalize_abundance must remain downstream of OTU clustering",
    )?;
    require_list_contains(
        &normalize_abundance.upstream_inputs,
        "otu_table",
        PROFILE_ID,
        "fastq.normalize_abundance must consume the OTU table handoff",
    )?;
    require_list_contains(
        &normalize_abundance.outputs,
        "normalized_abundance_table",
        PROFILE_ID,
        "fastq.normalize_abundance must keep the normalized abundance output explicit",
    )?;
    require_list_contains(
        &report_qc.depends_on,
        "fastq.normalize_abundance",
        PROFILE_ID,
        "fastq.report_qc must remain downstream of abundance normalization",
    )?;
    require_list_contains(
        &report_qc.upstream_inputs,
        "asv_metrics",
        PROFILE_ID,
        "fastq.report_qc must consume ASV metrics",
    )?;
    require_list_contains(
        &report_qc.upstream_inputs,
        "otu_table",
        PROFILE_ID,
        "fastq.report_qc must consume OTU table evidence",
    )?;
    require_list_contains(
        &report_qc.upstream_inputs,
        "normalized_abundance_table",
        PROFILE_ID,
        "fastq.report_qc must consume normalized abundance evidence",
    )?;

    if report_qc.upstream_inputs.iter().any(|value| value.contains("vcf") || value.contains("bam"))
    {
        return Err(anyhow!(
            "{PROFILE_ID}: amplicon reporting must not mix BAM or VCF germline outputs into the final report"
        ));
    }

    for node in &config.nodes {
        if (node.stage_id.starts_with("bam.") || node.stage_id.starts_with("vcf."))
            && node.upstream_inputs.iter().any(|value| {
                value == "asv_table"
                    || value == "asv_representatives"
                    || value == "otu_table"
                    || value == "otu_representatives"
                    || value == "normalized_abundance_table"
            })
        {
            return Err(anyhow!(
                "{PROFILE_ID}: ASV or OTU outputs must not bridge into BAM or VCF germline stages"
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
        "fastq.normalize_primers",
        "corpus-03",
        "corpus-03-amplicon-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.infer_asvs",
        "corpus-03",
        "corpus-03-amplicon-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.cluster_otus",
        "corpus-03",
        "corpus-03-amplicon-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.normalize_abundance",
        "corpus-03",
        "corpus-03-amplicon-mini",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-03-amplicon-mini for governed amplicon inputs"
            .to_string(),
        "pipeline remains fastq-only and rejects bam or vcf germline stages".to_string(),
        "fastq.normalize_primers, fastq.infer_asvs, fastq.cluster_otus, and fastq.normalize_abundance stay fixture-backed by corpus-03-amplicon-mini"
            .to_string(),
        "fastq.normalize_primers keeps the governed primer contract explicit".to_string(),
        "fastq.infer_asvs and fastq.remove_chimeras keep ASV and chimera handoffs explicit"
            .to_string(),
        "fastq.cluster_otus consumes normalized reads plus non-chimeric representatives".to_string(),
        "fastq.normalize_abundance stays downstream of OTU clustering with an explicit abundance table output"
            .to_string(),
        "fastq.report_qc consumes ASV, OTU, and abundance evidence without mixing bam or vcf germline outputs"
            .to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}
