//! Owner: bijux-dna-bench-workflows
//! Core germline observations variants.
use super::super::{anyhow, BTreeMap, Result};

pub(crate) fn required_tool_id(
    tool_ids: &BTreeMap<String, String>,
    stage_id: &str,
) -> Result<String> {
    tool_ids
        .get(stage_id)
        .cloned()
        .ok_or_else(|| anyhow!("missing retained tool id for `{stage_id}`"))
}
