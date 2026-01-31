use std::collections::{BTreeMap, BTreeSet};

use bijux_core::metrics_registry::metrics_schema_for_stage;
use bijux_core::FactsRowV1;

#[derive(Debug, Clone)]
pub struct FactRow {
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub params_hash: String,
    pub bank_hashes: serde_json::Value,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub metrics: serde_json::Value,
    pub reports: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct FactTable {
    pub rows: Vec<FactRow>,
    pub by_identity: BTreeMap<(String, String, String), Vec<usize>>,
    pub known_stages: BTreeSet<String>,
}

impl FactTable {
    /// # Errors
    /// Returns an error if a row is missing required identity fields or violates invariants.
    pub fn from_facts(rows: &[FactsRowV1]) -> anyhow::Result<Self> {
        let mut table = Vec::with_capacity(rows.len());
        let mut by_identity: BTreeMap<(String, String, String), Vec<usize>> = BTreeMap::new();
        let mut known_stages = BTreeSet::new();
        for (idx, row) in rows.iter().enumerate() {
            if row.stage_id.trim().is_empty() || row.tool_id.trim().is_empty() {
                return Err(anyhow::anyhow!("facts row missing stage/tool id"));
            }
            if metrics_schema_for_stage(&row.stage_id).is_none() {
                return Err(anyhow::anyhow!(
                    "facts row has unknown stage_id {}",
                    row.stage_id
                ));
            }
            if row.reads_in.is_some() ^ row.reads_out.is_some() {
                return Err(anyhow::anyhow!(
                    "facts row has partial reads delta for stage {}",
                    row.stage_id
                ));
            }
            if row.bases_in.is_some() ^ row.bases_out.is_some() {
                return Err(anyhow::anyhow!(
                    "facts row has partial bases delta for stage {}",
                    row.stage_id
                ));
            }
            if let (Some(reads_in), Some(reads_out)) = (row.reads_in, row.reads_out) {
                if reads_out > reads_in {
                    return Err(anyhow::anyhow!(
                        "facts row reads_out exceeds reads_in for stage {}",
                        row.stage_id
                    ));
                }
            }
            if let (Some(bases_in), Some(bases_out)) = (row.bases_in, row.bases_out) {
                if bases_out > bases_in {
                    return Err(anyhow::anyhow!(
                        "facts row bases_out exceeds bases_in for stage {}",
                        row.stage_id
                    ));
                }
            }
            known_stages.insert(row.stage_id.clone());
            let fact_row = FactRow {
                run_id: row.run_id.clone(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                tool_version: row.tool_version.clone(),
                params_hash: row.params_hash.clone(),
                bank_hashes: row.bank_hashes.clone(),
                runtime_s: row.runtime_s,
                memory_mb: row.memory_mb,
                reads_in: row.reads_in,
                reads_out: row.reads_out,
                bases_in: row.bases_in,
                bases_out: row.bases_out,
                metrics: row.metrics.clone(),
                reports: row.reports.clone(),
            };
            table.push(fact_row);
            by_identity
                .entry((
                    row.run_id.clone(),
                    row.stage_id.clone(),
                    row.tool_id.clone(),
                ))
                .or_default()
                .push(idx);
        }
        Ok(Self {
            rows: table,
            by_identity,
            known_stages,
        })
    }

    #[must_use]
    pub fn rows_for(&self, run_id: &str, stage_id: &str, tool_id: &str) -> Vec<&FactRow> {
        self.by_identity
            .get(&(
                run_id.to_string(),
                stage_id.to_string(),
                tool_id.to_string(),
            ))
            .map(|idxs| idxs.iter().filter_map(|idx| self.rows.get(*idx)).collect())
            .unwrap_or_default()
    }

    /// # Errors
    /// Returns an error if a stage is not present in the table.
    pub fn assert_has_stage(&self, stage_id: &str) -> anyhow::Result<()> {
        if self.known_stages.contains(stage_id) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("fact table missing stage {stage_id}"))
        }
    }

    /// # Errors
    /// Returns an error if a tool is missing expected metrics fields.
    pub fn assert_metric_present(&self, metric_id: &str) -> anyhow::Result<()> {
        let mut seen = false;
        for row in &self.rows {
            if row.metrics.get(metric_id).is_some() {
                seen = true;
                break;
            }
        }
        if seen {
            Ok(())
        } else {
            Err(anyhow::anyhow!("metric {metric_id} not present in facts"))
        }
    }

    /// # Errors
    /// Returns an error if any fact row has missing metric values.
    pub fn assert_metrics_explicit(&self, metric_ids: &[&str]) -> anyhow::Result<()> {
        for row in &self.rows {
            for metric_id in metric_ids {
                if row.metrics.get(*metric_id).is_none() {
                    return Err(anyhow::anyhow!(
                        "metric {metric_id} missing for stage {}",
                        row.stage_id
                    ));
                }
            }
        }
        Ok(())
    }
}
