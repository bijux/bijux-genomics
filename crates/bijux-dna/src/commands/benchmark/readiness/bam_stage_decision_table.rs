use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::catalog::{load_registry_tool_matrix, RegistryStagePolicy};
use super::tool_serving_map::{
    render_bam_tool_serving_map, ToolServingMapRow, DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_STAGE_DECISION_TABLE_PATH: &str =
    "target/bench-readiness/bam-stage-decision-table.tsv";
const BAM_STAGE_DECISION_TABLE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_stage_decision_table.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BamStageDecisionKind {
    BenchmarkReady,
    NeedsAdapter,
    NeedsParser,
    NeedsCorpus,
    FutureNotInHpcRound,
}

impl BamStageDecisionKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::BenchmarkReady => "benchmark_ready",
            Self::NeedsAdapter => "needs_adapter",
            Self::NeedsParser => "needs_parser",
            Self::NeedsCorpus => "needs_corpus",
            Self::FutureNotInHpcRound => "future_not_in_hpc_round",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamStageDecisionRow {
    pub(crate) stage_id: String,
    pub(crate) decision: BamStageDecisionKind,
    pub(crate) primary_tool_id: Option<String>,
    pub(crate) selected_tool_id: Option<String>,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamStageDecisionTableReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) row_count: usize,
    pub(crate) decision_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<BamStageDecisionRow>,
}

pub(crate) fn run_render_bam_stage_decision_table(
    args: &parse::BenchReadinessRenderBamStageDecisionTableArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_stage_decision_table(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_STAGE_DECISION_TABLE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_stage_decision_table(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamStageDecisionTableReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let inventory = load_local_stage_inventory(repo_root, BenchLocalDomain::Bam)?;
    let tool_map =
        render_bam_tool_serving_map(repo_root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))?;
    let registry = load_registry_tool_matrix(repo_root)?;
    let rows_by_stage = tool_map.rows.iter().cloned().fold(
        BTreeMap::<String, Vec<ToolServingMapRow>>::new(),
        |mut acc, row| {
            acc.entry(row.stage_id.clone()).or_default().push(row);
            acc
        },
    );

    let mut rows = Vec::with_capacity(inventory.stages.len());
    for stage in &inventory.stages {
        let stage_rows = rows_by_stage
            .get(stage.stage_id.as_str())
            .cloned()
            .ok_or_else(|| anyhow!("BAM readiness map is missing stage `{}`", stage.stage_id))?;
        rows.push(render_stage_decision_row(
            stage.stage_id.as_str(),
            &stage_rows,
            registry.stage_policies.get(stage.stage_id.as_str()),
        ));
    }
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_bam_stage_decision_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut decision_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *decision_counts.entry(row.decision.as_str().to_string()).or_default() += 1;
    }

    Ok(BamStageDecisionTableReport {
        schema_version: BAM_STAGE_DECISION_TABLE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count: inventory.stage_count,
        row_count: rows.len(),
        decision_counts,
        rows,
    })
}

