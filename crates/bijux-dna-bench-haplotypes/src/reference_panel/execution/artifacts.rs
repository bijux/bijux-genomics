//! Project prepared panel artifacts without changing native provenance.
use crate::reference_panel::model::DEFAULT_CHUNKS_NAME;
use crate::reference_panel::model::DEFAULT_OUTPUT_PANEL_NAME;
use crate::reference_panel::model::DEFAULT_OVERLAP_NAME;
use crate::reference_panel::model::DEFAULT_OVERLAP_TSV_NAME;
use crate::reference_panel::model::DEFAULT_PANEL_FILES_NAME;
use crate::reference_panel::model::DEFAULT_PANEL_MANIFEST_NAME;
use crate::reference_panel::model::DEFAULT_PANEL_OVERLAP_NAME;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::PrepareReferencePanelOutputs;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct PanelArtifacts {
    pub(crate) panel_vcf: PathBuf,
    pub(crate) panel_tbi: PathBuf,
    pub(crate) panel_manifest: PathBuf,
    pub(crate) overlap: PathBuf,
    pub(crate) panel_overlap: PathBuf,
    pub(crate) panel_files: PathBuf,
    pub(crate) overlap_tsv: PathBuf,
    pub(crate) chunks: PathBuf,
}

pub(crate) fn materialize_outputs(
    stage_outputs: &PrepareReferencePanelOutputs,
    output_root: &Path,
) -> Result<PanelArtifacts> {
    let panel_vcf_path = output_root.join(DEFAULT_OUTPUT_PANEL_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.prepared_panel_vcf, &panel_vcf_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.prepared_panel_vcf.display(),
                panel_vcf_path.display()
            )
        },
    )?;
    let panel_tbi_path = PathBuf::from(format!("{}.tbi", panel_vcf_path.display()));
    bijux_dna_infra::copy_file(&stage_outputs.prepared_panel_tbi, &panel_tbi_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.prepared_panel_tbi.display(),
                panel_tbi_path.display()
            )
        },
    )?;
    let panel_manifest_path = output_root.join(DEFAULT_PANEL_MANIFEST_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.panel_manifest_json, &panel_manifest_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.panel_manifest_json.display(),
                panel_manifest_path.display()
            )
        })?;
    let overlap_path = output_root.join(DEFAULT_OVERLAP_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.overlap_json, &overlap_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.overlap_json.display(), overlap_path.display())
    })?;
    let panel_overlap_path = output_root.join(DEFAULT_PANEL_OVERLAP_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.panel_overlap_json, &panel_overlap_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.panel_overlap_json.display(),
                panel_overlap_path.display()
            )
        })?;
    let panel_files_path = output_root.join(DEFAULT_PANEL_FILES_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.panel_files_json, &panel_files_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.panel_files_json.display(),
                panel_files_path.display()
            )
        },
    )?;
    let overlap_tsv_path = output_root.join(DEFAULT_OVERLAP_TSV_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.overlap_tsv, &overlap_tsv_path).with_context(
        || {
            format!(
                "copy {} to {}",
                stage_outputs.overlap_tsv.display(),
                overlap_tsv_path.display()
            )
        },
    )?;
    let chunks_path = output_root.join(DEFAULT_CHUNKS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.chunks_json, &chunks_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.chunks_json.display(), chunks_path.display())
    })?;

    Ok(PanelArtifacts {
        panel_vcf: panel_vcf_path,
        panel_tbi: panel_tbi_path,
        panel_manifest: panel_manifest_path,
        overlap: overlap_path,
        panel_overlap: panel_overlap_path,
        panel_files: panel_files_path,
        overlap_tsv: overlap_tsv_path,
        chunks: chunks_path,
    })
}
