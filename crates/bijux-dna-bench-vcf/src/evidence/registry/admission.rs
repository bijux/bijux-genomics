use super::declarations::RegistryDocument;
use anyhow::{ensure, Result};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone)]
pub(super) struct RegistryToolRecord {
    pub tool_id: String,
    pub registered_binary: String,
    pub stage_ids: BTreeSet<String>,
    pub production_stage_ids: BTreeSet<String>,
    pub statuses: BTreeSet<String>,
}
/// Immutable registry observation shared by coverage audits.
pub struct VcfRegistrySnapshot {
    pub(super) tool_records: Vec<RegistryToolRecord>,
}
pub(super) fn admit_documents(documents: Vec<RegistryDocument>) -> Result<VcfRegistrySnapshot> {
    let mut records = BTreeMap::<String, RegistryToolRecord>::new();
    for document in documents {
        let mut tools = BTreeSet::new();
        for tool in document.tools {
            ensure!(
                valid_identity(&tool.id)
                    && valid_identity(&tool.expected_bin)
                    && valid_identity(&tool.status),
                "VCF registry identities must be nonempty and unpadded"
            );
            ensure!(
                tools.insert(tool.id.clone()),
                "VCF registry repeats a tool within one document"
            );
            let mut stage_ids = BTreeSet::new();
            for stage in tool.stage_ids {
                ensure!(
                    valid_identity(&stage) && stage_ids.insert(stage),
                    "VCF registry has blank, padded or repeated stages"
                );
            }
            let record = records.entry(tool.id.clone()).or_insert_with(|| RegistryToolRecord {
                tool_id: tool.id.clone(),
                registered_binary: tool.expected_bin.clone(),
                stage_ids: BTreeSet::new(),
                production_stage_ids: BTreeSet::new(),
                statuses: BTreeSet::new(),
            });
            ensure!(
                record.registered_binary == tool.expected_bin,
                "VCF registry has conflicting binary declarations for `{}`",
                tool.id
            );
            if tool.status == "production" {
                record.production_stage_ids.extend(stage_ids.iter().cloned());
            }
            record.stage_ids.extend(stage_ids);
            record.statuses.insert(tool.status);
        }
    }
    Ok(VcfRegistrySnapshot { tool_records: records.into_values().collect() })
}
pub(super) fn valid_identity(value: &str) -> bool {
    !value.is_empty() && value.trim() == value
}
