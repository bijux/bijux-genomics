use anyhow::{anyhow, Context, Result};
use std::path::Path;
mod descent;
mod direct;
mod imputation;
pub(super) mod model;
mod phasing;
pub(crate) use descent::{ensure_descent_family_output, ensure_descent_family_output_contract};
pub(crate) use direct::render_and_validate_direct_vcf_ready_output;
pub(crate) use imputation::ensure_imputation_family_output;
pub(crate) use model::PublishedAdapterReport;
pub(crate) use phasing::ensure_single_phasing_output;
pub(crate) fn read_governed_json<T: serde::de::DeserializeOwned>(
    repo_root: &Path,
    relative_path: &str,
) -> Result<T> {
    let absolute_path = repo_root.join(relative_path);
    let payload = std::fs::read_to_string(&absolute_path)
        .with_context(|| format!("read {}", absolute_path.display()))?;
    serde_json::from_str(&payload)
        .with_context(|| format!("parse governed JSON {}", absolute_path.display()))
}
fn find_row<'a>(
    payload: &'a PublishedAdapterReport,
    tool_id: &str,
    stage_id: &str,
) -> Result<&'a model::PublishedAdapterRow> {
    payload
        .rows
        .as_deref()
        .unwrap_or_default()
        .iter()
        .find(|row| {
            row.tool_id.as_deref() == Some(tool_id) && row.stage_id.as_deref() == Some(stage_id)
        })
        .ok_or_else(|| anyhow!("governed report is missing `{stage_id}` / `{tool_id}`"))
}
