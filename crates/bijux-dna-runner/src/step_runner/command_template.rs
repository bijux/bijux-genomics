use std::path::Path;

use super::inputs::container_input_mapping;

pub(super) fn rewrite_container_path(
    value: &str,
    host_root: &Path,
    container_root: &str,
) -> String {
    let host_root_path = host_root;
    let host_root = host_root_path.display().to_string();
    let rewritten = value.replace(&host_root, container_root);
    if value == host_root {
        return container_root.to_string();
    }
    if host_root_path.is_file() {
        return rewritten;
    }
    let host_prefix = format!("{host_root}/");
    let container_prefix = format!("{container_root}/");
    rewritten.replace(&host_prefix, &container_prefix)
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
