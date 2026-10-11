use super::model::{DamageProbe, MaterializedProof};
use anyhow::{Context, Result};
use std::{fs, path::Path};

/// Materialize the declared tiny BAM parser proof.
///
/// # Errors
/// Propagates input, native observation and artifact publication failures.
pub fn generate_bam_damage_probe(
    repo_root: &Path,
    probe_root: &Path,
    request: &DamageProbe<'_>,
) -> Result<MaterializedProof> {
    super::inputs::admit_damage(request)?;
    let probe_dir = probe_root.join("bam_damage");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let input_bam = repo_root.join(request.bam);
    let damage_metrics = bijux_dna_domain_bam::metrics::DamageMetricsV1 {
        c_to_t_5p: request.c_to_t_5p,
        g_to_a_3p: request.g_to_a_3p,
        pmd_score_histogram: Vec::new(),
    };
    let evidence = bijux_dna_domain_bam::summarize_tiny_bam_damage_evidence(
        &input_bam,
        &damage_metrics,
        request.strict_profile_upgraded,
    )?;
    let report_path = probe_dir.join("damage.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &evidence)?;
    Ok(MaterializedProof { proof_path: report_path, observed_tool_id: request.tool_id.to_string() })
}
