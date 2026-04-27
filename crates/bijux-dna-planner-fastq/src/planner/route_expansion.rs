#![allow(clippy::uninlined_format_args, clippy::wildcard_imports)]

use super::*;

#[derive(Debug, Clone)]
pub struct StageToolSelection {
    pub stage_id: String,
    pub stage_instance_id: Option<String>,
    pub tool_id: String,
    pub reason: PlanDecisionReason,
}

#[derive(Debug, Clone)]
pub struct ToolsetSelection {
    pub stage_id: String,
    pub stage_instance_id: Option<String>,
    pub tool_ids: Vec<String>,
    pub reason: PlanDecisionReason,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
struct RouteContext(std::collections::BTreeMap<String, String>);

#[derive(Debug, Clone)]
struct ExpandedRouteNode {
    expanded_node_id: String,
    input_context: RouteContext,
    output_context: RouteContext,
}

/// # Errors
/// Returns an error if toolset selection fails.
pub fn select_preprocess_toolsets(
    pipeline: &PipelineSpec,
    mode: crate::stage_api::ToolsetExecutionMode,
    allow_planned: bool,
) -> Result<Vec<ToolsetSelection>> {
    let mut selections = Vec::new();
    for node in pipeline.ordered_nodes() {
        if planner_owned_graph_stage(&node.stage_id) {
            continue;
        }
        enforce_stage_status(&node.stage_id, allow_planned)?;
        let stage_id = StageId::new(node.stage_id.clone());
        let tool_ids = crate::stage_api::toolset_for_stage(&stage_id, mode)
            .into_iter()
            .map(|tool_id| tool_id.to_string())
            .collect::<Vec<_>>();
        selections.push(ToolsetSelection {
            stage_id: node.stage_id,
            stage_instance_id: node.stage_instance_id,
            tool_ids,
            reason: PlanDecisionReason::new(
                PlanReasonKind::Default,
                match mode {
                    crate::stage_api::ToolsetExecutionMode::DefaultChoice => {
                        "selected default toolset"
                    }
                    crate::stage_api::ToolsetExecutionMode::GovernedExecution => {
                        "selected governed execution toolset"
                    }
                    crate::stage_api::ToolsetExecutionMode::BenchmarkCohort => {
                        "selected benchmark cohort toolset"
                    }
                    crate::stage_api::ToolsetExecutionMode::AllBindings => {
                        "selected declared binding toolset"
                    }
                },
            ),
        });
    }
    Ok(selections)
}

/// # Errors
/// Returns an error when toolset selections do not align with the pipeline or
/// when route expansion would create invalid or excessive route-specific graphs.
pub fn expand_pipeline_stage_tool_routes(
    pipeline: &PipelineSpec,
    toolsets: &[ToolsetSelection],
) -> Result<(PipelineSpec, Vec<StageToolSelection>)> {
    let ordered_nodes = pipeline.ordered_nodes();
    let executable_nodes = ordered_nodes
        .iter()
        .filter(|node| !planner_owned_graph_stage(&node.stage_id))
        .collect::<Vec<_>>();
    if executable_nodes.len() != toolsets.len() {
        return Err(anyhow!(
            "pipeline node/toolset length mismatch: {} vs {}",
            executable_nodes.len(),
            toolsets.len()
        ));
    }
    for (node, toolset) in executable_nodes.iter().zip(toolsets.iter()) {
        if node.stage_id != toolset.stage_id || node.stage_instance_id != toolset.stage_instance_id
        {
            return Err(anyhow!(
                "toolset expansion requires node-aligned stage selections; got pipeline node {} and toolset {}",
                PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref()),
                PipelineSpec::stage_node_id(&toolset.stage_id, toolset.stage_instance_id.as_deref()),
            ));
        }
        if toolset.tool_ids.is_empty() {
            return Err(anyhow!(
                "toolset expansion requires at least one tool for {}",
                node.stage_id
            ));
        }
    }
    let toolset_by_node_id = toolsets
        .iter()
        .map(|toolset| {
            (
                PipelineSpec::stage_node_id(
                    &toolset.stage_id,
                    toolset.stage_instance_id.as_deref(),
                ),
                toolset,
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    let route_count = toolsets.iter().try_fold(1usize, |count, toolset| {
        count
            .checked_mul(toolset.tool_ids.len())
            .ok_or_else(|| anyhow!("preprocess tool route expansion overflowed route count"))
    })?;
    let max_route_specific_pipelines = max_route_specific_pipelines()?;
    if route_count > max_route_specific_pipelines {
        return Err(anyhow!(
            "preprocess tool route expansion would create {route_count} route-specific pipelines; configured limit is {max_route_specific_pipelines}. Narrow the stage toolsets or raise BIJUX_FASTQ_MAX_ROUTE_PIPELINES"
        ));
    }

    let base_edges = if pipeline.declares_graph_topology() {
        pipeline.edges.clone()
    } else {
        ordered_nodes
            .windows(2)
            .map(|window| PipelineEdgeSpec {
                from: PipelineSpec::stage_node_id(
                    &window[0].stage_id,
                    window[0].stage_instance_id.as_deref(),
                ),
                to: PipelineSpec::stage_node_id(
                    &window[1].stage_id,
                    window[1].stage_instance_id.as_deref(),
                ),
                from_output_id: None,
                to_input_id: None,
            })
            .collect::<Vec<_>>()
    };

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut selections = Vec::new();
    let mut expanded_nodes_by_original =
        std::collections::BTreeMap::<String, Vec<ExpandedRouteNode>>::new();
    let predecessor_sets = predecessor_context_sets(&base_edges);
    for node in ordered_nodes {
        let node_id =
            PipelineSpec::stage_node_id(&node.stage_id, node.stage_instance_id.as_deref());
        let input_contexts =
            incoming_route_contexts(&node_id, &predecessor_sets, &expanded_nodes_by_original)?;
        if planner_owned_graph_stage(&node.stage_id) {
            let collapsed_source_nodes = collapsed_source_nodes_for_select(&node_id, &base_edges)?;
            for input_context in input_contexts {
                let output_context = input_context.without(&collapsed_source_nodes);
                let expanded_node_id = expanded_planner_stage_instance_id(
                    &node.stage_id,
                    node.stage_instance_id.as_deref(),
                    &output_context.route_key(),
                );
                nodes.push(PipelineNodeSpec {
                    stage_id: node.stage_id.clone(),
                    stage_instance_id: Some(expanded_node_id.clone()),
                });
                expanded_nodes_by_original
                    .entry(node_id.clone())
                    .or_default()
                    .push(ExpandedRouteNode { expanded_node_id, input_context, output_context });
            }
            continue;
        }

        let toolset = toolset_by_node_id
            .get(&node_id)
            .copied()
            .ok_or_else(|| anyhow!("toolset expansion requires a stage toolset for {}", node_id))?;
        for input_context in input_contexts {
            for tool_id in &toolset.tool_ids {
                let output_context =
                    input_context.with_assignment(node_id.clone(), tool_id.clone());
                let stage_instance_id = expanded_stage_instance_id(
                    &node.stage_id,
                    node.stage_instance_id.as_deref(),
                    tool_id,
                    &output_context.route_key(),
                );
                nodes.push(PipelineNodeSpec {
                    stage_id: node.stage_id.clone(),
                    stage_instance_id: Some(stage_instance_id.clone()),
                });
                selections.push(StageToolSelection {
                    stage_id: node.stage_id.clone(),
                    stage_instance_id: Some(stage_instance_id.clone()),
                    tool_id: tool_id.clone(),
                    reason: toolset.reason.clone(),
                });
                expanded_nodes_by_original.entry(node_id.clone()).or_default().push(
                    ExpandedRouteNode {
                        expanded_node_id: stage_instance_id,
                        input_context: input_context.clone(),
                        output_context,
                    },
                );
            }
        }
    }

    for edge in &base_edges {
        let from_nodes = expanded_nodes_by_original
            .get(&edge.from)
            .ok_or_else(|| anyhow!("expanded route missing source node {}", edge.from))?;
        let to_nodes = expanded_nodes_by_original
            .get(&edge.to)
            .ok_or_else(|| anyhow!("expanded route missing target node {}", edge.to))?;
        for from_node in from_nodes {
            for to_node in to_nodes {
                if !from_node.output_context.is_subset_of(&to_node.input_context) {
                    continue;
                }
                edges.push(PipelineEdgeSpec {
                    from: from_node.expanded_node_id.clone(),
                    to: to_node.expanded_node_id.clone(),
                    from_output_id: edge.from_output_id.clone(),
                    to_input_id: expanded_to_input_id(edge, from_node),
                });
            }
        }
    }

    nodes.sort_by(|left, right| {
        PipelineSpec::stage_node_id(&left.stage_id, left.stage_instance_id.as_deref())
            .cmp(&PipelineSpec::stage_node_id(&right.stage_id, right.stage_instance_id.as_deref()))
    });
    nodes.dedup_by(|left, right| {
        left.stage_id == right.stage_id && left.stage_instance_id == right.stage_instance_id
    });
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.from_output_id.cmp(&right.from_output_id))
            .then_with(|| left.to_input_id.cmp(&right.to_input_id))
    });
    edges.dedup_by(|left, right| {
        left.from == right.from
            && left.to == right.to
            && left.from_output_id == right.from_output_id
            && left.to_input_id == right.to_input_id
    });
    selections.sort_by(|left, right| {
        left.stage_instance_id
            .cmp(&right.stage_instance_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    selections.dedup_by(|left, right| {
        left.stage_instance_id == right.stage_instance_id && left.tool_id == right.tool_id
    });

    Ok((PipelineSpec::graph(nodes, edges), selections))
}

fn max_route_specific_pipelines() -> Result<usize> {
    let Some(raw) = std::env::var_os("BIJUX_FASTQ_MAX_ROUTE_PIPELINES") else {
        return Ok(DEFAULT_MAX_ROUTE_SPECIFIC_PIPELINES);
    };
    let parsed = raw.to_string_lossy().parse::<usize>().map_err(|error| {
        anyhow!("BIJUX_FASTQ_MAX_ROUTE_PIPELINES must be a positive integer: {error}")
    })?;
    if parsed == 0 {
        return Err(anyhow!("BIJUX_FASTQ_MAX_ROUTE_PIPELINES must be greater than zero"));
    }
    Ok(parsed)
}

fn predecessor_context_sets(
    edges: &[PipelineEdgeSpec],
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut predecessors = std::collections::BTreeMap::<String, Vec<String>>::new();
    for edge in edges {
        predecessors.entry(edge.to.clone()).or_default().push(edge.from.clone());
    }
    for upstream_nodes in predecessors.values_mut() {
        upstream_nodes.sort();
        upstream_nodes.dedup();
    }
    predecessors
}

fn incoming_route_contexts(
    node_id: &str,
    predecessor_sets: &std::collections::BTreeMap<String, Vec<String>>,
    expanded_nodes_by_original: &std::collections::BTreeMap<String, Vec<ExpandedRouteNode>>,
) -> Result<Vec<RouteContext>> {
    let Some(predecessors) = predecessor_sets.get(node_id) else {
        return Ok(vec![RouteContext::default()]);
    };
    let predecessor_context_sets = predecessors
        .iter()
        .map(|predecessor| {
            expanded_nodes_by_original
                .get(predecessor)
                .ok_or_else(|| anyhow!("expanded route missing predecessor node {}", predecessor))
                .map(|nodes| {
                    nodes.iter().map(|node| node.output_context.clone()).collect::<Vec<_>>()
                })
        })
        .collect::<Result<Vec<_>>>()?;
    combine_route_context_sets(&predecessor_context_sets)
}

fn combine_route_context_sets(context_sets: &[Vec<RouteContext>]) -> Result<Vec<RouteContext>> {
    let mut combined = vec![RouteContext::default()];
    for contexts in context_sets {
        let mut next = Vec::new();
        for prior in &combined {
            for context in contexts {
                if let Some(merged) = prior.merge(context) {
                    next.push(merged);
                }
            }
        }
        if next.is_empty() {
            return Err(anyhow!(
                "toolset route expansion found incompatible branch contexts while rejoining graph inputs"
            ));
        }
        next.sort();
        next.dedup();
        combined = next;
    }
    Ok(combined)
}

fn collapsed_source_nodes_for_select(
    select_node_id: &str,
    edges: &[PipelineEdgeSpec],
) -> Result<std::collections::BTreeSet<String>> {
    let incoming_sources = edges
        .iter()
        .filter(|edge| edge.to == select_node_id)
        .map(|edge| edge.from.clone())
        .collect::<std::collections::BTreeSet<_>>();
    if incoming_sources.is_empty() {
        return Err(anyhow!("selection node {} requires incoming candidate edges", select_node_id));
    }
    if incoming_sources.len() > 1 {
        return Err(anyhow!(
            "toolset route expansion requires selection node {} to collapse one source stage node; use explicit stage_bindings for multi-source selection joins",
            select_node_id
        ));
    }
    Ok(incoming_sources)
}

fn expanded_planner_stage_instance_id(
    stage_id: &str,
    stage_instance_id: Option<&str>,
    route_key: &str,
) -> String {
    let base_node_id = stage_instance_id.unwrap_or(stage_id);
    if route_key.is_empty() {
        base_node_id.to_string()
    } else {
        format!("{base_node_id}.route.{route_key}")
    }
}

fn expanded_to_input_id(edge: &PipelineEdgeSpec, from_node: &ExpandedRouteNode) -> Option<String> {
    let base_input_id = edge.to_input_id.clone()?;
    if !edge.to.starts_with(crate::STAGE_SELECT_STAGE_TOOL.as_str()) {
        return Some(base_input_id);
    }
    let tool_id = from_node
        .output_context
        .0
        .get(&edge.from)
        .cloned()
        .unwrap_or_else(|| "candidate".to_string());
    Some(format!("{tool_id}_{base_input_id}"))
}

fn expanded_stage_instance_id(
    stage_id: &str,
    stage_instance_id: Option<&str>,
    tool_id: &str,
    route_key: &str,
) -> String {
    let base_node_id = stage_instance_id.unwrap_or(stage_id);
    format!("{base_node_id}.route.{route_key}.tool.{tool_id}")
}

impl RouteContext {
    fn with_assignment(&self, node_id: String, tool_id: String) -> Self {
        let mut assignments = self.0.clone();
        assignments.insert(node_id, tool_id);
        Self(assignments)
    }

    fn without(&self, node_ids: &std::collections::BTreeSet<String>) -> Self {
        let mut assignments = self.0.clone();
        for node_id in node_ids {
            assignments.remove(node_id);
        }
        Self(assignments)
    }

    fn merge(&self, other: &Self) -> Option<Self> {
        let mut assignments = self.0.clone();
        for (node_id, tool_id) in &other.0 {
            match assignments.get(node_id) {
                Some(existing) if existing != tool_id => return None,
                Some(_) => {}
                None => {
                    assignments.insert(node_id.clone(), tool_id.clone());
                }
            }
        }
        Some(Self(assignments))
    }

    fn route_key(&self) -> String {
        self.0
            .iter()
            .map(|(node_id, tool_id)| format!("{node_id}={tool_id}"))
            .collect::<Vec<_>>()
            .join("__")
    }

    fn is_subset_of(&self, other: &Self) -> bool {
        self.0.iter().all(|(node_id, tool_id)| other.0.get(node_id) == Some(tool_id))
    }
}
