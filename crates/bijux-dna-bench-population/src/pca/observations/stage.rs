//! Owner: bijux-dna-bench-population
//! PCA observations stage boundary.
use crate::pca::execution::artifacts::read_json;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME;
use crate::pca::model::GOVERNED_PCA_COMPONENT_COUNT;
use crate::pca::observations::evidence::validate_pca_manifest_samples;
use crate::pca::observations::records::parse_eigenvalues;
use crate::pca::observations::records::parse_pca_rows;
use crate::pca::LocalVcfPcaSmokeRow;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) struct PcaEvidence {
    pub(crate) execution_mode: String,
    pub(crate) tool_ok: bool,
    pub(crate) variant_count: u64,
    pub(crate) rows: Vec<LocalVcfPcaSmokeRow>,
    pub(crate) eigenvalues: Vec<f64>,
    pub(crate) excluded_samples: Vec<String>,
    pub(crate) unexpected_samples: Vec<String>,
}

pub(crate) fn observe_stage(
    cohort: &super::super::admission::input::CohortAdmission,
    stage_outputs: &bijux_dna_stages_vcf::pipeline::PcaStageOutputs,
    output_root: &Path,
) -> Result<PcaEvidence> {
    let super::super::admission::input::CohortAdmission {
        expected_samples,
        metadata_by_sample,
        population_labels,
        ..
    } = cohort;
    let source_eigenvec_path = output_root.join(DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.eigenvec_tsv, &source_eigenvec_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.eigenvec_tsv.display(),
                source_eigenvec_path.display()
            )
        },
    )?;
    let source_eigenval_path = output_root.join(DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.eigenval_tsv, &source_eigenval_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.eigenval_tsv.display(),
                source_eigenval_path.display()
            )
        },
    )?;
    let source_pca_manifest_path = output_root.join(DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.pca_manifest_json, &source_pca_manifest_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.pca_manifest_json.display(),
                source_pca_manifest_path.display()
            )
        })?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let source_manifest = read_json(&source_pca_manifest_path)?;
    let execution_mode = source_manifest
        .get("execution_mode")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("PCA manifest missing `execution_mode`"))?
        .to_string();
    let tool_ok = source_manifest
        .pointer("/tool_attempts/pca/ok")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("PCA manifest missing `tool_attempts.pca.ok`"))?;
    let variant_count = source_manifest
        .get("variants_passing")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("PCA manifest missing `variants_passing`"))?;
    validate_pca_manifest_samples(&source_manifest, expected_samples)?;

    let rows = parse_pca_rows(&source_eigenvec_path, metadata_by_sample, population_labels)?;
    let eigenvalues = parse_eigenvalues(&source_eigenval_path)?;
    if eigenvalues.len() < GOVERNED_PCA_COMPONENT_COUNT {
        bail!(
            "governed VCF PCA smoke expected at least {} eigenvalues, found {}",
            GOVERNED_PCA_COMPONENT_COUNT,
            eigenvalues.len()
        );
    }

    let observed_samples = rows.iter().map(|row| row.sample_id.clone()).collect::<Vec<_>>();
    let observed_sample_set = observed_samples.iter().cloned().collect::<BTreeSet<_>>();
    let expected_sample_set = expected_samples.iter().cloned().collect::<BTreeSet<_>>();
    let excluded_samples = expected_samples
        .iter()
        .filter(|sample_id| !observed_sample_set.contains(*sample_id))
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_samples = observed_samples
        .iter()
        .filter(|sample_id| !expected_sample_set.contains(*sample_id))
        .cloned()
        .collect::<Vec<_>>();
    if !excluded_samples.is_empty()
        || !unexpected_samples.is_empty()
        || rows.len() != expected_samples.len()
    {
        bail!(
            "governed VCF PCA smoke expected samples {expected_samples:?}, observed {observed_samples:?}"
        );
    }

    Ok(PcaEvidence {
        execution_mode,
        tool_ok,
        variant_count,
        rows,
        eigenvalues,
        excluded_samples,
        unexpected_samples,
    })
}
