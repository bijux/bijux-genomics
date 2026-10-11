#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__authenticct_container_policy__authenticct_container_uses_valid_entrypoint_script_path(
) {
    let root = support::workspace_root();
    let path = root.join("containers").join("docker").join("Dockerfile.arm64.authenticct");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let required_markers = [
        "COPY containers/tools/authenticct/authenticct /usr/local/bin/authenticct",
        "/opt/authenticct-venv/bin/pip install",
        "/usr/local/bin/authenticct",
        "ENTRYPOINT [\"/usr/local/bin/authenticct\"]",
    ];
    for marker in required_markers {
        assert!(
            raw.contains(marker),
            "AuthentiCT container missing marker `{marker}` in {}",
            path.display()
        );
    }

    let wrapper_path = root.join("containers/tools/authenticct/authenticct");
    let wrapper = std::fs::read_to_string(&wrapper_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", wrapper_path.display()));
    assert!(
        wrapper.contains("exec /opt/authenticct-venv/bin/AuthentiCT"),
        "AuthentiCT wrapper must invoke the locked virtual-environment entrypoint in {}",
        wrapper_path.display()
    );
}
