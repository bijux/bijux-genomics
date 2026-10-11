//! Owner: bijux-dna-bench-population
//! Typed paths for published reports and their declared source artifacts.
use crate::pca::model::DEFAULT_INPUT_POPULATION_LABELS_NAME;
use crate::pca::model::DEFAULT_INPUT_POPULATION_METADATA_NAME;
use crate::pca::model::DEFAULT_INPUT_SAMPLE_METADATA_NAME;
use crate::pca::model::DEFAULT_INPUT_VCF_NAME;
use crate::pca::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME;
use crate::pca::model::DEFAULT_OUTPUT_TSV_NAME;
use crate::pca::model::DEFAULT_STAGE_RESULT_NAME;
use std::path::Path;

pub(crate) struct PublishedPcaArtifacts {
    pub(crate) pca_tsv: std::path::PathBuf,
    pub(crate) pca_json: std::path::PathBuf,
    pub(crate) input_vcf: std::path::PathBuf,
    pub(crate) sample_metadata: std::path::PathBuf,
    pub(crate) population_metadata: std::path::PathBuf,
    pub(crate) population_labels_manifest: std::path::PathBuf,
    pub(crate) source_eigenvec: std::path::PathBuf,
    pub(crate) source_eigenval: std::path::PathBuf,
    pub(crate) source_pca_manifest: std::path::PathBuf,
    pub(crate) source_logs: std::path::PathBuf,
    pub(crate) stage_result_manifest: std::path::PathBuf,
}

pub(crate) fn publication_layout(published_output_root: &Path) -> PublishedPcaArtifacts {
    let published_artifacts_root = published_output_root.join("artifacts");
    let published_input_root = published_artifacts_root.join("input");
    let published_pca_tsv_path = published_output_root.join(DEFAULT_OUTPUT_TSV_NAME);
    let published_pca_json_path = published_output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let published_input_vcf_path = published_input_root.join(DEFAULT_INPUT_VCF_NAME);
    let published_sample_metadata_path =
        published_input_root.join(DEFAULT_INPUT_SAMPLE_METADATA_NAME);
    let published_population_metadata_path =
        published_input_root.join(DEFAULT_INPUT_POPULATION_METADATA_NAME);
    let published_population_labels_manifest_path =
        published_input_root.join(DEFAULT_INPUT_POPULATION_LABELS_NAME);
    let published_source_eigenvec_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME);
    let published_source_eigenval_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME);
    let published_source_pca_manifest_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME);
    let published_source_logs_path = published_output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    let published_stage_result_manifest_path =
        published_output_root.join(DEFAULT_STAGE_RESULT_NAME);
    PublishedPcaArtifacts {
        pca_tsv: published_pca_tsv_path,
        pca_json: published_pca_json_path,
        input_vcf: published_input_vcf_path,
        sample_metadata: published_sample_metadata_path,
        population_metadata: published_population_metadata_path,
        population_labels_manifest: published_population_labels_manifest_path,
        source_eigenvec: published_source_eigenvec_path,
        source_eigenval: published_source_eigenval_path,
        source_pca_manifest: published_source_pca_manifest_path,
        source_logs: published_source_logs_path,
        stage_result_manifest: published_stage_result_manifest_path,
    }
}
