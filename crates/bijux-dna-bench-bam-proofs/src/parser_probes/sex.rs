use super::model::{LocalBamSexConfig, MaterializedProof};
use anyhow::{Context, Result};
use std::{fs, path::Path};

use anyhow::anyhow;
/// Materialize the declared tiny BAM parser proof.
///
/// # Errors
/// Propagates input, native observation and artifact publication failures.
pub fn generate_bam_sex_probe(
    repo_root: &Path,
    probe_root: &Path,
    config: &LocalBamSexConfig,
) -> Result<MaterializedProof> {
    let case =
        config.cases.first().ok_or_else(|| anyhow!("missing governed local sex smoke case"))?;
    let probe_dir = probe_root.join("bam_sex");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_sex(
        &repo_root.join(&case.bam),
        &repo_root.join(&case.reference),
        &config.tool_id,
        Some(case.chromosome_system.as_str()),
        Some(case.minimum_y_sites),
    )?;
    let report_path = probe_dir.join("sex.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(MaterializedProof { proof_path: report_path, observed_tool_id: config.tool_id.clone() })
}
