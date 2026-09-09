use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use super::{
    require_compatibility_fixture, require_list_contains, require_profile_stage,
    LocalPipelineDagConfig, LocalPipelineDagValidationProfileReport, PipelineProfileEvidence,
};

pub(super) fn validate_diploid_small_sample_profile(
    evidence: &dyn PipelineProfileEvidence,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "diploid_small_sample";

    if config.default_corpus_id != "corpus-01-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed FASTQ corpus `corpus-01-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let bam_filter = require_profile_stage(&stage_index, PROFILE_ID, "bam.filter")?;
    let bam_recalibration = require_profile_stage(&stage_index, PROFILE_ID, "bam.recalibration")?;
    let vcf_call_diploid = require_profile_stage(&stage_index, PROFILE_ID, "vcf.call_diploid")?;
    let vcf_filter = require_profile_stage(&stage_index, PROFILE_ID, "vcf.filter")?;
    let vcf_stats = require_profile_stage(&stage_index, PROFILE_ID, "vcf.stats")?;
    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;
    let vcf_phasing = require_profile_stage(&stage_index, PROFILE_ID, "vcf.phasing")?;

    require_list_contains(
        &bam_filter.outputs,
        "filtered_bam",
        PROFILE_ID,
        "bam.filter must keep the filtered-BAM fallback output explicit",
    )?;
    require_list_contains(
        &bam_filter.outputs,
        "filtered_bai",
        PROFILE_ID,
        "bam.filter must keep the filtered-BAM index fallback output explicit",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_reference_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed recalibration reference contract",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_known_sites_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed known-sites recalibration contract",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_coverage_gate_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed low-coverage recalibration gate contract",
    )?;
    require_list_contains(
        &vcf_call_diploid.depends_on,
        "bam.filter",
        PROFILE_ID,
        "vcf.call_diploid must remain downstream of the filtered-BAM fallback branch",
    )?;
    require_list_contains(
        &vcf_call_diploid.depends_on,
        "bam.recalibration",
        PROFILE_ID,
        "vcf.call_diploid must remain downstream of the recalibrated-BAM run branch",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "filtered_bam",
        PROFILE_ID,
        "vcf.call_diploid must consume the filtered-BAM fallback handoff",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "filtered_bai",
        PROFILE_ID,
        "vcf.call_diploid must consume the filtered-BAM index fallback handoff",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "recalibrated_bam",
        PROFILE_ID,
        "vcf.call_diploid must consume the recalibrated-BAM run-path handoff",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "recalibrated_bai",
        PROFILE_ID,
        "vcf.call_diploid must consume the recalibrated-BAM index run-path handoff",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "recalibration_summary_json",
        PROFILE_ID,
        "vcf.call_diploid must consume the recalibration decision summary",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "coverage_report_json",
        PROFILE_ID,
        "vcf.call_diploid must consume explicit coverage evidence for the low-coverage skip path",
    )?;
    require_list_contains(
        &vcf_call_diploid.external_inputs,
        "reference_fasta_contract",
        PROFILE_ID,
        "vcf.call_diploid must declare the governed reference FASTA contract",
    )?;
    require_list_contains(
        &vcf_call_diploid.external_inputs,
        "reference_fai_contract",
        PROFILE_ID,
        "vcf.call_diploid must declare the governed reference FASTA index contract",
    )?;
    require_list_contains(
        &vcf_filter.depends_on,
        "vcf.call_diploid",
        PROFILE_ID,
        "vcf.filter must remain downstream of diploid calling",
    )?;
    require_list_contains(
        &vcf_filter.upstream_inputs,
        "diploid_vcf",
        PROFILE_ID,
        "vcf.filter must consume the diploid VCF handoff",
    )?;
    require_list_contains(
        &vcf_stats.depends_on,
        "vcf.filter",
        PROFILE_ID,
        "vcf.stats must remain downstream of variant filtering",
    )?;
    require_list_contains(
        &vcf_qc.depends_on,
        "vcf.filter",
        PROFILE_ID,
        "vcf.qc must remain downstream of filtered VCF outputs",
    )?;
    require_list_contains(
        &vcf_qc.depends_on,
        "vcf.stats",
        PROFILE_ID,
        "vcf.qc must remain downstream of explicit VCF stats evidence",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "filtered_vcf",
        PROFILE_ID,
        "vcf.qc must consume the filtered VCF handoff",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "stats_json",
        PROFILE_ID,
        "vcf.qc must consume explicit VCF stats evidence",
    )?;
    if vcf_qc.depends_on.iter().any(|value| value == "vcf.phasing") {
        return Err(anyhow!("{PROFILE_ID}: vcf.qc must not depend on optional phasing output"));
    }
    require_list_contains(
        &vcf_phasing.depends_on,
        "vcf.filter",
        PROFILE_ID,
        "vcf.phasing must remain downstream of the filtered VCF handoff",
    )?;
    require_list_contains(
        &vcf_phasing.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.phasing must remain downstream of VCF QC instead of blocking it",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "filtered_vcf",
        PROFILE_ID,
        "vcf.phasing must consume the filtered VCF handoff",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "qc_report",
        PROFILE_ID,
        "vcf.phasing must consume the QC report from the completed small-sample branch",
    )?;
    require_list_contains(
        &vcf_phasing.external_inputs,
        "genetic_map_contract",
        PROFILE_ID,
        "vcf.phasing must declare the governed genetic map contract",
    )?;
    require_list_contains(
        &vcf_phasing.external_inputs,
        "reference_panel_lock_contract",
        PROFILE_ID,
        "vcf.phasing must declare the governed reference-panel lock contract",
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
        "bam.filter",
        "corpus-01-bam",
        "corpus-01-bam-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.recalibration",
        "corpus-01-bam",
        "corpus-01-bam-mini",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-01-mini for governed small-sample FASTQ inputs"
            .to_string(),
        "bam.filter and bam.recalibration stay fixture-backed by corpus-01-bam-mini"
            .to_string(),
        "bam.recalibration keeps the low-coverage gate and known-sites contracts explicit"
            .to_string(),
        "vcf.call_diploid consumes both the filtered-BAM fallback and recalibrated-BAM run-path handoffs"
            .to_string(),
        "vcf.call_diploid keeps explicit coverage and recalibration decision evidence"
            .to_string(),
        "vcf.filter and vcf.stats stay downstream of diploid calling".to_string(),
        "vcf.qc stays downstream of filtered VCF and stats without depending on phasing"
            .to_string(),
        "vcf.phasing remains an optional downstream branch with explicit map and panel contracts"
            .to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

pub(super) fn validate_bam_genotyping_vcf_downstream_profile(
    evidence: &dyn PipelineProfileEvidence,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "bam_genotyping_vcf_downstream";

    if config.default_corpus_id != "corpus-01-bam-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed BAM corpus `corpus-01-bam-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let bam_filter = require_profile_stage(&stage_index, PROFILE_ID, "bam.filter")?;
    let bam_coverage = require_profile_stage(&stage_index, PROFILE_ID, "bam.coverage")?;
    let bam_recalibration = require_profile_stage(&stage_index, PROFILE_ID, "bam.recalibration")?;
    let bam_genotyping = require_profile_stage(&stage_index, PROFILE_ID, "bam.genotyping")?;
    let vcf_filter = require_profile_stage(&stage_index, PROFILE_ID, "vcf.filter")?;
    let vcf_stats = require_profile_stage(&stage_index, PROFILE_ID, "vcf.stats")?;
    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;
    let vcf_pca = require_profile_stage(&stage_index, PROFILE_ID, "vcf.pca")?;
    let vcf_roh = require_profile_stage(&stage_index, PROFILE_ID, "vcf.roh")?;

    let compatibility_report = evidence.corpus_stage_compatibility()?;
    let compatibility_index = compatibility_report
        .stages
        .iter()
        .map(|entry| (entry.stage_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.filter",
        "corpus-01-bam",
        "corpus-01-bam-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.coverage",
        "corpus-01-bam",
        "corpus-01-bam-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.recalibration",
        "corpus-01-bam",
        "corpus-01-bam-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.genotyping",
        "corpus-01-genotyping",
        "corpus-01-genotyping-mini",
    )?;

    require_list_contains(
        &bam_filter.outputs,
        "filtered_bam",
        PROFILE_ID,
        "bam.filter must export the governed filtered BAM handoff",
    )?;
    require_list_contains(
        &bam_filter.outputs,
        "filtered_bai",
        PROFILE_ID,
        "bam.filter must export the governed filtered BAM index handoff",
    )?;
    require_list_contains(
        &bam_coverage.depends_on,
        "bam.filter",
        PROFILE_ID,
        "bam.coverage must remain downstream of bam.filter",
    )?;
    require_list_contains(
        &bam_coverage.external_inputs,
        "coverage_region_contract",
        PROFILE_ID,
        "bam.coverage must declare the governed coverage-region contract",
    )?;
    require_list_contains(
        &bam_recalibration.depends_on,
        "bam.filter",
        PROFILE_ID,
        "bam.recalibration must remain downstream of bam.filter",
    )?;
    require_list_contains(
        &bam_recalibration.depends_on,
        "bam.coverage",
        PROFILE_ID,
        "bam.recalibration must remain downstream of bam.coverage",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_reference_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed recalibration reference contract",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_known_sites_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed recalibration known-sites contract",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_coverage_gate_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed recalibration coverage gate contract",
    )?;

    require_list_contains(
        &bam_genotyping.depends_on,
        "bam.filter",
        PROFILE_ID,
        "bam.genotyping must remain downstream of bam.filter",
    )?;
    require_list_contains(
        &bam_genotyping.depends_on,
        "bam.recalibration",
        PROFILE_ID,
        "bam.genotyping must remain downstream of bam.recalibration",
    )?;
    require_list_contains(
        &bam_genotyping.upstream_inputs,
        "filtered_bam",
        PROFILE_ID,
        "bam.genotyping must consume the governed filtered BAM fallback handoff",
    )?;
    require_list_contains(
        &bam_genotyping.upstream_inputs,
        "filtered_bai",
        PROFILE_ID,
        "bam.genotyping must consume the governed filtered BAM index fallback handoff",
    )?;
    require_list_contains(
        &bam_genotyping.upstream_inputs,
        "recalibrated_bam",
        PROFILE_ID,
        "bam.genotyping must consume the governed recalibrated BAM run-path handoff",
    )?;
    require_list_contains(
        &bam_genotyping.upstream_inputs,
        "recalibrated_bai",
        PROFILE_ID,
        "bam.genotyping must consume the governed recalibrated BAM index run-path handoff",
    )?;
    require_list_contains(
        &bam_genotyping.upstream_inputs,
        "recalibration_summary_json",
        PROFILE_ID,
        "bam.genotyping must consume the governed recalibration decision summary",
    )?;
    require_list_contains(
        &bam_genotyping.external_inputs,
        "genotyping_reference_contract",
        PROFILE_ID,
        "bam.genotyping must declare the governed genotyping reference contract",
    )?;
    require_list_contains(
        &bam_genotyping.external_inputs,
        "candidate_sites_vcf_contract",
        PROFILE_ID,
        "bam.genotyping must declare the governed candidate-sites VCF contract",
    )?;
    require_list_contains(
        &bam_genotyping.external_inputs,
        "target_regions_contract",
        PROFILE_ID,
        "bam.genotyping must declare the governed target-regions contract",
    )?;
    require_list_contains(
        &bam_genotyping.outputs,
        "genotyping_bcf",
        PROFILE_ID,
        "bam.genotyping must export the governed BCF bridge artifact",
    )?;
    require_list_contains(
        &bam_genotyping.outputs,
        "genotyping_vcf_gz",
        PROFILE_ID,
        "bam.genotyping must export the governed gzipped VCF bridge artifact",
    )?;
    require_list_contains(
        &bam_genotyping.outputs,
        "genotyping_vcf_tbi",
        PROFILE_ID,
        "bam.genotyping must export the governed tabix bridge artifact",
    )?;

    require_list_contains(
        &vcf_filter.depends_on,
        "bam.genotyping",
        PROFILE_ID,
        "vcf.filter must remain downstream of bam.genotyping",
    )?;
    require_list_contains(
        &vcf_filter.upstream_inputs,
        "genotyping_vcf_gz",
        PROFILE_ID,
        "vcf.filter must consume the exact bam.genotyping VCF handoff",
    )?;
    require_list_contains(
        &vcf_filter.upstream_inputs,
        "genotyping_vcf_tbi",
        PROFILE_ID,
        "vcf.filter must consume the exact bam.genotyping tabix handoff",
    )?;
    require_list_contains(
        &vcf_filter.upstream_inputs,
        "genotyping_report_json",
        PROFILE_ID,
        "vcf.filter must consume explicit bam.genotyping bridge evidence",
    )?;
    if vcf_filter
        .external_inputs
        .iter()
        .any(|value| value == "corpus.target_cohort_vcf" || value == "corpus.target_cohort_vcf_tbi")
    {
        return Err(anyhow!(
            "{PROFILE_ID}: vcf.filter must bridge from bam.genotyping outputs instead of a conceptual external cohort-vcf placeholder"
        ));
    }

    require_list_contains(
        &vcf_stats.depends_on,
        "vcf.filter",
        PROFILE_ID,
        "vcf.stats must remain downstream of the filtered VCF bridge",
    )?;
    require_list_contains(
        &vcf_stats.upstream_inputs,
        "filtered_vcf",
        PROFILE_ID,
        "vcf.stats must consume the filtered VCF bridge handoff",
    )?;
    require_list_contains(
        &vcf_stats.upstream_inputs,
        "filtered_vcf_tbi",
        PROFILE_ID,
        "vcf.stats must consume the filtered VCF bridge tabix handoff",
    )?;

    require_list_contains(
        &vcf_qc.depends_on,
        "vcf.filter",
        PROFILE_ID,
        "vcf.qc must remain downstream of filtered VCF outputs",
    )?;
    require_list_contains(
        &vcf_qc.depends_on,
        "vcf.stats",
        PROFILE_ID,
        "vcf.qc must remain downstream of explicit VCF stats evidence",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "filtered_vcf",
        PROFILE_ID,
        "vcf.qc must consume the filtered VCF bridge handoff",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "filtered_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must consume the filtered VCF bridge tabix handoff",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "stats_json",
        PROFILE_ID,
        "vcf.qc must consume explicit VCF stats evidence",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.qc must export the qc-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must export the qc-qualified cohort tabix handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.qc must export the governed pruning handoff for optional downstream branches",
    )?;

    require_list_contains(
        &vcf_pca.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.pca must remain downstream of vcf.qc",
    )?;
    require_list_contains(
        &vcf_pca.external_inputs,
        "sample_metadata_manifest_contract",
        PROFILE_ID,
        "vcf.pca must declare the governed sample-metadata contract",
    )?;
    require_list_contains(
        &vcf_pca.external_inputs,
        "population_metadata_manifest_contract",
        PROFILE_ID,
        "vcf.pca must declare the governed population-metadata contract",
    )?;
    require_list_contains(
        &vcf_pca.external_inputs,
        "population_labels_contract",
        PROFILE_ID,
        "vcf.pca must declare the governed population-label contract",
    )?;
    require_list_contains(
        &vcf_pca.upstream_inputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.pca must consume the qc-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_pca.upstream_inputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.pca must consume the governed pruning handoff",
    )?;

    require_list_contains(
        &vcf_roh.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.roh must remain downstream of vcf.qc",
    )?;
    require_list_contains(
        &vcf_roh.upstream_inputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.roh must consume the qc-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_roh.upstream_inputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.roh must consume the governed pruning handoff",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-01-bam-mini for governed bam-genotyping bridge inputs"
            .to_string(),
        "bam.filter, bam.coverage, and bam.recalibration stay fixture-backed by corpus-01-bam-mini while bam.genotyping stays fixture-backed by corpus-01-genotyping-mini"
            .to_string(),
        "bam.genotyping keeps the filtered fallback path, recalibrated run path, and governed genotyping contracts explicit"
            .to_string(),
        "vcf.filter consumes exact bam.genotyping vcf handoffs instead of a conceptual cohort-vcf placeholder"
            .to_string(),
        "vcf.stats stays downstream of the filtered vcf bridge".to_string(),
        "vcf.qc stays downstream of filtered vcf and stats while exporting qc-qualified cohort outputs"
            .to_string(),
        "optional vcf.pca stays downstream of qc with metadata contracts and pruned cohort handoffs"
            .to_string(),
        "optional vcf.roh stays downstream of qc with the same qc-qualified cohort handoff"
            .to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}
