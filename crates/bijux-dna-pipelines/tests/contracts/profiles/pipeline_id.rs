use bijux_dna_pipelines::{validate_pipeline_id_str, PipelineId};

#[test]
fn pipeline_id_validator_rejects_invalid_names() {
    let invalid = [
        "fastq__default__v1",
        "fastq-to-bam__adna_shotgun__v",
        "fastq-to-bam__adna_shotgun__v1_extra",
        "fastq_to_bam__adna_shotgun__v1",
        "fastq-to-bam__adna_shotgun__vX",
        "fastq-to-bam__adna shot__v1",
    ];
    for id in invalid {
        assert!(
            validate_pipeline_id_str(id).is_err(),
            "expected invalid pipeline id: {id}"
        );
    }
}

#[test]
fn pipeline_id_validator_accepts_valid_name() {
    let id = PipelineId::new("fastq-to-bam__adna_shotgun__v1");
    validate_pipeline_id_str(id.as_str())
        .unwrap_or_else(|_| panic!("expected valid pipeline id: {id}"));
}
