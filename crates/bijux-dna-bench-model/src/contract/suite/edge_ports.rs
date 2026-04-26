//! Owner: bijux-dna-bench-model
//! Edge and port validation for benchmark suite graphs.

use crate::diagnostics::BenchError;
use crate::model::{BenchmarkGraphNode, BenchmarkStageEdge};
use bijux_dna_domain_fastq::{stage_input_ids, stage_output_ids};

pub(crate) fn validate_edge_ports(
    edge: &BenchmarkStageEdge,
    declared_graph_nodes: &std::collections::BTreeMap<String, BenchmarkGraphNode>,
) -> Result<(), BenchError> {
    match (&edge.from_output_id, &edge.to_input_id) {
        (Some(from_output_id), Some(to_input_id)) => {
            validate_stage_output_port(&edge.from, from_output_id, declared_graph_nodes)?;
            validate_stage_input_port(&edge.to, to_input_id, declared_graph_nodes)?;
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(BenchError::InvalidPolicy(format!(
            "suite edge {} -> {} must set from_output_id and to_input_id together",
            edge.from, edge.to
        ))),
    }
}

fn validate_stage_output_port(
    node_id: &str,
    output_id: &str,
    declared_graph_nodes: &std::collections::BTreeMap<String, BenchmarkGraphNode>,
) -> Result<(), BenchError> {
    let Some(node) = declared_graph_nodes.get(node_id) else {
        return Ok(());
    };
    let Some(output_ids) = stage_output_ids(&node.stage_id) else {
        return Ok(());
    };
    if output_ids.contains(output_id) {
        return Ok(());
    }
    Err(BenchError::InvalidPolicy(format!(
        "suite edge source node {} does not expose output {} in the governed {} contract",
        node_id, output_id, node.stage_id
    )))
}

fn validate_stage_input_port(
    node_id: &str,
    input_id: &str,
    declared_graph_nodes: &std::collections::BTreeMap<String, BenchmarkGraphNode>,
) -> Result<(), BenchError> {
    let Some(node) = declared_graph_nodes.get(node_id) else {
        return Ok(());
    };
    let Some(input_ids) = stage_input_ids(&node.stage_id) else {
        return Ok(());
    };
    if input_ids.contains(input_id) {
        return Ok(());
    }
    Err(BenchError::InvalidPolicy(format!(
        "suite edge target node {} does not accept input {} in the governed {} contract",
        node_id, input_id, node.stage_id
    )))
}
