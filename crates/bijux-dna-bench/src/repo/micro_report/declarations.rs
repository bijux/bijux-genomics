//! Owner: acquire declared resources and project applicable scientific thresholds.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    validate_stage_tool_resources_config, MicroBenchmarkResultRow,
    MicroBenchmarkScienceThresholdRow, ScientificAcceptanceThresholdRow,
    ScientificAcceptanceThresholdsConfig, StageToolResourceRow, StageToolResourcesConfig,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
pub(super) fn load_stage_tool_resources(
    config_path: PathBuf,
) -> Result<BTreeMap<(String, String, String), StageToolResourceRow>> {
    let payload = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: StageToolResourcesConfig =
        toml::from_str(&payload).with_context(|| format!("parse {}", config_path.display()))?;
    validate_stage_tool_resources_config(&config)?;
    Ok(config
        .rows
        .into_iter()
        .map(|row| ((row.domain.clone(), row.stage_id.clone(), row.tool_id.clone()), row))
        .collect())
}

pub(super) fn load_science_threshold_rows(
    config_path: PathBuf,
    result_rows: &[MicroBenchmarkResultRow],
) -> Result<Vec<MicroBenchmarkScienceThresholdRow>> {
    let payload = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: ScientificAcceptanceThresholdsConfig =
        toml::from_str(&payload).with_context(|| format!("parse {}", config_path.display()))?;
    let stage_tools = result_rows.iter().fold(
        BTreeMap::<(String, String), BTreeSet<String>>::new(),
        |mut acc, row| {
            acc.entry((row.domain.clone(), row.stage_id.clone()))
                .or_default()
                .insert(row.tool_id.clone());
            acc
        },
    );
    let mut rows = config
        .rows
        .into_iter()
        .filter_map(|row| {
            stage_tools
                .get(&(row.domain.clone(), row.stage_id.clone()))
                .map(|tool_ids| science_threshold_row(row, tool_ids))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.metric_id.cmp(&right.metric_id))
    });
    Ok(rows)
}

fn science_threshold_row(
    row: ScientificAcceptanceThresholdRow,
    tool_ids: &BTreeSet<String>,
) -> MicroBenchmarkScienceThresholdRow {
    MicroBenchmarkScienceThresholdRow {
        domain: row.domain,
        stage_id: row.stage_id,
        metric_id: row.metric_id,
        metric_name: row.metric_name,
        unit: row.unit,
        direction: row.direction.as_str().to_string(),
        tolerance_kind: row.tolerance_kind.as_str().to_string(),
        tolerance_value: row.tolerance_value,
        pass_rule: row.pass_rule.as_str().to_string(),
        insufficiency_behavior: row.insufficiency_behavior.as_str().to_string(),
        required: row.required,
        declaration_origin: row.declaration_origin,
        covered_tool_ids: tool_ids.iter().cloned().collect(),
    }
}
