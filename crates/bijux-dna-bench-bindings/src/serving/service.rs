use super::{
    admission::ensure_fastq_amplicon_fixture_coverage,
    corpus::load_corpus_status_by_stage,
    labels::{adapter_status_label, bam_parser_status_label, parser_status_label},
    ToolServingMapRow,
};
use crate::{observations::Domain, ServingContext};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
/// Canonical inventory cardinalities paired with the admitted rows.
#[derive(Debug, Clone)]
pub struct ServingRows {
    pub stage_count: usize,
    pub tool_count: usize,
    pub rows: Vec<ToolServingMapRow>,
}
/// Collect canonical stage/tool declarations with explicit request-local inputs.
///
/// # Errors
/// Rejects missing or ambiguous declarations and governed row drift.
pub fn collect(context: &ServingContext<'_>, domain: Domain) -> Result<ServingRows> {
    let corpus = load_corpus_status_by_stage(context)?;
    let integrated = context.evidence.integrated_tools(context.repo_root)?;
    let inventory = context.evidence.stage_inventory(context.repo_root, domain)?;
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for stage_id in &inventory.stages {
        // BAM stage parsing precedes corpus lookup, preserving canonical error precedence.
        let bam = if domain == Domain::Bam {
            Some(context.evidence.bam_stage(context.repo_root, stage_id)?)
        } else {
            None
        };
        let corpus_status = corpus.get(stage_id).ok_or_else(|| {
            anyhow!(
                "{} local corpus compatibility report is missing stage `{stage_id}`",
                domain.label()
            )
        })?;
        let tools = match &bam {
            Some(stage) => stage.tools.clone(),
            None => context.evidence.fastq_tools(context.repo_root, stage_id)?,
        };
        for tool_id in tools {
            if !integrated.contains(&tool_id) {
                continue;
            }
            let (support, adapter, parser) = if let Some(stage) = &bam {
                let observed =
                    context.evidence.bam_assessment(context.repo_root, stage_id, &tool_id)?;
                (observed.support, observed.adapter, bam_parser_status_label(stage))
            } else {
                let observed =
                    context.evidence.fastq_assessment(context.repo_root, stage_id, &tool_id)?;
                if !observed.active {
                    continue;
                }
                (
                    observed.readiness,
                    adapter_status_label(&observed),
                    parser_status_label(&observed),
                )
            };
            if !seen.insert((stage_id.clone(), tool_id.clone())) {
                return Err(anyhow!(
                    "duplicate {} serving binding `{stage_id}` / `{tool_id}`",
                    domain.label()
                ));
            }
            rows.push(ToolServingMapRow {
                tool_id,
                stage_id: stage_id.clone(),
                support_status: support.to_string(),
                adapter_status: adapter.to_string(),
                parser_status: parser.to_string(),
                corpus_status: corpus_status.clone(),
            });
        }
    }
    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then_with(|| left.stage_id.cmp(&right.stage_id))
    });
    if domain == Domain::Fastq {
        ensure_fastq_amplicon_fixture_coverage(&rows)?;
    }
    let tool_count = rows.iter().map(|row| &row.tool_id).collect::<BTreeSet<_>>().len();
    Ok(ServingRows { stage_count: inventory.stage_count, tool_count, rows })
}
