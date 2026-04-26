use std::path::Path;

use super::inputs::container_input_mapping;

pub(super) fn rewrite_container_path(
    value: &str,
    host_root: &Path,
    container_root: &str,
) -> String {
    let host_root = host_root.display().to_string();
    let mut rewritten = String::with_capacity(value.len());
    let mut cursor = 0;

    for (index, _) in value.match_indices(&host_root) {
        if index < cursor {
            continue;
        }
        if !path_match_has_shell_boundaries(value, index, host_root.len()) {
            continue;
        }
        rewritten.push_str(&value[cursor..index]);
        rewritten.push_str(container_root);
        cursor = index + host_root.len();
    }
    rewritten.push_str(&value[cursor..]);
    rewritten
}

fn path_match_has_shell_boundaries(value: &str, start: usize, len: usize) -> bool {
    let before = value[..start].chars().next_back();
    let after = value[start + len..].chars().next();

    path_start_boundary(before) && path_end_boundary(after)
}

fn path_start_boundary(value: Option<char>) -> bool {
    value.is_none_or(|value| {
        value.is_ascii_whitespace()
            || matches!(
                value,
                '\'' | '"' | '=' | ':' | ',' | '(' | '[' | '{' | '|' | '&' | '<' | '>'
            )
    })
}

fn path_end_boundary(value: Option<char>) -> bool {
    value.is_none_or(|value| {
        value.is_ascii_whitespace()
            || matches!(
                value,
                '/' | '\'' | '"' | ':' | ',' | ')' | ']' | '}' | '|' | '&' | '<' | '>' | ';'
            )
    })
}

pub(super) fn container_command_template(
    template: &[String],
    input_root: &Path,
    out_dir: &Path,
    preserve_absolute_inputs: bool,
) -> Vec<String> {
    let (_, container_input_root) = container_input_mapping(input_root);
    template
        .iter()
        .map(|part| {
            let rewritten_output = rewrite_container_path(part, out_dir, "/data/output");
            if preserve_absolute_inputs {
                rewritten_output
            } else {
                rewrite_container_path(&rewritten_output, input_root, &container_input_root)
            }
        })
        .collect()
}
