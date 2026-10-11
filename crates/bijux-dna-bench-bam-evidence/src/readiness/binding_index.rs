use super::binding::{binding_key, BindingKey, BindingRow};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
pub(in crate::readiness) fn collect_rows_by_binding<T>(
    label: &str,
    rows: Vec<T>,
) -> Result<BTreeMap<BindingKey, T>>
where
    T: BindingRow,
{
    let mut by_binding = BTreeMap::new();
    for row in rows {
        let key = binding_key(row.stage_id(), row.tool_id());
        if by_binding.insert(key.clone(), row).is_some() {
            return Err(anyhow!(
                "duplicate {label} binding `{}` / `{}`",
                key.stage_id,
                key.tool_id
            ));
        }
    }
    Ok(by_binding)
}
