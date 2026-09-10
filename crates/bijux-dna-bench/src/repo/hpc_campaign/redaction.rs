//! Owner: current campaign environment observation for publication redaction.
use anyhow::Result;
use bijux_dna_bench_model::select_hpc_redaction_needles;
use std::path::Path;

/// Read current values through the same decoder used by campaign loading.
/// # Errors
/// Returns read/decode errors for an existing environment file; an absent file contributes no values.
pub fn collect_hpc_redaction_needles(path: &Path, explicit_keys: &[String]) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let values = super::environment::load_env_file(path)?;
    Ok(select_hpc_redaction_needles(&values, explicit_keys))
}
