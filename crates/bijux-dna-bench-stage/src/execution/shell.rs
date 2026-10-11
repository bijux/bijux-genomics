/// Portable command persisted in stage manifests.
#[must_use]
pub fn rendered_stage_materialize_argv(stage_id: &str) -> Vec<String> {
    vec![
        "cargo".to_string(),
        "run".to_string(),
        "-q".to_string(),
        "-p".to_string(),
        "bijux-dna".to_string(),
        "--features".to_string(),
        "bam_downstream".to_string(),
        "--".to_string(),
        "bench".to_string(),
        "local".to_string(),
        "materialize-stage".to_string(),
        "--stage-id".to_string(),
        stage_id.to_string(),
    ]
}

pub(super) fn render_shell_command(argv: &[String]) -> String {
    argv.iter()
        .map(|arg| {
            if arg
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':'))
            {
                arg.clone()
            } else {
                let escaped = arg.replace('\'', "'\"'\"'");
                format!("'{escaped}'")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
