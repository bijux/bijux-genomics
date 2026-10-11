//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted stage-, tool-, and corpus-centric readiness views.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    model::{
        CorpusCentricReport, ReadinessComparisonContractStatus, StageCentricReport,
        StageCentricStageReport, ToolCentricReport, ToolCentricStageRow,
    },
    BenchError,
};

use super::{
    CORPUS_CENTRIC_REPORT_SCHEMA_V1, STAGE_CENTRIC_REPORT_SCHEMA_V1, TOOL_CENTRIC_REPORT_SCHEMA_V1,
};

/// Validate a stage-centric benchmark readiness report.
///
/// # Errors
/// Returns an error when stage identity, nested tool state, or aggregate closure drifts.
pub fn validate_stage_centric_report(report: &StageCentricReport) -> Result<(), BenchError> {
    if report.schema_version != STAGE_CENTRIC_REPORT_SCHEMA_V1 {
        return invalid("unsupported stage-centric report schema");
    }
    require_text(&report.output_path, "output_path")?;
    if report.stages.is_empty() || report.stage_count != report.stages.len() {
        return invalid("stage-centric stage aggregate does not close");
    }
    if !report.stages.windows(2).all(|pair| {
        (&pair[0].domain, pair[0].canonical_stage_rank, &pair[0].stage_id)
            < (&pair[1].domain, pair[1].canonical_stage_rank, &pair[1].stage_id)
    }) {
        return invalid("stage-centric stages are not in canonical order");
    }

    let mut stage_keys = BTreeSet::new();
    let mut domain_counts = BTreeMap::new();
    let mut section_counts = BTreeMap::new();
    let mut readiness_gap_counts = BTreeMap::new();
    let mut row_count = 0;
    let mut ready_rows = 0;
    let mut blocked_stages = 0;
    let mut multi_tool_stages = 0;
    let mut declared_stages = 0;
    let mut not_declared_stages = 0;

    for stage in &report.stages {
        if !stage_keys.insert((stage.domain.as_str(), stage.stage_id.as_str())) {
            return invalid("stage-centric report repeats a domain/stage identity");
        }
        let observation = validate_stage_centric_stage(stage, &mut readiness_gap_counts)?;

        row_count += stage.tool_count;
        ready_rows += observation.ready_rows;
        blocked_stages += usize::from(observation.blocked_rows > 0);
        multi_tool_stages += usize::from(stage.tool_count > 1);
        declared_stages += usize::from(
            stage.comparison_contract_status == ReadinessComparisonContractStatus::Declared,
        );
        not_declared_stages += usize::from(
            stage.comparison_contract_status == ReadinessComparisonContractStatus::NotDeclared,
        );
        increment(&mut domain_counts, &stage.domain);
        increment(&mut section_counts, &stage.report_section_id);
    }

    if report.multi_tool_stage_count != multi_tool_stages
        || report.blocked_stage_count != blocked_stages
        || report.declared_shared_metric_stage_count != declared_stages
        || report.not_declared_shared_metric_stage_count != not_declared_stages
        || report.row_count != row_count
        || report.benchmark_ready_row_count != ready_rows
        || report.blocked_row_count != row_count - ready_rows
        || report.domain_counts != domain_counts
        || report.section_counts != section_counts
        || report.readiness_gap_counts != readiness_gap_counts
    {
        return invalid("stage-centric top-level aggregates do not close");
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct StageCentricObservation {
    ready_rows: usize,
    blocked_rows: usize,
}

fn validate_stage_centric_stage(
    stage: &StageCentricStageReport,
    readiness_gap_counts: &mut BTreeMap<String, usize>,
) -> Result<StageCentricObservation, BenchError> {
    for (value, field) in [
        (stage.domain.as_str(), "stages[].domain"),
        (stage.stage_id.as_str(), "stages[].stage_id"),
        (stage.report_section_id.as_str(), "stages[].report_section_id"),
        (stage.report_section_title.as_str(), "stages[].report_section_title"),
        (stage.summary_table_id.as_str(), "stages[].summary_table_id"),
        (stage.summary_table_title.as_str(), "stages[].summary_table_title"),
        (stage.anchor_tool_id.as_str(), "stages[].anchor_tool_id"),
        (stage.anchor_support_status.as_str(), "stages[].anchor_support_status"),
    ] {
        require_text(value, field)?;
    }
    if stage.tool_count == 0 || stage.tool_count != stage.tools.len() {
        return invalid("stage-centric nested tool aggregate does not close");
    }
    if !stage.tools.windows(2).all(|pair| pair[0].tool_id < pair[1].tool_id) {
        return invalid("stage-centric tools are not in canonical order");
    }
    validate_unique_text(&stage.shared_metric_fields, true, false, "shared_metric_fields")?;
    if stage.shared_metric_field_count != stage.shared_metric_fields.len() {
        return invalid("stage-centric shared-metric aggregate does not close");
    }

    let mut tool_ids = BTreeSet::new();
    let mut stage_gap_counts = BTreeMap::new();
    let mut ready_rows = 0;
    let mut blocked_tool_ids = Vec::new();
    for tool in &stage.tools {
        validate_stage_centric_tool(tool)?;
        if !tool_ids.insert(tool.tool_id.as_str()) {
            return invalid("stage-centric report repeats a tool within a stage");
        }
        increment(&mut stage_gap_counts, &tool.readiness_gap);
        increment(readiness_gap_counts, &tool.readiness_gap);
        if tool.benchmark_status == "benchmark_ready" {
            ready_rows += 1;
        } else {
            blocked_tool_ids.push(format!("{} ({})", tool.tool_id, tool.readiness_gap));
        }
    }
    if !tool_ids.contains(stage.anchor_tool_id.as_str()) {
        return invalid("stage-centric anchor tool is outside its stage roster");
    }
    let blocked_rows = stage.tool_count - ready_rows;
    if stage.benchmark_ready_tool_count != ready_rows
        || stage.blocked_tool_count != blocked_rows
        || stage.readiness_gap_counts != stage_gap_counts
        || stage.blocked_tool_ids != blocked_tool_ids
    {
        return invalid("stage-centric nested readiness aggregates do not close");
    }
    validate_comparison_contract(
        stage.comparison_contract_status,
        stage.tool_count,
        &stage.shared_metric_fields,
    )?;
    Ok(StageCentricObservation { ready_rows, blocked_rows })
}

fn validate_stage_centric_tool(tool: &crate::model::StageCentricToolRow) -> Result<(), BenchError> {
    for (value, field) in [
        (tool.tool_id.as_str(), "stages[].tools[].tool_id"),
        (tool.benchmark_status.as_str(), "stages[].tools[].benchmark_status"),
        (tool.readiness_gap.as_str(), "stages[].tools[].readiness_gap"),
        (tool.support_status.as_str(), "stages[].tools[].support_status"),
        (tool.adapter_status.as_str(), "stages[].tools[].adapter_status"),
        (tool.parser_status.as_str(), "stages[].tools[].parser_status"),
        (tool.corpus_status.as_str(), "stages[].tools[].corpus_status"),
        (tool.asset_status.as_str(), "stages[].tools[].asset_status"),
        (tool.reason.as_str(), "stages[].tools[].reason"),
    ] {
        require_text(value, field)?;
    }
    Ok(())
}

/// Validate a tool-centric benchmark readiness report.
///
/// # Errors
/// Returns an error when tool identity, nested stage state, or aggregate closure drifts.
pub fn validate_tool_centric_report(report: &ToolCentricReport) -> Result<(), BenchError> {
    if report.schema_version != TOOL_CENTRIC_REPORT_SCHEMA_V1 {
        return invalid("unsupported tool-centric report schema");
    }
    require_text(&report.output_path, "output_path")?;
    if report.tools.is_empty() || report.tool_count != report.tools.len() {
        return invalid("tool-centric tool aggregate does not close");
    }
    if !report.tools.windows(2).all(|pair| pair[0].tool_id < pair[1].tool_id) {
        return invalid("tool-centric tools are not in canonical order");
    }

    let mut tool_ids = BTreeSet::new();
    let mut unique_stages = BTreeSet::new();
    let mut domain_counts = BTreeMap::new();
    let mut readiness_gap_counts = BTreeMap::new();
    let mut row_count = 0;
    let mut ready_rows = 0;
    let mut blocked_tools = 0;

    for tool in &report.tools {
        require_text(&tool.tool_id, "tools[].tool_id")?;
        if !tool_ids.insert(tool.tool_id.as_str()) {
            return invalid("tool-centric report repeats a tool identity");
        }
        if tool.stage_count == 0 || tool.stage_count != tool.stages.len() {
            return invalid("tool-centric nested stage aggregate does not close");
        }
        if !tool
            .stages
            .windows(2)
            .all(|pair| (&pair[0].domain, &pair[0].stage_id) < (&pair[1].domain, &pair[1].stage_id))
        {
            return invalid("tool-centric stages are not in canonical order");
        }
        let mut stage_keys = BTreeSet::new();
        let mut domains = BTreeSet::new();
        let mut sections = BTreeSet::new();
        let mut tool_gap_counts = BTreeMap::new();
        let mut tool_ready_rows = 0;
        let mut blocked_stage_ids = Vec::new();
        for stage in &tool.stages {
            validate_tool_stage(stage)?;
            let key = (stage.domain.as_str(), stage.stage_id.as_str());
            if !stage_keys.insert(key) {
                return invalid("tool-centric report repeats a stage within a tool");
            }
            unique_stages.insert((stage.domain.clone(), stage.stage_id.clone()));
            domains.insert(stage.domain.clone());
            sections.insert(stage.report_section_id.clone());
            increment(&mut tool_gap_counts, &stage.readiness_gap);
            increment(&mut readiness_gap_counts, &stage.readiness_gap);
            increment(&mut domain_counts, &stage.domain);
            if stage.benchmark_status == "benchmark_ready" {
                tool_ready_rows += 1;
            } else {
                blocked_stage_ids.push(format!("{} ({})", stage.stage_id, stage.readiness_gap));
            }
        }
        let blocked_rows = tool.stage_count - tool_ready_rows;
        if tool.domains != domains.into_iter().collect::<Vec<_>>()
            || tool.report_section_ids != sections.into_iter().collect::<Vec<_>>()
            || tool.benchmark_ready_stage_count != tool_ready_rows
            || tool.blocked_stage_count != blocked_rows
            || tool.readiness_gap_counts != tool_gap_counts
            || tool.blocked_stage_ids != blocked_stage_ids
        {
            return invalid("tool-centric nested aggregates do not close");
        }
        row_count += tool.stage_count;
        ready_rows += tool_ready_rows;
        blocked_tools += usize::from(blocked_rows > 0);
    }

    if report.unique_stage_count != unique_stages.len()
        || report.row_count != row_count
        || report.benchmark_ready_row_count != ready_rows
        || report.blocked_row_count != row_count - ready_rows
        || report.blocked_tool_count != blocked_tools
        || report.domain_counts != domain_counts
        || report.readiness_gap_counts != readiness_gap_counts
    {
        return invalid("tool-centric top-level aggregates do not close");
    }
    Ok(())
}

fn validate_tool_stage(stage: &ToolCentricStageRow) -> Result<(), BenchError> {
    for (value, field) in [
        (stage.domain.as_str(), "tools[].stages[].domain"),
        (stage.stage_id.as_str(), "tools[].stages[].stage_id"),
        (stage.report_section_id.as_str(), "tools[].stages[].report_section_id"),
        (stage.report_section_title.as_str(), "tools[].stages[].report_section_title"),
        (stage.summary_table_id.as_str(), "tools[].stages[].summary_table_id"),
        (stage.summary_table_title.as_str(), "tools[].stages[].summary_table_title"),
        (stage.benchmark_status.as_str(), "tools[].stages[].benchmark_status"),
        (stage.readiness_gap.as_str(), "tools[].stages[].readiness_gap"),
        (stage.support_status.as_str(), "tools[].stages[].support_status"),
        (stage.adapter_status.as_str(), "tools[].stages[].adapter_status"),
        (stage.parser_status.as_str(), "tools[].stages[].parser_status"),
        (stage.corpus_status.as_str(), "tools[].stages[].corpus_status"),
        (stage.asset_status.as_str(), "tools[].stages[].asset_status"),
        (stage.reason.as_str(), "tools[].stages[].reason"),
    ] {
        require_text(value, field)?;
    }
    Ok(())
}

/// Validate a corpus-centric benchmark readiness report.
///
/// # Errors
/// Returns an error when corpus identity, assigned stage state, or aggregate closure drifts.
pub fn validate_corpus_centric_report(report: &CorpusCentricReport) -> Result<(), BenchError> {
    if report.schema_version != CORPUS_CENTRIC_REPORT_SCHEMA_V1 {
        return invalid("unsupported corpus-centric report schema");
    }
    require_text(&report.output_path, "output_path")?;
    if report.corpora.is_empty() || report.corpus_count != report.corpora.len() {
        return invalid("corpus-centric corpus aggregate does not close");
    }
    if !report.corpora.windows(2).all(|pair| pair[0].corpus_family_id < pair[1].corpus_family_id) {
        return invalid("corpus-centric corpora are not in canonical order");
    }

    let mut corpus_ids = BTreeSet::new();
    let mut global_stage_keys = BTreeSet::new();
    let mut domain_counts = BTreeMap::new();
    let mut corpus_stage_counts = BTreeMap::new();
    let mut stage_count = 0;
    let mut tool_rows = 0;
    let mut ready_rows = 0;
    let mut blocked_corpora = 0;

    for corpus in &report.corpora {
        require_text(&corpus.corpus_family_id, "corpora[].corpus_family_id")?;
        if !corpus_ids.insert(corpus.corpus_family_id.as_str()) {
            return invalid("corpus-centric report repeats a corpus identity");
        }
        if corpus.stage_count == 0 || corpus.stage_count != corpus.stages.len() {
            return invalid("corpus-centric nested stage aggregate does not close");
        }
        if !corpus.stages.windows(2).all(|pair| {
            (&pair[0].domain, pair[0].canonical_stage_rank, &pair[0].stage_id)
                < (&pair[1].domain, pair[1].canonical_stage_rank, &pair[1].stage_id)
        }) {
            return invalid("corpus-centric stages are not in canonical order");
        }
        let mut domains = BTreeSet::new();
        let mut fixtures = BTreeSet::new();
        let mut corpus_tool_rows = 0;
        let mut corpus_ready_rows = 0;
        let mut blocked_stage_ids = Vec::new();
        for stage in &corpus.stages {
            validate_corpus_stage(stage)?;
            if !global_stage_keys.insert((stage.domain.as_str(), stage.stage_id.as_str())) {
                return invalid("corpus-centric report assigns a stage more than once");
            }
            domains.insert(stage.domain.clone());
            fixtures.extend(stage.fixture_ids.iter().cloned());
            corpus_tool_rows += stage.tool_count;
            corpus_ready_rows += stage.benchmark_ready_tool_count;
            if stage.blocked_tool_count > 0 {
                blocked_stage_ids.push(stage.stage_id.clone());
            }
            increment(&mut domain_counts, &stage.domain);
        }
        if corpus.domains != domains.into_iter().collect::<Vec<_>>()
            || corpus.fixture_ids != fixtures.into_iter().collect::<Vec<_>>()
            || corpus.tool_row_count != corpus_tool_rows
            || corpus.benchmark_ready_tool_row_count != corpus_ready_rows
            || corpus.blocked_stage_count != blocked_stage_ids.len()
            || corpus.blocked_stage_ids != blocked_stage_ids
        {
            return invalid("corpus-centric nested aggregates do not close");
        }
        corpus_stage_counts.insert(corpus.corpus_family_id.clone(), corpus.stage_count);
        stage_count += corpus.stage_count;
        tool_rows += corpus_tool_rows;
        ready_rows += corpus_ready_rows;
        blocked_corpora += usize::from(!corpus.blocked_stage_ids.is_empty());
    }

    if report.stage_count != stage_count
        || report.tool_row_count != tool_rows
        || report.benchmark_ready_tool_row_count != ready_rows
        || report.blocked_tool_row_count != tool_rows - ready_rows
        || report.blocked_corpus_count != blocked_corpora
        || report.domain_counts != domain_counts
        || report.corpus_stage_counts != corpus_stage_counts
    {
        return invalid("corpus-centric top-level aggregates do not close");
    }
    Ok(())
}

fn validate_corpus_stage(stage: &crate::model::CorpusCentricStageRow) -> Result<(), BenchError> {
    for (value, field) in [
        (stage.domain.as_str(), "corpora[].stages[].domain"),
        (stage.stage_id.as_str(), "corpora[].stages[].stage_id"),
        (stage.report_section_id.as_str(), "corpora[].stages[].report_section_id"),
        (stage.report_section_title.as_str(), "corpora[].stages[].report_section_title"),
        (stage.summary_table_id.as_str(), "corpora[].stages[].summary_table_id"),
        (stage.summary_table_title.as_str(), "corpora[].stages[].summary_table_title"),
    ] {
        require_text(value, field)?;
    }
    validate_unique_text(&stage.fixture_ids, false, true, "corpora[].stages[].fixture_ids")?;
    validate_unique_text(&stage.shared_metric_fields, true, false, "shared_metric_fields")?;
    validate_unique_text(&stage.tool_ids, false, true, "corpora[].stages[].tool_ids")?;
    validate_unique_text(&stage.blocked_tool_ids, true, true, "blocked_tool_ids")?;
    if stage.shared_metric_field_count != stage.shared_metric_fields.len()
        || stage.tool_count == 0
        || stage.tool_count != stage.tool_ids.len()
        || stage.benchmark_ready_tool_count + stage.blocked_tool_count != stage.tool_count
        || stage.blocked_tool_count != stage.blocked_tool_ids.len()
    {
        return invalid("corpus-centric stage aggregates do not close");
    }
    validate_comparison_contract(
        stage.comparison_contract_status,
        stage.tool_count,
        &stage.shared_metric_fields,
    )
}

fn validate_comparison_contract(
    status: ReadinessComparisonContractStatus,
    tool_count: usize,
    shared_metrics: &[String],
) -> Result<(), BenchError> {
    match (status, tool_count, shared_metrics.is_empty()) {
        (ReadinessComparisonContractStatus::Declared, 2.., false)
        | (ReadinessComparisonContractStatus::NotDeclared, 2.., true)
        | (ReadinessComparisonContractStatus::NotApplicable, 1, true) => Ok(()),
        _ => invalid("readiness-view comparison contract disagrees with tool cardinality"),
    }
}

fn validate_unique_text(
    values: &[String],
    allow_empty: bool,
    require_canonical_order: bool,
    field: &str,
) -> Result<(), BenchError> {
    if (!allow_empty && values.is_empty())
        || values.iter().any(|value| value.trim().is_empty())
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
        || (require_canonical_order && !values.windows(2).all(|pair| pair[0] < pair[1]))
    {
        return invalid(format!(
            "readiness-view {field} contains invalid, duplicate, or non-canonical values"
        ));
    }
    Ok(())
}

fn increment(counts: &mut BTreeMap<String, usize>, key: &str) {
    *counts.entry(key.to_string()).or_default() += 1;
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("readiness-view requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
