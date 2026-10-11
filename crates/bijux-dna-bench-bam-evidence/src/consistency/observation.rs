use crate::observations::{BindingKey, BindingSurface};
use std::collections::BTreeSet;
pub(super) fn observe<'a>(
    output_path: String,
    row_count: usize,
    rows: impl Iterator<Item = (&'a str, &'a str)>,
) -> BindingSurface {
    let bindings = rows
        .map(|(stage_id, tool_id)| BindingKey {
            stage_id: stage_id.to_string(),
            tool_id: tool_id.to_string(),
        })
        .collect::<Vec<_>>();
    let stage_count = bindings.iter().map(|key| &key.stage_id).collect::<BTreeSet<_>>().len();
    let tool_count = bindings.iter().map(|key| &key.tool_id).collect::<BTreeSet<_>>().len();
    BindingSurface { output_path, row_count, stage_count, tool_count, bindings }
}
