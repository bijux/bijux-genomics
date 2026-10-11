//! Owner: bijux-dna-bench-vcf
//! descent adapter registry contracts.
use super::{anyhow, fs, Context, Path, RegistryToolContract, Result};

pub(super) fn load_registry_tool_contract(
    repo_root: &Path,
    tool_id: &str,
) -> Result<RegistryToolContract> {
    let path = repo_root.join("configs/ci/registry/tool_registry_vcf_downstream.toml");
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let value: toml::Value = toml::from_str(&raw).context("parse VCF downstream tool registry")?;
    let tools = value
        .get("tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("VCF downstream tool registry is missing `tools`"))?;
    let entry = tools
        .iter()
        .find(|item| item.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id))
        .ok_or_else(|| anyhow!("VCF downstream tool registry is missing `{tool_id}`"))?;
    Ok(RegistryToolContract {
        tool_id: tool_id.to_string(),
        tool_status: entry
            .get("status")
            .and_then(toml::Value::as_str)
            .unwrap_or("planned")
            .to_string(),
    })
}
