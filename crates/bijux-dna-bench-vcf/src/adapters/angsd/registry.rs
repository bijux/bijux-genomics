//! Owner: bijux-dna-bench-vcf
//! angsd adapter registry contracts.
use super::{anyhow, fs, Context, Path, Result, GOVERNED_ANGSD_TOOL_ID};

pub(super) fn load_admitted_angsd_stage_ids(repo_root: &Path) -> Result<Vec<String>> {
    let registry_path = repo_root.join("configs/ci/registry/tool_registry_vcf_downstream.toml");
    let raw = fs::read_to_string(&registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let parsed: toml::Value =
        toml::from_str(&raw).with_context(|| format!("parse {}", registry_path.display()))?;
    let tools = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("missing tools in {}", registry_path.display()))?;
    let angsd = tools
        .iter()
        .find(|entry| {
            entry
                .get("tool_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|tool_id| tool_id == GOVERNED_ANGSD_TOOL_ID)
        })
        .ok_or_else(|| anyhow!("missing angsd VCF registry row in {}", registry_path.display()))?;
    let stage_ids = angsd
        .get("stage_ids")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("angsd VCF registry row is missing stage_ids"))?
        .iter()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    Ok(stage_ids)
}
