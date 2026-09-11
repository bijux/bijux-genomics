#![cfg(test)]

use super::*;
use anyhow::{Context, Result};
use bijux_dna_core::id_catalog::BAM_DAMAGE;

fn serving(stage: &str, tool: &str) -> ToolServingMapRow {
    ToolServingMapRow {
        stage_id: stage.to_string(),
        tool_id: tool.to_string(),
        support_status: "active".to_string(),
        adapter_status: "complete".to_string(),
        parser_status: "complete".to_string(),
        corpus_status: "fixture".to_string(),
    }
}

#[test]
fn retained_scope_filters_unrelated_bindings_but_rejects_missing_authority() -> Result<()> {
    let scope = RetainedScope::new("damage", [(BAM_DAMAGE, "native")]);
    let rows = scope
        .select(vec![serving("unrelated", "other"), serving(BAM_DAMAGE, "native")], "active")?;
    assert_eq!(rows.len(), 1);
    let error = scope
        .select(vec![serving(BAM_DAMAGE, "wrong")], "active")
        .err()
        .context("missing binding must fail")?;
    assert_eq!(
        error.to_string(),
        format!("damage active rows must cover [{BAM_DAMAGE} / native], found []")
    );
    Ok(())
}

#[test]
fn retained_duplicate_bindings_cannot_overwrite_observations() -> Result<()> {
    let scope = RetainedScope::new("damage", [(BAM_DAMAGE, "native")]);
    let rows = scope
        .select(vec![serving(BAM_DAMAGE, "native"), serving(BAM_DAMAGE, "native")], "active")?;
    let error = collect_rows_by_binding(scope.label, rows).err().context("duplicate must fail")?;
    assert_eq!(error.to_string(), format!("duplicate damage binding `{BAM_DAMAGE}` / `native`"));
    Ok(())
}

#[test]
fn schema_stage_admission_rejects_additional_or_missing_stages() -> Result<()> {
    let scope = RetainedScope::new("damage", [(BAM_DAMAGE, "native")]);
    scope.stages("schema", [BAM_DAMAGE.to_string()].into_iter())?;
    for observed in [vec![], vec![BAM_DAMAGE.to_string(), "unrelated".to_string()]] {
        let error = scope
            .stages("schema", observed.into_iter())
            .err()
            .context("schema scope mismatch must fail")?;
        assert!(error.to_string().starts_with("damage schema rows must cover ["));
    }
    Ok(())
}

#[test]
fn report_binding_identity_retains_distinct_tools_and_rejects_duplicates() -> Result<()> {
    let row = BamReportMapRow {
        stage_id: BAM_DAMAGE.to_string(),
        tool_id: "native".to_string(),
        result_row_id: "result".to_string(),
        corpus_family_id: "family".to_string(),
        fixture_id: "fixture".to_string(),
        sample_scope: "sample".to_string(),
        canonical_stage_rank: 1,
        readiness_kind: "local".to_string(),
        criticality: "required".to_string(),
        support_status: "active".to_string(),
        anchor_tool_id: "native".to_string(),
        anchor_support_status: "active".to_string(),
        report_section_id: "damage".to_string(),
        report_section_title: "Damage".to_string(),
        summary_table_id: "summary".to_string(),
        summary_table_title: "Summary".to_string(),
        workflow_branch_id: None,
        scientific_context_required: vec![],
        report_focus: "damage".to_string(),
        reason: "retained".to_string(),
    };
    assert_eq!(collect_rows_by_binding("damage report-map", vec![row.clone()])?.len(), 1);
    let mut alternate = row.clone();
    alternate.tool_id = "alternate".to_string();
    assert_eq!(
        collect_rows_by_binding("damage report-map", vec![row.clone(), alternate])?.len(),
        2
    );
    let mut conflict = row.clone();
    conflict.report_section_id = "conflicting".to_string();
    let error = collect_rows_by_binding("damage report-map", vec![row, conflict])
        .err()
        .context("duplicate binding must fail")?;
    assert_eq!(
        error.to_string(),
        format!("duplicate damage report-map binding `{BAM_DAMAGE}` / `native`")
    );
    Ok(())
}
