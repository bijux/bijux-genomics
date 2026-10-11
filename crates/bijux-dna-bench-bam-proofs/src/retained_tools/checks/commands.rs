use super::super::acquisition::EvidenceSet;
use super::super::bindings::ensure_binding_subset;
use super::super::inventory::BindingInventory;
use anyhow::{bail, Result};
pub(in crate::retained_tools) fn check_commands(
    evidence: &EvidenceSet,
    bindings: &BindingInventory,
) -> Result<String> {
    if evidence.rendered_commands.row_count != bindings.expected_retained_row_count
        || evidence.rendered_commands.stage_count != bindings.expected_retained_stage_count
        || evidence.rendered_commands.tool_count != bindings.expected_retained_tool_count
        || !evidence.rendered_commands.bash_syntax_passed
    {
        bail!(
            "BAM rendered commands drifted: rows={}, stages={}, tools={}, bash_syntax_passed={}",
            evidence.rendered_commands.row_count,
            evidence.rendered_commands.stage_count,
            evidence.rendered_commands.tool_count,
            evidence.rendered_commands.bash_syntax_passed
        );
    }
    ensure_binding_subset(
        "bam retained command coverage",
        "command coverage",
        &bindings.retained_bindings,
        &bindings.command_bindings,
    )?;
    ensure_binding_subset(
        "bam retained rendered commands",
        "rendered command coverage",
        &bindings.retained_bindings,
        &bindings.rendered_command_bindings,
    )?;
    Ok("validated one bash-syntax-checked rendered command bundle for every retained BAM binding"
        .to_string())
}
