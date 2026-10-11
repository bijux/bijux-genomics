use anyhow::{anyhow, Context, Result};
use std::path::Path;
pub(crate) fn resolve_tools(
    default_tools: &[String],
    requested_tools: &[String],
) -> Result<Vec<String>> {
    if requested_tools.is_empty() {
        return Ok(default_tools.to_vec());
    }
    let mut expected = default_tools.to_vec();
    expected.sort();
    expected.dedup();
    let mut requested = requested_tools
        .iter()
        .filter_map(|row| {
            let trimmed = row.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect::<Vec<_>>();
    requested.sort();
    requested.dedup();
    if requested != expected {
        return Err(anyhow!(
            "tool roster drift for corpus benchmark: expected {expected:?}, found {requested:?}"
        ));
    }
    Ok(default_tools.to_vec())
}

pub(crate) fn validate_benchmark_layout(corpus_root: &Path, out_root: &Path) -> Result<()> {
    let corpus_identity = corpus_root
        .canonicalize()
        .with_context(|| format!("resolve corpus root {}", corpus_root.display()))?;
    let output_identity = crate::admission::resolve_output_path(out_root)?;
    if output_identity.starts_with(&corpus_identity)
        || corpus_identity.starts_with(&output_identity)
    {
        return Err(anyhow!(
            "benchmark output root must not live under the corpus data tree: {}",
            out_root.display()
        ));
    }
    let legacy_root = corpus_root.join("benchmarks");
    if legacy_root.exists() {
        return Err(anyhow!(
            "legacy benchmark outputs still exist under the corpus data tree: {}",
            legacy_root.display()
        ));
    }
    Ok(())
}
