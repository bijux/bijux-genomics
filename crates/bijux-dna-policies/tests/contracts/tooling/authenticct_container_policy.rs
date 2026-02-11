#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__authenticct_container_policy__authenticct_container_uses_valid_entrypoint_script_path(
) {
    let path = support::workspace_root()
        .join("containers")
        .join("docker")
        .join("arm64")
        .join("Dockerfile.authenticct");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    let required_markers = [
        "/opt/authenticct/AuthentiCT.py",
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
}
