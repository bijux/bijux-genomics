use anyhow::{Context, Result};
use bijux_dna_core::prelude::{StageId, ToolExecutionSpecV1, ToolId};
use bijux_dna_domain_fastq::{
    params::validate::{PairSyncPolicy, ValidationMode},
    STAGE_VALIDATE_READS,
};
use bijux_dna_planner_fastq::tool_adapters::fastq::validate_reads::{
    plan_with_options, validate_command_projection, ValidateReadsPlanOptions,
};
use std::path::{Path, PathBuf};
fn root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}
fn tool(id: &str) -> Result<ToolExecutionSpecV1> {
    bijux_dna_planner_fastq::stage_api::load_fastq_domain_tool_execution_spec(
        &root()?,
        &STAGE_VALIDATE_READS,
        &ToolId::new(id.to_string()),
    )
}
#[test]
fn retained_tools_admit_both_layouts_and_validation_exit_policies() -> Result<()> {
    for id in ["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"] {
        let tool = tool(id)?;
        for mode in [ValidationMode::Strict, ValidationMode::ReportOnly] {
            for (right, policy) in [
                (None, PairSyncPolicy::NotApplicable),
                (Some(Path::new("reads 'R2'.fastq")), PairSyncPolicy::RequireHeaderSync),
                (Some(Path::new("reads 'R2'.fastq")), PairSyncPolicy::SkipHeaderSync),
            ] {
                let plan = plan_with_options(
                    &tool,
                    Path::new("reads 'R1'.fastq"),
                    right,
                    Path::new("output with spaces"),
                    &ValidateReadsPlanOptions {
                        threads: Some(2),
                        validation_mode: mode.clone(),
                        pair_sync_policy: policy,
                    },
                )?;
                validate_command_projection(&tool, &plan, &plan.command.template)?;
            }
        }
    }
    Ok(())
}
#[test]
fn convincing_text_and_changed_argument_boundaries_do_not_prove_a_command() -> Result<()> {
    let tool = tool("seqtk")?;
    let plan = plan_with_options(
        &tool,
        Path::new("R1.fastq"),
        None,
        Path::new("result"),
        &ValidateReadsPlanOptions {
            pair_sync_policy: PairSyncPolicy::NotApplicable,
            ..Default::default()
        },
    )?;
    let forged = vec!["printf".into(), "%s".into(), plan.command.template.join(" ")];
    assert!(validate_command_projection(&tool, &plan, &forged).is_err());
    let mut forged = plan.command.template.clone();
    forged.push("ignored argument".into());
    assert!(validate_command_projection(&tool, &plan, &forged).is_err());
    let mut changed = plan.clone();
    changed.command.template[0] = "echo".into();
    assert!(validate_command_projection(&tool, &changed, &changed.command.template).is_err());
    Ok(())
}
#[test]
fn contradictory_plan_metadata_and_artifacts_are_rejected() -> Result<()> {
    let tool = tool("seqtk")?;
    let plan = plan_with_options(
        &tool,
        Path::new("R1.fastq"),
        None,
        Path::new("result"),
        &ValidateReadsPlanOptions {
            pair_sync_policy: PairSyncPolicy::NotApplicable,
            ..Default::default()
        },
    )?;
    let mut changed = plan.clone();
    changed.stage_id = StageId::new("foreign");
    assert!(validate_command_projection(&tool, &changed, &plan.command.template).is_err());
    let mut changed = plan.clone();
    changed.tool_version.push_str("-foreign");
    assert!(validate_command_projection(&tool, &changed, &plan.command.template).is_err());
    let mut changed = plan.clone();
    changed.io.outputs[0].path = PathBuf::from("foreign.json");
    assert!(validate_command_projection(&tool, &changed, &plan.command.template).is_err());
    let mut changed = plan.clone();
    changed.io.inputs.push(changed.io.inputs[0].clone());
    assert!(validate_command_projection(&tool, &changed, &plan.command.template).is_err());
    for (key, value) in [
        ("threads", serde_json::json!(0)),
        ("paired_mode", serde_json::json!("paired_end")),
        ("schema_version", serde_json::json!("future")),
    ] {
        let mut changed = plan.clone();
        changed.effective_params[key] = value;
        assert!(
            validate_command_projection(&tool, &changed, &plan.command.template).is_err(),
            "accepted {key}"
        );
    }
    Ok(())
}
