use crate::bam::assignment::admission::governed::ensure_bam_fixture_assignment_coverage;
use crate::bam::assignment::admission::rows::ensure_row_completeness;
use crate::bam::assignment::compatibility::load_bam_stage_compatibility;
use crate::bam::assignment::evidence::acquisition::load_bam_corpus_assignment_context;
use crate::bam::assignment::models::BamCorpusAssignmentRow;
use crate::bam::assignment::AssignmentContext;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_core::ids::StageId;
use bijux_dna_core::ids::ToolId;
use bijux_dna_domain_bam::benchmark_corpus_assignment_for_stage_tool;
/// Construct the governed BAM corpus assignment evidence.
///
/// # Errors
/// Returns canonical source, fixture identity, scientific contract or publication failure.
pub fn collect_bam_corpus_assignment_rows(
    context: &AssignmentContext<'_>,
) -> Result<(usize, usize, Vec<BamCorpusAssignmentRow>)> {
    let repo_root = context.repo_root;
    let compatibility_by_stage = load_bam_stage_compatibility(context)?;
    let stage_context = load_bam_corpus_assignment_context(repo_root)?;
    let coverage = context.coverage.coverage(repo_root)?;
    let (stage_count, tool_count, coverage_rows) =
        (coverage.stage_count, coverage.tool_count, coverage.rows);
    let mut rows = Vec::with_capacity(coverage_rows.len());

    for row in coverage_rows {
        let stage_id = StageId::new(row.stage_id.clone());
        let tool_id = ToolId::new(row.tool_id.clone());
        let domain_assignment = benchmark_corpus_assignment_for_stage_tool(&stage_id, &tool_id)
            .ok_or_else(|| {
                anyhow!("missing BAM corpus assignment for `{}` / `{}`", row.stage_id, row.tool_id)
            })?;
        let stage_compatibility = compatibility_by_stage.get(&row.stage_id).ok_or_else(|| {
            anyhow!("missing BAM corpus compatibility for stage `{}`", row.stage_id)
        })?;
        let compatibility_family =
            stage_compatibility.corpus_family_id.as_deref().ok_or_else(|| {
                anyhow!(
                    "BAM stage `{}` is missing corpus_family_id in local compatibility",
                    row.stage_id
                )
            })?;
        let fixture_id = stage_compatibility.fixture_id.as_deref().ok_or_else(|| {
            anyhow!("BAM stage `{}` is missing fixture_id in local compatibility", row.stage_id)
        })?;
        let assigned_family = domain_assignment.assigned_family();
        if compatibility_family != assigned_family.as_str() {
            return Err(anyhow!(
                "BAM stage `{}` assigns `{}` in the domain contract but `{}` in local compatibility",
                row.stage_id,
                assigned_family.as_str(),
                compatibility_family
            ));
        }
        let evidence = stage_context.evidence_for_stage(&row.stage_id);

        rows.push(BamCorpusAssignmentRow {
            tool_id: row.tool_id,
            stage_id: row.stage_id,
            benchmark_status: benchmark_status_label(row.benchmark_ready).to_string(),
            support_status: row.support_status,
            adapter_status: row.adapter_status,
            parser_status: row.parser_status,
            corpus_family_id: compatibility_family.to_string(),
            fixture_id: fixture_id.to_string(),
            sample_id: evidence.sample_id,
            input_contract: evidence.input_contract,
            benchmark_limits: evidence.benchmark_limits,
            required_assets: evidence.required_assets,
            expected_outputs: evidence.expected_outputs,
            skip_behavior: evidence.skip_behavior,
            reason: format!(
                "row `{}` / `{}` is {} and maps to `{}` via fixture `{}`: {}",
                stage_id.as_str(),
                tool_id.as_str(),
                benchmark_status_label(row.benchmark_ready),
                compatibility_family,
                fixture_id,
                domain_assignment.rationale()
            ),
        });
    }

    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then_with(|| left.stage_id.cmp(&right.stage_id))
    });
    ensure_row_completeness(&rows)?;
    ensure_bam_fixture_assignment_coverage(&rows)?;
    Ok((stage_count, tool_count, rows))
}
pub(in crate::bam::assignment) fn benchmark_status_label(status: bool) -> &'static str {
    if status {
        "benchmark_ready"
    } else {
        "not_benchmark_ready"
    }
}
