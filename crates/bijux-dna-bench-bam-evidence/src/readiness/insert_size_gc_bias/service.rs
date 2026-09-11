use super::declarations::BAM_INSERT_SIZE_GC_BIAS_READY_SCHEMA_VERSION;
use super::declarations::BAM_STAGE_SPECS;
use super::declarations::CHECKED_SURFACE_COUNT;
use super::declarations::COVERAGE_STATUS_COMPLETE;
use super::models::BamInsertSizeGcBiasReadyReport;
use super::projection::build_bam_insert_size_gc_bias_ready_row;
use super::projection::RowInput;
use super::proofs::collect_local_smoke_proofs;
use crate::readiness::paths::path_relative_to_repo;
use crate::readiness::schema::collect_bam_stage_schema_contracts;
use crate::readiness::surfaces::{
    BindingEvidence, IndexedSources, PublishedSources, RetainedScope,
};
use crate::readiness::ReadinessContext;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
pub(in crate::readiness) fn build_bam_insert_size_gc_bias_ready_report(
    context: &ReadinessContext<'_>,
    output_path: &Path,
) -> Result<BamInsertSizeGcBiasReadyReport> {
    let repo_root = context.repo_root;
    let published = PublishedSources::acquire(context)?;
    let schema_contracts = collect_bam_stage_schema_contracts(
        &context.declarations.schema()?,
        BAM_STAGE_SPECS.iter().map(|spec| spec.stage_id),
    )?;
    let local_smoke_by_stage = collect_local_smoke_proofs(context)?;

    let scope = RetainedScope::new(
        "BAM insert-size/gc-bias",
        BAM_STAGE_SPECS.iter().map(|spec| (spec.stage_id, spec.tool_id)),
    );
    let indexed = IndexedSources::admit(&scope, published, schema_contracts)?;

    let mut rows = Vec::with_capacity(BAM_STAGE_SPECS.len());
    for spec in BAM_STAGE_SPECS {
        let BindingEvidence {
            active_row,
            command_row,
            output_row,
            parser_row,
            expected_row,
            report_map_row,
            schema_contract,
        } = indexed.binding(scope.label, spec.stage_id, spec.tool_id)?;
        let local_smoke = local_smoke_by_stage.get(spec.stage_id).ok_or_else(|| {
            anyhow!("missing BAM insert-size/gc-bias local-smoke proof for `{}`", spec.stage_id)
        })?;

        rows.push(build_bam_insert_size_gc_bias_ready_row(
            context,
            &RowInput {
                active_row,
                command_row,
                output_row,
                parser_row,
                expected_row,
                report_map_row,
                schema_contract,
                local_smoke,
                spec,
            },
        ));
    }

    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));

    let complete_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COMPLETE).count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COMPLETE)
        .cloned()
        .collect::<Vec<_>>();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }

    Ok(BamInsertSizeGcBiasReadyReport {
        schema_version: BAM_INSERT_SIZE_GC_BIAS_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        stage_count: BAM_STAGE_SPECS.len(),
        expected_tool_ids_by_stage: BAM_STAGE_SPECS
            .iter()
            .map(|spec| (spec.stage_id.to_string(), vec![spec.tool_id.to_string()]))
            .collect(),
        required_output_ids_by_stage: BAM_STAGE_SPECS
            .iter()
            .map(|spec| {
                (
                    spec.stage_id.to_string(),
                    spec.required_output_ids.iter().map(|value| (*value).to_string()).collect(),
                )
            })
            .collect(),
        required_local_smoke_fields_by_stage: BAM_STAGE_SPECS
            .iter()
            .map(|spec| {
                (
                    spec.stage_id.to_string(),
                    spec.required_local_smoke_fields
                        .iter()
                        .map(|value| (*value).to_string())
                        .collect(),
                )
            })
            .collect(),
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    })
}
