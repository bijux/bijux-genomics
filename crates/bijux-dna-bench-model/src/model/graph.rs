//! Owner: bijux-dna-bench-model
//! Graph nodes and edges for benchmark suite execution topology.
//! Must not perform IO or depend on compare/gate logic.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkGraphNodeKind {
    Stage,
    StageTool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkGraphNode {
    pub node_id: String,
    pub kind: BenchmarkGraphNodeKind,
    pub stage_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instance_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageEdge {
    pub from: String,
    pub to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_output_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_input_id: Option<String>,
}
