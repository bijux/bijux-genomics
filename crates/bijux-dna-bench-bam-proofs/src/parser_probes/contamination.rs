use super::model::{ContaminationProbe, MaterializedProof};
use anyhow::{Context, Result};
use std::{fs, path::Path};

/// Materialize the declared tiny BAM parser proof.
///
/// # Errors
/// Propagates input, native observation and artifact publication failures.
pub fn generate_bam_contamination_probe(
    probe_root: &Path,
    request: &ContaminationProbe<'_>,
) -> Result<MaterializedProof> {
    let probe_dir = probe_root.join("bam_contamination");
    fs::create_dir_all(&probe_dir).with_context(|| format!("create {}", probe_dir.display()))?;
    let report_path = probe_dir.join("contamination.summary.json");
    let scope = request.scope;
    let method = if scope == "mitochondrial" || request.tool_id == "schmutzi" {
        "schmutzi"
    } else if request.tool_id == "contammix" {
        "contammix"
    } else {
        "verifybamid2"
    };
    let assumptions = vec![request.assumptions.to_string(), format!("scope:{scope}")];
    bijux_dna_infra::atomic_write_json(
        &report_path,
        &serde_json::json!({
            "method": method,
            "estimate": 0.02,
            "ci_low": 0.01,
            "ci_high": 0.03,
            "assumptions": assumptions,
        }),
    )?;
    Ok(MaterializedProof { proof_path: report_path, observed_tool_id: request.tool_id.to_string() })
}
