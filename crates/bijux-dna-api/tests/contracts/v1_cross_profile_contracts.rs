use bijux_dna_api::v1::api::plan::{explain_pipeline_profile, validate_pipeline_profile};

#[test]
fn explain_cross_profile_surfaces_workflow_templates_and_batch_contracts() {
    let explain =
        explain_pipeline_profile("fastq-to-vcf__minimal__v1").expect("cross explain profile");

    assert_eq!(explain["profile_id_resolved"], "fastq-to-vcf__minimal__v1");
    assert_eq!(explain["supports_sample_sheet"], true);
    assert_eq!(
        explain["workflow_templates"][0]["template_id"],
        "cross.fastq_to_vcf_minimal"
    );
    assert!(explain["batch_semantics"]["per_sample_stages"]
        .as_array()
        .is_some_and(|stages| !stages.is_empty()));
    assert!(explain["evidence_summary"]["story_order"]
        .as_array()
        .is_some_and(|sections| !sections.is_empty()));
}

#[test]
fn validate_cross_profile_no_longer_reports_unsupported_domain_mix() {
    let validation =
        validate_pipeline_profile("bam-to-vcf__default__v1").expect("cross validate profile");

    assert_eq!(validation["profile_id"], "bam-to-vcf__default__v1");
    assert_eq!(validation["domain"], "cross");
    assert_eq!(validation["valid"], true);
    assert_eq!(validation["template_registry_consistent"], true);
    assert_eq!(validation["sample_sheet_contract_consistent"], true);
    assert_eq!(
        validation["workflow_templates"][0]["template_id"],
        "cross.bam_to_vcf_default"
    );
    assert!(validation["violations"]
        .as_array()
        .is_some_and(|violations| violations.is_empty()));
}
