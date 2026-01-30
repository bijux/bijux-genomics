use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use bijux_core::FactsRowV1;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FactsSummary {
    pub runs: usize,
    pub stages: usize,
    pub total_runtime_s: f64,
    pub avg_runtime_s: f64,
}

/// Load facts rows from a jsonl file.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn load_facts_jsonl(path: &Path) -> Result<Vec<FactsRowV1>> {
    let file = File::open(path).with_context(|| format!("open facts {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let row: FactsRowV1 = serde_json::from_str(&line)?;
        rows.push(row);
    }
    Ok(rows)
}

#[must_use]
pub fn summarize_facts(rows: &[FactsRowV1]) -> FactsSummary {
    let stages = rows.len();
    let mut run_ids = std::collections::BTreeSet::new();
    let mut total_runtime_s = 0.0;
    for row in rows {
        run_ids.insert(row.run_id.clone());
        total_runtime_s += row.runtime_s;
    }
    let runs = run_ids.len();
    let avg_runtime_s = if stages == 0 {
        0.0
    } else {
        let denom = f64::from(u32::try_from(stages).unwrap_or(u32::MAX));
        total_runtime_s / denom
    };
    FactsSummary {
        runs,
        stages,
        total_runtime_s,
        avg_runtime_s,
    }
}
