//! Owner: bijux-dna-bench
use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use super::{
    require_compatibility_fixture, require_list_contains, require_profile_stage,
    LocalPipelineDagConfig, LocalPipelineDagValidationProfileReport, PipelineProfileEvidence,
};

pub(super) fn validate_ancient_dna_pseudohaploid_profile(
    evidence: &dyn PipelineProfileEvidence,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "ancient_dna_pseudohaploid";

    if config.default_corpus_id != "corpus-01-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed FASTQ corpus `corpus-01-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let trim_terminal_damage =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.trim_terminal_damage")?;
    let remove_duplicates =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.remove_duplicates")?;
    let bam_align = require_profile_stage(&stage_index, PROFILE_ID, "bam.align")?;
    let bam_damage = require_profile_stage(&stage_index, PROFILE_ID, "bam.damage")?;
    let bam_contamination = require_profile_stage(&stage_index, PROFILE_ID, "bam.contamination")?;
    let bam_authenticity = require_profile_stage(&stage_index, PROFILE_ID, "bam.authenticity")?;
    let vcf_call = require_profile_stage(&stage_index, PROFILE_ID, "vcf.call_pseudohaploid")?;
    let vcf_damage_filter = require_profile_stage(&stage_index, PROFILE_ID, "vcf.damage_filter")?;
    let vcf_stats = require_profile_stage(&stage_index, PROFILE_ID, "vcf.stats")?;

    require_list_contains(
        &remove_duplicates.upstream_inputs,
        "terminal_damage_trimmed_reads_r1_path",
        PROFILE_ID,
        "fastq.remove_duplicates must consume the terminal-damage-trimmed R1 handoff",
    )?;
    require_list_contains(
        &remove_duplicates.upstream_inputs,
        "terminal_damage_trimmed_reads_r2_path",
        PROFILE_ID,
        "fastq.remove_duplicates must consume the terminal-damage-trimmed R2 handoff",
    )?;
    require_list_contains(
        &bam_align.depends_on,
        "fastq.filter_reads",
        PROFILE_ID,
        "bam.align must remain downstream of the duplicate-aware FASTQ preprocessing branch",
    )?;
    require_list_contains(
        &bam_damage.external_inputs,
        "expected_damage_contract",
        PROFILE_ID,
        "bam.damage must declare the governed expected-damage asset contract",
    )?;
    require_list_contains(
        &bam_contamination.external_inputs,
        "adna_contamination_panel_contract",
        PROFILE_ID,
        "bam.contamination must declare the governed ancient-DNA contamination panel contract",
    )?;
    require_list_contains(
        &bam_authenticity.external_inputs,
        "adna_authenticity_policy_contract",
        PROFILE_ID,
        "bam.authenticity must declare the governed ancient-DNA authenticity policy contract",
    )?;
    require_list_contains(
        &vcf_call.upstream_inputs,
        "damage_report_json",
        PROFILE_ID,
        "vcf.call_pseudohaploid must consume the BAM damage evidence handoff",
    )?;
    require_list_contains(
        &vcf_call.upstream_inputs,
        "authenticity_report_json",
        PROFILE_ID,
        "vcf.call_pseudohaploid must consume the BAM authenticity evidence handoff",
    )?;
    require_list_contains(
        &vcf_call.external_inputs,
        "adna_reference_fasta_contract",
        PROFILE_ID,
        "vcf.call_pseudohaploid must declare the governed ancient-DNA reference FASTA contract",
    )?;
    require_list_contains(
        &vcf_call.external_inputs,
        "adna_reference_fai_contract",
        PROFILE_ID,
        "vcf.call_pseudohaploid must declare the governed ancient-DNA reference index contract",
    )?;
    require_list_contains(
        &vcf_damage_filter.depends_on,
        "bam.damage",
        PROFILE_ID,
        "vcf.damage_filter must remain downstream of BAM damage evidence",
    )?;
    require_list_contains(
        &vcf_damage_filter.upstream_inputs,
        "pseudohaploid_vcf",
        PROFILE_ID,
        "vcf.damage_filter must consume the pseudohaploid VCF handoff",
    )?;
    require_list_contains(
        &vcf_damage_filter.upstream_inputs,
        "damage_report_json",
        PROFILE_ID,
        "vcf.damage_filter must consume the BAM damage evidence handoff",
    )?;
    require_list_contains(
        &vcf_damage_filter.external_inputs,
        "damage_filter_policy_contract",
        PROFILE_ID,
        "vcf.damage_filter must declare the governed damage-filter policy contract",
    )?;
    require_list_contains(
        &vcf_stats.depends_on,
        "vcf.damage_filter",
        PROFILE_ID,
        "vcf.stats must remain downstream of damage-aware filtering",
    )?;
    require_list_contains(
        &trim_terminal_damage.external_inputs,
        "terminal_damage_policy_contract",
        PROFILE_ID,
        "fastq.trim_terminal_damage must declare the governed terminal-damage trimming policy contract",
    )?;

    let compatibility_report = evidence.corpus_stage_compatibility()?;
    let compatibility_index = compatibility_report
        .stages
        .iter()
        .map(|entry| (entry.stage_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.trim_terminal_damage",
        "corpus-01",
        "corpus-01-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.damage",
        "corpus-01-adna-bam",
        "corpus-01-adna-damage-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.authenticity",
        "corpus-01-adna-bam",
        "corpus-01-adna-damage-mini",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-01-mini for governed aDNA-like FASTQ inputs".to_string(),
        "fastq.trim_terminal_damage is fixture-backed by corpus-01-mini".to_string(),
        "fastq.remove_duplicates stays downstream of terminal-damage trimming".to_string(),
        "bam.damage and bam.authenticity stay fixture-backed by corpus-01-adna-damage-mini".to_string(),
        "bam.contamination declares the ancient-DNA contamination panel contract".to_string(),
        "vcf.call_pseudohaploid consumes BAM damage and authenticity evidence with ancient-DNA reference contracts".to_string(),
        "vcf.damage_filter stays downstream of BAM damage evidence and pseudohaploid calling".to_string(),
        "vcf.stats stays downstream of damage-aware filtering".to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

pub(super) fn validate_ancient_dna_gl_profile(
    evidence: &dyn PipelineProfileEvidence,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "ancient_dna_gl";

    if config.default_corpus_id != "corpus-01-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed FASTQ corpus `corpus-01-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let trim_terminal_damage =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.trim_terminal_damage")?;
    let remove_duplicates =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.remove_duplicates")?;
    let bam_align = require_profile_stage(&stage_index, PROFILE_ID, "bam.align")?;
    let bam_damage = require_profile_stage(&stage_index, PROFILE_ID, "bam.damage")?;
    let bam_contamination = require_profile_stage(&stage_index, PROFILE_ID, "bam.contamination")?;
    let bam_authenticity = require_profile_stage(&stage_index, PROFILE_ID, "bam.authenticity")?;
    let vcf_call_gl = require_profile_stage(&stage_index, PROFILE_ID, "vcf.call_gl")?;
    let vcf_gl_propagation = require_profile_stage(&stage_index, PROFILE_ID, "vcf.gl_propagation")?;
    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;

    require_list_contains(
        &remove_duplicates.upstream_inputs,
        "terminal_damage_trimmed_reads_r1_path",
        PROFILE_ID,
        "fastq.remove_duplicates must consume the terminal-damage-trimmed R1 handoff",
    )?;
    require_list_contains(
        &remove_duplicates.upstream_inputs,
        "terminal_damage_trimmed_reads_r2_path",
        PROFILE_ID,
        "fastq.remove_duplicates must consume the terminal-damage-trimmed R2 handoff",
    )?;
    require_list_contains(
        &bam_align.depends_on,
        "fastq.filter_reads",
        PROFILE_ID,
        "bam.align must remain downstream of the duplicate-aware FASTQ preprocessing branch",
    )?;
    require_list_contains(
        &bam_damage.external_inputs,
        "expected_damage_contract",
        PROFILE_ID,
        "bam.damage must declare the governed expected-damage asset contract",
    )?;
    require_list_contains(
        &bam_contamination.external_inputs,
        "adna_contamination_panel_contract",
        PROFILE_ID,
        "bam.contamination must declare the governed ancient-DNA contamination panel contract",
    )?;
    require_list_contains(
        &bam_authenticity.external_inputs,
        "adna_authenticity_policy_contract",
        PROFILE_ID,
        "bam.authenticity must declare the governed ancient-DNA authenticity policy contract",
    )?;
    require_list_contains(
        &vcf_call_gl.upstream_inputs,
        "damage_report_json",
        PROFILE_ID,
        "vcf.call_gl must consume the BAM damage evidence handoff",
    )?;
    require_list_contains(
        &vcf_call_gl.upstream_inputs,
        "authenticity_report_json",
        PROFILE_ID,
        "vcf.call_gl must consume the BAM authenticity evidence handoff",
    )?;
    require_list_contains(
        &vcf_call_gl.external_inputs,
        "adna_reference_fasta_contract",
        PROFILE_ID,
        "vcf.call_gl must declare the governed ancient-DNA reference FASTA contract",
    )?;
    require_list_contains(
        &vcf_call_gl.external_inputs,
        "adna_reference_fai_contract",
        PROFILE_ID,
        "vcf.call_gl must declare the governed ancient-DNA reference index contract",
    )?;
    require_list_contains(
        &vcf_call_gl.outputs,
        "gl_sites_vcf",
        PROFILE_ID,
        "vcf.call_gl must produce the likelihood-bearing VCF handoff",
    )?;
    require_list_contains(
        &vcf_gl_propagation.depends_on,
        "vcf.call_gl",
        PROFILE_ID,
        "vcf.gl_propagation must remain downstream of genotype-likelihood calling",
    )?;
    require_list_contains(
        &vcf_gl_propagation.upstream_inputs,
        "gl_sites_vcf",
        PROFILE_ID,
        "vcf.gl_propagation must consume the genotype-likelihood VCF handoff",
    )?;
    require_list_contains(
        &vcf_gl_propagation.outputs,
        "gl_propagated_vcf",
        PROFILE_ID,
        "vcf.gl_propagation must keep a propagated likelihood-bearing VCF handoff explicit",
    )?;
    require_list_contains(
        &vcf_qc.depends_on,
        "vcf.gl_propagation",
        PROFILE_ID,
        "vcf.qc must remain downstream of genotype-likelihood propagation",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "gl_propagated_vcf",
        PROFILE_ID,
        "vcf.qc must consume the propagated genotype-likelihood VCF handoff",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "gl_propagation_report_json",
        PROFILE_ID,
        "vcf.qc must consume explicit genotype-likelihood propagation evidence",
    )?;
    require_list_contains(
        &trim_terminal_damage.external_inputs,
        "terminal_damage_policy_contract",
        PROFILE_ID,
        "fastq.trim_terminal_damage must declare the governed terminal-damage trimming policy contract",
    )?;

    let compatibility_report = evidence.corpus_stage_compatibility()?;
    let compatibility_index = compatibility_report
        .stages
        .iter()
        .map(|entry| (entry.stage_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.trim_terminal_damage",
        "corpus-01",
        "corpus-01-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.damage",
        "corpus-01-adna-bam",
        "corpus-01-adna-damage-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.authenticity",
        "corpus-01-adna-bam",
        "corpus-01-adna-damage-mini",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-01-mini for governed aDNA-like FASTQ inputs".to_string(),
        "fastq.trim_terminal_damage is fixture-backed by corpus-01-mini".to_string(),
        "fastq.remove_duplicates stays downstream of terminal-damage trimming".to_string(),
        "bam.damage and bam.authenticity stay fixture-backed by corpus-01-adna-damage-mini".to_string(),
        "bam.contamination declares the ancient-DNA contamination panel contract".to_string(),
        "vcf.call_gl consumes BAM damage and authenticity evidence with ancient-DNA reference contracts".to_string(),
        "vcf.gl_propagation stays downstream of genotype-likelihood calling with explicit GL handoff coverage".to_string(),
        "vcf.qc stays downstream of propagated genotype-likelihood outputs".to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}
