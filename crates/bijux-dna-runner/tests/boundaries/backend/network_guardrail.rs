use std::path::{Path, PathBuf};

fn step_runner_root() -> PathBuf {
    let src_root = crate::support::crate_src("bijux-dna-runner")
        .unwrap_or_else(|err| panic!("resolve runner src: {err}"));
    let split_root = src_root.join("step_runner");
    if split_root.is_dir() {
        split_root
    } else {
        src_root
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn legacy_step_runner_path(root: &Path) -> PathBuf {
    root.join("step_runner.rs")
}

fn docker_network_policy_sources(root: &Path) -> Vec<PathBuf> {
    if root.ends_with("step_runner") {
        return vec![
            root.join("docker_execution.rs"),
            root.join("runtime_policy.rs"),
        ];
    }
    vec![legacy_step_runner_path(root)]
}

#[test]
fn runner_defaults_to_offline_network_mode() {
    let src_root = step_runner_root();
    let sources = docker_network_policy_sources(&src_root);
    let content = sources
        .iter()
        .map(|path| read(path))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        content.contains("--network") && content.contains("none"),
        "{} must enforce docker --network none by default",
        sources
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    assert!(
        content.contains("BIJUX_ALLOW_NETWORK"),
        "{} must support explicit BIJUX_ALLOW_NETWORK override",
        sources
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}
