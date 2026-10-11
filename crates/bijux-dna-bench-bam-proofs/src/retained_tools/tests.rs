#![cfg(test)]
use super::{
    bindings::{ensure_binding_subset, BamBindingKey},
    declarations::BamStageCompletionGoal,
    recording::record_goal_check,
    stages::validate_stage_completion_counts,
};
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
fn key(stage: &str, tool: &str) -> BamBindingKey {
    BamBindingKey { stage_id: stage.into(), tool_id: tool.into() }
}
#[test]
fn binding_subset_accepts_other_stages_but_rejects_substituted_tools() -> Result<()> {
    let expected = BTreeSet::from([key("bam.damage", "mapdamage")]);
    let mut actual = expected.clone();
    actual.insert(key("bam.sex", "ry_compute"));
    ensure_binding_subset("retained", "parsers", &expected, &actual)?;
    actual.insert(key("bam.damage", "foreign-tool"));
    let error = ensure_binding_subset("retained", "parsers", &expected, &actual)
        .err()
        .ok_or_else(|| anyhow!("foreign tool admitted"))?;
    assert!(error.to_string().contains("extra=[\"bam.damage/foreign-tool\"]"));
    Ok(())
}
#[test]
fn binding_subset_rejects_missing_stage_identity() -> Result<()> {
    let expected = BTreeSet::from([key("bam.damage", "mapdamage")]);
    let error = ensure_binding_subset("retained", "outputs", &expected, &BTreeSet::new())
        .err()
        .ok_or_else(|| anyhow!("missing binding admitted"))?;
    assert!(error.to_string().contains("missing=[\"bam.damage/mapdamage\"]"));
    Ok(())
}
#[test]
fn stage_counts_require_complete_rows_and_exact_surface_cardinality() {
    let goal = BamStageCompletionGoal {
        goal_id: 431,
        stage_id: "bam.overlap_correction",
        output_path: "proof.json",
        expected_active_row_count: 1,
        expected_checked_surface_count: 11,
    };
    assert!(validate_stage_completion_counts(goal, 1, 1, 11).is_ok());
    assert!(validate_stage_completion_counts(goal, 0, 1, 11).is_err());
    assert!(validate_stage_completion_counts(goal, 1, 2, 11).is_err());
    assert!(validate_stage_completion_counts(goal, 1, 1, 10).is_err());
}
#[test]
fn failed_stage_diagnostics_do_not_skip_later_checks() {
    let mut checks = Vec::new();
    record_goal_check(&mut checks, 431, "first", Some("first.json".into()), || {
        Err(anyhow!("scientific disagreement"))
    });
    record_goal_check(&mut checks, 432, "second", Some("second.json".into()), || {
        Ok("complete".into())
    });
    assert_eq!(checks.len(), 2);
    assert_eq!(checks[0].goal_id, 431);
    assert!(!checks[0].ok);
    assert_eq!(checks[0].detail, "scientific disagreement");
    assert_eq!(checks[1].goal_id, 432);
    assert!(checks[1].ok);
}