fn render_stage_decision_row(
    stage_id: &str,
    rows: &[ToolServingMapRow],
    stage_policy: Option<&RegistryStagePolicy>,
) -> BamStageDecisionRow {
    if let Some(stage_policy) = stage_policy {
        let primary_tool_id = stage_policy.primary_tool_ids.first().cloned();
        if let Some(row) = select_row(rows, stage_policy, row_is_benchmark_ready) {
            return build_stage_decision_row(
                stage_id,
                BamStageDecisionKind::BenchmarkReady,
                primary_tool_id,
                Some(row),
                format!(
                    "stage `{stage_id}` is benchmark_ready via `{}` with a fixture-backed parser-validated BAM benchmark row{}",
                    row.tool_id,
                    primary_gap_suffix(stage_policy, row)
                ),
            );
        }
        if let Some(row) = select_row(rows, stage_policy, row_has_supported_adapter_and_parser) {
            return build_stage_decision_row(
                stage_id,
                BamStageDecisionKind::NeedsCorpus,
                primary_tool_id,
                Some(row),
                format!(
                    "stage `{stage_id}` has parser-validated BAM benchmark tooling via `{}` but still resolves only planner-only corpus coverage{}",
                    row.tool_id,
                    primary_gap_suffix(stage_policy, row)
                ),
            );
        }
        if let Some(row) = select_row(rows, stage_policy, row_has_supported_adapter) {
            return build_stage_decision_row(
                stage_id,
                BamStageDecisionKind::NeedsParser,
                primary_tool_id,
                Some(row),
                format!(
                    "stage `{stage_id}` has a supported adapter-backed BAM benchmark row via `{}` but no parser-fixture-validated result normalizer{}",
                    row.tool_id,
                    primary_gap_suffix(stage_policy, row)
                ),
            );
        }
        if let Some(row) = select_row(rows, stage_policy, row_is_supported) {
            return build_stage_decision_row(
                stage_id,
                BamStageDecisionKind::NeedsAdapter,
                primary_tool_id,
                Some(row),
                format!(
                    "stage `{stage_id}` has a supported benchmark tool declaration via `{}` but no runnable or plannable adapter{}",
                    row.tool_id,
                    primary_gap_suffix(stage_policy, row)
                ),
            );
        }
        let fallback_row = best_row(rows, stage_policy, |_| true);
        return build_stage_decision_row(
            stage_id,
            BamStageDecisionKind::FutureNotInHpcRound,
            primary_tool_id,
            fallback_row,
            match fallback_row {
                Some(row) => format!(
                    "stage `{stage_id}` is in the governed BAM registry, but no registered row is currently supported; strongest admitted row `{}` remains `{}`{}",
                    row.tool_id,
                    row.support_status,
                    primary_gap_suffix(stage_policy, row)
                ),
                None => format!(
                    "stage `{stage_id}` is in the governed BAM registry, but no admitted BAM tool rows are currently available"
                ),
            },
        );
    }

    let fallback_row = best_row(rows, &RegistryStagePolicy::empty(stage_id), |_| true);
    build_stage_decision_row(
        stage_id,
        BamStageDecisionKind::FutureNotInHpcRound,
        None,
        fallback_row,
        match fallback_row {
            Some(row) => format!(
                "stage `{stage_id}` is not yet in the governed BAM benchmark registry; strongest admitted row `{}` remains `{}`",
                row.tool_id, row.support_status
            ),
            None => format!(
                "stage `{stage_id}` is not yet in the governed BAM benchmark registry and has no admitted BAM tool rows"
            ),
        },
    )
}

fn build_stage_decision_row(
    stage_id: &str,
    decision: BamStageDecisionKind,
    primary_tool_id: Option<String>,
    selected_row: Option<&ToolServingMapRow>,
    reason: String,
) -> BamStageDecisionRow {
    BamStageDecisionRow {
        stage_id: stage_id.to_string(),
        decision,
        primary_tool_id,
        selected_tool_id: selected_row.map(|row| row.tool_id.clone()),
        support_status: selected_row
            .map(|row| row.support_status.clone())
            .unwrap_or_else(|| "missing".to_string()),
        adapter_status: selected_row
            .map(|row| row.adapter_status.clone())
            .unwrap_or_else(|| "missing".to_string()),
        parser_status: selected_row
            .map(|row| row.parser_status.clone())
            .unwrap_or_else(|| "missing".to_string()),
        corpus_status: selected_row
            .map(|row| row.corpus_status.clone())
            .unwrap_or_else(|| "missing".to_string()),
        reason,
    }
}

fn select_row<'a>(
    rows: &'a [ToolServingMapRow],
    stage_policy: &RegistryStagePolicy,
    predicate: impl Fn(&ToolServingMapRow) -> bool,
) -> Option<&'a ToolServingMapRow> {
    best_row(rows, stage_policy, predicate)
}

