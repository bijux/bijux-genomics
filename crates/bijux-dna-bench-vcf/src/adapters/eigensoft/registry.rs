//! Owner: bijux-dna-bench-vcf
//! eigensoft adapter registry contracts.
use super::{anyhow, fs, Context, Path, RegistryToolContract, Result};

pub(super) fn load_registry_tool_contract(
    repo_root: &Path,
    tool_id: &str,
) -> Result<RegistryToolContract> {
    let registry_path = repo_root.join("configs/ci/registry/tool_registry_vcf_downstream.toml");
    let raw = fs::read_to_string(&registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let parsed: toml::Value =
        toml::from_str(&raw).with_context(|| format!("parse {}", registry_path.display()))?;
    let tools = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("missing tools in {}", registry_path.display()))?;
    let tool = tools
        .iter()
        .find(|entry| {
            entry
                .get("tool_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|candidate| candidate == tool_id)
        })
        .ok_or_else(|| {
            anyhow!("missing {tool_id} VCF registry row in {}", registry_path.display())
        })?;
    let tool_status = tool
        .get("status")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| anyhow!("{tool_id} VCF registry row is missing status"))?;
    let stage_ids = tool
        .get("stage_ids")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("{tool_id} VCF registry row is missing stage_ids"))?
        .iter()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    Ok(RegistryToolContract {
        tool_id: tool_id.to_string(),
        tool_status: tool_status.to_string(),
        stage_ids,
    })
}
