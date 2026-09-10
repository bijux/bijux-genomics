//! Owner: bijux-dna-bench-vcf
//! imputation adapter registry contracts.
use super::{
    anyhow, fs, BTreeSet, Context, Path, RegistryToolContract, Result,
    GOVERNED_IMPUTATION_STAGE_IDS, VCF_IMPUTATION_REGISTRY_PATHS,
};

pub(super) fn load_registry_tool_contract(
    repo_root: &Path,
    tool_id: &str,
) -> Result<RegistryToolContract> {
    let required_stage_ids = GOVERNED_IMPUTATION_STAGE_IDS.iter().copied().collect::<BTreeSet<_>>();

    for relative_path in VCF_IMPUTATION_REGISTRY_PATHS {
        let registry_path = repo_root.join(relative_path);
        let raw = fs::read_to_string(&registry_path)
            .with_context(|| format!("read {}", registry_path.display()))?;
        let parsed: toml::Value =
            toml::from_str(&raw).with_context(|| format!("parse {}", registry_path.display()))?;
        let tools = parsed
            .get("tools")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing tools in {}", registry_path.display()))?;
        let Some(tool) = tools.iter().find(|entry| {
            let Some(candidate) = entry.get("tool_id").and_then(toml::Value::as_str) else {
                return false;
            };
            if candidate != tool_id {
                return false;
            }
            let stage_ids = entry
                .get("stage_ids")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(toml::Value::as_str)
                .collect::<BTreeSet<_>>();
            stage_ids.is_superset(&required_stage_ids)
        }) else {
            continue;
        };

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
        return Ok(RegistryToolContract {
            tool_id: tool_id.to_string(),
            tool_status: tool_status.to_string(),
            stage_ids,
        });
    }

    Err(anyhow!(
        "missing {tool_id} VCF registry row covering {GOVERNED_IMPUTATION_STAGE_IDS:?} in the governed VCF registries"
    ))
}
