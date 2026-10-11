use super::model::{AlignmentProbe, MaterializedProof};
use anyhow::{Context, Result};
use std::{fs, path::Path};

/// Materialize the declared tiny BAM parser proof.
///
/// # Errors
/// Propagates input, native observation and artifact publication failures.
pub fn generate_bam_align_probe(
    repo_root: &Path,
    probe_root: &Path,
    request: &AlignmentProbe<'_>,
) -> Result<MaterializedProof> {
    let backend = super::inputs::alignment_backend(request.tool_id)?;
    let probe_dir = probe_root.join("bam_align");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let resolved_r2 = request.input_r2.map(|path| repo_root.join(path));
    let (mut provenance, _mapping) = match backend {
        super::inputs::AlignmentBackend::Bowtie2 => {
            bijux_dna_domain_bam::align_fastq_to_bam_bowtie2_style(
                &repo_root.join(request.reference_fasta),
                &repo_root.join(request.input_r1),
                resolved_r2.as_deref(),
                &probe_dir,
                request.sample_id,
                request.read_group,
                request.sensitivity_profile,
            )?
        }
        super::inputs::AlignmentBackend::Bwa => bijux_dna_domain_bam::align_fastq_to_bam_bwa_style(
            &repo_root.join(request.reference_fasta),
            &repo_root.join(request.input_r1),
            resolved_r2.as_deref(),
            &probe_dir,
            request.sample_id,
            request.read_group,
            request.preset,
            request.seed_length,
        )?,
    };
    // Preserve the declared reference address while resolving filesystem reads at the boundary.
    provenance.reference_fasta = request.reference_fasta.to_path_buf();
    let report_path = probe_dir.join("alignment.provenance.json");
    bijux_dna_infra::atomic_write_json(&report_path, &provenance)?;
    Ok(MaterializedProof { proof_path: report_path, observed_tool_id: request.tool_id.to_string() })
}
