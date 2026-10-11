use super::{
    identity::{token, tool_key, ToolKey},
    rows::{expected_key, parser_key, retained_key},
    VcfEvidenceRows, VcfStageReadinessBinding, VcfStageReadinessBindingKey,
};
use crate::commands::admission::validate_vcf_rendered_command_row;
use crate::model::{parsers::VcfParserFixtureCoverageStatus, scope::VcfActiveStageToolMatrixRow};
use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::fmt::Debug;

fn unique<K: Ord + Debug, V>(
    rows: Vec<V>,
    surface: &str,
    key: impl Fn(&V) -> Result<K>,
) -> Result<BTreeMap<K, V>> {
    let mut index = BTreeMap::new();
    for row in rows {
        let identity = key(&row)?;
        if index.contains_key(&identity) {
            bail!("VCF evidence has duplicate {surface} identity: {identity:?}");
        }
        index.insert(identity, row);
    }
    Ok(index)
}

fn binding_key(key: VcfStageReadinessBindingKey) -> Result<VcfStageReadinessBindingKey> {
    key.validate()?;
    Ok(key)
}

fn same_contract(
    retained: &VcfActiveStageToolMatrixRow,
    adapter: &str,
    parser: &str,
    schema: &str,
    surface: &str,
) -> Result<()> {
    if (adapter, parser, schema)
        != (retained.adapter_id.as_str(), retained.parser_id.as_str(), retained.schema_id.as_str())
    {
        bail!(
            "VCF {surface} evidence disagrees with retained adapter/parser/schema for `{}` / `{}`",
            retained.stage_id,
            retained.tool_id
        );
    }
    Ok(())
}

fn known_binding(
    key: &VcfStageReadinessBindingKey,
    retained: &BTreeMap<VcfStageReadinessBindingKey, VcfActiveStageToolMatrixRow>,
    surface: &str,
) -> Result<()> {
    if !retained.contains_key(key) {
        bail!("VCF {surface} evidence names an unretained binding: {key:?}");
    }
    Ok(())
}

fn known_pair(
    key: &ToolKey,
    retained: &BTreeMap<VcfStageReadinessBindingKey, VcfActiveStageToolMatrixRow>,
    surface: &str,
) -> Result<()> {
    if !retained.keys().any(|binding| binding.pair() == *key) {
        bail!("VCF {surface} evidence names an unretained stage/tool: {key:?}");
    }
    Ok(())
}

pub(super) fn admit(
    rows: VcfEvidenceRows,
) -> Result<BTreeMap<String, Vec<VcfStageReadinessBinding>>> {
    let retained = unique(rows.retained, "retained", |row| {
        for (value, name) in
            [(&row.adapter_id, "adapter"), (&row.parser_id, "parser"), (&row.schema_id, "schema")]
        {
            token(value, name)?;
        }
        binding_key(retained_key(row))
    })?;
    let active = unique(rows.active, "active", |row| binding_key(row.key.clone()))?;
    let commands = unique(rows.commands, "command", |row| {
        validate_vcf_rendered_command_row(row)?;
        tool_key(&row.stage_id, &row.tool_id)
    })?;
    let outputs = unique(rows.outputs, "output", |row| tool_key(&row.stage_id, &row.tool_id))?;
    let parsers = unique(rows.parsers, "parser", |row| binding_key(parser_key(row)))?;
    let expected = unique(rows.expected, "expected result", |row| {
        if row.domain != "vcf" {
            bail!("VCF expected-result evidence has foreign domain `{}`", row.domain);
        }
        binding_key(expected_key(row))
    })?;
    let reports = unique(rows.reports, "report", |row| tool_key(&row.stage_id, &row.tool_id))?;
    for (key, row) in &active {
        known_binding(key, &retained, "active")?;
        if let Some(binding) = retained.get(key) {
            same_contract(binding, &row.adapter_id, &row.parser_id, &row.schema_id, "active")?;
        }
    }
    for (key, row) in &parsers {
        known_binding(key, &retained, "parser")?;
        if let Some(binding) = retained.get(key) {
            same_contract(binding, &row.adapter_id, &row.parser_id, &row.schema_id, "parser")?;
        }
        if row.coverage_status == VcfParserFixtureCoverageStatus::Covered {
            // The native fixture parser/schema and the stage contract use different namespaces.
            token(&row.parser_fixture_parser_id, "fixture parser identity")?;
            token(&row.parser_fixture_schema_id, "fixture schema identity")?;
        }
        if row.raw_fixture_count != row.raw_fixture_paths.len() {
            bail!("VCF parser fixture count disagrees with its paths for {key:?}");
        }
    }
    for key in expected.keys() {
        known_binding(key, &retained, "expected result")?;
    }
    for (surface, keys) in [
        ("command", commands.keys().collect::<Vec<_>>()),
        ("output", outputs.keys().collect()),
        ("report", reports.keys().collect()),
    ] {
        for key in keys {
            known_pair(key, &retained, surface)?;
        }
    }
    let mut stages = BTreeMap::<String, Vec<VcfStageReadinessBinding>>::new();
    for (key, row) in retained {
        let pair = key.pair();
        if let (Some(expected), Some(report)) = (expected.get(&key), reports.get(&pair)) {
            if expected.report_section != report.section_id {
                bail!("VCF report section disagrees with expected result for {key:?}");
            }
        }
        stages.entry(key.stage_id.clone()).or_default().push(VcfStageReadinessBinding {
            retained_row: row,
            active_row: active.get(&key).cloned(),
            command_row: commands.get(&pair).cloned(),
            output_row: outputs.get(&pair).cloned(),
            parser_row: parsers.get(&key).cloned(),
            expected_row: expected.get(&key).cloned(),
            report_row: reports.get(&pair).cloned(),
        });
    }
    Ok(stages)
}
