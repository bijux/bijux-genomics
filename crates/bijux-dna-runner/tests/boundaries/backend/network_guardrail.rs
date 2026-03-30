use std::path::PathBuf;

fn step_runner_path() -> PathBuf {
    let src_root = crate::support::crate_src("bijux-dna-runner")
        .unwrap_or_else(|err| panic!("resolve runner src: {err}"));
    let split_module = src_root.join("step_runner").join("mod.rs");
    if split_module.is_file() {
        split_module
    } else {
        src_root.join("step_runner.rs")
    }
}

#[test]
fn runner_defaults_to_offline_network_mode() {
    let path = step_runner_path();
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));

    assert!(
        content.contains("--network") && content.contains("none"),
        "{} must enforce docker --network none by default",
        path.display()
    );
    assert!(
        content.contains("BIJUX_ALLOW_NETWORK"),
        "{} must support explicit BIJUX_ALLOW_NETWORK override",
        path.display()
    );
}
