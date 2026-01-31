//! Owner: bijux-analyze
//! Typed facts and invariants.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use bijux_core::metrics_registry::metrics_schema_for_stage;
use bijux_core::FactsRowV1;

use crate::model::JsonBlob;

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

    #[must_use]
    pub fn order_key(&self) -> (&str, &str, &str, &str, &str) {
        (
            self.run_id.as_str(),
            self.stage_id.as_str(),
            self.tool_id.as_str(),
            self.params_hash.as_str(),
            self.input_hash.as_str(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct FactTable {
    pub rows: Vec<FactRow>,
    pub by_identity: BTreeMap<(String, String, String, String, String), Vec<usize>>,
    pub known_stages: BTreeSet<String>,
}

impl FactTable {
    #[must_use]
    pub fn primary_key(row: &FactRow) -> (String, String, String, String, String) {
        (
            row.run_id.clone(),
            row.stage_id.clone(),
            row.tool_id.clone(),
            row.params_hash.clone(),
            row.input_hash.clone(),
        )
    }

    /// # Errors
    /// Returns an error if a row is missing required identity fields or violates invariants.
    pub fn from_facts(rows: &[FactsRowV1]) -> Result<Self> {
        let mut table = Vec::with_capacity(rows.len());
        let mut by_identity: BTreeMap<(String, String, String, String, String), Vec<usize>> =
            BTreeMap::new();
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
            let fact = FactRow {
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
                bank_hashes: JsonBlob::new(row.bank_hashes.clone()),
                runtime_s: row.runtime_s,
                memory_mb: row.memory_mb,
                exit_code: row.exit_code,
                reads_in: row.reads_in,
                reads_out: row.reads_out,
                bases_in: row.bases_in,
                bases_out: row.bases_out,
                pairs_in: row.pairs_in,
                pairs_out: row.pairs_out,
                metrics: JsonBlob::new(row.metrics.clone()),
                reports: JsonBlob::new(row.reports.clone()),
                artifacts: JsonBlob::new(row.artifacts.clone()),
            };
            known_stages.insert(fact.stage_id.clone());
            by_identity
                .entry((
                    fact.run_id.clone(),
                    fact.stage_id.clone(),
                    fact.tool_id.clone(),
                    fact.params_hash.clone(),
                    fact.input_hash.clone(),
                ))
                .or_default()
                .push(idx);
            table.push(fact);
        }
        Ok(Self {
            rows: table,
            by_identity,
            known_stages,
        })
    }

    pub fn stable_sort(rows: &mut [FactRow]) {
        rows.sort_by(|a, b| a.order_key().cmp(&b.order_key()));
    }
}
