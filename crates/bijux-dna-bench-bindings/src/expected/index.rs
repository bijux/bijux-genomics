use crate::observations::{Domain, OutputContract};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
pub(super) type OutputIndex = BTreeMap<(String, String), OutputContract>;
pub(super) fn admit_output_contracts(
    domain: Domain,
    rows: Vec<OutputContract>,
) -> Result<OutputIndex> {
    let mut index = OutputIndex::new();
    for row in rows {
        let key = (row.stage_id.clone(), row.tool_id.clone());
        if index.insert(key.clone(), row).is_some() {
            return Err(anyhow!(
                "duplicate {} adapter output contract row for `{}` / `{}`",
                domain.label(),
                key.0,
                key.1
            ));
        }
    }
    Ok(index)
}
