//! Owner: bijux-dna-bench
//! Stage and tool governance checks for benchmark suites.

use crate::error::BenchError;
use bijux_dna_core::{id_catalog, ids};
use bijux_dna_domain_fastq::{
    admitted_execution_tools_for_stage, contract_for_stage, execution_support_for_stage,
    stage_tool_binding,
};
use bijux_dna_stage_contract::has_executor;

pub(crate) fn ensure_supported_stage(stage_id: &str) -> Result<(), BenchError> {
    if planner_owned_graph_stage(stage_id) {
        return Ok(());
    }
    if stage_id.starts_with(id_catalog::FASTQ_PREFIX) {
        if contract_for_stage(stage_id).is_none() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {stage_id} is not declared in the FASTQ domain catalog"
            )));
        }
        let stage_id = ids::parse_stage_id(stage_id).map_err(|err| {
            BenchError::InvalidPolicy(format!("suite stage {stage_id} is not a valid id: {err}"))
        })?;
        let support = execution_support_for_stage(&stage_id).ok_or_else(|| {
            BenchError::InvalidPolicy(format!(
                "suite stage {} is missing FASTQ execution support",
                stage_id.as_str()
            ))
        })?;
        if !support.is_plannable() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} is not plannable under FASTQ execution support",
                stage_id.as_str()
            )));
        }
        return Ok(());
    }
    if !has_executor(stage_id) {
        return Err(BenchError::InvalidPolicy(format!(
            "suite stage {stage_id} is not registered in the stage executor catalog"
        )));
    }
    Ok(())
}

pub(crate) fn validate_stage_tools(stage_id: &str, tools: &[String]) -> Result<(), BenchError> {
    if planner_owned_graph_stage(stage_id) {
        if tools.is_empty() {
            return Ok(());
        }
        return Err(BenchError::InvalidPolicy(format!(
            "suite planner-owned stage {stage_id} must not declare tool bindings"
        )));
    }
    if !stage_id.starts_with(id_catalog::FASTQ_PREFIX) {
        return Ok(());
    }
    let stage_id = ids::parse_stage_id(stage_id).map_err(|err| {
        BenchError::InvalidPolicy(format!("suite stage {stage_id} is not a valid id: {err}"))
    })?;
    let admitted_tools = admitted_execution_tools_for_stage(&stage_id);
    for tool in tools {
        let tool_id = ids::parse_tool_id(tool).map_err(|err| {
            BenchError::InvalidPolicy(format!(
                "suite stage {} tool {tool} is not a valid tool id: {err}",
                stage_id.as_str()
            ))
        })?;
        if stage_tool_binding(&stage_id, &tool_id).is_none() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} tool {tool} is not declared in the FASTQ stage-tool matrix",
                stage_id.as_str()
            )));
        }
        if !admitted_tools.iter().any(|admitted| admitted == &tool_id) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} tool {tool} is not admitted by FASTQ execution support",
                stage_id.as_str()
            )));
        }
    }
    Ok(())
}

pub(crate) fn planner_owned_graph_stage(stage_id: &str) -> bool {
    matches!(
        stage_id,
        "benchmark.compare_stage_tools" | "benchmark.select_stage_tool"
    )
}
