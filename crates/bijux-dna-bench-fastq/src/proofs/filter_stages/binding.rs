use super::models::BindingKey;
use crate::proofs::inputs::observations::ExpectedResultObservation as ExpectedBenchmarkResultRow;
use crate::readiness::adapters::FastqCommandAdapterCoverageRow;
use crate::readiness::output_contract::FastqAdapterOutputContractRow;
use crate::readiness::parsers::FastqParserCoverageRow;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeMap;
pub(super) fn collect_rows_by_binding<T>(rows: Vec<T>) -> Result<BTreeMap<BindingKey, T>>
where
    T: BindingRow,
{
    let mut by_binding = BTreeMap::new();
    for row in rows {
        let key = binding_key(row.stage_id(), row.tool_id());
        if by_binding.insert(key.clone(), row).is_some() {
            return Err(anyhow!(
                "duplicate filter-stage binding `{}` / `{}`",
                key.stage_id,
                key.tool_id
            ));
        }
    }
    Ok(by_binding)
}

pub(super) trait BindingRow {
    fn stage_id(&self) -> &str;
    fn tool_id(&self) -> &str;
}

impl BindingRow for FastqAdapterOutputContractRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

impl BindingRow for FastqParserCoverageRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

impl BindingRow for ExpectedBenchmarkResultRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

impl BindingRow for FastqCommandAdapterCoverageRow {
    fn stage_id(&self) -> &str {
        &self.stage_id
    }

    fn tool_id(&self) -> &str {
        &self.tool_id
    }
}

pub(super) fn binding_key(stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey { stage_id: stage_id.to_string(), tool_id: tool_id.to_string() }
}
