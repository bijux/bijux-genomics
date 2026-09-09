//! Symbolic artifact producer and dependency validation, independent of benchmark policy.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{anyhow, Result};

/// Borrowed graph input. Symbols identify declared artifacts, not their file formats.
/// Domain owners must separately validate scientific compatibility and stage readiness.
#[derive(Debug, Clone, Copy)]
pub struct ArtifactFlowNode<'a> {
    pub node_id: &'a str,
    pub depends_on: &'a [String],
    pub external_inputs: &'a [String],
    pub upstream_inputs: &'a [String],
    pub outputs: &'a [String],
}

/// Validate unique producers, exact dependencies and acyclicity.
///
/// Returns deterministic topological order, retaining declaration order for ready
/// nodes. An edge is required exactly when a node consumes its producer's output;
/// scheduling-only dependencies are rejected. External symbols are caller supplied
/// and do not prove acquisition, type compatibility or availability.
pub fn validate_artifact_flow(
    pipeline_id: &str,
    nodes: &[ArtifactFlowNode<'_>],
) -> Result<Vec<String>> {
    let mut seen_node_ids = BTreeSet::new();
    let mut seen_outputs = BTreeSet::new();
    let mut node_index = BTreeMap::<String, usize>::new();
    let mut output_producers = BTreeMap::<String, String>::new();
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    let mut indegree = BTreeMap::<String, usize>::new();

    for (index, node) in nodes.iter().enumerate() {
        if node.node_id.trim().is_empty() {
            return Err(anyhow!("local pipeline DAG nodes must declare a non-empty `node_id`"));
        }
        if !seen_node_ids.insert(node.node_id.to_owned()) {
            return Err(anyhow!("local pipeline DAG repeats node_id `{}`", node.node_id));
        }
        validate_symbol_list(
            node.external_inputs,
            &format!("node `{}` external_inputs", node.node_id),
        )?;
        validate_symbol_list(
            node.upstream_inputs,
            &format!("node `{}` upstream_inputs", node.node_id),
        )?;
        validate_symbol_list(node.outputs, &format!("node `{}` outputs", node.node_id))?;
        validate_symbol_list(node.depends_on, &format!("node `{}` depends_on", node.node_id))?;
        if node.external_inputs.is_empty() && node.upstream_inputs.is_empty() {
            return Err(anyhow!(
                "local pipeline DAG node `{}` must declare at least one external or upstream input",
                node.node_id
            ));
        }
        if node.outputs.is_empty() {
            return Err(anyhow!(
                "local pipeline DAG node `{}` must declare at least one output",
                node.node_id
            ));
        }
        for output in node.outputs {
            if !seen_outputs.insert(output.clone()) {
                return Err(anyhow!(
                    "local pipeline DAG output `{output}` is produced more than once"
                ));
            }
            output_producers.insert(output.clone(), node.node_id.to_owned());
        }
        node_index.insert(node.node_id.to_owned(), index);
        indegree.insert(node.node_id.to_owned(), node.depends_on.len());
        adjacency.entry(node.node_id.to_owned()).or_default();
    }

    for node in nodes {
        let mut required_dependencies = BTreeSet::new();
        for dependency in node.depends_on {
            if !node_index.contains_key(dependency) {
                return Err(anyhow!(
                    "local pipeline DAG node `{}` depends on unknown node `{dependency}`",
                    node.node_id
                ));
            }
            adjacency.entry(dependency.clone()).or_default().push(node.node_id.to_owned());
        }
        for upstream_input in node.upstream_inputs {
            let Some(producer) = output_producers.get(upstream_input) else {
                return Err(anyhow!(
                    "local pipeline DAG node `{}` consumes upstream input `{upstream_input}` that no node produces",
                    node.node_id
                ));
            };
            if producer == node.node_id {
                return Err(anyhow!(
                    "local pipeline DAG node `{}` cannot consume its own output `{upstream_input}` as an upstream input",
                    node.node_id
                ));
            }
            required_dependencies.insert(producer.clone());
        }
        let declared_dependencies = node.depends_on.iter().cloned().collect::<BTreeSet<_>>();
        if declared_dependencies != required_dependencies {
            return Err(anyhow!(
                "local pipeline DAG node `{}` declares dependencies {:?} but upstream inputs require {:?}",
                node.node_id,
                node.depends_on,
                required_dependencies.into_iter().collect::<Vec<_>>()
            ));
        }
    }

    let mut ready = VecDeque::new();
    for node in nodes {
        if indegree.get(node.node_id).copied().unwrap_or_default() == 0 {
            ready.push_back(node.node_id.to_owned());
        }
    }
    let mut topological_order = Vec::with_capacity(nodes.len());
    while let Some(node_id) = ready.pop_front() {
        topological_order.push(node_id.clone());
        for successor in adjacency.get(&node_id).into_iter().flatten() {
            let Some(entry) = indegree.get_mut(successor) else {
                continue;
            };
            *entry -= 1;
            if *entry == 0 {
                ready.push_back(successor.clone());
            }
        }
    }
    if topological_order.len() != nodes.len() {
        return Err(anyhow!("local pipeline DAG `{}` contains a cycle", pipeline_id));
    }

    Ok(topological_order)
}

fn validate_symbol_list(values: &[String], label: &str) -> Result<()> {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            return Err(anyhow!("{label} contains an empty symbol"));
        }
        if !seen.insert(value.clone()) {
            return Err(anyhow!("{label} repeats symbol `{value}`"));
        }
    }
    Ok(())
}
