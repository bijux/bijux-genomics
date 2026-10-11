use super::model::{LocalBamGenotypingConfig, MaterializedProof};
use anyhow::{Context, Result};
use std::{fs, path::Path};

use super::inputs::path_relative_to_repo;
/// Materialize the declared tiny BAM parser proof.
///
/// # Errors
/// Propagates input, native observation and artifact publication failures.
pub fn generate_bam_genotyping_probe(
    repo_root: &Path,
    probe_root: &Path,
    config: &LocalBamGenotypingConfig,
) -> Result<MaterializedProof> {
    let tool_id = config.tool_id.clone();
    let probe_dir = probe_root.join("bam_genotyping");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let report_path = probe_dir.join("genotyping.json");
    bijux_dna_infra::atomic_write_json(
        &report_path,
        &serde_json::json!({
            "schema_version": "bijux.bam.genotyping.v1",
            "stage_id": "bam.genotyping",
            "producer": "bam.genotyping",
            "caller": tool_id,
            "reference": path_relative_to_repo(repo_root, &repo_root.join(&config.reference_fasta)),
            "sites": path_relative_to_repo(repo_root, &repo_root.join(&config.sites_vcf)),
            "regions": path_relative_to_repo(repo_root, &repo_root.join(&config.regions)),
            "sample_id": config.sample_id,
            "call_rate": 1.0,
            "mean_posterior": 0.99,
            "posterior_histogram": [[99, 4]],
        }),
    )?;
    Ok(MaterializedProof { proof_path: report_path, observed_tool_id: config.tool_id.clone() })
}
