//! Owner: bijux-analyze
//! Canonical internal representation (IR) for analysis.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use bijux_core::metrics_registry::metrics_schema_for_stage;
use bijux_core::FactsRowV1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonBlob(serde_json::Value);

impl JsonBlob {
    #[must_use]
    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn as_value(&self) -> &serde_json::Value {
        &self.0
    }

    #[must_use]
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let mut map = serde_json::Map::new();
        for (key, value) in pairs {
            map.insert(
                (*key).to_string(),
                serde_json::Value::String((*value).to_string()),
            );
        }
        Self(serde_json::Value::Object(map))
    }

    /// # Errors
    /// Returns an error if the value cannot be serialized to JSON.
    pub fn from_serializable<T: Serialize>(value: &T) -> Result<Self> {
        let json = serde_json::to_value(value)?;
        Ok(Self(json))
    }

    /// # Errors
    /// Returns an error if the raw string cannot be parsed as JSON.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(raw: &str) -> Result<Self> {
        let json = serde_json::from_str(raw)?;
        Ok(Self(json))
    }
}

impl From<serde_json::Value> for JsonBlob {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl Default for JsonBlob {
    fn default() -> Self {
        Self(serde_json::Value::Object(serde_json::Map::new()))
    }
}

#[derive(Debug, Clone)]
pub struct FactRow {
    pub schema_version: String,
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub trace_id: String,
    pub span_id: String,
    pub params_hash: String,
    pub input_hash: String,
    pub output_hashes: Vec<String>,
    pub bank_hashes: JsonBlob,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub metrics: JsonBlob,
    pub reports: JsonBlob,
    pub artifacts: JsonBlob,
}

impl FactRow {
    #[must_use]
    pub fn to_facts_row_v1(&self) -> FactsRowV1 {
        FactsRowV1 {
            schema_version: self.schema_version.clone(),
            run_id: self.run_id.clone(),
            stage_id: self.stage_id.clone(),
            tool_id: self.tool_id.clone(),
            tool_version: self.tool_version.clone(),
            image_digest: self.image_digest.clone(),
            trace_id: self.trace_id.clone(),
            span_id: self.span_id.clone(),
            params_hash: self.params_hash.clone(),
            input_hash: self.input_hash.clone(),
            output_hashes: self.output_hashes.clone(),
            runtime_s: self.runtime_s,
            memory_mb: self.memory_mb,
            exit_code: self.exit_code,
            bank_hashes: self.bank_hashes.as_value().clone(),
            reads_in: self.reads_in,
            reads_out: self.reads_out,
            bases_in: self.bases_in,
            bases_out: self.bases_out,
            pairs_in: self.pairs_in,
            pairs_out: self.pairs_out,
            metrics: self.metrics.as_value().clone(),
            reports: self.reports.as_value().clone(),
            artifacts: self.artifacts.as_value().clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StageRecord {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub params_hash: String,
    pub input_hash: String,
    pub metrics: JsonBlob,
}

#[derive(Debug, Clone)]
pub struct ToolRecord {
    pub tool_id: String,
    pub tool_version: String,
    pub records: Vec<StageRecord>,
}

#[derive(Debug, Clone)]
pub struct MetricEnvelope {
    pub metric_id: String,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct RunSummary {
    pub run_id: String,
    pub stages: Vec<StageRecord>,
    pub reports: JsonBlob,
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
    pub fn from_facts(rows: &[FactsRowV1]) -> Result<Self> {
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
                schema_version: row.schema_version.clone(),
                run_id: row.run_id.clone(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                tool_version: row.tool_version.clone(),
                image_digest: row.image_digest.clone(),
                trace_id: row.trace_id.clone(),
                span_id: row.span_id.clone(),
                params_hash: row.params_hash.clone(),
                input_hash: row.input_hash.clone(),
                output_hashes: row.output_hashes.clone(),
                bank_hashes: JsonBlob::from(row.bank_hashes.clone()),
                runtime_s: row.runtime_s,
                memory_mb: row.memory_mb,
                exit_code: row.exit_code,
                reads_in: row.reads_in,
                reads_out: row.reads_out,
                bases_in: row.bases_in,
                bases_out: row.bases_out,
                pairs_in: row.pairs_in,
                pairs_out: row.pairs_out,
                metrics: JsonBlob::from(row.metrics.clone()),
                reports: JsonBlob::from(row.reports.clone()),
                artifacts: JsonBlob::from(row.artifacts.clone()),
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
    pub fn assert_has_stage(&self, stage_id: &str) -> Result<()> {
        if self.known_stages.contains(stage_id) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("fact table missing stage {stage_id}"))
        }
    }

    /// # Errors
    /// Returns an error if a tool is missing expected metrics fields.
    pub fn assert_metric_present(&self, metric_id: &str) -> Result<()> {
        let mut seen = false;
        for row in &self.rows {
            if row.metrics.as_value().get(metric_id).is_some() {
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
    pub fn assert_metrics_explicit(&self, metric_ids: &[&str]) -> Result<()> {
        for row in &self.rows {
            for metric_id in metric_ids {
                if row.metrics.as_value().get(*metric_id).is_none() {
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
