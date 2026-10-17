use crate::support::Evidence;
use anyhow::{bail, Result};
use bijux_dna_bench_bam_evidence::{
    command_coverage::render_bam_command_adapter_coverage,
    observations::ServingBinding,
    output_contract::render_bam_adapter_output_contract,
    parser_coverage::render_bam_parser_fixture_coverage,
    report_map::{BamReportMapRow, BAM_REPORT_MAP_SCHEMA_VERSION},
};
use bijux_dna_bench_bam_evidence::{
    command_coverage::BamCommandAdapterCoverageReport,
    output_contract::BamAdapterOutputContractReport,
    parser_coverage::BamParserFixtureCoverageReport,
    readiness::{
        ports::{DeclarationEvidence, LocalProofKind, LocalProofWriter},
        ReadinessContext, ReadinessPaths,
    },
    report_map::BamReportMapReport,
};
use bijux_dna_bench_bindings::{expected::ExpectedBenchmarkResultRow, serving::ToolServingMapRow};
use bijux_dna_bench_bindings::{
    expected::{ExpectedBenchmarkResultsReport, EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION},
    serving::{ToolServingMapReport, BAM_TOOL_SERVING_MAP_SCHEMA_VERSION},
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

const DAMAGE_AUTHENTICITY: &[(&str, &str)] = &[
    ("bam.authenticity", "authenticct"),
    ("bam.authenticity", "damageprofiler"),
    ("bam.authenticity", "pmdtools"),
    ("bam.damage", "addeam"),
    ("bam.damage", "damageprofiler"),
    ("bam.damage", "mapdamage2"),
    ("bam.damage", "pmdtools"),
    ("bam.damage", "pydamage"),
];

/// Captured declaration reports deliberately stop at the native boundary, without
/// substituting generated scientific proof bytes or claiming completed readiness.
pub struct AcquiredDeclarations {
    serving: ToolServingMapReport,
    commands: BamCommandAdapterCoverageReport,
    outputs: BamAdapterOutputContractReport,
    parsers: BamParserFixtureCoverageReport,
    expected: ExpectedBenchmarkResultsReport,
    report_map: BamReportMapReport,
    pub calls: RefCell<Vec<&'static str>>,
    pub native_calls: RefCell<Vec<LocalProofKind>>,
}

impl AcquiredDeclarations {
    pub fn damage_authenticity(root: &Path) -> Result<Self> {
        let evidence = Evidence::default();
        let bindings = DAMAGE_AUTHENTICITY
            .iter()
            .map(|(stage, tool)| ServingBinding {
                stage_id: (*stage).to_string(),
                tool_id: (*tool).to_string(),
                support_status: "supported".to_string(),
                adapter_status: "runnable".to_string(),
                parser_status: "parser_fixture_validated".to_string(),
                corpus_status: "fixture:observed-corpus".to_string(),
            })
            .collect::<Vec<_>>();
        let stage_count = bindings.iter().map(|row| &row.stage_id).collect::<BTreeSet<_>>().len();
        let tool_count = bindings.iter().map(|row| &row.tool_id).collect::<BTreeSet<_>>().len();
        *evidence.serving_override.borrow_mut() = Some(bindings.clone());
        let context = evidence.context(root);
        let mut commands =
            render_bam_command_adapter_coverage(&context, Path::new("captured/commands.tsv"))?;
        let outputs =
            render_bam_adapter_output_contract(&context, Path::new("captured/outputs.tsv"))?;
        let mut parsers =
            render_bam_parser_fixture_coverage(&context, Path::new("captured/parsers.tsv"))?;
        commands.stage_count = stage_count;
        commands.tool_count = tool_count;
        parsers.stage_count = stage_count;
        parsers.tool_count = tool_count;
        let active_rows = bindings
            .iter()
            .map(|row| ToolServingMapRow {
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                support_status: row.support_status.clone(),
                adapter_status: row.adapter_status.clone(),
                parser_status: row.parser_status.clone(),
                corpus_status: row.corpus_status.clone(),
            })
            .collect::<Vec<_>>();
        Ok(Self {
            serving: ToolServingMapReport {
                schema_version: BAM_TOOL_SERVING_MAP_SCHEMA_VERSION,
                domain: "bam",
                output_path: "captured/serving.tsv".to_string(),
                stage_count,
                tool_count,
                row_count: active_rows.len(),
                rows: active_rows,
            },
            commands,
            outputs,
            parsers,
            expected: Self::expected_report(&bindings, stage_count, tool_count),
            report_map: Self::report_map(&bindings, stage_count, tool_count),
            calls: RefCell::new(Vec::new()),
            native_calls: RefCell::new(Vec::new()),
        })
    }

    fn expected_report(
        bindings: &[ServingBinding],
        stage_count: usize,
        tool_count: usize,
    ) -> ExpectedBenchmarkResultsReport {
        let expected_rows = bindings
            .iter()
            .map(|row| ExpectedBenchmarkResultRow {
                result_row_id: format!("{}:{}", row.stage_id, row.tool_id),
                domain: "bam".to_string(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                readiness_kind: "local".to_string(),
                corpus_family_id: "observed-corpus".to_string(),
                fixture_id: "fixture".to_string(),
                sample_scope: "sample".to_string(),
                expected_output_artifact_ids: vec!["declared.metrics".to_string()],
                raw_output_artifact_ids: vec![],
                normalized_metrics_output_id: Some("declared.metrics".to_string()),
                result_root: "captured/results".to_string(),
                stage_result_manifest_path: "captured/results/manifest.json".to_string(),
                stdout_path: "captured/results/stdout.log".to_string(),
                stderr_path: "captured/results/stderr.log".to_string(),
                reason: "captured binding declaration".to_string(),
            })
            .collect::<Vec<_>>();
        ExpectedBenchmarkResultsReport {
            schema_version: EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION,
            output_path: "captured/expected.tsv".to_string(),
            stage_count,
            tool_count,
            row_count: expected_rows.len(),
            domain_counts: BTreeMap::from([("bam".to_string(), expected_rows.len())]),
            fixture_counts: BTreeMap::from([("fixture".to_string(), expected_rows.len())]),
            rows: expected_rows,
        }
    }

    fn report_map(
        bindings: &[ServingBinding],
        stage_count: usize,
        tool_count: usize,
    ) -> BamReportMapReport {
        let map_rows = bindings
            .iter()
            .enumerate()
            .map(|(rank, row)| BamReportMapRow {
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                result_row_id: format!("{}:{}", row.stage_id, row.tool_id),
                corpus_family_id: "observed-corpus".to_string(),
                fixture_id: "fixture".to_string(),
                sample_scope: "sample".to_string(),
                canonical_stage_rank: rank + 1,
                readiness_kind: "local".to_string(),
                criticality: "required".to_string(),
                support_status: row.support_status.clone(),
                anchor_tool_id: row.tool_id.clone(),
                anchor_support_status: row.support_status.clone(),
                report_section_id: "ancient_signal".to_string(),
                report_section_title: "Ancient signal".to_string(),
                summary_table_id: "damage_authenticity".to_string(),
                summary_table_title: "Damage and authenticity".to_string(),
                workflow_branch_id: None,
                scientific_context_required: vec![],
                report_focus: "damage and authenticity".to_string(),
                reason: "captured binding declaration".to_string(),
            })
            .collect::<Vec<_>>();
        BamReportMapReport {
            schema_version: BAM_REPORT_MAP_SCHEMA_VERSION,
            domain: "bam",
            output_path: "captured/map.tsv".to_string(),
            expected_result_row_count: map_rows.len(),
            row_count: map_rows.len(),
            stage_count,
            tool_count,
            section_count: 1,
            summary_table_count: 1,
            section_counts: BTreeMap::from([("ancient_signal".to_string(), map_rows.len())]),
            branch_counts: BTreeMap::new(),
            rows: map_rows,
        }
    }

    pub fn context<'a>(&'a self, root: &'a Path) -> ReadinessContext<'a> {
        ReadinessContext {
            repo_root: root,
            declarations: self,
            native: self,
            paths: ReadinessPaths {
                serving: "captured/serving.tsv",
                commands: "captured/commands.tsv",
                outputs: "captured/outputs.tsv",
                parsers: "captured/parsers.tsv",
                expected: "captured/expected.tsv",
                report_map: "captured/map.tsv",
                schema: "captured/schema.json",
            },
        }
    }

    pub fn remove_retained_binding(&mut self) {
        self.serving.rows.remove(0);
        self.serving.row_count = self.serving.rows.len();
    }

    pub fn duplicate_retained_binding(&mut self) {
        self.serving.rows.push(self.serving.rows[0].clone());
        self.serving.row_count = self.serving.rows.len();
    }

    pub fn substitute_foreign_stage(&mut self) {
        self.serving.rows[0].stage_id = "bam.foreign-stage".to_string();
    }
}

impl DeclarationEvidence for AcquiredDeclarations {
    fn serving(&self, _: &Path) -> Result<ToolServingMapReport> {
        self.calls.borrow_mut().push("serving");
        Ok(self.serving.clone())
    }
    fn commands(&self, _: &Path) -> Result<BamCommandAdapterCoverageReport> {
        self.calls.borrow_mut().push("commands");
        Ok(self.commands.clone())
    }
    fn outputs(&self, _: &Path) -> Result<BamAdapterOutputContractReport> {
        self.calls.borrow_mut().push("outputs");
        Ok(self.outputs.clone())
    }
    fn parsers(&self, _: &Path) -> Result<BamParserFixtureCoverageReport> {
        self.calls.borrow_mut().push("parsers");
        Ok(self.parsers.clone())
    }
    fn expected(&self, _: &Path) -> Result<ExpectedBenchmarkResultsReport> {
        self.calls.borrow_mut().push("expected");
        Ok(self.expected.clone())
    }
    fn report_map(&self, _: &Path) -> Result<BamReportMapReport> {
        self.calls.borrow_mut().push("report-map");
        Ok(self.report_map.clone())
    }
    fn schema(&self) -> Result<serde_json::Value> {
        self.calls.borrow_mut().push("schema");
        let required = json!([
            "schema_version",
            "stage_id",
            "tool_id",
            "tool_version",
            "execution",
            "outputs_count",
            "artifacts",
            "contracts",
            "normalized_keys"
        ]);
        Ok(json!({"$defs":{"stages":{
            "bam.damage":{"allOf":[{}, {"x-bijux-extension-id":"bam_damage_normalized_v1","required":required}]},
            "bam.authenticity":{"allOf":[{}, {"x-bijux-extension-id":"bam_authenticity_normalized_v1","required":required}]}
        }}}))
    }
}

impl LocalProofWriter for AcquiredDeclarations {
    fn write(&self, _: &Path, kind: LocalProofKind) -> Result<PathBuf> {
        self.native_calls.borrow_mut().push(kind);
        bail!("captured native boundary reached")
    }
}
