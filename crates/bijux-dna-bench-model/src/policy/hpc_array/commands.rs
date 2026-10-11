//! Owner: persisted HPC command argument and shell projections.
/// Project the public command for one governed benchmark result.
#[must_use]
pub fn rendered_benchmark_result_execution_argv(result_id: &str) -> Vec<String> {
    vec![
        "bijux-dna".to_string(),
        "bench".to_string(),
        "local".to_string(),
        "execute-all-domain-benchmark-result".to_string(),
        "--result-id".to_string(),
        result_id.to_string(),
    ]
}

/// Project the public command for one governed pipeline node.
#[must_use]
pub fn rendered_essential_pipeline_node_execution_argv(
    pipeline_id: &str,
    node_id: &str,
) -> Vec<String> {
    vec![
        "bijux-dna".to_string(),
        "bench".to_string(),
        "local".to_string(),
        "execute-essential-pipeline-node".to_string(),
        "--pipeline-id".to_string(),
        pipeline_id.to_string(),
        "--node-id".to_string(),
        node_id.to_string(),
    ]
}

/// Serialize command arguments with the persisted command quotation rules.
#[must_use]
pub fn render_shell_command(argv: &[String]) -> String {
    argv.iter().map(|segment| shell_quote(segment)).collect::<Vec<_>>().join(" ")
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':' | '+'))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
