use super::model::{LocalBamKinshipConfig, MaterializedProof};
use anyhow::{Context, Result};
use std::{fs, path::Path};

use anyhow::anyhow;
/// Materialize the declared tiny BAM parser proof.
///
/// # Errors
/// Propagates input, native observation and artifact publication failures.
pub fn generate_bam_kinship_probe(
    repo_root: &Path,
    probe_root: &Path,
    config: &LocalBamKinshipConfig,
) -> Result<MaterializedProof> {
    let case = config
        .cases
        .iter()
        .find(|case| case.expected_status == "ok")
        .or_else(|| config.cases.first())
        .ok_or_else(|| anyhow!("missing governed local kinship smoke case"))?;
    let probe_dir = probe_root.join("bam_kinship");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let report_path = probe_dir.join("kinship.summary.json");
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_kinship(
        &repo_root.join(&case.bam),
        &config.tool_id,
        &case.reference_panel,
        &case.reference_build,
        &case.population_scope,
        case.min_overlap_snps,
        case.requires_cohort_context,
    )?;
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    Ok(MaterializedProof { proof_path: report_path, observed_tool_id: config.tool_id.clone() })
}