fn best_row<'a>(
    rows: &'a [ToolServingMapRow],
    stage_policy: &RegistryStagePolicy,
    predicate: impl Fn(&ToolServingMapRow) -> bool,
) -> Option<&'a ToolServingMapRow> {
    let preferred_tool_order = stage_policy.preferred_tool_order();
    rows.iter().filter(|row| predicate(row)).min_by(|left, right| {
        row_preference_key(left, &preferred_tool_order)
            .cmp(&row_preference_key(right, &preferred_tool_order))
    })
}

fn row_preference_key<'a>(
    row: &'a ToolServingMapRow,
    preferred_tool_order: &[String],
) -> (usize, usize, usize, usize, usize, &'a str) {
    let preferred_index = preferred_tool_order
        .iter()
        .position(|tool_id| tool_id == row.tool_id.as_str())
        .unwrap_or(preferred_tool_order.len());
    (
        preferred_index,
        support_rank(&row.support_status),
        adapter_rank(&row.adapter_status),
        parser_rank(&row.parser_status),
        corpus_rank(&row.corpus_status),
        row.tool_id.as_str(),
    )
}

fn support_rank(value: &str) -> usize {
    match value {
        "supported" => 0,
        "planned" => 1,
        "mismatched_contract" => 2,
        "missing_contract" => 3,
        _ => 4,
    }
}

fn adapter_rank(value: &str) -> usize {
    match value {
        "runnable" => 0,
        "plannable" => 1,
        "declared_only" => 2,
        _ => 3,
    }
}

fn parser_rank(value: &str) -> usize {
    match value {
        "parser_fixture_validated" => 0,
        "scientific_report_contract" => 1,
        "artifact_contract_only" => 2,
        _ => 3,
    }
}

fn corpus_rank(value: &str) -> usize {
    if value.starts_with("fixture:") {
        0
    } else if value == "planner_only" {
        1
    } else {
        2
    }
}

fn row_is_supported(row: &ToolServingMapRow) -> bool {
    row.support_status == "supported"
}

fn row_has_supported_adapter(row: &ToolServingMapRow) -> bool {
    row_is_supported(row) && matches!(row.adapter_status.as_str(), "runnable" | "plannable")
}

fn row_has_supported_adapter_and_parser(row: &ToolServingMapRow) -> bool {
    row_has_supported_adapter(row) && row.parser_status == "parser_fixture_validated"
}

fn row_is_benchmark_ready(row: &ToolServingMapRow) -> bool {
    row_has_supported_adapter_and_parser(row) && row.corpus_status.starts_with("fixture:")
}

fn primary_gap_suffix(stage_policy: &RegistryStagePolicy, row: &ToolServingMapRow) -> String {
    match stage_policy.primary_tool_ids.first() {
        Some(primary_tool_id) if primary_tool_id != &row.tool_id => {
            format!("; primary `{primary_tool_id}` is not currently the strongest eligible row")
        }
        _ => String::new(),
    }
}

