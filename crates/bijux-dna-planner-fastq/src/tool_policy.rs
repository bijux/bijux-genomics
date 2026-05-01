use anyhow::{anyhow, Result};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq::FastqReadLayout;

pub(crate) fn enforce_stage_tool(stage_id: &str, tool_id: &ToolId) -> Result<()> {
    let stage_id = StageId::new(stage_id.to_string());
    let allowed_tools = crate::selection::allowed_tools_for_stage(&stage_id);
    if allowed_tools.is_empty() {
        return Err(anyhow!(
            "{} has no admitted execution tools in the FASTQ domain registry",
            stage_id.as_str()
        ));
    }
    if allowed_tools.iter().any(|allowed| allowed == tool_id) {
        return Ok(());
    }
    Err(anyhow!(
        "{} is not admitted for {}; allowed tools: {}",
        tool_id.as_str(),
        stage_id.as_str(),
        allowed_tools
            .iter()
            .map(bijux_dna_core::contract::ToolId::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

pub(crate) fn enforce_input_layout(
    stage_id: &str,
    tool_id: &ToolId,
    paired_end: bool,
) -> Result<()> {
    let declared = bijux_dna_domain_fastq::declared_input_layouts_for_stage(stage_id)
        .ok_or_else(|| anyhow!("missing declared FASTQ layout policy for {stage_id}"))?;
    if !declared.accepted_layouts.is_empty() {
        let layout =
            if paired_end { FastqReadLayout::PairedEnd } else { FastqReadLayout::SingleEnd };
        enforce_declared_input_layout(stage_id, layout)?;
    }
    if !paired_end {
        let paired_required = matches!(
            (stage_id, tool_id.as_str()),
            ("fastq.merge_pairs" | "fastq.extract_umis", _)
                | ("fastq.remove_duplicates", "fastuniq")
        );
        if paired_required {
            return Err(anyhow!(
                "{} does not support single-end inputs for {}",
                tool_id.as_str(),
                stage_id
            ));
        }
    }
    Ok(())
}

pub(crate) fn enforce_declared_input_layout(stage_id: &str, layout: FastqReadLayout) -> Result<()> {
    if bijux_dna_domain_fastq::stage_accepts_input_layout(stage_id, layout) {
        return Ok(());
    }
    Err(anyhow!(
        "{stage_id} does not admit {layout:?} inputs under the governed FASTQ layout contract"
    ))
}

#[cfg(test)]
mod tests {
    use super::{enforce_declared_input_layout, enforce_input_layout};
    use bijux_dna_core::ids::ToolId;
    use bijux_dna_domain_fastq::FastqReadLayout;

    #[test]
    fn planner_routes_single_and_paired_layouts_through_declared_contracts() {
        assert!(
            enforce_declared_input_layout("fastq.trim_reads", FastqReadLayout::SingleEnd).is_ok()
        );
        assert!(
            enforce_declared_input_layout("fastq.trim_reads", FastqReadLayout::PairedEnd).is_ok()
        );
        assert!(enforce_declared_input_layout("fastq.report_qc", FastqReadLayout::Merged).is_ok());
    }

    #[test]
    fn planner_refuses_undeclared_layouts() {
        for layout in [
            FastqReadLayout::Interleaved,
            FastqReadLayout::Deinterleaved,
            FastqReadLayout::Singleton,
            FastqReadLayout::Rejected,
        ] {
            assert!(enforce_declared_input_layout("fastq.trim_reads", layout).is_err());
        }
        assert!(enforce_declared_input_layout("fastq.extract_umis", FastqReadLayout::SingleEnd)
            .is_err());
    }

    #[test]
    fn planner_keeps_tool_level_single_end_refusals_for_paired_only_backends() {
        assert!(enforce_input_layout(
            "fastq.remove_duplicates",
            &ToolId::from_static("fastuniq"),
            false
        )
        .is_err());
    }
}
