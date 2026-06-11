use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

use crate::model::JsonBlob;
use bijux_dna_core::contract::objective_spec;
use bijux_dna_core::contract::Objective;
use bijux_dna_domain_bam::prelude::STAGE_PREFIX as BAM_STAGE_PREFIX;
use bijux_dna_domain_fastq::prelude::STAGE_PREFIX as FASTQ_STAGE_PREFIX;
use bijux_dna_runtime::FactsRowV1;

pub fn required_vcf_metric_keys(stage_id: &str) -> &'static [&'static str] {
    match stage_id {
        "vcf.impute" => &["imputation_info_mean", "rsq_mean", "missingness_post"],
        "vcf.roh" => &["segment_count", "total_length"],
        "vcf.ibd" => &["ibd_segment_count", "ibd_total_length_cM"],
        "vcf.demography" => &["ne_recent"],
        _ => &[],
    }
}

pub fn analysis_selection_contract_section() -> serde_json::Value {
    let objectives =
        [Objective::Speed, Objective::Memory, Objective::Retention, Objective::Balanced]
            .iter()
            .map(|objective| {
                let spec = objective_spec(*objective);
                serde_json::json!({
                    "objective": objective.as_str(),
                    "weights": {
                        "runtime": spec.weights.runtime,
                        "memory": spec.weights.memory,
                        "retention": spec.weights.retention,
                    },
                    "scoring": "weighted_sum(runtime, memory, retention)",
                    "ranking": "lower score is better",
                })
            })
            .collect::<Vec<_>>();

    serde_json::json!({
        "selection_strategy": "objective_weights",
        "criteria": [
            "runtime_s",
            "memory_mb",
            "retention_ratio",
        ],
        "objectives": objectives,
        "notes": "Selection uses bench medians per tool and the configured objective weights.",
    })
}

pub fn pipeline_defaults_section(base_dir: &Path) -> Result<serde_json::Value> {
    let defaults_path = base_dir.join("defaults_ledger.json");
    let raw = std::fs::read_to_string(&defaults_path)
        .with_context(|| format!("missing defaults ledger at {}", defaults_path.display()))?;
    let typed = serde_json::from_str::<bijux_dna_pipelines::DefaultsLedgerV1>(&raw)
        .context("parse typed defaults ledger json")?;
    typed.validate_strict()?;
    let parsed =
        serde_json::from_str::<serde_json::Value>(&raw).context("parse defaults ledger json")?;
    Ok(serde_json::json!({
        "defaults_ledger": parsed,
        "overrides": [],
    }))
}

pub fn enforce_report_completeness_contract(
    rows: &[FactsRowV1],
    sections: &BTreeMap<String, JsonBlob>,
) -> Result<()> {
    let has_fastq = rows.iter().any(|row| row.stage_id.starts_with(FASTQ_STAGE_PREFIX));
    let has_bam = rows.iter().any(|row| row.stage_id.starts_with(BAM_STAGE_PREFIX));
    let has_vcf = rows.iter().any(|row| row.stage_id.starts_with("vcf."));
    if has_fastq && !sections.contains_key("fastq") {
        return Err(anyhow::anyhow!(
            "report completeness contract violation: missing fastq section"
        ));
    }
    if has_bam && !sections.contains_key("bam") {
        return Err(anyhow::anyhow!("report completeness contract violation: missing bam section"));
    }
    if has_vcf && !sections.contains_key("vcf") {
        return Err(anyhow::anyhow!("report completeness contract violation: missing vcf section"));
    }
    Ok(())
}
