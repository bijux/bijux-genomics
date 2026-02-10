use bijux_dna_core::ids::{id_catalog, StageId, ToolId};
use bijux_dna_domain_fastq::{
    STAGE_CORRECT, STAGE_DETECT_ADAPTERS, STAGE_FILTER, STAGE_MERGE, STAGE_PREPROCESS,
    STAGE_QC_POST, STAGE_SCREEN, STAGE_STATS_NEUTRAL, STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};
use bijux_dna_pipelines::STAGE_CORE_PREPARE_REFERENCE;
use std::collections::BTreeSet;

#[must_use]
pub fn allowed_tools_for_stage(stage_id: &StageId) -> Vec<String> {
    let Some(adapter_id) = adapter_id_for_stage(stage_id) else {
        return Vec::new();
    };
    let mut tools = crate::selection::tool_registry::tool_registry()
        .into_values()
        .filter(|entry| entry.adapter_id == adapter_id)
        .map(|entry| entry.tool_id.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    tools.sort();
    tools
}

#[must_use]
pub fn default_tool_for_stage(stage_id: &StageId) -> Option<ToolId> {
    if stage_id == &STAGE_PREPROCESS {
        Some(ToolId::from_static(id_catalog::TOOL_PLANNER))
    } else if stage_id.as_str() == STAGE_CORE_PREPARE_REFERENCE {
        Some(ToolId::from_static(id_catalog::TOOL_SAMTOOLS))
    } else if stage_id == &STAGE_VALIDATE_PRE {
        Some(ToolId::from_static(id_catalog::TOOL_FASTQVALIDATOR_OFFICIAL))
    } else if stage_id == &STAGE_DETECT_ADAPTERS {
        Some(ToolId::from_static(id_catalog::TOOL_FASTQC))
    } else if stage_id == &STAGE_TRIM {
        Some(ToolId::from_static(id_catalog::TOOL_FASTP))
    } else if stage_id == &STAGE_FILTER {
        Some(ToolId::from_static(id_catalog::TOOL_SEQKIT))
    } else if stage_id == &STAGE_STATS_NEUTRAL {
        Some(ToolId::from_static(id_catalog::TOOL_SEQKIT_STATS))
    } else if stage_id == &STAGE_QC_POST {
        Some(ToolId::from_static(id_catalog::TOOL_MULTIQC))
    } else if stage_id == &STAGE_MERGE {
        Some(ToolId::from_static(id_catalog::TOOL_VSEARCH))
    } else if stage_id == &STAGE_CORRECT {
        Some(ToolId::from_static(id_catalog::TOOL_RCORRECTOR))
    } else if stage_id == &STAGE_UMI {
        Some(ToolId::from_static(id_catalog::TOOL_UMI_TOOLS))
    } else if stage_id == &STAGE_SCREEN {
        Some(ToolId::from_static(id_catalog::TOOL_KRAKEN2))
    } else {
        None
    }
}

#[must_use]
fn adapter_id_for_stage(stage_id: &StageId) -> Option<&'static str> {
    if stage_id == &STAGE_PREPROCESS {
        Some(id_catalog::FASTQ_PREPROCESS)
    } else if stage_id.as_str() == STAGE_CORE_PREPARE_REFERENCE {
        Some(id_catalog::CORE_PREPARE_REFERENCE)
    } else if stage_id == &STAGE_VALIDATE_PRE || stage_id == &STAGE_DETECT_ADAPTERS {
        Some(id_catalog::FASTQ_VALIDATE_PRE)
    } else if stage_id == &STAGE_TRIM {
        Some(id_catalog::FASTQ_TRIM)
    } else if stage_id == &STAGE_FILTER {
        Some(id_catalog::FASTQ_FILTER)
    } else if stage_id == &STAGE_STATS_NEUTRAL {
        Some(id_catalog::FASTQ_STATS_NEUTRAL)
    } else if stage_id == &STAGE_QC_POST {
        Some(id_catalog::FASTQ_QC_POST)
    } else if stage_id == &STAGE_MERGE {
        Some(id_catalog::FASTQ_MERGE)
    } else if stage_id == &STAGE_CORRECT {
        Some(id_catalog::FASTQ_CORRECT)
    } else if stage_id == &STAGE_UMI {
        Some(id_catalog::FASTQ_UMI)
    } else if stage_id == &STAGE_SCREEN {
        Some(id_catalog::FASTQ_SCREEN)
    } else {
        None
    }
}
