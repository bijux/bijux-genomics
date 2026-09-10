//! Owner: bijux-dna-bench-population
//! Admixture fixtures proxy.
use anyhow::anyhow;
use anyhow::Result;

pub(crate) fn build_admixture_proxy_eigenval(
    cluster_headers: &[String],
    source_manifest: &serde_json::Value,
) -> Result<String> {
    let selected_k = source_manifest
        .get("selected_k")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("admixture proxy manifest missing `selected_k`"))?;
    let population_count = source_manifest
        .get("population_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("admixture proxy manifest missing `population_count`"))?;
    let mut lines = vec!["component\tvalue".to_string()];
    lines.push(format!("selected_k\t{selected_k}"));
    lines.push(format!("population_count\t{population_count}"));
    for (index, header) in cluster_headers.iter().enumerate() {
        lines.push(format!("{header}\t{}", index + 1));
    }
    Ok(format!("{}\n", lines.join("\n")))
}
