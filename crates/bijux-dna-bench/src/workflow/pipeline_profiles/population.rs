//! Owner: bijux-dna-bench
use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use super::{
    require_list_contains, require_profile_stage, require_vcf_matrix_binding,
    LocalPipelineDagConfig, LocalPipelineDagValidationProfileReport, PipelineProfileEvidence,
};

pub(super) fn validate_population_structure_vcf_profile(
    evidence: &dyn PipelineProfileEvidence,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "population_structure_vcf";

    if config.default_corpus_id != "vcf_production_regression" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed VCF corpus `vcf_production_regression`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;
    let vcf_pca = require_profile_stage(&stage_index, PROFILE_ID, "vcf.pca")?;
    let vcf_admixture = require_profile_stage(&stage_index, PROFILE_ID, "vcf.admixture")?;
    let vcf_population_structure =
        require_profile_stage(&stage_index, PROFILE_ID, "vcf.population_structure")?;

    let matrix_index = evidence.vcf_stage_matrix()?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.qc",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.pca",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.admixture",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.population_structure",
        "vcf_cohort",
        "vcf_production_regression",
    )?;

    require_list_contains(
        &vcf_qc.external_inputs,
        "corpus.target_cohort_vcf",
        PROFILE_ID,
        "vcf.qc must declare the governed target-cohort VCF input",
    )?;
    require_list_contains(
        &vcf_qc.external_inputs,
        "corpus.target_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must declare the governed target-cohort tabix input",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.qc must export the QC-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must export the QC-qualified cohort tabix handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.qc must export the governed pruning/filtering handoff",
    )?;

    for (stage_id, node) in [("vcf.pca", vcf_pca), ("vcf.admixture", vcf_admixture)] {
        require_list_contains(
            &node.external_inputs,
            "sample_metadata_manifest_contract",
            PROFILE_ID,
            &format!("{stage_id} must declare the governed sample-metadata contract"),
        )?;
        require_list_contains(
            &node.external_inputs,
            "population_labels_contract",
            PROFILE_ID,
            &format!("{stage_id} must declare the governed population-label contract"),
        )?;
        require_list_contains(
            &node.external_inputs,
            "population_metadata_manifest_contract",
            PROFILE_ID,
            &format!("{stage_id} must declare the governed population-metadata contract"),
        )?;
        require_list_contains(
            &node.depends_on,
            "vcf.qc",
            PROFILE_ID,
            &format!("{stage_id} must remain downstream of cohort QC and pruning/filtering"),
        )?;
        require_list_contains(
            &node.upstream_inputs,
            "qc_cohort_vcf",
            PROFILE_ID,
            &format!("{stage_id} must consume the QC-qualified cohort VCF handoff"),
        )?;
        require_list_contains(
            &node.upstream_inputs,
            "pruned_variants_tsv",
            PROFILE_ID,
            &format!("{stage_id} must consume the governed pruning/filtering handoff"),
        )?;
    }

    require_list_contains(
        &vcf_pca.outputs,
        "pca_report",
        PROFILE_ID,
        "vcf.pca must export the normalized PCA report",
    )?;
    require_list_contains(
        &vcf_pca.outputs,
        "pca_metadata_join_tsv",
        PROFILE_ID,
        "vcf.pca must export a sample-metadata join handoff",
    )?;

    require_list_contains(
        &vcf_admixture.depends_on,
        "vcf.pca",
        PROFILE_ID,
        "vcf.admixture must remain downstream of PCA in the governed population-structure branch",
    )?;
    require_list_contains(
        &vcf_admixture.upstream_inputs,
        "pca_report",
        PROFILE_ID,
        "vcf.admixture must consume explicit PCA evidence",
    )?;
    require_list_contains(
        &vcf_admixture.outputs,
        "admixture_report",
        PROFILE_ID,
        "vcf.admixture must export the normalized admixture report",
    )?;
    require_list_contains(
        &vcf_admixture.outputs,
        "admixture_metadata_join_tsv",
        PROFILE_ID,
        "vcf.admixture must export a sample-metadata join handoff",
    )?;

    require_list_contains(
        &vcf_population_structure.external_inputs,
        "sample_metadata_manifest_contract",
        PROFILE_ID,
        "vcf.population_structure must declare the governed sample-metadata contract",
    )?;
    require_list_contains(
        &vcf_population_structure.external_inputs,
        "population_labels_contract",
        PROFILE_ID,
        "vcf.population_structure must declare the governed population-label contract",
    )?;
    require_list_contains(
        &vcf_population_structure.external_inputs,
        "population_metadata_manifest_contract",
        PROFILE_ID,
        "vcf.population_structure must declare the governed population-metadata contract",
    )?;
    require_list_contains(
        &vcf_population_structure.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.population_structure must remain downstream of cohort QC",
    )?;
    require_list_contains(
        &vcf_population_structure.depends_on,
        "vcf.pca",
        PROFILE_ID,
        "vcf.population_structure must remain downstream of PCA",
    )?;
    require_list_contains(
        &vcf_population_structure.depends_on,
        "vcf.admixture",
        PROFILE_ID,
        "vcf.population_structure must remain downstream of admixture",
    )?;
    require_list_contains(
        &vcf_population_structure.upstream_inputs,
        "pca_report",
        PROFILE_ID,
        "vcf.population_structure must consume PCA evidence",
    )?;
    require_list_contains(
        &vcf_population_structure.upstream_inputs,
        "admixture_report",
        PROFILE_ID,
        "vcf.population_structure must consume admixture evidence",
    )?;
    require_list_contains(
        &vcf_population_structure.upstream_inputs,
        "pca_metadata_join_tsv",
        PROFILE_ID,
        "vcf.population_structure must consume the PCA sample-metadata join handoff",
    )?;
    require_list_contains(
        &vcf_population_structure.upstream_inputs,
        "admixture_metadata_join_tsv",
        PROFILE_ID,
        "vcf.population_structure must consume the admixture sample-metadata join handoff",
    )?;

    let checks = vec![
        "default corpus anchored to vcf_production_regression for governed cohort-vcf population-structure inputs".to_string(),
        "vcf.qc, vcf.pca, vcf.admixture, and vcf.population_structure stay bound to the vcf_cohort asset profile".to_string(),
        "vcf.qc exports the governed pruning/filtering handoff for downstream cohort analysis".to_string(),
        "vcf.pca requires sample metadata, population metadata, and population labels as explicit inputs".to_string(),
        "vcf.admixture requires sample metadata, population metadata, and population labels as explicit inputs".to_string(),
        "vcf.admixture stays downstream of pca with explicit pca evidence".to_string(),
        "vcf.pca and vcf.admixture both export sample-metadata join handoffs".to_string(),
        "vcf.population_structure consumes the pca and admixture metadata-join handoffs with mandatory labels".to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

pub(super) fn validate_relatedness_segments_vcf_profile(
    evidence: &dyn PipelineProfileEvidence,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "relatedness_segments_vcf";

    if config.default_corpus_id != "vcf_production_regression" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed VCF corpus `vcf_production_regression`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;
    let vcf_ibd = require_profile_stage(&stage_index, PROFILE_ID, "vcf.ibd")?;
    let vcf_roh = require_profile_stage(&stage_index, PROFILE_ID, "vcf.roh")?;
    let vcf_demography = require_profile_stage(&stage_index, PROFILE_ID, "vcf.demography")?;

    let matrix_index = evidence.vcf_stage_matrix()?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.qc",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.ibd",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.roh",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.demography",
        "json_ibd_segments",
        "vcf_production_regression",
    )?;

    require_list_contains(
        &vcf_qc.external_inputs,
        "corpus.target_cohort_vcf",
        PROFILE_ID,
        "vcf.qc must declare the governed target-cohort VCF input",
    )?;
    require_list_contains(
        &vcf_qc.external_inputs,
        "corpus.target_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must declare the governed target-cohort tabix input",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.qc must export the QC-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must export the QC-qualified cohort tabix handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_report",
        PROFILE_ID,
        "vcf.qc must export explicit QC evidence for downstream segment analysis",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.qc must export the governed pruning and filtering handoff",
    )?;

    require_list_contains(
        &vcf_ibd.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.ibd must remain downstream of cohort QC",
    )?;
    require_list_contains(
        &vcf_ibd.upstream_inputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.ibd must consume the QC-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_ibd.upstream_inputs,
        "qc_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.ibd must consume the QC-qualified cohort tabix handoff",
    )?;
    require_list_contains(
        &vcf_ibd.upstream_inputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.ibd must consume the governed pruning and filtering handoff",
    )?;
    require_list_contains(
        &vcf_ibd.outputs,
        "ibd_segments",
        PROFILE_ID,
        "vcf.ibd must export the normalized IBD segment handoff",
    )?;
    require_list_contains(
        &vcf_ibd.outputs,
        "ibd_report",
        PROFILE_ID,
        "vcf.ibd must export the normalized IBD report",
    )?;
    require_list_contains(
        &vcf_ibd.outputs,
        "ibd_insufficient_data_json",
        PROFILE_ID,
        "vcf.ibd must export explicit insufficiency evidence for downstream demography",
    )?;

    require_list_contains(
        &vcf_roh.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.roh must remain downstream of cohort QC",
    )?;
    require_list_contains(
        &vcf_roh.upstream_inputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.roh must consume the QC-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_roh.upstream_inputs,
        "qc_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.roh must consume the QC-qualified cohort tabix handoff",
    )?;
    require_list_contains(
        &vcf_roh.upstream_inputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.roh must consume the governed pruning and filtering handoff",
    )?;
    require_list_contains(
        &vcf_roh.outputs,
        "roh_report",
        PROFILE_ID,
        "vcf.roh must export the normalized ROH report",
    )?;
    require_list_contains(
        &vcf_roh.outputs,
        "roh_segments_tsv",
        PROFILE_ID,
        "vcf.roh must export explicit ROH segment rows",
    )?;
    if vcf_roh.depends_on.iter().any(|value| value == "vcf.ibd")
        || vcf_roh.upstream_inputs.iter().any(|value| value == "ibd_segments")
    {
        return Err(anyhow!(
            "{PROFILE_ID}: vcf.roh must stay independent of the IBD insufficiency branch"
        ));
    }

    require_list_contains(
        &vcf_demography.depends_on,
        "vcf.ibd",
        PROFILE_ID,
        "vcf.demography must remain downstream of IBD inference",
    )?;
    require_list_contains(
        &vcf_demography.upstream_inputs,
        "ibd_segments",
        PROFILE_ID,
        "vcf.demography must consume the normalized IBD segment handoff",
    )?;
    require_list_contains(
        &vcf_demography.upstream_inputs,
        "ibd_report",
        PROFILE_ID,
        "vcf.demography must consume explicit IBD report evidence",
    )?;
    require_list_contains(
        &vcf_demography.upstream_inputs,
        "ibd_insufficient_data_json",
        PROFILE_ID,
        "vcf.demography must consume explicit IBD insufficiency evidence",
    )?;
    require_list_contains(
        &vcf_demography.outputs,
        "demography_report",
        PROFILE_ID,
        "vcf.demography must export the normalized demography report",
    )?;
    require_list_contains(
        &vcf_demography.outputs,
        "demography_insufficient_data_json",
        PROFILE_ID,
        "vcf.demography must export explicit downstream insufficiency evidence",
    )?;
    if vcf_demography.depends_on.iter().any(|value| value == "vcf.roh")
        || vcf_demography.upstream_inputs.iter().any(|value| value == "roh_report")
    {
        return Err(anyhow!(
            "{PROFILE_ID}: vcf.demography must stay local to the IBD branch and must not consume ROH outputs"
        ));
    }

    let checks = vec![
        "default corpus anchored to vcf_production_regression for governed cohort-vcf relatedness inputs".to_string(),
        "vcf.qc, vcf.ibd, and vcf.roh stay bound to the vcf_cohort asset profile".to_string(),
        "vcf.demography stays bound to the json_ibd_segments asset profile".to_string(),
        "vcf.qc exports the qc-qualified cohort and pruning handoffs for downstream segment analysis".to_string(),
        "vcf.ibd stays downstream of qc with explicit insufficiency evidence".to_string(),
        "vcf.roh stays downstream of qc without depending on ibd outputs".to_string(),
        "vcf.demography stays downstream only of ibd with explicit insufficiency propagation".to_string(),
        "ibd insufficiency remains local to demography and does not block roh or qc outputs".to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}