fn render_bam_stage_decision_tsv(rows: &[BamStageDecisionRow]) -> String {
    let mut rendered = String::from(
        "stage_id\tdecision\tprimary_tool_id\tselected_tool_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            row.decision.as_str(),
            sanitize_tsv(row.primary_tool_id.as_deref().unwrap_or("")),
            sanitize_tsv(row.selected_tool_id.as_deref().unwrap_or("")),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

impl RegistryStagePolicy {
    fn preferred_tool_order(&self) -> Vec<String> {
        let mut seen = BTreeSet::new();
        let mut tool_ids = Vec::new();
        for tool_id in self
            .primary_tool_ids
            .iter()
            .chain(self.optional_alternative_tool_ids.iter())
            .chain(self.validation_tool_ids.iter())
            .chain(self.reporting_tool_ids.iter())
        {
            if seen.insert(tool_id.clone()) {
                tool_ids.push(tool_id.clone());
            }
        }
        tool_ids
    }

    fn empty(stage_id: &str) -> Self {
        Self {
            stage_id: stage_id.to_string(),
            primary_tool_ids: Vec::new(),
            optional_alternative_tool_ids: Vec::new(),
            validation_tool_ids: Vec::new(),
            reporting_tool_ids: Vec::new(),
            default_rationale: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_stage_decision_table, BamStageDecisionKind,
        BAM_STAGE_DECISION_TABLE_SCHEMA_VERSION, DEFAULT_BAM_STAGE_DECISION_TABLE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn bam_stage_decision_table_reports_governed_stage_blockers() {
        let report = render_bam_stage_decision_table(
            &repo_root(),
            PathBuf::from(DEFAULT_BAM_STAGE_DECISION_TABLE_PATH),
        )
        .expect("render BAM stage decision table");

        assert_eq!(report.schema_version, BAM_STAGE_DECISION_TABLE_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 24);
        assert_eq!(report.row_count, 24);
        assert_eq!(report.decision_counts.get("benchmark_ready"), Some(&15));
        assert_eq!(report.decision_counts.get("needs_corpus"), None);
        assert_eq!(report.decision_counts.get("needs_parser"), Some(&7));
        assert_eq!(report.decision_counts.get("future_not_in_hpc_round"), Some(&2));
        assert_eq!(report.decision_counts.get("needs_adapter"), None);

        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.validate"
                && row.decision == BamStageDecisionKind::BenchmarkReady
                && row.primary_tool_id.as_deref() == Some("samtools")
                && row.selected_tool_id.as_deref() == Some("samtools")
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.damage"
                && row.decision == BamStageDecisionKind::BenchmarkReady
                && row.primary_tool_id.as_deref() == Some("mapdamage2")
                && row.selected_tool_id.as_deref() == Some("mapdamage2")
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-adna-damage-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.align"
                && row.decision == BamStageDecisionKind::NeedsParser
                && row.primary_tool_id.as_deref() == Some("bwa")
                && row.selected_tool_id.as_deref() == Some("bwa")
                && row.parser_status == "artifact_contract_only"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.coverage"
                && row.decision == BamStageDecisionKind::BenchmarkReady
                && row.primary_tool_id.as_deref() == Some("mosdepth")
                && row.selected_tool_id.as_deref() == Some("mosdepth")
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.gc_bias"
                && row.decision == BamStageDecisionKind::BenchmarkReady
                && row.primary_tool_id.as_deref() == Some("picard")
                && row.selected_tool_id.as_deref() == Some("picard")
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.insert_size"
                && row.decision == BamStageDecisionKind::BenchmarkReady
                && row.primary_tool_id.as_deref() == Some("picard")
                && row.selected_tool_id.as_deref() == Some("picard")
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.complexity"
                && row.decision == BamStageDecisionKind::BenchmarkReady
                && row.primary_tool_id.as_deref() == Some("preseq")
                && row.selected_tool_id.as_deref() == Some("preseq")
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.endogenous_content"
                && row.decision == BamStageDecisionKind::BenchmarkReady
                && row.primary_tool_id.as_deref() == Some("samtools")
                && row.selected_tool_id.as_deref() == Some("samtools")
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.overlap_correction"
                && row.decision == BamStageDecisionKind::BenchmarkReady
                && row.primary_tool_id.as_deref() == Some("bamutil")
                && row.selected_tool_id.as_deref() == Some("bamutil")
                && row.support_status == "supported"
                && row.adapter_status == "runnable"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.bias_mitigation"
                && row.decision == BamStageDecisionKind::NeedsParser
                && row.primary_tool_id.as_deref() == Some("samtools")
                && row.selected_tool_id.as_deref() == Some("mapdamage2")
                && row.support_status == "supported"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.haplogroups"
                && row.decision == BamStageDecisionKind::NeedsParser
                && row.primary_tool_id.as_deref() == Some("samtools")
                && row.selected_tool_id.as_deref() == Some("yleaf")
        }));
        for stage_id in ["bam.genotyping", "bam.recalibration"] {
            assert!(report.rows.iter().any(|row| {
                row.stage_id == stage_id
                    && row.decision == BamStageDecisionKind::FutureNotInHpcRound
                    && row.primary_tool_id.is_none()
            }));
        }
    }
}
