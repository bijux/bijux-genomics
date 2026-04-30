use bijux_dna_core::id_catalog;
use bijux_dna_core::ids::{
    validate_artifact_id_str, validate_pipeline_id_str, validate_profile_id_str,
    validate_stage_id_str, validate_tool_id_str,
};

#[test]
fn pipeline_catalog_constants_match_pipeline_validation() {
    let pipelines = [
        id_catalog::PIPELINE_FASTQ_DEFAULT,
        id_catalog::PIPELINE_FASTQ_MINIMAL,
        id_catalog::PIPELINE_FASTQ_ADNA,
        id_catalog::PIPELINE_FASTQ_AMPLICON_STANDARD,
        id_catalog::PIPELINE_FASTQ_AMPLICON_UMI,
        id_catalog::PIPELINE_FASTQ_CONTAMINANT_DEPLETION,
        id_catalog::PIPELINE_FASTQ_EDNA_METABARCODING,
        id_catalog::PIPELINE_FASTQ_HOST_DEPLETION,
        id_catalog::PIPELINE_FASTQ_QC_ONLY,
        id_catalog::PIPELINE_FASTQ_REFERENCE_ADNA,
        id_catalog::PIPELINE_FASTQ_RRNA_DEPLETION,
        id_catalog::PIPELINE_FASTQ_TRIM_QC,
        id_catalog::PIPELINE_FASTQ_UMI,
        id_catalog::PIPELINE_FASTQ_TO_BAM_DEFAULT,
        id_catalog::PIPELINE_FASTQ_TO_BAM_ADNA_SHOTGUN,
        id_catalog::PIPELINE_BAM_DEFAULT,
        id_catalog::PIPELINE_BAM_ADNA_SHOTGUN,
        id_catalog::PIPELINE_BAM_ADNA_CAPTURE,
        id_catalog::PIPELINE_BAM_REFERENCE_ADNA,
        id_catalog::PIPELINE_VCF_MINIMAL,
        id_catalog::PIPELINE_VCF_REFERENCE_BASIC,
    ];

    for pipeline in pipelines {
        assert!(
            validate_pipeline_id_str(pipeline).is_ok(),
            "pipeline catalog constant must stay valid: {pipeline}"
        );
    }
}

#[test]
fn stage_catalog_constants_keep_domain_prefixes() {
    let fastq_stages = [
        id_catalog::FASTQ_VALIDATE_READS,
        id_catalog::FASTQ_TRIM,
        id_catalog::FASTQ_FILTER,
        id_catalog::FASTQ_MERGE,
        id_catalog::FASTQ_QC_POST,
    ];
    let bam_stages = [
        id_catalog::BAM_ALIGN,
        id_catalog::BAM_VALIDATE,
        id_catalog::BAM_MARKDUP,
        id_catalog::BAM_COVERAGE,
    ];
    let vcf_stages = [id_catalog::VCF_CALL, id_catalog::VCF_FILTER, id_catalog::VCF_STATS];

    for stage in fastq_stages {
        assert!(stage.starts_with(id_catalog::FASTQ_PREFIX));
        assert!(validate_stage_id_str(stage).is_ok());
    }
    for stage in bam_stages {
        assert!(stage.starts_with(id_catalog::BAM_PREFIX));
        assert!(validate_stage_id_str(stage).is_ok());
    }
    for stage in vcf_stages {
        assert!(stage.starts_with(id_catalog::VCF_PREFIX));
        assert!(validate_stage_id_str(stage).is_ok());
    }
}

#[test]
fn tool_and_free_form_catalog_identifiers_remain_valid() {
    let tools = [
        id_catalog::TOOL_FASTQVALIDATOR_OFFICIAL,
        id_catalog::TOOL_FASTP,
        id_catalog::TOOL_ADAPTERREMOVAL,
        id_catalog::TOOL_BOWTIE2,
        id_catalog::TOOL_DADA2,
        id_catalog::TOOL_KRAKEN2,
        id_catalog::TOOL_SAMTOOLS,
        id_catalog::TOOL_SORTMERNA,
        id_catalog::TOOL_BCFTOOLS,
    ];

    for tool in tools {
        assert!(validate_tool_id_str(tool).is_ok());
    }

    assert!(validate_artifact_id_str("reads_out").is_ok());
    assert!(validate_profile_id_str("reference_adna").is_ok());
}
