use super::model::{BindingKey, RetainedVcfBinding};
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
#[derive(Deserialize)]
struct Registry {
    tools: Vec<Tool>,
}
#[derive(Deserialize)]
struct Tool {
    id: String,
    status: String,
    #[serde(default)]
    stage_ids: Vec<String>,
}

pub(super) fn load_retained_vcf_bindings(repo_root: &Path) -> Result<Vec<RetainedVcfBinding>> {
    let mut bindings = BTreeMap::<BindingKey, BTreeSet<String>>::new();
    for relative in [
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let path = repo_root.join(relative);
        let raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let registry: Registry =
            toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
        for tool in registry.tools {
            ensure!(
                !tool.id.trim().is_empty() && tool.id.trim() == tool.id,
                "retained VCF tool identity must be nonempty and unpadded"
            );
            if tool.status == "unsupported" {
                continue;
            }
            ensure!(
                matches!(tool.status.as_str(), "production" | "supported"),
                "retained VCF tool has unsupported lifecycle status"
            );
            ensure!(!tool.stage_ids.is_empty(), "retained VCF tool has no stages");
            let mut stages = BTreeSet::new();
            for stage in tool.stage_ids {
                ensure!(
                    !stage.trim().is_empty()
                        && stage.trim() == stage
                        && stages.insert(stage.clone()),
                    "retained VCF tool contains blank, padded or repeated stage identity"
                );
                bindings
                    .entry(BindingKey { stage_id: stage, tool_id: tool.id.clone() })
                    .or_default()
                    .insert(tool.status.clone());
            }
        }
    }
    Ok(bindings
        .into_iter()
        .map(|(key, tool_statuses)| RetainedVcfBinding {
            stage_id: key.stage_id,
            tool_id: key.tool_id,
            tool_statuses,
        })
        .collect())
}
