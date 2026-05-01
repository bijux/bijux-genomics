use bijux_dna_pipelines::bam::{
    bam_workflow_template_by_id, bam_workflow_templates, bam_workflow_templates_for_pipeline,
};
use bijux_dna_pipelines::cross::{
    cross_workflow_template_by_id, cross_workflow_templates, cross_workflow_templates_for_pipeline,
};
use bijux_dna_pipelines::fastq::{
    fastq_workflow_template_by_id, fastq_workflow_templates, fastq_workflow_templates_for_pipeline,
};
use bijux_dna_pipelines::vcf::{
    vcf_workflow_template_by_id, vcf_workflow_templates, vcf_workflow_templates_for_pipeline,
};

#[test]
fn workflow_template_registry_is_distinct_and_lookupable() {
    let templates = cross_workflow_templates();
    let mut ids =
        templates.iter().map(|template| template.template_id.as_str()).collect::<Vec<_>>();
    let mut sorted = ids.clone();
    ids.sort_unstable();
    sorted.sort_unstable();
    sorted.dedup();

    assert_eq!(ids, sorted, "workflow template ids must stay unique");
    assert!(cross_workflow_template_by_id("cross.bam_to_vcf_default").is_some());
    assert_eq!(
        cross_workflow_templates_for_pipeline("bam-to-vcf__default__v1")
            .into_iter()
            .map(|template| template.template_id)
            .collect::<Vec<_>>(),
        vec!["cross.bam_to_vcf_default".to_string()]
    );
}

#[test]
fn fastq_workflow_template_registry_is_distinct_and_lookupable() {
    let templates = fastq_workflow_templates();
    let mut ids =
        templates.iter().map(|template| template.template_id.as_str()).collect::<Vec<_>>();
    let mut sorted = ids.clone();
    ids.sort_unstable();
    sorted.sort_unstable();
    sorted.dedup();

    assert_eq!(ids, sorted, "FASTQ workflow template ids must stay unique");
    assert!(fastq_workflow_template_by_id("fastq.qc_only_review").is_some());
    assert_eq!(
        fastq_workflow_templates_for_pipeline("fastq-to-fastq__trim_qc__v1")
            .into_iter()
            .map(|template| template.template_id)
            .collect::<Vec<_>>(),
        vec!["fastq.trim_qc".to_string(), "fastq.preprocessing_policy_diff".to_string(),]
    );
}

#[test]
fn bam_workflow_template_registry_is_distinct_and_lookupable() {
    let templates = bam_workflow_templates();
    let mut ids =
        templates.iter().map(|template| template.template_id.as_str()).collect::<Vec<_>>();
    let mut sorted = ids.clone();
    ids.sort_unstable();
    sorted.sort_unstable();
    sorted.dedup();

    assert_eq!(ids, sorted, "BAM workflow template ids must stay unique");
    assert!(bam_workflow_template_by_id("bam.modern_wgs_qc").is_some());
    assert_eq!(
        bam_workflow_templates_for_pipeline("bam-to-bam__adna_capture__v1")
            .into_iter()
            .map(|template| template.template_id)
            .collect::<Vec<_>>(),
        vec!["bam.targeted_amplicon_qc".to_string()]
    );
}

#[test]
fn vcf_workflow_template_registry_is_distinct_and_lookupable() {
    let templates = vcf_workflow_templates();
    let mut ids =
        templates.iter().map(|template| template.template_id.as_str()).collect::<Vec<_>>();
    let mut sorted = ids.clone();
    ids.sort_unstable();
    sorted.sort_unstable();
    sorted.dedup();

    assert_eq!(ids, sorted, "VCF workflow template ids must stay unique");
    assert!(vcf_workflow_template_by_id("vcf.validation_normalization_qc").is_some());
    assert_eq!(
        vcf_workflow_templates_for_pipeline("vcf-to-vcf__reference_basic__v1")
            .into_iter()
            .map(|template| template.template_id)
            .collect::<Vec<_>>(),
        vec![
            "vcf.validation_normalization_qc".to_string(),
            "vcf.cohort_qc_review".to_string(),
            "vcf.population_structure_guardrail".to_string(),
            "vcf.roh_ibd_boundary".to_string(),
            "vcf.annotation_provenance_workflow".to_string(),
        ]
    );
}
