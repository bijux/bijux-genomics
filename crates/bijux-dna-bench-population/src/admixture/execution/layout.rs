//! Owner: bijux-dna-bench-population
//! Typed paths for published reports and their declared source artifacts.
use crate::admixture::model::DEFAULT_INPUT_POPULATION_LABELS_NAME;
use crate::admixture::model::DEFAULT_INPUT_POPULATION_METADATA_NAME;
use crate::admixture::model::DEFAULT_INPUT_SAMPLE_METADATA_NAME;
use crate::admixture::model::DEFAULT_INPUT_VCF_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_SOURCE_K_SELECTION_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_TSV_NAME;
use crate::admixture::model::DEFAULT_STAGE_RESULT_NAME;
use std::path::Path;

pub(crate) struct PublishedAdmixtureArtifacts {
    pub(crate) admixture_tsv: std::path::PathBuf,
    pub(crate) admixture_json: std::path::PathBuf,
    pub(crate) input_vcf: std::path::PathBuf,
    pub(crate) sample_metadata: std::path::PathBuf,
    pub(crate) population_metadata: std::path::PathBuf,
    pub(crate) population_labels_manifest: std::path::PathBuf,
    pub(crate) source_q_matrix: std::path::PathBuf,
    pub(crate) source_k_selection: std::path::PathBuf,
    pub(crate) source_logs: std::path::PathBuf,
    pub(crate) stage_result_manifest: std::path::PathBuf,
}

pub(crate) fn publication_layout(published_output_root: &Path) -> PublishedAdmixtureArtifacts {
    let published_artifacts_root = published_output_root.join("artifacts");
    let published_input_root = published_artifacts_root.join("input");
    let published_admixture_tsv_path = published_output_root.join(DEFAULT_OUTPUT_TSV_NAME);
    let published_admixture_json_path = published_output_root.join(DEFAULT_OUTPUT_JSON_NAME);
    let published_input_vcf_path = published_input_root.join(DEFAULT_INPUT_VCF_NAME);
    let published_sample_metadata_path =
        published_input_root.join(DEFAULT_INPUT_SAMPLE_METADATA_NAME);
    let published_population_metadata_path =
        published_input_root.join(DEFAULT_INPUT_POPULATION_METADATA_NAME);
    let published_population_labels_manifest_path =
        published_input_root.join(DEFAULT_INPUT_POPULATION_LABELS_NAME);
    let published_source_q_matrix_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME);
    let published_source_k_selection_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_K_SELECTION_NAME);
    let published_source_logs_path = published_output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    let published_stage_result_manifest_path =
        published_output_root.join(DEFAULT_STAGE_RESULT_NAME);
    PublishedAdmixtureArtifacts {
        admixture_tsv: published_admixture_tsv_path,
        admixture_json: published_admixture_json_path,
        input_vcf: published_input_vcf_path,
        sample_metadata: published_sample_metadata_path,
        population_metadata: published_population_metadata_path,
        population_labels_manifest: published_population_labels_manifest_path,
        source_q_matrix: published_source_q_matrix_path,
        source_k_selection: published_source_k_selection_path,
        source_logs: published_source_logs_path,
        stage_result_manifest: published_stage_result_manifest_path,
    }
}
