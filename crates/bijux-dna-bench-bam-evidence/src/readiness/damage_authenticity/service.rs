use super::declarations::expected_stage_ids;
use super::declarations::expected_tool_ids_by_stage;
use super::declarations::required_local_smoke_fields_by_stage;
use super::declarations::required_output_ids_by_stage;
use super::declarations::BAM_BINDING_SPECS;
use super::declarations::BAM_DAMAGE_AUTHENTICITY_READY_SCHEMA_VERSION;
use super::declarations::CHECKED_SURFACE_COUNT;
use super::declarations::COVERAGE_STATUS_COMPLETE;
use super::models::BamDamageAuthenticityReadyReport;
use super::projection::build_bam_damage_authenticity_ready_row;
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
pub(in crate::readiness) fn build_bam_damage_authenticity_ready_report(
    context: &ReadinessContext<'_>,
    output_path: &Path,
) -> Result<BamDamageAuthenticityReadyReport> {
    let repo_root = context.repo_root;
    let published = PublishedSources::acquire(context)?;
    let schema_contracts =
        collect_bam_stage_schema_contracts(&context.declarations.schema()?, expected_stage_ids())?;
    let local_smoke_by_stage = collect_local_smoke_proofs(context)?;

    let scope = RetainedScope::new(
        "BAM damage/authenticity",
        BAM_BINDING_SPECS.iter().map(|spec| (spec.stage_id, spec.tool_id)),
    );
    let indexed = IndexedSources::admit(&scope, published, schema_contracts)?;

    let mut rows = Vec::with_capacity(BAM_BINDING_SPECS.len());
    for spec in BAM_BINDING_SPECS {
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
            anyhow!("missing BAM damage/authenticity local-smoke proof for `{}`", spec.stage_id)
        })?;

        rows.push(build_bam_damage_authenticity_ready_row(
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

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });

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

    Ok(BamDamageAuthenticityReadyReport {
        schema_version: BAM_DAMAGE_AUTHENTICITY_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        stage_count: expected_stage_ids().len(),
        expected_tool_ids_by_stage: expected_tool_ids_by_stage(),
        required_output_ids_by_stage: required_output_ids_by_stage(),
        required_local_smoke_fields_by_stage: required_local_smoke_fields_by_stage(),
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    })
}
