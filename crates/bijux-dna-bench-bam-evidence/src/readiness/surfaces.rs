use super::{
    binding::{binding_key, BindingKey, BindingRow},
    binding_index::collect_rows_by_binding,
    schema::BamStageSchemaContract,
    ReadinessContext,
};
use crate::{
    command_coverage::{BamCommandAdapterCoverageReport, BamCommandAdapterCoverageRow},
    output_contract::{BamAdapterOutputContractReport, BamAdapterOutputContractRow},
    parser_coverage::{BamParserFixtureCoverageReport, BamParserFixtureCoverageRow},
    report_map::{BamReportMapReport, BamReportMapRow},
};
use anyhow::{anyhow, Result};
use bijux_dna_bench_bindings::{
    expected::{ExpectedBenchmarkResultRow, ExpectedBenchmarkResultsReport},
    serving::{ToolServingMapReport, ToolServingMapRow},
};
use std::collections::{BTreeMap, BTreeSet};
pub(super) struct PublishedSources {
    pub active: ToolServingMapReport,
    pub command: BamCommandAdapterCoverageReport,
    pub output: BamAdapterOutputContractReport,
    pub parser: BamParserFixtureCoverageReport,
    pub expected: ExpectedBenchmarkResultsReport,
    pub report_map: BamReportMapReport,
}
impl PublishedSources {
    pub fn acquire(context: &ReadinessContext<'_>) -> Result<Self> {
        let root = context.repo_root;
        let evidence = context.declarations;
        Ok(Self {
            active: evidence.serving(root)?,
            command: evidence.commands(root)?,
            output: evidence.outputs(root)?,
            parser: evidence.parsers(root)?,
            expected: evidence.expected(root)?,
            report_map: evidence.report_map(root)?,
        })
    }
}
pub(super) struct RetainedScope<'a> {
    pub label: &'a str,
    pub bindings: BTreeSet<BindingKey>,
    pub stages: BTreeSet<String>,
}
impl<'a> RetainedScope<'a> {
    pub fn new(
        label: &'a str,
        bindings: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> Self {
        let bindings = bindings
            .into_iter()
            .map(|(stage, tool)| binding_key(stage, tool))
            .collect::<BTreeSet<_>>();
        let stages = bindings.iter().map(|key| key.stage_id.clone()).collect();
        Self { label, bindings, stages }
    }
    fn select<T: BindingRow>(&self, rows: Vec<T>, surface: &str) -> Result<Vec<T>> {
        let rows = rows
            .into_iter()
            .filter(|row| self.bindings.contains(&binding_key(row.stage_id(), row.tool_id())))
            .collect::<Vec<_>>();
        let observed = rows
            .iter()
            .map(|row| binding_key(row.stage_id(), row.tool_id()))
            .collect::<BTreeSet<_>>();
        if observed != self.bindings {
            return Err(anyhow!(
                "{} {surface} rows must cover [{}], found [{}]",
                self.label,
                describe_bindings(&self.bindings),
                describe_bindings(&observed)
            ));
        }
        Ok(rows)
    }
    fn stages(&self, surface: &str, stages: impl Iterator<Item = String>) -> Result<()> {
        let observed = stages.collect::<BTreeSet<_>>();
        if observed != self.stages {
            return Err(anyhow!(
                "{} {surface} rows must cover [{}], found [{}]",
                self.label,
                self.stages.iter().cloned().collect::<Vec<_>>().join(", "),
                observed.iter().cloned().collect::<Vec<_>>().join(", ")
            ));
        }
        Ok(())
    }
}
fn describe_bindings(bindings: &BTreeSet<BindingKey>) -> String {
    bindings
        .iter()
        .map(|key| format!("{} / {}", key.stage_id, key.tool_id))
        .collect::<Vec<_>>()
        .join(", ")
}
pub(super) struct IndexedSources {
    pub active: BTreeMap<BindingKey, ToolServingMapRow>,
    pub command: BTreeMap<BindingKey, BamCommandAdapterCoverageRow>,
    pub output: BTreeMap<BindingKey, BamAdapterOutputContractRow>,
    pub parser: BTreeMap<BindingKey, BamParserFixtureCoverageRow>,
    pub expected: BTreeMap<BindingKey, ExpectedBenchmarkResultRow>,
    pub report_map: BTreeMap<BindingKey, BamReportMapRow>,
    pub schema: BTreeMap<String, BamStageSchemaContract>,
}
impl IndexedSources {
    pub fn admit(
        scope: &RetainedScope<'_>,
        sources: PublishedSources,
        schema: BTreeMap<String, BamStageSchemaContract>,
    ) -> Result<Self> {
        let active = scope.select(sources.active.rows, "active")?;
        let command = scope.select(sources.command.rows, "command")?;
        let output = scope.select(sources.output.rows, "output-contract")?;
        let parser = scope.select(sources.parser.rows, "parser")?;
        let expected = scope.select(
            sources.expected.rows.into_iter().filter(|row| row.domain == "bam").collect(),
            "expected-result",
        )?;
        let reports = sources
            .report_map
            .rows
            .into_iter()
            .filter(|row| scope.stages.contains(&row.stage_id))
            .collect::<Vec<_>>();
        scope.stages("report-map", reports.iter().map(|row| row.stage_id.clone()))?;
        scope.stages("schema", schema.keys().cloned())?;
        let active = collect_rows_by_binding(scope.label, active)?;
        let command = collect_rows_by_binding(scope.label, command)?;
        let output = collect_rows_by_binding(scope.label, output)?;
        let parser = collect_rows_by_binding(scope.label, parser)?;
        let expected = collect_rows_by_binding(scope.label, expected)?;
        let report_map = collect_rows_by_binding(&format!("{} report-map", scope.label), reports)?;
        Ok(Self { active, command, output, parser, expected, report_map, schema })
    }
    pub fn binding(&self, scope: &str, stage: &str, tool: &str) -> Result<BindingEvidence<'_>> {
        let key = binding_key(stage, tool);
        Ok(BindingEvidence {
            active_row: self
                .active
                .get(&key)
                .ok_or_else(|| anyhow!("missing {scope} active row for `{stage}` / `{tool}`"))?,
            command_row: self
                .command
                .get(&key)
                .ok_or_else(|| anyhow!("missing {scope} command row for `{stage}` / `{tool}`"))?,
            output_row: self.output.get(&key).ok_or_else(|| {
                anyhow!("missing {scope} output-contract row for `{stage}` / `{tool}`")
            })?,
            parser_row: self
                .parser
                .get(&key)
                .ok_or_else(|| anyhow!("missing {scope} parser row for `{stage}` / `{tool}`"))?,
            expected_row: self.expected.get(&key).ok_or_else(|| {
                anyhow!("missing {scope} expected-result row for `{stage}` / `{tool}`")
            })?,
            report_map_row: self
                .report_map
                .get(&key)
                .ok_or_else(|| anyhow!("missing {scope} report-map row for `{stage}`"))?,
            schema_contract: self
                .schema
                .get(stage)
                .ok_or_else(|| anyhow!("missing {scope} schema row for `{stage}`"))?,
        })
    }
}
pub(super) struct BindingEvidence<'a> {
    pub active_row: &'a ToolServingMapRow,
    pub command_row: &'a BamCommandAdapterCoverageRow,
    pub output_row: &'a BamAdapterOutputContractRow,
    pub parser_row: &'a BamParserFixtureCoverageRow,
    pub expected_row: &'a ExpectedBenchmarkResultRow,
    pub report_map_row: &'a BamReportMapRow,
    pub schema_contract: &'a BamStageSchemaContract,
}

#[cfg(test)]
mod tests;
