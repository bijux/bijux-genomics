use bijux_dna_pipelines::artifact_flow::{validate_artifact_flow, ArtifactFlowNode};

#[derive(Clone)]
struct Node {
    id: String,
    dependencies: Vec<String>,
    external: Vec<String>,
    upstream: Vec<String>,
    outputs: Vec<String>,
}

fn symbols(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn node(id: &str, dependencies: &[&str], upstream: &[&str], outputs: &[&str]) -> Node {
    Node {
        id: id.to_owned(),
        dependencies: symbols(dependencies),
        external: if upstream.is_empty() { symbols(&["source"]) } else { vec![] },
        upstream: symbols(upstream),
        outputs: symbols(outputs),
    }
}

fn validate(nodes: &[Node]) -> anyhow::Result<Vec<String>> {
    let inputs = nodes
        .iter()
        .map(|node| ArtifactFlowNode {
            node_id: &node.id,
            depends_on: &node.dependencies,
            external_inputs: &node.external,
            upstream_inputs: &node.upstream,
            outputs: &node.outputs,
        })
        .collect::<Vec<_>>();
    validate_artifact_flow("alignment", &inputs)
}

fn chain() -> Vec<Node> {
    vec![node("reads", &[], &[], &["r1", "r2"]), node("align", &["reads"], &["r1", "r2"], &["bam"])]
}

#[test]
fn order_respects_producers_and_preserves_ready_declaration_order() {
    let nodes = vec![
        node("merge", &["left", "right"], &["a", "b"], &["merged"]),
        node("right", &["reads"], &["r2"], &["b"]),
        node("reference", &[], &[], &["reference"]),
        node("reads", &[], &[], &["r1", "r2"]),
        node("left", &["reads"], &["r1"], &["a"]),
    ];
    assert_eq!(
        validate(&nodes).expect("valid fork and join"),
        ["reference", "reads", "right", "left", "merge"]
    );
}

#[test]
fn multiple_inputs_from_one_producer_require_one_dependency() {
    assert_eq!(validate(&chain()).expect("paired input"), ["reads", "align"]);
}

#[test]
fn cyclic_artifact_flow_is_rejected_even_with_an_independent_root() {
    let nodes = vec![
        node("root", &[], &[], &["source"]),
        node("a", &["b"], &["b-out"], &["a-out"]),
        node("b", &["a"], &["a-out"], &["b-out"]),
    ];
    assert_eq!(
        validate(&nodes).expect_err("cycle").to_string(),
        "local pipeline DAG `alignment` contains a cycle"
    );
}

#[test]
fn missing_upstream_producer_is_rejected() {
    let mut nodes = chain();
    nodes[1].upstream.push("missing".into());
    assert!(validate(&nodes)
        .expect_err("missing producer")
        .to_string()
        .contains("that no node produces"));
}

#[test]
fn consuming_own_output_is_rejected() {
    let mut nodes = chain();
    nodes[1].upstream.push("bam".into());
    assert!(validate(&nodes)
        .expect_err("self consumption")
        .to_string()
        .contains("cannot consume its own output"));
}

#[test]
fn unknown_dependency_is_rejected() {
    let mut nodes = chain();
    nodes[1].dependencies.push("unknown".into());
    assert!(validate(&nodes)
        .expect_err("unknown node")
        .to_string()
        .contains("depends on unknown node `unknown`"));
}

#[test]
fn dependencies_must_exactly_cover_artifact_producers() {
    let mut nodes = chain();
    nodes[1].dependencies.clear();
    assert!(validate(&nodes)
        .expect_err("missing edge")
        .to_string()
        .contains("but upstream inputs require"));
    nodes[1].dependencies = symbols(&["reads", "reference"]);
    nodes.push(node("reference", &[], &[], &["reference"]));
    assert!(validate(&nodes)
        .expect_err("unjustified edge")
        .to_string()
        .contains("but upstream inputs require"));
}

#[test]
fn node_identity_is_unique_and_nonempty() {
    let mut nodes = chain();
    nodes[1].id = "reads".into();
    assert!(validate(&nodes).expect_err("duplicate node").to_string().contains("repeats node_id"));
    nodes[1].id = " \t".into();
    assert!(validate(&nodes).expect_err("blank node").to_string().contains("non-empty `node_id`"));
}

#[test]
fn each_output_has_exactly_one_producer() {
    let mut nodes = chain();
    nodes[1].outputs.push("r1".into());
    assert!(validate(&nodes)
        .expect_err("duplicate producer")
        .to_string()
        .contains("output `r1` is produced more than once"));
}

#[test]
fn every_symbol_list_rejects_blanks_and_duplicates() {
    for field in 0..4 {
        for values in [symbols(&[" "]), symbols(&["repeated", "repeated"])] {
            let mut nodes = chain();
            let selected = match field {
                0 => &mut nodes[1].dependencies,
                1 => &mut nodes[1].external,
                2 => &mut nodes[1].upstream,
                _ => &mut nodes[1].outputs,
            };
            *selected = values;
            let error = validate(&nodes).expect_err("invalid symbol list").to_string();
            assert!(error.contains("empty symbol") || error.contains("repeats symbol"), "{error}");
        }
    }
}

#[test]
fn nodes_require_inputs_and_outputs() {
    let mut nodes = chain();
    nodes[0].external.clear();
    assert!(validate(&nodes)
        .expect_err("missing input")
        .to_string()
        .contains("at least one external or upstream input"));
    nodes[0].external = symbols(&["source"]);
    nodes[0].outputs.clear();
    assert!(validate(&nodes)
        .expect_err("missing output")
        .to_string()
        .contains("at least one output"));
}
