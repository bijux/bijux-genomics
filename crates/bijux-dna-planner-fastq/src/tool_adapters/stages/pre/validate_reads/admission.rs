use super::{plan_with_options, ValidateReadsPlanOptions, STAGE_ID, STAGE_VERSION};
use anyhow::{ensure, Context, Result};
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_domain_fastq::params::validate::ValidateEffectiveParams;
use bijux_dna_stage_contract::StagePlanV1;

/// Admit an observed argv against a typed validation plan and its resolved tool binding.
///
/// Reconstructs the canonical projection from declared inputs and effective parameters. Exact
/// argument boundaries and shell payload bytes must agree; unrelated text is not evidence.
/// This validates planning consistency, not execution or scientific output equivalence.
///
/// # Errors
/// Rejects unsupported identities, parameters, artifact layouts or command projections.
pub fn validate_command_projection(
    tool: &ToolExecutionSpecV1,
    plan: &StagePlanV1,
    observed_argv: &[String],
) -> Result<()> {
    ensure!(
        plan.stage_id == STAGE_ID && plan.stage_version == STAGE_VERSION,
        "validation command has a foreign stage identity or version"
    );
    ensure!(
        plan.tool_id == tool.tool_id && plan.tool_version == tool.tool_version,
        "validation command tool identity or version mismatch"
    );
    let params: ValidateEffectiveParams = serde_json::from_value(plan.effective_params.clone())
        .context("decode validation command effective parameters")?;
    let left = plan.io.inputs.first().context("validation command lacks a read input")?;
    ensure!(
        !left.path.as_os_str().is_empty() && !plan.out_dir.as_os_str().is_empty(),
        "validation command requires nonempty input and output addresses"
    );
    let right = plan.io.inputs.get(1).map(|input| input.path.as_path());
    ensure!(
        right.is_none_or(|path| !path.as_os_str().is_empty()),
        "validation command has an empty right input address"
    );
    let expected = plan_with_options(
        tool,
        &left.path,
        right,
        &plan.out_dir,
        &ValidateReadsPlanOptions {
            threads: Some(params.threads),
            validation_mode: params.validation_mode,
            pair_sync_policy: params.pair_sync_policy,
        },
    )?;
    // Compare the governed execution surface. Stage-instance and attribution metadata may be
    // attached by composition without changing this tool's input/output/command projection.
    for (name, actual, canonical) in [
        (
            "effective parameters",
            serde_json::to_value(&plan.effective_params)?,
            serde_json::to_value(&expected.effective_params)?,
        ),
        (
            "parameter projection",
            serde_json::to_value(&plan.params)?,
            serde_json::to_value(&expected.params)?,
        ),
        (
            "input/output contract",
            serde_json::to_value(&plan.io)?,
            serde_json::to_value(&expected.io)?,
        ),
        ("tool image", serde_json::to_value(&plan.image)?, serde_json::to_value(&expected.image)?),
        (
            "resources",
            serde_json::to_value(&plan.resources)?,
            serde_json::to_value(&expected.resources)?,
        ),
        (
            "operating mode",
            serde_json::to_value(plan.operating_mode)?,
            serde_json::to_value(expected.operating_mode)?,
        ),
        (
            "auxiliary images",
            serde_json::to_value(&plan.aux_images)?,
            serde_json::to_value(&expected.aux_images)?,
        ),
    ] {
        ensure!(
            actual == canonical,
            "validation command {name} contradicts the canonical projection"
        );
    }
    ensure!(
        plan.command.template == expected.command.template
            && observed_argv == expected.command.template,
        "validation command argv contradicts the canonical projection"
    );
    Ok(())
}
