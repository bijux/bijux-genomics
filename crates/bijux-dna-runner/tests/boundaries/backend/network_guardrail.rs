use std::path::{Path, PathBuf};

fn execute_rs_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("execute.rs")
}

#[test]
fn runner_defaults_to_offline_network_mode() {
    let path = execute_rs_path();
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));

    assert!(
        content.contains("--network") && content.contains("none"),
        "execute.rs must enforce docker --network none by default"
    );
    assert!(
        content.contains("BIJUX_ALLOW_NETWORK"),
        "execute.rs must support explicit BIJUX_ALLOW_NETWORK override"
    );
}
