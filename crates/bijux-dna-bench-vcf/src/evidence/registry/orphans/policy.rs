use super::model::VcfOrphanToolRow;
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
pub(super) fn ensure_vcf_orphan_tool_contract(
    rows: &[VcfOrphanToolRow],
    _required_tool_ids: &BTreeSet<String>,
) -> Result<()> {
    if !rows.is_empty() {
        return Err(anyhow!(
            "every governed VCF tool must serve at least one supported stage; orphan tools: {:?}",
            rows.iter().map(|row| row.tool_id.as_str()).collect::<Vec<_>>()
        ));
    }
    Ok(())
}
