use crate::ServingContext;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
/// Construct the declared evidence from explicit inputs.
///
/// # Errors
/// Returns acquisition, semantic admission or publication failure.
pub fn load_corpus_status_by_stage(
    context: &ServingContext<'_>,
) -> Result<BTreeMap<String, String>> {
    let entries = context.evidence.corpus_declarations(context.repo_root)?;
    let mut rows = BTreeMap::new();
    for entry in entries {
        let corpus_status = match entry.compatibility_kind.as_str() {
            "fixture" => {
                let fixture_id = entry.fixture_id.as_deref().ok_or_else(|| {
                    anyhow!(
                        "stage `{}` is marked fixture-backed without a fixture_id",
                        entry.stage_id
                    )
                })?;
                Ok(format!("fixture:{fixture_id}"))
            }
            "asset_backed" => {
                let benchmark_scope_id = entry.benchmark_scope_id.as_deref().ok_or_else(|| {
                    anyhow!(
                        "stage `{}` is marked asset-backed without a benchmark_scope_id",
                        entry.stage_id
                    )
                })?;
                Ok(format!("asset:{benchmark_scope_id}"))
            }
            "planner_only" => Ok("planner_only".to_string()),
            other => Err(anyhow!(
                "stage `{}` declares unsupported corpus compatibility kind `{other}`",
                entry.stage_id
            )),
        }?;
        if rows.insert(entry.stage_id.clone(), corpus_status).is_some() {
            return Err(anyhow!("duplicate corpus compatibility stage `{}`", entry.stage_id));
        }
    }
    Ok(rows)
}
