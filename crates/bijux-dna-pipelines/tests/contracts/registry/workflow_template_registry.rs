use bijux_dna_pipelines::cross::{
    cross_workflow_template_by_id, cross_workflow_templates, cross_workflow_templates_for_pipeline,
};
use bijux_dna_pipelines::fastq::{
    fastq_workflow_template_by_id, fastq_workflow_templates, fastq_workflow_templates_for_pipeline,
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
