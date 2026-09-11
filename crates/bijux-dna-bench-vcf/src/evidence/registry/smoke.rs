use super::{admission::VcfRegistrySnapshot, declarations::RegistrySmokeFields};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Default)]
pub(crate) struct VcfRegistrySmokeRecord {
    pub(crate) tool_id: String,
    pub(crate) registered_binary: String,
    pub(crate) dockerfile: String,
    pub(crate) apptainer_def: String,
    pub(crate) smoke_minimal_cmd: String,
    pub(crate) smoke_minimal_rationale: String,
}
impl VcfRegistrySnapshot {
    pub(crate) fn smoke_records(&self) -> BTreeMap<String, VcfRegistrySmokeRecord> {
        self.tool_records
            .iter()
            .map(|record| (record.tool_id.clone(), record.smoke.clone()))
            .collect()
    }
}
pub(super) fn merge_smoke(
    record: &mut VcfRegistrySmokeRecord,
    incoming: &RegistrySmokeFields,
) -> Result<()> {
    merge_registry_string(
        &mut record.dockerfile,
        incoming.dockerfile.trim().to_owned(),
        "dockerfile",
        &record.tool_id,
    )?;
    merge_registry_string(
        &mut record.apptainer_def,
        incoming.apptainer_def.trim().to_owned(),
        "apptainer_def",
        &record.tool_id,
    )?;
    merge_registry_string(
        &mut record.smoke_minimal_cmd,
        incoming.smoke_minimal_cmd.trim().to_owned(),
        "smoke_minimal_cmd",
        &record.tool_id,
    )?;
    merge_registry_string(
        &mut record.smoke_minimal_rationale,
        incoming.smoke_minimal_rationale.trim().to_owned(),
        "smoke_minimal_rationale",
        &record.tool_id,
    )?;
    Ok(())
}
fn merge_registry_string(
    target: &mut String,
    incoming: String,
    field: &str,
    tool_id: &str,
) -> Result<()> {
    if incoming.trim().is_empty() {
        return Ok(());
    }
    if target.trim().is_empty() {
        *target = incoming;
        return Ok(());
    }
    if target != &incoming {
        return Err(anyhow!(
            "VCF local-container smoke registry field `{field}` drifted for `{tool_id}` (`{target}` vs `{incoming}`)"
        ));
    }
    Ok(())
}
