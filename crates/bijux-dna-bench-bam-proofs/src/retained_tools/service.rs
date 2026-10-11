use super::{
    acquisition::acquire,
    checks::{
        check_commands, check_consistency, check_expected_results, check_micro, check_outputs,
        check_parsers, check_report_map, check_runtime, check_science,
    },
    declarations::BAM_STAGE_COMPLETION_GOALS,
    inventory::inventory,
    models::BamAllRetainedToolsCompleteReport,
    projection::project_report,
    recording::record_goal_check,
    stages::validate_stage_completion_goal,
    RetainedContext,
};
use anyhow::Result;
use std::path::Path;
pub(super) fn assess(
    context: &RetainedContext<'_>,
    absolute_output_path: &Path,
) -> Result<BamAllRetainedToolsCompleteReport> {
    let evidence = acquire(context)?;
    let bindings = inventory(&evidence);
    let mut checks = Vec::new();

    for goal in BAM_STAGE_COMPLETION_GOALS {
        record_goal_check(
            &mut checks,
            goal.goal_id,
            goal.stage_id.to_string(),
            Some(goal.output_path.to_string()),
            || validate_stage_completion_goal(context, *goal),
        );
    }

    record_goal_check(
        &mut checks,
        441,
        "bam local and container smoke coverage",
        Some(context.paths.local_smoke.to_string()),
        || check_runtime(&evidence, &bindings),
    );

    record_goal_check(
        &mut checks,
        442,
        "bam parser fixture coverage",
        Some(context.paths.parser_coverage.to_string()),
        || check_parsers(&evidence, &bindings),
    );

    record_goal_check(
        &mut checks,
        443,
        "bam rendered commands",
        Some(context.paths.rendered_commands.to_string()),
        || check_commands(&evidence, &bindings),
    );

    record_goal_check(
        &mut checks,
        444,
        "bam output declarations",
        Some(context.paths.output_declarations.to_string()),
        || check_outputs(&evidence, &bindings),
    );

    record_goal_check(
        &mut checks,
        445,
        "bam expected results",
        Some(context.paths.expected_results.to_string()),
        || check_expected_results(&bindings),
    );

    record_goal_check(
        &mut checks,
        446,
        "bam report map",
        Some(context.paths.report_map.to_string()),
        || check_report_map(&evidence, &bindings),
    );

    record_goal_check(
        &mut checks,
        447,
        "bam active row consistency",
        Some(context.paths.active_row_consistency.to_string()),
        || check_consistency(&evidence, &bindings),
    );

    record_goal_check(
        &mut checks,
        448,
        "bam real micro smoke subset",
        Some(context.paths.micro_smoke_summary.to_string()),
        || check_micro(&evidence),
    );

    record_goal_check(
        &mut checks,
        449,
        "bam scientific thresholds",
        Some(context.paths.science_thresholds.to_string()),
        || check_science(&evidence),
    );

    Ok(project_report(context.repo_root, absolute_output_path, &evidence, &bindings, checks))
}
